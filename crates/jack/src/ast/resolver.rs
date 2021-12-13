use super::*;
use crate::{
    symbol_table::{
        extend::SymbolTableExtendError, ClassMethodKind, ExternalClassSymbolTable,
        GlobalSymbolTable, InternalClassSymbolTable, SubroutineSymbol, SubroutineSymbolTable,
        Symbol, VarSymbol,
    },
    token::Location,
    typed_ast::*,
};
use either::Either;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResolveError {
    #[error(transparent)]
    SymbolTable(#[from] SymbolTableExtendError),
    #[error("invalid type `{}` found at {}", _0.data.as_str(), _0.loc)]
    InvalidType(WithLoc<Type>),
    #[error("return values from void function at `{}`", _0)]
    ReturnVoidValue(Location),
    #[error("no return value from non-void function at `{}`", _0)]
    NoReturnValue(Location),
    #[error("undefined symbol `{}` found at {}", _0.data.as_str(), _0.loc)]
    UndefinedSymbol(WithLoc<Ident>),
    #[error("symbol `{}` is not a variable found at {}", _0.data.as_str(), _0.loc)]
    NotVariable(WithLoc<Ident>),
    #[error("symbol `{}` is not a subroutine found at {}", _0.data.as_str(), _0.loc)]
    NotSubroutine(WithLoc<Ident>),
    #[error("symbol `{}` is not a receiver found at {}", _0.data.as_str(), _0.loc)]
    NotReceiver(WithLoc<Ident>),
    #[error(
        "number of argument for subroutine `{}` at {} is not correct: expected {}, actual: {}",
        subroutine.data.as_str(),
        subroutine.loc,
        expected,
        actual
    )]
    ArgumentCountMismatch {
        subroutine: WithLoc<Ident>,
        expected: usize,
        actual: usize,
    },
    #[error("void subroutine `{}` called in expression at {}", _0.data.as_str(), _0.loc)]
    VoidSubroutineCall(WithLoc<Ident>),
    #[error("class `{}` does not have a method `{}` called at {}", _0.as_str(), _1.data.as_str(), _1.loc)]
    MethodNotFound(Ident, WithLoc<Ident>),
    #[error("method `{}` call on primitive type `{}` at {}", _0.data.as_str(), _1.as_str(), _0.loc)]
    MethodCallOnPrimitive(WithLoc<Ident>, Type),
    #[error("class `{}` does not have a class method `{}` called at {}", _0.as_str(), _1.data.as_str(), _1.loc)]
    ClassMethodNotFound(Ident, WithLoc<Ident>),
    #[error("non-array variable `{}` cannot be indexed at {}", _0.data.as_str(), _0.loc)]
    IndexOfNonArray(WithLoc<Ident>),
    #[error("cannot cast `{}` to `{}` at {}", _1.as_str(), _2.as_str(), _0)]
    ImplicitCast(Location, Type, Type),
}

impl WithLoc<Class> {
    pub fn resolve(
        &self,
        sym_tab: &GlobalSymbolTable,
    ) -> Result<WithLoc<TypedClass>, ResolveError> {
        self.as_ref()
            .map(|class| class.resolve(sym_tab))
            .transpose_ok()
    }
}

