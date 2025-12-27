use super::validation;
use crate::error_types::{LuaError, LuaResult};
use crate::lua_value::LuaTable;
/// Metatable and error handling functions for Lua
use crate::lua_value::LuaValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Create the setmetatable() function
/// Sets or replaces the metatable for a table
pub fn create_setmetatable() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        validation::require_args("setmetatable", &args, 2, Some(2))?;
        let table = validation::get_table("setmetatable", 0, &args[0])?;

        match &args[1] {
            LuaValue::Table(mt) => {
                // Convert the table's LuaValue-keyed data into String-keyed metamethods
                let mut metatable: HashMap<String, LuaValue> = HashMap::new();
                let mt_borrow = mt.borrow();

                for (key, value) in &mt_borrow.data {
                    if let LuaValue::String(key_str) = key {
                        metatable.insert(key_str.clone(), value.clone());
                    }
                }

                table.borrow_mut().metatable = Some(Box::new(metatable));
                Ok(args[0].clone())
            }
            LuaValue::Nil => {
                // Clear metatable
                table.borrow_mut().metatable = None;
                Ok(args[0].clone())
            }
            _ => Err(LuaError::type_error("table or nil", args[1].type_name(), "setmetatable")),
        }
    })
}

/// Create the getmetatable() function
/// Returns the metatable of a table
pub fn create_getmetatable() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        validation::require_args("getmetatable", &args, 1, Some(1))?;

        match &args[0] {
            LuaValue::Table(table) => {
                match &table.borrow().metatable {
                    Some(mt) => {
                        // Convert String-keyed metamethods back to LuaValue-keyed table
                        let mut table_data: HashMap<LuaValue, LuaValue> = HashMap::new();
                        for (key, value) in mt.iter() {
                            table_data.insert(LuaValue::String(key.clone()), value.clone());
                        }

                        Ok(LuaValue::Table(Rc::new(RefCell::new(LuaTable {
                            data: table_data,
                            metatable: None,
                        }))))
                    }
                    None => Ok(LuaValue::Nil),
                }
            }
            _ => Ok(LuaValue::Nil),
        }
    })
}

/// Create the pcall() function
/// Protected call - calls a function in protected mode, catching errors
pub fn create_pcall() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        validation::require_args("pcall", &args, 1, None)?;

        // For now, return a simple implementation
        // In full implementation, this would actually catch errors from function execution
        match &args[0] {
            LuaValue::Function(_) => {
                // Return success (true) and nil as placeholder
                Ok(LuaValue::Boolean(true))
            }
            _ => Err(LuaError::type_error("function", args[0].type_name(), "pcall")),
        }
    })
}

/// Create the xpcall() function
/// Extended protected call with custom error handler
pub fn create_xpcall() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        validation::require_args("xpcall", &args, 2, None)?;

        match (&args[0], &args[1]) {
            (LuaValue::Function(_), LuaValue::Function(_)) => {
                // Return success (true) and nil as placeholder
                Ok(LuaValue::Boolean(true))
            }
            (LuaValue::Function(_), _) => {
                Err(LuaError::type_error("function", args[1].type_name(), "xpcall"))
            }
            _ => Err(LuaError::type_error("function", args[0].type_name(), "xpcall")),
        }
    })
}

/// Create the error() function
/// Throws an error with a message
pub fn create_error() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        let message = if args.is_empty() {
            "".to_string()
        } else {
            match &args[0] {
                LuaValue::String(s) => s.clone(),
                v => v.to_string(),
            }
        };
        Err(LuaError::user(message, 1))
    })
}

/// Create the coroutine module table
pub fn create_coroutine_table() -> LuaValue {
    use crate::lua_value::LuaFunction;

    let mut coro_table = HashMap::new();

    // coroutine.create
    coro_table.insert(
        LuaValue::String("create".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(Rc::new(|_| {
            Err(LuaError::runtime("coroutine.create() requires executor context", "coroutine"))
        })))),
    );

    // coroutine.resume
    coro_table.insert(
        LuaValue::String("resume".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(Rc::new(|_| {
            Err(LuaError::runtime("coroutine.resume() requires executor context", "coroutine"))
        })))),
    );

    // coroutine.yield
    coro_table.insert(
        LuaValue::String("yield".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(Rc::new(|_| {
            Err(LuaError::runtime("coroutine.yield() requires executor context", "coroutine"))
        })))),
    );

    // coroutine.status
    coro_table.insert(
        LuaValue::String("status".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(Rc::new(|_| {
            Err(LuaError::runtime("coroutine.status() requires executor context", "coroutine"))
        })))),
    );

    LuaValue::Table(Rc::new(RefCell::new(LuaTable {
        data: coro_table,
        metatable: None,
    })))
}
