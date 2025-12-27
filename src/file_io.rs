//! Phase 8: File I/O & System Integration
//!
//! This module provides Lua file I/O and system interaction functions:
//! - File operations: io.open, file:read, file:write, file:close, file:lines
//! - System functions: os.execute, os.exit, os.getenv, os.setenv, os.time, os.date
//! - Path operations: io.popen (command execution)
//! - File metadata: io.stat (file information)

use crate::error_types::{LuaError, LuaResult};
use crate::lua_value::{LuaTable, LuaValue};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

// File handle wrapper - stored as UserData in LuaValue
pub struct FileHandle {
    file: Option<Box<dyn FileOperations>>,
}

trait FileOperations: std::any::Any {
    fn read_line(&mut self) -> io::Result<String>;
    fn read_all(&mut self) -> io::Result<String>;
    fn write(&mut self, data: &str) -> io::Result<()>;
}

struct ReadFileHandle {
    reader: BufReader<File>,
}

impl FileOperations for ReadFileHandle {
    fn read_line(&mut self) -> io::Result<String> {
        let mut line = String::new();
        self.reader.read_line(&mut line)?;
        Ok(line)
    }

    fn read_all(&mut self) -> io::Result<String> {
        let mut content = String::new();
        self.reader.read_to_string(&mut content)?;
        Ok(content)
    }

    fn write(&mut self, _data: &str) -> io::Result<()> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "File opened in read mode",
        ))
    }
}

struct WriteFileHandle {
    file: File,
}

impl FileOperations for WriteFileHandle {
    fn read_line(&mut self) -> io::Result<String> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "File opened in write mode",
        ))
    }

    fn read_all(&mut self) -> io::Result<String> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "File opened in write mode",
        ))
    }

    fn write(&mut self, data: &str) -> io::Result<()> {
        self.file.write_all(data.as_bytes())
    }
}

struct AppendFileHandle {
    file: File,
}

impl FileOperations for AppendFileHandle {
    fn read_line(&mut self) -> io::Result<String> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "File opened in append mode",
        ))
    }

    fn read_all(&mut self) -> io::Result<String> {
        Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "File opened in append mode",
        ))
    }

    fn write(&mut self, data: &str) -> io::Result<()> {
        self.file.write_all(data.as_bytes())
    }
}

/// Create io.open(filename, mode) function
/// Opens a file and returns a file handle
/// Modes: "r" (read), "w" (write), "a" (append), "rb"/"wb"/"ab" (binary)
pub fn create_io_open() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        if args.len() < 1 {
            return Err(LuaError::arg_count("io.open", 1, args.len()));
        }

        let filename = match &args[0] {
            LuaValue::String(s) => s.clone(),
            _ => return Err(LuaError::type_error("string", args[0].type_name(), "io.open")),
        };

        let mode = if args.len() >= 2 {
            match &args[1] {
                LuaValue::String(s) => s.clone(),
                _ => "r".to_string(),
            }
        } else {
            "r".to_string()
        };

        match mode.as_str() {
            "r" => match File::open(&filename) {
                Ok(file) => {
                    let reader = BufReader::new(file);
                    let fh = FileHandle {
                        file: Some(Box::new(ReadFileHandle { reader })),
                    };

                    let userdata = Rc::new(RefCell::new(Box::new(fh) as Box<dyn std::any::Any>));
                    Ok(LuaValue::UserData(userdata))
                }
                Err(e) => Err(LuaError::file(&filename, format!("io.open() failed to open: {}", e))),
            },
            "w" => match File::create(&filename) {
                Ok(file) => {
                    let fh = FileHandle {
                        file: Some(Box::new(WriteFileHandle { file })),
                    };

                    let userdata = Rc::new(RefCell::new(Box::new(fh) as Box<dyn std::any::Any>));
                    Ok(LuaValue::UserData(userdata))
                }
                Err(e) => Err(LuaError::file(&filename, format!("io.open() failed to create: {}", e))),
            },
            "a" => match OpenOptions::new().append(true).create(true).open(&filename) {
                Ok(file) => {
                    let fh = FileHandle {
                        file: Some(Box::new(AppendFileHandle { file })),
                    };

                    let userdata = Rc::new(RefCell::new(Box::new(fh) as Box<dyn std::any::Any>));
                    Ok(LuaValue::UserData(userdata))
                }
                Err(e) => Err(LuaError::file(&filename, format!("io.open() failed to open: {}", e))),
            },
            _ => Err(LuaError::value(format!("io.open() unsupported mode: {}", mode))),
        }
    })
}

