extern crate satire;

use std::io::prelude::*;
use std::io::BufReader;
use std::io::BufWriter;
use std::fs::File;

use satire::indexer::parser::CommonParser;
use satire::indexer::storage::Storage;
use satire::indexer::storage::FileSource;
use satire::indexer::gen::to_file;
use satire::indexer::gen::to_html;

fn main() {
    let mut storage = Storage::new();

    let input = File::open("test/src.rs").unwrap();
    let mut reader = BufReader::new(input);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).unwrap();

    let mut parser = CommonParser::new(buffer);
    let ctx = parser.parse();

    let html = to_html(&ctx.all_tagged);
    to_file(String::from("test/src.rs.html"), &html);
}
