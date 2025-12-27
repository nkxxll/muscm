use super::validation;
use crate::error_types::LuaResult;
use crate::lua_value::LuaTable;
/// String library functions for Lua
use crate::lua_value::LuaValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Create string.len() function
pub fn create_string_len() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        validation::require_args("string.len", &args, 1, Some(1))?;
        let s = validation::get_string("string.len", 0, &args[0])?;
        Ok(LuaValue::Number(s.len() as f64))
    })
}

/// Create string.sub() function
pub fn create_string_sub() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        validation::require_args("string.sub", &args, 2, None)?;

        let s = validation::get_string("string.sub", 0, &args[0])?;
        let start_lua = validation::get_integer("string.sub", 1, &args[1])? as i32;

        let end_lua = if args.len() >= 3 {
            validation::get_integer("string.sub", 2, &args[2])? as i32
        } else {
            s.len() as i32
        };

        let len = s.len() as i32;

        // Convert Lua 1-based indices to 0-based Rust indices
        let i = if start_lua < 0 {
            (len + start_lua).max(0) as usize
        } else {
            ((start_lua - 1).min(len)).max(0) as usize
        };

        let j = if end_lua < 0 {
            (len + end_lua).max(-1) as usize
        } else {
            (end_lua.min(len)) as usize
        };

        // j is the last character index (1-based), so we need j as the exclusive end
        let end = if j < i { i } else { j };

        if i > s.len() {
            return Ok(LuaValue::String(String::new()));
        }

        Ok(LuaValue::String(s[i..end.min(s.len())].to_string()))
    })
}

/// Create string.upper() function
pub fn create_string_upper() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        validation::require_args("string.upper", &args, 1, Some(1))?;
        let s = validation::get_string("string.upper", 0, &args[0])?;
        Ok(LuaValue::String(s.to_uppercase()))
    })
}

/// Create string.lower() function
pub fn create_string_lower() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        validation::require_args("string.lower", &args, 1, Some(1))?;
        let s = validation::get_string("string.lower", 0, &args[0])?;
        Ok(LuaValue::String(s.to_lowercase()))
    })
}

/// Create the string table with all string functions
pub fn create_string_table() -> LuaValue {
    use crate::lua_value::LuaFunction;

    let mut string_table = HashMap::new();
    string_table.insert(
        LuaValue::String("len".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_string_len()))),
    );
    string_table.insert(
        LuaValue::String("sub".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_string_sub()))),
    );
    string_table.insert(
        LuaValue::String("upper".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_string_upper()))),
    );
    string_table.insert(
        LuaValue::String("lower".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_string_lower()))),
    );

    LuaValue::Table(Rc::new(RefCell::new(LuaTable {
        data: string_table,
        metatable: None,
    })))
}
