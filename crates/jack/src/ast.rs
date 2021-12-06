use crate::{Ident, WithLoc};
pub use parser::*;

mod parser;

#[derive(Debug, Clone)]
pub struct Class {
    pub name: WithLoc<Ident>,
    pub vars: Vec<WithLoc<ClassVar>>,
    pub subs: Vec<WithLoc<Subroutine>>,
}

#[derive(Debug, Clone)]
pub struct ClassVar {
    pub kind: WithLoc<ClassVarKind>,
    pub ty: WithLoc<Type>,
    pub var_names: Vec<WithLoc<Ident>>,
}

#[derive(Debug, Clone)]
pub enum ClassVarKind {
    Static,
    Field,
}

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Char,
    Boolean,
    Class(Ident),
}

#[derive(Debug, Clone)]
pub struct Subroutine {
    pub kind: WithLoc<SubroutineKind>,
    pub return_type: Option<WithLoc<Type>>,
    pub name: WithLoc<Ident>,
    pub params: WithLoc<ParameterList>,
    pub body: WithLoc<SubroutineBody>,
}

#[derive(Debug, Clone)]
pub enum SubroutineKind {
    Constructor,
    Function,
    Method,
}

#[derive(Debug, Clone)]
pub struct ParameterList(pub Vec<WithLoc<Parameter>>);

#[derive(Debug, Clone)]
pub struct Parameter {
    pub ty: WithLoc<Type>,
    pub var_name: WithLoc<Ident>,
}

#[derive(Debug, Clone)]
pub struct SubroutineBody {
    pub vars: Vec<WithLoc<Var>>,
    pub stmts: WithLoc<StatementList>,
}

#[derive(Debug, Clone)]
pub struct Var {
    pub ty: WithLoc<Type>,
    pub names: Vec<WithLoc<Ident>>,
}

#[derive(Debug, Clone)]
pub struct StatementList(pub Vec<WithLoc<Statement>>);

#[derive(Debug, Clone)]
pub enum Statement {
    Let(WithLoc<LetStatement>),
    If(WithLoc<IfStatement>),
    While(WithLoc<WhileStatement>),
    Do(WithLoc<DoStatement>),
    Return(WithLoc<ReturnStatement>),
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub var_name: WithLoc<Ident>,
    pub index: Option<WithLoc<Expression>>,
    pub expr: WithLoc<Expression>,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub cond: WithLoc<Expression>,
    pub then_stmts: WithLoc<StatementList>,
    pub else_stmts: Option<WithLoc<StatementList>>,
}

#[derive(Debug, Clone)]
pub struct WhileStatement {
    pub cond: WithLoc<Expression>,
    pub stmts: WithLoc<StatementList>,
}

#[derive(Debug, Clone)]
pub struct DoStatement {
    pub sub_call: WithLoc<SubroutineCall>,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub expr: Option<WithLoc<Expression>>,
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub term: WithLoc<Term>,
    pub binary_ops: Vec<(WithLoc<BinaryOp>, WithLoc<Term>)>,
}

#[derive(Debug, Clone)]
pub enum Term {
    IntConstant(WithLoc<u16>),
    StringConstant(WithLoc<String>),
    KeywordConstant(WithLoc<KeywordConstant>),
    Variable(WithLoc<Ident>),
    Index(WithLoc<Ident>, Box<WithLoc<Expression>>),
    SubroutineCall(WithLoc<SubroutineCall>),
    Expression(Box<WithLoc<Expression>>),
    UnaryOp(WithLoc<UnaryOp>, Box<WithLoc<Term>>),
}

#[derive(Debug, Clone)]
pub enum SubroutineCall {
    SubroutineCall(WithLoc<Ident>, WithLoc<ExpressionList>),
    PropertyCall(WithLoc<Ident>, WithLoc<Ident>, WithLoc<ExpressionList>),
}

#[derive(Debug, Clone)]
pub struct ExpressionList(pub Vec<WithLoc<Expression>>);

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum KeywordConstant {
    True,
    False,
    Null,
    This,
}
