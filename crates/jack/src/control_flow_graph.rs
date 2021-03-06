use crate::{
    ast::{ReturnType, SubroutineKind},
    token::{Ident, WithLoc},
    typed_ast::{TypedDoStatement, TypedExpression, TypedLetStatement, Variable},
};
use std::{collections::HashMap, fmt};

mod codegen;
mod optimizer;
mod update;

#[derive(Debug)]
pub struct CfgClass {
    pub name: WithLoc<Ident>,
    pub static_vars: Vec<WithLoc<Variable>>,
    pub fields: Vec<WithLoc<Variable>>,
    pub subs: Vec<WithLoc<CfgSubroutine>>,
}

#[derive(Debug, Clone)]
pub struct CfgSubroutine {
    pub name: WithLoc<Ident>,
    pub kind: WithLoc<SubroutineKind>,
    pub return_type: WithLoc<ReturnType>,
    pub params: Vec<WithLoc<Variable>>,
    pub vars: Vec<WithLoc<Variable>>,
    pub entry_id: BbId,
    pub block_index_map: HashMap<BbId, usize>,
    pub blocks: Vec<WithLoc<BasicBlock>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BbId {
    pub label: &'static str,
    pub index: u32,
}

impl fmt::Display for BbId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.label, self.index)
    }
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BbId,
    pub src_ids: Vec<BbId>,
    pub stmts: Vec<CfgStatement>,
    pub exit: Exit,
}

#[derive(Debug, Clone)]
pub enum CfgStatement {
    Let(WithLoc<TypedLetStatement>),
    Do(WithLoc<TypedDoStatement>),
}

#[derive(Debug, Clone)]
pub enum Exit {
    Return(Option<WithLoc<TypedExpression>>),
    Goto(BbId),
    If(WithLoc<TypedExpression>, BbId, BbId),
    Unreachable,
}
