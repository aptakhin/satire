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
}

struct Lexem {
    lexem_type: LexemType,
    content: String,
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
        content: String::from("\n")
    });

    loop {
        if let Some(lexem) = lexer.next() {
            lexems.push(lexem);
        } else {
            break;
        }
    }

    let mut units = vec![];

    let mut parse_state = ParseState::Wait;

    let mut line_counter = 0;

    let mut lex_iter = 0;

    let mut unit_type = String::new();

    loop {
        let ref cur = lexems[lex_iter];
        let ref fmt = cur.content;

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
                            }
                        });
                        parse_state = ParseState::Wait;
                    },
                    _ => {},
                }
            },
        }

        if lex_iter == lexems.len() - 1 {
            break;
        }

        lex_iter += 1;
    }

    for unit in units {
        println!("U: {}, {}, {}:{}", unit.unit_type, unit.path[0], unit.source.file, unit.source.line);
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
                fmt = format!("{}<a name=\"line{}\">", fmt, line_counter);
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
