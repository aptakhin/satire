extern crate satire;

use satire::indexer::parser::CommonParser;
use satire::indexer::storage::{SourceFile, Index, IndexBuilder};
use satire::indexer::gen;

fn main() {
    let file = "test/src.rs".to_string();
    let root_dir = "test/";

    let mut index_builder = IndexBuilder::new();
    index_builder.build_dir(root_dir);
    index_builder.gen();

    //server.listen("127.0.0.1:3003");

    // for i in &deduced {
    //     let generated = i.gen();
    //     gen::to_file(i.file.clone(), &i.content, &generated[..]);
    // }

    //gen::generate(&generated, root_dir);

    // let source = SourceFile::new(file.clone());
    // let mut parser = CommonParser::new(file[root_dir.len()..].to_string(), source.content.clone());
    // let mut ctx = parser.parse();
    //
    // {
    //     let mut index = Index { set: vec![] };
    //     ctx.deduce(&index);
    //     index.add(&mut ctx);
    // }
    //
    // let baked = ctx.gen();
    // gen::to_file("test/src.rs.html".to_string(), &source.content, &baked[..]);
}
