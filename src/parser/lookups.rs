use std::collections::HashMap;
use crate::ast::{Expr, Stmt, Type};
use crate::lexer::TokenKind;
use crate::parser::Parser;

// ---------------------------------------------------------------------------
// Binding power
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BindingPower {
    Default = 0,
    Comma,
    Assignment,
    Logical,
    Relational,
    Additive,
    Multiplicative,
    Unary,
    Call,
    Member,
    Primary,
}

// ---------------------------------------------------------------------------
// Handler types
// ---------------------------------------------------------------------------

pub type StmtHandler = fn(&mut Parser, &Lookups) -> Stmt;
pub type NudHandler  = fn(&mut Parser, &Lookups) -> Expr;
pub type LedHandler  = fn(&mut Parser, &Lookups, Expr, BindingPower) -> Expr;

pub type TypeNudHandler = fn(&mut Parser, &Lookups) -> Type;
pub type TypeLedHandler = fn(&mut Parser, &Lookups, Type, BindingPower) -> Type;

// ---------------------------------------------------------------------------
// Lookup tables
// ---------------------------------------------------------------------------

pub struct Lookups {
    pub bp:       HashMap<TokenKind, BindingPower>,
    pub nud:      HashMap<TokenKind, NudHandler>,
    pub led:      HashMap<TokenKind, LedHandler>,
    pub stmt:     HashMap<TokenKind, StmtHandler>,

    pub type_bp:  HashMap<TokenKind, BindingPower>,
    pub type_nud: HashMap<TokenKind, TypeNudHandler>,
    pub type_led: HashMap<TokenKind, TypeLedHandler>,
}

impl Lookups {
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

    pub fn led(&mut self, kind: TokenKind, bp: BindingPower, f: LedHandler) {
        self.bp.insert(kind, bp);
        self.led.insert(kind, f);
    }

    pub fn nud(&mut self, kind: TokenKind, _bp: BindingPower, f: NudHandler) {
        self.bp.insert(kind, BindingPower::Primary);
        self.nud.insert(kind, f);
    }

    pub fn stmt(&mut self, kind: TokenKind, f: StmtHandler) {
        self.bp.insert(kind, BindingPower::Default);
        self.stmt.insert(kind, f);
    }

    pub fn type_led(&mut self, kind: TokenKind, bp: BindingPower, f: TypeLedHandler) {
        self.type_bp.insert(kind, bp);
        self.type_led.insert(kind, f);
    }

    pub fn type_nud(&mut self, kind: TokenKind, _bp: BindingPower, f: TypeNudHandler) {
        self.type_bp.insert(kind, BindingPower::Primary);
        self.type_nud.insert(kind, f);
    }
}

// ---------------------------------------------------------------------------
// Token lookup registration
// ---------------------------------------------------------------------------

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
}

pub fn create_type_token_lookups(lu: &mut Lookups) {
    use crate::types::*;
    use BindingPower as BP;
    use TokenKind as TK;

    lu.type_nud(TK::Identifier,  BP::Primary, parse_symbol_type);
    lu.type_nud(TK::OpenBracket, BP::Member,  parse_list_type);
}
