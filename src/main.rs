extern crate satire;

use satire::indexer::parser::CommonParser;

fn main() {
    let mut parser = CommonParser::new();

    parser.parse();
}
