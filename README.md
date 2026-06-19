# Relix

A Pratt parser implementation in Rust.

## Overview

Relix provides a complete lexing and parsing pipeline built on [Pratt parsing](https://en.wikipedia.org/wiki/Pratt_parser) (top-down operator precedence) techniques. It tokenizes source code via a regex-driven lexer, then builds an abstract syntax tree (AST) using configurable binding-power tables.

It can parse a custom language with support for:

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

Create a `test.lang` file with your source code:

```typescript
// Variable declarations
let x = 10;
const name = "hello";

// Functions
fn add(a: number, b: number): number {
    return a + b;
}

// Conditionals
if x > 5 {
    let y = x * 2;
} else {
    let y = 0;
}

// Loops
foreach item in items {
    print(item);
}

// Classes
class Point {
    let x: number;
    let y: number;
}

// Expressions
let result = add(1, 2) + 3 * 4;
let arr = [1, 2, 3];
let p = new Point();
```

Run the parser:

```bash
cargo run
```

Or use as a library:

```rust
use relix::parser::parse;

let source = r#"
    let x = 10;
    fn add(a: number, b: number): number {
        return a + b;
    }
"#;

let ast = parse(source);
println!("{:#?}", ast);
```

## Documentation

Full API documentation is available on [docs.rs](https://docs.rs/relix) once published.

To generate documentation locally:

```bash
cargo doc --open
```

## License

MIT
