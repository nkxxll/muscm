//! Comprehensive error handling for the Lua interpreter
//!
//! Provides strongly-typed error variants with location tracking,
//! context information, and better error messages.

use std::fmt;

/// Comprehensive error type for the Lua interpreter
#[derive(Debug, Clone, PartialEq)]
pub enum LuaError {
    /// Parse error with location information
    ParseError {
        message: String,
        line: usize,
        column: usize,
    },
    /// Runtime error with execution context
    RuntimeError {
        message: String,
        context: String,
    },
    /// Type mismatch error
    TypeError {
        expected: String,
        got: String,
        function: String,
    },
    /// Value validation error
    ValueError {
        message: String,
    },
    /// File I/O error
    FileError {
        path: String,
        reason: String,
    },
    /// Module loading error
    ModuleError {
        module: String,
        reason: String,
    },
    /// Tokenization error
    TokenError {
        message: String,
        position: usize,
    },
    /// User-raised error (from error() function)
    UserError {
        message: String,
        level: usize,
    },
    /// Control flow: break outside loop
    BreakOutsideLoop,
    /// Control flow: goto to undefined label
    UndefinedLabel {
        label: String,
    },
    /// Function argument count mismatch
    ArgumentCountError {
        function: String,
        expected: usize,
        got: usize,
    },
    /// Division by zero
    DivisionByZero,
    /// Attempt to index non-indexable type
    IndexError {
        indexing_type: String,
        key_type: String,
    },
    /// Attempt to call non-callable
    CallError {
        value_type: String,
    },
}

impl LuaError {
    /// Create a parse error with location information
    pub fn parse(message: impl Into<String>, line: usize, column: usize) -> Self {
        LuaError::ParseError {
            message: message.into(),
            line,
            column,
        }
    }

    /// Create a runtime error with context
    pub fn runtime(message: impl Into<String>, context: impl Into<String>) -> Self {
        LuaError::RuntimeError {
            message: message.into(),
            context: context.into(),
        }
    }

    /// Create a type error
    pub fn type_error(
        expected: impl Into<String>,
        got: impl Into<String>,
        function: impl Into<String>,
    ) -> Self {
        LuaError::TypeError {
            expected: expected.into(),
            got: got.into(),
            function: function.into(),
        }
    }

    /// Create a value error
    pub fn value(message: impl Into<String>) -> Self {
        LuaError::ValueError {
            message: message.into(),
        }
    }

