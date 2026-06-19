//! Expression parsing for the Relix language.
//!
//! This module implements the Pratt parsing driver for expressions, along with
//! all NUD and LED handlers for individual expression forms (literals, binary
//! operators, calls, member access, etc.).
//!
//! The core entry point is [`parse_expr`], which drives the Pratt parsing loop
//! using binding powers and handler tables from [`Lookups`].

use crate::ast::{
    ArrayLiteral, AssignmentExpr, BinaryExpr, CallExpr, ComputedExpr, Expr, FunctionExpr,
    MemberExpr, NewExpr, Parameter, PrefixExpr, RangeExpr, Stmt, Type,
};
use crate::error::RelixError;
use crate::lexer::TokenKind;
use crate::lookups::{BindingPower, Lookups};
use crate::parser::Parser;
use crate::stmts::parse_block_stmt;
use crate::types::parse_type;

type FnParamsAndBody = (Vec<Parameter>, Option<Type>, Vec<Stmt>);

/// The core Pratt expression parsing driver.
///
/// Parses an expression starting at the current token position, using the NUD
/// and LED handler tables from `lu` to handle each token. The `bp` parameter
/// controls the binding power threshold: the parser will stop consuming tokens
/// when it encounters a token with binding power less than or equal to `bp`.
///
/// # Algorithm
///
/// 1. Look up the NUD handler for the current token and call it to produce the
///    initial left-hand expression.
/// 2. While the current token's binding power (from the `bp` table) is greater
///    than `bp`, look up the LED handler and call it with the left-hand
///    expression to produce a new left-hand expression.
/// 3. Return the final left-hand expression.
pub fn parse_expr(p: &mut Parser, lu: &Lookups, bp: BindingPower) -> Result<Expr, RelixError> {
    let token_kind = p.current_token_kind();

    let nud_fn = lu.nud.get(&token_kind).ok_or_else(|| {
        RelixError::new(format!("NUD Handler expected for token {}\n", token_kind))
    })?;

    let mut left = nud_fn(p, lu)?;

    while lu.bp.get(&p.current_token_kind()).copied().unwrap_or(BindingPower::Default) > bp {
        let token_kind = p.current_token_kind();

        let led_fn = lu.led.get(&token_kind).ok_or_else(|| {
            RelixError::new(format!("LED Handler expected for token {}\n", token_kind))
        })?;

        left = led_fn(p, lu, left, bp)?;
    }

    Ok(left)
}

/// NUD handler for prefix / unary expressions.
///
/// Handles tokens like `-`, `!`, and `typeof` that appear at the start of an
/// expression and take a single operand on the right.
pub fn parse_prefix_expr(p: &mut Parser, lu: &Lookups) -> Result<Expr, RelixError> {
    let operator = p.advance();
    let right = parse_expr(p, lu, BindingPower::Unary)?;
    Ok(Expr::Prefix(PrefixExpr {
        operator,
        right: Box::new(right),
    }))
}

/// LED handler for assignment expressions.
///
/// Handles `=`, `+=`, `-=`, and similar operators. Parses the right-hand side
/// with the same binding power, allowing right-associative chaining.
pub fn parse_assignment_expr(
    p: &mut Parser,
    lu: &Lookups,
    left: Expr,
    bp: BindingPower,
) -> Result<Expr, RelixError> {
    p.advance();
    let rhs = parse_expr(p, lu, bp)?;
    Ok(Expr::Assignment(AssignmentExpr {
        assignee:       Box::new(left),
        assigned_value: Box::new(rhs),
    }))
}

/// LED handler for range expressions.
///
/// Handles the `..` operator (e.g., `1..10`).
pub fn parse_range_expr(
    p: &mut Parser,
    lu: &Lookups,
    left: Expr,
    bp: BindingPower,
) -> Result<Expr, RelixError> {
    p.advance();
    Ok(Expr::Range(RangeExpr {
        lower: Box::new(left),
        upper: Box::new(parse_expr(p, lu, bp)?),
    }))
}

/// LED handler for binary / infix expressions.
///
/// Handles arithmetic (`+`, `-`, `*`, `/`, `%`), comparison (`<`, `==`, etc.),
/// and logical (`&&`, `||`) operators.
pub fn parse_binary_expr(
    p: &mut Parser,
    lu: &Lookups,
    left: Expr,
    _bp: BindingPower,
) -> Result<Expr, RelixError> {
    let operator = p.advance();
    let right = parse_expr(p, lu, BindingPower::Default)?;
    Ok(Expr::Binary(BinaryExpr {
        left:     Box::new(left),
        operator,
        right:    Box::new(right),
    }))
}

/// NUD handler for primary literal expressions.
///
/// Handles numeric literals, string literals, and identifiers.
pub fn parse_primary_expr(p: &mut Parser, _lu: &Lookups) -> Result<Expr, RelixError> {
    match p.current_token_kind() {
        TokenKind::Number => {
            let token = p.advance();
            let value: f64 = token.value.parse().map_err(|_| {
                RelixError::new(format!("Invalid number literal: {}", token.value))
            })?;
            Ok(Expr::Number(crate::ast::NumberExpr { value }))
        }
        TokenKind::String => Ok(Expr::String(crate::ast::StringExpr {
            value: p.advance().value,
        })),
        TokenKind::Identifier => Ok(Expr::Symbol(crate::ast::SymbolExpr {
            value: p.advance().value,
        })),
        other => Err(RelixError::new(format!(
            "Cannot create primary_expr from {}\n",
            other
        ))),
    }
}

