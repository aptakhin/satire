extern crate satire;

use std::io::prelude::*;
use std::io::BufReader;
use std::fs::File;

use satire::indexer::parser::CommonParser;
use satire::indexer::storage::Storage;
use satire::indexer::gen;

fn main() {
    let mut storage = Storage::new();

    let input = File::open("test/src.rs").unwrap();
    let mut reader = BufReader::new(input);
    let mut buffer = String::new();
    reader.read_to_string(&mut buffer).unwrap();

    let mut parser = CommonParser::new(buffer);
    let ctx = parser.parse();

    let html = gen::to_html(&ctx.all_tagged);
    gen::to_file(String::from("test/src.rs.html"), &html);
}
