use super::*;
use crate::{
    ast::{
        Class, ClassVarDec, ClassVarKind, ParameterList, Subroutine, SubroutineBody, SubroutineKind,
    },
    token::Location,
};
use either::Either;
use std::{collections::hash_map::Entry, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SymbolTableExtendError {
    #[error("invalid file path: {}", _0.display())]
    InvalidPath(PathBuf),
    #[error("file name `{}` and class name `{}` (defined at {}) do not match", _0, _1.data, _1.loc)]
    ClassNameMismatch(String, WithLoc<Ident>),
    #[error("class `{}` is already defined (defined at {}, first defined at {}:{})", _0.data, _0.loc, _1.display(), _2)]
    DuplicateClass(WithLoc<Ident>, PathBuf, Location),
    #[error("class method `{}` is already defined (defined at {}, first defined at {})", _0.data, _0.loc, _1)]
    DuplicateClassMethod(WithLoc<Ident>, Location),
    #[error("method `{}` is already defined (defined at {}, first defined at {})", _0.data, _0.loc, _1)]
    DuplicateMethod(WithLoc<Ident>, Location),
    #[error("invalid constructor return type `{}` found at {}", _0.data.as_str(), _0.loc)]
    InvalidConstructorReturnType(WithLoc<ReturnType>),
    #[error("property `{}` is already defined (defined at {}, first defined at {})", _0.data, _0.loc, _1)]
    DuplicateProperty(WithLoc<Ident>, Location),
    #[error("class symbol `{}` is already defined (defined at {}, first defined at {})", _0.data, _0.loc, _1)]
    DuplicateClassSymbol(WithLoc<Ident>, Location),
    #[error("subroutine symbol `{}` is already defined (defined at {}, first defined at {})", _0.data, _0.loc, _1)]
    DuplicateSubroutineSymbol(WithLoc<Ident>, Location),
}

fn convert_params(params: &WithLoc<ParameterList>) -> Vec<WithLoc<Variable>> {
    params
        .data
        .0
        .iter()
        .map(|p| {
            p.as_ref().map(|p| Variable {
                name: p.name.clone(),
                ty: p.ty.clone(),
            })
        })
        .collect()
}

impl GlobalSymbolTable {
    pub fn extend_with_class(
        &mut self,
        path: impl AsRef<Path>,
        class: &Class,
    ) -> Result<(), SymbolTableExtendError> {
        let path = path.as_ref();
        let Class {
            name: class_name,
            vars: _,
            subs: _,
        } = class;

        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| SymbolTableExtendError::InvalidPath(path.to_owned()))?;

        if file_name != class_name.data.as_str() {
            return Err(SymbolTableExtendError::ClassNameMismatch(
                file_name.to_owned(),
                class_name.clone(),
            ));
        }

        let ent = match self.table.entry(class_name.data.clone()) {
            Entry::Occupied(ent) => {
                if let Symbol::Class(class) = ent.get() {
                    return Err(SymbolTableExtendError::DuplicateClass(
                        class_name.clone(),
                        class.path.clone(),
                        class.class_name.loc,
                    ));
                } else {
                    unreachable!()
                }
            }
            Entry::Vacant(ent) => ent,
        };

        let table = ExternalClassSymbolTable::from_class(class, path)?;
        ent.insert(Symbol::Class(table));

        Ok(())
    }
}

impl ExternalClassSymbolTable {
    fn from_class(class: &Class, path: &Path) -> Result<Self, SymbolTableExtendError> {
        let Class {
            name: class_name,
            vars: _,
            subs,
        } = class;

        let mut methods = HashMap::<Ident, Method>::new();
        let mut class_methods = HashMap::<Ident, ClassMethod>::new();

        for sub in subs {
            let Subroutine {
                kind,
                return_type,
                name,
                params,
                ..
            } = &sub.data;
            let meth = {
                let name = name.clone();
                let return_type = return_type.clone();
                let params = convert_params(params);
                match kind.data {
                    SubroutineKind::Constructor => Either::Left(ClassMethod {
                        kind: ClassMethodKind::Constructor,
                        name,
                        return_type,
                        params,
                    }),
                    SubroutineKind::Function => Either::Left(ClassMethod {
                        kind: ClassMethodKind::Function,
                        name,
                        return_type,
                        params,
                    }),
                    SubroutineKind::Method => Either::Right(Method {
                        name,
                        return_type,
                        params,
                    }),
                }
            };

            match meth {
                Either::Left(meth) => match class_methods.entry(name.data.clone()) {
                    Entry::Occupied(ent) => {
                        return Err(SymbolTableExtendError::DuplicateClassMethod(
                            name.clone(),
                            ent.get().name.loc,
                        ))
                    }
                    Entry::Vacant(ent) => {
                        ent.insert(meth);
                    }
                },
                Either::Right(meth) => match methods.entry(name.data.clone()) {
                    Entry::Occupied(ent) => {
                        return Err(SymbolTableExtendError::DuplicateMethod(
                            name.clone(),
                            ent.get().name.loc,
                        ))
                    }
                    Entry::Vacant(ent) => {
                        ent.insert(meth);
                    }
                },
            }
        }

        Ok(ExternalClassSymbolTable {
            class_name: class_name.clone(),
            path: path.to_owned(),
            methods,
            class_methods,
        })
    }
}

