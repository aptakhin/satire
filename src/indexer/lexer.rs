
use std::io::Read;
//use std::io::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub enum WhitespaceType {
    Newline(usize),
    Spaces,
    //Tabs(i32),
    //Spaces(i32),
}

#[derive(Debug, Clone)]
pub enum Token {
    NoToken,
    Ident(String),

    Use,
    Struct,
    Fn,
    Pub,
    Impl,
    Let,
    Mut,

    LParen,
    RParen,

    Whitespace(WhitespaceType),
    Comment,

    Eof,
    Other,
}

lexer! {
    fn next_token(text: 'a) -> (Token, &'a str);

    r#"[\n]"# => (Token::Whitespace(WhitespaceType::Newline(0)), text),
    //r#"\r\n"# => (Token::Whitespace(WhitespaceType::Newline), text),
    r#"[ \t]+"# => (Token::Whitespace(WhitespaceType::Spaces), text),
    // "C-style" comments (/* .. */) - can't contain "*/"
    r#"/[*](~(.*[*]/.*))[*]/"# => (Token::Comment, text),
    // "C++-style" comments (// ...)
    r#"//[^\n]*"# => (Token::Comment, text),

    r#"use"# => (Token::Use, text),
    r#"struct"# => (Token::Struct, text),
    r#"fn"# => (Token::Fn, text),
    r#"pub"# => (Token::Pub, text),
    r#"impl"# => (Token::Impl, text),
    r#"let"# => (Token::Let, text),
    r#"mut"# => (Token::Mut, text),

    r#"[a-zA-Z_][a-zA-Z0-9_]*"# => (Token::Ident(text.to_owned()), text),

    r#"\("# => (Token::LParen, text),
    r#"\)"# => (Token::RParen, text),

    r#"."# => (Token::Other, text),
}

pub struct CommonLexer<'a> {
    original: &'a str,
    remaining: &'a str,
}

impl<'a> CommonLexer<'a> {
    pub fn new(s: &'a str) -> CommonLexer<'a> {
        CommonLexer { original: s, remaining: s }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
}

fn span_in(s: &str, t: &str) -> Span {
    let lo = s.as_ptr() as usize - t.as_ptr() as usize;
    Span {
        lo: lo,
        hi: lo + s.len(),
    }
}

impl<'a> Iterator for CommonLexer<'a> {
    type Item = (Token, Span);
    fn next(&mut self) -> Option<(Token, Span)> {
        loop {
            if let Some((tok, span)) = next_token(&mut self.remaining) {
                return Some((tok, span_in(span, self.original)));
            } else {
                return Some((Token::Eof, Span{lo: self.original.len(), hi: self.original.len()}))
            };
        }
    }
}
