/// Phase 6: Built-in Standard Library Functions
/// Phase 7: Advanced Features (Metatables, Coroutines, Error Handling)
/// Phase 8: File I/O & System Integration
/// Phase 9: Module System
/// 
/// This module provides essential Lua standard library functions:
/// - I/O: print, io.read, io.write, io.open, io.input, io.output
/// - File operations: file:read, file:write, file:close
/// - Table: table.insert, table.remove, pairs, ipairs
/// - String: string.len, string.sub, string.upper, string.lower, tostring
/// - Math: math.abs, math.floor, math.ceil, math.min, math.max, math.random
/// - Type: type(), tonumber(), tostring()
/// - Iteration: next()
/// - Metatables: setmetatable, getmetatable
/// - Error Handling: pcall, xpcall, error
/// - Coroutines: coroutine.create, coroutine.resume, coroutine.yield, coroutine.status
/// - OS: os.execute, os.exit, os.getenv, os.setenv, os.time, os.remove, os.rename, os.tmpname
/// - Module System: require()

pub mod validation;

use crate::lua_value::{LuaValue, LuaTable};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

/// Create the print function that outputs values to stdout
pub fn create_print() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
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
            LuaValue::String(s) => {
                match s.trim().parse::<f64>() {
                    Ok(n) => Ok(LuaValue::Number(n)),
                    Err(_) => Ok(LuaValue::Nil),
                }
            }
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

/// Create string.len() function
pub fn create_string_len() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        validation::require_args("string.len", &args, 1, Some(1))?;
        let s = validation::get_string("string.len", 0, &args[0])?;
        Ok(LuaValue::Number(s.len() as f64))
    })
}

/// Create string.sub() function
pub fn create_string_sub() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        if args.len() < 2 {
            return Err("string.sub() requires at least 2 arguments".to_string());
        }
        
        let s = match &args[0] {
            LuaValue::String(s) => s.clone(),
            _ => return Err(format!("string.sub() expects string as first argument")),
        };
        
        let start_lua = match &args[1] {
            LuaValue::Number(n) => *n as i32,
            _ => return Err("string.sub() expects number as second argument".to_string()),
        };
        
        let end_lua = if args.len() >= 3 {
            match &args[2] {
                LuaValue::Number(n) => *n as i32,
                _ => return Err("string.sub() expects number as third argument".to_string()),
            }
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
pub fn create_string_upper() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        validation::require_args("string.upper", &args, 1, Some(1))?;
        let s = validation::get_string("string.upper", 0, &args[0])?;
        Ok(LuaValue::String(s.to_uppercase()))
    })
}

/// Create string.lower() function
pub fn create_string_lower() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        validation::require_args("string.lower", &args, 1, Some(1))?;
        let s = validation::get_string("string.lower", 0, &args[0])?;
        Ok(LuaValue::String(s.to_lowercase()))
    })
}

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

/// Create table.insert() function
pub fn create_table_insert() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
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
        
        let pos = if index < 0 {
            len + 1
        } else {
            index
        };
        
        table.data.insert(LuaValue::Number(pos as f64), value);
        Ok(LuaValue::Nil)
    })
}

/// Create table.remove() function
pub fn create_table_remove() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
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
        
        let removed = table.data.remove(&LuaValue::Number(pos as f64))
            .unwrap_or(LuaValue::Nil);
        
        Ok(removed)
    })
}

