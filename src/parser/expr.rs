use crate::ast::{
    ArrayLiteral, AssignmentExpr, BinaryExpr, CallExpr, ComputedExpr, Expr, FunctionExpr,
    MemberExpr, NewExpr, Parameter, PrefixExpr, RangeExpr, Type,
};
use crate::lexer::TokenKind;
use crate::lookups::{BindingPower, Lookups};
use crate::parser::Parser;
use crate::stmts::parse_block_stmt;
use crate::types::parse_type;

// ---------------------------------------------------------------------------
// Core Pratt expression driver
// ---------------------------------------------------------------------------

pub fn parse_expr(p: &mut Parser, lu: &Lookups, bp: BindingPower) -> Expr {
    let token_kind = p.current_token_kind();

    let nud_fn = lu.nud.get(&token_kind).unwrap_or_else(|| {
        panic!("NUD Handler expected for token {}\n", token_kind)
    });

    let mut left = nud_fn(p, lu);

    while lu.bp.get(&p.current_token_kind()).copied().unwrap_or(BindingPower::Default) > bp {
        let token_kind = p.current_token_kind();

        let led_fn = lu.led.get(&token_kind).unwrap_or_else(|| {
            panic!("LED Handler expected for token {}\n", token_kind)
        });

        left = led_fn(p, lu, left, bp);
    }

    left
}

// ---------------------------------------------------------------------------
// Prefix / unary
// ---------------------------------------------------------------------------

pub fn parse_prefix_expr(p: &mut Parser, lu: &Lookups) -> Expr {
    let operator = p.advance();
    let right = parse_expr(p, lu, BindingPower::Unary);
    Expr::Prefix(PrefixExpr {
        operator,
        right: Box::new(right),
    })
}

// ---------------------------------------------------------------------------
// Assignment
// ---------------------------------------------------------------------------

pub fn parse_assignment_expr(
    p: &mut Parser,
    lu: &Lookups,
    left: Expr,
    bp: BindingPower,
) -> Expr {
    p.advance();
    let rhs = parse_expr(p, lu, bp);
    Expr::Assignment(AssignmentExpr {
        assignee:       Box::new(left),
        assigned_value: Box::new(rhs),
    })
}

// ---------------------------------------------------------------------------
// Range
// ---------------------------------------------------------------------------

pub fn parse_range_expr(
    p: &mut Parser,
    lu: &Lookups,
    left: Expr,
    bp: BindingPower,
) -> Expr {
    p.advance();
    Expr::Range(RangeExpr {
        lower: Box::new(left),
        upper: Box::new(parse_expr(p, lu, bp)),
    })
}

// ---------------------------------------------------------------------------
// Binary / infix
// ---------------------------------------------------------------------------

pub fn parse_binary_expr(
    p: &mut Parser,
    lu: &Lookups,
    left: Expr,
    _bp: BindingPower,
) -> Expr {
    let operator = p.advance();
    let right = parse_expr(p, lu, BindingPower::Default);
    Expr::Binary(BinaryExpr {
        left:     Box::new(left),
        operator,
        right:    Box::new(right),
    })
}

// ---------------------------------------------------------------------------
// Primary literals
// ---------------------------------------------------------------------------

pub fn parse_primary_expr(p: &mut Parser, _lu: &Lookups) -> Expr {
    match p.current_token_kind() {
        TokenKind::Number => {
            let value: f64 = p.advance().value.parse().unwrap_or(0.0);
            Expr::Number(crate::ast::NumberExpr { value })
        }
        TokenKind::String => Expr::String(crate::ast::StringExpr {
            value: p.advance().value,
        }),
        TokenKind::Identifier => Expr::Symbol(crate::ast::SymbolExpr {
            value: p.advance().value,
        }),
        other => panic!(
            "Cannot create primary_expr from {}\n",
            other
        ),
    }
}

// ---------------------------------------------------------------------------
// Member / computed
// ---------------------------------------------------------------------------

pub fn parse_member_expr(
    p: &mut Parser,
    lu: &Lookups,
    left: Expr,
    bp: BindingPower,
) -> Expr {
    let is_computed = p.advance().kind == TokenKind::OpenBracket;

    if is_computed {
        let rhs = parse_expr(p, lu, bp);
        p.expect(TokenKind::CloseBracket);
        Expr::Computed(ComputedExpr {
            member:   Box::new(left),
            property: Box::new(rhs),
        })
    } else {
        Expr::Member(MemberExpr {
            member:   Box::new(left),
            property: p.expect(TokenKind::Identifier).value,
        })
    }
}

