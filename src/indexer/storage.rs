use std::io::prelude::*;
use std::io::BufReader;
use std::fmt;
use std::io;
use std::fs::{self, DirEntry, File};
use std::path::{Path, PathBuf};
use std::io::BufWriter;

use indexer::parser::{Tagged, CommonParser};
use indexer::lexer::Span;
use indexer::gen;

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
    pub set: Vec<&'a PreparsedFile>,
}

impl<'a> Index<'a> {
    pub fn new() -> Index<'a> {
        Index {
            set: vec![],
        }
    }

    pub fn add(&mut self, preparsed: &'a PreparsedFile) {
        self.set.push(preparsed);
    }

    pub fn find(&self, path: &str) -> Vec<FileSource> {
        let mut found = vec![];

        for preparsed in &self.set {
            found.append(&mut preparsed.find(path));
        }

        found
    }

    // pub fn build(&mut self) -> Index {
    //     Index {
    //         set: self.set,
    //     }
    // }
}

pub struct ParsedFile {
    pub file: String,
    pub content: String,
    pub preparsed: PreparsedFile,
    //pub ctx: Context,
}

pub struct IndexBuilder {
    //pub index: Index<'a>,
    pub set: Vec<ParsedFile>,
}

impl IndexBuilder {
    pub fn new() -> IndexBuilder {
        IndexBuilder {
            set: vec![]
        }
    }

    pub fn build_dir(&mut self, root_dir: &str) {
        self.add_dir_rec(root_dir, root_dir);
    }

    pub fn deduce(&self) -> Vec<DeducedFile> {
        let mut index = Index::new();
        for parsed_file in &self.set {
            index.add(&parsed_file.preparsed);
        }

        let mut res = vec![];
        for parsed_file in &self.set {
            let deduced_file = parsed_file.preparsed.deduce(&index);
            res.push(deduced_file);
        }

        res
    }

    pub fn gen(&self) {
        let mut index = Index::new();
        for parsed_file in &self.set {
            index.add(&parsed_file.preparsed);
        }

        let mut deduced = vec![];
        for parsed_file in &self.set {
            let deduced_file = parsed_file.preparsed.deduce(&index);
            deduced.push(deduced_file);
        }

        for i in 0..self.set.len() {
            let generated = deduced[i].gen();

            //let template = mustache::compile_path("web/code_template.html").unwrap();
            let mut template_file = File::open("web/code_template.html").unwrap();
            let mut template = String::new();
            template_file.read_to_string(&mut template);

            let code = gen::to_string(&self.set[i].content, &generated[..]);
            template = template.replace("{{code}}", &code);

            let output = File::create(format!("web/{}.html", self.set[i].file)).unwrap();
            let mut writer = BufWriter::new(output);
            //let out = to_string(content, items);
            //writer.write(out.as_bytes()).unwrap();

            //template.render_data(&mut writer, &template);
            writer.write(template.as_bytes()).unwrap();

            //gen::to_file(format!("{}.html", self.set[i].file), &self.set[i].content, &generated[..]);
        }
    }

    pub fn add_dir_rec(&mut self, dir: &str, root_dir: &str) -> io::Result<()> {
        if try!(fs::metadata(dir)).is_dir() {
            for entry in try!(fs::read_dir(dir)) {
                let entry = try!(entry);
                if try!(fs::metadata(entry.path())).is_dir() {
                    try!(self.add_dir_rec(&entry.path().to_str().unwrap(), root_dir));
                } else {
                    try!(self.add_file(&entry.path(), root_dir));
                }
            }
        }
        Ok(())
    }

