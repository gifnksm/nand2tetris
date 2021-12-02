use crate::{xml::XmlEscape, Error};
use common::fs::FileWriter;
use jack::*;
use std::{
    io::{self, prelude::*},
    path::PathBuf,
};

#[derive(Debug)]
pub(crate) struct AstWriter {
    path: PathBuf,
    writer: FileWriter,
}

impl AstWriter {
    pub(crate) fn open(path: PathBuf) -> Result<Self, Error> {
        let writer = FileWriter::open(&path)
            .map_err(|e| Error::CreateOutputFile(path.to_owned(), e.into()))?;

        Ok(Self { path, writer })
    }

    pub(crate) fn write(&mut self, class: &Class) -> Result<(), Error> {
        self.write_class(0, class)
            .map_err(|e| Error::WriteAst(self.path.to_owned(), e.into()))?;
        Ok(())
    }

    pub(crate) fn persist(self) -> Result<(), Error> {
        self.writer
            .persist()
            .map_err(|e| Error::PersistOutputFile(self.path, e.into()))?;
        Ok(())
    }

    fn write_open(&mut self, indent: usize, tag: &str) -> io::Result<()> {
        writeln!(
            self.writer.writer(),
            "{:indent$}<{tag}>",
            "",
            tag = tag,
            indent = indent * 2
        )?;
        Ok(())
    }

    fn write_close(&mut self, indent: usize, tag: &str) -> io::Result<()> {
        writeln!(
            self.writer.writer(),
            "{:indent$}</{tag}>",
            "",
            tag = tag,
            indent = indent * 2
        )?;
        Ok(())
    }

    fn write_multi(
        &mut self,
        indent: usize,
        tag: &str,
        mut f: impl FnMut(&mut Self, usize) -> io::Result<()>,
    ) -> io::Result<()> {
        self.write_open(indent, tag)?;
        f(self, indent + 1)?;
        self.write_close(indent, tag)?;
        Ok(())
    }

    fn write_single(&mut self, indent: usize, tag: &str, value: &str) -> io::Result<()> {
        writeln!(
            self.writer.writer(),
            "{:indent$}<{tag}> {} </{tag}>",
            "",
            XmlEscape(value),
            tag = tag,
            indent = indent * 2
        )?;
        Ok(())
    }

    fn write_keyword(&mut self, indent: usize, keyword: Keyword) -> io::Result<()> {
        self.write_single(indent, "keyword", keyword.as_str())?;
        Ok(())
    }

    fn write_symbol(&mut self, indent: usize, symbol: Symbol) -> io::Result<()> {
        self.write_single(indent, "symbol", symbol.as_str())?;
        Ok(())
    }

    fn write_ident(&mut self, indent: usize, ident: &Ident) -> io::Result<()> {
        self.write_single(indent, "identifier", ident.as_str())?;
        Ok(())
    }

    fn write_class(&mut self, indent: usize, class: &Class) -> io::Result<()> {
        self.write_multi(indent, "class", |this, indent| {
            this.write_keyword(indent, Keyword::Class)?;
            this.write_ident(indent, &class.name)?;
            this.write_symbol(indent, Symbol::OpenBrace)?;
            for var in &class.vars {
                this.write_class_var(indent, var)?;
            }
            for sub in &class.subs {
                this.write_subroutine(indent, sub)?;
            }
            this.write_symbol(indent, Symbol::CloseBrace)?;
            Ok(())
        })?;
        Ok(())
    }

    fn write_class_var(&mut self, indent: usize, class_var_dec: &ClassVar) -> io::Result<()> {
        self.write_multi(indent, "classVarDec", |this, indent| {
            this.write_class_var_kind(indent, &class_var_dec.kind)?;
            this.write_type(indent, &class_var_dec.ty)?;
            for (i, var_name) in class_var_dec.var_names.iter().enumerate() {
                if i > 0 {
                    this.write_symbol(indent, Symbol::Comma)?;
                }
                this.write_ident(indent, var_name)?;
            }
            this.write_symbol(indent, Symbol::Semicolon)?;
            Ok(())
        })?;
        Ok(())
    }