// ---------------------------------------------------------------------------
// Array literal
// ---------------------------------------------------------------------------

pub fn parse_array_literal_expr(p: &mut Parser, lu: &Lookups) -> Expr {
    p.expect(TokenKind::OpenBracket);
    let mut contents: Vec<Expr> = Vec::new();

    while p.has_tokens() && p.current_token_kind() != TokenKind::CloseBracket {
        contents.push(parse_expr(p, lu, BindingPower::Logical));
        if !p.current_token().is_one_of_many(&[TokenKind::Eof, TokenKind::CloseBracket]) {
            p.expect(TokenKind::Comma);
        }
    }

    p.expect(TokenKind::CloseBracket);
    Expr::Array(ArrayLiteral { contents })
}

// ---------------------------------------------------------------------------
// Grouping
// ---------------------------------------------------------------------------

pub fn parse_grouping_expr(p: &mut Parser, lu: &Lookups) -> Expr {
    p.expect(TokenKind::OpenParen);
    let expr = parse_expr(p, lu, BindingPower::Default);
    p.expect(TokenKind::CloseParen);
    expr
}

// ---------------------------------------------------------------------------
// Call
// ---------------------------------------------------------------------------

pub fn parse_call_expr(
    p: &mut Parser,
    lu: &Lookups,
    left: Expr,
    _bp: BindingPower,
) -> Expr {
    p.advance(); // consume `(`
    let mut arguments: Vec<Expr> = Vec::new();

    while p.has_tokens() && p.current_token_kind() != TokenKind::CloseParen {
        arguments.push(parse_expr(p, lu, BindingPower::Assignment));
        if !p.current_token().is_one_of_many(&[TokenKind::Eof, TokenKind::CloseParen]) {
            p.expect(TokenKind::Comma);
        }
    }

    p.expect(TokenKind::CloseParen);
    Expr::Call(CallExpr {
        method:    Box::new(left),
        arguments,
    })
}

// ---------------------------------------------------------------------------
// Function expression
// ---------------------------------------------------------------------------

pub fn parse_fn_expr(p: &mut Parser, lu: &Lookups) -> Expr {
    p.expect(TokenKind::Fn);
    let (parameters, return_type, body) = parse_fn_params_and_body(p, lu);
    Expr::Function(FunctionExpr {
        parameters,
        return_type,
        body,
    })
}

// ---------------------------------------------------------------------------
// New expression
// ---------------------------------------------------------------------------

pub fn parse_new_expr(p: &mut Parser, lu: &Lookups) -> Expr {
    p.advance(); // consume `new`
    let instantiation = parse_expr(p, lu, BindingPower::Default);
    let call = match instantiation {
        Expr::Call(c) => c,
        _ => panic!("Expected call expression after `new`"),
    };
    Expr::New(NewExpr {
        instantiation: call,
    })
}

// ---------------------------------------------------------------------------
// Shared: function params + body (used by both fn decl and fn expr)
// ---------------------------------------------------------------------------

pub fn parse_fn_params_and_body(
    p: &mut Parser,
    lu: &Lookups,
) -> (Vec<Parameter>, Option<Type>, Vec<crate::ast::Stmt>) {
    let mut params: Vec<Parameter> = Vec::new();

    p.expect(TokenKind::OpenParen);
    while p.has_tokens() && p.current_token_kind() != TokenKind::CloseParen {
        let param_name = p.expect(TokenKind::Identifier).value;
        p.expect(TokenKind::Colon);
        let param_type = parse_type(p, lu, BindingPower::Default);

        params.push(Parameter {
            name: param_name,
            ty:   param_type,
        });

        if !p.current_token().is_one_of_many(&[TokenKind::CloseParen, TokenKind::Eof]) {
            p.expect(TokenKind::Comma);
        }
    }
    p.expect(TokenKind::CloseParen);

    let return_type = if p.current_token_kind() == TokenKind::Colon {
        p.advance();
        Some(parse_type(p, lu, BindingPower::Default))
    } else {
        None
    };

    let body = match parse_block_stmt(p, lu) {
        crate::ast::Stmt::Block(b) => b.body,
        _ => panic!("Expected block statement for function body"),
    };

    (params, return_type, body)
}
