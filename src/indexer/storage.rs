use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::fmt;

use indexer::parser::Tagged;
use indexer::lexer::Span;

pub struct SourceFile {
    pub filename: String,
    pub content: String,
}

impl SourceFile {
    pub fn new(filename: String) -> SourceFile {
        let input = File::open(&filename).unwrap();
        let mut reader = BufReader::new(input);
        let mut content = String::new();
        reader.read_to_string(&mut content).unwrap();

        SourceFile {
            filename: filename.clone(),
            content: content,
        }
    }
}

#[derive(Clone)]
pub struct FileSource {
    pub file: String,
    pub line: usize,
}

impl FileSource {
    pub fn render_html(&self, name: &str) -> String {
        format!("<a href=\"#l{}\">{}</a>", self.line, name)
    }
}

#[derive(Clone, Debug)]
pub struct Info {
    pub dst: FileSource,
}

impl fmt::Debug for FileSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.line)
    }
}

pub struct Context {
    pub syntax: Vec<(Tagged, Span)>,
    pub parsed: Vec<(Tagged, Span)>,
}

impl Context {
    pub fn new(syntax: Vec<(Tagged, Span)>, parsed: Vec<(Tagged, Span)>) -> Context {
        Context {
            syntax: syntax,
            parsed: parsed,
        }
    }

    pub fn merge(&mut self, mut ctx: Context) {
        //self.all_tagged.append(&mut ctx.all_tagged);
    }

    pub fn find(&self, path: &str) -> Option<(&Tagged, &Span)> {
        let mut found = None;
        for &(ref tagged, ref span) in &self.parsed {
            //println!("  l: {:?}", tagged);
            match tagged {
                &Tagged::Definition(ref name) if name == path => {
                    found = Some((tagged, span));
                    break;
                },
                _ => {},
            }
        }
        found
    }

    pub fn gen(&self) -> Vec<(Tagged, Span, Option<Box<Info>>)> {
        let mut pars: Vec<(Tagged, Span, Option<Box<Info>>)> = vec![];

        for &(ref tagged, ref span) in &self.parsed {
            let mut info = None;
            //println!("QQQ: {:?} {:?}", tagged, span);

            match tagged {
                &Tagged::Calling(ref name) => {
                    if let Some((ftagged, fspan)) = self.find(&name) {
                        //println!("  c: {:?} {:?}", tagged, span);
                        //println!("  f: {:?} {:?}", ftagged, fspan);

                        let file_source = FileSource {
                            file: "a.rs".to_string(),
                            line: fspan.line,
                        };
                        info = Some(Box::new(Info{
                            dst: file_source,
                        }));
                    }
                },
                _ => {},
            }

            pars.push((tagged.clone(), span.clone(), info));
            //println!("E: {}", pars.len());
        }
        pars.push((Tagged::Eof, Span::end(), None));
        //println!("D: {}", pars.len());

        let mut synt: Vec<(Tagged, Span, Option<Box<Info>>)> = vec![];
        for i in 0..self.syntax.len() {
            let &(ref tagged, ref span) = &self.syntax[i];

            synt.push((tagged.clone(), span.clone(), None));
        }
        synt.push((Tagged::Eof, Span::end(), None));

        let mut merged = vec![];
        let mut a = 0;
        let mut b = 0;

        while a < pars.len() || b < synt.len() {
            //println!("s: {}/{} {}/{}", a, pars.len(), b, synt.len());
            if a < pars.len() {
                //println!("  a: {}/{:?}", a, pars[a]);
            }
            if b < synt.len() {
                //println!("  b: {}/{:?}", b, synt[b]);
            }
            if a < pars.len() && b >= synt.len() || a < pars.len() && b < synt.len() && pars[a].1.lo <= synt[b].1.lo {
                let push = pars[a].clone();
                match push.0 {
                    Tagged::Eof => {},
                    _ => { merged.push(push) },
                }
                a += 1;
            } else { //if b < synt.len() && a >= pars.len() || a < pars.len() && b < synt.len() && pars[a].1.lo > synt[b].1.lo {
                let push = synt[b].clone();
                match push.0 {
                    Tagged::Eof => {},
                    _ => { merged.push(push) },
                }
                b += 1;
            }
        }
        println!("M: {:?}/{}", merged, merged.len());

        merged
    }
}

// pub struct Storage {
//     pub ctx: Context,
// }
//
// impl Storage {
//     pub fn new() -> Storage {
//         Storage {
//             ctx: Context::new(),
//         }
//     }
//
//     pub fn merge(&mut self, mut merge_ctx: Context) {
//         self.ctx.merge(merge_ctx);
//     }
//
//     pub fn gen(&mut self) {
//         //self.ctx.gen();
//     }
// }
