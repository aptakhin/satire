extern crate satire;

use satire::indexer::parser::CommonParser;
//use satire::indexer::storage::Storage;
use satire::indexer::storage::SourceFile;
use satire::indexer::gen;

fn main() {
    //let mut storage = Storage::new();

    let source = SourceFile::new("test/src.rs".to_string());

    let mut parser = CommonParser::new(source.content.clone());
    let ctx = parser.parse();


    //
    let baked = ctx.gen();
    //
    // let html = gen::to_html(&ctx.all_tagged);
    gen::to_file("test/src.rs.html".to_string(), &source.content, &baked[..]);
}
