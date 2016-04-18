use std::collections::vec_deque::VecDeque;
use std::intrinsics::discriminant_value;
use std::cmp::min;

use indexer::lexer::{CommonLexer, Token, Span, WhitespaceType};
use indexer::storage::Context;

#[derive(Debug)]
pub enum ParserState {
    Wait,
    KeywordThenName,
    NameThenCall,
}

pub struct CommonParser {
    pub buffer: String,
    pub lexems: Vec<(Token, Span)>,
}

#[derive(Debug, Clone)]
pub enum Tagged {
    Definition(String),
    Calling(String),
    Whitespace(WhitespaceType),
    Comment,
    Keyword(Token),
}

#[derive(Debug)]
pub enum FuzzyRuleState {
    NotMatches,
    Cont(usize),
    Ready(Vec<(Tagged, Span)>),
}

pub trait FuzzyRule<'a> {
    fn match_tokens(&mut self, &VecDeque<(&'a Token, &'a Span)>) -> FuzzyRuleState;
}

pub trait FuzzyCallback<'a> {
    fn on_tokens(&mut self, &VecDeque<(&'a Token, &'a Span)>) -> Vec<(Tagged, Span)>;
}

pub struct FuzzyTokenRule<'a> {
    pub tokens: Vec<Token>,
    pub callback: Box<FuzzyCallback<'a>>,
}

impl<'a> FuzzyTokenRule<'a> {
    fn new(tokens: Vec<Token>, callback: Box<FuzzyCallback<'a>>) -> FuzzyTokenRule<'a> {
        FuzzyTokenRule {
            tokens: tokens,
            callback: callback,
        }
    }
}
impl<'a> FuzzyRule<'a> for FuzzyTokenRule<'a> {
    fn match_tokens(&mut self, tokens: &VecDeque<(&'a Token, &'a Span)>) -> FuzzyRuleState {
        let till = min(self.tokens.len(), tokens.len());
        let mut matched = 0;

        for i in 0..till {
            if token_eq(&self.tokens[i], &tokens[i].0) {
                matched += 1;
            } else {
                matched = 0;
                break;
            }
        }

        if matched == 0 {
            FuzzyRuleState::NotMatches
        } else if matched == self.tokens.len() {
            let res = self.callback.on_tokens(tokens);
            FuzzyRuleState::Ready(res)
        } else {
            FuzzyRuleState::Cont(self.tokens.len())
        }
    }
}

pub fn token_eq(a: &Token, b: &Token) -> bool {
    unsafe {
        let x = discriminant_value(a);
        let y = discriminant_value(b);
        //println!("A {}, {}, {:?} {:?}", x, y, a, b);
        return x == y;
    }
}

pub struct KwMatch {
    line_counter: usize,
}

impl KwMatch {
    fn new() -> KwMatch {
        KwMatch { line_counter: 1 }
    }
}

impl<'a> FuzzyRule<'a> for KwMatch {
    fn match_tokens(&mut self, tokens: &VecDeque<(&'a Token, &'a Span)>) -> FuzzyRuleState {
        use indexer::lexer::Token::*;
        //println!("Q: {:?}", tokens[0].0);

        match tokens[0].0 {
            &Fn | &Use | &Struct | &Pub | &Let | &Impl => FuzzyRuleState::Ready(
                vec![(Tagged::Keyword(tokens[0].0.clone()), tokens[0].1.clone())]
            ),
            &Comment => FuzzyRuleState::Ready(
                vec![(Tagged::Comment, tokens[0].1.clone())]
            ),
            &Whitespace(ref wh) => {
                //println!("WH: {:?} {:?}", tokens[0].0, wh);
                match wh {
                    &WhitespaceType::Newline(_) => {
                        //println!("Rz: {:?}", wh);
                        let state = FuzzyRuleState::Ready(
                            vec![(Tagged::Whitespace(WhitespaceType::Newline(self.line_counter)), tokens[0].1.clone())]
                        );
                        self.line_counter += 1;
                        state
                    },
                    &WhitespaceType::Spaces => {
                        //println!("Bz: {:?}", wh);
                        FuzzyRuleState::NotMatches
                    }
                }
            },

            _ => FuzzyRuleState::NotMatches,
        }
    }
}

fn match_tokens<'a>(rule_tokens: &[Token], tokens: &VecDeque<(&'a Token, &'a Span)>) -> FuzzyRuleState {
    let till = min(rule_tokens.len(), tokens.len());
    let mut matched = 0;

    for i in 0..till {
        if token_eq(&rule_tokens[i], &tokens[i].0) {
            matched += 1;
        } else {
            matched = 0;
            break;
        }
    }

    if matched == 0 {
        FuzzyRuleState::NotMatches
    } else {
        FuzzyRuleState::Cont(rule_tokens.len())
    }
}


fn match3<'a>(tokens: &VecDeque<(&'a Token, &'a Span)>) -> (Token, Token, Token) {
    let mut a = Token::NoToken;
    let mut b = Token::NoToken;
    let mut c = Token::NoToken;

    if tokens.len() >= 1 {
        a = tokens[0].0.clone();
        if tokens.len() >= 2 {
            b = tokens[1].0.clone();
            if tokens.len() >= 3 {
                c = tokens[2].0.clone();
            }
        }
    }

    (a, b, c)
}

struct FnMatch;

