//! Parsing infrastructure for the Relix language.
//!
//! This module implements a Pratt parser (top-down operator precedence) that
//! consumes tokens from the [`lexer`](crate::lexer) module and produces an
//! abstract syntax tree defined in the [`ast`](crate::ast) module.
//!
//! The parser is organised around configurable **lookup tables** that map token
//! kinds to handler functions. This design makes it straightforward to extend
//! the language with new syntax by registering additional handlers.
//!
//! # Entry point
//!
//! The main entry point is the [`parse`] function, which accepts a source
//! string and returns a [`BlockStmt`](crate::ast::BlockStmt) representing the
//! entire program.
//!
//! # Architecture
//!
//! - [`parser::Parser`] — the `Parser` struct and main parsing loop
//! - [`expr`] — expression parsing (Pratt driver, binary ops, calls, etc.)
//! - [`stmt`] — statement parsing (declarations, control flow, etc.)
//! - [`types`] — type annotation parsing
//! - [`lookups`] — binding powers and handler registration

mod core;
pub mod expr;
pub mod stmt;
pub mod types;
pub mod lookups;

pub use self::core::*;
