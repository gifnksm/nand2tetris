use vm::Segment;

use crate::{
    ast::{ReturnType, SubroutineKind, Type},
    token::{Ident, WithLoc},
    typed_ast::Variable,
};
use std::{borrow::Borrow, collections::HashMap, hash::Hash, path::PathBuf};

mod builtin;
pub(crate) mod extend;

#[derive(Debug, Clone)]
pub struct GlobalSymbolTable {
    table: HashMap<Ident, Symbol>,
}

#[derive(Debug, Clone)]
pub(crate) struct ExternalClassSymbolTable {
    class_name: WithLoc<Ident>,
    path: PathBuf,
    methods: HashMap<Ident, Method>,
    class_methods: HashMap<Ident, ClassMethod>,
}

#[derive(Debug, Clone)]
pub struct InternalClassSymbolTable<'a> {
    outer: &'a GlobalSymbolTable,
    class_name: WithLoc<Ident>,
    table: HashMap<Ident, Symbol>,
}

#[derive(Debug, Clone)]
pub(crate) struct SubroutineSymbolTable<'a> {
    outer: &'a InternalClassSymbolTable<'a>,
    kind: SubroutineKind,
    return_type: ReturnType,
    table: HashMap<Ident, Symbol>,
}

#[derive(Debug, Clone)]
pub(crate) enum Symbol {
    Class(ExternalClassSymbolTable),
    Method(Method),
    StaticVariable(StaticVariable),
    Field(Field),
    ClassMethod(ClassMethod),
    LocalVariable(LocalVariable),
    Parameter(Parameter),
}

impl Symbol {
    pub(crate) fn to_var(&self) -> Option<VarSymbol> {
        match self {
            Self::StaticVariable(v) => Some(VarSymbol::StaticVariable(v.clone())),
            Self::Field(f) => Some(VarSymbol::Field(f.clone())),
            Self::LocalVariable(v) => Some(VarSymbol::LocalVariable(v.clone())),
            Self::Parameter(p) => Some(VarSymbol::Parameter(p.clone())),
            Self::Class(_) | Symbol::Method(_) | Symbol::ClassMethod(_) => None,
        }
    }

    pub(crate) fn to_subroutine(&self) -> Option<SubroutineSymbol> {
        match self {
            Self::Method(m) => Some(SubroutineSymbol::Method(m.clone())),
            Self::ClassMethod(m) => Some(SubroutineSymbol::ClassMethod(m.clone())),
            Self::Class(_)
            | Self::StaticVariable(_)
            | Self::Field(_)
            | Self::LocalVariable(_)
            | Self::Parameter(_) => None,
        }
    }

