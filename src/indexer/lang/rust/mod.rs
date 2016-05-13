use std::collections::vec_deque::VecDeque;
use std::rc::Rc;

use indexer::lexer::{Token, Span, WhitespaceType, span_in};
use indexer::storage::PreparsedFile;
use indexer::parser::{CommonParser, Tagged, Preprocessing, CPreprocessing, FuzzyParser, FuzzyRule, FuzzyRuleState,
    match_tokens, merge_result, Path, UseContext};


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

    r#""[^\n"]*""# => (Token::QuotedString, text),


    r#"as"# => (Token::T_as, text),
    r#"break"# => (Token::T_break, text),
    r#"crate"# => (Token::T_crate, text),
    r#"else"# => (Token::T_else, text),
    r#"enum"# => (Token::T_enum, text),
    r#"extern"# => (Token::T_extern, text),
    r#"false"# => (Token::T_false, text),
    r#"fn"# => (Token::T_fn, text),
    r#"for"# => (Token::T_for, text),
    r#"if"# => (Token::T_if, text),
    r#"impl"# => (Token::T_impl, text),
    r#"in"# => (Token::T_in, text),
    r#"let"# => (Token::T_let, text),
    r#"loop"# => (Token::T_loop, text),
    r#"match"# => (Token::T_match, text),
    r#"mod"# => (Token::T_mod, text),
    r#"move"# => (Token::T_move, text),
    r#"mut"# => (Token::T_mut, text),
    r#"pub"# => (Token::T_pub, text),
    r#"ref"# => (Token::T_ref, text),
    r#"return"# => (Token::T_return, text),
    r#"static"# => (Token::T_static, text),
    r#"self"# => (Token::T_self, text),
    r#"struct"# => (Token::T_struct, text),
    r#"super"# => (Token::T_super, text),
    r#"true"# => (Token::T_true, text),
    r#"trait"# => (Token::T_trait, text),
    r#"type"# => (Token::T_type, text),
    r#"unsafe"# => (Token::T_unsafe, text),
    r#"use"# => (Token::T_use, text),
    r#"virtual"# => (Token::T_virtual, text),
    r#"while"# => (Token::T_while, text),
    r#"continue"# => (Token::T_continue, text),
    r#"box"# => (Token::T_box, text),
    r#"const"# => (Token::T_const, text),
    r#"where"# => (Token::T_where, text),
    r#"proc"# => (Token::T_proc, text),
    r#"alignof"# => (Token::T_alignof, text),
    r#"become"# => (Token::T_become, text),
    r#"offsetof"# => (Token::T_offsetof, text),
    r#"priv"# => (Token::T_priv, text),
    r#"pure"# => (Token::T_pure, text),
    r#"sizeof"# => (Token::T_sizeof, text),
    r#"typeof"# => (Token::T_typeof, text),
    r#"unsized"# => (Token::T_unsized, text),
    r#"yield"# => (Token::T_yield, text),
    r#"do"# => (Token::T_do, text),
    r#"abstract"# => (Token::T_abstract, text),
    r#"final"# => (Token::T_final, text),
    r#"override"# => (Token::T_override, text),
    r#"macro"# => (Token::T_macro, text),

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
        let lexer = RustLexer::new(&self.buffer);

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
            &T_as | &T_break | &T_crate | &T_else | &T_enum | &T_extern | &T_false | &T_fn |
            &T_for | &T_if | &T_impl | &T_in | &T_let | &T_loop | &T_match | &T_mod | &T_move |
            &T_mut | &T_pub | &T_ref | &T_return | &T_static | &T_self |
            &T_struct | &T_super | &T_true | &T_trait | &T_type | &T_unsafe | &T_use |
            &T_virtual | &T_while | &T_continue | &T_box | &T_const | &T_where | &T_proc |
            &T_alignof | &T_become | &T_offsetof | &T_priv | &T_pure | &T_sizeof | &T_typeof |
            &T_unsized | &T_yield | &T_do | &T_abstract | &T_final | &T_override | &T_macro
            => FuzzyRuleState::Ready(
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

        let cur_context = Path::named(T_mod, ".".to_string());

        {
            let rr = vec![T_fn, Ident(String::new()), LParen];
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
                        vec![(Tagged::Definition(UseContext::new(Path::named(T_fn, name), cur_context.clone())), tokens[1].1.clone())],
                    );
                },
                _ => {},
            }
            res = merge_result(cur_match, res);
        }

        {
            let rr = vec![T_struct, Ident(String::new()), LFigureParen];
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
                        vec![(Tagged::Definition(UseContext::new(Path::named(T_struct, name), cur_context.clone())), tokens[1].1.clone())],
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
                        vec![(Tagged::Calling(UseContext::new(Path::named(T_struct, name), cur_context.clone())), tokens[0].1.clone())],
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
                        vec![(Tagged::Calling(UseContext::new(Path::named(T_fn, name), cur_context.clone())), tokens[0].1.clone())],
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

                    let reference = Path::path(vec![(T_struct, struct_name), (T_fn, method_name)]);

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
