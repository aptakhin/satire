use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;

use indexer::parser::Tagged;
use indexer::lexer::Span;
use indexer::lexer::WhitespaceType;
use indexer::storage::FileSource;
use indexer::storage::Info;

pub fn to_file(filename: String, content: &str, items: &[(Tagged, Span, Option<Box<Info>>)]) {
    let output = File::create(filename).unwrap();
    let mut writer = BufWriter::new(output);

    //println!("---------------------------");
    let mut out = String::new();
    out.push_str("<pre>");

    let mut till = 0;
    for &(ref tagged, ref span, ref info) in items {
        out.push_str(&content[till..span.lo]);
        let fmt;
        let cnt = &content[span.lo..span.hi];

        match tagged {
            &Tagged::Keyword(ref kw) => {
                fmt = format!("<b>{}</b>", &cnt)
            },
            &Tagged::Comment => {
                fmt = format!("<span style='color: green;'>{}</span>", &cnt)
            },
            &Tagged::Whitespace(ref wh) => {
                match wh {
                    &WhitespaceType::Newline(line_counter) => {
                        if line_counter == 1 {
                            fmt = format!("<a name=\"l{}\">", line_counter);
                        } else {
                            fmt = format!("\n<a name=\"l{}\">", line_counter);
                        }
                    },
                    _ => { fmt = cnt.to_string() },
                }
            },
            _ => {
                fmt = cnt.to_string()
            },
        };
        out.push_str(&fmt);

        till = span.hi;
    }
    if till < content.len() {
        out.push_str(&content[till..]);
    }

    out.push_str("</pre>");
    writer.write(out.as_bytes()).unwrap();
}
