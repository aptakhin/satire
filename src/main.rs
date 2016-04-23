extern crate satire;

use satire::indexer::parser::CommonParser;
use satire::indexer::storage::SourceFile;
use satire::indexer::storage::Index;
use satire::indexer::gen;

fn main() {
    let file = "test/src.rs".to_string();
    let root_dir = "test/";
    let source = SourceFile::new(file.clone());
    let mut parser = CommonParser::new(file[root_dir.len()..].to_string(), source.content.clone());
    let mut ctx = parser.parse();

    {
        let mut index = Index { set: vec![] };
        ctx.deduce(&index);
        index.add(&mut ctx);
    }

    let baked = ctx.gen();
    gen::to_file("test/src.rs.html".to_string(), &source.content, &baked[..]);
}
