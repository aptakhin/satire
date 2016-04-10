use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;
use std::collections::HashSet;

use indexer::lexer::Lexem;
use indexer::lexer::Lexer;
use indexer::lexer::LexemType;
use indexer::lexer::CommonLexer;
use indexer::storage::Unit;
use indexer::storage::FileSource;
use indexer::storage::Context;
use indexer::storage::Tagged;

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

pub struct Definition {
    pub unit_type: String,
    pub path: Vec<String>,
    pub source: FileSource,
}

impl Tagged for Definition {
    fn get_content(&self) -> String {
        self.path[0].clone()
    }

    fn render_html(&self) -> String {
        self.path[0].clone()
    }
}

pub struct Calling {
    pub unit_type: String,
    pub path: Vec<String>,
    pub source: FileSource,
    pub origins: Vec<FileSource>,
}

impl Tagged for Calling {
    fn get_content(&self) -> String {
        self.path[0].clone()
    }

    fn render_html(&self) -> String {
        if self.origins.len() > 0 {
            self.origins[0].render_html(&self.path[0][..])
        } else {
            self.path[0].clone()
        }
    }
}

pub struct Newline {
    pub source: FileSource,
}

impl Tagged for Newline {
    fn get_content(&self) -> String {
        String::from("\n")
    }

    fn render_html(&self) -> String {
        if self.source.line == 1 {
            format!("<a name=\"l{}\">", self.source.line)
        } else {
            format!("\n<a name=\"l{}\">", self.source.line)
        }
    }
}

pub struct Text {
    pub content: String,
    pub source: FileSource,
}

impl Tagged for Text {
    fn get_content(&self) -> String {
        self.content.clone()
    }

    fn render_html(&self) -> String {
        self.content.clone()
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

        loop {
            let ref cur = &self.lexems[lex_iter];
            let ref fmt = cur.content;

            //println!("S0: {} ({:?})", fmt, parse_state);

            let mut added = false;

            match cur.lexem_type {
                LexemType::Newline => {
                    line_counter += 1;
                    ctx.all_tagged.push(Box::new(Newline {
                        source: FileSource{
                            file: String::from("src.rs"),
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
                            ctx.all_tagged.push(Box::new(Definition {
                                unit_type: unit_type.clone(),
                                path: unit_path,
                                source: FileSource{
                                    file: String::from("src.rs"),
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
                                ctx.all_tagged.push(Box::new(Calling {
                                    unit_type: String::from("call_fn"),
                                    path: unit_path,
                                    source: FileSource{
                                        file: String::from("src.rs"),
                                        line: line_counter,
                                        id_iter: use_id_iter,
                                        lexem_iter: use_lex_iter,
                                    },
                                    origins: vec![],
                                }));

                                ctx.all_tagged.push(Box::new(Text {
                                    content: fmt.clone(),
                                    source: FileSource{
                                        file: String::from("src.rs"),
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
                ctx.all_tagged.push(Box::new(Text {
                    content: fmt.clone(),
                    source: FileSource{
                        file: String::from("src.rs"),
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

        // for unit in &units {
        //     println!("U: {}, {}, {}:{}", unit.unit_type, unit.path[0], unit.source.file, unit.source.line);
        // }
        //
        // for unit in &use_units {
        //     println!("UU: {}, {}, {}:{}", unit.unit_type, unit.path[0], unit.source.file, unit.source.line);
        //
        //     let ref path = unit.path[0];
        //     let res = &units.iter().find(|u| *u.path[0] == path.to_string());
        //
        //     for u in res {
        //         println!("Z: {}", u.path[0]);
        //         self.lexems[unit.source.lexem_iter].content = format!("<a href=\"#l{}\">{}</a>", u.source.line, self.lexems[unit.source.lexem_iter].content);
        //     }
        // }
        //
        // let mut words: HashSet<&str> = HashSet::new();
        // words.insert("struct");
        // words.insert("use");
        // words.insert("fn");
        // words.insert("let");
        // //HashSet<str> = vec!(b"struct", b"main()").iter().collect();
        //
        // {
        //     //let x = &lexems[0];
        //     //println!("F: {} ({:?})", x.content, x.lexem_type);
        // }
        //
        // let output = File::create("test/src.rs.html").unwrap();
        // let mut writer = BufWriter::new(output);
        //
        // let mut out = String::new();
        // out.push_str("<pre>");
        //
        // line_counter = 1;
        // for lexem in &self.lexems {
        //     //println!("G: {} ({:?})", lexem.content, lexem.lexem_type);
        //     let ref fmt = &lexem.content;
        //
        //     match lexem.lexem_type {
        //         LexemType::Newline => {
        //             //fmt = format!("{}<a name=\"l{}\">", fmt, line_counter);
        //             line_counter += 1;
        //         },
        //         _ => {},
        //     }
        //
        //     if words.contains(&fmt[..]) {
        //         //fmt = format!("<b>{}</b>", fmt);
        //     }
        //     out.push_str(&fmt[..]);
        // }
        // out.push_str("</pre>");
        // writer.write(out.as_bytes()).unwrap();
    }
}
