/// Error handling for Lua interpreter
/// Supports error() function and error propagation for pcall/xpcall

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct LuaError {
    pub message: String,
    pub level: usize, // Stack level for error reporting
}

impl LuaError {
    pub fn new(message: String) -> Self {
        LuaError { message, level: 0 }
    }

    pub fn with_level(message: String, level: usize) -> Self {
        LuaError { message, level }
    }
}

impl fmt::Display for LuaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for LuaError {}
