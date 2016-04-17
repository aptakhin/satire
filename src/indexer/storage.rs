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

impl fmt::Debug for FileSource {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.line)
    }
}

pub struct Context {
    pub syntax: Vec<(Tagged, Span)>,
}

impl Context {
    pub fn new(syntax: Vec<(Tagged, Span)>) -> Context {
        Context {
            syntax: syntax,
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
    //
    // pub fn gen(&self) {
    //     for tag in self.all_tagged.iter() {
    //         let tagged: &Tagged = &tag;
    //         match tagged {
    //             &Tagged::Calling { ref unit_type, ref path, ref source, ref defs } => {
    //                 //println!("FC: {:?}", tagged);
    //                 if let Some(p) = self.find(&path[0]) {
    //                     println!("Matched!");
    //                 }
    //             },
    //             _ => {},
    //         }
    //     }
    // }
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
