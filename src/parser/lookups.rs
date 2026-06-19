//! Binding powers and handler registration for the Pratt parser.
//!
//! This module defines the [`Lookups`] struct, which holds the tables that map
//! token kinds to handler functions and binding powers. The Pratt parser uses
//! these tables to decide how to parse each token it encounters.
//!
//! # Binding powers
//!
//! [`BindingPower`] controls operator precedence. Higher binding power means
//! tighter binding. The parser uses binding powers to decide when to stop
//! consuming tokens in a Pratt parsing loop.
//!
//! # Handler types
//!
//! There are three kinds of handlers:
//!
//! - **NUD** (null denotation) — handles tokens that appear at the start of an
//!   expression (e.g., literals, prefix operators, grouping).
//! - **LED** (left denotation) — handles tokens that appear after a left-hand
//!   expression (e.g., binary operators, member access, function calls).
//! - **Stmt** — handles tokens that start a statement (e.g., `let`, `fn`, `if`).
//!
//! # Extending the parser
//!
//! To add new syntax, register additional handlers via the [`Lookups`] methods
//! in [`create_token_lookups`] or [`create_type_token_lookups`].

use std::collections::HashMap;
use crate::ast::{Expr, Stmt, Type};
use crate::error::RelixError;
use crate::lexer::TokenKind;
use crate::parser::Parser;

/// Binding power levels for Pratt parsing.
///
/// Binding power controls operator precedence. Higher values bind more tightly.
/// The parser uses these levels to decide when to stop consuming tokens in a
/// Pratt parsing loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum BindingPower {
    /// Default / lowest binding power.
    #[default]
    Default = 0,
    /// Comma operator.
    Comma,
    /// Assignment operators (`=`, `+=`, `-=`, etc.).
    Assignment,
    /// Logical operators (`&&`, `||`) and range (`..`).
    Logical,
    /// Relational / comparison operators (`<`, `==`, etc.).
    Relational,
    /// Additive operators (`+`, `-`).
    Additive,
    /// Multiplicative operators (`*`, `/`, `%`).
    Multiplicative,
    /// Unary / prefix operators (`-`, `!`, `typeof`).
    Unary,
    /// Function call `()`.
    Call,
    /// Member access `.` and computed access `[]`.
    Member,
    /// Primary expressions (literals, identifiers, grouping).
    Primary,
}

/// Function signature for statement handlers.
pub type StmtHandler = fn(&mut Parser, &Lookups) -> Result<Stmt, RelixError>;

/// Function signature for NUD (null denotation) expression handlers.
///
/// NUD handlers are called when a token appears at the start of an expression.
pub type NudHandler  = fn(&mut Parser, &Lookups) -> Result<Expr, RelixError>;

/// Function signature for LED (left denotation) expression handlers.
///
/// LED handlers are called when a token appears after a left-hand expression.
/// They receive the left-hand expression and the current binding power.
pub type LedHandler  = fn(&mut Parser, &Lookups, Expr, BindingPower) -> Result<Expr, RelixError>;

/// Function signature for type NUD handlers.
pub type TypeNudHandler = fn(&mut Parser, &Lookups) -> Result<Type, RelixError>;

/// Function signature for type LED handlers.
pub type TypeLedHandler = fn(&mut Parser, &Lookups, Type, BindingPower) -> Result<Type, RelixError>;

/// Lookup tables for the Pratt parser.
///
/// Holds maps from token kinds to handler functions and binding powers. The
/// parser consults these tables to decide how to handle each token it
/// encounters.
///
/// Use the registration methods ([`nud`](Lookups::nud), [`led`](Lookups::led),
/// [`stmt`](Lookups::stmt), etc.) to populate the tables before parsing.
pub struct Lookups {
    /// Binding power table for expression parsing.
    pub bp:       HashMap<TokenKind, BindingPower>,
    /// NUD (null denotation) handler table for expression parsing.
    pub nud:      HashMap<TokenKind, NudHandler>,
    /// LED (left denotation) handler table for expression parsing.
    pub led:      HashMap<TokenKind, LedHandler>,
    /// Statement handler table.
    pub stmt:     HashMap<TokenKind, StmtHandler>,

    /// Binding power table for type parsing.
    pub type_bp:  HashMap<TokenKind, BindingPower>,
    /// NUD handler table for type parsing.
    pub type_nud: HashMap<TokenKind, TypeNudHandler>,
    /// LED handler table for type parsing.
    pub type_led: HashMap<TokenKind, TypeLedHandler>,
}

impl Default for Lookups {
    fn default() -> Self {
        Self::new()
    }
}

impl Lookups {
    /// Creates a new, empty `Lookups` instance.
    pub fn new() -> Self {
        Self {
            bp:       HashMap::new(),
            nud:      HashMap::new(),
            led:      HashMap::new(),
            stmt:     HashMap::new(),
            type_bp:  HashMap::new(),
            type_nud: HashMap::new(),
            type_led: HashMap::new(),
        }
    }

    /// Registers a LED (left denotation) handler for expression parsing.
    ///
    /// Also sets the binding power for the token kind in the `bp` table.
    pub fn led(&mut self, kind: TokenKind, bp: BindingPower, f: LedHandler) {
        self.bp.insert(kind, bp);
        self.led.insert(kind, f);
    }

