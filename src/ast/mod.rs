use crate::lexer::Token;

// ---------------------------------------------------------------------------
// Trait-marker enums (Go interfaces → Rust enums)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum Expr {
    Number(NumberExpr),
    String(StringExpr),
    Symbol(SymbolExpr),
    Binary(BinaryExpr),
    Assignment(AssignmentExpr),
    Prefix(PrefixExpr),
    Member(MemberExpr),
    Call(CallExpr),
    Computed(ComputedExpr),
    Range(RangeExpr),
    Function(FunctionExpr),
    Array(ArrayLiteral),
    New(NewExpr),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Block(BlockStmt),
    VarDecl(VarDeclarationStmt),
    Expression(ExpressionStmt),
    FunctionDecl(FunctionDeclarationStmt),
    If(IfStmt),
    Import(ImportStmt),
    Foreach(ForeachStmt),
    ClassDecl(ClassDeclarationStmt),
}

#[derive(Debug, Clone)]
pub enum Type {
    Symbol(SymbolType),
    List(ListType),
}

// ---------------------------------------------------------------------------
// Literal expressions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct NumberExpr {
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct StringExpr {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct SymbolExpr {
    pub value: String,
}

// ---------------------------------------------------------------------------
// Complex expressions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct AssignmentExpr {
    pub assignee: Box<Expr>,
    pub assigned_value: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct PrefixExpr {
    pub operator: Token,
    pub right: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct MemberExpr {
    pub member: Box<Expr>,
    pub property: String,
}

#[derive(Debug, Clone)]
pub struct CallExpr {
    pub method: Box<Expr>,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct ComputedExpr {
    pub member: Box<Expr>,
    pub property: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct RangeExpr {
    pub lower: Box<Expr>,
    pub upper: Box<Expr>,
}

#[derive(Debug, Clone)]
pub struct FunctionExpr {
    pub parameters: Vec<Parameter>,
    pub body: Vec<Stmt>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub contents: Vec<Expr>,
}

#[derive(Debug, Clone)]
pub struct NewExpr {
    pub instantiation: CallExpr,
}

// ---------------------------------------------------------------------------
// Statements
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BlockStmt {
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct VarDeclarationStmt {
    pub identifier: String,
    pub constant: bool,
    pub assigned_value: Option<Expr>,
    pub explicit_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct ExpressionStmt {
    pub expression: Expr,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct FunctionDeclarationStmt {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub body: Vec<Stmt>,
    pub return_type: Option<Type>,
}

#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub consequent: Box<Stmt>,
    pub alternate: Option<Box<Stmt>>,
}

#[derive(Debug, Clone)]
pub struct ImportStmt {
    pub name: String,
    pub from: String,
}

#[derive(Debug, Clone)]
pub struct ForeachStmt {
    pub value: String,
    pub index: bool,
    pub iterable: Expr,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct ClassDeclarationStmt {
    pub name: String,
    pub body: Vec<Stmt>,
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct SymbolType {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct ListType {
    pub underlying: Box<Type>,
}
