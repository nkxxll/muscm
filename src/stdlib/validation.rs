/// Validation helpers for stdlib functions
///
/// Provides consistent argument validation and type checking
/// to eliminate ~150 lines of duplicated boilerplate across stdlib.
use crate::error_types::{LuaError, LuaResult};
use crate::lua_value::{LuaTable, LuaValue};
use std::cell::RefCell;
use std::rc::Rc;

/// Validate argument count with optional bounds
///
/// # Arguments
/// * `name` - Function name for error messages
/// * `args` - Arguments array
/// * `min` - Minimum required arguments
/// * `max` - Maximum allowed arguments (None for unlimited)
pub fn require_args(
    name: &str,
    args: &[LuaValue],
    min: usize,
    max: Option<usize>,
) -> LuaResult<()> {
    if args.len() < min {
        return Err(LuaError::arg_count(name, min, args.len()));
    }

    if let Some(max_args) = max {
        if args.len() > max_args {
            return Err(LuaError::arg_count(name, max_args, args.len()));
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
pub fn require_type(
    name: &str,
    index: usize,
    arg: &LuaValue,
    expected: &str,
) -> LuaResult<()> {
    let got = arg.type_name();
    if got != expected {
        Err(LuaError::type_error(expected, got, name))
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
pub fn get_number(name: &str, index: usize, arg: &LuaValue) -> LuaResult<f64> {
    match arg {
        LuaValue::Number(n) => Ok(*n),
        _ => Err(LuaError::type_error("number", arg.type_name(), name)),
    }
}

/// Extract string with type checking
///
/// # Arguments
/// * `name` - Function name for error messages
/// * `index` - Argument position (0-based)
/// * `arg` - The argument to extract
pub fn get_string(name: &str, index: usize, arg: &LuaValue) -> LuaResult<String> {
    match arg {
        LuaValue::String(s) => Ok(s.clone()),
        _ => Err(LuaError::type_error("string", arg.type_name(), name)),
    }
}

/// Extract table with type checking
///
/// # Arguments
/// * `name` - Function name for error messages
/// * `index` - Argument position (0-based)
/// * `arg` - The argument to extract
pub fn get_table(
    name: &str,
    index: usize,
    arg: &LuaValue,
) -> LuaResult<Rc<RefCell<LuaTable>>> {
    match arg {
        LuaValue::Table(t) => Ok(t.clone()),
        _ => Err(LuaError::type_error("table", arg.type_name(), name)),
    }
}

/// Extract boolean with type checking
///
/// # Arguments
/// * `name` - Function name for error messages
/// * `index` - Argument position (0-based)
/// * `arg` - The argument to extract
pub fn get_boolean(name: &str, index: usize, arg: &LuaValue) -> LuaResult<bool> {
    match arg {
        LuaValue::Boolean(b) => Ok(*b),
        _ => Err(LuaError::type_error("boolean", arg.type_name(), name)),
    }
}

/// Extract number as integer with type checking
///
/// # Arguments
/// * `name` - Function name for error messages
/// * `index` - Argument position (0-based)
/// * `arg` - The argument to extract
pub fn get_integer(name: &str, index: usize, arg: &LuaValue) -> LuaResult<i64> {
    match arg {
        LuaValue::Number(n) => Ok(*n as i64),
        _ => Err(LuaError::type_error("number", arg.type_name(), name)),
    }
}
