pub use self::control_flow_graph::*;

mod control_flow_graph;

use crate::{
    ast::{BinaryOp, ReturnType, SubroutineKind, Type, UnaryOp},
    symbol_table::VarSymbol,
    token::{Ident, WithLoc},
};

#[derive(Debug, Clone)]
pub struct TypedClass {
    pub name: WithLoc<Ident>,
    pub static_vars: Vec<WithLoc<Variable>>,
    pub fields: Vec<WithLoc<Variable>>,
    pub subs: Vec<WithLoc<TypedSubroutine>>,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: WithLoc<Ident>,
    pub ty: WithLoc<Type>,
}

#[derive(Debug, Clone)]
pub struct TypedSubroutine {
    pub name: WithLoc<Ident>,
    pub kind: WithLoc<SubroutineKind>,
    pub return_type: WithLoc<ReturnType>,
    pub params: Vec<WithLoc<Variable>>,
    pub vars: Vec<WithLoc<Variable>>,
    pub stmts: Vec<WithLoc<TypedStatement>>,
}

#[derive(Debug, Clone)]
pub enum TypedStatement {
    Let(WithLoc<TypedLetStatement>),
    If(WithLoc<TypedIfStatement>),
    While(WithLoc<TypedWhileStatement>),
    Do(WithLoc<TypedDoStatement>),
    Return(WithLoc<TypedReturnStatement>),
}

#[derive(Debug, Clone)]
pub struct TypedLetStatement {
    pub target: VarSymbol,
    pub target_index: Option<WithLoc<TypedExpression>>,
    pub expr: WithLoc<TypedExpression>,
}

#[derive(Debug, Clone)]
pub struct TypedIfStatement {
    pub cond: WithLoc<TypedExpression>,
    pub then_stmts: Vec<WithLoc<TypedStatement>>,
    pub else_stmts: Option<Vec<WithLoc<TypedStatement>>>,
}

#[derive(Debug, Clone)]
pub struct TypedWhileStatement {
    pub cond: WithLoc<TypedExpression>,
    pub stmts: Vec<WithLoc<TypedStatement>>,
}

#[derive(Debug, Clone)]
pub struct TypedReturnStatement {
    pub expr: Option<WithLoc<TypedExpression>>,
}

#[derive(Debug, Clone)]
pub struct TypedDoStatement {
    pub sub_call: WithLoc<TypedSubroutineCall>,
}

#[derive(Debug, Clone)]
pub struct TypedExpression {
    pub ty: Type,
    pub term: Box<TypedTerm>,
}

#[derive(Debug, Clone)]
pub enum TypedTerm {
    Int(WithLoc<u16>),
    String(WithLoc<String>),
    Bool(WithLoc<bool>),
    Null,
    This,
    Var(VarSymbol),
    Index(VarSymbol, WithLoc<TypedExpression>),
    SubroutineCall(WithLoc<TypedSubroutineCall>),
    UnaryOp(WithLoc<UnaryOp>, WithLoc<TypedExpression>),
    BinaryOp(
        WithLoc<BinaryOp>,
        WithLoc<TypedExpression>,
        WithLoc<TypedExpression>,
    ),
}

#[derive(Debug, Clone)]
pub enum TypedSubroutineCall {
    Method(
        Option<VarSymbol>,
        WithLoc<Ident>,
        Vec<WithLoc<TypedExpression>>,
    ),
    Function(
        Option<WithLoc<Ident>>,
        WithLoc<Ident>,
        Vec<WithLoc<TypedExpression>>,
    ),
    Constructor(
        Option<WithLoc<Ident>>,
        WithLoc<Ident>,
        Vec<WithLoc<TypedExpression>>,
    ),
}
