use crate::xml::{WriteXml, XmlWriter};
use jack::*;
use std::io;

impl WriteXml for Class {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "class", |writer, indent| {
            Keyword::Class.write_xml(indent, writer)?;
            self.name.write_xml(indent, writer)?;
            Symbol::OpenBrace.write_xml(indent, writer)?;
            for var in &self.vars {
                var.write_xml(indent, writer)?;
            }
            for sub in &self.subs {
                sub.write_xml(indent, writer)?;
            }
            Symbol::CloseBrace.write_xml(indent, writer)?;
            Ok(())
        })
    }
}

impl WriteXml for ClassVar {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "classVarDec", |writer, indent| {
            self.kind.write_xml(indent, writer)?;
            self.ty.write_xml(indent, writer)?;
            for (i, var_name) in self.var_names.iter().enumerate() {
                if i > 0 {
                    Symbol::Comma.write_xml(indent, writer)?;
                }
                var_name.write_xml(indent, writer)?;
            }
            Symbol::Semicolon.write_xml(indent, writer)?;
            Ok(())
        })
    }
}

impl WriteXml for ClassVarKind {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let kw = match self {
            ClassVarKind::Static => Keyword::Static,
            ClassVarKind::Field => Keyword::Field,
        };
        kw.write_xml(indent, writer)
    }
}

impl WriteXml for Type {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let kw = match self {
            Type::Int => Keyword::Int,
            Type::Char => Keyword::Char,
            Type::Boolean => Keyword::Boolean,
            Type::Class(class_name) => return class_name.write_xml(indent, writer),
        };
        kw.write_xml(indent, writer)
    }
}

impl WriteXml for Subroutine {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "subroutineDec", |writer, indent| {
            self.kind.write_xml(indent, writer)?;
            if let Some(ty) = &self.return_type {
                ty.write_xml(indent, writer)?;
            } else {
                Keyword::Void.write_xml(indent, writer)?;
            }
            self.name.write_xml(indent, writer)?;
            Symbol::OpenParen.write_xml(indent, writer)?;
            self.params.write_xml(indent, writer)?;
            Symbol::CloseParen.write_xml(indent, writer)?;
            self.body.write_xml(indent, writer)?;
            Ok(())
        })
    }
}

impl WriteXml for SubroutineKind {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let kw = match self {
            SubroutineKind::Constructor => Keyword::Constructor,
            SubroutineKind::Method => Keyword::Method,
            SubroutineKind::Function => Keyword::Function,
        };
        kw.write_xml(indent, writer)
    }
}

impl WriteXml for ParameterList {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "parameterList", |writer, indent| {
            for (i, param) in self.0.iter().enumerate() {
                if i > 0 {
                    Symbol::Comma.write_xml(indent, writer)?;
                }
                param.write_xml(indent, writer)?;
            }
            Ok(())
        })
    }
}

impl WriteXml for Parameter {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        self.ty.write_xml(indent, writer)?;
        self.var_name.write_xml(indent, writer)?;
        Ok(())
    }
}

impl WriteXml for SubroutineBody {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "subroutineBody", |writer, indent| {
            Symbol::OpenBrace.write_xml(indent, writer)?;
            for var in &self.vars {
                var.write_xml(indent, writer)?;
            }
            self.stmts.write_xml(indent, writer)?;
            Symbol::CloseBrace.write_xml(indent, writer)?;
            Ok(())
        })
    }
}

impl WriteXml for Var {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "varDec", |writer, indent| {
            Keyword::Var.write_xml(indent, writer)?;
            self.ty.write_xml(indent, writer)?;
            for (i, var_name) in self.names.iter().enumerate() {
                if i > 0 {
                    Symbol::Comma.write_xml(indent, writer)?;
                }
                var_name.write_xml(indent, writer)?;
            }
            Symbol::Semicolon.write_xml(indent, writer)?;
            Ok(())
        })
    }
}

impl WriteXml for StatementList {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "statements", |writer, indent| {
            for stmt in &self.0 {
                stmt.write_xml(indent, writer)?;
            }
            Ok(())
        })
    }
}

impl WriteXml for Statement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        match self {
            Statement::Let(stmt) => stmt.write_xml(indent, writer),
            Statement::If(stmt) => stmt.write_xml(indent, writer),
            Statement::While(stmt) => stmt.write_xml(indent, writer),
            Statement::Do(stmt) => stmt.write_xml(indent, writer),
            Statement::Return(stmt) => stmt.write_xml(indent, writer),
        }
    }
}

impl WriteXml for LetStatement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "letStatement", |writer, indent| {
            Keyword::Let.write_xml(indent, writer)?;
            self.var_name.write_xml(indent, writer)?;
            if let Some(index) = &self.index {
                Symbol::OpenBracket.write_xml(indent, writer)?;
                index.write_xml(indent, writer)?;
                Symbol::CloseBracket.write_xml(indent, writer)?;
            }
            Symbol::Equal.write_xml(indent, writer)?;
            self.expr.write_xml(indent, writer)?;
            Symbol::Semicolon.write_xml(indent, writer)?;
            Ok(())
        })
    }
}

