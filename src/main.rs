extern crate satire;

use satire::indexer::parser::CommonParser;
use satire::indexer::storage::Storage;
use satire::indexer::storage::SourceFile;
use satire::indexer::gen;

fn main() {
    let mut storage = Storage::new();

    let source = SourceFile::new("test/src.rs".to_string());

    let mut parser = CommonParser::new(source.content);
    let mut ctx = parser.parse();

    ctx.gen();

    let html = gen::to_html(&ctx.all_tagged);
    gen::to_file("test/src.rs.html".to_string(), &html);
}