/// Create pairs() iterator function
pub fn create_pairs() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|_args| {
        // Return a dummy function for now - full iterator support in future
        Ok(LuaValue::Function(Rc::new(
            crate::lua_value::LuaFunction::Builtin(
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
            crate::lua_value::LuaFunction::Builtin(
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

/// Create a string table with all string functions
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

/// Create a math table with all math functions
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

/// Create a table table with all table functions
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

/// Create an io table with I/O functions (delegates to file_io module)
pub fn create_io_table() -> LuaValue {
    crate::file_io::create_enhanced_io_table()
}

// ============================================================================
// PHASE 7: METATABLES, COROUTINES, AND ERROR HANDLING
// ============================================================================

/// Create the setmetatable() function
/// Sets or replaces the metatable for a table
pub fn create_setmetatable() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
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
            _ => Err("setmetatable() second argument must be a table or nil".to_string()),
        }
    })
}

/// Create the getmetatable() function
/// Returns the metatable of a table
pub fn create_getmetatable() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
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
pub fn create_pcall() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        validation::require_args("pcall", &args, 1, None)?;
        
        // For now, return a simple implementation
        // In full implementation, this would actually catch errors from function execution
        match &args[0] {
            LuaValue::Function(_) => {
                // Return success (true) and nil as placeholder
                Ok(LuaValue::Boolean(true))
            }
            _ => Err("pcall() first argument must be a function".to_string()),
        }
    })
}

/// Create the xpcall() function
/// Extended protected call with custom error handler
pub fn create_xpcall() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        validation::require_args("xpcall", &args, 2, None)?;
        
        match (&args[0], &args[1]) {
            (LuaValue::Function(_), LuaValue::Function(_)) => {
                // Return success (true) and nil as placeholder
                Ok(LuaValue::Boolean(true))
            }
            _ => Err("xpcall() first two arguments must be functions".to_string()),
        }
    })
}

/// Create the error() function
/// Throws an error with a message
pub fn create_error() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        let message = if args.is_empty() {
            "".to_string()
        } else {
            match &args[0] {
                LuaValue::String(s) => s.clone(),
                v => v.to_string(),
            }
        };
        Err(message)
    })
}

/// Create the coroutine module table
pub fn create_coroutine_table() -> LuaValue {
    use crate::lua_value::LuaFunction;
    
    let mut coro_table = HashMap::new();
    
    // coroutine.create
    coro_table.insert(
        LuaValue::String("create".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(
            Rc::new(|_| {
                Err("coroutine.create() requires executor context".to_string())
            })
        ))),
    );
    
    // coroutine.resume
    coro_table.insert(
        LuaValue::String("resume".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(
            Rc::new(|_| {
                Err("coroutine.resume() requires executor context".to_string())
            })
        ))),
    );
    
    // coroutine.yield
    coro_table.insert(
        LuaValue::String("yield".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(
            Rc::new(|_| {
                Err("coroutine.yield() requires executor context".to_string())
            })
        ))),
    );
    
    // coroutine.status
    coro_table.insert(
        LuaValue::String("status".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(
            Rc::new(|_| {
                Err("coroutine.status() requires executor context".to_string())
            })
        ))),
    );
    
    LuaValue::Table(Rc::new(RefCell::new(LuaTable {
        data: coro_table,
        metatable: None,
    })))
}

// ============================================================================
// PHASE 8: FILE I/O & SYSTEM INTEGRATION
// ============================================================================

/// Create an os table with all os functions (delegates to file_io module)
pub fn create_os_table() -> LuaValue {
    crate::file_io::create_os_table()
}

// ============================================================================
// PHASE 9: MODULE SYSTEM
// ============================================================================

/// Create the require() function for loading modules
/// 
/// Takes a module name (string) and loads the corresponding .lua file
/// Returns the module's exported value or exports table
pub fn create_require(
    _loader: std::rc::Rc<std::cell::RefCell<crate::module_loader::ModuleLoader>>,
) -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(move |args| {
        if args.is_empty() {
            return Err("require() needs a module name argument".to_string());
        }

        let module_name = match &args[0] {
            LuaValue::String(s) => s.clone(),
            _ => return Err("Module name must be a string".to_string()),
        };

        // Note: The actual module loading happens in Executor::execute_call
        // because we need access to the Executor and Interpreter instances.
        // This function is a placeholder that returns an error.
        // The real require() is handled specially in execute_call.
        Err(format!(
            "require() must be called through executor, not directly (module: {})",
            module_name
        ))
    })
}
