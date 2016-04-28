
#[derive(Debug, Clone, PartialEq)]
pub enum WhitespaceType {
    Newline,
    Spaces,
    //Tabs(i32),
    //Spaces(i32),
}

#[derive(Debug, Clone)]
pub enum Token {
    NoToken,
    Ident(String),
    QuotedString,

    As,
    Break,
    Crate,
    Else,
    Enum,
    Extern,
    False,
    Fn,
    For,
    If,
    Impl,
    In,
    Let,
    Loop,
    Match,
    Mod,
    Move,
    Mut,
    Pub,
    Ref,
    Return,
    Static,
    SelfType,
    Struct,
    Super,
    True,
    Trait,
    Type,
    Unsafe,
    Use,
    Virtual,
    While,
    Continue,
    BoxT,
    Const,
    Where,
    Proc,
    Alignof,
    Become,
    Offsetof,
    Priv,
    Pure,
    Sizeof,
    Typeof,
    Unsized,
    Yield,
    Do,
    Abstract,
    Final,
    Override,
    Macro,

    LParen,
    RParen,

    LFigureParen,
    RFigureParen,

    Whitespace(WhitespaceType),
    Comment,

    Eof,
    Other,
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

    r#"."# => (Token::Other, text),
}

pub struct CommonLexer<'a> {
    original: &'a str,
    remaining: &'a str,
    line_counter: usize,
}

impl<'a> CommonLexer<'a> {
    pub fn new(s: &'a str) -> CommonLexer<'a> {
        CommonLexer { original: s, remaining: s, line_counter: 0 }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
    pub line: usize,
}

fn span_in(s: &str, t: &str, line: usize) -> Span {
    let lo = s.as_ptr() as usize - t.as_ptr() as usize;
    Span {
        lo: lo,
        hi: lo + s.len(),
        line: line,
    }
}

impl Span {
    pub fn end() -> Span {
        Span {
            lo: usize::max_value(),
            hi: usize::max_value(),
            line: usize::max_value(),
        }
    }
}

impl<'a> Iterator for CommonLexer<'a> {
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
