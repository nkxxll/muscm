/// Coroutine support for cooperative multitasking
/// Enables yield/resume patterns for generator-like behavior

use crate::lua_value::LuaValue;
use crate::lua_parser::Statement;
use std::collections::HashMap;

/// State of a coroutine
#[derive(Debug, Clone, PartialEq)]
pub enum CoroutineStatus {
    /// Coroutine has been created but never resumed
    Suspended,
    /// Coroutine is currently running
    Running,
    /// Coroutine has finished execution
    Dead,
}

impl ToString for CoroutineStatus {
    fn to_string(&self) -> String {
        match self {
            CoroutineStatus::Suspended => "suspended".to_string(),
            CoroutineStatus::Running => "running".to_string(),
            CoroutineStatus::Dead => "dead".to_string(),
        }
    }
}

/// A Lua coroutine that can be suspended and resumed
#[derive(Debug, Clone)]
pub struct Coroutine {
    /// Unique identifier for this coroutine
    pub id: usize,
    /// Current status
    pub status: CoroutineStatus,
    /// Saved program counter (instruction index)
    pub pc: usize,
    /// Parameters for the function
    pub params: Vec<String>,
    /// Function body to execute
    pub body: Vec<Statement>,
    /// Local variables at current suspension point
    pub locals: HashMap<String, LuaValue>,
    /// Values from the last yield or arguments to resume
    pub yield_values: Vec<LuaValue>,
    /// Execution stack at suspension point
    pub stack: Vec<LuaValue>,
}

impl Coroutine {
    /// Create a new coroutine with a function body
    pub fn new(id: usize, params: Vec<String>, body: Vec<Statement>) -> Self {
        Coroutine {
            id,
            status: CoroutineStatus::Suspended,
            pc: 0,
            params,
            body,
            locals: HashMap::new(),
            yield_values: Vec::new(),
            stack: Vec::new(),
        }
    }

    /// Resume the coroutine with given arguments
    pub fn resume(&mut self, args: Vec<LuaValue>) -> (bool, Vec<LuaValue>) {
        if self.status == CoroutineStatus::Dead {
            return (false, vec![LuaValue::String("cannot resume dead coroutine".to_string())]);
        }

        if self.status == CoroutineStatus::Suspended {
            // Set the arguments as the values for this resume
            self.yield_values = args;
            self.status = CoroutineStatus::Running;
            (true, vec![])
        } else {
            (false, vec![LuaValue::String("cannot resume running coroutine".to_string())])
        }
    }

    /// Suspend the coroutine at the current point
    pub fn yield_values(&mut self, values: Vec<LuaValue>) {
        self.status = CoroutineStatus::Suspended;
        self.yield_values = values;
    }

    /// Mark the coroutine as finished
    pub fn finish(&mut self, return_values: Vec<LuaValue>) -> Vec<LuaValue> {
        self.status = CoroutineStatus::Dead;
        self.yield_values = return_values;
        self.yield_values.clone()
    }

    /// Get the status as a Lua value
    pub fn status_value(&self) -> LuaValue {
        LuaValue::String(self.status.to_string())
    }

    /// Check if coroutine can be resumed
    pub fn is_resumable(&self) -> bool {
        self.status == CoroutineStatus::Suspended
    }
}

/// Registry to manage all active coroutines
#[derive(Debug, Clone)]
pub struct CoroutineRegistry {
    /// Map of coroutine ID to coroutine
    coroutines: HashMap<usize, Coroutine>,
    /// Next ID to assign
    next_id: usize,
    /// Currently active coroutine ID
    active_id: Option<usize>,
}

impl CoroutineRegistry {
    pub fn new() -> Self {
        CoroutineRegistry {
            coroutines: HashMap::new(),
            next_id: 1,
            active_id: None,
        }
    }

    /// Create a new coroutine and return its ID
    pub fn create(&mut self, params: Vec<String>, body: Vec<Statement>) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let co = Coroutine::new(id, params, body);
        self.coroutines.insert(id, co);
        id
    }

    /// Get a mutable reference to a coroutine by ID
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Coroutine> {
        self.coroutines.get_mut(&id)
    }

    /// Get a coroutine by ID
    pub fn get(&self, id: usize) -> Option<&Coroutine> {
        self.coroutines.get(&id)
    }

    /// Set the currently active coroutine
    pub fn set_active(&mut self, id: usize) {
        self.active_id = Some(id);
    }

    /// Get the active coroutine ID
    pub fn get_active(&self) -> Option<usize> {
        self.active_id
    }

    /// Clear active coroutine
    pub fn clear_active(&mut self) {
        self.active_id = None;
    }
}

impl Default for CoroutineRegistry {
    fn default() -> Self {
        Self::new()
    }
}
