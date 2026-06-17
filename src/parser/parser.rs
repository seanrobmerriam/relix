use crate::ast::{BlockStmt, Stmt};
use crate::lexer::{tokenize, Token, TokenKind};
use crate::lookups::{create_token_lookups, create_type_token_lookups, Lookups};

// ---------------------------------------------------------------------------
// Parser struct
// ---------------------------------------------------------------------------

pub struct Parser {
    pub tokens: Vec<Token>,
    pub pos:    usize,
}

impl Parser {
    pub fn current_token(&self) -> &Token {
        &self.tokens[self.pos]
    }

    pub fn advance(&mut self) -> Token {
        let tk = self.tokens[self.pos].clone();
        self.pos += 1;
        tk
    }

    pub fn has_tokens(&self) -> bool {
        self.pos < self.tokens.len() && self.current_token_kind() != TokenKind::Eof
    }

    pub fn next_token(&self) -> &Token {
        &self.tokens[self.pos + 1]
    }

    pub fn previous_token(&self) -> &Token {
        &self.tokens[self.pos - 1]
    }

    pub fn current_token_kind(&self) -> TokenKind {
        self.tokens[self.pos].kind
    }

    pub fn expect_error(&mut self, expected: TokenKind, err: Option<String>) -> Token {
        let token = self.current_token().clone();
        if token.kind != expected {
            let msg = err.unwrap_or_else(|| {
                format!(
                    "Expected {} but received {} instead\n",
                    expected, token.kind
                )
            });
            panic!("{}", msg);
        }
        self.advance()
    }

    pub fn expect(&mut self, expected: TokenKind) -> Token {
        self.expect_error(expected, None)
    }
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

pub fn parse(source: &str) -> BlockStmt {
    let tokens = tokenize(source);
    let mut lookups = Lookups::new();
    create_token_lookups(&mut lookups);
    create_type_token_lookups(&mut lookups);

    let mut p = Parser { tokens, pos: 0 };
    let mut body: Vec<Stmt> = Vec::new();

    while p.has_tokens() {
        body.push(crate::stmts::parse_stmt(&mut p, &lookups));
    }

    BlockStmt { body }
}
