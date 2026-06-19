//! Lexical analysis (tokenization) for the Relix language.
//!
//! This module converts raw source text into a sequence of [`Token`] values
//! that the parser can consume. The lexer is regex-driven: each pattern in an
//! ordered table is tested against the remaining input, and the first match at
//! position zero wins.
//!
//! # Example
//!
//! ```
//! use relix::lexer::{tokenize, TokenKind};
//!
//! let tokens = tokenize("let x = 42;").unwrap();
//! assert_eq!(tokens[0].kind, TokenKind::Let);
//! assert_eq!(tokens[1].kind, TokenKind::Identifier);
//! assert_eq!(tokens[2].kind, TokenKind::Assignment);
//! assert_eq!(tokens[3].kind, TokenKind::Number);
//! ```

mod token;
mod tokenizer;

pub use token::*;
pub use tokenizer::*;
