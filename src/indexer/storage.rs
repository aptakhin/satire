use std::io::prelude::*;
use std::io::BufReader;
use std::fmt;
use std::io;
use std::fs::{self, DirEntry, File};
use std::path::Path;

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

#[derive(Clone, Debug)]
pub struct Info {
    pub refs: Vec<FileSource>,
}

impl fmt::Debug for FileSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.line)
    }
}

pub struct Index<'a> {
    pub set: Vec<&'a mut Context>,
}

impl<'a> Index<'a> {
    pub fn new() -> Index<'a> {
        Index {
            set: vec![],
        }
    }

    pub fn add(&mut self, ctx: &'a mut Context) {
        self.set.push(ctx);
    }

    pub fn find(&self, path: &str) -> Vec<FileSource> {
        let mut found = vec![];

        for ctx in &self.set {
            found.append(&mut ctx.find(path));
        }

        found
    }

    // pub fn build(&mut self) -> Index {
    //     Index {
    //         set: self.set,
    //     }
    // }
}

struct IndexBuilder<'a> {
    pub index: Index<'a>,
}

impl<'a> IndexBuilder<'a> {
    pub fn new() -> IndexBuilder<'a> {
        IndexBuilder {
            index: Index::new(),
        }
    }

    pub fn build_dir(&mut self, root_dir: &str) -> io::Result<()> {
        self.add_dir_rec(root_dir)
    }

    pub fn add_dir_rec(&mut self, dir: &str) -> io::Result<()> {
        if try!(fs::metadata(dir)).is_dir() {
            for entry in try!(fs::read_dir(dir)) {
                let entry = try!(entry);
                if try!(fs::metadata(entry.path())).is_dir() {
                    try!(self.add_dir_rec(&entry.path().to_str().unwrap()));
                } else {
                    try!(self.add_file(&entry.path().to_str().unwrap()));
                }
            }
        }
        Ok(())
    }

    pub fn add_file(&mut self, filepath: &str) -> io::Result<()> {
        Ok(())
    }
}

pub struct Context {
    pub file: String,
    pub syntax: Vec<(Tagged, Span)>,
    pub parsed: Vec<(Tagged, Span)>,
    pub synt: Vec<(Tagged, Span, Option<Box<Info>>)>,
    pub pars: Vec<(Tagged, Span, Option<Box<Info>>)>,
}

impl Context {
    pub fn new(file: String, syntax: Vec<(Tagged, Span)>, parsed: Vec<(Tagged, Span)>) -> Context {
        Context {
            file: file,
            syntax: syntax,
            parsed: parsed,
            synt: vec![],
            pars: vec![],
        }
    }

    pub fn merge(&mut self, mut ctx: Context) {
        //self.all_tagged.append(&mut ctx.all_tagged);
    }

    pub fn find(&self, path: &str) -> Vec<FileSource> {
        let mut found = vec![];
        for &(ref tagged, ref span) in &self.parsed {
            //println!("  l: {:?}", tagged);
            match tagged {
                &Tagged::Definition(ref name) if name == path => {
                    found.push(FileSource{
                        file: self.file.clone(),
                        line: span.line,
                    })
                },
                _ => {},
            }
        }
        found
    }

    pub fn find_with_index(&self, path: &str, index: &Index) -> Vec<FileSource> {
        let mut found = index.find(path);
        found.append(&mut self.find(path));
        found
    }

    pub fn deduce(&mut self, index: &Index) {
        for &(ref tagged, ref span) in &self.parsed {
            let mut info = None;
            //println!("QQQ: {:?} {:?}", tagged, span);

            match tagged {
                &Tagged::Calling(ref name) => {
                    let refs = self.find_with_index(&name, index);
                    if refs.len() > 0 {
                        //println!("  c: {:?} {:?}", tagged, span);
                        //println!("  f: {:?} {:?}", ftagged, fspan);

                        info = Some(Box::new(Info{
                            refs: refs,
                        }));
                    }
                },
                _ => {},
            }

            self.pars.push((tagged.clone(), span.clone(), info));
            //println!("E: {}", pars.len());
        }
        self.pars.push((Tagged::Eof, Span::end(), None));

        let mut synt: Vec<(Tagged, Span, Option<Box<Info>>)> = vec![];
        for i in 0..self.syntax.len() {
            let &(ref tagged, ref span) = &self.syntax[i];

            self.synt.push((tagged.clone(), span.clone(), None));
        }
        self.synt.push((Tagged::Eof, Span::end(), None));
    }


    pub fn gen(&self) -> Vec<(Tagged, Span, Option<Box<Info>>)> {
        let mut merged = vec![];
        let mut a = 0;
        let mut b = 0;

        let ref pars = self.pars;
        let ref synt = self.synt;

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
