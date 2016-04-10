use std::io::prelude::*;

#[derive(Debug)]
pub enum LexemType {
    Unknown,
    Whitespace,
    Newline,
    Token,
}

pub struct Lexem {
    pub lexem_type: LexemType,
    pub content: String,
    pub start_iter: usize,
}

pub trait Lexer {
    fn next(&mut self) -> Option<Lexem>;
}

pub struct CommonLexer<'a> {
    pub buffer: &'a String,
    pub read_iter: usize,
}

impl<'a> CommonLexer<'a> {
    pub fn new(buffer: &String) -> CommonLexer {
        CommonLexer {
            buffer: buffer,
            read_iter: 0,
        }
    }
}

impl<'a> Lexer for CommonLexer<'a> {
    fn next(&mut self) -> Option<Lexem> {
        let mut read = String::new();
        let mut lexem_type = LexemType::Unknown;
        let start_iter = self.read_iter;

        loop {
            if let Some(ch) = self.buffer.chars().nth(self.read_iter) {
                let is_whitespace = ch.is_whitespace();
                match lexem_type {
                    LexemType::Unknown => {
                        if ch == '\r' {
                            lexem_type = LexemType::Newline;
                            self.read_iter += 2;
                            read = String::from("\n");
                            break
                        } else if ch == '\n' {
                            lexem_type = LexemType::Newline;
                        } else if is_whitespace {
                            lexem_type = LexemType::Whitespace;
                        }
                        else {
                            lexem_type = LexemType::Token;
                        }
                    },
                    LexemType::Token => {
                        if is_whitespace {
                            break;
                        } else if ch == '(' || ch == ')' {
                            break;
                        }
                    },
                    LexemType::Newline => {
                        break;
                    },
                    LexemType::Whitespace => {
                        if !is_whitespace {
                            break;
                        }
                    },
                }

                self.read_iter += 1;
                read.push_str(&ch.to_string());
            }
            else {
                match lexem_type {
                    LexemType::Unknown => return None,
                    _ => break,
                }
            }
        }

        Some(Lexem {
            lexem_type: lexem_type,
            content: read,
            start_iter: start_iter,
        })
    }
}
