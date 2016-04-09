#[derive(Debug)]
enum LexemType {
    Unknown,
    Whitespace,
    Newline,
    Token,
}

#[derive(Debug)]
enum ParseState {
    Wait,
    KeywordThenName,
    NameThenCall,
}

struct Lexem {
    lexem_type: LexemType,
    content: String,
    start_iter: usize,
}

trait Lexer {
    fn next(&mut self) -> Option<Lexem>;
}

struct CommonLexer {
    buffer: String,
    read_iter: usize,
}

impl CommonLexer {
    fn new(buffer: String) -> CommonLexer {
        CommonLexer {
            buffer: buffer,
            read_iter: 0,
        }
    }
}


struct FileSource {
    file: String,
    line: i32,
    id_iter: usize,
    lexem_iter: usize,
}

struct Unit {
    unit_type: String,
    path: Vec<String>,
    source: FileSource,
}

impl Lexer for CommonLexer {
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

fn main() {
    use std::io::prelude::*;
    use std::io::BufReader;
    use std::io::BufWriter;
    use std::fs::File;
    use std::collections::HashSet;

    let input = File::open("test/src.rs").unwrap();

    let mut reader = BufReader::new(input);
    let mut buffer = String::new();

    reader.read_to_string(&mut buffer).unwrap();

    let mut lexems = vec![];
    let mut lexer = CommonLexer::new(buffer);

    let output = File::create("test/src.rs.html").unwrap();
    let mut writer = BufWriter::new(output);

    lexems.push(Lexem{
        lexem_type: LexemType::Newline,
        content: String::from("\n"),
        start_iter: 0,
    });

    loop {
        if let Some(lexem) = lexer.next() {
            lexems.push(lexem);
        } else {
            break;
        }
    }

    let mut units = vec![];
    let mut use_units = vec![];

    let mut parse_state = ParseState::Wait;

    let mut line_counter = 0;

    let mut lex_iter = 0;

    let mut use_unit_name = String::new();
    let mut use_lex_iter = 0;
    let mut unit_type = String::new();

    loop {
        let ref cur = lexems[lex_iter];
        let ref fmt = cur.content;

        println!("S0: {} ({:?})", fmt, parse_state);


        match cur.lexem_type {
            LexemType::Newline => {
                line_counter += 1;
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
                        units.push(Unit{
                            unit_type: unit_type.clone(),
                            path: unit_path,
                            source: FileSource{
                                file: String::from("src.rs"),
                                line: line_counter,
                                id_iter: cur.start_iter,
                                lexem_iter: lex_iter,
                            }
                        });
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
                            use_units.push(Unit{
                                unit_type: String::from("fn"),
                                path: unit_path,
                                source: FileSource{
                                    file: String::from("src.rs"),
                                    line: line_counter,
                                    id_iter: cur.start_iter,
                                    lexem_iter: use_lex_iter,
                                }
                            });
                            println!("CC: {}, {}", use_lex_iter, lexems[use_lex_iter].content);
                        } else {
                            parse_state = ParseState::Wait;
                            use_unit_name = String::new();
                        }
                    },
                    _ => {},
                }
            },
        }

        println!("  -> ({:?})", parse_state);

        if lex_iter == lexems.len() - 1 {
            break;
        }

        lex_iter += 1;
    }

    for unit in &units {
        println!("U: {}, {}, {}:{}", unit.unit_type, unit.path[0], unit.source.file, unit.source.line);
    }

    for unit in &use_units {
        println!("UU: {}, {}, {}:{}", unit.unit_type, unit.path[0], unit.source.file, unit.source.line);

        let ref path = unit.path[0];
        let res = &units.iter().find(|u| *u.path[0] == path.to_string());

        for u in res {
            println!("Z: {}", u.path[0]);
            lexems[unit.source.lexem_iter].content = format!("<a href=\"#l{}\">{}</a>", u.source.line, lexems[unit.source.lexem_iter].content);
        }
    }

    let mut words: HashSet<&str> = HashSet::new();
    words.insert("struct");
    words.insert("use");
    words.insert("fn");
    words.insert("let");
    //HashSet<str> = vec!(b"struct", b"main()").iter().collect();

    {
        //let x = &lexems[0];
        //println!("F: {} ({:?})", x.content, x.lexem_type);
    }

    let mut out = String::new();
    out.push_str("<pre>");

    line_counter = 1;
    for lexem in lexems {
        //println!("G: {} ({:?})", lexem.content, lexem.lexem_type);
        let mut fmt = lexem.content;

        match lexem.lexem_type {
            LexemType::Newline => {
                fmt = format!("{}<a name=\"l{}\">", fmt, line_counter);
                line_counter += 1;
            },
            _ => {},
        }

        if words.contains(&fmt[..]) {
            fmt = format!("<b>{}</b>", fmt);
        }
        out.push_str(&fmt[..]);
    }
    out.push_str("</pre>");
    writer.write(out.as_bytes()).unwrap();
}