    pub(crate) fn to_class(&self) -> Option<&ExternalClassSymbolTable> {
        match self {
            Self::Class(c) => Some(c),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum VarSymbol {
    StaticVariable(StaticVariable),
    Field(Field),
    LocalVariable(LocalVariable),
    Parameter(Parameter),
}

#[derive(Debug, Clone)]
pub enum SubroutineSymbol {
    Method(Method),
    ClassMethod(ClassMethod),
}

impl VarSymbol {
    pub(crate) fn ty(&self) -> &WithLoc<Type> {
        match self {
            Self::StaticVariable(v) => &v.ty,
            Self::Field(f) => &f.ty,
            Self::LocalVariable(v) => &v.ty,
            Self::Parameter(p) => &p.ty,
        }
    }

    pub(crate) fn segment_slot(&self) -> (Segment, usize) {
        match self {
            Self::StaticVariable(v) => (Segment::Static, v.slot_index),
            Self::Field(f) => (Segment::This, f.slot_index),
            Self::LocalVariable(f) => (Segment::Local, f.slot_index),
            Self::Parameter(f) => (Segment::Argument, f.slot_index),
        }
    }
}

impl SubroutineSymbol {
    pub(crate) fn params(&self) -> &[WithLoc<Variable>] {
        match self {
            Self::Method(m) => &m.params,
            Self::ClassMethod(m) => &m.params,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Method {
    pub(crate) name: WithLoc<Ident>,
    pub(crate) return_type: WithLoc<ReturnType>,
    pub(crate) params: Vec<WithLoc<Variable>>,
}

#[derive(Debug, Clone)]
pub struct ClassMethod {
    pub(crate) kind: ClassMethodKind,
    pub(crate) name: WithLoc<Ident>,
    pub(crate) return_type: WithLoc<ReturnType>,
    pub(crate) params: Vec<WithLoc<Variable>>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ClassMethodKind {
    Constructor,
    Function,
}

#[derive(Debug, Clone)]
pub struct StaticVariable {
    pub name: WithLoc<Ident>,
    pub ty: WithLoc<Type>,
    pub slot_index: usize,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub name: WithLoc<Ident>,
    pub ty: WithLoc<Type>,
    pub slot_index: usize,
}

#[derive(Debug, Clone)]
pub struct LocalVariable {
    pub name: WithLoc<Ident>,
    pub ty: WithLoc<Type>,
    pub slot_index: usize,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: WithLoc<Ident>,
    pub ty: WithLoc<Type>,
    pub slot_index: usize,
}

impl SubroutineSymbolTable<'_> {
    pub(crate) fn get<Q>(&self, name: &Q) -> Option<&Symbol>
    where
        Ident: Borrow<Q>,
        Q: Hash + Eq,
    {
        let is_function = matches!(self.kind, SubroutineKind::Function);
        let sym = self.table.get(name).or_else(|| self.outer.get(name))?;
        match (sym, is_function) {
            (
                Symbol::Class(_)
                | Symbol::StaticVariable(_)
                | Symbol::ClassMethod(_)
                | Symbol::LocalVariable(_)
                | Symbol::Parameter(_),
                _,
            )
            | (Symbol::Method(_) | Symbol::Field(_), false) => Some(sym),
            (Symbol::Method(_) | Symbol::Field(_), true) => None,
        }
    }

    pub(crate) fn is_valid_type(&self, ty: &Type) -> bool {
        self.outer.is_valid_type(ty)
    }

    pub(crate) fn return_type(&self) -> &ReturnType {
        &self.return_type
    }

    pub(crate) fn class_name(&self) -> &WithLoc<Ident> {
        self.outer.class_name()
    }
}

impl GlobalSymbolTable {
    pub(crate) fn get<Q>(&self, name: &Q) -> Option<&Symbol>
    where
        Ident: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.table.get(name)
    }

    fn is_valid_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Int | Type::Char | Type::Boolean => true,
            Type::Class(class) => matches!(self.table.get(class), Some(Symbol::Class(_))),
        }
    }
}

impl ExternalClassSymbolTable {
    pub(crate) fn method<Q>(&self, name: &Q) -> Option<&Method>
    where
        Ident: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.methods.get(name)
    }

    pub(crate) fn class_method<Q>(&self, name: &Q) -> Option<&ClassMethod>
    where
        Ident: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.class_methods.get(name)
    }
}

impl InternalClassSymbolTable<'_> {
    pub(crate) fn get<Q>(&self, name: &Q) -> Option<&Symbol>
    where
        Ident: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.table.get(name).or_else(|| self.outer.get(name))
    }

    pub(crate) fn is_valid_type(&self, ty: &Type) -> bool {
        self.outer.is_valid_type(ty)
    }

    fn class_name(&self) -> &WithLoc<Ident> {
        &self.class_name
    }
}

impl Symbol {
    fn name(&self) -> &WithLoc<Ident> {
        match self {
            Symbol::Class(c) => &c.class_name,
            Symbol::Method(m) => &m.name,
            Symbol::StaticVariable(v) => &v.name,
            Symbol::Field(f) => &f.name,
            Symbol::ClassMethod(m) => &m.name,
            Symbol::LocalVariable(v) => &v.name,
            Symbol::Parameter(p) => &p.name,
        }
    }
}
