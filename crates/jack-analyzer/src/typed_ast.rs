use crate::xml::{WriteXml, XmlWriter};
use jack::{
    symbol_table::{Field, LocalVariable, Parameter, StaticVariable, VarSymbol},
    token::Keyword,
    typed_ast::*,
};
use std::io;

impl WriteXml for TypedClass {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let TypedClass {
            name,
            static_vars,
            fields,
            subs,
        } = self;
        writer.write_multi(indent, "class", |indent, writer| {
            writer.write_single(indent, "name", name.data.as_str())?;
            writer.write_list(indent, "staticVarList", static_vars)?;
            writer.write_list(indent, "fieldList", fields)?;
            writer.write_list(indent, "subroutineList", subs)?;
            Ok(())
        })
    }
}

impl WriteXml for Variable {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "variable", |indent, writer| {
            writer.write_single(indent, "name", self.name.data.as_str())?;
            writer.write_single(indent, "type", self.ty.data.as_str())?;
            Ok(())
        })
    }
}

impl WriteXml for VarSymbol {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        match self {
            VarSymbol::StaticVariable(v) => v.write_xml(indent, writer),
            VarSymbol::Field(f) => f.write_xml(indent, writer),
            VarSymbol::LocalVariable(v) => v.write_xml(indent, writer),
            VarSymbol::Parameter(p) => p.write_xml(indent, writer),
        }
    }
}

impl WriteXml for StaticVariable {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let Self { name, ty } = self;
        writer.write_multi(indent, "staticVariable", |indent, writer| {
            writer.write_single(indent, "name", name.data.as_str())?;
            writer.write_single(indent, "type", ty.data.as_str())?;
            Ok(())
        })
    }
}

impl WriteXml for Field {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let Self {
            name,
            ty,
            slot_index,
        } = self;
        writer.write_multi(indent, "field", |indent, writer| {
            writer.write_single(indent, "name", name.data.as_str())?;
            writer.write_single(indent, "type", ty.data.as_str())?;
            writer.write_single(indent, "slotIndex", &slot_index.to_string())?;
            Ok(())
        })
    }
}

impl WriteXml for LocalVariable {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let Self {
            name,
            ty,
            slot_index,
        } = self;
        writer.write_multi(indent, "localVariable", |indent, writer| {
            writer.write_single(indent, "name", name.data.as_str())?;
            writer.write_single(indent, "type", ty.data.as_str())?;
            writer.write_single(indent, "slotIndex", &slot_index.to_string())?;
            Ok(())
        })
    }
}

impl WriteXml for Parameter {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let Self {
            name,
            ty,
            slot_index,
        } = self;
        writer.write_multi(indent, "parameter", |indent, writer| {
            writer.write_single(indent, "name", name.data.as_str())?;
            writer.write_single(indent, "type", ty.data.as_str())?;
            writer.write_single(indent, "slotIndex", &slot_index.to_string())?;
            Ok(())
        })
    }
}

impl WriteXml for TypedSubroutine {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let Self {
            kind,
            return_type,
            name,
            params,
            vars,
            stmts,
        } = self;

        writer.write_multi(indent, "subroutine", |indent, writer| {
            writer.write_single(indent, "kind", kind.data.as_str())?;
            writer.write_single(indent, "name", name.data.as_str())?;
            writer.write_single(indent, "returnType", return_type.data.as_str())?;
            writer.write_list(indent, "parameterList", params)?;
            writer.write_list(indent, "variableList", vars)?;
            writer.write_list(indent, "statementList", stmts)?;

            Ok(())
        })
    }
}

impl WriteXml for TypedStatement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        match self {
            Self::Let(stmt) => stmt.write_xml(indent, writer),
            Self::If(stmt) => stmt.write_xml(indent, writer),
            Self::While(stmt) => stmt.write_xml(indent, writer),
            Self::Do(stmt) => stmt.write_xml(indent, writer),
            Self::Return(stmt) => stmt.write_xml(indent, writer),
        }
    }
}

impl WriteXml for TypedLetStatement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let Self {
            target,
            target_index,
            expr,
        } = self;
        writer.write_multi(indent, "letStatement", |indent, writer| {
            writer.write_multi(indent, "target", |indent, writer| {
                target.write_xml(indent, writer)?;
                writer.write_opt(indent, "index", target_index)?;
                Ok(())
            })?;
            expr.write_xml(indent, writer)?;
            Ok(())
        })
    }
}

