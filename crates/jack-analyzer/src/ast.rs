use crate::xml::{WriteXml, XmlWriter};
use jack::{ast::*, token::*};
use std::io;

impl WriteXml for Class {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "class", |indent, writer| {
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

impl WriteXml for ClassVarDec {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "classVarDec", |indent, writer| {
            self.kind.write_xml(indent, writer)?;
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

impl WriteXml for ReturnType {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        match self {
            ReturnType::Void => Keyword::Void.write_xml(indent, writer),
            ReturnType::Type(ty) => ty.write_xml(indent, writer),
        }
    }
}

impl WriteXml for Subroutine {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "subroutineDec", |indent, writer| {
            self.kind.write_xml(indent, writer)?;
            self.return_type.write_xml(indent, writer)?;
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
        writer.write_list_with_sep(indent, "parameterList", &self.0, Symbol::Comma)
    }
}

impl WriteXml for Parameter {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        self.ty.write_xml(indent, writer)?;
        self.name.write_xml(indent, writer)?;
        Ok(())
    }
}

impl WriteXml for SubroutineBody {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "subroutineBody", |indent, writer| {
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

impl WriteXml for LocalVarDec {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "varDec", |indent, writer| {
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
        writer.write_list(indent, "statements", &self.0)
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
        writer.write_multi(indent, "letStatement", |indent, writer| {
            Keyword::Let.write_xml(indent, writer)?;
            self.target.write_xml(indent, writer)?;
            if let Some(index) = &self.target_index {
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
        writer.write_multi(indent, "ifStatement", |indent, writer| {
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
        writer.write_multi(indent, "whileStatement", |indent, writer| {
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
        writer.write_multi(indent, "doStatement", |indent, writer| {
            Keyword::Do.write_xml(indent, writer)?;
            self.sub_call.write_xml(indent, writer)?;
            Symbol::Semicolon.write_xml(indent, writer)?;
            Ok(())
        })
    }
}

impl WriteXml for ReturnStatement {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "returnStatement", |indent, writer| {
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
        writer.write_multi(indent, "expression", |indent, writer| {
            self.term.write_xml(indent, writer)?;
            for (op, term) in &self.binary_ops {
                op.write_xml(indent, writer)?;
                term.write_xml(indent, writer)?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

impl WriteXml for Term {
    fn write_xml(&self, indent: usize, writer: &mut XmlWriter) -> io::Result<()> {
        writer.write_multi(indent, "term", |indent, writer| match self {
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
        writer.write_multi(indent, "expressionList", |indent, writer| {
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
