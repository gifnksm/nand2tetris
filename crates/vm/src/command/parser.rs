use super::{Command, FuncName, Ident, Label, Segment};
use std::{num::ParseIntError, str::FromStr};
use thiserror::Error;

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
            SegmentIndex(fn(Segment, u16) -> Command),
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

impl FromStr for FuncName {
    type Err = ParseIdentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Ident::from_str(s)?.into())
    }
}

impl FromStr for Label {
    type Err = ParseIdentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Ident::from_str(s)?.into())
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
            Command::Push(Segment::Argument, 0)
        );
        assert_eq!(
            Command::from_str("push local 0").unwrap(),
            Command::Push(Segment::Local, 0)
        );
        assert_eq!(
            Command::from_str("push static 0").unwrap(),
            Command::Push(Segment::Static, 0)
        );
        assert_eq!(
            Command::from_str("push constant 0").unwrap(),
            Command::Push(Segment::Constant, 0)
        );
        assert_eq!(
            Command::from_str("push this 0").unwrap(),
            Command::Push(Segment::This, 0)
        );
        assert_eq!(
            Command::from_str("push that 0").unwrap(),
            Command::Push(Segment::That, 0)
        );
        assert_eq!(
            Command::from_str("push pointer 0").unwrap(),
            Command::Push(Segment::Pointer, 0)
        );
        assert_eq!(
            Command::from_str("push temp 0").unwrap(),
            Command::Push(Segment::Temp, 0)
        );
        assert_eq!(
            Command::from_str("pop argument 0").unwrap(),
            Command::Pop(Segment::Argument, 0)
        );
        assert_eq!(
            Command::from_str("pop local 0").unwrap(),
            Command::Pop(Segment::Local, 0)
        );
        assert_eq!(
            Command::from_str("pop static 0").unwrap(),
            Command::Pop(Segment::Static, 0)
        );
        assert_eq!(
            Command::from_str("pop this 0").unwrap(),
            Command::Pop(Segment::This, 0)
        );
        assert_eq!(
            Command::from_str("pop that 0").unwrap(),
            Command::Pop(Segment::That, 0)
        );
        assert_eq!(
            Command::from_str("pop pointer 0").unwrap(),
            Command::Pop(Segment::Pointer, 0)
        );
        assert_eq!(
            Command::from_str("pop temp 0").unwrap(),
            Command::Pop(Segment::Temp, 0)
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