    /// Create a file error
    pub fn file(path: impl Into<String>, reason: impl Into<String>) -> Self {
        LuaError::FileError {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Create a module error
    pub fn module(module: impl Into<String>, reason: impl Into<String>) -> Self {
        LuaError::ModuleError {
            module: module.into(),
            reason: reason.into(),
        }
    }

    /// Create a tokenization error
    pub fn token(message: impl Into<String>, position: usize) -> Self {
        LuaError::TokenError {
            message: message.into(),
            position,
        }
    }

    /// Create a user-raised error
    pub fn user(message: impl Into<String>, level: usize) -> Self {
        LuaError::UserError {
            message: message.into(),
            level,
        }
    }

    /// Create an argument count error
    pub fn arg_count(function: impl Into<String>, expected: usize, got: usize) -> Self {
        LuaError::ArgumentCountError {
            function: function.into(),
            expected,
            got,
        }
    }

    /// Create an index error
    pub fn index(indexing_type: impl Into<String>, key_type: impl Into<String>) -> Self {
        LuaError::IndexError {
            indexing_type: indexing_type.into(),
            key_type: key_type.into(),
        }
    }

    /// Create a call error
    pub fn call(value_type: impl Into<String>) -> Self {
        LuaError::CallError {
            value_type: value_type.into(),
        }
    }

    /// Get error category for matching
    pub fn category(&self) -> &str {
        match self {
            LuaError::ParseError { .. } => "parse",
            LuaError::RuntimeError { .. } => "runtime",
            LuaError::TypeError { .. } => "type",
            LuaError::ValueError { .. } => "value",
            LuaError::FileError { .. } => "file",
            LuaError::ModuleError { .. } => "module",
            LuaError::TokenError { .. } => "token",
            LuaError::UserError { .. } => "user",
            LuaError::BreakOutsideLoop => "control_flow",
            LuaError::UndefinedLabel { .. } => "label",
            LuaError::ArgumentCountError { .. } => "argument",
            LuaError::DivisionByZero => "arithmetic",
            LuaError::IndexError { .. } => "index",
            LuaError::CallError { .. } => "call",
        }
    }

    /// Get the message string for error reporting
    pub fn message(&self) -> String {
        match self {
            LuaError::ParseError {
                message,
                line,
                column,
            } => format!("Parse error at {}:{}: {}", line, column, message),
            LuaError::RuntimeError { message, context } => {
                format!("Runtime error ({}): {}", context, message)
            }
            LuaError::TypeError {
                expected,
                got,
                function,
            } => format!(
                "Type error in {}: expected {}, got {}",
                function, expected, got
            ),
            LuaError::ValueError { message } => format!("Value error: {}", message),
            LuaError::FileError { path, reason } => {
                format!("File error ({}): {}", path, reason)
            }
            LuaError::ModuleError { module, reason } => {
                format!("Module error ({}): {}", module, reason)
            }
            LuaError::TokenError { message, position } => {
                format!("Token error at position {}: {}", position, message)
            }
            LuaError::UserError { message, .. } => message.clone(),
            LuaError::BreakOutsideLoop => "break statement outside loop".to_string(),
            LuaError::UndefinedLabel { label } => format!("undefined label: {}", label),
            LuaError::ArgumentCountError {
                function,
                expected,
                got,
            } => format!(
                "Function {} expects {} argument(s), got {}",
                function, expected, got
            ),
            LuaError::DivisionByZero => "division by zero".to_string(),
            LuaError::IndexError {
                indexing_type,
                key_type,
            } => format!(
                "Cannot index {} with {}",
                indexing_type, key_type
            ),
            LuaError::CallError { value_type } => {
                format!("Attempt to call {} (not a function)", value_type)
            }
        }
    }
}

impl fmt::Display for LuaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for LuaError {}

/// Convenience type alias for Result with LuaError
pub type LuaResult<T> = Result<T, LuaError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_creation() {
        let err = LuaError::parse("unexpected token", 10, 5);
        assert_eq!(err.category(), "parse");
        assert!(err.message().contains("10:5"));
    }

    #[test]
    fn test_runtime_error_creation() {
        let err = LuaError::runtime("invalid operation", "assignment");
        assert_eq!(err.category(), "runtime");
        assert!(err.message().contains("assignment"));
    }

    #[test]
    fn test_type_error_creation() {
        let err = LuaError::type_error("number", "string", "math.abs");
        assert_eq!(err.category(), "type");
        assert!(err.message().contains("math.abs"));
        assert!(err.message().contains("number"));
    }

    #[test]
    fn test_file_error_creation() {
        let err = LuaError::file("test.lua", "file not found");
        assert_eq!(err.category(), "file");
        assert!(err.message().contains("test.lua"));
    }

    #[test]
    fn test_module_error_creation() {
        let err = LuaError::module("mylib", "circular dependency detected");
        assert_eq!(err.category(), "module");
        assert!(err.message().contains("mylib"));
    }

    #[test]
    fn test_user_error_creation() {
        let err = LuaError::user("custom error message", 1);
        assert_eq!(err.category(), "user");
        match err {
            LuaError::UserError { message, level } => {
                assert_eq!(message, "custom error message");
                assert_eq!(level, 1);
            }
            _ => panic!("Expected UserError"),
        }
    }

    #[test]
    fn test_argument_count_error() {
        let err = LuaError::arg_count("print", 1, 3);
        assert_eq!(err.category(), "argument");
        assert!(err.message().contains("print"));
        assert!(err.message().contains("1"));
        assert!(err.message().contains("3"));
    }

    #[test]
    fn test_break_outside_loop() {
        let err = LuaError::BreakOutsideLoop;
        assert_eq!(err.category(), "control_flow");
        assert!(err.message().contains("break"));
    }

    #[test]
    fn test_division_by_zero() {
        let err = LuaError::DivisionByZero;
        assert_eq!(err.category(), "arithmetic");
        assert!(err.message().contains("division"));
    }

    #[test]
    fn test_display_impl() {
        let err = LuaError::value("test error");
        let display_str = format!("{}", err);
        assert!(display_str.contains("test error"));
    }

    #[test]
    fn test_error_conversion_chain() {
        let err: LuaResult<i32> = Err(LuaError::value("oops"));
        assert!(err.is_err());
    }
}