    /// Registers a NUD (null denotation) handler for expression parsing.
    ///
    /// Also sets the binding power to [`BindingPower::Primary`] in the `bp`
    /// table, since NUD tokens typically terminate parsing loops.
    pub fn nud(&mut self, kind: TokenKind, _bp: BindingPower, f: NudHandler) {
        self.nud.insert(kind, f);
    }

    /// Registers a statement handler.
    ///
    /// Also sets the binding power to [`BindingPower::Default`] in the `bp`
    /// table.
    pub fn stmt(&mut self, kind: TokenKind, f: StmtHandler) {
        self.bp.insert(kind, BindingPower::Default);
        self.stmt.insert(kind, f);
    }

    /// Registers a LED handler for type parsing.
    ///
    /// Also sets the binding power for the token kind in the `type_bp` table.
    pub fn type_led(&mut self, kind: TokenKind, bp: BindingPower, f: TypeLedHandler) {
        self.type_bp.insert(kind, bp);
        self.type_led.insert(kind, f);
    }

    /// Registers a NUD handler for type parsing.
    ///
    /// Also sets the binding power to [`BindingPower::Primary`] in the
    /// `type_bp` table.
    pub fn type_nud(&mut self, kind: TokenKind, _bp: BindingPower, f: TypeNudHandler) {
        self.type_nud.insert(kind, f);
    }
}

/// Populates the lookup tables with handlers for all standard Relix expression
/// and statement syntax.
///
/// This function registers NUD, LED, and statement handlers for all built-in
/// operators, literals, keywords, and punctuation.
pub fn create_token_lookups(lu: &mut Lookups) {
    use crate::exprs::*;
    use crate::stmts::*;
    use BindingPower as BP;
    use TokenKind as TK;

    // Assignment
    lu.led(TK::Assignment,  BP::Assignment, parse_assignment_expr);
    lu.led(TK::PlusEquals,  BP::Assignment, parse_assignment_expr);
    lu.led(TK::MinusEquals, BP::Assignment, parse_assignment_expr);

    // Logical
    lu.led(TK::And,    BP::Logical, parse_binary_expr);
    lu.led(TK::Or,     BP::Logical, parse_binary_expr);
    lu.led(TK::DotDot, BP::Logical, parse_range_expr);

    // Relational
    lu.led(TK::Less,          BP::Relational, parse_binary_expr);
    lu.led(TK::LessEquals,    BP::Relational, parse_binary_expr);
    lu.led(TK::Greater,       BP::Relational, parse_binary_expr);
    lu.led(TK::GreaterEquals, BP::Relational, parse_binary_expr);
    lu.led(TK::Equals,        BP::Relational, parse_binary_expr);
    lu.led(TK::NotEquals,     BP::Relational, parse_binary_expr);

    // Additive & Multiplicative
    lu.led(TK::Plus,    BP::Additive,       parse_binary_expr);
    lu.led(TK::Dash,    BP::Additive,       parse_binary_expr);
    lu.led(TK::Slash,   BP::Multiplicative, parse_binary_expr);
    lu.led(TK::Star,    BP::Multiplicative, parse_binary_expr);
    lu.led(TK::Percent, BP::Multiplicative, parse_binary_expr);

    // Literals & symbols
    lu.nud(TK::Number,     BP::Primary, parse_primary_expr);
    lu.nud(TK::String,     BP::Primary, parse_primary_expr);
    lu.nud(TK::Identifier, BP::Primary, parse_primary_expr);

    // Unary / prefix
    lu.nud(TK::Typeof,      BP::Unary,   parse_prefix_expr);
    lu.nud(TK::Dash,        BP::Unary,   parse_prefix_expr);
    lu.nud(TK::Not,         BP::Unary,   parse_prefix_expr);
    lu.nud(TK::OpenBracket, BP::Primary, parse_array_literal_expr);

    // Member / computed / call
    lu.led(TK::Dot,         BP::Member, parse_member_expr);
    lu.led(TK::OpenBracket, BP::Member, parse_member_expr);
    lu.led(TK::OpenParen,   BP::Call,   parse_call_expr);

    // Grouping / fn / new
    lu.nud(TK::OpenParen, BP::Default, parse_grouping_expr);
    lu.nud(TK::Fn,        BP::Default, parse_fn_expr);
    lu.nud(TK::New,       BP::Default, parse_new_expr);

    // Statements
    lu.stmt(TK::OpenCurly, parse_block_stmt);
    lu.stmt(TK::Let,       parse_var_decl_stmt);
    lu.stmt(TK::Const,     parse_var_decl_stmt);
    lu.stmt(TK::Fn,        parse_fn_declaration);
    lu.stmt(TK::If,        parse_if_stmt);
    lu.stmt(TK::Import,    parse_import_stmt);
    lu.stmt(TK::Foreach,   parse_foreach_stmt);
    lu.stmt(TK::Class,     parse_class_declaration_stmt);
    lu.stmt(TK::Return,    parse_return_stmt);
}

/// Populates the lookup tables with handlers for Relix type annotation syntax.
///
/// This function registers NUD and LED handlers for symbol types (e.g.,
/// `number`) and list types (e.g., `[]number`).
pub fn create_type_token_lookups(lu: &mut Lookups) {
    use crate::types::*;
    use BindingPower as BP;
    use TokenKind as TK;

    lu.type_nud(TK::Identifier,  BP::Primary, parse_symbol_type);
    lu.type_nud(TK::OpenBracket, BP::Member,  parse_list_type);
}
