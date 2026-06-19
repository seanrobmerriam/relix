use std::fs;
use relix::parser::parse;

fn main() {
    let source = fs::read_to_string("test.lang").expect("Failed to read test.lang");
    match parse(&source) {
        Ok(ast) => println!("{:#?}", ast),
        Err(e) => eprintln!("Parse error: {}", e),
    }
}
