use self::private::FromTokensImpl;
use super::*;
use crate::{Keyword, Location, Symbol, Token, WithLoc};
use common::iter::Prependable;
use std::error::Error as StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError<E> {
    #[error(transparent)]
    Tokenize(#[from] E),
    #[error("unexpected EOF")]
    UnexpectedEof,
    #[error("expected {}, found `{}` at {}", _0, _1.data, _1.loc)]
    Expected(String, WithLoc<Token>),
    #[error("in parsing {} at {}", _0, _1.map(|loc| loc.to_string()).unwrap_or_else(|| "<EOF>".to_owned()))]
    Context(
        String,
        Option<Location>,
        #[source] Box<dyn StdError + Send + Sync + 'static>,
    ),
}

fn expected<E>(expected: impl Into<String>, found: WithLoc<Token>) -> ParseError<E> {
    ParseError::Expected(expected.into(), found)
}

trait TokensExt<E> {
    fn token(&mut self) -> Result<WithLoc<Token>, ParseError<E>>;
    fn try_token_with<T, F>(&mut self, f: F) -> Result<Option<WithLoc<T>>, ParseError<E>>
    where
        F: FnOnce(Token) -> Result<T, Token>;
    fn token_with<T, F>(
        &mut self,
        f: F,
    ) -> Result<Result<WithLoc<T>, WithLoc<Token>>, ParseError<E>>
    where
        F: FnOnce(Token) -> Result<T, Token>;
    fn try_keyword(&mut self, keyword: Keyword) -> Result<Option<WithLoc<Keyword>>, ParseError<E>>;
    fn keyword(&mut self, keyword: Keyword) -> Result<WithLoc<Keyword>, ParseError<E>>;
    fn try_symbol(&mut self, symbol: Symbol) -> Result<Option<WithLoc<Symbol>>, ParseError<E>>;
    fn symbol(&mut self, symbol: Symbol) -> Result<WithLoc<Symbol>, ParseError<E>>;
    fn try_ident(&mut self) -> Result<Option<WithLoc<Ident>>, ParseError<E>>;
    fn ident(&mut self) -> Result<WithLoc<Ident>, ParseError<E>>;
    fn try_int(&mut self) -> Result<Option<WithLoc<u16>>, ParseError<E>>;
    fn try_string(&mut self) -> Result<Option<WithLoc<String>>, ParseError<E>>;
    fn void_or_type(&mut self) -> Result<Option<WithLoc<Type>>, ParseError<E>>;
    fn repeat_opt<F, T>(&mut self, f: F) -> Result<Vec<WithLoc<T>>, ParseError<E>>
    where
        F: FnMut(&mut Self) -> Result<Option<WithLoc<T>>, ParseError<E>>;
    fn comma_separated<F, T>(&mut self, f: F) -> Result<Vec<WithLoc<T>>, ParseError<E>>
    where
        F: FnMut(&mut Self) -> Result<WithLoc<T>, ParseError<E>>;
    fn comma_separated_opt<F, T>(&mut self, f: F) -> Result<Vec<WithLoc<T>>, ParseError<E>>
    where
        F: FnMut(&mut Self) -> Result<Option<WithLoc<T>>, ParseError<E>>;
}

impl<I, E> TokensExt<E> for Prependable<I>
where
    I: Iterator<Item = Result<WithLoc<Token>, E>>,
    E: StdError + Send + Sync + 'static,
{
    fn token(&mut self) -> Result<WithLoc<Token>, ParseError<E>> {
        Ok(self.next().ok_or(ParseError::UnexpectedEof)??)
    }

    fn try_token_with<T, F>(&mut self, f: F) -> Result<Option<WithLoc<T>>, ParseError<E>>
    where
        F: FnOnce(Token) -> Result<T, Token>,
    {
        Ok(self
            .next()
            .transpose()?
            .and_then(|token| match token.map(f).transpose() {
                Ok(val) => Some(val),
                Err(token) => {
                    self.prepend(Ok(token));
                    None
                }
            }))
    }

    fn token_with<T, F>(
        &mut self,
        f: F,
    ) -> Result<Result<WithLoc<T>, WithLoc<Token>>, ParseError<E>>
    where
        F: FnOnce(Token) -> Result<T, Token>,
    {
        Ok(self.token()?.map(f).transpose())
    }

    fn try_keyword(&mut self, keyword: Keyword) -> Result<Option<WithLoc<Keyword>>, ParseError<E>> {
        self.try_token_with(|token| match token {
            Token::Keyword(kw) if kw == keyword => Ok(kw),
            _ => Err(token),
        })
    }

    fn keyword(&mut self, keyword: Keyword) -> Result<WithLoc<Keyword>, ParseError<E>> {
        self.token_with(|token| match token {
            Token::Keyword(kw) if kw == keyword => Ok(kw),
            token => Err(token),
        })?
        .map_err(|token| expected(format!("`keyword {}`", keyword), token))
    }

    fn try_symbol(&mut self, symbol: Symbol) -> Result<Option<WithLoc<Symbol>>, ParseError<E>> {
        self.try_token_with(|token| match token {
            Token::Symbol(sym) if sym == symbol => Ok(sym),
            _ => Err(token),
        })
    }

    fn symbol(&mut self, symbol: Symbol) -> Result<WithLoc<Symbol>, ParseError<E>> {
        self.token_with(|token| match token {
            Token::Symbol(sym) if sym == symbol => Ok(sym),
            token => Err(token),
        })?
        .map_err(|token| expected(format!("symbol `{}`", symbol), token))
    }

    fn try_ident(&mut self) -> Result<Option<WithLoc<Ident>>, ParseError<E>> {
        self.try_token_with(|token| match token {
            Token::Ident(ident) => Ok(ident),
            _ => Err(token),
        })
    }

    fn ident(&mut self) -> Result<WithLoc<Ident>, ParseError<E>> {
        self.token_with(|token| match token {
            Token::Ident(ident) => Ok(ident),
            token => Err(token),
        })?
        .map_err(|token| expected("ident", token))
    }

    fn try_int(&mut self) -> Result<Option<WithLoc<u16>>, ParseError<E>> {
        self.try_token_with(|token| match token {
            Token::Int(n) => Ok(n),
            token => Err(token),
        })
    }

    fn try_string(&mut self) -> Result<Option<WithLoc<String>>, ParseError<E>> {
        self.try_token_with(|token| match token {
            Token::String(s) => Ok(s),
            token => Err(token),
        })
    }

    fn void_or_type(&mut self) -> Result<Option<WithLoc<Type>>, ParseError<E>> {
        if self.try_keyword(Keyword::Void)?.is_some() {
            return Ok(None);
        }
        if let Some(ty) = Type::try_from_tokens(self)? {
            return Ok(Some(ty));
        }
        Err(expected("type of `void`", self.token()?))
    }

    fn repeat_opt<F, T>(&mut self, mut f: F) -> Result<Vec<WithLoc<T>>, ParseError<E>>
    where
        F: FnMut(&mut Self) -> Result<Option<WithLoc<T>>, ParseError<E>>,
    {
        let mut values = vec![];
        while let Some(val) = f(self)? {
            values.push(val);
        }
        Ok(values)
    }

    fn comma_separated<F, T>(&mut self, mut f: F) -> Result<Vec<WithLoc<T>>, ParseError<E>>
    where
        F: FnMut(&mut Self) -> Result<WithLoc<T>, ParseError<E>>,
    {
        let mut values = vec![f(self)?];
        while self.try_symbol(Symbol::Comma)?.is_some() {
            values.push(f(self)?);
        }
        Ok(values)
    }

    fn comma_separated_opt<F, T>(&mut self, mut f: F) -> Result<Vec<WithLoc<T>>, ParseError<E>>
    where
        F: FnMut(&mut Self) -> Result<Option<WithLoc<T>>, ParseError<E>>,
    {
        let mut values = vec![];
        loop {
            if let Some(value) = f(self)? {
                values.push(value);
            } else {
                break;
            }
            if self.try_symbol(Symbol::Comma)?.is_none() {
                break;
            }
        }
        Ok(values)
    }
}

mod private {
    use super::*;
    pub trait FromTokensImpl {
        fn context() -> Option<String> {
            None
        }

        fn is_start_token(token: &Token) -> bool;

        fn from_tokens_impl<I, E>(_tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
        where
            Self: Sized,
            I: Iterator<Item = Result<WithLoc<Token>, E>>,
            E: StdError + Send + Sync + 'static,
        {
            unimplemented!()
        }
    }
}

pub trait FromTokens: FromTokensImpl {
    fn try_from_tokens<I, E>(
        tokens: &mut Prependable<I>,
    ) -> Result<Option<WithLoc<Self>>, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        match tokens.peek() {
            Some(Ok(token)) if Self::is_start_token(&token.data) => {
                Ok(Some(Self::from_tokens(tokens)?))
            }
            _ => Ok(None),
        }
    }

    fn from_tokens<I, E>(tokens: &mut Prependable<I>) -> Result<WithLoc<Self>, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        let loc = tokens
            .peek()
            .and_then(|token| token.as_ref().ok())
            .map(|token| token.loc);
        let data = Self::from_tokens_impl(tokens).map_err(|e| {
            if let Some(context) = Self::context() {
                ParseError::Context(context, loc, Box::new(e))
            } else {
                e
            }
        })?;
        let loc = loc.unwrap();
        Ok(WithLoc { loc, data })
    }
}

