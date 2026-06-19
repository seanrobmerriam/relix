//! Statement parsing for the Relix language.
//!
//! This module implements handlers for all statement forms: variable
//! declarations, function declarations, control flow (`if`, `foreach`),
//! imports, class declarations, and block statements.
//!
//! The main entry point is [`parse_stmt`], which dispatches to the appropriate
//! handler based on the current token kind using the statement handler table
//! from [`Lookups`].

use crate::ast::{
    BlockStmt, ClassDeclarationStmt, ExpressionStmt, ForeachStmt, FunctionDeclarationStmt,
    IfStmt, ImportStmt, Stmt, VarDeclarationStmt,
};
use crate::error::RelixError;
use crate::exprs::{parse_expr, parse_fn_params_and_body};
use crate::lexer::TokenKind;
use crate::lookups::{BindingPower, Lookups};
use crate::parser::Parser;
use crate::types::parse_type;

/// Statement dispatcher.
///
/// Examines the current token kind and dispatches to the appropriate statement
/// handler from the `lu.stmt` table. If no handler is registered for the
/// current token, falls back to [`parse_expression_stmt`].
pub fn parse_stmt(p: &mut Parser, lu: &Lookups) -> Result<Stmt, RelixError> {
    if let Some(&handler) = lu.stmt.get(&p.current_token_kind()) {
        return handler(p, lu);
    }
    parse_expression_stmt(p, lu)
}

/// Parses an expression statement.
///
/// An expression statement is an expression followed by a semicolon (e.g.,
/// `foo();`).
pub fn parse_expression_stmt(p: &mut Parser, lu: &Lookups) -> Result<Stmt, RelixError> {
    let expression = parse_expr(p, lu, BindingPower::Default)?;
    p.expect(TokenKind::SemiColon)?;
    Ok(Stmt::Expression(ExpressionStmt { expression }))
}

/// Parses a block statement.
///
/// A block statement is a sequence of statements enclosed in braces (e.g.,
/// `{ let x = 1; let y = 2; }`). Expects the cursor to be positioned at the
/// opening `{`.
pub fn parse_block_stmt(p: &mut Parser, lu: &Lookups) -> Result<Stmt, RelixError> {
    p.expect(TokenKind::OpenCurly)?;
    let mut body: Vec<Stmt> = Vec::new();

    while p.has_tokens() && p.current_token_kind() != TokenKind::CloseCurly {
        body.push(parse_stmt(p, lu)?);
    }

    p.expect(TokenKind::CloseCurly)?;
    Ok(Stmt::Block(BlockStmt { body }))
}

/// Parses a variable declaration statement.
///
/// Handles both `let` and `const` declarations. Supports optional type
/// annotations and initial values.
///
/// # Syntax
///
/// ```text
/// let x = 10;
/// const y: number = 42;
/// let z: string;
/// ```
///
/// # Errors
///
/// - Returns an error if the variable name is missing.
/// - Returns an error if neither an explicit type nor an initial value is provided.
/// - Returns an error if a `const` declaration has no initial value.
pub fn parse_var_decl_stmt(p: &mut Parser, lu: &Lookups) -> Result<Stmt, RelixError> {
    let start_token = p.advance();
    let is_constant = start_token.kind == TokenKind::Const;

    let symbol_name = p.expect_error(
        TokenKind::Identifier,
        Some(format!(
            "Following {} expected variable name however instead received {} instead\n",
            start_token.kind,
            p.current_token_kind(),
        )),
    )?;

    let explicit_type = if p.current_token_kind() == TokenKind::Colon {
        p.expect(TokenKind::Colon)?;
        Some(parse_type(p, lu, BindingPower::Default)?)
    } else {
        None
    };

    let assigned_value = if p.current_token_kind() != TokenKind::SemiColon {
        p.expect(TokenKind::Assignment)?;
        Some(parse_expr(p, lu, BindingPower::Assignment)?)
    } else {
        if explicit_type.is_none() {
            return Err(RelixError::new("Missing explicit type for variable declaration."));
        }
        None
    };

    p.expect(TokenKind::SemiColon)?;

    if is_constant && assigned_value.is_none() {
        return Err(RelixError::new("Cannot define constant variable without providing default value."));
    }

    Ok(Stmt::VarDecl(VarDeclarationStmt {
        constant:       is_constant,
        identifier:     symbol_name.value,
        assigned_value,
        explicit_type,
    }))
}

