use std::io::prelude::*;

struct Foo {

}

impl Foo {
    fn new() -> Foo {
        Foo {}
    }
}

fn to_string(a: &str) -> String {
    
    String::new("abc")
}

fn foo() -> String {
    return "abc".to_string();
}