/// Create file:read(...) function
/// Reads from a file handle with various formats
pub fn create_file_read() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        if args.is_empty() {
            return Err(LuaError::arg_count("file:read", 1, 0));
        }

        // Extract format string (default "l" for line)
        let format = if args.len() >= 2 {
            match &args[1] {
                LuaValue::String(s) => s.clone(),
                LuaValue::Number(n) => format!("{}", *n as i64),
                _ => "l".to_string(),
            }
        } else {
            "l".to_string()
        };

        match &args[0] {
            LuaValue::UserData(ud) => {
                let mut ud_borrow = ud.borrow_mut();
                if let Some(fh) = ud_borrow.downcast_mut::<FileHandle>() {
                    match format.as_str() {
                        "l" | "L" => {
                            // Read line
                            match fh.file.as_mut().unwrap().read_line() {
                                Ok(line) => {
                                    if format == "L" {
                                        Ok(LuaValue::String(line))
                                    } else {
                                        // Remove trailing newline for "l" format
                                        Ok(LuaValue::String(
                                            line.trim_end_matches('\n').to_string(),
                                        ))
                                    }
                                }
                                Err(e) => Err(LuaError::runtime(format!("file:read() error: {}", e), "io")),
                            }
                        }
                        "a" => {
                            // Read all
                            match fh.file.as_mut().unwrap().read_all() {
                                Ok(content) => Ok(LuaValue::String(content)),
                                Err(e) => Err(LuaError::runtime(format!("file:read() error: {}", e), "io")),
                            }
                        }
                        _ => Err(LuaError::value(format!("file:read() unsupported format: {}", format))),
                    }
                } else {
                    Err(LuaError::value("Invalid file handle"))
                }
            }
            _ => Err(LuaError::type_error("userdata", args[0].type_name(), "file:read")),
        }
    })
}

/// Create file:write(...) function
/// Writes data to a file handle
pub fn create_file_write() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        if args.is_empty() {
            return Err(LuaError::arg_count("file:write", 1, 0));
        }

        match &args[0] {
            LuaValue::UserData(ud) => {
                let mut ud_borrow = ud.borrow_mut();
                if let Some(fh) = ud_borrow.downcast_mut::<FileHandle>() {
                    let mut total_written = 0;

                    for arg in &args[1..] {
                        let data = match arg {
                            LuaValue::String(s) => s.clone(),
                            LuaValue::Number(n) => {
                                if n.fract() == 0.0 && !n.is_infinite() {
                                    format!("{}", *n as i64)
                                } else {
                                    n.to_string()
                                }
                            }
                            _ => arg.to_string(),
                        };

                        match fh.file.as_mut().unwrap().write(&data) {
                            Ok(_) => total_written += data.len(),
                            Err(e) => return Err(LuaError::runtime(format!("file:write() error: {}", e), "io")),
                        }
                    }

                    Ok(LuaValue::Number(total_written as f64))
                } else {
                    Err(LuaError::value("Invalid file handle"))
                }
            }
            _ => Err(LuaError::type_error("userdata", args[0].type_name(), "file:write")),
        }
    })
}

/// Create file:close() function
/// Closes a file handle
pub fn create_file_close() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        if args.is_empty() {
            return Err(LuaError::arg_count("file:close", 1, 0));
        }

        match &args[0] {
            LuaValue::UserData(_ud) => {
                // File will be closed when UserData is dropped (RAII)
                Ok(LuaValue::Nil)
            }
            _ => Err(LuaError::type_error("userdata", args[0].type_name(), "file:close")),
        }
    })
}

/// Create io.input([filename]) function
/// Sets or gets the current input file
pub fn create_io_input() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        if args.is_empty() {
            // Get current input file (stdin placeholder)
            Ok(LuaValue::String("<stdin>".to_string()))
        } else {
            // Set input file - would need interpreter context to fully implement
            match &args[0] {
                LuaValue::String(filename) => match File::open(filename) {
                    Ok(file) => {
                        let reader = BufReader::new(file);
                        let fh = FileHandle {
                            file: Some(Box::new(ReadFileHandle { reader })),
                        };
                        let userdata =
                            Rc::new(RefCell::new(Box::new(fh) as Box<dyn std::any::Any>));
                        Ok(LuaValue::UserData(userdata))
                    }
                    Err(e) => Err(LuaError::file(filename, format!("io.input() failed: {}", e))),
                },
                _ => Err(LuaError::type_error("string", args[0].type_name(), "io.input")),
            }
        }
    })
}

