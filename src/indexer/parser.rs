use std::collections::HashSet;

use indexer::lexer::Lexem;
use indexer::lexer::Lexer;
use indexer::lexer::LexemType;
use indexer::lexer::CommonLexer;
use indexer::storage::FileSource;
use indexer::storage::Context;

#[derive(Debug)]
pub enum ParseState {
    Wait,
    KeywordThenName,
    NameThenCall,
}

pub struct CommonParser {
    pub buffer: String,
    pub lexems: Vec<Lexem>,
}

pub enum Tagged {
    Definition { unit_type: String, path: Vec<String>, source: FileSource },
    Calling { unit_type: String, path: Vec<String>, source: FileSource, defs: Vec<FileSource> },
    Newline { source: FileSource },
    Keyword { content: String, source: FileSource },
    Text { content: String, source: FileSource },
}

impl CommonParser {
    pub fn new(buffer: String) -> CommonParser {
        CommonParser {
            buffer: buffer,
            lexems: vec![],
        }
    }

    pub fn parse(&mut self) -> Context {
        let mut ctx = Context::new();

        let mut lexer = CommonLexer::new(&self.buffer);

        self.lexems.push(Lexem{
            lexem_type: LexemType::Newline,
            content: String::from("\n"),
            start_iter: 0,
        });

        loop {
            if let Some(lexem) = lexer.next() {
                self.lexems.push(lexem);
            } else {
                break;
            }
        }

        let mut parse_state = ParseState::Wait;

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

        loop {
            let ref cur = &self.lexems[lex_iter];
            let ref fmt = cur.content;

            //println!("S0: {} ({:?})", fmt, parse_state);

            let mut added = false;

            match cur.lexem_type {
                LexemType::Newline => {
                    line_counter += 1;
                    ctx.all_tagged.push(Box::new(Tagged::Newline {
                        source: FileSource{
                            //file: String::from("src.rs"),
                            line: line_counter,
                            id_iter: cur.start_iter,
                            lexem_iter: lex_iter,
                        }
                    }));
                    added = true;
                }
                _ => {},
            }

            match parse_state {
                ParseState::Wait => {
                    match cur.lexem_type {
                        LexemType::Token => {
                            if fmt == "struct" || fmt == "fn" {
                                unit_type = fmt.to_string();
                                parse_state = ParseState::KeywordThenName;
                            } else if fmt == "{" || fmt == "}" {
                                // Skip this yet
                            } else { // if like identifier
                                use_unit_name = fmt.to_string();
                                use_lex_iter = lex_iter;
                                use_id_iter = cur.start_iter;
                                parse_state = ParseState::NameThenCall;
                            }
                        },
                        _ => {},
                    }
                },
                ParseState::KeywordThenName => {
                    match cur.lexem_type {
                        LexemType::Token => {
                            let unit_path = vec![cur.content.clone()];
                            ctx.all_tagged.push(Box::new(Tagged::Definition {
                                unit_type: unit_type.clone(),
                                path: unit_path,
                                source: FileSource{
                                    //file: String::from("src.rs"),
                                    line: line_counter,
                                    id_iter: cur.start_iter,
                                    lexem_iter: lex_iter,
                                }
                            }));
                            added = true;
                            parse_state = ParseState::Wait;
                        },
                        _ => {},
                    }
                },
                ParseState::NameThenCall => {
                    match cur.lexem_type {
                        LexemType::Token => {
                            if fmt == "(" {
                                let unit_path = vec![use_unit_name.clone()];
                                ctx.all_tagged.push(Box::new(Tagged::Calling {
                                    unit_type: String::from("call_fn"),
                                    path: unit_path,
                                    source: FileSource{
                                        //file: String::from("src.rs"),
                                        line: line_counter,
                                        id_iter: use_id_iter,
                                        lexem_iter: use_lex_iter,
                                    },
                                    defs: vec![],
                                }));

                                ctx.all_tagged.push(Box::new(Tagged::Text {
                                    content: fmt.clone(),
                                    source: FileSource{
                                        //file: String::from("src.rs"),
                                        line: line_counter,
                                        id_iter: cur.start_iter,
                                        lexem_iter: lex_iter,
                                    }
                                }));

                                added = true;
                                println!("CC: {}, {}", use_lex_iter, &self.lexems[use_lex_iter].content);
                                println!("DD: {}, {}", &self.lexems[lex_iter].content, fmt);
                            } else {
                                parse_state = ParseState::Wait;
                                use_unit_name = String::new();
                            }
                        },
                        _ => {},
                    }
                },
            }

            if !added {
                if words.contains(&fmt[..]) {
                    ctx.all_tagged.push(Box::new(Tagged::Keyword {
                        content: fmt.clone(),
                        source: FileSource{
                            //file: String::from("src.rs"),
                            line: line_counter,
                            id_iter: cur.start_iter,
                            lexem_iter: lex_iter,
                        }
                    }));
                    added = true;
                }
            }

            if !added {
                ctx.all_tagged.push(Box::new(Tagged::Text {
                    content: fmt.clone(),
                    source: FileSource{
                        //file: String::from("src.rs"),
                        line: line_counter,
                        id_iter: cur.start_iter,
                        lexem_iter: lex_iter,
                    }
                }));
                added = true;
            }

            //println!("  -> ({:?})", parse_state);

            if lex_iter == &self.lexems.len() - 1 {
                break;
            }

            lex_iter += 1;
        }

        ctx
    }
}
