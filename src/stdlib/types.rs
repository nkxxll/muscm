use super::validation;
/// Type conversion and type-related functions for Lua
use crate::lua_value::LuaValue;
use std::rc::Rc;

/// Create the type() function that returns the type name of a value
pub fn create_type() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        validation::require_args("type", &args, 1, Some(1))?;
        Ok(LuaValue::String(args[0].type_name().to_string()))
    })
}

/// Create the tonumber() function that converts strings to numbers
pub fn create_tonumber() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        if args.is_empty() {
            return Ok(LuaValue::Nil);
        }

        match &args[0] {
            LuaValue::Number(n) => Ok(LuaValue::Number(*n)),
            LuaValue::String(s) => match s.trim().parse::<f64>() {
                Ok(n) => Ok(LuaValue::Number(n)),
                Err(_) => Ok(LuaValue::Nil),
            },
            LuaValue::Boolean(b) => Ok(LuaValue::Number(if *b { 1.0 } else { 0.0 })),
            _ => Ok(LuaValue::Nil),
        }
    })
}

/// Create the tostring() function that converts values to strings
pub fn create_tostring() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        if args.is_empty() {
            return Ok(LuaValue::String("nil".to_string()));
        }

        match &args[0] {
            LuaValue::String(s) => Ok(LuaValue::String(s.clone())),
            LuaValue::Nil => Ok(LuaValue::String("nil".to_string())),
            LuaValue::Boolean(b) => Ok(LuaValue::String(b.to_string())),
            LuaValue::Number(n) => {
                let s = if n.fract() == 0.0 && !n.is_infinite() {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                };
                Ok(LuaValue::String(s))
            }
            LuaValue::Table(_) => Ok(LuaValue::String("table".to_string())),
            LuaValue::Function(_) => Ok(LuaValue::String("function".to_string())),
            LuaValue::UserData(_) => Ok(LuaValue::String("userdata".to_string())),
        }
    })
}