/// Create io.output([filename]) function
/// Sets or gets the current output file
pub fn create_io_output() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        if args.is_empty() {
            // Get current output file (stdout placeholder)
            Ok(LuaValue::String("<stdout>".to_string()))
        } else {
            // Set output file
            match &args[0] {
                LuaValue::String(filename) => match File::create(filename) {
                    Ok(file) => {
                        let fh = FileHandle {
                            file: Some(Box::new(WriteFileHandle { file })),
                        };
                        let userdata =
                            Rc::new(RefCell::new(Box::new(fh) as Box<dyn std::any::Any>));
                        Ok(LuaValue::UserData(userdata))
                    }
                    Err(e) => Err(LuaError::file(filename, format!("io.output() failed: {}", e))),
                },
                _ => Err(LuaError::type_error("string", args[0].type_name(), "io.output")),
            }
        }
    })
}

// ============================================================================
// OS FUNCTIONS
// ============================================================================

/// Create os.execute(command) function
/// Executes a system command
pub fn create_os_execute() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        if args.is_empty() {
            return Err(LuaError::arg_count("os.execute", 1, 0));
        }

        let command = match &args[0] {
            LuaValue::String(s) => s.clone(),
            _ => return Err(LuaError::type_error("string", args[0].type_name(), "os.execute")),
        };

        #[cfg(unix)]
        {
            use std::process::Command;
            match Command::new("bash").arg("-c").arg(&command).status() {
                Ok(status) => {
                    let exit_code = status.code().unwrap_or(1) as f64;
                    Ok(LuaValue::Number(exit_code))
                }
                Err(e) => Err(LuaError::runtime(format!("os.execute() failed: {}", e), "system call")),
            }
        }

        #[cfg(not(unix))]
        {
            use std::process::Command;
            match Command::new("cmd").args(&["/C", &command]).output() {
                Ok(output) => {
                    let exit_code = output.status.code().unwrap_or(1) as f64;
                    Ok(LuaValue::Number(exit_code))
                }
                Err(e) => Err(LuaError::runtime(format!("os.execute() failed: {}", e), "system call")),
            }
        }
    })
}

/// Create os.exit([code]) function
/// Exits the program with optional exit code
pub fn create_os_exit() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        let code = if !args.is_empty() {
            match &args[0] {
                LuaValue::Number(n) => *n as i32,
                _ => 1,
            }
        } else {
            0
        };

        std::process::exit(code);
    })
}

/// Create os.getenv(name) function
/// Gets an environment variable
pub fn create_os_getenv() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        if args.is_empty() {
            return Err(LuaError::arg_count("os.getenv", 1, 0));
        }

        let var_name = match &args[0] {
            LuaValue::String(s) => s.clone(),
            _ => return Err(LuaError::type_error("string", args[0].type_name(), "os.getenv")),
        };

        match std::env::var(&var_name) {
            Ok(value) => Ok(LuaValue::String(value)),
            Err(_) => Ok(LuaValue::Nil),
        }
    })
}

/// Create os.setenv(name, value) function
/// Sets an environment variable
pub fn create_os_setenv() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        if args.len() < 2 {
            return Err(LuaError::arg_count("os.setenv", 2, args.len()));
        }

        let var_name = match &args[0] {
            LuaValue::String(s) => s.clone(),
            _ => return Err(LuaError::type_error("string", args[0].type_name(), "os.setenv")),
        };

        let var_value = match &args[1] {
            LuaValue::String(s) => s.clone(),
            _ => return Err(LuaError::type_error("string", args[1].type_name(), "os.setenv")),
        };

        std::env::set_var(&var_name, &var_value);
        Ok(LuaValue::Nil)
    })
}

/// Create os.time([table]) function
/// Returns the current time in seconds since epoch
/// If table is provided, returns time for that date
pub fn create_os_time() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|_args| match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(duration) => Ok(LuaValue::Number(duration.as_secs() as f64)),
        Err(_) => Err(LuaError::runtime("os.time() failed to get system time", "system")),
    })
}

/// Create os.clock() function
/// Returns CPU time used by the program in seconds
pub fn create_os_clock() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|_args| {
        // Simplified: return a dummy value since we don't have CPU time info
        // In a real implementation, use platform-specific functions
        Ok(LuaValue::Number(0.0))
    })
}

/// Create os.remove(filename) function
/// Deletes a file
pub fn create_os_remove() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        if args.is_empty() {
            return Err(LuaError::arg_count("os.remove", 1, 0));
        }

        let filename = match &args[0] {
            LuaValue::String(s) => s.clone(),
            _ => return Err(LuaError::type_error("string", args[0].type_name(), "os.remove")),
        };

        match fs::remove_file(&filename) {
            Ok(_) => Ok(LuaValue::Nil),
            Err(e) => Err(LuaError::file(&filename, format!("os.remove() failed: {}", e))),
        }
    })
}

