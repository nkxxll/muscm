use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    Atom(String),
    Number(f64),
    String(String),
    Bool(bool),
    Char(char),
    List(Vec<SExpr>),
    Quote(Box<SExpr>),
    QuasiQuote(Box<SExpr>),
    Unquote(Box<SExpr>),
    UnquoteSplicing(Box<SExpr>),
    Vector(Vec<SExpr>),
}

impl fmt::Display for SExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SExpr::Atom(s) => write!(f, "{}", s),
            SExpr::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            SExpr::String(s) => write!(f, "\"{}\"", s),
            SExpr::Bool(b) => write!(f, "#{}", if *b { 't' } else { 'f' }),
            SExpr::Char(c) => write!(f, "#\\{}", c),
            SExpr::List(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            SExpr::Quote(e) => write!(f, "'{}", e),
            SExpr::QuasiQuote(e) => write!(f, "`{}", e),
            SExpr::Unquote(e) => write!(f, ",{}", e),
            SExpr::UnquoteSplicing(e) => write!(f, ",@{}", e),
            SExpr::Vector(items) => {
                write!(f, "#(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
        }
    }
}