impl FromTokensImpl for Class {
    fn context() -> Option<String> {
        Some("class declaration".into())
    }

    fn is_start_token(token: &Token) -> bool {
        matches!(token, Token::Keyword(Keyword::Class))
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens.keyword(Keyword::Class)?;
        let name = tokens.ident()?;
        tokens.symbol(Symbol::OpenBrace)?;
        let vars = tokens.repeat_opt(ClassVar::try_from_tokens)?;
        let subs = tokens.repeat_opt(Subroutine::try_from_tokens)?;
        tokens.symbol(Symbol::CloseBrace)?;
        Ok(Self { name, vars, subs })
    }
}
impl FromTokens for Class {}

impl FromTokensImpl for ClassVar {
    fn context() -> Option<String> {
        Some("class variable declaration".into())
    }

    fn is_start_token(token: &Token) -> bool {
        ClassVarKind::is_start_token(token)
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        let kind = ClassVarKind::from_tokens(tokens)?;
        let ty = Type::from_tokens(tokens)?;
        let var_names = tokens.comma_separated(|tokens| tokens.ident())?;
        tokens.symbol(Symbol::Semicolon)?;
        Ok(Self {
            kind,
            ty,
            var_names,
        })
    }
}
impl FromTokens for ClassVar {}

impl FromTokensImpl for ClassVarKind {
    fn is_start_token(token: &Token) -> bool {
        matches!(token, Token::Keyword(Keyword::Static | Keyword::Field))
    }
}
impl FromTokens for ClassVarKind {
    fn from_tokens<I, E>(tokens: &mut Prependable<I>) -> Result<WithLoc<Self>, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens
            .token_with(|token| match token {
                Token::Keyword(Keyword::Static) => Ok(ClassVarKind::Static),
                Token::Keyword(Keyword::Field) => Ok(ClassVarKind::Field),
                token => Err(token),
            })?
            .map_err(|token| expected("`static` or `field`", token))
    }
}