/// Parses a function declaration statement.
///
/// # Syntax
///
/// ```text
/// fn add(a: number, b: number): number {
///     return a + b;
/// }
/// ```
pub fn parse_fn_declaration(p: &mut Parser, lu: &Lookups) -> Result<Stmt, RelixError> {
    p.advance();
    let function_name = p.expect(TokenKind::Identifier)?.value;
    let (params, return_type, body) = parse_fn_params_and_body(p, lu)?;

    Ok(Stmt::FunctionDecl(FunctionDeclarationStmt {
        name:        function_name,
        parameters:  params,
        return_type,
        body,
    }))
}

/// Parses an `if` / `else` statement.
///
/// Supports `else if` chains by recursively parsing nested `if` statements in
/// the alternate branch.
///
/// # Syntax
///
/// ```text
/// if x > 5 {
///     let y = x * 2;
/// } else if x > 0 {
///     let y = 0;
/// } else {
///     let y = -1;
/// }
/// ```
pub fn parse_if_stmt(p: &mut Parser, lu: &Lookups) -> Result<Stmt, RelixError> {
    p.advance();
    let condition  = parse_expr(p, lu, BindingPower::Assignment)?;
    let consequent = parse_block_stmt(p, lu)?;

    let alternate = if p.current_token_kind() == TokenKind::Else {
        p.advance();
        if p.current_token_kind() == TokenKind::If {
            Some(Box::new(parse_if_stmt(p, lu)?))
        } else {
            Some(Box::new(parse_block_stmt(p, lu)?))
        }
    } else {
        None
    };

    Ok(Stmt::If(IfStmt {
        condition,
        consequent: Box::new(consequent),
        alternate,
    }))
}

/// Parses an `import` statement.
///
/// # Syntax
///
/// ```text
/// import foo from "bar";
/// import baz;
/// ```
///
/// When the `from` clause is omitted, the `from` field is set to the same
/// value as the `name` field.
pub fn parse_import_stmt(p: &mut Parser, _lu: &Lookups) -> Result<Stmt, RelixError> {
    p.advance();
    let import_name = p.expect(TokenKind::Identifier)?.value;

    let import_from = if p.current_token_kind() == TokenKind::From {
        p.advance();
        p.expect(TokenKind::String)?.value
    } else {
        import_name.clone()
    };

    p.expect(TokenKind::SemiColon)?;
    Ok(Stmt::Import(ImportStmt {
        name: import_name,
        from: import_from,
    }))
}

/// Parses a `foreach` loop statement.
///
/// Supports an optional index variable (separated by a comma from the value
/// variable).
///
/// # Syntax
///
/// ```text
/// foreach item in items {
///     print(item);
/// }
///
/// foreach item, index in items {
///     print(index, item);
/// }
/// ```
pub fn parse_foreach_stmt(p: &mut Parser, lu: &Lookups) -> Result<Stmt, RelixError> {
    p.advance();
    let value_name = p.expect(TokenKind::Identifier)?.value;

    let has_index = if p.current_token_kind() == TokenKind::Comma {
        p.expect(TokenKind::Comma)?;
        p.expect(TokenKind::Identifier)?;
        true
    } else {
        false
    };

    p.expect(TokenKind::In)?;
    let iterable = parse_expr(p, lu, BindingPower::Default)?;

    let body = match parse_block_stmt(p, lu)? {
        Stmt::Block(b) => b.body,
        _ => return Err(RelixError::new("Expected block statement in foreach body")),
    };

    Ok(Stmt::Foreach(ForeachStmt {
        value:    value_name,
        index:    has_index,
        iterable,
        body,
    }))
}

/// Parses a class declaration statement.
///
/// # Syntax
///
/// ```text
/// class Point {
///     let x: number;
///     let y: number;
/// }
/// ```
pub fn parse_class_declaration_stmt(p: &mut Parser, lu: &Lookups) -> Result<Stmt, RelixError> {
    p.advance();
    let class_name = p.expect(TokenKind::Identifier)?.value;

    let body = match parse_block_stmt(p, lu)? {
        Stmt::Block(b) => b.body,
        _ => return Err(RelixError::new("Expected block statement for class body")),
    };

    Ok(Stmt::ClassDecl(ClassDeclarationStmt {
        name: class_name,
        body,
    }))
}
