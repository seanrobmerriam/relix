# Tokenizer

A Pratt parser implementation in Rust.

## Overview

This project implements a lexer and recursive descent parser using Pratt parsing techniques. It can parse a custom language with support for:

- Expressions (arithmetic, logical, comparison, assignment)
- Statements (variable declarations, functions, conditionals, loops, classes)
- Type annotations
- Member access and function calls

## Project Structure

```
src/
├── lexer/          # Tokenization
│   ├── token.rs    # Token types and kinds
│   └── tokenizer.rs # Regex-based lexer
├── parser/         # Parsing
│   ├── parser.rs   # Parser struct and main entry
│   ├── expr.rs     # Expression parsing
│   ├── stmt.rs     # Statement parsing
│   ├── types.rs    # Type parsing
│   └── lookups.rs  # Binding power and handler tables
├── ast/            # AST node definitions
│   └── mod.rs      # All expression, statement, and type nodes
├── lib.rs          # Library root
└── main.rs         # Binary entry point
```

## Building

```bash
cargo build
```

## Testing

```bash
cargo test
```

## Usage

```rust
use tokenizer::parser::parse;

let source = r#"
    let x = 10;
    fn add(a: number, b: number): number {
        return a + b;
    }
"#;

let ast = parse(source);
println!("{:#?}", ast);
```

## License

MIT