impl FromTokensImpl for Type {
    fn is_start_token(token: &Token) -> bool {
        matches!(
            token,
            Token::Keyword(Keyword::Int | Keyword::Char | Keyword::Boolean) | Token::Ident(_)
        )
    }
}
impl FromTokens for Type {
    fn from_tokens<I, E>(tokens: &mut Prependable<I>) -> Result<WithLoc<Self>, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens
            .token_with(|token| match token {
                Token::Keyword(Keyword::Int) => Ok(Type::Int),
                Token::Keyword(Keyword::Char) => Ok(Type::Char),
                Token::Keyword(Keyword::Boolean) => Ok(Type::Boolean),
                Token::Ident(ident) => Ok(Type::Class(ident)),
                token => Err(token),
            })?
            .map_err(|token| expected("`int`, `char`, `boolean` or class name", token))
    }
}

impl FromTokensImpl for Subroutine {
    fn context() -> Option<String> {
        Some("subroutine declaration".into())
    }

    fn is_start_token(token: &Token) -> bool {
        SubroutineKind::is_start_token(token)
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        let kind = SubroutineKind::from_tokens(tokens)?;
        let return_type = tokens.void_or_type()?;
        let name = tokens.ident()?;
        tokens.symbol(Symbol::OpenParen)?;
        let params = ParameterList::from_tokens(tokens)?;
        tokens.symbol(Symbol::CloseParen)?;
        let body = SubroutineBody::from_tokens(tokens)?;
        Ok(Self {
            kind,
            return_type,
            name,
            params,
            body,
        })
    }
}
impl FromTokens for Subroutine {}

