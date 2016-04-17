use std::collections::HashSet;
use std::collections::vec_deque::VecDeque;
use std::intrinsics::discriminant_value;

use indexer::lexer::{CommonLexer, Token, Span};
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
    Whitespace,

    Keyword(String),
}

// pub trait RuleCallback {
//     fn on_rule(&self, tokens: Vec<Token>) -> Tagged;
// }

enum FuzzyRuleState {
    NotMatches,
    Cont(i32, i32),
    Ready(Vec<(Tagged, Span)>),
}


pub trait FuzzyRule {
    fn match(&mut self, &VecDeque<(&'a Token, &'a Span)>) -> FuzzyRuleState;
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

impl FuzzyRule for FuzzyTokenRule {
    fn new(tokens: Vec<Token>, callback: Box<FuzzyCallback<'a>>) -> FuzzyTokenRule<'a> {
        FuzzyTokenRule {
            tokens: tokens,
            callback: callback,
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

pub struct FuzzyParser<'a> {
    pub rules: Vec<FuzzyRule<'a>>,
    pub size: usize,
    pub cache: VecDeque<(&'a Token, &'a Span)>,
    pub variants: Vec<usize>,
}

impl<'a> FuzzyParser<'a> {
    fn new(rules: Vec<FuzzyRule<'a>>) -> FuzzyParser<'a> {
        let size = 5;
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
        //self.rules[0].callback.on_tokens(&self.cache);

        self.cache.push_back(lex);
        println!("{:?}", self.cache);

        for rule in &mut self.rules {
            //let () = rule;
            if rule.tokens.len() <= self.cache.len() {
                let mut iter = 0;
                let mut matched = true;
                for rule_token in &rule.tokens {
                    //println!("A {:?} {:?}", rule_token, self.cache[iter].0);
                    if !token_eq(rule_token, self.cache[iter].0) {
                        matched = false;
                        break;
                    }
                    iter += 1;
                }

                if matched {
                    println!("Matched! {:?}", rule.tokens);
                    // let mut cc = vec![];
                    //
                    // for x in &self.cache {
                    //     let (&wtok, &wspan) = *x;
                    //     cc.push(*x);
                    // }

                    let res = rule.callback.on_tokens(&self.cache);

                    self.cache.clear();

                    return res
                }
            }
        }

        vec![]
    }

    fn finish(&mut self) -> Vec<(Tagged, Span)> {
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

        let mut ctx = Context::new();

        let mut lexer = CommonLexer::new(&self.buffer);

        for (tok, span) in lexer {
            self.lexems.push((tok, span));
        }

        let mut preproc = CPreprocessing{};
        //
        let callback = |tokens: &Vec<(&Token, &Span)>| -> Vec<(Tagged, Span)> {
            vec![]
        };
        let fn_rule = FuzzyRule::new(vec![Fn, Ident(String::new()), LParen], Box::new(Fnn{}));
        let call_fn_rule = FuzzyRule::new(vec![Ident(String::new()), LParen], Box::new(CallFn{}));
        let kw_rule = FuzzyMatchRule::new(Box::new(KwMatch{}));
        //
        let mut parser = FuzzyParser::new(vec![fn_rule, call_fn_rule]);
        let mut syntax_parser = FuzzyParser::new(vec![kw_rule]);

        let mut parser_out = vec![];

        for &(ref tok, ref span) in &self.lexems {
            if let Some((wtok, wspan)) = preproc.filter((tok, span)) {
                let res = parser.push((wtok, wspan));

                if res.len() != 0 {
                    println!("PR: {:?}", res);
                    parser_out.extend(res);
                }
            }
        }

        // let program = parse(lexer);
        //
        // match program {
        //     Ok(o) => {
        //         for p in o.stmts {
        //             println!("{:?}", p);
        //         }
        //     },
        //     Err(e) => {
        //         println!("Err: {:?}", e);
        //     }
        // }

        let mut parser_state = ParserState::Wait;

        let mut line_counter = 0;

        let mut lex_iter = 0;

        let mut use_unit_name = String::new();
        let mut use_lex_iter = 0;
        let mut use_id_iter = 0;
        let mut unit_type = String::new();

        let mut words: HashSet<&str> = HashSet::new();
        words.insert("struct");
        words.insert("use");
        words.insert("fn");
        words.insert("let");

        // loop {
        //     let ref cur = &self.lexems[lex_iter];
        //     let ref fmt = cur.content;
        //
        //     //println!("S0: {} ({:?})", fmt, parser_state);
        //
        //     let mut added = false;
        //
        //     match cur.lexem_type {
        //         LexemType::Newline => {
        //             line_counter += 1;
        //             ctx.all_tagged.push(Box::new(Tagged::Newline {
        //                 source: FileSource{
        //                     //file: String::from("src.rs"),
        //                     line: line_counter,
        //                     id_iter: cur.start_iter,
        //                     lexem_iter: lex_iter,
        //                 }
        //             }));
        //             added = true;
        //         }
        //         _ => {},
        //     }
        //
        //     match parser_state {
        //         ParserState::Wait => {
        //             match cur.lexem_type {
        //                 LexemType::Token => {
        //                     if fmt == "struct" || fmt == "fn" {
        //                         unit_type = fmt.to_string();
        //                         parser_state = ParserState::KeywordThenName;
        //                     } else if fmt == "{" || fmt == "}" || fmt == "(" || fmt == ")" {
        //                         // Skip this yet
        //                     } else { // if like identifier
        //                         use_unit_name = fmt.to_string();
        //                         use_lex_iter = lex_iter;
        //                         use_id_iter = cur.start_iter;
        //                         parser_state = ParserState::NameThenCall;
        //                     }
        //                 },
        //                 _ => {},
        //             }
        //         },
        //         ParserState::KeywordThenName => {
        //             match cur.lexem_type {
        //                 LexemType::Token => {
        //                     let unit_path = vec![cur.content.clone()];
        //                     ctx.all_tagged.push(Box::new(Tagged::Definition {
        //                         unit_type: unit_type.clone(),
        //                         path: unit_path,
        //                         source: FileSource{
        //                             //file: String::from("src.rs"),
        //                             line: line_counter,
        //                             id_iter: cur.start_iter,
        //                             lexem_iter: lex_iter,
        //                         }
        //                     }));
        //                     added = true;
        //                     parser_state = ParserState::Wait;
        //                 },
        //                 _ => {},
        //             }
        //         },
        //         ParserState::NameThenCall => {
        //             match cur.lexem_type {
        //                 LexemType::Token => {
        //                     if fmt == "(" {
        //                         let unit_path = vec![use_unit_name.clone()];
        //                         ctx.all_tagged.push(Box::new(Tagged::Calling {
        //                             unit_type: String::from("call_fn"),
        //                             path: unit_path,
        //                             source: FileSource{
        //                                 //file: String::from("src.rs"),
        //                                 line: line_counter,
        //                                 id_iter: use_id_iter,
        //                                 lexem_iter: use_lex_iter,
        //                             },
        //                             defs: vec![],
        //                         }));
        //
        //                         ctx.all_tagged.push(Box::new(Tagged::Text {
        //                             content: fmt.clone(),
        //                             source: FileSource{
        //                                 //file: String::from("src.rs"),
        //                                 line: line_counter,
        //                                 id_iter: cur.start_iter,
        //                                 lexem_iter: lex_iter,
        //                             }
        //                         }));
        //
        //                         added = true;
        //                         println!("CC: {}, {}", use_lex_iter, &self.lexems[use_lex_iter].content);
        //                         println!("DD: {}, {}", &self.lexems[lex_iter].content, fmt);
        //                     } else {
        //                         parser_state = ParserState::Wait;
        //                         use_unit_name = String::new();
        //                     }
        //                 },
        //                 _ => {},
        //             }
        //         },
        //     }
        //
        //     if !added {
        //         if words.contains(&fmt[..]) {
        //             ctx.all_tagged.push(Box::new(Tagged::Keyword {
        //                 content: fmt.clone(),
        //                 source: FileSource{
        //                     //file: String::from("src.rs"),
        //                     line: line_counter,
        //                     id_iter: cur.start_iter,
        //                     lexem_iter: lex_iter,
        //                 }
        //             }));
        //             parser_state = ParserState::Wait;
        //
        //             added = true;
        //         }
        //     }
        //
        //     if !added {
        //         match parser_state {
        //             //ParserState::NameThenCall => {},
        //             _ => {
        //                 ctx.all_tagged.push(Box::new(Tagged::Text {
        //                     content: fmt.clone(),
        //                     source: FileSource{
        //                         //file: String::from("src.rs"),
        //                         line: line_counter,
        //                         id_iter: cur.start_iter,
        //                         lexem_iter: lex_iter,
        //                     }
        //                 }));
        //                 added = true;
        //             },
        //         }
        //     }
        //
        //     //println!("  -> ({:?})", parser_state);
        //
        //     if lex_iter == &self.lexems.len() - 1 {
        //         break;
        //     }
        //
        //     lex_iter += 1;
        // }

        ctx
    }
}
