use std::collections::vec_deque::VecDeque;
use std::intrinsics::discriminant_value;
use std::cmp::{min, max};
use std::rc::Rc;

use indexer::lexer::{CommonLexer, Token, Span, WhitespaceType, span_in};
use indexer::storage::PreparsedFile;
use indexer::parser::{CommonParser, Tagged, Preprocessing, CPreprocessing, FuzzyParser, FuzzyRule, FuzzyRuleState,
    match_tokens, merge_result, token_eq, Path, UseContext};


pub struct RustParser {
    pub file: String,
    pub buffer: Rc<String>,
    pub lexems: Vec<(Token, Span)>,
}

impl RustParser {
    pub fn new(file: String, buffer: Rc<String>) -> RustParser {
        RustParser {
            file: file,
            buffer: buffer,
            lexems: vec![],
        }
    }
}

lexer! {
    fn next_token(text: 'a) -> (Token, &'a str);

    r#"[\n]"# => (Token::Whitespace(WhitespaceType::Newline), text),
    //r#"\r\n"# => (Token::Whitespace(WhitespaceType::Newline), text),
    r#"[ \t]+"# => (Token::Whitespace(WhitespaceType::Spaces), text),
    // "C-style" comments (/* .. */) - can't contain "*/"
    r#"/[*](~(.*[*]/.*))[*]/"# => (Token::Comment, text),
    // "C++-style" comments (// ...)
    r#"//[^\n]*"# => (Token::Comment, text),

    r#"\"(?:[^"\\]|\\.)*\""# => (Token::QuotedString, text),


    r#"as"# => (Token::As, text),
    r#"break"# => (Token::Break, text),
    r#"crate"# => (Token::Crate, text),
    r#"else"# => (Token::Else, text),
    r#"enum"# => (Token::Enum, text),
    r#"extern"# => (Token::Extern, text),
    r#"false"# => (Token::False, text),
    r#"fn"# => (Token::Fn, text),
    r#"for"# => (Token::For, text),
    r#"if"# => (Token::If, text),
    r#"impl"# => (Token::Impl, text),
    r#"in"# => (Token::In, text),
    r#"let"# => (Token::Let, text),
    r#"loop"# => (Token::Loop, text),
    r#"match"# => (Token::Match, text),
    r#"mod"# => (Token::Mod, text),
    r#"move"# => (Token::Move, text),
    r#"mut"# => (Token::Mut, text),
    r#"pub"# => (Token::Pub, text),
    r#"ref"# => (Token::Ref, text),
    r#"return"# => (Token::Return, text),
    r#"static"# => (Token::Static, text),
    r#"self"# => (Token::SelfType, text),
    r#"struct"# => (Token::Struct, text),
    r#"super"# => (Token::Super, text),
    r#"true"# => (Token::True, text),
    r#"trait"# => (Token::Trait, text),
    r#"type"# => (Token::Type, text),
    r#"unsafe"# => (Token::Unsafe, text),
    r#"use"# => (Token::Use, text),
    r#"virtual"# => (Token::Virtual, text),
    r#"while"# => (Token::While, text),
    r#"continue"# => (Token::Continue, text),
    r#"box"# => (Token::BoxT, text),
    r#"const"# => (Token::Const, text),
    r#"where"# => (Token::Where, text),
    r#"proc"# => (Token::Proc, text),
    r#"alignof"# => (Token::Alignof, text),
    r#"become"# => (Token::Become, text),
    r#"offsetof"# => (Token::Offsetof, text),
    r#"priv"# => (Token::Priv, text),
    r#"pure"# => (Token::Pure, text),
    r#"sizeof"# => (Token::Sizeof, text),
    r#"typeof"# => (Token::Typeof, text),
    r#"unsized"# => (Token::Unsized, text),
    r#"yield"# => (Token::Yield, text),
    r#"do"# => (Token::Do, text),
    r#"abstract"# => (Token::Abstract, text),
    r#"final"# => (Token::Final, text),
    r#"override"# => (Token::Override, text),
    r#"macro"# => (Token::Macro, text),

    r#"[a-zA-Z_][a-zA-Z0-9_]*"# => (Token::Ident(text.to_owned()), text),

    r#"\("# => (Token::LParen, text),
    r#"\)"# => (Token::RParen, text),

    r#"{"# => (Token::LFigureParen, text),
    r#"}"# => (Token::RFigureParen, text),

    r#"::"# => (Token::Colon2, text),

    r#"."# => (Token::Other, text),
}

pub struct RustLexer<'a> {
    original: &'a str,
    remaining: &'a str,
    line_counter: usize,
}

impl<'a> RustLexer<'a> {
    pub fn new(s: &'a str) -> RustLexer<'a> {
        RustLexer { original: s, remaining: s, line_counter: 0 }
    }
}

impl<'a> Iterator for RustLexer<'a> {
    type Item = (Token, Span);
    fn next(&mut self) -> Option<(Token, Span)> {
        if self.line_counter == 0 {
            self.line_counter = 1;
            let item = Some((
                Token::Whitespace(WhitespaceType::Newline),
                Span {
                    lo: 0,
                    hi: 0,
                    line: self.line_counter,
                }
            ));
            return item
        }

        loop {
            if let Some((tok, span)) = next_token(&mut self.remaining) {
                match &tok {
                    &Token::Whitespace(WhitespaceType::Newline) => {
                        self.line_counter += 1;
                    },
                    _ => {},
                }
                return Some((tok, span_in(span, self.original, self.line_counter)));
            } else {
                return Some((
                    Token::Eof,
                    Span {
                        lo: self.original.len(),
                        hi: self.original.len(),
                        line: self.line_counter,
                    }
                ))
            };
        }
    }
}


pub struct KwMatch;
pub struct FnMatch;

impl CommonParser for RustParser {
    fn parse(&mut self) -> PreparsedFile {
        let mut lexer = RustLexer::new(&self.buffer);

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

        let kw_rule = Box::new(KwMatch{});
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

        PreparsedFile::new(self.file.clone(), self.buffer.clone(), syntax_parser_out, parser_out)
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