    fn write_class_var_kind(
        &mut self,
        indent: usize,
        class_var_kind: &ClassVarKind,
    ) -> io::Result<()> {
        match class_var_kind {
            ClassVarKind::Static => self.write_keyword(indent, Keyword::Static)?,
            ClassVarKind::Field => self.write_keyword(indent, Keyword::Field)?,
        }
        Ok(())
    }

    fn write_subroutine(&mut self, indent: usize, sub: &Subroutine) -> io::Result<()> {
        self.write_multi(indent, "subroutineDec", |this, indent| {
            this.write_subroutine_kind(indent, &sub.kind)?;
            if let Some(ty) = &sub.return_type {
                this.write_type(indent, ty)?;
            } else {
                this.write_keyword(indent, Keyword::Void)?;
            }
            this.write_ident(indent, &sub.name)?;
            this.write_symbol(indent, Symbol::OpenParen)?;
            this.write_parameter_list(indent, &sub.params)?;
            this.write_symbol(indent, Symbol::CloseParen)?;
            this.write_subroutine_body(indent, &sub.body)?;
            Ok(())
        })?;
        Ok(())
    }

    fn write_parameter_list(&mut self, indent: usize, params: &[Parameter]) -> io::Result<()> {
        self.write_multi(indent, "parameterList", |this, indent| {
            for (i, param) in params.iter().enumerate() {
                if i > 0 {
                    this.write_symbol(indent, Symbol::Comma)?;
                }
                this.write_type(indent, &param.ty)?;
                this.write_ident(indent, &param.var_name)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn write_type(&mut self, indent: usize, ty: &Type) -> io::Result<()> {
        match ty {
            Type::Int => self.write_keyword(indent, Keyword::Int),
            Type::Char => self.write_keyword(indent, Keyword::Char),
            Type::Boolean => self.write_keyword(indent, Keyword::Boolean),
            Type::Class(class_name) => self.write_ident(indent, class_name),
        }
    }

    fn write_subroutine_kind(&mut self, indent: usize, kind: &SubroutineKind) -> io::Result<()> {
        match kind {
            SubroutineKind::Constructor => self.write_keyword(indent, Keyword::Constructor)?,
            SubroutineKind::Method => self.write_keyword(indent, Keyword::Method)?,
            SubroutineKind::Function => self.write_keyword(indent, Keyword::Function)?,
        }
        Ok(())
    }

    fn write_subroutine_body(&mut self, indent: usize, body: &SubroutineBody) -> io::Result<()> {
        self.write_multi(indent, "subroutineBody", |this, indent| {
            this.write_symbol(indent, Symbol::OpenBrace)?;
            for var in &body.vars {
                this.write_var(indent, var)?;
            }
            this.write_statements(indent, &body.stmts)?;
            this.write_symbol(indent, Symbol::CloseBrace)?;
            Ok(())
        })?;
        Ok(())
    }

    fn write_var(&mut self, indent: usize, var: &Var) -> io::Result<()> {
        self.write_multi(indent, "varDec", |this, indent| {
            this.write_keyword(indent, Keyword::Var)?;
            this.write_type(indent, &var.ty)?;
            for (i, var_name) in var.names.iter().enumerate() {
                if i > 0 {
                    this.write_symbol(indent, Symbol::Comma)?;
                }
                this.write_ident(indent, var_name)?;
            }
            this.write_symbol(indent, Symbol::Semicolon)?;
            Ok(())
        })?;
        Ok(())
    }

    fn write_statements(&mut self, indent: usize, stmts: &[Statement]) -> io::Result<()> {
        self.write_multi(indent, "statements", |this, indent| {
            for stmt in stmts {
                this.write_statement(indent, stmt)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn write_statement(&mut self, indent: usize, stmt: &Statement) -> io::Result<()> {
        match stmt {
            Statement::Let(stmt) => self.write_let_statement(indent, stmt),
            Statement::If(stmt) => self.write_if_statement(indent, stmt),
            Statement::While(stmt) => self.write_while_statement(indent, stmt),
            Statement::Do(stmt) => self.write_do_statement(indent, stmt),
            Statement::Return(stmt) => self.write_return_statement(indent, stmt),
        }
    }

    fn write_let_statement(&mut self, indent: usize, stmt: &LetStatement) -> io::Result<()> {
        self.write_multi(indent, "letStatement", |this, indent| {
            this.write_keyword(indent, Keyword::Let)?;
            this.write_ident(indent, &stmt.var_name)?;
            if let Some(index) = &stmt.index {
                this.write_symbol(indent, Symbol::OpenBracket)?;
                this.write_expression(indent, index)?;
                this.write_symbol(indent, Symbol::CloseBracket)?;
            }
            this.write_symbol(indent, Symbol::Equal)?;
            this.write_expression(indent, &stmt.expr)?;
            this.write_symbol(indent, Symbol::Semicolon)?;
            Ok(())
        })?;
        Ok(())
    }

    fn write_if_statement(&mut self, indent: usize, stmt: &IfStatement) -> io::Result<()> {
        self.write_multi(indent, "ifStatement", |this, indent| {
            this.write_keyword(indent, Keyword::If)?;
            this.write_symbol(indent, Symbol::OpenParen)?;
            this.write_expression(indent, &stmt.cond)?;
            this.write_symbol(indent, Symbol::CloseParen)?;
            this.write_symbol(indent, Symbol::OpenBrace)?;
            this.write_statements(indent, &stmt.then_stmts)?;
            this.write_symbol(indent, Symbol::CloseBrace)?;
            if let Some(stmts) = &stmt.else_stmts {
                this.write_keyword(indent, Keyword::Else)?;
                this.write_symbol(indent, Symbol::OpenBrace)?;
                this.write_statements(indent, stmts)?;
                this.write_symbol(indent, Symbol::CloseBrace)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn write_while_statement(&mut self, indent: usize, stmt: &WhileStatement) -> io::Result<()> {
        self.write_multi(indent, "whileStatement", |this, indent| {
            this.write_keyword(indent, Keyword::While)?;
            this.write_symbol(indent, Symbol::OpenParen)?;
            this.write_expression(indent, &stmt.cond)?;
            this.write_symbol(indent, Symbol::CloseParen)?;
            this.write_symbol(indent, Symbol::OpenBrace)?;
            this.write_statements(indent, &stmt.stmts)?;
            this.write_symbol(indent, Symbol::CloseBrace)?;
            Ok(())
        })?;
        Ok(())
    }

    fn write_do_statement(&mut self, indent: usize, stmt: &DoStatement) -> io::Result<()> {
        self.write_multi(indent, "doStatement", |this, indent| {
            this.write_keyword(indent, Keyword::Do)?;
            this.write_subroutine_call(indent, &stmt.sub_call)?;
            this.write_symbol(indent, Symbol::Semicolon)?;
            Ok(())
        })?;
        Ok(())
    }

    fn write_return_statement(&mut self, indent: usize, stmt: &ReturnStatement) -> io::Result<()> {
        self.write_multi(indent, "returnStatement", |this, indent| {
            this.write_keyword(indent, Keyword::Return)?;
            if let Some(expr) = &stmt.expr {
                this.write_expression(indent, expr)?;
            }
            this.write_symbol(indent, Symbol::Semicolon)?;
            Ok(())
        })?;
        Ok(())
    }

    fn write_expression(&mut self, indent: usize, expr: &Expression) -> io::Result<()> {
        self.write_multi(indent, "expression", |this, indent| {
            this.write_term(indent, &expr.term)?;
            for (op, term) in &expr.binary_ops {
                this.write_binary_op(indent, op)?;
                this.write_term(indent, term)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn write_term(&mut self, indent: usize, term: &Term) -> io::Result<()> {
        self.write_multi(indent, "term", |this, indent| match term {
            Term::IntConstant(n) => this.write_single(indent, "integerConstant", &n.to_string()),
            Term::StringConstant(s) => this.write_single(indent, "stringConstant", s),
            Term::KeywordConstant(k) => this.write_keyword_constant(indent, k),
            Term::Variable(v) => this.write_ident(indent, v),
            Term::Index(var, index) => {
                this.write_ident(indent, var)?;
                this.write_symbol(indent, Symbol::OpenBracket)?;
                this.write_expression(indent, index)?;
                this.write_symbol(indent, Symbol::CloseBracket)?;
                Ok(())
            }
            Term::SubroutineCall(subroutine_call) => {
                this.write_subroutine_call(indent, subroutine_call)
            }
            Term::Expression(expression) => {
                this.write_symbol(indent, Symbol::OpenParen)?;
                this.write_expression(indent, expression)?;
                this.write_symbol(indent, Symbol::CloseParen)?;
                Ok(())
            }
            Term::UnaryOp(op, term) => {
                this.write_unary_op(indent, op)?;
                this.write_term(indent, term)?;
                Ok(())
            }
        })?;

        Ok(())
    }

    fn write_subroutine_call(
        &mut self,
        indent: usize,
        subroutine_call: &SubroutineCall,
    ) -> io::Result<()> {
        match subroutine_call {
            SubroutineCall::SubroutineCall(function_name, args) => {
                self.write_ident(indent, function_name)?;
                self.write_symbol(indent, Symbol::OpenParen)?;
                self.write_expression_list(indent, args)?;
                self.write_symbol(indent, Symbol::CloseParen)?;
                Ok(())
            }
            SubroutineCall::PropertyCall(class_name, method_name, args) => {
                self.write_ident(indent, class_name)?;
                self.write_symbol(indent, Symbol::Dot)?;
                self.write_ident(indent, method_name)?;
                self.write_symbol(indent, Symbol::OpenParen)?;
                self.write_expression_list(indent, args)?;
                self.write_symbol(indent, Symbol::CloseParen)?;
                Ok(())
            }
        }
    }

    fn write_expression_list(&mut self, indent: usize, args: &[Expression]) -> io::Result<()> {
        self.write_multi(indent, "expressionList", |this, indent| {
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    this.write_symbol(indent, Symbol::Comma)?;
                }
                this.write_expression(indent, arg)?;
            }
            Ok(())
        })?;
        Ok(())
    }

    fn write_binary_op(&mut self, indent: usize, op: &BinaryOp) -> io::Result<()> {
        match op {
            BinaryOp::Add => self.write_symbol(indent, Symbol::Plus),
            BinaryOp::Sub => self.write_symbol(indent, Symbol::Minus),
            BinaryOp::Mul => self.write_symbol(indent, Symbol::Star),
            BinaryOp::Div => self.write_symbol(indent, Symbol::Slash),
            BinaryOp::And => self.write_symbol(indent, Symbol::Ampersand),
            BinaryOp::Or => self.write_symbol(indent, Symbol::VertBar),
            BinaryOp::Lt => self.write_symbol(indent, Symbol::Less),
            BinaryOp::Gt => self.write_symbol(indent, Symbol::Greater),
            BinaryOp::Eq => self.write_symbol(indent, Symbol::Equal),
        }
    }

    fn write_unary_op(&mut self, indent: usize, op: &UnaryOp) -> io::Result<()> {
        match op {
            UnaryOp::Neg => self.write_symbol(indent, Symbol::Minus),
            UnaryOp::Not => self.write_symbol(indent, Symbol::Tilde),
        }
    }

    fn write_keyword_constant(&mut self, indent: usize, k: &KeywordConstant) -> io::Result<()> {
        match k {
            KeywordConstant::True => self.write_keyword(indent, Keyword::True),
            KeywordConstant::False => self.write_keyword(indent, Keyword::False),
            KeywordConstant::Null => self.write_keyword(indent, Keyword::Null),
            KeywordConstant::This => self.write_keyword(indent, Keyword::This),
        }
    }
}
