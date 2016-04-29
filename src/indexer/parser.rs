use std::collections::vec_deque::VecDeque;
use std::intrinsics::discriminant_value;
use std::cmp::{min, max};

use indexer::lexer::{CommonLexer, Token, Span, WhitespaceType};
use indexer::storage::PreparsedFile;

#[derive(Debug)]
pub enum ParserState {
    Wait,
    KeywordThenName,
    NameThenCall,
}

#[derive(Debug, Clone)]
pub struct Path {
    pub path: Vec<(Token, String)>,
}

impl Path {
    fn named(token: Token, name: String) -> Path {
        Path {
            path: vec![(token, name)],
        }
    }

    fn path(paths: Vec<(Token, String)>) -> Path {
        Path {
            path: paths,
        }
    }
}

impl PartialEq for Path {
    fn eq(&self, other: &Path) -> bool {
        if self.path.len() != other.path.len() {
            return false;
        }

        for i in 0..self.path.len() {
            if !token_eq(&self.path[i].0, &other.path[i].0) || self.path[i].1 != other.path[i].1 {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone)]
pub struct UseContext {
    pub reference: Path,
    pub used_from: Path,
}

impl UseContext {
    fn new(reference: Path, used_from: Path) -> UseContext {
        UseContext {
            reference: reference,
            used_from: used_from,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Tagged {
    Definition(UseContext),
    Calling(UseContext),
    Whitespace(WhitespaceType),
    Comment,
    QuotedString,
    Keyword(Token),
    Eof,
}

#[derive(Debug)]
pub enum FuzzyRuleState {
    NotMatches,
    Cont(usize),
    Ready(usize, Vec<(Tagged, Span)>),
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
            FuzzyRuleState::Ready(matched, res)
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
    line_counter__: usize,
}

impl KwMatch {
    fn new() -> KwMatch {
        KwMatch { line_counter__: 1 }
    }
}

impl<'a> FuzzyRule<'a> for KwMatch {
    fn match_tokens(&mut self, tokens: &VecDeque<(&'a Token, &'a Span)>) -> FuzzyRuleState {
        use indexer::lexer::Token::*;
        //println!("Q: {:?}", tokens[0].0);

        match tokens[0].0 {
            &As | &Break | &Crate | &Else | &Enum | &Extern | &False | &Fn | &For | &If |
            &Impl | &In | &Let | &Loop | &Match | &Mod | &Move | &Mut | &Pub | &Ref |
            &Return | &Static | &SelfType | &Struct | &Super | &True |
            &Trait | &Type | &Unsafe | &Use | &Virtual | &While | &Continue | &BoxT |
            &Const | &Where | &Proc | &Alignof | &Become | &Offsetof | &Priv | &Pure |
            &Sizeof | &Typeof | &Unsized | &Yield | &Do | &Abstract | &Final | &Override |
            &Macro => FuzzyRuleState::Ready(
                1,
                vec![(Tagged::Keyword(tokens[0].0.clone()), tokens[0].1.clone())],
            ),
            &Comment => FuzzyRuleState::Ready(
                1,
                vec![(Tagged::Comment, tokens[0].1.clone())],
            ),
            &QuotedString => FuzzyRuleState::Ready(
                1,
                vec![(Tagged::QuotedString, tokens[0].1.clone())],
            ),
            &Whitespace(ref wh) => {
                match wh {
                    &WhitespaceType::Newline => {
                        let state = FuzzyRuleState::Ready(
                            1,
                            vec![(Tagged::Whitespace(WhitespaceType::Newline), tokens[0].1.clone())],
                        );
                        state
                    },
                    &WhitespaceType::Spaces => {
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

fn merge_result(cur_res: FuzzyRuleState, prev_res: FuzzyRuleState) -> FuzzyRuleState {
    let merged_res;
    match (&cur_res, &prev_res) {
        (&FuzzyRuleState::Cont(cur_len), &FuzzyRuleState::Cont(prev_len)) if cur_len < prev_len => {
            merged_res = prev_res;
        },
        (&FuzzyRuleState::NotMatches, _) => {
            merged_res = prev_res;
        }
        _ => { merged_res = cur_res; },
    }
    merged_res
}

impl<'a> FuzzyRule<'a> for FnMatch {
    fn match_tokens(&mut self, tokens: &VecDeque<(&'a Token, &'a Span)>) -> FuzzyRuleState {
        use indexer::lexer::Token::*;
        //println!("Q: {:?}", tokens);
        let mut res = FuzzyRuleState::NotMatches;

        let cur_context = Path::named(Token::Mod, ".".to_string());

        {
            let rr = vec![Fn, Ident(String::new()), LParen];
            let mut cur_match = match_tokens(&rr, tokens);

            match cur_match {
                FuzzyRuleState::Cont(len) if tokens.len() >= len => {
                    let mut name = String::new();
                    match tokens[1].0 {
                        &Token::Ident(ref n) => { name = n.clone(); },
                        _ => {},
                    }

                    cur_match = FuzzyRuleState::Ready(
                        rr.len(),
                        vec![(Tagged::Definition(UseContext::new(Path::named(Fn, name), cur_context.clone())), tokens[1].1.clone())],
                    );
                },
                _ => {},
            }
            res = merge_result(cur_match, res);
        }

        {
            let rr = vec![Struct, Ident(String::new()), LFigureParen];
            let mut cur_match = match_tokens(&rr, tokens);

            match cur_match {
                FuzzyRuleState::Cont(len) if tokens.len() >= len => {
                    let mut name = String::new();
                    match tokens[1].0 {
                        &Token::Ident(ref n) => { name = n.clone(); },
                        _ => {},
                    }

                    cur_match = FuzzyRuleState::Ready(
                        rr.len(),
                        vec![(Tagged::Definition(UseContext::new(Path::named(Struct, name), cur_context.clone())), tokens[1].1.clone())],
                    );
                },
                _ => {},
            }
            res = merge_result(cur_match, res);
        }

        {
            let rr = vec![Ident(String::new()), LFigureParen];
            let mut cur_match = match_tokens(&rr, tokens);

            match cur_match {
                FuzzyRuleState::Cont(len) if tokens.len() >= len => {
                    let mut name = String::new();
                    match tokens[0].0 {
                        &Token::Ident(ref n) => { name = n.clone(); },
                        _ => {},
                    }

                    cur_match = FuzzyRuleState::Ready(
                        rr.len(),
                        vec![(Tagged::Calling(UseContext::new(Path::named(Struct, name), cur_context.clone())), tokens[0].1.clone())],
                    );
                },
                _ => {},
            }
            res = merge_result(cur_match, res);
        }

        {
            let rr = vec![Ident(String::new()), LParen];
            let mut cur_match = match_tokens(&rr, tokens);

            match cur_match {
                FuzzyRuleState::Cont(len) if tokens.len() >= len => {
                    let mut name = String::new();
                    match tokens[0].0 {
                        &Token::Ident(ref n) => { name = n.clone(); },
                        _ => {},
                    }

                    cur_match = FuzzyRuleState::Ready(
                        rr.len(),
                        vec![(Tagged::Calling(UseContext::new(Path::named(Fn, name), cur_context.clone())), tokens[0].1.clone())],
                    );
                },
                _ => {},
            }
            res = merge_result(cur_match, res);
        }

        {
            let rr = vec![Ident(String::new()), Colon2, Ident(String::new()), LParen];
            let mut cur_match = match_tokens(&rr, tokens);

            match cur_match {
                FuzzyRuleState::Cont(len) if tokens.len() >= len => {
                    //println!("Match with colons: {:?}", tokens);
                    let mut struct_name = String::new();
                    match tokens[0].0 {
                        &Token::Ident(ref n) => { struct_name = n.clone(); },
                        _ => {},
                    }

                    let mut method_name = String::new();
                    match tokens[2].0 {
                        &Token::Ident(ref n) => { method_name = n.clone(); },
                        _ => {},
                    }

                    let reference = Path::path(vec![(Token::Struct, struct_name), (Token::Fn, method_name)]);

                    cur_match = FuzzyRuleState::Ready(
                        rr.len(),
                        vec![(Tagged::Calling(UseContext::new(reference, cur_context.clone())), tokens[2].1.clone())],
                    );
                },
                _ => {},
            }
            res = merge_result(cur_match, res);
        }

        res
    }
}

pub struct FuzzyParser<'a> {
    pub rules: Vec<Box<FuzzyRule<'a>>>,
    pub current_size: usize,
    pub cache: VecDeque<(&'a Token, &'a Span)>,
    pub variants: Vec<usize>,
}

impl<'a> FuzzyParser<'a> {
    fn new(rules: Vec<Box<FuzzyRule<'a>>>) -> FuzzyParser<'a> {
        FuzzyParser {
            rules: rules,
            current_size: 1,
            cache: VecDeque::new(),
            variants: Vec::new(),
        }
    }

    fn push(&mut self, lex: (&'a Token, &'a Span)) -> Vec<(Tagged, Span)> {
        //println!("P: {:?}, {:?}", lex.0, lex.1);
        if self.cache.len() >= self.current_size {
            // Delete not more one token at once
            self.cache.pop_front();
        }

        self.cache.push_back(lex);
        //println!("{:?}", self.cache);

        let mut new_queue_size = 1;

        for rule in &mut self.rules {
            //let () = rule;
            let res = rule.match_tokens(&self.cache);
            //println!("R: {:?}", res);
            match res {
                FuzzyRuleState::NotMatches => {},
                FuzzyRuleState::Cont(max_size) => { new_queue_size = max(max_size, new_queue_size); },
                FuzzyRuleState::Ready(tokens_eaten, tagged) => {
                    //println!("Matched! {:?}, {}", self.cache, tokens_eaten);
                    for i in 0..tokens_eaten {
                        self.cache.pop_front();
                    }
                    return tagged
                },
            }
        }

        self.current_size = new_queue_size;

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

pub struct CommonParser {
    pub file: String,
    pub buffer: String,
    pub lexems: Vec<(Token, Span)>,
}

impl CommonParser {
    pub fn new(file: String, buffer: String) -> CommonParser {
        CommonParser {
            file: file,
            buffer: buffer,
            lexems: vec![],
        }
    }

    pub fn parse(&mut self) -> PreparsedFile {
        let mut lexer = CommonLexer::new(&self.buffer);

        for (tok, span) in lexer {
            //println!("L: {:?} {:?} {}", tok, span, line_counter);
            match tok {
                Token::Eof => {
                    self.lexems.push((tok, span));
                    break;
                }
                _ => {},
            }

            self.lexems.push((tok, span));
        }

        let mut preproc = CPreprocessing{};

        let kw_rule = Box::new(KwMatch::new());
        let fn_rule = Box::new(FnMatch{});

        let mut parser = FuzzyParser::new(vec![fn_rule]);
        let mut syntax_parser = FuzzyParser::new(vec![kw_rule]);

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

        //println!("SYN: {:?}", syntax_parser_out);
        //println!("PRS: {:?}", parser_out);

        PreparsedFile::new(self.file.clone(), syntax_parser_out, parser_out)
    }
}
