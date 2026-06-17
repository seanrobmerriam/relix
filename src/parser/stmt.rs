use crate::ast::{
    BlockStmt, ClassDeclarationStmt, ExpressionStmt, ForeachStmt, FunctionDeclarationStmt,
    IfStmt, ImportStmt, Stmt, VarDeclarationStmt,
};
use crate::exprs::{parse_expr, parse_fn_params_and_body};
use crate::lexer::TokenKind;
use crate::lookups::{BindingPower, Lookups};
use crate::parser::Parser;
use crate::types::parse_type;

// ---------------------------------------------------------------------------
// Statement dispatcher
// ---------------------------------------------------------------------------

pub fn parse_stmt(p: &mut Parser, lu: &Lookups) -> Stmt {
    if let Some(&handler) = lu.stmt.get(&p.current_token_kind()) {
        return handler(p, lu);
    }
    parse_expression_stmt(p, lu)
}

// ---------------------------------------------------------------------------
// Expression statement
// ---------------------------------------------------------------------------

pub fn parse_expression_stmt(p: &mut Parser, lu: &Lookups) -> Stmt {
    let expression = parse_expr(p, lu, BindingPower::Default);
    p.expect(TokenKind::SemiColon);
    Stmt::Expression(ExpressionStmt { expression })
}

// ---------------------------------------------------------------------------
// Block statement
// ---------------------------------------------------------------------------

pub fn parse_block_stmt(p: &mut Parser, lu: &Lookups) -> Stmt {
    p.expect(TokenKind::OpenCurly);
    let mut body: Vec<Stmt> = Vec::new();

    while p.has_tokens() && p.current_token_kind() != TokenKind::CloseCurly {
        body.push(parse_stmt(p, lu));
    }

    p.expect(TokenKind::CloseCurly);
    Stmt::Block(BlockStmt { body })
}

// ---------------------------------------------------------------------------
// Variable declaration
// ---------------------------------------------------------------------------

pub fn parse_var_decl_stmt(p: &mut Parser, lu: &Lookups) -> Stmt {
    let start_token = p.advance();
    let is_constant = start_token.kind == TokenKind::Const;

    let symbol_name = p.expect_error(
        TokenKind::Identifier,
        Some(format!(
            "Following {} expected variable name however instead received {} instead\n",
            start_token.kind,
            p.current_token_kind(),
        )),
    );

    let explicit_type = if p.current_token_kind() == TokenKind::Colon {
        p.expect(TokenKind::Colon);
        Some(parse_type(p, lu, BindingPower::Default))
    } else {
        None
    };

    let assigned_value = if p.current_token_kind() != TokenKind::SemiColon {
        p.expect(TokenKind::Assignment);
        Some(parse_expr(p, lu, BindingPower::Assignment))
    } else {
        if explicit_type.is_none() {
            panic!("Missing explicit type for variable declaration.");
        }
        None
    };

    p.expect(TokenKind::SemiColon);

    if is_constant && assigned_value.is_none() {
        panic!("Cannot define constant variable without providing default value.");
    }

    Stmt::VarDecl(VarDeclarationStmt {
        constant:       is_constant,
        identifier:     symbol_name.value,
        assigned_value,
        explicit_type,
    })
}

// ---------------------------------------------------------------------------
// Function declaration
// ---------------------------------------------------------------------------

pub fn parse_fn_declaration(p: &mut Parser, lu: &Lookups) -> Stmt {
    p.advance(); // consume `fn`
    let function_name = p.expect(TokenKind::Identifier).value;
    let (params, return_type, body) = parse_fn_params_and_body(p, lu);

    Stmt::FunctionDecl(FunctionDeclarationStmt {
        name:        function_name,
        parameters:  params,
        return_type,
        body,
    })
}

// ---------------------------------------------------------------------------
// If statement
// ---------------------------------------------------------------------------

pub fn parse_if_stmt(p: &mut Parser, lu: &Lookups) -> Stmt {
    p.advance(); // consume `if`
    let condition  = parse_expr(p, lu, BindingPower::Assignment);
    let consequent = parse_block_stmt(p, lu);

    let alternate = if p.current_token_kind() == TokenKind::Else {
        p.advance();
        if p.current_token_kind() == TokenKind::If {
            Some(Box::new(parse_if_stmt(p, lu)))
        } else {
            Some(Box::new(parse_block_stmt(p, lu)))
        }
    } else {
        None
    };

    Stmt::If(IfStmt {
        condition,
        consequent: Box::new(consequent),
        alternate,
    })
}

// ---------------------------------------------------------------------------
// Import statement
// ---------------------------------------------------------------------------

pub fn parse_import_stmt(p: &mut Parser, _lu: &Lookups) -> Stmt {
    p.advance(); // consume `import`
    let import_name = p.expect(TokenKind::Identifier).value;

    let import_from = if p.current_token_kind() == TokenKind::From {
        p.advance();
        p.expect(TokenKind::String).value
    } else {
        import_name.clone()
    };

    p.expect(TokenKind::SemiColon);
    Stmt::Import(ImportStmt {
        name: import_name,
        from: import_from,
    })
}

// ---------------------------------------------------------------------------
// Foreach statement
// ---------------------------------------------------------------------------

pub fn parse_foreach_stmt(p: &mut Parser, lu: &Lookups) -> Stmt {
    p.advance(); // consume `foreach`
    let value_name = p.expect(TokenKind::Identifier).value;

    let has_index = if p.current_token_kind() == TokenKind::Comma {
        p.expect(TokenKind::Comma);
        p.expect(TokenKind::Identifier);
        true
    } else {
        false
    };

    p.expect(TokenKind::In);
    let iterable = parse_expr(p, lu, BindingPower::Default);

    let body = match parse_block_stmt(p, lu) {
        Stmt::Block(b) => b.body,
        _ => panic!("Expected block statement in foreach body"),
    };

    Stmt::Foreach(ForeachStmt {
        value:    value_name,
        index:    has_index,
        iterable,
        body,
    })
}

// ---------------------------------------------------------------------------
// Class declaration
// ---------------------------------------------------------------------------

pub fn parse_class_declaration_stmt(p: &mut Parser, lu: &Lookups) -> Stmt {
    p.advance(); // consume `class`
    let class_name = p.expect(TokenKind::Identifier).value;

    let body = match parse_block_stmt(p, lu) {
        Stmt::Block(b) => b.body,
        _ => panic!("Expected block statement for class body"),
    };

    Stmt::ClassDecl(ClassDeclarationStmt {
        name: class_name,
        body,
    })
}
