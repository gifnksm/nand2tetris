use crate::token::{Ident, WithLoc};
pub use parser::*;
pub use resolver::*;

mod parser;
mod resolver;

#[derive(Debug, Clone)]
pub struct Class {
    pub name: WithLoc<Ident>,
    pub vars: Vec<WithLoc<ClassVarDec>>,
    pub subs: Vec<WithLoc<Subroutine>>,
}

#[derive(Debug, Clone)]
pub struct ClassVarDec {
    pub kind: WithLoc<ClassVarKind>,
    pub ty: WithLoc<Type>,
    pub names: Vec<WithLoc<Ident>>,
}

#[derive(Debug, Clone, Copy)]
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

impl Type {
    pub fn string() -> Self {
        Self::Class(Ident::new("String"))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Type::Class(i) if i.as_str() == "Array")
    }

    pub fn as_str(&self) -> &str {
        match self {
            Type::Int => "int",
            Type::Char => "char",
            Type::Boolean => "boolean",
            Type::Class(name) => name.as_str(),
        }
    }

    pub fn to_class(&self) -> Option<&Ident> {
        if let Type::Class(name) = self {
            Some(name)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReturnType {
    Void,
    Type(WithLoc<Type>),
}

impl ReturnType {
    pub fn as_str(&self) -> &str {
        match self {
            ReturnType::Void => "void",
            ReturnType::Type(ty) => ty.data.as_str(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Subroutine {
    pub kind: WithLoc<SubroutineKind>,
    pub return_type: WithLoc<ReturnType>,
    pub name: WithLoc<Ident>,
    pub params: WithLoc<ParameterList>,
    pub body: WithLoc<SubroutineBody>,
}

#[derive(Debug, Clone, Copy)]
pub enum SubroutineKind {
    Constructor,
    Function,
    Method,
}

impl SubroutineKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SubroutineKind::Constructor => "constructor",
            SubroutineKind::Function => "function",
            SubroutineKind::Method => "method",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParameterList(pub Vec<WithLoc<Parameter>>);

#[derive(Debug, Clone)]
pub struct Parameter {
    pub ty: WithLoc<Type>,
    pub name: WithLoc<Ident>,
}

#[derive(Debug, Clone)]
pub struct SubroutineBody {
    pub vars: Vec<WithLoc<LocalVarDec>>,
    pub stmts: WithLoc<StatementList>,
}

#[derive(Debug, Clone)]
pub struct LocalVarDec {
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
    pub target: WithLoc<Ident>,
    pub target_index: Option<WithLoc<Expression>>,
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

impl SubroutineCall {
    pub(crate) fn subroutine_name(&self) -> &WithLoc<Ident> {
        match self {
            Self::SubroutineCall(name, _) => name,
            Self::PropertyCall(_, name, _) => name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionList(pub Vec<WithLoc<Expression>>);

#[derive(Debug, Clone, Copy)]
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

impl BinaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Add => "Add",
            Self::Sub => "Sub",
            Self::Mul => "Mul",
            Self::Div => "Div",
            Self::And => "And",
            Self::Or => "Or",
            Self::Lt => "Lt",
            Self::Gt => "Gt",
            Self::Eq => "Eq",
        }
    }

    pub(crate) fn get_ty(&self, lhs_ty: &Type, rhs_ty: &Type) -> (Type, Type) {
        match self {
            Self::Add | Self::Sub | Self::Mul | Self::Div => (Type::Int, Type::Int),
            Self::Lt | Self::Gt | Self::Eq => (Type::Int, Type::Boolean),
            Self::And | Self::Or => {
                if *lhs_ty == Type::Boolean && *rhs_ty == Type::Boolean {
                    (Type::Boolean, Type::Boolean)
                } else {
                    (Type::Int, Type::Int)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Neg,
    Not,
}

impl UnaryOp {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Neg => "Neg",
            Self::Not => "Not",
        }
    }

    pub(crate) fn get_ty(&self, operand_ty: &Type) -> (Type, Type) {
        match self {
            Self::Neg => (Type::Int, Type::Int),
            Self::Not => {
                if *operand_ty == Type::Boolean {
                    (Type::Boolean, Type::Boolean)
                } else {
                    (Type::Int, Type::Int)
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum KeywordConstant {
    True,
    False,
    Null,
    This,
}
