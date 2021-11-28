use crate::{Comp, Dest, Error, ErrorKind, InstC, Jump, Label, Statement, StatementWithLine};
use std::io::BufRead;

pub(crate) fn parse(mut reader: impl BufRead) -> Result<Vec<StatementWithLine>, Error> {
    let mut stmts = vec![];
    let mut line_buf = String::new();
    for line in 1.. {
        line_buf.clear();
        let res = reader
            .read_line(&mut line_buf)
            .map_err(|e| Error::new(line, e))?;
        if res == 0 {
            break;
        }

        if let Some(stmt) = parse_line(&line_buf).map_err(|e| Error::new(line, e))? {
            stmts.push(StatementWithLine::new(line, stmt));
        }
    }

    Ok(stmts)
}

fn parse_line(line: &str) -> Result<Option<Statement>, ErrorKind> {
    let line = trim_spaces_or_comment(line);
    if line.is_empty() {
        return Ok(None);
    }

    if let Some(stmt) = try_parse_label_statement(line)? {
        return Ok(Some(stmt));
    }

    if let Some(stmt) = try_parse_a_statement(line)? {
        return Ok(Some(stmt));
    }

    let stmt = parse_c_statement(line)?;
    Ok(Some(stmt))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Parts<'a> {
    dest: &'a str,
    comp: &'a str,
    jump: &'a str,
}

fn trim_spaces_or_comment(s: &str) -> &str {
    if let Some((pre, _post)) = s.split_once("//") {
        pre.trim()
    } else {
        s.trim()
    }
}

fn split_into_parts(s: &str) -> Parts {
    let mut dest = "";
    let comp;
    let mut jump = "";

    let mut s = s;

    if let Some((d, rest)) = s.split_once("=") {
        s = rest.trim();
        dest = d.trim();
    }

    if let Some((c, j)) = s.split_once(";") {
        comp = c.trim();
        jump = j.trim();
    } else {
        comp = s.trim();
    }

    Parts { dest, comp, jump }
}

