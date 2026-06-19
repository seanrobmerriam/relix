//! # Relix
//!
//! A Pratt parser library for a custom language with first-class support for
//! expressions, statements, and type annotations.
//!
//! Relix provides a complete lexing and parsing pipeline built on
//! [Pratt parsing](https://en.wikipedia.org/wiki/Pratt_parser) (top-down
//! operator precedence) techniques. It tokenizes source code via a
//! regex-driven lexer, then builds an abstract syntax tree (AST) using
//! configurable binding-power tables.
//!
//! ## Quick start
//!
//! ```
//! use relix::parser::parse;
//!
//! let source = r#"
//!     let x = 10;
//!     fn add(a: number, b: number): number {
//!         a + b;
//!     }
//! "#;
//!
//! let ast = parse(source).unwrap();
//! println!("{:#?}", ast);
//! ```
//!
//! ## Architecture
//!
//! The library is organised into three layers:
//!
//! | Layer   | Module               | Purpose                               |
//! |---------|----------------------|---------------------------------------|
//! | Lexer   | [`lexer`]            | Regex-based tokenization              |
//! | AST     | [`ast`]              | Expression, statement, and type nodes |
//! | Parser  | [`parser`]           | Pratt-parser driver and handlers      |
//!
//! ### Lexer
//!
//! The [`lexer`] module converts raw source text into a flat sequence of
//! [`Token`](lexer::Token) values. It recognises literals (numbers, strings,
//! identifiers), operators, punctuation, and reserved keywords.
//!
//! ### AST
//!
//! The [`ast`] module defines the node types that make up the abstract syntax
//! tree. Every construct the parser can produce is represented as a variant of
//! one of three top-level enums:
//!
//! - [`ast::Expr`] — expression nodes (literals, binary ops, calls, …)
//! - [`ast::Stmt`] — statement nodes (variable declarations, functions, …)
//! - [`ast::Type`] — type annotation nodes (symbol types, list types)
//!
//! ### Parser
//!
//! The [`parser`] module ties everything together. The main entry point is
//! [`parser::parse`], which accepts a source string and returns a
//! [`BlockStmt`](ast::BlockStmt) representing the entire program.
//!
//! Internally the parser uses Pratt parsing with configurable **nud** (null
//! denotation), **led** (left denotation), and **binding power** tables
//! registered via [`parser::lookups::Lookups`].
//!
//! ## Supported language features
//!
//! - **Expressions**: arithmetic, logical, comparison, assignment, member
//!   access, computed access, function calls, array literals, range
//!   expressions, anonymous functions, `new` instantiation
//! - **Statements**: variable declarations (`let`/`const`), function
//!   declarations, `if`/`else`, `foreach` loops, `import`, class declarations,
//!   block statements
//! - **Type annotations**: symbol types (`number`, `string`) and list types
//!   (`[]T`)
//!
//! ## Extending the parser
//!
//! New syntax can be added by registering additional nud/led handlers in the
//! [`Lookups`](parser::lookups::Lookups) table. See [`parser::lookups`] for
//! details on binding powers and handler signatures.

pub mod lexer;
pub mod ast;
pub mod parser;
pub mod error;

pub use error::RelixError;
pub use parser::lookups;
pub use parser::expr as exprs;
pub use parser::stmt as stmts;
pub use parser::types;
