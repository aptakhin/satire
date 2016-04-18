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

pub struct Info {

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

    // pub fn find(&self, path: &String) -> Option<&Tagged> {
    //     let mut found = None;
    //     for tag in self.all_tagged.iter() {
    //         let tagged: &Tagged = &tag;
    //         //println!("  l: {:?}", tagged);
    //         match tagged {
    //             &Tagged::Definition { ref unit_type, ref path, ref source } => {
    //                 found = Some(tagged);
    //                 break
    //             },
    //             _ => {},
    //         }
    //     }
    //     //println!("  R: {:?}", found);
    //     found
    // }

    pub fn gen(&self) -> Vec<(Tagged, Span, Option<Box<Info>>)> {
        let mut res = vec![];

        for i in 0..self.syntax.len() {
            let &(ref tagged, ref span) = &self.syntax[i];
            res.push((tagged.clone(), span.clone(), None));
        }

        res
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
