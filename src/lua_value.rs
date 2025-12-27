use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

/// Runtime value representation for Lua
#[derive(Clone)]
pub enum LuaValue {
    /// Nil/null value
    Nil,
    /// Boolean values
    Boolean(bool),
    /// Numeric values (Lua uses only f64)
    Number(f64),
    /// String values
    String(String),
    /// Table (hash map with metatable support)
    Table(Rc<RefCell<LuaTable>>),
    /// Function (built-in or user-defined)
    Function(Rc<LuaFunction>),
    /// User data (opaque data for extensions)
    UserData(Rc<RefCell<Box<dyn std::any::Any>>>),
}

/// A Lua table with potential metatable
#[derive(Debug)]
pub struct LuaTable {
    pub data: HashMap<LuaValue, LuaValue>,
    pub metatable: Option<Box<HashMap<String, LuaValue>>>,
}

/// A Lua function (closure with captured variables)
#[derive(Clone)]
pub enum LuaFunction {
    /// Built-in function with a closure
    Builtin(Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>>),
    /// User-defined function with AST and captured variables
    User {
        /// Function parameters
        params: Vec<String>,
        /// Whether function accepts varargs (...)
        varargs: bool,
        /// Function body (AST)
        body: Box<crate::lua_parser::Block>,
        /// Variables captured from defining scope (shared reference for proper closure semantics)
        captured: Rc<RefCell<HashMap<String, LuaValue>>>,
    },
}

impl fmt::Debug for LuaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LuaValue::Nil => write!(f, "nil"),
            LuaValue::Boolean(b) => write!(f, "{}", b),
            LuaValue::Number(n) => write!(f, "{}", n),
            LuaValue::String(s) => write!(f, "\"{}\"", s),
            LuaValue::Table(_) => write!(f, "<table>"),
            LuaValue::Function(_) => write!(f, "<function>"),
            LuaValue::UserData(_) => write!(f, "<userdata>"),
        }
    }
}

impl fmt::Display for LuaValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LuaValue::Nil => write!(f, "nil"),
            LuaValue::Boolean(b) => write!(f, "{}", b),
            LuaValue::Number(n) => {
                if n.fract() == 0.0 && !n.is_infinite() {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            LuaValue::String(s) => write!(f, "{}", s),
            LuaValue::Table(_) => write!(f, "table"),
            LuaValue::Function(_) => write!(f, "function"),
            LuaValue::UserData(_) => write!(f, "userdata"),
        }
    }
}

impl PartialEq for LuaValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (LuaValue::Nil, LuaValue::Nil) => true,
            (LuaValue::Boolean(a), LuaValue::Boolean(b)) => a == b,
            (LuaValue::Number(a), LuaValue::Number(b)) => a == b,
            (LuaValue::String(a), LuaValue::String(b)) => a == b,
            (LuaValue::Table(a), LuaValue::Table(b)) => Rc::ptr_eq(a, b),
            (LuaValue::Function(_), LuaValue::Function(_)) => false, // Functions compared by identity
            (LuaValue::UserData(a), LuaValue::UserData(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl Eq for LuaValue {}

impl std::hash::Hash for LuaValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            LuaValue::Nil => 0.hash(state),
            LuaValue::Boolean(b) => {
                1.hash(state);
                b.hash(state);
            }
            LuaValue::Number(n) => {
                2.hash(state);
                n.to_bits().hash(state);
            }
            LuaValue::String(s) => {
                3.hash(state);
                s.hash(state);
            }
            LuaValue::Table(t) => {
                4.hash(state);
                (t.as_ptr() as *const () as usize).hash(state);
            }
            LuaValue::Function(f) => {
                5.hash(state);
                (f.as_ref() as *const _ as usize).hash(state);
            }
            LuaValue::UserData(u) => {
                6.hash(state);
                (u.as_ptr() as *const () as usize).hash(state);
            }
        }
    }
}

impl LuaValue {
    /// Check if a value is truthy (false and nil are falsy, everything else is truthy)
    pub fn is_truthy(&self) -> bool {
        !matches!(self, LuaValue::Nil | LuaValue::Boolean(false))
    }

    /// Convert value to number (Lua type coercion)
    pub fn to_number(&self) -> Result<f64, String> {
        match self {
            LuaValue::Number(n) => Ok(*n),
            LuaValue::String(s) => {
                s.trim().parse::<f64>()
                    .map_err(|_| format!("Cannot convert string '{}' to number", s))
            }
            LuaValue::Boolean(true) => Ok(1.0),
            LuaValue::Boolean(false) => Ok(0.0),
            _ => Err(format!("Cannot convert {:?} to number", self)),
        }
    }

    /// Convert value to string
    pub fn to_string_value(&self) -> String {
        match self {
            LuaValue::String(s) => s.clone(),
            _ => self.to_string(),
        }
    }

    /// Get the type name of the value
    pub fn type_name(&self) -> &'static str {
        match self {
            LuaValue::Nil => "nil",
            LuaValue::Boolean(_) => "boolean",
            LuaValue::Number(_) => "number",
            LuaValue::String(_) => "string",
            LuaValue::Table(_) => "table",
            LuaValue::Function(_) => "function",
            LuaValue::UserData(_) => "userdata",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truthy_values() {
        assert!(LuaValue::Number(1.0).is_truthy());
        assert!(LuaValue::Number(0.0).is_truthy());
        assert!(LuaValue::String("hello".to_string()).is_truthy());
        assert!(LuaValue::Boolean(true).is_truthy());
        assert!(!LuaValue::Boolean(false).is_truthy());
        assert!(!LuaValue::Nil.is_truthy());
    }

    #[test]
    fn test_to_number() {
        assert_eq!(LuaValue::Number(42.0).to_number(), Ok(42.0));
        assert_eq!(LuaValue::String("123".to_string()).to_number(), Ok(123.0));
        assert_eq!(LuaValue::Boolean(true).to_number(), Ok(1.0));
        assert_eq!(LuaValue::Boolean(false).to_number(), Ok(0.0));
        assert!(LuaValue::String("abc".to_string()).to_number().is_err());
    }

    #[test]
    fn test_type_names() {
        assert_eq!(LuaValue::Nil.type_name(), "nil");
        assert_eq!(LuaValue::Boolean(true).type_name(), "boolean");
        assert_eq!(LuaValue::Number(42.0).type_name(), "number");
        assert_eq!(LuaValue::String("hello".to_string()).type_name(), "string");
    }
}
