
#[derive(Debug, Clone, PartialEq)]
pub enum WhitespaceType {
    Newline,
    Spaces,
    //Tabs(i32),
    //Spaces(i32),
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum Token {
    NoToken,
    Ident(String),
    QuotedString,

    T_as,
    T_break,
    T_crate,
    T_else,
    T_enum,
    T_extern,
    T_false,
    T_fn,
    T_for,
    T_if,
    T_impl,
    T_in,
    T_let,
    T_loop,
    T_match,
    T_mod,
    T_move,
    T_mut,
    T_pub,
    T_ref,
    T_return,
    T_static,
    T_self,
    T_struct,
    T_super,
    T_true,
    T_trait,
    T_type,
    T_unsafe,
    T_use,
    T_virtual,
    T_while,
    T_continue,
    T_box,
    T_const,
    T_where,
    T_proc,
    T_alignof,
    T_become,
    T_offsetof,
    T_priv,
    T_pure,
    T_sizeof,
    T_typeof,
    T_unsized,
    T_yield,
    T_do,
    T_abstract,
    T_final,
    T_override,
    T_macro,

    LParen,
    RParen,

    LFigureParen,
    RFigureParen,

    Colon2,

    Whitespace(WhitespaceType),
    Comment,

    Eof,
    Other,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub lo: usize,
    pub hi: usize,
    pub line: usize,
}

pub fn span_in(s: &str, t: &str, line: usize) -> Span {
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