impl FromTokensImpl for SubroutineKind {
    fn is_start_token(token: &Token) -> bool {
        matches!(
            token,
            Token::Keyword(Keyword::Constructor | Keyword::Function | Keyword::Method)
        )
    }
}
impl FromTokens for SubroutineKind {
    fn from_tokens<I, E>(tokens: &mut Prependable<I>) -> Result<WithLoc<Self>, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens
            .token_with(|token| match token {
                Token::Keyword(Keyword::Constructor) => Ok(SubroutineKind::Constructor),
                Token::Keyword(Keyword::Function) => Ok(SubroutineKind::Function),
                Token::Keyword(Keyword::Method) => Ok(SubroutineKind::Method),
                token => Err(token),
            })?
            .map_err(|token| expected("`constructor` or `method`", token))
    }
}

impl FromTokensImpl for ParameterList {
    fn is_start_token(token: &Token) -> bool {
        Parameter::is_start_token(token)
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        Ok(Self(
            tokens.comma_separated_opt(Parameter::try_from_tokens)?,
        ))
    }
}
impl FromTokens for ParameterList {}

impl FromTokensImpl for Parameter {
    fn is_start_token(token: &Token) -> bool {
        Type::is_start_token(token)
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        let ty = Type::from_tokens(tokens)?;
        let var_name = tokens.ident()?;
        Ok(Self { ty, var_name })
    }
}
impl FromTokens for Parameter {}

impl FromTokensImpl for SubroutineBody {
    fn context() -> Option<String> {
        Some("subroutine body".into())
    }

    fn is_start_token(token: &Token) -> bool {
        matches!(token, Token::Symbol(Symbol::OpenBrace))
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens.symbol(Symbol::OpenBrace)?;
        let vars = tokens.repeat_opt(Var::try_from_tokens)?;
        let stmts = StatementList::from_tokens(tokens)?;
        tokens.symbol(Symbol::CloseBrace)?;
        Ok(Self { vars, stmts })
    }
}
impl FromTokens for SubroutineBody {}

impl FromTokensImpl for Var {
    fn context() -> Option<String> {
        Some("variable declaration".into())
    }

    fn is_start_token(token: &Token) -> bool {
        matches!(token, Token::Keyword(Keyword::Var))
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens.keyword(Keyword::Var)?;
        let ty = Type::from_tokens(tokens)?;
        let names = tokens.comma_separated(|tokens| tokens.ident())?;
        tokens.symbol(Symbol::Semicolon)?;
        Ok(Self { ty, names })
    }
}
impl FromTokens for Var {}

impl FromTokensImpl for StatementList {
    fn is_start_token(token: &Token) -> bool {
        Statement::is_start_token(token)
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        Ok(Self(tokens.repeat_opt(Statement::try_from_tokens)?))
    }
}
impl FromTokens for StatementList {}