impl WriteXml for TypedIfStatement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let Self {
            cond,
            then_stmts,
            else_stmts,
        } = self;
        writer.write_multi(indent, "ifStatement", |indent, writer| {
            writer.write_labeled(indent, "condition", cond)?;
            writer.write_list(indent, "thenStatementList", then_stmts)?;
            if let Some(stmts) = else_stmts {
                writer.write_list(indent, "elseStatementList", stmts)?;
            }
            Ok(())
        })
    }
}

impl WriteXml for TypedWhileStatement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let Self { cond, stmts } = self;
        writer.write_multi(indent, "whileStatement", |indent, writer| {
            writer.write_labeled(indent, "condition", cond)?;
            writer.write_list(indent, "statementList", stmts)?;
            Ok(())
        })
    }
}

impl WriteXml for TypedDoStatement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let Self { sub_call } = self;
        writer.write_labeled(indent, "doStatement", sub_call)
    }
}

impl WriteXml for TypedReturnStatement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let Self { expr } = self;
        writer.write_multi(indent, "returnStatement", |indent, writer| {
            writer.write_opt(indent, "expression", expr)?;
            Ok(())
        })
    }
}

impl WriteXml for TypedExpression {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let TypedExpression { ty, term } = self;
        writer.write_multi(indent, "expression", |indent, writer| {
            writer.write_single(indent, "type", ty.as_str())?;
            term.write_xml(indent, writer)?;
            Ok(())
        })
    }
}

impl WriteXml for TypedTerm {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        match self {
            Self::Int(n) => writer.write_single(indent, "int", &n.data.to_string()),
            Self::String(s) => writer.write_single(indent, "string", &s.data),
            Self::Bool(b) => {
                if b.data {
                    Keyword::True.write_xml(indent, writer)
                } else {
                    Keyword::False.write_xml(indent, writer)
                }
            }
            Self::Null => Keyword::Null.write_xml(indent, writer),
            Self::This => Keyword::This.write_xml(indent, writer),
            Self::Var(v) => v.write_xml(indent, writer),
            Self::Index(arr, idx) => writer.write_multi(indent, "index", |indent, writer| {
                writer.write_labeled(indent, "array", arr)?;
                writer.write_labeled(indent, "index", idx)?;
                Ok(())
            }),
            Self::SubroutineCall(sub) => writer.write_labeled(indent, "subroutineCall", sub),
            Self::UnaryOp(operator, operand) => {
                writer.write_multi(indent, "unaryOp", |indent, writer| {
                    writer.write_single(indent, "operator", operator.data.as_str())?;
                    writer.write_labeled(indent, "operand", operand)?;
                    Ok(())
                })
            }
            Self::BinaryOp(operator, lhs, rhs) => {
                writer.write_multi(indent, "binaryOp", |indent, writer| {
                    writer.write_single(indent, "operator", operator.data.as_str())?;
                    writer.write_labeled(indent, "lhs", lhs)?;
                    writer.write_labeled(indent, "rhs", rhs)?;
                    Ok(())
                })
            }
        }
    }
}

impl WriteXml for TypedSubroutineCall {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        match self {
            Self::Method(receiver, method, args) => {
                writer.write_multi(indent, "methodCall", |indent, writer| {
                    writer.write_opt(indent, "receiver", receiver)?;
                    writer.write_single(indent, "method", method.data.as_str())?;
                    writer.write_list(indent, "argumentList", args)?;
                    Ok(())
                })
            }
            Self::Function(class, func, args) => {
                writer.write_multi(indent, "functionCall", |indent, writer| {
                    if let Some(class) = class {
                        writer.write_single(indent, "class", class.data.as_str())?;
                    }
                    writer.write_single(indent, "function", func.data.as_str())?;
                    writer.write_list(indent, "argumentList", args)?;
                    Ok(())
                })
            }
            Self::Constructor(class, cons, args) => {
                writer.write_multi(indent, "constructorCall", |indent, writer| {
                    if let Some(class) = class {
                        writer.write_single(indent, "class", class.data.as_str())?;
                    }
                    writer.write_single(indent, "constructor", cons.data.as_str())?;
                    writer.write_list(indent, "argumentList", args)?;
                    Ok(())
                })
            }
        }
    }
}
