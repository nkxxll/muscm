pub mod iterators;
pub mod math;
pub mod metatables;
pub mod string;
pub mod table;
pub mod types;
/// Standard Library Module Organization
///
/// This module provides essential Lua standard library functions organized by submodule:
/// - string: string.len, string.sub, string.upper, string.lower
/// - math: math.abs, math.floor, math.ceil, math.min, math.max, math.random
/// - table: table.insert, table.remove
/// - types: type(), tonumber(), tostring()
/// - iterators: pairs(), ipairs(), next()
/// - metatables: setmetatable(), getmetatable(), pcall(), xpcall(), error(), coroutine
/// - io: print, io.read, io.write, io.open, io.input, io.output
/// - os: os.execute, os.exit, os.getenv, os.setenv, os.time, os.remove, os.rename, os.tmpname
/// - require: Module system for loading .lua files
pub mod validation;

use crate::error_types::{LuaError, LuaResult};
use crate::lua_value::LuaValue;
use std::rc::Rc;

/// Create the print function that outputs values to stdout
pub fn create_print() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        let output = args
            .iter()
            .map(|v| match v {
                LuaValue::String(s) => s.clone(),
                LuaValue::Nil => "nil".to_string(),
                LuaValue::Boolean(b) => b.to_string(),
                LuaValue::Number(n) => {
                    if n.fract() == 0.0 && !n.is_infinite() {
                        format!("{}", *n as i64)
                    } else {
                        n.to_string()
                    }
                }
                LuaValue::Table(_) => "table".to_string(),
                LuaValue::Function(_) => "function".to_string(),
                LuaValue::UserData(_) => "userdata".to_string(),
            })
            .collect::<Vec<_>>()
            .join("\t");

        println!("{}", output);
        Ok(LuaValue::Nil)
    })
}

// Re-export public functions from submodules for backward compatibility
pub use iterators::{create_ipairs, create_next, create_pairs};
pub use math::{
    create_math_abs, create_math_ceil, create_math_floor, create_math_max, create_math_min,
    create_math_random, create_math_table,
};
pub use metatables::{
    create_coroutine_table, create_error, create_getmetatable, create_pcall, create_setmetatable,
    create_xpcall,
};
pub use string::{
    create_string_len, create_string_lower, create_string_sub, create_string_table,
    create_string_upper,
};
pub use table::{create_table_insert, create_table_remove, create_table_table};
pub use types::{create_tonumber, create_tostring, create_type};

/// Create an io table with I/O functions (delegates to file_io module)
pub fn create_io_table() -> LuaValue {
    crate::file_io::create_enhanced_io_table()
}

/// Create an os table with all os functions (delegates to file_io module)
pub fn create_os_table() -> LuaValue {
    crate::file_io::create_os_table()
}

/// Create the require() function for loading modules
///
/// Takes a module name (string) and loads the corresponding .lua file
/// Returns the module's exported value or exports table
pub fn create_require(
    _loader: std::rc::Rc<std::cell::RefCell<crate::module_loader::ModuleLoader>>,
) -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(move |args| {
        if args.is_empty() {
            return Err(LuaError::arg_count("require", 1, 0));
        }

        let module_name = match &args[0] {
            LuaValue::String(s) => s.clone(),
            _ => return Err(LuaError::type_error("string", args[0].type_name(), "require")),
        };

        // Note: The actual module loading happens in Executor::execute_call
        // because we need access to the Executor and Interpreter instances.
        // This function is a placeholder that returns an error.
        // The real require() is handled specially in execute_call.
        Err(LuaError::module(
            module_name,
            "require() must be called through executor, not directly",
        ))
    })
}
