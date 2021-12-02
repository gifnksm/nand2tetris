use std::fmt;

#[derive(Debug)]
pub(crate) struct XmlEscape<'a>(pub &'a str);

impl fmt::Display for XmlEscape<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = self.0;
        while !s.is_empty() {
            if let Some(idx) = s.find(['&', '<', '>', '"', '\''].as_ref()) {
                let (l, r) = s.split_at(idx);
                write!(f, "{}", l)?;
                s = r;
                let mut cs = s.chars();
                match cs.next() {
                    Some('&') => write!(f, "&amp;")?,
                    Some('<') => write!(f, "&lt;")?,
                    Some('>') => write!(f, "&gt;")?,
                    Some('"') => write!(f, "&quot;")?,
                    Some('\'') => write!(f, "&apos;")?,
                    _ => unreachable!(),
                }
                s = cs.as_str();
                continue;
            }
            write!(f, "{}", s)?;
            break;
        }
        Ok(())
    }
}
