/// Iterator functions for Lua
use crate::lua_value::LuaValue;
use crate::lua_value::LuaFunction;
use std::rc::Rc;
use super::validation;

/// Create pairs() iterator function
pub fn create_pairs() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|_args| {
        // Return a dummy function for now - full iterator support in future
        Ok(LuaValue::Function(Rc::new(
            LuaFunction::Builtin(
                Rc::new(|_| Ok(LuaValue::Nil))
            )
        )))
    })
}

/// Create ipairs() iterator function
pub fn create_ipairs() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|_args| {
        // Return a dummy function for now - full iterator support in future
        Ok(LuaValue::Function(Rc::new(
            LuaFunction::Builtin(
                Rc::new(|_| Ok(LuaValue::Nil))
            )
        )))
    })
}

/// Create next() function for generic iteration
pub fn create_next() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        validation::require_args("next", &args, 1, Some(2))?;
        let table_ref = validation::get_table("next", 0, &args[0])?;
        
        let table = table_ref.borrow();
        
        if args.len() == 1 {
            // Get first key
            if let Some(key) = table.data.keys().next() {
                return Ok(key.clone());
            }
        } else {
            // Get next key after given key
            let mut found = false;
            for key in table.data.keys() {
                if found {
                    return Ok(key.clone());
                }
                if key == &args[1] {
                    found = true;
                }
            }
        }
        
        Ok(LuaValue::Nil)
    })
}
