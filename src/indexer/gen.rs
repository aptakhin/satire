use std::io::prelude::*;
use std::io::BufWriter;
use std::fs::File;
use std::rc::Rc;

use indexer::parser::Tagged;
use indexer::lexer::Span;
use indexer::lexer::WhitespaceType;
use indexer::storage::{FileSource, Info, IndexBuilder, ParsedFile};

impl FileSource {
    pub fn render_html(&self, name: &str) -> String {
        let file = format!("test/{}.html", self.file);
        format!("<a href='/{}#l{}'>{}</a>", file, self.line, name)
    }
}

// pub fn generate(index: &IndexBuilder, root_dir: &str) {
//     for parsed_file in &index.set {
//         let gen = parsed_file.ctx.gen();
//         to_file(
//             format!("{}.html", parsed_file.file),
//             &parsed_file.content,
//             &gen,
//         )
//     }
// }

pub fn to_string(content: Rc<String>, items: &[(Tagged, Span, Option<Box<Info>>)]) -> String {
    let mut out = String::new();

    let mut till = 0;
    for &(ref tagged, ref span, ref info) in items {
        //println!("A: {}, {}, {}", till, span.lo, span.hi);
        out.push_str(&content[till..span.lo]);

        let cnt = &content[span.lo..span.hi];
        let fmt;

        match tagged {
            &Tagged::Keyword(ref kw) => {
                fmt = format!("<b>{}</b>", &cnt)
            },
            &Tagged::Comment => {
                fmt = format!("<span style='color: green;'>{}</span>", &cnt)
            },
            &Tagged::QuotedString => {
                fmt = format!("<span style='color: green;'>{}</span>", &cnt)
            },
            &Tagged::Calling(ref use_context) => {
                match info {
                    &Some(ref add_info) => {
                        let refs = add_info.refs.iter().enumerate().fold(String::new(), |res, (_, i)| {
                            let file = format!("test/{}.html", i.file);
                            res + &format!("<li><a href='/{}#l{}' target='_blank'>{}: {}
                            </a></li>", file, i.line, file, i.line)
                        });
                        fmt = format!("<a tabindex='0' role='button' data-container='body' data-trigger='focus' data-toggle='popover' data-placement='bottom' data-content=\"<ul>{}</ul>\">{}</a>", refs, use_context.reference.path[0].1);
                    },
                    _ => { fmt = cnt.to_string() },
                }
            },
            &Tagged::Whitespace(WhitespaceType::Newline) => {
                if span.line == 1 {
                    fmt = format!("<a name=\"l{}\"></a>", span.line);
                } else {
                    fmt = format!("\n<a name=\"l{}\"></a>", span.line);
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

    out
}

pub fn to_file(filename: String, content: Rc<String>, items: &[(Tagged, Span, Option<Box<Info>>)]) {
    let output = File::create(filename).unwrap();
    let mut writer = BufWriter::new(output);
    let out = to_string(content, items);
    writer.write(out.as_bytes()).unwrap();
}