    pub fn add_file(&mut self, filepath: &PathBuf, root_dir: &str) -> io::Result<()> {
        let file = filepath.to_str().unwrap();
        println!("F: {}", file);

        let mut handle = true;

        if let Some(file_ext) = filepath.extension() {
            if file_ext != "rs" {
                handle = false;
            }
        }

        if handle {
            let source = SourceFile::new(file.to_string().clone());
            let mut parser = CommonParser::new(file[root_dir.len()..].to_string(), source.content.clone());
            let preparsed = parser.parse();
            //println!("  f: {}", ctx.pars);
            self.set.push(ParsedFile{
                file: file.to_string().clone(),
                content: source.content.clone(),
                preparsed: preparsed,
            });

        }

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


pub struct DeducedFile {
    pub file: String,
    //pub content: &'a str,
    pub synt: Vec<(Tagged, Span, Option<Box<Info>>)>,
    pub pars: Vec<(Tagged, Span, Option<Box<Info>>)>,
}

impl DeducedFile {
    pub fn new(file: String, synt: Vec<(Tagged, Span, Option<Box<Info>>)>, pars: Vec<(Tagged, Span, Option<Box<Info>>)>) -> DeducedFile {
        DeducedFile {
            file: file,
            //content: content,
            synt: synt,
            pars: pars,
        }
    }

    pub fn gen(&self) -> Vec<(Tagged, Span, Option<Box<Info>>)> {
        let mut merged = vec![];
        let mut a = 0;
        let mut b = 0;

        let ref pars = self.pars;
        let ref synt = self.synt;

        println!("T: {:?}, {:?}", pars, synt);

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

pub struct PreparsedFile {
    pub file: String,
    //pub content: &'a str,
    pub syntax: Vec<(Tagged, Span)>,
    pub parsed: Vec<(Tagged, Span)>,
}

impl PreparsedFile {
    pub fn new(file: String, syntax: Vec<(Tagged, Span)>, parsed: Vec<(Tagged, Span)>) -> PreparsedFile {
        PreparsedFile {
            file: file,
            //content: content,
            syntax: syntax,
            parsed: parsed,
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

    pub fn deduce(&self, index: &Index) -> DeducedFile {
        let mut pars: Vec<(Tagged, Span, Option<Box<Info>>)> = vec![];
        let mut synt: Vec<(Tagged, Span, Option<Box<Info>>)> = vec![];

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

            pars.push((tagged.clone(), span.clone(), info));
            //println!("E: {}", pars.len());
        }
        pars.push((Tagged::Eof, Span::end(), None));

        for i in 0..self.syntax.len() {
            let &(ref tagged, ref span) = &self.syntax[i];

            synt.push((tagged.clone(), span.clone(), None));
        }
        synt.push((Tagged::Eof, Span::end(), None));

        DeducedFile::new(self.file.clone(), pars, synt)
    }
}
//
// impl Context {
//     pub fn new(file: String, syntax: Vec<(Tagged, Span)>, parsed: Vec<(Tagged, Span)>) -> Context {
//         Context {
//             file: file,
//             syntax: syntax,
//             parsed: parsed,
//             synt: vec![],
//             pars: vec![],
//         }
//     }
//
//     pub fn merge(&mut self, mut ctx: Context) {
//         //self.all_tagged.append(&mut ctx.all_tagged);
//     }
//
//     pub fn find(&self, path: &str) -> Vec<FileSource> {
//         let mut found = vec![];
//         for &(ref tagged, ref span) in &self.parsed {
//             //println!("  l: {:?}", tagged);
//             match tagged {
//                 &Tagged::Definition(ref name) if name == path => {
//                     found.push(FileSource{
//                         file: self.file.clone(),
//                         line: span.line,
//                     })
//                 },
//                 _ => {},
//             }
//         }
//         found
//     }
//
//     pub fn find_with_index(&self, path: &str, index: &Index) -> Vec<FileSource> {
//         let mut found = index.find(path);
//         found.append(&mut self.find(path));
//         found
//     }
//
//     pub fn deduce(&mut self, index: &Index) {
//         for &(ref tagged, ref span) in &self.parsed {
//             let mut info = None;
//             //println!("QQQ: {:?} {:?}", tagged, span);
//
//             match tagged {
//                 &Tagged::Calling(ref name) => {
//                     let refs = self.find_with_index(&name, index);
//                     if refs.len() > 0 {
//                         //println!("  c: {:?} {:?}", tagged, span);
//                         //println!("  f: {:?} {:?}", ftagged, fspan);
//
//                         info = Some(Box::new(Info{
//                             refs: refs,
//                         }));
//                     }
//                 },
//                 _ => {},
//             }
//
//             self.pars.push((tagged.clone(), span.clone(), info));
//             //println!("E: {}", pars.len());
//         }
//         self.pars.push((Tagged::Eof, Span::end(), None));
//
//         let mut synt: Vec<(Tagged, Span, Option<Box<Info>>)> = vec![];
//         for i in 0..self.syntax.len() {
//             let &(ref tagged, ref span) = &self.syntax[i];
//
//             self.synt.push((tagged.clone(), span.clone(), None));
//         }
//         self.synt.push((Tagged::Eof, Span::end(), None));
//     }
// }

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
