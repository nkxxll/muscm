/// Validation helpers for stdlib functions
/// 
/// Provides consistent argument validation and type checking
/// to eliminate ~150 lines of duplicated boilerplate across stdlib.

use crate::lua_value::{LuaValue, LuaTable};
use std::rc::Rc;
use std::cell::RefCell;

/// Validate argument count with optional bounds
/// 
/// # Arguments
/// * `name` - Function name for error messages
/// * `args` - Arguments array
/// * `min` - Minimum required arguments
/// * `max` - Maximum allowed arguments (None for unlimited)
pub fn require_args(name: &str, args: &[LuaValue], min: usize, max: Option<usize>) -> Result<(), String> {
    if args.len() < min {
        return Err(format!(
            "{}() expects at least {} argument{}, got {}",
            name,
            min,
            if min == 1 { "" } else { "s" },
            args.len()
        ));
    }
    
    if let Some(max_args) = max {
        if args.len() > max_args {
            return Err(format!(
                "{}() expects at most {} argument{}, got {}",
                name,
                max_args,
                if max_args == 1 { "" } else { "s" },
                args.len()
            ));
        }
    }
    
    Ok(())
}

/// Require specific type for argument at given index
/// 
/// # Arguments
/// * `name` - Function name for error messages
/// * `index` - Argument position (0-based)
/// * `arg` - The argument to validate
/// * `expected` - Expected type name
pub fn require_type(name: &str, index: usize, arg: &LuaValue, expected: &str) -> Result<(), String> {
    let got = arg.type_name();
    if got != expected {
        Err(format!(
            "{}() expects {} as argument {}, got {}",
            name,
            expected,
            index + 1,
            got
        ))
    } else {
        Ok(())
    }
}

/// Extract number with type checking
/// 
/// # Arguments
/// * `name` - Function name for error messages
/// * `index` - Argument position (0-based)
/// * `arg` - The argument to extract
pub fn get_number(name: &str, index: usize, arg: &LuaValue) -> Result<f64, String> {
    match arg {
        LuaValue::Number(n) => Ok(*n),
        _ => Err(format!(
            "{}() expects number as argument {}, got {}",
            name,
            index + 1,
            arg.type_name()
        )),
    }
}

/// Extract string with type checking
/// 
/// # Arguments
/// * `name` - Function name for error messages
/// * `index` - Argument position (0-based)
/// * `arg` - The argument to extract
pub fn get_string(name: &str, index: usize, arg: &LuaValue) -> Result<String, String> {
    match arg {
        LuaValue::String(s) => Ok(s.clone()),
        _ => Err(format!(
            "{}() expects string as argument {}, got {}",
            name,
            index + 1,
            arg.type_name()
        )),
    }
}

/// Extract table with type checking
/// 
/// # Arguments
/// * `name` - Function name for error messages
/// * `index` - Argument position (0-based)
/// * `arg` - The argument to extract
pub fn get_table(name: &str, index: usize, arg: &LuaValue) -> Result<Rc<RefCell<LuaTable>>, String> {
    match arg {
        LuaValue::Table(t) => Ok(t.clone()),
        _ => Err(format!(
            "{}() expects table as argument {}, got {}",
            name,
            index + 1,
            arg.type_name()
        )),
    }
}

/// Extract boolean with type checking
/// 
/// # Arguments
/// * `name` - Function name for error messages
/// * `index` - Argument position (0-based)
/// * `arg` - The argument to extract
pub fn get_boolean(name: &str, index: usize, arg: &LuaValue) -> Result<bool, String> {
    match arg {
        LuaValue::Boolean(b) => Ok(*b),
        _ => Err(format!(
            "{}() expects boolean as argument {}, got {}",
            name,
            index + 1,
            arg.type_name()
        )),
    }
}

/// Extract number as integer with type checking
/// 
/// # Arguments
/// * `name` - Function name for error messages
/// * `index` - Argument position (0-based)
/// * `arg` - The argument to extract
pub fn get_integer(name: &str, index: usize, arg: &LuaValue) -> Result<i64, String> {
    match arg {
        LuaValue::Number(n) => Ok(*n as i64),
        _ => Err(format!(
            "{}() expects number as argument {}, got {}",
            name,
            index + 1,
            arg.type_name()
        )),
    }
}
