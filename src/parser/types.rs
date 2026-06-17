use crate::ast::{ListType, SymbolType, Type};
use crate::lexer::TokenKind;
use crate::lookups::{BindingPower, Lookups};
use crate::parser::Parser;

// ---------------------------------------------------------------------------
// Core Pratt type driver
// ---------------------------------------------------------------------------

pub fn parse_type(p: &mut Parser, lu: &Lookups, bp: BindingPower) -> Type {
    let token_kind = p.current_token_kind();

    let nud_fn = lu.type_nud.get(&token_kind).unwrap_or_else(|| {
        panic!("type: NUD Handler expected for token {}\n", token_kind)
    });

    let mut left = nud_fn(p, lu);

    while lu
        .type_bp
        .get(&p.current_token_kind())
        .copied()
        .unwrap_or(BindingPower::Default)
        > bp
    {
        let token_kind = p.current_token_kind();

        let led_fn = lu.type_led.get(&token_kind).unwrap_or_else(|| {
            panic!("type: LED Handler expected for token {}\n", token_kind)
        });

        left = led_fn(p, lu, left, bp);
    }

    left
}

// ---------------------------------------------------------------------------
// Type NUD handlers (registered in lookups.rs)
// ---------------------------------------------------------------------------

/// `identifier` → SymbolType
pub fn parse_symbol_type(p: &mut Parser, _lu: &Lookups) -> Type {
    Type::Symbol(SymbolType {
        value: p.advance().value,
    })
}

/// `[]T` → ListType
pub fn parse_list_type(p: &mut Parser, lu: &Lookups) -> Type {
    p.advance();                         // consume `[`
    p.expect(TokenKind::CloseBracket);   // consume `]`
    let inner = parse_type(p, lu, BindingPower::Default);
    Type::List(ListType {
        underlying: Box::new(inner),
    })
}