impl<'a> FuzzyRule<'a> for FnMatch {
    fn match_tokens(&mut self, tokens: &VecDeque<(&'a Token, &'a Span)>) -> FuzzyRuleState {
        use indexer::lexer::Token::*;
        //println!("Q: {:?}", tokens[0]);
        //Fn, Ident(String::new()), LParen
        let mut res = FuzzyRuleState::NotMatches;

        {
            let rr = vec![Fn, Ident(String::new()), LParen];
            let m = match_tokens(&rr, tokens);

            match m {
                FuzzyRuleState::Cont(len) if tokens.len() >= len => {
                    //println!("W: {:?} {:?} {} {}", tokens, rr, len, rr.len());

                    let mut name = String::new();
                    match tokens[1].0 {
                        &Token::Ident(ref n) => { name = n.clone(); },
                        _ => {},
                    }

                    res = FuzzyRuleState::Ready(vec![
                        (Tagged::Definition(name), tokens[1].1.clone()),
                    ]);
                },
                _ => { res = m },
            }
        }

        match res {
            FuzzyRuleState::NotMatches => {
                //println!("J: {:?}", tokens);
                let rr = vec![Ident(String::new()), LParen];
                let m = match_tokens(&rr, tokens);

                //println!("S: {:?}", m);
                match m {
                    FuzzyRuleState::Cont(len) if tokens.len() >= len => {
                        //println!("W: {:?}", tokens);

                        let mut name = String::new();
                        match tokens[0].0 {
                            &Token::Ident(ref n) => { name = n.clone(); },
                            _ => {},
                        }

                        res = FuzzyRuleState::Ready(vec![
                            (Tagged::Calling(name), tokens[0].1.clone()),
                        ]);
                    },
                    _ => { res = m },
                }
            }
            _ => {},
        }

        res
    }
}

pub struct FuzzyParser<'a> {
    pub rules: Vec<Box<FuzzyRule<'a>>>,
    pub size: usize,
    pub cache: VecDeque<(&'a Token, &'a Span)>,
    pub variants: Vec<usize>,
}

impl<'a> FuzzyParser<'a> {
    fn new(size: usize, rules: Vec<Box<FuzzyRule<'a>>>) -> FuzzyParser<'a> {
        FuzzyParser {
            rules: rules,
            size: size,
            cache: VecDeque::with_capacity(size),
            variants: Vec::with_capacity(size),
        }
    }

    fn push(&mut self, lex: (&'a Token, &'a Span)) -> Vec<(Tagged, Span)> {
        //println!("P: {:?}, {:?}", lex.0, lex.1);
        if self.cache.len() == self.size {
            self.cache.pop_front();
        }

        self.cache.push_back(lex);
        //println!("{:?}", self.cache);

        for rule in &mut self.rules {
            //let () = rule;
            let res = rule.match_tokens(&self.cache);
            //println!("R: {:?}", res);
            match res {
                FuzzyRuleState::NotMatches => {},
                FuzzyRuleState::Cont(usize) => {},
                FuzzyRuleState::Ready(e) => {
                    //println!("Matched! {:?}", self.cache);
                    for i in 0..e.len() {
                        self.cache.pop_front();
                    }
                    return e
                },
            }
        }

        vec![]
    }
}

pub trait Preprocessing<'a> {
    fn filter(&mut self, lex: (&'a Token, &'a Span)) -> Option<(&'a Token, &'a Span)>;
}

pub struct CPreprocessing;

impl<'a> Preprocessing<'a> for CPreprocessing {
    fn filter(&mut self, lex: (&'a Token, &'a Span)) -> Option<(&'a Token, &'a Span)> {
        let (token, _) = lex;

        match token {
            &Token::Whitespace(_) | &Token::Comment => None,
            _ => Some(lex),
        }
    }
}

impl CommonParser {
    pub fn new(buffer: String) -> CommonParser {
        CommonParser {
            buffer: buffer,
            lexems: vec![],
        }
    }

    pub fn parse(&mut self) -> Context {
        use indexer::lexer::Token::*;

        let mut lexer = CommonLexer::new(&self.buffer);

        self.lexems.push((Token::Whitespace(WhitespaceType::Newline(0)), Span{lo: 0, hi: 0}));

        for (tok, span) in lexer {
            //println!("L: {:?} {:?}", tok, span);
            match tok {
                Token::Eof => { self.lexems.push((tok, span)); break; }
                _ => { self.lexems.push((tok, span));},
            }
        }

        let mut preproc = CPreprocessing{};

        let kw_rule = Box::new(KwMatch::new());
        let fn_rule = Box::new(FnMatch{});

        let mut parser = FuzzyParser::new(3, vec![fn_rule]);
        let mut syntax_parser = FuzzyParser::new(1, vec![kw_rule]);

        let mut syntax_parser_out = vec![];
        let mut parser_out = vec![];

        for &(ref tok, ref span) in &self.lexems {
            let lsyn = syntax_parser.push((tok, span));
            if lsyn.len() != 0 {
                //println!("PR: {:?}", lsyn);
                syntax_parser_out.extend(lsyn);
            }

            if let Some((wtok, wspan)) = preproc.filter((tok, span)) {
                let pres = parser.push((wtok, wspan));
                if pres.len() != 0 {
                    //println!("PR: {:?}", res);
                    parser_out.extend(pres);
                }
            }
        }

        println!("SYN: {:?}", syntax_parser_out);
        println!("PRS: {:?}", parser_out);

        return Context::new(syntax_parser_out, parser_out);
    }
}
