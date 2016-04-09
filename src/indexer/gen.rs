use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::fs::File;
use std::collections::HashSet;
use std::collections::LinkedList;

use indexer::storage::Storage;
use indexer::storage::FileSource;

pub trait HtmlItem {
    fn render(&self) -> String;
}

pub struct Plain {
    pub content: String,
}

impl HtmlItem for Plain {
    fn render(&self) -> String {
        self.content.clone()
    }
}

pub struct Newline {
    pub this_line: i32,
}

impl HtmlItem for Newline {
    fn render(&self) -> String {
        format!("\n<a name=\"l{}\">", self.this_line)
    }
}

pub struct Reference {
    pub content: String,
    pub source: FileSource,
}

impl HtmlItem for Reference {
    fn render(&self) -> String {
        format!("<a href=\"#l{}\">{}</a>", self.source.line, self.content)
    }
}

pub fn to_file(filename: String, items: Vec<Box<HtmlItem>>, storage: &Storage) {
    let output = File::create(filename).unwrap();
    let mut writer = BufWriter::new(output);

    let mut out = String::new();
    out.push_str("<pre>");

    let mut line_counter = 1;
    for item in &items {
        //println!("G: {} ({:?})", lexem.content, lexem.lexem_type);

        let fmt = item.render();

        // match lexem.lexem_type {
        //     LexemType::Newline => {
        //         //fmt = format!("{}<a name=\"l{}\">", fmt, line_counter);
        //         line_counter += 1;
        //     },
        //     _ => {},
        // }
        //
        // if words.contains(&fmt[..]) {
        //     //fmt = format!("<b>{}</b>", fmt);
        // }
        out.push_str(&fmt[..]);
    }
    out.push_str("</pre>");
    writer.write(out.as_bytes()).unwrap();
}
