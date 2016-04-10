use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use indexer::storage::Tagged;
use indexer::storage::FileSource;

pub trait HtmlItem {
    fn render_html(&self) -> String;
}

pub struct Plain {
    pub content: String,
}

impl HtmlItem for Plain {
    fn render_html(&self) -> String {
        self.content.clone()
    }
}

pub struct Newline {
    pub this_line: i32,
}

impl HtmlItem for Newline {
    fn render_html(&self) -> String {
        if self.this_line == 1 {
            format!("<a name=\"l{}\">", self.this_line)
        } else {
            format!("\n<a name=\"l{}\">", self.this_line)
        }
    }
}

pub struct Reference {
    pub content: String,
    pub defs: Vec<FileSource>,
}

impl HtmlItem for Reference {
    fn render_html(&self) -> String {
        if self.defs.len() > 0 {
            self.defs[0].render_html(&self.content[..])
        } else {
            self.content.clone()
        }
    }
}

pub trait Tagged2Html {
    fn tagged2html(&self) -> Box<HtmlItem>;
}

// impl Tagged2Html for Box<Tagged> {
//     fn tagged2html(&self) -> Box<HtmlItem> {
//         self.tagged2html()
//         // Box::new(Plain{
//         //     content: String::from(self.get_content()),
//         // })
//     }
// }

pub fn to_html(tagged: &Vec<Box<Tagged>>) -> Vec<Box<HtmlItem>> {
    let mut items = vec![];

    for tag in tagged.iter() {
        let mut item: Box<HtmlItem>;

        item = Box::new(Plain {
            content: tag.render_html(),
        });
        println!("A: {}", tag.render_html());
        items.push(item);
    }

    items
}

pub fn to_file(filename: String, items: &Vec<Box<HtmlItem>>) {
    let output = File::create(filename).unwrap();
    let mut writer = BufWriter::new(output);

    let mut out = String::new();
    out.push_str("<pre>");

    let mut line_counter = 1;
    for item in items.iter() {
        let fmt = item.render_html().clone();
        out.push_str(&fmt[..]);
    }
    out.push_str("</pre>");
    writer.write(out.as_bytes()).unwrap();
}