impl FromTokensImpl for Statement {
    fn is_start_token(token: &Token) -> bool {
        LetStatement::is_start_token(token)
            || IfStatement::is_start_token(token)
            || WhileStatement::is_start_token(token)
            || DoStatement::is_start_token(token)
            || ReturnStatement::is_start_token(token)
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        if let Some(stmt) = LetStatement::try_from_tokens(tokens)? {
            return Ok(Statement::Let(stmt));
        }
        if let Some(stmt) = IfStatement::try_from_tokens(tokens)? {
            return Ok(Statement::If(stmt));
        }
        if let Some(stmt) = WhileStatement::try_from_tokens(tokens)? {
            return Ok(Statement::While(stmt));
        }
        if let Some(stmt) = DoStatement::try_from_tokens(tokens)? {
            return Ok(Statement::Do(stmt));
        }
        if let Some(stmt) = ReturnStatement::try_from_tokens(tokens)? {
            return Ok(Statement::Return(stmt));
        }
        Err(expected("statement", tokens.token()?))
    }
}
impl FromTokens for Statement {}

impl FromTokensImpl for LetStatement {
    fn context() -> Option<String> {
        Some("let statement".into())
    }

    fn is_start_token(token: &Token) -> bool {
        matches!(token, Token::Keyword(Keyword::Let))
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens.keyword(Keyword::Let)?;
        let var_name = tokens.ident()?;
        let mut index = None;
        if tokens.try_symbol(Symbol::OpenBracket)?.is_some() {
            index = Some(Expression::from_tokens(tokens)?);
            tokens.symbol(Symbol::CloseBracket)?;
        }
        tokens.symbol(Symbol::Equal)?;
        let expr = Expression::from_tokens(tokens)?;
        tokens.symbol(Symbol::Semicolon)?;
        Ok(Self {
            var_name,
            index,
            expr,
        })
    }
}
impl FromTokens for LetStatement {}

impl FromTokensImpl for IfStatement {
    fn context() -> Option<String> {
        Some("if statement".into())
    }

    fn is_start_token(token: &Token) -> bool {
        matches!(token, Token::Keyword(Keyword::If))
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens.keyword(Keyword::If)?;
        tokens.symbol(Symbol::OpenParen)?;
        let cond = Expression::from_tokens(tokens)?;
        tokens.symbol(Symbol::CloseParen)?;
        tokens.symbol(Symbol::OpenBrace)?;
        let then_stmts = StatementList::from_tokens(tokens)?;
        tokens.symbol(Symbol::CloseBrace)?;
        let mut else_stmts = None;
        if tokens.try_keyword(Keyword::Else)?.is_some() {
            tokens.symbol(Symbol::OpenBrace)?;
            else_stmts = Some(StatementList::from_tokens(tokens)?);
            tokens.symbol(Symbol::CloseBrace)?;
        }
        Ok(Self {
            cond,
            then_stmts,
            else_stmts,
        })
    }
}
impl FromTokens for IfStatement {}

impl FromTokensImpl for WhileStatement {
    fn context() -> Option<String> {
        Some("while statement".into())
    }

    fn is_start_token(token: &Token) -> bool {
        matches!(token, Token::Keyword(Keyword::While))
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens.keyword(Keyword::While)?;
        tokens.symbol(Symbol::OpenParen)?;
        let cond = Expression::from_tokens(tokens)?;
        tokens.symbol(Symbol::CloseParen)?;
        tokens.symbol(Symbol::OpenBrace)?;
        let stmts = StatementList::from_tokens(tokens)?;
        tokens.symbol(Symbol::CloseBrace)?;
        Ok(Self { cond, stmts })
    }
}
impl FromTokens for WhileStatement {}

impl FromTokensImpl for DoStatement {
    fn context() -> Option<String> {
        Some("do statement".into())
    }

