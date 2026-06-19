//! Type annotation parsing for the Relix language.
//!
//! This module implements a Pratt parsing driver for type annotations, along
//! with NUD and LED handlers for the supported type forms.
//!
//! Currently supported type forms:
//!
//! - **Symbol types**: simple type names like `number`, `string`, `MyClass`.
//! - **List types**: array / list types like `[]number`, `[]string`.

use crate::ast::{ListType, SymbolType, Type};
use crate::error::RelixError;
use crate::lexer::TokenKind;
use crate::lookups::{BindingPower, Lookups};
use crate::parser::Parser;

/// The core Pratt type parsing driver.
///
/// Parses a type annotation starting at the current token position, using the
/// NUD and LED handler tables from `lu` to handle each token. The `bp`
/// parameter controls the binding power threshold.
///
/// This function mirrors the structure of
/// [`parse_expr`](crate::exprs::parse_expr) but operates on type nodes instead
/// of expression nodes.
pub fn parse_type(p: &mut Parser, lu: &Lookups, bp: BindingPower) -> Result<Type, RelixError> {
    let token_kind = p.current_token_kind();

    let nud_fn = lu.type_nud.get(&token_kind).ok_or_else(|| {
        RelixError::new(format!("type: NUD Handler expected for token {}\n", token_kind))
    })?;

    let mut left = nud_fn(p, lu)?;

    while lu
        .type_bp
        .get(&p.current_token_kind())
        .copied()
        .unwrap_or(BindingPower::Default)
        > bp
    {
        let token_kind = p.current_token_kind();

        let led_fn = lu.type_led.get(&token_kind).ok_or_else(|| {
            RelixError::new(format!("type: LED Handler expected for token {}\n", token_kind))
        })?;

        left = led_fn(p, lu, left, bp)?;
    }

    Ok(left)
}

/// NUD handler for symbol type annotations.
///
/// Parses a simple type name (e.g., `number`, `string`, `MyClass`) from an
/// identifier token.
pub fn parse_symbol_type(p: &mut Parser, _lu: &Lookups) -> Result<Type, RelixError> {
    Ok(Type::Symbol(SymbolType {
        value: p.advance().value,
    }))
}

/// NUD handler for list type annotations.
///
/// Parses a list type of the form `[]T` (e.g., `[]number`, `[]string`).
pub fn parse_list_type(p: &mut Parser, lu: &Lookups) -> Result<Type, RelixError> {
    p.advance();
    p.expect(TokenKind::CloseBracket)?;
    let inner = parse_type(p, lu, BindingPower::Default)?;
    Ok(Type::List(ListType {
        underlying: Box::new(inner),
    }))
}
