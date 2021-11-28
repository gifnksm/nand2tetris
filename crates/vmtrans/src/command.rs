use crate::{code_gen::CodeGen, ModuleName};
use hasm::{Comp, Imm, Jump, Label as AsmLabel, Statement};
use std::{borrow::Borrow, fmt, num::ParseIntError, str::FromStr};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Command {
    Add,
    Sub,
    Neg,
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,
    Push(Segment, Imm),
    Pop(Segment, Imm),
    Label(Label),
    Goto(Label),
    IfGoto(Label),
    Function(FuncName, u8),
    Call(FuncName, u8),
    Return,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Segment {
    Argument,
    Local,
    Static,
    Constant,
    This,
    That,
    Pointer,
    Temp,
}

impl Segment {
    fn len(&self) -> u16 {
        match self {
            Self::Argument => u16::from(u8::MAX) + 1,
            Self::Local | Self::Static | Self::Constant | Self::This | Self::That => {
                Imm::MAX.value() + 1
            }
            Self::Pointer => 2,
            Self::Temp => 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Ident(String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncName(String);

impl Borrow<str> for FuncName {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<Ident> for FuncName {
    fn from(s: Ident) -> Self {
        Self(s.0)
    }
}

impl fmt::Display for FuncName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl FuncName {
    pub(crate) fn toplevel() -> Self {
        Self("$toplevel".to_string())
    }

    pub(crate) fn bootstrap() -> Self {
        Self("$bootstrap".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Label(String);

impl Borrow<str> for Label {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl From<Ident> for Label {
    fn from(s: Ident) -> Self {
        Self(s.0)
    }
}

impl fmt::Display for Label {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Label {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Error)]
pub enum ParseCommandError {
    #[error("cannot parse command from empty string")]
    Empty,
    #[error("invalid command: {}", _0)]
    InvalidCommand(String),
    #[error("too few arguments")]
    TooFewArguments,
    #[error("too many arguments")]
    TooManyArguments,
    #[error(transparent)]
    InvalidSegment(#[from] ParseSegmentError),
    #[error("invalid index: {}", _1)]
    InvalidIndex(#[source] ParseIntError, String),
    #[error("too large index: {} (segment length: {})", _0, _1)]
    TooLargeIndex(u16, u16),
    #[error("invalid operand: {} {}", _0, _1)]
    InvalidOperand(String, String),
    #[error(transparent)]
    InvalidIdent(#[from] ParseIdentError),
    #[error("invalid arity: {}", _1)]
    InvalidArity(#[source] ParseIntError, String),
    #[error("too large arity: {}", _0)]
    TooLargeArity(u16),
}

impl FromStr for Command {
    type Err = ParseCommandError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        enum Kind {
            NoArg(Command),
            SegmentIndex(fn(Segment, Imm) -> Command),
            Label(fn(Label) -> Command),
            FuncNameArity(fn(FuncName, u8) -> Command),
        }
        let mut cs = s.split_whitespace();
        let kind_str = cs.next().ok_or(Self::Err::Empty)?;
        let kind = match kind_str {
            "add" => Kind::NoArg(Self::Add),
            "sub" => Kind::NoArg(Self::Sub),
            "neg" => Kind::NoArg(Self::Neg),
            "eq" => Kind::NoArg(Self::Eq),
            "gt" => Kind::NoArg(Self::Gt),
            "lt" => Kind::NoArg(Self::Lt),
            "and" => Kind::NoArg(Self::And),
            "or" => Kind::NoArg(Self::Or),
            "not" => Kind::NoArg(Self::Not),
            "push" => Kind::SegmentIndex(Self::Push),
            "pop" => Kind::SegmentIndex(Self::Pop),
            "label" => Kind::Label(Self::Label),
            "goto" => Kind::Label(Self::Goto),
            "if-goto" => Kind::Label(Self::IfGoto),
            "function" => Kind::FuncNameArity(Self::Function),
            "call" => Kind::FuncNameArity(Self::Call),
            "return" => Kind::NoArg(Self::Return),
            command => return Err(Self::Err::InvalidCommand(command.into())),
        };

        let command = match kind {
            Kind::NoArg(command) => command,
            Kind::SegmentIndex(f) => {
                let segment_str = cs.next().ok_or(Self::Err::TooFewArguments)?;
                let index_str = cs.next().ok_or(Self::Err::TooFewArguments)?;

                let segment = Segment::from_str(segment_str)?;
                if kind_str == "pop" && segment == Segment::Constant {
                    return Err(Self::Err::InvalidOperand(
                        kind_str.into(),
                        segment_str.into(),
                    ));
                }
                let segment_len = segment.len();
                let index = u16::from_str(index_str)
                    .map_err(|e| Self::Err::InvalidIndex(e, index_str.into()))?;
                if index >= segment_len {
                    return Err(Self::Err::TooLargeIndex(index, segment_len));
                }
                let index = Imm::try_new(index).unwrap(); // must be successful

                f(segment, index)
            }
            Kind::Label(f) => {
                let label_str = cs.next().ok_or(Self::Err::TooFewArguments)?;
                let label = Ident::from_str(label_str)?.into();
                f(label)
            }
            Kind::FuncNameArity(f) => {
                let function_str = cs.next().ok_or(Self::Err::TooFewArguments)?;
                let arity_str = cs.next().ok_or(Self::Err::TooFewArguments)?;

                let function = Ident::from_str(function_str)?.into();
                let arity = u8::from_str(arity_str)
                    .map_err(|e| Self::Err::InvalidArity(e, arity_str.into()))?;

                f(function, arity)
            }
        };

        if cs.next().is_some() {
            return Err(Self::Err::TooManyArguments);
        }

        Ok(command)
    }
}

#[derive(Debug, Error)]
pub enum ParseSegmentError {
    #[error("invalid segment: {}", _0)]
    InvalidSegment(String),
}

impl FromStr for Segment {
    type Err = ParseSegmentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let seg = match s {
            "argument" => Self::Argument,
            "local" => Self::Local,
            "static" => Self::Static,
            "constant" => Self::Constant,
            "this" => Self::This,
            "that" => Self::That,
            "pointer" => Self::Pointer,
            "temp" => Self::Temp,
            _ => return Err(Self::Err::InvalidSegment(s.into())),
        };
        Ok(seg)
    }
}

#[derive(Debug, Error)]
pub enum ParseIdentError {
    #[error("invalid ident: {}", _0)]
    InvalidIdent(String),
}

impl FromStr for Ident {
    type Err = ParseIdentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut cs = s.chars();
        let is_valid = cs
            .next()
            .map(|ch| ch.is_ascii_alphabetic() || ch == '_' || ch == '.' || ch == ':')
            .unwrap_or(false)
            && cs.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == ':');
        if !is_valid {
            return Err(Self::Err::InvalidIdent(s.into()));
        }
        Ok(Ident(s.into()))
    }
}

impl Command {
    pub(crate) fn translate(
        &self,
        module_name: &ModuleName,
        func_name: &FuncName,
        index: usize,
    ) -> Vec<Statement> {
        let mut gen = CodeGen::new(module_name, func_name, index);
        match self {
            Command::Add => gen.binary_op(Comp::DPlusA),
            Command::Sub => gen.binary_op(Comp::AMinusD),
            Command::Neg => gen.unary_op(Comp::MinusD),
            Command::Eq => gen.cond("eq", Jump::Eq),
            Command::Gt => gen.cond("gt", Jump::Gt),
            Command::Lt => gen.cond("lt", Jump::Lt),
            Command::And => gen.binary_op(Comp::DAndA),
            Command::Or => gen.binary_op(Comp::DOrA),
            Command::Not => gen.unary_op(Comp::NotD),
            Command::Push(Segment::Local, index) => gen.push_dynamic_segment(AsmLabel::LCL, *index),
            Command::Push(Segment::Argument, index) => {
                gen.push_dynamic_segment(AsmLabel::ARG, *index)
            }
            Command::Push(Segment::This, index) => gen.push_dynamic_segment(AsmLabel::THIS, *index),
            Command::Push(Segment::That, index) => gen.push_dynamic_segment(AsmLabel::THAT, *index),
            Command::Push(Segment::Pointer, index) => gen.push_fixed_segment(Imm::THIS, *index),
            Command::Push(Segment::Temp, index) => gen.push_fixed_segment(Imm::R5, *index),
            Command::Push(Segment::Static, index) => gen.push_static_segment(*index),
            Command::Push(Segment::Constant, imm) => gen.push_imm(*imm),
            Command::Pop(Segment::Local, index) => gen.pop_dynamic_segment(AsmLabel::LCL, *index),
            Command::Pop(Segment::Argument, index) => {
                gen.pop_dynamic_segment(AsmLabel::ARG, *index)
            }
            Command::Pop(Segment::This, index) => gen.pop_dynamic_segment(AsmLabel::THIS, *index),
            Command::Pop(Segment::That, index) => gen.pop_dynamic_segment(AsmLabel::THAT, *index),
            Command::Pop(Segment::Pointer, index) => gen.pop_fixed_segment(Imm::THIS, *index),
            Command::Pop(Segment::Temp, index) => gen.pop_fixed_segment(Imm::R5, *index),
            Command::Pop(Segment::Static, index) => gen.pop_static_segment(*index),
            Command::Pop(_, _) => unreachable!("{:?}", self),
            Command::Label(label) => gen.label(label),
            Command::Goto(label) => gen.goto(label),
            Command::IfGoto(label) => gen.if_goto(label),
            Command::Function(name, arity) => gen.function(name, *arity),
            Command::Call(name, arity) => gen.call(name, *arity),
            Command::Return => gen.return_(),
        }
        gen.into_statements()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_segment() {
        assert_eq!(Segment::from_str("argument").unwrap(), Segment::Argument);
        assert_eq!(Segment::from_str("local").unwrap(), Segment::Local);
        assert_eq!(Segment::from_str("static").unwrap(), Segment::Static);
        assert_eq!(Segment::from_str("constant").unwrap(), Segment::Constant);
        assert_eq!(Segment::from_str("this").unwrap(), Segment::This);
        assert_eq!(Segment::from_str("that").unwrap(), Segment::That);
        assert_eq!(Segment::from_str("pointer").unwrap(), Segment::Pointer);
        assert_eq!(Segment::from_str("temp").unwrap(), Segment::Temp);
        assert!(
            matches!(Segment::from_str("foo").unwrap_err(), ParseSegmentError::InvalidSegment(s) if s == "foo")
        );
    }

    #[test]
    fn parse_label() {
        assert_eq!(Ident::from_str("foo").unwrap(), Ident("foo".into()));
        assert_eq!(
            Ident::from_str(".:_foo12").unwrap(),
            Ident(".:_foo12".into())
        );
        assert!(
            matches!(Ident::from_str("1foo").unwrap_err(), ParseIdentError::InvalidIdent(s) if s == "1foo")
        );
    }

    #[test]
    fn parse_command() {
        let zero = Imm::try_new(0).unwrap();
        assert_eq!(Command::from_str("add").unwrap(), Command::Add);
        assert_eq!(Command::from_str("sub").unwrap(), Command::Sub);
        assert_eq!(Command::from_str("neg").unwrap(), Command::Neg);
        assert_eq!(Command::from_str("eq").unwrap(), Command::Eq);
        assert_eq!(Command::from_str("gt").unwrap(), Command::Gt);
        assert_eq!(Command::from_str("lt").unwrap(), Command::Lt);
        assert_eq!(Command::from_str("and").unwrap(), Command::And);
        assert_eq!(Command::from_str("or").unwrap(), Command::Or);
        assert_eq!(Command::from_str("not").unwrap(), Command::Not);
        assert_eq!(
            Command::from_str("push argument 0").unwrap(),
            Command::Push(Segment::Argument, zero)
        );
        assert_eq!(
            Command::from_str("push local 0").unwrap(),
            Command::Push(Segment::Local, zero)
        );
        assert_eq!(
            Command::from_str("push static 0").unwrap(),
            Command::Push(Segment::Static, zero)
        );
        assert_eq!(
            Command::from_str("push constant 0").unwrap(),
            Command::Push(Segment::Constant, zero)
        );
        assert_eq!(
            Command::from_str("push this 0").unwrap(),
            Command::Push(Segment::This, zero)
        );
        assert_eq!(
            Command::from_str("push that 0").unwrap(),
            Command::Push(Segment::That, zero)
        );
        assert_eq!(
            Command::from_str("push pointer 0").unwrap(),
            Command::Push(Segment::Pointer, zero)
        );
        assert_eq!(
            Command::from_str("push temp 0").unwrap(),
            Command::Push(Segment::Temp, zero)
        );
        assert_eq!(
            Command::from_str("pop argument 0").unwrap(),
            Command::Pop(Segment::Argument, zero)
        );
        assert_eq!(
            Command::from_str("pop local 0").unwrap(),
            Command::Pop(Segment::Local, zero)
        );
        assert_eq!(
            Command::from_str("pop static 0").unwrap(),
            Command::Pop(Segment::Static, zero)
        );
        assert_eq!(
            Command::from_str("pop this 0").unwrap(),
            Command::Pop(Segment::This, zero)
        );
        assert_eq!(
            Command::from_str("pop that 0").unwrap(),
            Command::Pop(Segment::That, zero)
        );
        assert_eq!(
            Command::from_str("pop pointer 0").unwrap(),
            Command::Pop(Segment::Pointer, zero)
        );
        assert_eq!(
            Command::from_str("pop temp 0").unwrap(),
            Command::Pop(Segment::Temp, zero)
        );
        assert_eq!(
            Command::from_str("label foo").unwrap(),
            Command::Label(Label("foo".into()))
        );

        assert!(matches!(
            Command::from_str("").unwrap_err(),
            ParseCommandError::Empty
        ));
        assert!(
            matches!(Command::from_str("foo").unwrap_err(), ParseCommandError::InvalidCommand(s) if s == "foo")
        );
        assert!(matches!(
            Command::from_str("add foo").unwrap_err(),
            ParseCommandError::TooManyArguments
        ));
        assert!(matches!(
            Command::from_str("push argument").unwrap_err(),
            ParseCommandError::TooFewArguments
        ));
        assert!(matches!(
            Command::from_str("push foo 0").unwrap_err(),
            ParseCommandError::InvalidSegment(ParseSegmentError::InvalidSegment(s)) if s == "foo"
        ));
        assert!(matches!(
            Command::from_str("push argument -10").unwrap_err(),
            ParseCommandError::InvalidIndex(e, s) if *e.kind() == std::num::IntErrorKind::InvalidDigit && s == "-10"
        ));
        assert!(matches!(
            Command::from_str("push argument 65535").unwrap_err(),
            ParseCommandError::TooLargeIndex(65535, 256)
        ));
        assert!(matches!(
            Command::from_str("pop constant 0").unwrap_err(),
            ParseCommandError::InvalidOperand(op, opr) if op == "pop" && opr == "constant"
        ));
    }
}