impl Class {
    fn resolve(&self, sym_tab: &GlobalSymbolTable) -> Result<TypedClass, ResolveError> {
        let Self { name, vars, subs } = self;
        let sym_tab = InternalClassSymbolTable::from_class(self, sym_tab)?;
        let mut static_vars = vec![];
        let mut fields = vec![];
        for var in vars {
            let target = match var.data.kind.data {
                ClassVarKind::Field => &mut fields,
                ClassVarKind::Static => &mut static_vars,
            };
            let vars = var.resolve(&sym_tab)?;
            target.extend(vars);
        }
        Ok(TypedClass {
            name: name.clone(),
            static_vars,
            fields,
            subs: subs
                .iter()
                .map(|sub| sub.resolve(&sym_tab))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl WithLoc<ClassVarDec> {
    fn resolve(
        &self,
        sym_tab: &InternalClassSymbolTable,
    ) -> Result<impl Iterator<Item = WithLoc<Variable>> + '_, ResolveError> {
        let vars = self.data.resolve(sym_tab)?;
        Ok(vars.map(|data| WithLoc {
            data,
            loc: self.loc,
        }))
    }
}

impl ClassVarDec {
    fn resolve(
        &self,
        sym_tab: &InternalClassSymbolTable,
    ) -> Result<impl Iterator<Item = Variable> + '_, ResolveError> {
        let Self { kind: _, names, ty } = self;
        if !sym_tab.is_valid_type(&ty.data) {
            return Err(ResolveError::InvalidType(ty.clone()));
        }
        Ok(names.iter().map(move |name| Variable {
            name: name.clone(),
            ty: ty.clone(),
        }))
    }
}

impl WithLoc<Subroutine> {
    fn resolve(
        &self,
        sym_tab: &InternalClassSymbolTable,
    ) -> Result<WithLoc<TypedSubroutine>, ResolveError> {
        self.as_ref().map(|sub| sub.resolve(sym_tab)).transpose_ok()
    }
}

impl Subroutine {
    fn resolve(&self, sym_tab: &InternalClassSymbolTable) -> Result<TypedSubroutine, ResolveError> {
        let Self {
            kind,
            return_type,
            name,
            params,
            body,
        } = self;
        let SubroutineBody {
            vars: var_decs,
            stmts,
        } = &body.data;
        let sym_tab = SubroutineSymbolTable::from_subroutine(self, sym_tab)?;
        let params = params
            .data
            .0
            .iter()
            .map(|param| param.resolve(&sym_tab))
            .collect::<Result<_, _>>()?;
        let mut vars = vec![];
        for var_dec in var_decs {
            vars.extend(var_dec.resolve(&sym_tab)?);
        }
        let stmts = stmts.data.resolve_local(&sym_tab)?;
        Ok(TypedSubroutine {
            name: name.clone(),
            kind: *kind,
            return_type: return_type.clone(),
            params,
            vars,
            stmts,
        })
    }
}

impl WithLoc<Parameter> {
    fn resolve(&self, sym_tab: &SubroutineSymbolTable) -> Result<WithLoc<Variable>, ResolveError> {
        self.as_ref()
            .map(|param| param.resolve(sym_tab))
            .transpose_ok()
    }
}

impl Parameter {
    fn resolve(&self, sym_tab: &SubroutineSymbolTable) -> Result<Variable, ResolveError> {
        let Self { name, ty } = self;
        if !sym_tab.is_valid_type(&ty.data) {
            return Err(ResolveError::InvalidType(ty.clone()));
        }
        Ok(Variable {
            name: name.clone(),
            ty: ty.clone(),
        })
    }
}

impl WithLoc<LocalVarDec> {
    fn resolve(
        &self,
        sym_tab: &SubroutineSymbolTable,
    ) -> Result<impl Iterator<Item = WithLoc<Variable>> + '_, ResolveError> {
        let vars = self.data.resolve(sym_tab)?;
        Ok(vars.map(|data| WithLoc {
            data,
            loc: self.loc,
        }))
    }
}

impl LocalVarDec {
    fn resolve(
        &self,
        sym_tab: &SubroutineSymbolTable,
    ) -> Result<impl Iterator<Item = Variable> + '_, ResolveError> {
        let Self { names, ty } = self;
        if !sym_tab.is_valid_type(&ty.data) {
            return Err(ResolveError::InvalidType(ty.clone()));
        }
        Ok(names.iter().map(|name| Variable {
            name: name.clone(),
            ty: ty.clone(),
        }))
    }
}

impl SubroutineSymbolTable<'_> {
    fn get_sym(&self, name: &WithLoc<Ident>) -> Result<&Symbol, ResolveError> {
        self.get(&name.data)
            .ok_or_else(|| ResolveError::UndefinedSymbol(name.clone()))
    }

    fn get_var(&self, name: &WithLoc<Ident>) -> Result<VarSymbol, ResolveError> {
        let var = self
            .get_sym(name)?
            .to_var()
            .ok_or_else(|| ResolveError::NotVariable(name.clone()))?;
        Ok(var)
    }

    fn get_sub(&self, name: &WithLoc<Ident>) -> Result<SubroutineSymbol, ResolveError> {
        let sub = self
            .get_sym(name)?
            .to_subroutine()
            .ok_or_else(|| ResolveError::NotSubroutine(name.clone()))?;
        Ok(sub)
    }

    fn get_receiver(
        &self,
        name: &WithLoc<Ident>,
    ) -> Result<Either<VarSymbol, &ExternalClassSymbolTable>, ResolveError> {
        let sym = self.get_sym(name)?;
        if let Some(var) = sym.to_var() {
            return Ok(Either::Left(var));
        }
        if let Symbol::Class(class) = sym {
            return Ok(Either::Right(class));
        }
        Err(ResolveError::NotReceiver(name.clone()))
    }
}

trait ResolveLocal {
    type Output;
    fn resolve_local(&self, sym_tab: &SubroutineSymbolTable) -> Result<Self::Output, ResolveError>;
}

