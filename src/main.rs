extern crate satire;

use satire::indexer::storage::{IndexBuilder};

fn main() {
    let root_dir = "test/";

    let mut index_builder = IndexBuilder::new();
    index_builder.build_dir(root_dir);
    index_builder.gen();
}
