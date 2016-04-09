extern crate satire;

use satire::indexer::parser::CommonParser;
use satire::indexer::storage::Storage;
use satire::indexer::storage::FileSource;
use satire::indexer::gen::to_file;
use satire::indexer::gen::HtmlItem;
use satire::indexer::gen::Reference;
use std::collections::LinkedList;

fn main() {
    let mut storage = Storage::new();
    //let storage = &mut storage;
    let mut parser = CommonParser::new();
    let ctx = parser.parse();
    storage.merge(ctx);

    let mut items: Vec<Box<HtmlItem>> = vec![Box::new(Reference{
        content: String::from("xx"),
        source: FileSource {
            file: String::from("test/src.rs.html"),
            line: 0,
            id_iter: 0,
            lexem_iter: 0,
        },
    })];

    to_file(String::from("test/src.rs.html"), items, &storage);

}
