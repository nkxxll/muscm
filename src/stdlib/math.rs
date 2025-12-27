/// Math library functions for Lua
use crate::lua_value::LuaValue;
use crate::lua_value::LuaTable;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use super::validation;

/// Create math.abs() function
pub fn create_math_abs() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        validation::require_args("math.abs", &args, 1, Some(1))?;
        let n = validation::get_number("math.abs", 0, &args[0])?;
        Ok(LuaValue::Number(n.abs()))
    })
}

/// Create math.floor() function
pub fn create_math_floor() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        validation::require_args("math.floor", &args, 1, Some(1))?;
        let n = validation::get_number("math.floor", 0, &args[0])?;
        Ok(LuaValue::Number(n.floor()))
    })
}

/// Create math.ceil() function
pub fn create_math_ceil() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        validation::require_args("math.ceil", &args, 1, Some(1))?;
        let n = validation::get_number("math.ceil", 0, &args[0])?;
        Ok(LuaValue::Number(n.ceil()))
    })
}

/// Create math.min() function
pub fn create_math_min() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        validation::require_args("math.min", &args, 1, None)?;
        let mut min = validation::get_number("math.min", 0, &args[0])?;
        
        for (i, arg) in args[1..].iter().enumerate() {
            let n = validation::get_number("math.min", i + 1, arg)?;
            min = min.min(n);
        }
        
        Ok(LuaValue::Number(min))
    })
}

/// Create math.max() function
pub fn create_math_max() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        validation::require_args("math.max", &args, 1, None)?;
        let mut max = validation::get_number("math.max", 0, &args[0])?;
        
        for (i, arg) in args[1..].iter().enumerate() {
            let n = validation::get_number("math.max", i + 1, arg)?;
            max = max.max(n);
        }
        
        Ok(LuaValue::Number(max))
    })
}

/// Create math.random() function
pub fn create_math_random() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // Simple pseudo-random using system time
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        
        let rand = ((seed.wrapping_mul(1103515245).wrapping_add(12345)) / 65536) % 32768;
        let normalized = (rand as f64) / 32768.0;
        
        match args.len() {
            0 => Ok(LuaValue::Number(normalized)),
            1 => {
                match &args[0] {
                    LuaValue::Number(n) => {
                        let max = *n as i64;
                        Ok(LuaValue::Number(((rand % (max as u64)) + 1) as f64))
                    }
                    _ => Err("math.random() expects a number".to_string()),
                }
            }
            2 => {
                match (&args[0], &args[1]) {
                    (LuaValue::Number(a), LuaValue::Number(b)) => {
                        let min = (*a as i64).min(*b as i64);
                        let max = (*a as i64).max(*b as i64);
                        let range = (max - min + 1) as u64;
                        Ok(LuaValue::Number(((rand % range) + min as u64) as f64))
                    }
                    _ => Err("math.random() expects numbers".to_string()),
                }
            }
            _ => Err("math.random() takes 0-2 arguments".to_string()),
        }
    })
}

/// Create the math table with all math functions
pub fn create_math_table() -> LuaValue {
    use crate::lua_value::LuaFunction;
    
    let mut math_table = HashMap::new();
    math_table.insert(
        LuaValue::String("abs".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_math_abs()))),
    );
    math_table.insert(
        LuaValue::String("floor".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_math_floor()))),
    );
    math_table.insert(
        LuaValue::String("ceil".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_math_ceil()))),
    );
    math_table.insert(
        LuaValue::String("min".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_math_min()))),
    );
    math_table.insert(
        LuaValue::String("max".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_math_max()))),
    );
    math_table.insert(
        LuaValue::String("random".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_math_random()))),
    );
    
    LuaValue::Table(Rc::new(RefCell::new(LuaTable {
        data: math_table,
        metatable: None,
    })))
}
