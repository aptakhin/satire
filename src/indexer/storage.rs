use std::io::prelude::*;
use std::io::BufReader;
use std::fmt;
use std::io;
use std::fs::{self, DirEntry, File};
use std::path::{Path, PathBuf};
use std::io::BufWriter;
use std::rc::Rc;

use std::collections::HashMap;

use indexer::parser;
use indexer::parser::{Tagged, CommonParser};
use indexer::lexer::Span;
use indexer::gen;

pub struct SourceFile {
    pub filename: String,
    pub content: Rc<String>,
}

impl SourceFile {
    pub fn new(filename: String) -> SourceFile {
        let input = File::open(&filename).unwrap();
        let mut reader = BufReader::new(input);
        let mut content = String::new();
        reader.read_to_string(&mut content).unwrap();

        SourceFile {
            filename: filename.clone(),
            content: Rc::new(content),
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
        write!(f, "{}:{}", self.file, self.line)
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

    pub fn find(&self, path: &parser::Path) -> Vec<FileSource> {
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
    pub content: Rc<String>,
    pub preparsed: PreparsedFile,
    //pub ctx: Context,
}

pub struct IndexBuilder {
    //pub index: Index<'a>,
    pub set: Vec<ParsedFile>,
    pub dir_files: HashMap<String, Vec<PathBuf>>,
}

impl IndexBuilder {
    pub fn new() -> IndexBuilder {
        IndexBuilder {
            set: vec![],
            dir_files: HashMap::new(),
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

        for i in 0..deduced.len() {
            let generated = deduced[i].gen();

            //let template = mustache::compile_path("web/code_template.html").unwrap();
            let mut template_file = File::open("web/code_template.html").unwrap();
            let mut template = String::new();
            template_file.read_to_string(&mut template);

            let filepath = Path::new(&self.set[i].file);
            let mut tree = String::new();

            if let Some(parent) = filepath.parent() {
                //println!("FF: {}", parent.to_str().unwrap());
                tree.push_str("<ul>");
                if let Some(value) = self.dir_files.get(parent.to_str().unwrap()) {
                    for i in value {
                        //tree.push_str(i.to_str().unwrap());
                        let path = i.to_str().unwrap();
                        tree.push_str(&format!("<li><a href=\"/{}.html\">{}</a></li>", path, path));
                    }
                }
                tree.push_str("</ul>");
            }

            template = template.replace("{{tree}}", &tree);

            let code = gen::to_string(deduced[i].content.clone(), &generated[..]);
            template = template.replace("{{code}}", &code);

            let title = format!("{}", self.set[i].file);
            template = template.replace("{{title}}", &title);

            let dir = format!("web/{}", Path::new(&self.set[i].file).parent().unwrap().to_str().unwrap());
            //println!("Create dir: {}", dir);
            let res = fs::create_dir_all(dir);
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
        let mut files = vec![];
        if try!(fs::metadata(dir)).is_dir() {
            for entry in try!(fs::read_dir(dir)) {
                let entry = try!(entry);
                if try!(fs::metadata(entry.path())).is_dir() {
                    try!(self.add_dir_rec(&entry.path().to_str().unwrap(), root_dir));
                } else {
                    files.push(entry.path());
                    try!(self.add_file(&entry.path(), root_dir));
                }
            }
        }

        let add_dir = &dir[..dir.len() - 1];
        //println!("AA: {}", add_dir);
        self.dir_files.insert(add_dir.to_string(), files);

        Ok(())
    }

    pub fn add_file(&mut self, filepath: &PathBuf, root_dir: &str) -> io::Result<()> {
        let file = filepath.to_str().unwrap();
        //println!("F: {}", file);

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
    pub content: Rc<String>,
    pub synt: Vec<(Tagged, Span, Option<Box<Info>>)>,
    pub pars: Vec<(Tagged, Span, Option<Box<Info>>)>,
}

impl DeducedFile {
    pub fn new(file: String, content: Rc<String>, synt: Vec<(Tagged, Span, Option<Box<Info>>)>, pars: Vec<(Tagged, Span, Option<Box<Info>>)>) -> DeducedFile {
        DeducedFile {
            file: file,
            content: content,
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

        //println!("Merged: {}", merged.len());
        // for i in &merged {
        //     println!("  {:?}", i);
        // }

        merged
    }
}

pub struct PreparsedFile {
    pub file: String,
    pub content: Rc<String>,
    pub syntax: Vec<(Tagged, Span)>,
    pub parsed: Vec<(Tagged, Span)>,
}

impl PreparsedFile {
    pub fn new(file: String, content: Rc<String>, syntax: Vec<(Tagged, Span)>, parsed: Vec<(Tagged, Span)>) -> PreparsedFile {
        PreparsedFile {
            file: file,
            content: content,
            syntax: syntax,
            parsed: parsed,
        }
    }

    pub fn merge(&mut self, mut ctx: Context) {
        //self.all_tagged.append(&mut ctx.all_tagged);
    }

    pub fn find(&self, path: &parser::Path) -> Vec<FileSource> {
        let mut found = vec![];
        for &(ref tagged, ref span) in &self.parsed {

            match tagged {
                &Tagged::Definition(ref use_context) => {
                    //println!("  l: {:?} {:?} {:?}", tagged, &use_context.reference, path);
                    if use_context.reference == *path {
                        //println!("    matched: {:?}", tagged);
                        found.push(FileSource{
                            file: self.file.clone(),
                            line: span.line,
                        })
                    }
                },
                _ => {},
            }
        }
        found
    }

    pub fn deduce(&self, index: &Index) -> DeducedFile {
        let mut pars: Vec<(Tagged, Span, Option<Box<Info>>)> = vec![];
        let mut synt: Vec<(Tagged, Span, Option<Box<Info>>)> = vec![];

        for &(ref tagged, ref span) in &self.parsed {
            let mut info = None;


            match tagged {
                &Tagged::Calling(ref use_context) => {
                    //println!("QQQ: {:?} {:?}", tagged, span);
                    let refs = index.find(&use_context.reference);
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

        DeducedFile::new(self.file.clone(), self.content.clone(), pars, synt)
    }
}