impl WriteXml for IfStatement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "ifStatement", |writer, indent| {
            Keyword::If.write_xml(indent, writer)?;
            Symbol::OpenParen.write_xml(indent, writer)?;
            self.cond.write_xml(indent, writer)?;
            Symbol::CloseParen.write_xml(indent, writer)?;
            Symbol::OpenBrace.write_xml(indent, writer)?;
            self.then_stmts.write_xml(indent, writer)?;
            Symbol::CloseBrace.write_xml(indent, writer)?;
            if let Some(stmts) = &self.else_stmts {
                Keyword::Else.write_xml(indent, writer)?;
                Symbol::OpenBrace.write_xml(indent, writer)?;
                stmts.write_xml(indent, writer)?;
                Symbol::CloseBrace.write_xml(indent, writer)?;
            }
            Ok(())
        })
    }
}

impl WriteXml for WhileStatement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "whileStatement", |writer, indent| {
            Keyword::While.write_xml(indent, writer)?;
            Symbol::OpenParen.write_xml(indent, writer)?;
            self.cond.write_xml(indent, writer)?;
            Symbol::CloseParen.write_xml(indent, writer)?;
            Symbol::OpenBrace.write_xml(indent, writer)?;
            self.stmts.write_xml(indent, writer)?;
            Symbol::CloseBrace.write_xml(indent, writer)?;
            Ok(())
        })
    }
}

impl WriteXml for DoStatement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "doStatement", |write, indent| {
            Keyword::Do.write_xml(indent, write)?;
            self.sub_call.write_xml(indent, write)?;
            Symbol::Semicolon.write_xml(indent, write)?;
            Ok(())
        })
    }
}

impl WriteXml for ReturnStatement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "returnStatement", |writer, indent| {
            Keyword::Return.write_xml(indent, writer)?;
            if let Some(expr) = &self.expr {
                expr.write_xml(indent, writer)?;
            }
            Symbol::Semicolon.write_xml(indent, writer)?;
            Ok(())
        })
    }
}

impl WriteXml for Expression {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "expression", |this, indent| {
            self.term.write_xml(indent, this)?;
            for (op, term) in &self.binary_ops {
                op.write_xml(indent, this)?;
                term.write_xml(indent, this)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl WriteXml for Term {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "term", |writer, indent| match self {
            Term::IntConstant(n) => n.write_xml(indent, writer),
            Term::StringConstant(s) => s.write_xml(indent, writer),
            Term::KeywordConstant(k) => k.write_xml(indent, writer),
            Term::Variable(v) => v.write_xml(indent, writer),
            Term::Index(var, index) => {
                var.write_xml(indent, writer)?;
                Symbol::OpenBracket.write_xml(indent, writer)?;
                index.write_xml(indent, writer)?;
                Symbol::CloseBracket.write_xml(indent, writer)?;
                Ok(())
            }
            Term::SubroutineCall(subroutine_call) => subroutine_call.write_xml(indent, writer),
            Term::Expression(expression) => {
                Symbol::OpenParen.write_xml(indent, writer)?;
                expression.write_xml(indent, writer)?;
                Symbol::CloseParen.write_xml(indent, writer)?;
                Ok(())
            }
            Term::UnaryOp(op, term) => {
                op.write_xml(indent, writer)?;
                term.write_xml(indent, writer)?;
                Ok(())
            }
        })
    }
}

impl WriteXml for SubroutineCall {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        match self {
            SubroutineCall::SubroutineCall(function_name, args) => {
                function_name.write_xml(indent, writer)?;
                Symbol::OpenParen.write_xml(indent, writer)?;
                args.write_xml(indent, writer)?;
                Symbol::CloseParen.write_xml(indent, writer)?;
                Ok(())
            }
            SubroutineCall::PropertyCall(class_name, method_name, args) => {
                class_name.write_xml(indent, writer)?;
                Symbol::Dot.write_xml(indent, writer)?;
                method_name.write_xml(indent, writer)?;
                Symbol::OpenParen.write_xml(indent, writer)?;
                args.write_xml(indent, writer)?;
                Symbol::CloseParen.write_xml(indent, writer)?;
                Ok(())
            }
        }
    }
}

impl WriteXml for ExpressionList {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "expressionList", |writer, indent| {
            for (i, expr) in self.0.iter().enumerate() {
                if i > 0 {
                    Symbol::Comma.write_xml(indent, writer)?;
                }
                expr.write_xml(indent, writer)?;
            }
            Ok(())
        })
    }
}

impl WriteXml for BinaryOp {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let sym = match self {
            BinaryOp::Add => Symbol::Plus,
            BinaryOp::Sub => Symbol::Minus,
            BinaryOp::Mul => Symbol::Star,
            BinaryOp::Div => Symbol::Slash,
            BinaryOp::And => Symbol::Ampersand,
            BinaryOp::Or => Symbol::VertBar,
            BinaryOp::Lt => Symbol::Less,
            BinaryOp::Gt => Symbol::Greater,
            BinaryOp::Eq => Symbol::Equal,
        };
        sym.write_xml(indent, writer)
    }
}

impl WriteXml for UnaryOp {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let sym = match self {
            UnaryOp::Neg => Symbol::Minus,
            UnaryOp::Not => Symbol::Tilde,
        };
        sym.write_xml(indent, writer)
    }
}

impl WriteXml for KeywordConstant {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        let kw = match self {
            KeywordConstant::True => Keyword::True,
            KeywordConstant::False => Keyword::False,
            KeywordConstant::Null => Keyword::Null,
            KeywordConstant::This => Keyword::This,
        };
        kw.write_xml(indent, writer)
    }
}