/// LED handler for member access and computed access expressions.
///
/// Handles `.` (member access, e.g., `obj.field`) and `[` (computed access,
/// e.g., `arr[0]`).
pub fn parse_member_expr(
    p: &mut Parser,
    lu: &Lookups,
    left: Expr,
    bp: BindingPower,
) -> Result<Expr, RelixError> {
    let is_computed = p.advance().kind == TokenKind::OpenBracket;

    if is_computed {
        let rhs = parse_expr(p, lu, bp)?;
        p.expect(TokenKind::CloseBracket)?;
        Ok(Expr::Computed(ComputedExpr {
            member:   Box::new(left),
            property: Box::new(rhs),
        }))
    } else {
        Ok(Expr::Member(MemberExpr {
            member:   Box::new(left),
            property: p.expect(TokenKind::Identifier)?.value,
        }))
    }
}

/// NUD handler for array literal expressions.
///
/// Handles `[` followed by a comma-separated list of expressions and `]`.
pub fn parse_array_literal_expr(p: &mut Parser, lu: &Lookups) -> Result<Expr, RelixError> {
    p.expect(TokenKind::OpenBracket)?;
    let mut contents: Vec<Expr> = Vec::new();

    while p.has_tokens() && p.current_token_kind() != TokenKind::CloseBracket {
        contents.push(parse_expr(p, lu, BindingPower::Logical)?);
        if !p.current_token().is_one_of_many(&[TokenKind::Eof, TokenKind::CloseBracket]) {
            p.expect(TokenKind::Comma)?;
        }
    }

    p.expect(TokenKind::CloseBracket)?;
    Ok(Expr::Array(ArrayLiteral { contents }))
}

/// NUD handler for grouping expressions.
///
/// Handles `(` followed by an expression and `)`, used for overriding operator
/// precedence.
pub fn parse_grouping_expr(p: &mut Parser, lu: &Lookups) -> Result<Expr, RelixError> {
    p.expect(TokenKind::OpenParen)?;
    let expr = parse_expr(p, lu, BindingPower::Default)?;
    p.expect(TokenKind::CloseParen)?;
    Ok(expr)
}

/// LED handler for function call expressions.
///
/// Handles `(` followed by a comma-separated list of argument expressions and
/// `)`.
pub fn parse_call_expr(
    p: &mut Parser,
    lu: &Lookups,
    left: Expr,
    _bp: BindingPower,
) -> Result<Expr, RelixError> {
    p.advance();
    let mut arguments: Vec<Expr> = Vec::new();

    while p.has_tokens() && p.current_token_kind() != TokenKind::CloseParen {
        arguments.push(parse_expr(p, lu, BindingPower::Assignment)?);
        if !p.current_token().is_one_of_many(&[TokenKind::Eof, TokenKind::CloseParen]) {
            p.expect(TokenKind::Comma)?;
        }
    }

    p.expect(TokenKind::CloseParen)?;
    Ok(Expr::Call(CallExpr {
        method:    Box::new(left),
        arguments,
    }))
}

/// NUD handler for anonymous function expressions.
///
/// Handles `fn` followed by a parameter list, optional return type, and body.
pub fn parse_fn_expr(p: &mut Parser, lu: &Lookups) -> Result<Expr, RelixError> {
    p.expect(TokenKind::Fn)?;
    let (parameters, return_type, body) = parse_fn_params_and_body(p, lu)?;
    Ok(Expr::Function(FunctionExpr {
        parameters,
        return_type,
        body,
    }))
}

/// NUD handler for `new` instantiation expressions.
///
/// Handles `new` followed by a call expression (e.g., `new Point()`).
pub fn parse_new_expr(p: &mut Parser, lu: &Lookups) -> Result<Expr, RelixError> {
    p.advance();
    let instantiation = parse_expr(p, lu, BindingPower::Default)?;
    let call = match instantiation {
        Expr::Call(c) => c,
        _ => return Err(RelixError::new("Expected call expression after `new`")),
    };
    Ok(Expr::New(NewExpr {
        instantiation: call,
    }))
}

/// Shared helper for parsing function parameters, return type, and body.
///
/// Used by both [`parse_fn_expr`] (anonymous functions) and
/// [`parse_fn_declaration`](crate::stmts::parse_fn_declaration) (named function
/// declarations).
///
/// Expects the cursor to be positioned at the opening `(` of the parameter
/// list. Returns a tuple of (parameters, return_type, body).
pub fn parse_fn_params_and_body(
    p: &mut Parser,
    lu: &Lookups,
) -> Result<FnParamsAndBody, RelixError> {
    let mut params: Vec<Parameter> = Vec::new();

    p.expect(TokenKind::OpenParen)?;
    while p.has_tokens() && p.current_token_kind() != TokenKind::CloseParen {
        let param_name = p.expect(TokenKind::Identifier)?.value;
        p.expect(TokenKind::Colon)?;
        let param_type = parse_type(p, lu, BindingPower::Default)?;

        params.push(Parameter {
            name: param_name,
            ty:   param_type,
        });

        if !p.current_token().is_one_of_many(&[TokenKind::CloseParen, TokenKind::Eof]) {
            p.expect(TokenKind::Comma)?;
        }
    }
    p.expect(TokenKind::CloseParen)?;

    let return_type = if p.current_token_kind() == TokenKind::Colon {
        p.advance();
        Some(parse_type(p, lu, BindingPower::Default)?)
    } else {
        None
    };

    let body = match parse_block_stmt(p, lu)? {
        crate::ast::Stmt::Block(b) => b.body,
        _ => return Err(RelixError::new("Expected block statement for function body")),
    };

    Ok((params, return_type, body))
}