impl<'a> InternalClassSymbolTable<'a> {
    pub(crate) fn from_class(
        class: &Class,
        outer: &'a GlobalSymbolTable,
    ) -> Result<Self, SymbolTableExtendError> {
        let Class {
            name: class_name,
            vars,
            subs,
        } = class;
        let mut table = HashMap::<Ident, Symbol>::new();
        let mut props = HashMap::<Ident, Property>::new();

        let mut static_vars = vec![];
        let mut fields = vec![];
        let mut class_methods = vec![];
        let mut methods = vec![];

        for var in vars {
            let ClassVarDec { kind, ty, names } = &var.data;
            match &kind.data {
                ClassVarKind::Static => {
                    for name in names {
                        static_vars.push(StaticVariable {
                            name: name.clone(),
                            ty: ty.clone(),
                        })
                    }
                }
                ClassVarKind::Field => {
                    for name in names {
                        fields.push(Field {
                            name: name.clone(),
                            ty: ty.clone(),
                            slot_index: fields.len(),
                        });
                    }
                }
            }
        }

        for sub in subs {
            let Subroutine {
                kind,
                return_type,
                name,
                params,
                body: _,
            } = &sub.data;
            let params = convert_params(params);
            match &kind.data {
                SubroutineKind::Constructor => {
                    if !matches!(&return_type.data, ReturnType::Type(WithLoc { data: Type::Class(class), ..}) if *class != name.data)
                    {
                        return Err(SymbolTableExtendError::InvalidConstructorReturnType(
                            return_type.clone(),
                        ));
                    }
                    class_methods.push(ClassMethod {
                        kind: ClassMethodKind::Constructor,
                        name: name.clone(),
                        return_type: return_type.clone(),
                        params,
                    })
                }
                SubroutineKind::Function => class_methods.push(ClassMethod {
                    kind: ClassMethodKind::Function,
                    name: name.clone(),
                    return_type: return_type.clone(),
                    params,
                }),
                SubroutineKind::Method => methods.push(Method {
                    name: name.clone(),
                    return_type: return_type.clone(),
                    params,
                }),
            }
        }

        let prop_syms = fields
            .iter()
            .cloned()
            .map(Property::Field)
            .chain(methods.iter().cloned().map(Property::Method));
        for sym in prop_syms {
            match props.entry(sym.name().data.clone()) {
                Entry::Occupied(ent) => {
                    return Err(SymbolTableExtendError::DuplicateProperty(
                        sym.name().clone(),
                        ent.get().name().loc,
                    ))
                }
                Entry::Vacant(ent) => {
                    ent.insert(sym);
                }
            }
        }

        let tables_syms = static_vars
            .into_iter()
            .map(Symbol::StaticVariable)
            .chain(fields.into_iter().map(Symbol::Field))
            .chain(class_methods.into_iter().map(Symbol::ClassMethod))
            .chain(methods.into_iter().map(Symbol::Method));
        for sym in tables_syms {
            match table.entry(sym.name().data.clone()) {
                Entry::Occupied(ent) => {
                    return Err(SymbolTableExtendError::DuplicateClassSymbol(
                        sym.name().clone(),
                        ent.get().name().loc,
                    ))
                }
                Entry::Vacant(ent) => {
                    ent.insert(sym);
                }
            }
        }

        Ok(InternalClassSymbolTable {
            class_name: class_name.clone(),
            outer,
            table,
            props,
        })
    }
}

impl<'a> SubroutineSymbolTable<'a> {
    pub(crate) fn from_subroutine(
        sub: &Subroutine,
        outer: &'a InternalClassSymbolTable,
    ) -> Result<Self, SymbolTableExtendError> {
        let Subroutine {
            kind,
            return_type,
            name: _,
            params,
            body,
        } = sub;
        let SubroutineBody { vars, stmts: _ } = &body.data;

        let params = params
            .data
            .0
            .iter()
            .enumerate()
            .map(|(slot_index, p)| Parameter {
                name: p.data.name.clone(),
                ty: p.data.ty.clone(),
                slot_index,
            })
            .map(Symbol::Parameter);
        let vars = vars
            .iter()
            .flat_map(|v| v.data.names.iter().map(|n| (v.data.ty.clone(), n.clone())))
            .enumerate()
            .map(|(slot_index, (ty, name))| LocalVariable {
                name,
                ty,
                slot_index,
            })
            .map(Symbol::LocalVariable);

        let mut table = HashMap::<Ident, Symbol>::new();

        for sym in params.chain(vars) {
            match table.entry(sym.name().data.clone()) {
                Entry::Occupied(ent) => {
                    return Err(SymbolTableExtendError::DuplicateSubroutineSymbol(
                        sym.name().clone(),
                        ent.get().name().loc,
                    ))
                }
                Entry::Vacant(ent) => {
                    ent.insert(sym);
                }
            }
        }

        Ok(SubroutineSymbolTable {
            outer,
            kind: kind.data,
            return_type: return_type.data.clone(),
            table,
        })
    }
}
