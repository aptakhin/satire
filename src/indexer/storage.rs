use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;
use std::fmt;

use indexer::parser::Tagged;

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

#[derive(Clone, Copy)]
pub struct FileSource {
    //pub file: String,
    pub line: i32,
    pub id_iter: usize,
    pub lexem_iter: usize,
}

impl FileSource {
    pub fn render_html(&self, name: &str) -> String {
        format!("<a href=\"#l{}\">{}</a>", self.line, name)
    }
}

impl fmt::Debug for FileSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.line)
    }
}

pub struct Context {
    pub all_tagged: Vec<Box<Tagged>>,
}

impl Context {
    pub fn new() -> Context {
        Context {
            all_tagged: vec![],
        }
    }

    pub fn merge(&mut self, mut ctx: Context) {
        self.all_tagged.append(&mut ctx.all_tagged);
    }

    pub fn gen(&mut self) {

    }
}

pub struct Storage {
    pub ctx: Context,
}

impl Storage {
    pub fn new() -> Storage {
        Storage {
            ctx: Context::new(),
        }
    }

    pub fn merge(&mut self, mut merge_ctx: Context) {
        self.ctx.merge(merge_ctx);
    }

    pub fn gen(&mut self) {
        self.ctx.gen();
    }
}
