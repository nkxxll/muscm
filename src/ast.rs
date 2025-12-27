use std::fmt;

pub type NodeId = usize;

#[derive(Debug)]
pub struct Arena {
    nodes: Vec<SExpr>,
}

impl Arena {
    pub fn new() -> Self {
        Arena { nodes: Vec::new() }
    }

    pub fn alloc(&mut self, expr: SExpr) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(expr);
        id
    }

    pub fn get(&self, id: NodeId) -> Option<&SExpr> {
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut SExpr> {
        self.nodes.get_mut(id)
    }
}

impl Default for Arena {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SExpr {
    Atom(String),
    Number(f64),
    String(String),
    Bool(bool),
    Char(char),
    List(Vec<NodeId>),
    Quote(NodeId),
    QuasiQuote(NodeId),
    Unquote(NodeId),
    UnquoteSplicing(NodeId),
    Vector(Vec<NodeId>),
}

impl SExpr {
    pub fn display_with_arena(&self, arena: &Arena, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            SExpr::List(ids) => {
                write!(f, "(")?;
                for (i, id) in ids.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    if let Some(item) = arena.get(*id) {
                        item.display_with_arena(arena, f)?;
                    } else {
                        write!(f, "#<invalid>")?;
                    }
                }
                write!(f, ")")
            }
            SExpr::Quote(id) => {
                write!(f, "'")?;
                if let Some(node) = arena.get(*id) {
                    node.display_with_arena(arena, f)
                } else {
                    write!(f, "#<invalid>")
                }
            }
            SExpr::QuasiQuote(id) => {
                write!(f, "`")?;
                if let Some(node) = arena.get(*id) {
                    node.display_with_arena(arena, f)
                } else {
                    write!(f, "#<invalid>")
                }
            }
            SExpr::Unquote(id) => {
                write!(f, ",")?;
                if let Some(node) = arena.get(*id) {
                    node.display_with_arena(arena, f)
                } else {
                    write!(f, "#<invalid>")
                }
            }
            SExpr::UnquoteSplicing(id) => {
                write!(f, ",@")?;
                if let Some(node) = arena.get(*id) {
                    node.display_with_arena(arena, f)
                } else {
                    write!(f, "#<invalid>")
                }
            }
            SExpr::Vector(ids) => {
                write!(f, "#(")?;
                for (i, id) in ids.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    if let Some(item) = arena.get(*id) {
                        item.display_with_arena(arena, f)?;
                    } else {
                        write!(f, "#<invalid>")?;
                    }
                }
                write!(f, ")")
            }
        }
    }
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
            SExpr::List(_) => {
                write!(f, "#<node-list>")
            }
            SExpr::Quote(_) | SExpr::QuasiQuote(_) | SExpr::Unquote(_) | SExpr::UnquoteSplicing(_) => {
                write!(f, "#<node-ref>")
            }
            SExpr::Vector(_) => {
                write!(f, "#<node-vector>")
            }
        }
    }
}