    fn is_start_token(token: &Token) -> bool {
        matches!(token, Token::Keyword(Keyword::Do))
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens.keyword(Keyword::Do)?;
        let sub_call = SubroutineCall::from_tokens(tokens)?;
        tokens.symbol(Symbol::Semicolon)?;

        Ok(Self { sub_call })
    }
}
impl FromTokens for DoStatement {}

impl FromTokensImpl for ReturnStatement {
    fn context() -> Option<String> {
        Some("return statement".into())
    }

    fn is_start_token(token: &Token) -> bool {
        matches!(token, Token::Keyword(Keyword::Return))
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens.keyword(Keyword::Return)?;
        let expr = Expression::try_from_tokens(tokens)?;
        tokens.symbol(Symbol::Semicolon)?;

        Ok(Self { expr })
    }
}
impl FromTokens for ReturnStatement {}

impl FromTokensImpl for Expression {
    fn is_start_token(token: &Token) -> bool {
        Term::is_start_token(token)
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        let term = Term::from_tokens(tokens)?;
        let mut binary_ops = vec![];
        while let Some(op) = BinaryOp::try_from_tokens(tokens)? {
            let operand = Term::from_tokens(tokens)?;
            binary_ops.push((op, operand));
        }
        Ok(Self { term, binary_ops })
    }
}
impl FromTokens for Expression {}

impl FromTokensImpl for Term {
    fn is_start_token(token: &Token) -> bool {
        matches!(
            token,
            Token::Int(_)
                | Token::String(_)
                | Token::Keyword(_)
                | Token::Ident(_)
                | Token::Symbol(Symbol::OpenParen)
        ) || UnaryOp::is_start_token(token)
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        if let Some(n) = tokens.try_int()? {
            return Ok(Self::IntConstant(n));
        }
        if let Some(s) = tokens.try_string()? {
            return Ok(Self::StringConstant(s));
        }
        if let Some(k) = KeywordConstant::try_from_tokens(tokens)? {
            return Ok(Self::KeywordConstant(k));
        }
        if let Some(var) = tokens.try_ident()? {
            if tokens.try_symbol(Symbol::OpenBracket)?.is_some() {
                let index = Expression::from_tokens(tokens)?;
                tokens.symbol(Symbol::CloseBracket)?;
                return Ok(Self::Index(var, Box::new(index)));
            }
            let is_subroutime_call = matches!(
                tokens.peek(),
                Some(Ok(WithLoc {
                    data: Token::Symbol(Symbol::OpenParen | Symbol::Dot),
                    ..
                }))
            );
            if is_subroutime_call {
                let loc = var.loc;
                let data = SubroutineCall::from_tokens_with_ident(var, tokens)?;
                return Ok(Self::SubroutineCall(WithLoc { loc, data }));
            }
            return Ok(Self::Variable(var));
        }
        if tokens.try_symbol(Symbol::OpenParen)?.is_some() {
            let expression = Expression::from_tokens(tokens)?;
            tokens.symbol(Symbol::CloseParen)?;
            return Ok(Self::Expression(Box::new(expression)));
        }
        if let Some(op) = UnaryOp::try_from_tokens(tokens)? {
            let term = Term::from_tokens(tokens)?;
            return Ok(Self::UnaryOp(op, Box::new(term)));
        }
        Err(expected("term", tokens.token()?))
    }
}
impl FromTokens for Term {}

impl FromTokensImpl for SubroutineCall {
    fn is_start_token(token: &Token) -> bool {
        matches!(token, Token::Ident(_))
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        let ident = tokens.ident()?;
        Self::from_tokens_with_ident(ident, tokens)
    }
}
impl FromTokens for SubroutineCall {}

impl SubroutineCall {
    fn from_tokens_with_ident<I, E>(
        ident: WithLoc<Ident>,
        tokens: &mut Prependable<I>,
    ) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        let mut prop = None;
        if tokens.try_symbol(Symbol::Dot)?.is_some() {
            prop = Some(tokens.ident()?);
        }
        tokens.symbol(Symbol::OpenParen)?;
        let args = ExpressionList::from_tokens(tokens)?;
        tokens.symbol(Symbol::CloseParen)?;
        if let Some(prop) = prop {
            Ok(Self::PropertyCall(ident, prop, args))
        } else {
            Ok(Self::SubroutineCall(ident, args))
        }
    }
}

