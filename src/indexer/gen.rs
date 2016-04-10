use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use indexer::parser::Tagged;
use indexer::storage::FileSource;

pub enum Style {
    Normal,
    Bold,
}

pub enum HtmlItem {
    Plain { content: String, style: Style },
    Newline { this_line: i32 },
    Reference { path: Vec<String>, defs: Vec<FileSource> }
}

fn render_html(item: &HtmlItem) -> String {
    match item {
        &HtmlItem::Plain { content: ref content, style: ref style } => {
            match style {
                &Style::Normal => content.clone(),
                &Style::Bold => format!("<b>{}</b>", content),
            }
        },
        &HtmlItem::Newline { this_line: this_line } => {
            if this_line == 1 {
                format!("<a name=\"l{}\">", this_line)
            } else {
                format!("\n<a name=\"l{}\">", this_line)
            }
        },
        &HtmlItem::Reference { path: ref path, defs: ref defs } => {
            path[0].clone()
        },
    }
}

pub fn to_html_tag(tagged: &Tagged) -> HtmlItem {
    match tagged {
        &Tagged::Definition { unit_type: ref unit_type, path: ref path, source: ref source } => {
            HtmlItem::Plain{ content: path[0].clone(), style: Style::Normal }
        },
        &Tagged::Calling { unit_type: ref unit_type, path: ref path, source: ref source, defs: ref defs } => {
            HtmlItem::Reference{ path: path.clone(), defs: defs.clone() }
        },
        &Tagged::Newline { source: ref source } => {
            HtmlItem::Newline{ this_line: source.line }
        },
        &Tagged::Keyword { content: ref content, source: ref source } => {
            HtmlItem::Plain{ content: content.clone(), style: Style::Bold }
        },
        &Tagged::Text { content: ref content, source: ref source } => {
            HtmlItem::Plain{ content: content.clone(), style: Style::Normal }
        },
    }
}

pub fn to_html(tagged: &Vec<Box<Tagged>>) -> Vec<Box<HtmlItem>> {
    let mut items = vec![];

    for tag in tagged.iter() {
        let item = Box::new(to_html_tag(tag));
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
        let fmt = render_html(item);
        out.push_str(&fmt[..]);
    }
    out.push_str("</pre>");
    writer.write(out.as_bytes()).unwrap();
}
