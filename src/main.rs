use std::fs;
use tokenizer::parser::parse;

fn main() {
    let source = fs::read_to_string("test.lang").expect("Failed to read test.lang");
    let ast = parse(&source);
    println!("{:#?}", ast);
}
