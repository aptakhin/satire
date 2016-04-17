use std::collections::HashSet;
use std::collections::vec_deque::VecDeque;
use std::intrinsics::discriminant_value;
use std::cmp::max;

use indexer::lexer::{CommonLexer, Token, Span, WhitespaceType};
use indexer::storage::FileSource;
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

#[derive(Debug)]
pub enum Tagged {
    Definition(String),
    Calling(String),
    Whitespace(WhitespaceType),

    Keyword(String),
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
        let till = max(self.tokens.len(), tokens.len());
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

pub struct KwMatch;

impl<'a> FuzzyRule<'a> for KwMatch {
    fn match_tokens(&mut self, tokens: &VecDeque<(&'a Token, &'a Span)>) -> FuzzyRuleState {
        use indexer::lexer::Token::*;
        println!("Q: {:?}", tokens[0].0);
        match tokens[0].0 {
            &Fn => FuzzyRuleState::Ready(
                vec![(Tagged::Keyword("fn".to_string()), tokens[0].1.clone())]
            ),
            &Use => FuzzyRuleState::Ready(
                vec![(Tagged::Keyword("use".to_string()), tokens[0].1.clone())]
            ),
            &Whitespace(ref wh) => {
                println!("WH: {:?} {:?}", tokens[0].0, wh);
                match wh {
                    &WhitespaceType::Newline => {
                        println!("Rz: {:?}", wh);
                        FuzzyRuleState::Ready(
                            vec![(Tagged::Whitespace(WhitespaceType::Newline), tokens[0].1.clone())]
                        )
                    },
                    &WhitespaceType::Spaces => {
                        println!("Bz: {:?}", wh);
                        FuzzyRuleState::NotMatches
                    }
                }
            },

            _ => FuzzyRuleState::NotMatches,
        }
    }
}

pub struct FuzzyParser<'a> {
    pub rules: Vec<Box<FuzzyRule<'a>>>,
    pub size: usize,
    pub cache: VecDeque<(&'a Token, &'a Span)>,
    pub variants: Vec<usize>,
}

impl<'a> FuzzyParser<'a> {
    fn new(rules: Vec<Box<FuzzyRule<'a>>>) -> FuzzyParser<'a> {
        let size = 1;
        FuzzyParser {
            rules: rules,
            size: size,
            cache: VecDeque::with_capacity(size),
            variants: Vec::with_capacity(size),
        }
    }

    fn push(&mut self, lex: (&'a Token, &'a Span)) -> Vec<(Tagged, Span)> {
        println!("P: {:?}, {:?}", lex.0, lex.1);
        if self.cache.len() == self.size {
            self.cache.pop_front();
        }

        self.cache.push_back(lex);
        println!("{:?}", self.cache);

        for rule in &mut self.rules {
            //let () = rule;
            let res = rule.match_tokens(&self.cache);
            println!("R: {:?}", res);
            match res {
                FuzzyRuleState::NotMatches => {},
                FuzzyRuleState::Cont(usize) => {},
                FuzzyRuleState::Ready(e) => {
                    println!("Matched! {:?}", self.cache);
                    self.cache.clear();
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
        let (token, span) = lex;

        match token {
            &Token::Whitespace(_) | &Token::Comment => None,
            _ => Some(lex),
        }
    }
}

struct Fnn;

impl<'a> FuzzyCallback<'a> for Fnn {
    fn on_tokens(&mut self, tokens: &VecDeque<(&'a Token, &'a Span)>) -> Vec<(Tagged, Span)> {
        let mut kw = String::new();
        let mut name = String::new();

        match tokens[0].0 {
            &Token::Fn => { kw = "fn".to_string(); },
            _ => {},
        }

        match tokens[1].0 {
            &Token::Ident(ref n) => { name = n.clone(); },
            _ => {},
        }

        vec![
            (Tagged::Keyword(kw), tokens[0].1.clone()),
            (Tagged::Definition(name), tokens[1].1.clone()),
        ]
    }
}

struct CallFn;

impl<'a> FuzzyCallback<'a> for CallFn {
    fn on_tokens(&mut self, tokens: &VecDeque<(&'a Token, &'a Span)>) -> Vec<(Tagged, Span)> {
        let mut name = String::new();

        match tokens[0].0 {
            &Token::Ident(ref n) => { name = n.clone(); },
            _ => {},
        }

        vec![
            (Tagged::Calling(name), tokens[0].1.clone()),
        ]
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

        self.lexems.push((Token::Whitespace(WhitespaceType::Newline), Span{lo: 0, hi: 0}));

        for (tok, span) in lexer {
            println!("L: {:?} {:?}", tok, span);
            match tok {
                Token::Eof => { self.lexems.push((tok, span)); break; }
                _ => { self.lexems.push((tok, span));},
            }
        }

        let mut preproc = CPreprocessing{};
        //
        let callback = |tokens: &Vec<(&Token, &Span)>| -> Vec<(Tagged, Span)> {
            vec![]
        };
        let fn_rule = FuzzyTokenRule::new(vec![Fn, Ident(String::new()), LParen], Box::new(Fnn{}));
        let call_fn_rule = FuzzyTokenRule::new(vec![Ident(String::new()), LParen], Box::new(CallFn{}));
        let kw_rule = Box::new(KwMatch{});
        //
        let mut parser = FuzzyParser::new(vec![Box::new(fn_rule), Box::new(call_fn_rule)]);
        let mut syntax_parser = FuzzyParser::new(vec![kw_rule]);

        let mut syntax_parser_out = vec![];
        //let mut parser_out = vec![];

        for &(ref tok, ref span) in &self.lexems {
            let lsyn = syntax_parser.push((tok, span));
            if lsyn.len() != 0 {
                println!("PR: {:?}", lsyn);
                syntax_parser_out.extend(lsyn);
            }

            // if let Some((wtok, wspan)) = preproc.filter((tok, span)) {
            //     let res = parser.push((wtok, wspan));
            //
            //     if res.len() != 0 {
            //         println!("PR: {:?}", res);
            //         parser_out.extend(res);
            //     }
            // }
        }

        println!("SYN: {:?}", syntax_parser_out);

        return Context::new(syntax_parser_out);
    }
}