impl FromTokensImpl for ExpressionList {
    fn is_start_token(token: &Token) -> bool {
        Expression::is_start_token(token)
    }

    fn from_tokens_impl<I, E>(tokens: &mut Prependable<I>) -> Result<Self, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        Ok(Self(
            tokens.comma_separated_opt(Expression::try_from_tokens)?,
        ))
    }
}
impl FromTokens for ExpressionList {}

impl FromTokensImpl for BinaryOp {
    fn is_start_token(token: &Token) -> bool {
        matches!(
            token,
            Token::Symbol(
                Symbol::Plus
                    | Symbol::Minus
                    | Symbol::Star
                    | Symbol::Slash
                    | Symbol::Ampersand
                    | Symbol::VertBar
                    | Symbol::Less
                    | Symbol::Greater
                    | Symbol::Equal
            )
        )
    }
}
impl FromTokens for BinaryOp {
    fn from_tokens<I, E>(tokens: &mut Prependable<I>) -> Result<WithLoc<Self>, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens
            .token_with(|token| match token {
                Token::Symbol(Symbol::Plus) => Ok(BinaryOp::Add),
                Token::Symbol(Symbol::Minus) => Ok(BinaryOp::Sub),
                Token::Symbol(Symbol::Star) => Ok(BinaryOp::Mul),
                Token::Symbol(Symbol::Slash) => Ok(BinaryOp::Div),
                Token::Symbol(Symbol::Ampersand) => Ok(BinaryOp::And),
                Token::Symbol(Symbol::VertBar) => Ok(BinaryOp::Or),
                Token::Symbol(Symbol::Less) => Ok(BinaryOp::Lt),
                Token::Symbol(Symbol::Greater) => Ok(BinaryOp::Gt),
                Token::Symbol(Symbol::Equal) => Ok(BinaryOp::Eq),
                token => Err(token),
            })?
            .map_err(|token| expected("binary operator", token))
    }
}

impl FromTokensImpl for UnaryOp {
    fn is_start_token(token: &Token) -> bool {
        matches!(token, Token::Symbol(Symbol::Minus | Symbol::Tilde))
    }
}
impl FromTokens for UnaryOp {
    fn from_tokens<I, E>(tokens: &mut Prependable<I>) -> Result<WithLoc<Self>, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens
            .token_with(|token| match token {
                Token::Symbol(Symbol::Minus) => Ok(UnaryOp::Neg),
                Token::Symbol(Symbol::Tilde) => Ok(UnaryOp::Not),
                token => Err(token),
            })?
            .map_err(|token| expected("`-` or `~`", token))
    }
}

impl FromTokensImpl for KeywordConstant {
    fn is_start_token(token: &Token) -> bool {
        matches!(
            token,
            Token::Keyword(Keyword::True | Keyword::False | Keyword::Null | Keyword::This)
        )
    }
}
impl FromTokens for KeywordConstant {
    fn from_tokens<I, E>(tokens: &mut Prependable<I>) -> Result<WithLoc<Self>, ParseError<E>>
    where
        Self: Sized,
        I: Iterator<Item = Result<WithLoc<Token>, E>>,
        E: StdError + Send + Sync + 'static,
    {
        tokens
            .token_with(|token| match token {
                Token::Keyword(Keyword::True) => Ok(KeywordConstant::True),
                Token::Keyword(Keyword::False) => Ok(KeywordConstant::False),
                Token::Keyword(Keyword::Null) => Ok(KeywordConstant::Null),
                Token::Keyword(Keyword::This) => Ok(KeywordConstant::This),
                token => Err(token),
            })?
            .map_err(|token| expected("`true`, `false`, `null` or `this`", token))
    }
}
