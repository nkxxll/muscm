use super::validation;
use crate::error_types::LuaResult;
use crate::lua_value::LuaTable;
/// Table library functions for Lua
use crate::lua_value::LuaValue;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Create table.insert() function
pub fn create_table_insert() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        validation::require_args("table.insert", &args, 2, None)?;
        let table_ref = validation::get_table("table.insert", 0, &args[0])?;

        let value = args.get(2).cloned().unwrap_or_else(|| args[1].clone());
        let index = if args.len() >= 3 {
            validation::get_integer("table.insert", 1, &args[1])?
        } else {
            -1 // Append at end
        };

        let mut table = table_ref.borrow_mut();

        // Find the length of the table (count numeric keys)
        let mut len = 0i64;
        for key in table.data.keys() {
            if let LuaValue::Number(n) = key {
                if n.fract() == 0.0 {
                    len = len.max(*n as i64);
                }
            }
        }

        let pos = if index < 0 { len + 1 } else { index };

        table.data.insert(LuaValue::Number(pos as f64), value);
        Ok(LuaValue::Nil)
    })
}

/// Create table.remove() function
pub fn create_table_remove() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        validation::require_args("table.remove", &args, 1, Some(2))?;
        let table_ref = validation::get_table("table.remove", 0, &args[0])?;

        let index = if args.len() >= 2 {
            validation::get_integer("table.remove", 1, &args[1])?
        } else {
            -1 // Remove from end
        };

        let mut table = table_ref.borrow_mut();

        // Find the length
        let mut len = 0i64;
        for key in table.data.keys() {
            if let LuaValue::Number(n) = key {
                if n.fract() == 0.0 {
                    len = len.max(*n as i64);
                }
            }
        }

        let pos = if index < 0 { len } else { index };

        if pos <= 0 || pos > len {
            return Ok(LuaValue::Nil);
        }

        let removed = table
            .data
            .remove(&LuaValue::Number(pos as f64))
            .unwrap_or(LuaValue::Nil);

        Ok(removed)
    })
}

/// Create the table table with all table functions
pub fn create_table_table() -> LuaValue {
    use crate::lua_value::LuaFunction;

    let mut table_table = HashMap::new();
    table_table.insert(
        LuaValue::String("insert".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_table_insert()))),
    );
    table_table.insert(
        LuaValue::String("remove".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_table_remove()))),
    );

    LuaValue::Table(Rc::new(RefCell::new(LuaTable {
        data: table_table,
        metatable: None,
    })))
}