impl<T> ResolveLocal for WithLoc<T>
where
    T: ResolveLocal,
{
    type Output = WithLoc<T::Output>;

    fn resolve_local(&self, sym_tab: &SubroutineSymbolTable) -> Result<Self::Output, ResolveError> {
        Ok(Self::Output {
            data: self.data.resolve_local(sym_tab)?,
            loc: self.loc,
        })
    }
}

impl ResolveLocal for StatementList {
    type Output = Vec<WithLoc<TypedStatement>>;

    fn resolve_local(&self, sym_tab: &SubroutineSymbolTable) -> Result<Self::Output, ResolveError> {
        self.0
            .iter()
            .map(|stmt| stmt.resolve_local(sym_tab))
            .collect()
    }
}

impl ResolveLocal for Statement {
    type Output = TypedStatement;

    fn resolve_local(&self, sym_tab: &SubroutineSymbolTable) -> Result<Self::Output, ResolveError> {
        let stmt = match &self {
            Statement::Let(stmt) => TypedStatement::Let(stmt.resolve_local(sym_tab)?),
            Statement::If(stmt) => TypedStatement::If(stmt.resolve_local(sym_tab)?),
            Statement::While(stmt) => TypedStatement::While(stmt.resolve_local(sym_tab)?),
            Statement::Do(stmt) => TypedStatement::Do(stmt.resolve_local(sym_tab)?),
            Statement::Return(stmt) => TypedStatement::Return(stmt.resolve_local(sym_tab)?),
        };
        Ok(stmt)
    }
}

fn resolve_var_direct_access(
    var: &WithLoc<Ident>,
    sym_tab: &SubroutineSymbolTable,
) -> Result<(Type, VarSymbol), ResolveError> {
    let var_sym = sym_tab.get_var(var)?;
    Ok((var_sym.ty().data.clone(), var_sym))
}

fn resolve_var_index_access(
    var: &WithLoc<Ident>,
    index: &WithLoc<Expression>,
    sym_tab: &SubroutineSymbolTable,
) -> Result<(Type, VarSymbol, WithLoc<TypedExpression>), ResolveError> {
    let var_sym = sym_tab.get_var(var)?;
    let index = index.resolve_local(sym_tab)?.implicit_cast(&Type::Int)?;
    if !var_sym.ty().data.is_array() {
        return Err(ResolveError::IndexOfNonArray(var.clone()));
    }
    Ok((Type::Int, var_sym, index))
}

fn resolve_var_any_access(
    var: &WithLoc<Ident>,
    index: &Option<WithLoc<Expression>>,
    sym_tab: &SubroutineSymbolTable,
) -> Result<(Type, VarSymbol, Option<WithLoc<TypedExpression>>), ResolveError> {
    match index {
        Some(index) => {
            let (ty, var, index) = resolve_var_index_access(var, index, sym_tab)?;
            Ok((ty, var, Some(index)))
        }
        None => {
            let (ty, var) = resolve_var_direct_access(var, sym_tab)?;
            Ok((ty, var, None))
        }
    }
}

impl ResolveLocal for LetStatement {
    type Output = TypedLetStatement;

    fn resolve_local(&self, sym_tab: &SubroutineSymbolTable) -> Result<Self::Output, ResolveError> {
        let (target_ty, target, target_index) =
            resolve_var_any_access(&self.target, &self.target_index, sym_tab)?;
        let expr = self
            .expr
            .resolve_local(sym_tab)?
            .implicit_cast(&target_ty)?;
        Ok(Self::Output {
            target,
            target_index,
            expr,
        })
    }
}

impl ResolveLocal for IfStatement {
    type Output = TypedIfStatement;

    fn resolve_local(&self, sym_tab: &SubroutineSymbolTable) -> Result<Self::Output, ResolveError> {
        let cond = self
            .cond
            .resolve_local(sym_tab)?
            .implicit_cast(&Type::Boolean)?;
        let then_stmts = self.then_stmts.data.resolve_local(sym_tab)?;
        let else_stmts = self
            .else_stmts
            .as_ref()
            .map(|stmt| stmt.data.resolve_local(sym_tab))
            .transpose()?;
        Ok(Self::Output {
            cond,
            then_stmts,
            else_stmts,
        })
    }
}

impl ResolveLocal for WhileStatement {
    type Output = TypedWhileStatement;

    fn resolve_local(&self, sym_tab: &SubroutineSymbolTable) -> Result<Self::Output, ResolveError> {
        let cond = self
            .cond
            .resolve_local(sym_tab)?
            .implicit_cast(&Type::Boolean)?;
        let stmts = self.stmts.data.resolve_local(sym_tab)?;
        Ok(Self::Output { cond, stmts })
    }
}

