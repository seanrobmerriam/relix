use crate::ast::{BlockStmt, Stmt};
use crate::error::RelixError;
use crate::lexer::{tokenize, Token, TokenKind};
use crate::lookups::{create_token_lookups, create_type_token_lookups, Lookups};

/// The core parser state.
///
/// Holds the token stream produced by the lexer and a cursor tracking the
/// current position. Provides methods for advancing through tokens, peeking at
/// upcoming tokens, and asserting expected token kinds.
///
/// The `Parser` struct is usually not constructed directly; use the [`parse`]
/// function instead.
#[derive(Debug)]
pub struct Parser {
    /// The token stream.
    pub tokens: Vec<Token>,
    /// The current position in the token stream.
    pub pos:    usize,
}

impl Parser {
    /// Returns a reference to the token at the current position.
    pub fn current_token(&self) -> &Token {
        &self.tokens[self.pos]
    }

    /// Advances the cursor by one position and returns the consumed token.
    pub fn advance(&mut self) -> Token {
        let tk = self.tokens[self.pos].clone();
        self.pos += 1;
        tk
    }

    /// Returns `true` if there are more tokens to consume (i.e., the cursor has
    /// not yet reached the end-of-file token).
    pub fn has_tokens(&self) -> bool {
        self.pos < self.tokens.len() && self.current_token_kind() != TokenKind::Eof
    }

    /// Returns a reference to the next token (one position ahead of the cursor).
    pub fn next_token(&self) -> &Token {
        &self.tokens[self.pos + 1]
    }

    /// Returns a reference to the previous token (one position behind the cursor).
    pub fn previous_token(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }

    /// Returns the [`TokenKind`] of the token at the current position.
    pub fn current_token_kind(&self) -> TokenKind {
        self.tokens[self.pos].kind
    }

    /// Asserts that the current token matches `expected`, returning it if so.
    ///
    /// If the token does not match, returns either the provided error
    /// message or a default message indicating what was expected vs. received.
    pub fn expect_error(&mut self, expected: TokenKind, err: Option<String>) -> Result<Token, RelixError> {
        let token = self.current_token().clone();
        if token.kind != expected {
            let msg = err.unwrap_or_else(|| {
                format!(
                    "Expected {} but received {} instead\n",
                    expected, token.kind
                )
            });
            return Err(RelixError::new(msg));
        }
        Ok(self.advance())
    }

    /// Asserts that the current token matches `expected`, returning it if so.
    ///
    /// Returns an error with a default message if the token does not match.
    pub fn expect(&mut self, expected: TokenKind) -> Result<Token, RelixError> {
        self.expect_error(expected, None)
    }
}

/// Parses a Relix source string into an abstract syntax tree.
///
/// This is the main entry point for the parser. It tokenizes the input, sets up
/// the Pratt parsing lookup tables, and then parses statements until the entire
/// input has been consumed.
///
/// # Returns
///
/// A [`BlockStmt`] containing all top-level statements in the source.
///
/// # Errors
///
/// Returns a [`RelixError`] if the lexer encounters an unrecognized token, or
/// if the parser encounters a syntax error (e.g., an unexpected token).
///
/// # Example
///
/// ```
/// use relix::parser::parse;
///
/// let source = "let x = 10;";
/// let ast = parse(source).unwrap();
/// println!("{:#?}", ast);
/// ```
pub fn parse(source: &str) -> Result<BlockStmt, RelixError> {
    let tokens = tokenize(source)?;
    let mut lookups = Lookups::new();
    create_token_lookups(&mut lookups);
    create_type_token_lookups(&mut lookups);

    let mut p = Parser { tokens, pos: 0 };
    let mut body: Vec<Stmt> = Vec::new();

    while p.has_tokens() {
        body.push(crate::stmts::parse_stmt(&mut p, &lookups)?);
    }

    Ok(BlockStmt { body })
}