fn read_matches_once(s: &str, p: impl FnMut(char) -> bool) -> Option<(&str, &str)> {
    let trimmed = s.trim_start_matches(p);
    if trimmed.len() != s.len() {
        Some((&s[..s.len() - trimmed.len()], trimmed))
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Token<'a> {
    Number(&'a str),
    Symbol(&'a str),
    Punct(char),
}

fn read_token(s: &str) -> Option<(Token, &str)> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }

    if let Some((num, rest)) = read_matches_once(s, |ch| ch.is_ascii_digit()) {
        return Some((Token::Number(num), rest.trim()));
    }
    if let Some((sym, rest)) = read_matches_once(s, |ch| {
        ch.is_ascii_alphanumeric() || ch == '_' || ch == '.' || ch == '$' || ch == ':'
    }) {
        return Some((Token::Symbol(sym), rest.trim()));
    }

    let mut cs = s.chars();
    let ch = cs.next();
    ch.map(|ch| (Token::Punct(ch), cs.as_str().trim()))
}

fn try_parse_label_statement(s: &str) -> Result<Option<Statement>, ErrorKind> {
    let s = s.trim();

    if let Some((Token::Punct('('), rest)) = read_token(s) {
        if let Some((Token::Symbol(label), ")")) = read_token(rest) {
            return Ok(Some(Statement::Label(Label::from(label))));
        }
        return Err(ErrorKind::InvalidLabelStatement(s.into()));
    }

    Ok(None)
}

fn try_parse_a_statement(s: &str) -> Result<Option<Statement>, ErrorKind> {
    let s = s.trim();

    if let Some((Token::Punct('@'), rest)) = read_token(s) {
        match read_token(rest) {
            Some((Token::Symbol(sym), "")) => {
                return Ok(Some(Statement::AtLabel(Label::from(sym))));
            }
            Some((Token::Number(num), "")) => {
                let value = num
                    .parse()
                    .map_err(|_| ErrorKind::TooLargeNumber(num.into()))?;
                return Ok(Some(Statement::A(value)));
            }
            _ => {}
        }
        return Err(ErrorKind::InvalidAStatement(s.into()));
    }

    Ok(None)
}

fn parse_dest(s: &str) -> Result<Dest, ErrorKind> {
    let mut m = false;
    let mut d = false;
    let mut a = false;
    for ch in s.trim().chars() {
        match ch {
            'M' => m = true,
            'D' => d = true,
            'A' => a = true,
            ch if ch.is_ascii_whitespace() => {}
            _ => return Err(ErrorKind::InvalidCStatementDest(s.into())),
        }
    }
    let dest = match (a, m, d) {
        (false, false, false) => Dest::Null,
        (false, true, false) => Dest::M,
        (false, false, true) => Dest::D,
        (false, true, true) => Dest::MD,
        (true, false, false) => Dest::A,
        (true, true, false) => Dest::AM,
        (true, false, true) => Dest::AD,
        (true, true, true) => Dest::AMD,
    };
    Ok(dest)
}

fn parse_comp(s: &str) -> Result<Comp, ErrorKind> {
    let comp = match read_token(s) {
        Some((Token::Number("0"), "")) => Comp::Zero,
        Some((Token::Number("1"), "")) => Comp::One,
        Some((Token::Symbol("D"), "")) => Comp::D,
        Some((Token::Symbol("A"), "")) => Comp::A,
        Some((Token::Symbol("M"), "")) => Comp::M,
        Some((Token::Punct('-'), "1")) => Comp::MinusOne,
        Some((Token::Punct('-'), "D")) => Comp::MinusD,
        Some((Token::Punct('-'), "A")) => Comp::MinusA,
        Some((Token::Punct('-'), "M")) => Comp::MinusM,
        Some((Token::Punct('!'), "D")) => Comp::NotD,
        Some((Token::Punct('!'), "A")) => Comp::NotA,
        Some((Token::Punct('!'), "M")) => Comp::NotM,
        Some((Token::Symbol("D"), rest)) => match read_token(rest) {
            Some((Token::Punct('+'), "1")) => Comp::DPlusOne,
            Some((Token::Punct('+'), "A")) => Comp::DPlusA,
            Some((Token::Punct('+'), "M")) => Comp::DPlusM,
            Some((Token::Punct('-'), "1")) => Comp::DMinusOne,
            Some((Token::Punct('-'), "A")) => Comp::DMinusA,
            Some((Token::Punct('-'), "M")) => Comp::DMinusM,
            Some((Token::Punct('&'), "A")) => Comp::DAndA,
            Some((Token::Punct('&'), "M")) => Comp::DAndM,
            Some((Token::Punct('|'), "A")) => Comp::DOrA,
            Some((Token::Punct('|'), "M")) => Comp::DOrM,
            _ => return Err(ErrorKind::InvalidCStatementComp(s.into())),
        },
        Some((Token::Symbol("A"), rest)) => match read_token(rest) {
            Some((Token::Punct('+'), "1")) => Comp::APlusOne,
            Some((Token::Punct('+'), "D")) => Comp::DPlusA,
            Some((Token::Punct('-'), "1")) => Comp::AMinusOne,
            Some((Token::Punct('-'), "D")) => Comp::AMinusD,
            Some((Token::Punct('&'), "D")) => Comp::DAndA,
            Some((Token::Punct('|'), "D")) => Comp::DOrA,
            _ => return Err(ErrorKind::InvalidCStatementComp(s.into())),
        },
        Some((Token::Symbol("M"), rest)) => match read_token(rest) {
            Some((Token::Punct('+'), "1")) => Comp::MPlusOne,
            Some((Token::Punct('+'), "D")) => Comp::DPlusM,
            Some((Token::Punct('-'), "1")) => Comp::MMinusOne,
            Some((Token::Punct('-'), "D")) => Comp::MMinusD,
            Some((Token::Punct('&'), "D")) => Comp::DAndM,
            Some((Token::Punct('|'), "D")) => Comp::DOrM,
            _ => return Err(ErrorKind::InvalidCStatementComp(s.into())),
        },
        _ => return Err(ErrorKind::InvalidCStatementComp(s.into())),
    };
    Ok(comp)
}

fn parse_jump(s: &str) -> Result<Jump, ErrorKind> {
    let jump = match s.trim() {
        "" => Jump::Null,
        "JGT" => Jump::Gt,
        "JEQ" => Jump::Eq,
        "JGE" => Jump::Ge,
        "JLT" => Jump::Lt,
        "JNE" => Jump::Ne,
        "JLE" => Jump::Le,
        "JMP" => Jump::Jmp,
        _ => return Err(ErrorKind::InvalidCStatementJump(s.into())),
    };
    Ok(jump)
}

fn parse_c_statement(s: &str) -> Result<Statement, ErrorKind> {
    let parts = split_into_parts(s.trim());
    let dest = parse_dest(parts.dest)?;
    let comp = parse_comp(parts.comp)?;
    let jump = parse_jump(parts.jump)?;
    Ok(Statement::C(InstC::new(dest, comp, jump)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trim_spaces_or_comment() {
        use super::trim_spaces_or_comment as t;
        assert_eq!(t(""), "");
        assert_eq!(t(" aaa \n"), "aaa");
        assert_eq!(t("foo bar // baz"), "foo bar");
        assert_eq!(t("foo bar baz //\n"), "foo bar baz");
    }

    #[test]
    fn split_into_parts() {
        use super::split_into_parts as s;
        fn p<'a>(dest: &'a str, comp: &'a str, jump: &'a str) -> Parts<'a> {
            Parts { dest, comp, jump }
        }

        assert_eq!(s(""), p("", "", ""));
        assert_eq!(s("="), p("", "", ""));
        assert_eq!(s(";"), p("", "", ""));
        assert_eq!(s("=;"), p("", "", ""));
        assert_eq!(s(" aa  a =bbb;ccc"), p("aa  a", "bbb", "ccc"));
        assert_eq!(s(" !!+++!???"), p("", "!!+++!???", ""));
    }

    #[test]
    fn read_matches_once() {
        use super::read_matches_once as r;
        assert_eq!(r("", |c| c == ' '), None);
        assert_eq!(r("aaa", |c| c == 'b'), None);
        assert_eq!(r("aaa", |c| c == 'a'), Some(("aaa", "")));
        assert_eq!(r("abaab", |c| c == 'a'), Some(("a", "baab")));
    }

    #[test]
    fn read_token() {
        use super::read_token as r;
        assert_eq!(r(""), None);
        assert_eq!(r(" "), None);
        assert_eq!(r("123"), Some((Token::Number("123"), "")));
        assert_eq!(r("foo"), Some((Token::Symbol("foo"), "")));
        assert_eq!(r("foo bar"), Some((Token::Symbol("foo"), "bar")));
        assert_eq!(r("foo bar baz"), Some((Token::Symbol("foo"), "bar baz")));
        assert_eq!(
            r("f:o$o_1.23 456"),
            Some((Token::Symbol("f:o$o_1.23"), "456"))
        );
        assert_eq!(r("123foo"), Some((Token::Number("123"), "foo")));
        assert_eq!(r("! ! !"), Some((Token::Punct('!'), "! !")));
    }

    #[test]
    fn try_parse_label_statement() {
        use super::try_parse_label_statement as t;
        assert_eq!(t("").unwrap(), None);
        assert_eq!(t("(foo)").unwrap(), Some(Statement::Label("foo".into())));
        assert_eq!(t("(f123)").unwrap(), Some(Statement::Label("f123".into())));
        assert_eq!(
            t("  ( f123  )").unwrap(),
            Some(Statement::Label("f123".into()))
        );

        assert!(matches!(
            t("(123)"),
            Err(ErrorKind::InvalidLabelStatement(s)) if s == "(123)"
        ));
        assert!(matches!(
            t("( foo"),
            Err(ErrorKind::InvalidLabelStatement(s)) if s == "( foo"
        ));
    }

    #[test]
    fn try_parse_a_statement() {
        use super::try_parse_a_statement as t;
        assert_eq!(t("").unwrap(), None);
        assert_eq!(t("@foo").unwrap(), Some(Statement::AtLabel("foo".into())));
        assert_eq!(
            t("@  fo_:.$o  ").unwrap(),
            Some(Statement::AtLabel("fo_:.$o".into()))
        );
        assert_eq!(t("@1234").unwrap(), Some(Statement::A(1234)));
        assert_eq!(t("@65535").unwrap(), Some(Statement::A(65535)));
        assert!(matches!(t("@65536"), Err(ErrorKind::TooLargeNumber(s)) if s == "65536"));
        assert!(matches!(t("@0x123"), Err(ErrorKind::InvalidAStatement(s)) if s == "@0x123"));
        assert!(matches!(t("@foo  bar"), Err(ErrorKind::InvalidAStatement(s)) if s == "@foo  bar"));
    }

    #[test]
    fn parse_dest() {
        use super::parse_dest as p;
        assert_eq!(p("").unwrap(), Dest::Null);
        assert_eq!(p("M").unwrap(), Dest::M);
        assert_eq!(p("D").unwrap(), Dest::D);
        assert_eq!(p("A").unwrap(), Dest::A);
        assert_eq!(p("MD").unwrap(), Dest::MD);
        assert_eq!(p("AM").unwrap(), Dest::AM);
        assert_eq!(p("AD").unwrap(), Dest::AD);
        assert_eq!(p("AMD").unwrap(), Dest::AMD);

        assert_eq!(p("M A D").unwrap(), Dest::AMD);
        assert_eq!(p("M A D A M").unwrap(), Dest::AMD);
        assert!(matches!(p("ABC"), Err(ErrorKind::InvalidCStatementDest(s)) if s == "ABC"));
    }

    #[test]
    fn parse_comp() {
        use super::parse_comp as p;
        assert_eq!(p("0").unwrap(), Comp::Zero);
        assert_eq!(p("1").unwrap(), Comp::One);
        assert_eq!(p("-1").unwrap(), Comp::MinusOne);
        assert_eq!(p("D").unwrap(), Comp::D);
        assert_eq!(p("A").unwrap(), Comp::A);
        assert_eq!(p("!D").unwrap(), Comp::NotD);
        assert_eq!(p("!A").unwrap(), Comp::NotA);
        assert_eq!(p("-D").unwrap(), Comp::MinusD);
        assert_eq!(p("-A").unwrap(), Comp::MinusA);
        assert_eq!(p("D+1").unwrap(), Comp::DPlusOne);
        assert_eq!(p("A+1").unwrap(), Comp::APlusOne);
        assert_eq!(p("D-1").unwrap(), Comp::DMinusOne);
        assert_eq!(p("A-1").unwrap(), Comp::AMinusOne);
        assert_eq!(p("D+A").unwrap(), Comp::DPlusA);
        assert_eq!(p("D-A").unwrap(), Comp::DMinusA);
        assert_eq!(p("A-D").unwrap(), Comp::AMinusD);
        assert_eq!(p("D&A").unwrap(), Comp::DAndA);
        assert_eq!(p("D|A").unwrap(), Comp::DOrA);
        assert_eq!(p("M").unwrap(), Comp::M);
        assert_eq!(p("!M").unwrap(), Comp::NotM);
        assert_eq!(p("-M").unwrap(), Comp::MinusM);
        assert_eq!(p("M+1").unwrap(), Comp::MPlusOne);
        assert_eq!(p("M-1").unwrap(), Comp::MMinusOne);
        assert_eq!(p("D+M").unwrap(), Comp::DPlusM);
        assert_eq!(p("D-M").unwrap(), Comp::DMinusM);
        assert_eq!(p("M-D").unwrap(), Comp::MMinusD);
        assert_eq!(p("D&M").unwrap(), Comp::DAndM);
        assert_eq!(p("D|M").unwrap(), Comp::DOrM);

        assert_eq!(p(" ! D  ").unwrap(), Comp::NotD);
        assert_eq!(p(" D & M  ").unwrap(), Comp::DAndM);

        assert!(matches!(p("D+A+M"), Err(ErrorKind::InvalidCStatementComp(s)) if s == "D+A+M"));
        assert!(matches!(p("  "), Err(ErrorKind::InvalidCStatementComp(s)) if s == "  "));
    }

    #[test]
    fn parse_jump() {
        use super::parse_jump as p;
        assert_eq!(p("").unwrap(), Jump::Null);
        assert_eq!(p("JGT").unwrap(), Jump::Gt);
        assert_eq!(p("JEQ").unwrap(), Jump::Eq);
        assert_eq!(p("JGE").unwrap(), Jump::Ge);
        assert_eq!(p("JLT").unwrap(), Jump::Lt);
        assert_eq!(p("JNE").unwrap(), Jump::Ne);
        assert_eq!(p("JLE").unwrap(), Jump::Le);
        assert_eq!(p("JMP").unwrap(), Jump::Jmp);

        assert_eq!(p("  JLE  ").unwrap(), Jump::Le);

        assert!(matches!(p("JGTJEQ"), Err(ErrorKind::InvalidCStatementJump(s)) if s == "JGTJEQ"));
        assert!(matches!(p("J GT"), Err(ErrorKind::InvalidCStatementJump(s)) if s == "J GT"));
    }

    #[test]
    fn parse_c_statement() {
        use super::parse_c_statement as p;
        fn s(dest: Dest, comp: Comp, jump: Jump) -> Statement {
            Statement::C(InstC::new(dest, comp, jump))
        }

        assert_eq!(p("0").unwrap(), s(Dest::Null, Comp::Zero, Jump::Null));
        assert_eq!(p("M = 0; JLT").unwrap(), s(Dest::M, Comp::Zero, Jump::Lt));

        assert!(matches!(p(""), Err(ErrorKind::InvalidCStatementComp(s)) if s.is_empty()));
        assert!(matches!(p("X"), Err(ErrorKind::InvalidCStatementComp(s)) if s == "X"));
    }

    #[test]
    fn parse_line() {
        use super::parse_line as p;
        assert_eq!(p("").unwrap(), None);
        assert_eq!(p("// foo").unwrap(), None);
        assert_eq!(
            p("M=M+1;JEQ // comment").unwrap(),
            Some(Statement::C(InstC::new(Dest::M, Comp::MPlusOne, Jump::Eq)))
        );
    }
}