impl ResolveLocal for DoStatement {
    type Output = TypedDoStatement;

    fn resolve_local(&self, sym_tab: &SubroutineSymbolTable) -> Result<Self::Output, ResolveError> {
        let WithLoc {
            data: (_, sub_call),
            loc,
        } = self.sub_call.resolve_local(sym_tab)?;
        Ok(Self::Output {
            sub_call: WithLoc {
                data: sub_call,
                loc,
            },
        })
    }
}

impl ResolveLocal for WithLoc<ReturnStatement> {
    type Output = WithLoc<TypedReturnStatement>;

    fn resolve_local(&self, sym_tab: &SubroutineSymbolTable) -> Result<Self::Output, ResolveError> {
        let stmt = match (sym_tab.return_type(), &self.data.expr) {
            (ReturnType::Void, Some(expr)) => return Err(ResolveError::ReturnVoidValue(expr.loc)),
            (ReturnType::Type(_), None) => return Err(ResolveError::NoReturnValue(self.loc)),
            (ReturnType::Void, None) => TypedReturnStatement { expr: None },
            (ReturnType::Type(result_ty), Some(expr)) => {
                let expr = expr
                    .resolve_local(sym_tab)?
                    .implicit_cast(&result_ty.data)?;
                TypedReturnStatement { expr: Some(expr) }
            }
        };
        Ok(WithLoc {
            data: stmt,
            loc: self.loc,
        })
    }
}

impl WithLoc<TypedExpression> {
    fn implicit_cast(self, to_ty: &Type) -> Result<Self, ResolveError> {
        match (&self.data.ty, to_ty) {
            // char <-> int
            (Type::Int, Type::Char) | (Type::Char, Type::Int) => Ok(self),
            // class <-> int
            (Type::Class(_), Type::Int) | (Type::Int, Type::Class(_)) => Ok(self),
            // array <-> class
            (a @ Type::Class(_), Type::Class(_)) | (Type::Class(_), a @ Type::Class(_))
                if a.is_array() =>
            {
                Ok(self)
            }
            (a, b) if a == b => Ok(self),
            _ => Err(ResolveError::ImplicitCast(
                self.loc,
                self.data.ty,
                to_ty.clone(),
            )),
        }
    }
}

impl ResolveLocal for Expression {
    type Output = TypedExpression;

    fn resolve_local(&self, sym_tab: &SubroutineSymbolTable) -> Result<Self::Output, ResolveError> {
        let Expression { term, binary_ops } = self;
        let mut lhs = term.resolve_local(sym_tab)?;
        for (op, rhs) in binary_ops {
            let rhs = rhs.resolve_local(sym_tab)?;
            let (operand_ty, result_ty) = op.data.get_ty(&lhs.data.ty, &rhs.data.ty);
            lhs = lhs.implicit_cast(&operand_ty)?;
            let rhs = rhs.implicit_cast(&operand_ty)?;
            lhs = WithLoc {
                data: TypedExpression {
                    ty: result_ty,
                    term: Box::new(TypedTerm::BinaryOp(*op, lhs, rhs)),
                },
                loc: term.loc,
            };
        }
        Ok(lhs.data)
    }
}

impl ResolveLocal for Term {
    type Output = TypedExpression;

    fn resolve_local(&self, sym_tab: &SubroutineSymbolTable) -> Result<Self::Output, ResolveError> {
        let (ty, term) = match &self {
            Term::IntConstant(n) => (Type::Int, TypedTerm::Int(*n)),
            Term::StringConstant(s) => (Type::string(), TypedTerm::String(s.clone())),
            Term::KeywordConstant(kw) => match kw.data {
                KeywordConstant::True => (Type::Boolean, TypedTerm::Bool(kw.map(|_| true))),
                KeywordConstant::False => (Type::Boolean, TypedTerm::Bool(kw.map(|_| false))),
                KeywordConstant::Null => (Type::Int, TypedTerm::Null),
                KeywordConstant::This => (
                    Type::Class(sym_tab.class_name().data.clone()),
                    TypedTerm::This,
                ),
            },
            Term::Variable(name) => {
                let (ty, var) = resolve_var_direct_access(name, sym_tab)?;
                (ty, TypedTerm::Var(var))
            }
            Term::Index(name, index) => {
                let (ty, var, index) = resolve_var_index_access(name, index, sym_tab)?;
                (ty, TypedTerm::Index(var, index))
            }
            Term::SubroutineCall(sub) => {
                let WithLoc {
                    data: (ty, term),
                    loc,
                } = sub.resolve_local(sym_tab)?;
                match ty {
                    ReturnType::Type(ty) => (
                        ty.data,
                        TypedTerm::SubroutineCall(WithLoc { data: term, loc }),
                    ),
                    ReturnType::Void => {
                        return Err(ResolveError::VoidSubroutineCall(
                            sub.data.subroutine_name().clone(),
                        ))
                    }
                }
            }
            Term::Expression(expr) => {
                let expr = expr.resolve_local(sym_tab)?.data;
                (expr.ty, *expr.term)
            }
            Term::UnaryOp(op, operand) => {
                let operand = operand.resolve_local(sym_tab)?;
                let (operand_ty, result_ty) = op.data.get_ty(&operand.data.ty);
                let operand = operand.implicit_cast(&operand_ty)?;
                (result_ty, TypedTerm::UnaryOp(*op, operand))
            }
        };
        Ok(Self::Output {
            ty,
            term: Box::new(term),
        })
    }
}

