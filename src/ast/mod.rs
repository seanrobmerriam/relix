//! Abstract syntax tree (AST) node definitions for the Relix language.
//!
//! This module defines the data structures that represent parsed Relix source
//! code. Every construct the parser can produce is represented as a variant of
//! one of three top-level enums:
//!
//! - [`Expr`] — expression nodes (literals, binary operations, calls, etc.)
//! - [`Stmt`] — statement nodes (variable declarations, functions, etc.)
//! - [`Type`] — type annotation nodes (symbol types, list types)
//!
//! Each enum variant wraps a dedicated struct that holds the node's fields.
//! This design keeps the enums compact while allowing each node type to evolve
//! independently.

use crate::lexer::Token;

/// An expression node in the AST.
///
/// Expressions produce values and can be nested arbitrarily. The parser builds
/// these nodes using Pratt parsing with configurable binding powers.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A numeric literal (e.g., `42`, `3.14`).
    Number(NumberExpr),
    /// A string literal (e.g., `"hello"`).
    String(StringExpr),
    /// A symbol / identifier reference (e.g., `x`, `foo`).
    Symbol(SymbolExpr),
    /// A binary operation (e.g., `a + b`, `x == y`).
    Binary(BinaryExpr),
    /// An assignment expression (e.g., `x = 10`, `x += 1`).
    Assignment(AssignmentExpr),
    /// A prefix / unary operation (e.g., `-x`, `!flag`, `typeof x`).
    Prefix(PrefixExpr),
    /// A member access expression (e.g., `obj.field`).
    Member(MemberExpr),
    /// A function call expression (e.g., `foo(1, 2)`).
    Call(CallExpr),
    /// A computed / indexed access expression (e.g., `arr[0]`).
    Computed(ComputedExpr),
    /// A range expression (e.g., `1..10`).
    Range(RangeExpr),
    /// An anonymous function expression (e.g., `fn(x: number) { return x; }`).
    Function(FunctionExpr),
    /// An array literal (e.g., `[1, 2, 3]`).
    Array(ArrayLiteral),
    /// A `new` instantiation expression (e.g., `new Point()`).
    New(NewExpr),
}

/// A statement node in the AST.
///
/// Statements perform actions but do not produce values. They typically appear
/// at the top level of a program or within block statements.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// A block of statements enclosed in braces (e.g., `{ ... }`).
    Block(BlockStmt),
    /// A variable declaration (e.g., `let x = 10;`, `const y: number;`).
    VarDecl(VarDeclarationStmt),
    /// An expression used as a statement (e.g., `foo();`).
    Expression(ExpressionStmt),
    /// A function declaration (e.g., `fn add(a: number, b: number): number { ... }`).
    FunctionDecl(FunctionDeclarationStmt),
    /// An `if` / `else` statement.
    If(IfStmt),
    /// An `import` statement (e.g., `import foo from "bar";`).
    Import(ImportStmt),
    /// A `foreach` loop statement (e.g., `foreach item in items { ... }`).
    Foreach(ForeachStmt),
    /// A class declaration (e.g., `class Point { ... }`).
    ClassDecl(ClassDeclarationStmt),
}

/// A type annotation node in the AST.
///
/// Type annotations appear in variable declarations, function parameters, and
/// function return types.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// A symbol type (e.g., `number`, `string`, `MyClass`).
    Symbol(SymbolType),
    /// A list type (e.g., `[]number`, `[]string`).
    List(ListType),
}

/// A numeric literal expression.
#[derive(Debug, Clone, PartialEq)]
pub struct NumberExpr {
    /// The numeric value.
    pub value: f64,
}

/// A string literal expression.
#[derive(Debug, Clone, PartialEq)]
pub struct StringExpr {
    /// The string value (including quotes).
    pub value: String,
}

/// A symbol / identifier reference expression.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolExpr {
    /// The identifier name.
    pub value: String,
}

/// A binary operation expression.
///
/// Represents operations like `a + b`, `x == y`, `flag && other`.
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    /// The left-hand operand.
    pub left: Box<Expr>,
    /// The operator token (e.g., `+`, `==`, `&&`).
    pub operator: Token,
    /// The right-hand operand.
    pub right: Box<Expr>,
}

/// An assignment expression.
///
/// Represents assignments like `x = 10`, `x += 1`, `x ??= default`.
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpr {
    /// The target being assigned to (typically a symbol or member access).
    pub assignee: Box<Expr>,
    /// The value being assigned.
    pub assigned_value: Box<Expr>,
}

/// A prefix / unary operation expression.
///
/// Represents operations like `-x`, `!flag`, `typeof x`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrefixExpr {
    /// The operator token (e.g., `-`, `!`, `typeof`).
    pub operator: Token,
    /// The operand.
    pub right: Box<Expr>,
}

/// A member access expression.
///
/// Represents dotted access like `obj.field`.
#[derive(Debug, Clone, PartialEq)]
pub struct MemberExpr {
    /// The object being accessed.
    pub member: Box<Expr>,
    /// The property name being accessed.
    pub property: String,
}

