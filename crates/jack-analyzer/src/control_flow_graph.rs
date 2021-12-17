use crate::xml::{WriteXml, XmlWriter};
use jack::control_flow_graph::{BasicBlock, CfgClass, CfgStatement, CfgSubroutine, Exit};
use std::io;

impl WriteXml for CfgClass {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> std::io::Result<()> {
        let Self {
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

impl WriteXml for CfgSubroutine {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let Self {
            kind,
            return_type,
            name,
            params,
            vars,
            entry_id,
            blocks,
        } = self;

        writer.write_multi(indent, "subroutine", |indent, writer| {
            writer.write_single(indent, "kind", kind.data.as_str())?;
            writer.write_single(indent, "name", name.data.as_str())?;
            writer.write_single(indent, "returnType", return_type.data.as_str())?;
            writer.write_list(indent, "parameterList", params)?;
            writer.write_list(indent, "variableList", vars)?;
            writer.write_single(indent, "entryId", &entry_id.to_string())?;
            writer.write_list(indent, "blockList", blocks)?;

            Ok(())
        })
    }
}

impl WriteXml for BasicBlock {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let Self { id, stmts, exit } = self;
        writer.write_multi(indent, "basicBlock", |indent, writer| {
            writer.write_single(indent, "id", &id.to_string())?;
            writer.write_list(indent, "statementList", stmts)?;
            exit.write_xml(indent, writer)?;
            Ok(())
        })
    }
}

impl WriteXml for CfgStatement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        match self {
            Self::Let(stmt) => stmt.write_xml(indent, writer),
            Self::Do(stmt) => stmt.write_xml(indent, writer),
        }
    }
}

impl WriteXml for Exit {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "exit", |indent, writer| {
            match self {
                Self::Return(expr) => writer.write_multi(indent, "return", |indent, writer| {
                    writer.write_opt(indent, "expression", expr)?;
                    Ok(())
                })?,
                Self::Goto(label) => writer.write_single(indent, "goto", &label.to_string())?,
                Self::If(cond, then_label, else_label) => {
                    writer.write_multi(indent, "if", |indent, writer| {
                        writer.write_labeled(indent, "condition", cond)?;
                        writer.write_single(indent, "thenLabel", &then_label.to_string())?;
                        writer.write_single(indent, "elseLabel", &else_label.to_string())?;
                        Ok(())
                    })?
                }
                Self::Unreachable => writer.write_single(indent, "unreachable", "")?,
            }
            Ok(())
        })
    }
}
