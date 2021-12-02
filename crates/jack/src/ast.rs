use crate::Ident;
pub use parser::*;

mod parser;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Class {
    pub name: Ident,
    pub vars: Vec<ClassVar>,
    pub subs: Vec<Subroutine>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassVar {
    pub kind: ClassVarKind,
    pub ty: Type,
    pub var_names: Vec<Ident>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClassVarKind {
    Static,
    Field,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Int,
    Char,
    Boolean,
    Class(Ident),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subroutine {
    pub kind: SubroutineKind,
    pub return_type: Option<Type>,
    pub name: Ident,
    pub params: Vec<Parameter>,
    pub body: SubroutineBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubroutineKind {
    Constructor,
    Function,
    Method,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parameter {
    pub ty: Type,
    pub var_name: Ident,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubroutineBody {
    pub vars: Vec<Var>,
    pub stmts: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Var {
    pub ty: Type,
    pub names: Vec<Ident>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Let(LetStatement),
    If(IfStatement),
    While(WhileStatement),
    Do(DoStatement),
    Return(ReturnStatement),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LetStatement {
    pub var_name: Ident,
    pub index: Option<Expression>,
    pub expr: Expression,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IfStatement {
    pub cond: Expression,
    pub then_stmts: Vec<Statement>,
    pub else_stmts: Option<Vec<Statement>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhileStatement {
    pub cond: Expression,
    pub stmts: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoStatement {
    pub sub_call: SubroutineCall,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStatement {
    pub expr: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expression {
    pub term: Term,
    pub binary_ops: Vec<(BinaryOp, Term)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Term {
    IntConstant(u16),
    StringConstant(String),
    KeywordConstant(KeywordConstant),
    Variable(Ident),
    Index(Ident, Box<Expression>),
    SubroutineCall(SubroutineCall),
    Expression(Box<Expression>),
    UnaryOp(UnaryOp, Box<Term>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubroutineCall {
    SubroutineCall(Ident, Vec<Expression>),
    PropertyCall(Ident, Ident, Vec<Expression>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Lt,
    Gt,
    Eq,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeywordConstant {
    True,
    False,
    Null,
    This,
}
