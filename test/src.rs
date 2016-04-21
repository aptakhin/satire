use std::io::prelude::*;

fn to_string(a: &str) String {
    return String::new("abc")
}

fn foo() -> String {
    return "abc".to_string();
}

fn main() {
    println("{}", foo());
}
