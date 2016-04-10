use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use indexer::parser::Tagged;
use indexer::storage::FileSource;

#[derive(Debug)]
pub enum Style {
    Normal,
    Bold,
}

#[derive(Debug)]
pub enum HtmlItem {
    Plain { content: String, style: Style },
    Newline { this_line: i32 },
    Reference { path: Vec<String>, defs: Vec<FileSource> }
}

fn render_html(item: &HtmlItem) -> String {
    match item {
        &HtmlItem::Plain { ref content, ref style } => {
            match style {
                &Style::Normal => content.clone(),
                &Style::Bold => format!("<b>{}</b>", content),
            }
        },
        &HtmlItem::Newline { this_line } => {
            if this_line == 1 {
                format!("<a name=\"l{}\">", this_line)
            } else {
                format!("\n<a name=\"l{}\">", this_line)
            }
        },
        &HtmlItem::Reference { ref path, ref defs } => {
            path[0].clone()
        },
    }
}

pub fn to_html_tag(tagged: &Tagged) -> HtmlItem {
    match tagged {
        &Tagged::Definition { ref unit_type, ref path, ref source } => {
            HtmlItem::Plain{ content: path[0].clone(), style: Style::Normal }
        },
        &Tagged::Calling { ref unit_type, ref path, ref source, ref defs } => {
            HtmlItem::Reference{ path: path.clone(), defs: defs.clone() }
        },
        &Tagged::Newline { ref source } => {
            HtmlItem::Newline{ this_line: source.line }
        },
        &Tagged::Keyword { ref content, ref source } => {
            HtmlItem::Plain{ content: content.clone(), style: Style::Bold }
        },
        &Tagged::Text { ref content, ref source } => {
            HtmlItem::Plain{ content: content.clone(), style: Style::Normal }
        },
    }
}

pub fn to_html(tagged: &Vec<Box<Tagged>>) -> Vec<Box<HtmlItem>> {
    let mut items = vec![];

    for tag in tagged.iter() {
        println!("{:?}", tag);
        let item = Box::new(to_html_tag(tag));
        items.push(item);
    }

    items
}

pub fn to_file(filename: String, items: &Vec<Box<HtmlItem>>) {
    let output = File::create(filename).unwrap();
    let mut writer = BufWriter::new(output);

    println!("---------------------------");
    let mut out = String::new();
    out.push_str("<pre>");

    let mut line_counter = 1;
    for item in items.iter() {
        let fmt = render_html(item);
        println!("{:?}| {}", item, &fmt[..]);
        out.push_str(&fmt[..]);
    }
    out.push_str("</pre>");
    writer.write(out.as_bytes()).unwrap();
}