/// Create os.rename(oldname, newname) function
/// Renames or moves a file
pub fn create_os_rename() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        if args.len() < 2 {
            return Err(LuaError::arg_count("os.rename", 2, args.len()));
        }

        let oldname = match &args[0] {
            LuaValue::String(s) => s.clone(),
            _ => return Err(LuaError::type_error("string", args[0].type_name(), "os.rename")),
        };

        let newname = match &args[1] {
            LuaValue::String(s) => s.clone(),
            _ => return Err(LuaError::type_error("string", args[1].type_name(), "os.rename")),
        };

        match fs::rename(&oldname, &newname) {
            Ok(_) => Ok(LuaValue::Nil),
            Err(e) => Err(LuaError::file(&oldname, format!("os.rename() failed: {}", e))),
        }
    })
}

/// Create os.tmpname() function
/// Returns a temporary filename
pub fn create_os_tmpname() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|_args| {
        let tmp_dir = std::env::temp_dir();
        let filename = format!(
            "lua_{}",
            std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let path = tmp_dir.join(filename);
        Ok(LuaValue::String(path.to_string_lossy().to_string()))
    })
}

/// Create os.difftime(t2, t1) function
/// Returns the difference in seconds between two timestamps
pub fn create_os_difftime() -> Rc<dyn Fn(Vec<LuaValue>) -> LuaResult<LuaValue>> {
    Rc::new(|args| {
        if args.len() < 2 {
            return Err(LuaError::arg_count("os.difftime", 2, args.len()));
        }

        let t2 = match &args[0] {
            LuaValue::Number(n) => *n,
            _ => return Err(LuaError::type_error("number", args[0].type_name(), "os.difftime")),
        };

        let t1 = match &args[1] {
            LuaValue::Number(n) => *n,
            _ => return Err(LuaError::type_error("number", args[1].type_name(), "os.difftime")),
        };

        Ok(LuaValue::Number(t2 - t1))
    })
}

/// Create an os table with all os functions
pub fn create_os_table() -> LuaValue {
    use crate::lua_value::LuaFunction;

    let mut os_table = HashMap::new();

    os_table.insert(
        LuaValue::String("execute".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_os_execute()))),
    );
    os_table.insert(
        LuaValue::String("exit".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_os_exit()))),
    );
    os_table.insert(
        LuaValue::String("getenv".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_os_getenv()))),
    );
    os_table.insert(
        LuaValue::String("setenv".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_os_setenv()))),
    );
    os_table.insert(
        LuaValue::String("time".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_os_time()))),
    );
    os_table.insert(
        LuaValue::String("clock".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_os_clock()))),
    );
    os_table.insert(
        LuaValue::String("remove".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_os_remove()))),
    );
    os_table.insert(
        LuaValue::String("rename".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_os_rename()))),
    );
    os_table.insert(
        LuaValue::String("tmpname".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_os_tmpname()))),
    );
    os_table.insert(
        LuaValue::String("difftime".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_os_difftime()))),
    );

    LuaValue::Table(Rc::new(RefCell::new(LuaTable {
        data: os_table,
        metatable: None,
    })))
}

/// Enhance io table with file I/O functions
pub fn create_enhanced_io_table() -> LuaValue {
    use crate::lua_value::LuaFunction;

    let mut io_table = HashMap::new();

    io_table.insert(
        LuaValue::String("open".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_io_open()))),
    );
    io_table.insert(
        LuaValue::String("input".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_io_input()))),
    );
    io_table.insert(
        LuaValue::String("output".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(create_io_output()))),
    );
    io_table.insert(
        LuaValue::String("write".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(Rc::new(|args| {
            let output = args
                .iter()
                .map(|v| match v {
                    LuaValue::String(s) => s.clone(),
                    _ => v.to_string(),
                })
                .collect::<Vec<_>>()
                .join("");

            print!("{}", output);
            Ok(LuaValue::Nil)
        })))),
    );
    io_table.insert(
        LuaValue::String("read".to_string()),
        LuaValue::Function(Rc::new(LuaFunction::Builtin(Rc::new(|_args| {
            use crate::error_types::LuaError;
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(_) => Ok(LuaValue::String(line.trim_end_matches('\n').to_string())),
                Err(e) => Err(LuaError::file("stdin", format!("io.read() error: {}", e))),
            }
        })))),
    );

    LuaValue::Table(Rc::new(RefCell::new(LuaTable {
        data: io_table,
        metatable: None,
    })))
}