impl ResolveLocal for SubroutineCall {
    type Output = (ReturnType, TypedSubroutineCall);

    fn resolve_local(&self, sym_tab: &SubroutineSymbolTable) -> Result<Self::Output, ResolveError> {
        let (ty, term) = match &self {
            SubroutineCall::SubroutineCall(sub_name, args) => {
                let sub = sym_tab.get_sub(sub_name)?;
                let args = resolve_args(sub_name, sub.params(), args, sym_tab)?;
                match sub {
                    SubroutineSymbol::Method(m) => (
                        m.return_type.data,
                        TypedSubroutineCall::Method(None, m.name, args),
                    ),
                    SubroutineSymbol::ClassMethod(m) => match m.kind {
                        ClassMethodKind::Function => (
                            m.return_type.data,
                            TypedSubroutineCall::Function(None, m.name, args),
                        ),
                        ClassMethodKind::Constructor => (
                            m.return_type.data,
                            TypedSubroutineCall::Constructor(None, m.name, args),
                        ),
                    },
                }
            }
            SubroutineCall::PropertyCall(receiver, sub_name, args) => {
                match sym_tab.get_receiver(receiver)? {
                    Either::Left(var) => {
                        let receiver_ty = &var.ty().data;
                        let class_name = receiver_ty.to_class().ok_or_else(|| {
                            ResolveError::MethodCallOnPrimitive(
                                receiver.clone(),
                                receiver_ty.clone(),
                            )
                        })?;
                        let class = sym_tab.get(class_name).unwrap().to_class().unwrap(); // class definition should be exists
                        let method = class.method(&sub_name.data).ok_or_else(|| {
                            ResolveError::MethodNotFound(receiver.data.clone(), sub_name.clone())
                        })?;
                        let args = resolve_args(sub_name, &method.params, args, sym_tab)?;
                        (
                            method.return_type.data.clone(),
                            TypedSubroutineCall::Method(Some(var), sub_name.clone(), args),
                        )
                    }
                    Either::Right(class) => {
                        let receiver = receiver.clone();
                        let method = class.class_method(&sub_name.data).ok_or_else(|| {
                            ResolveError::ClassMethodNotFound(
                                receiver.data.clone(),
                                sub_name.clone(),
                            )
                        })?;
                        let args = resolve_args(sub_name, &method.params, args, sym_tab)?;
                        match method.kind {
                            ClassMethodKind::Function => (
                                method.return_type.data.clone(),
                                TypedSubroutineCall::Function(
                                    Some(receiver),
                                    sub_name.clone(),
                                    args,
                                ),
                            ),
                            ClassMethodKind::Constructor => (
                                method.return_type.data.clone(),
                                TypedSubroutineCall::Constructor(
                                    Some(receiver),
                                    sub_name.clone(),
                                    args,
                                ),
                            ),
                        }
                    }
                }
            }
        };
        Ok((ty, term))
    }
}

fn resolve_args(
    subroutine: &WithLoc<Ident>,
    params: &[WithLoc<Variable>],
    args: &WithLoc<ExpressionList>,
    sym_tab: &SubroutineSymbolTable,
) -> Result<Vec<WithLoc<TypedExpression>>, ResolveError> {
    let args = &args.data.0;
    if args.len() != params.len() {
        return Err(ResolveError::ArgumentCountMismatch {
            subroutine: subroutine.clone(),
            expected: params.len(),
            actual: args.len(),
        });
    }
    let args = args
        .iter()
        .zip(params)
        .map(|(arg, param)| {
            arg.resolve_local(sym_tab)
                .and_then(|expr| expr.implicit_cast(&param.data.ty.data))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(args)
}