/// A function call expression.
///
/// Represents calls like `foo(1, 2)` or `obj.method()`.
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    /// The function or method being called.
    pub method: Box<Expr>,
    /// The arguments passed to the function.
    pub arguments: Vec<Expr>,
}

/// A computed / indexed access expression.
///
/// Represents bracket access like `arr[0]` or `map["key"]`.
#[derive(Debug, Clone, PartialEq)]
pub struct ComputedExpr {
    /// The object being indexed.
    pub member: Box<Expr>,
    /// The index expression.
    pub property: Box<Expr>,
}

/// A range expression.
///
/// Represents ranges like `1..10`.
#[derive(Debug, Clone, PartialEq)]
pub struct RangeExpr {
    /// The lower bound of the range.
    pub lower: Box<Expr>,
    /// The upper bound of the range.
    pub upper: Box<Expr>,
}

/// An anonymous function expression.
///
/// Represents inline function definitions like `fn(x: number) { return x; }`.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionExpr {
    /// The function parameters.
    pub parameters: Vec<Parameter>,
    /// The function body (a list of statements).
    pub body: Vec<Stmt>,
    /// The optional return type annotation.
    pub return_type: Option<Type>,
}

/// An array literal expression.
///
/// Represents array literals like `[1, 2, 3]`.
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayLiteral {
    /// The elements of the array.
    pub contents: Vec<Expr>,
}

/// A `new` instantiation expression.
///
/// Represents object instantiation like `new Point()`.
#[derive(Debug, Clone, PartialEq)]
pub struct NewExpr {
    /// The constructor call being instantiated.
    pub instantiation: CallExpr,
}

/// A block statement containing a sequence of statements.
///
/// Represents a brace-delimited block like `{ let x = 1; let y = 2; }`.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt {
    /// The statements in the block.
    pub body: Vec<Stmt>,
}

/// A variable declaration statement.
///
/// Represents declarations like `let x = 10;`, `const y: number = 42;`, or
/// `let z: string;`.
#[derive(Debug, Clone, PartialEq)]
pub struct VarDeclarationStmt {
    /// The variable name.
    pub identifier: String,
    /// Whether this is a constant declaration (`const` vs `let`).
    pub constant: bool,
    /// The optional initial value.
    pub assigned_value: Option<Expr>,
    /// The optional explicit type annotation.
    pub explicit_type: Option<Type>,
}

/// An expression used as a statement.
///
/// Wraps an expression that appears in statement position (e.g., `foo();`).
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionStmt {
    /// The expression.
    pub expression: Expr,
}

/// A function parameter.
///
/// Represents a single parameter in a function declaration or expression.
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    /// The parameter name.
    pub name: String,
    /// The parameter type annotation.
    pub ty: Type,
}

/// A function declaration statement.
///
/// Represents named function definitions like `fn add(a: number, b: number): number { ... }`.
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDeclarationStmt {
    /// The function name.
    pub name: String,
    /// The function parameters.
    pub parameters: Vec<Parameter>,
    /// The function body (a list of statements).
    pub body: Vec<Stmt>,
    /// The optional return type annotation.
    pub return_type: Option<Type>,
}

/// An `if` / `else` statement.
///
/// Represents conditional branching like `if x > 5 { ... } else { ... }`.
#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    /// The condition expression.
    pub condition: Expr,
    /// The consequent (then) branch.
    pub consequent: Box<Stmt>,
    /// The optional alternate (else) branch.
    pub alternate: Option<Box<Stmt>>,
}

/// An `import` statement.
///
/// Represents imports like `import foo from "bar";` or `import baz;`.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportStmt {
    /// The name being imported.
    pub name: String,
    /// The module path being imported from.
    pub from: String,
}

/// A `foreach` loop statement.
///
/// Represents iteration like `foreach item in items { ... }` or
/// `foreach item, index in items { ... }`.
#[derive(Debug, Clone, PartialEq)]
pub struct ForeachStmt {
    /// The loop variable name.
    pub value: String,
    /// Whether an index variable is also declared.
    pub index: bool,
    /// The iterable expression being looped over.
    pub iterable: Expr,
    /// The loop body (a list of statements).
    pub body: Vec<Stmt>,
}

/// A class declaration statement.
///
/// Represents class definitions like `class Point { let x: number; let y: number; }`.
#[derive(Debug, Clone, PartialEq)]
pub struct ClassDeclarationStmt {
    /// The class name.
    pub name: String,
    /// The class body (a list of statements, typically field declarations).
    pub body: Vec<Stmt>,
}

/// A symbol type annotation.
///
/// Represents simple type names like `number`, `string`, or `MyClass`.
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolType {
    /// The type name.
    pub value: String,
}

/// A list type annotation.
///
/// Represents list / array types like `[]number` or `[]string`.
#[derive(Debug, Clone, PartialEq)]
pub struct ListType {
    /// The element type of the list.
    pub underlying: Box<Type>,
}
