use crate::lua_value::{LuaValue, LuaTable};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::cell::RefCell;

/// A call frame representing a function call context
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// Local variables in this frame
    pub locals: HashMap<String, LuaValue>,
    /// Name of the function (for debugging)
    pub func_name: String,
    /// Return values from this function call
    pub return_values: Vec<LuaValue>,
    /// Number of expected return values (-1 means variadic)
    pub expected_returns: i32,
}

impl CallFrame {
    pub fn new(func_name: String) -> Self {
        CallFrame {
            locals: HashMap::new(),
            func_name,
            return_values: Vec::new(),
            expected_returns: -1,
        }
    }

    pub fn with_returns(func_name: String, expected_returns: i32) -> Self {
        CallFrame {
            locals: HashMap::new(),
            func_name,
            return_values: Vec::new(),
            expected_returns,
        }
    }
}

/// Manages value stack for temporary storage during computation
#[derive(Debug, Clone)]
pub struct ValueStack {
    /// Stack of values for intermediate computation
    values: Vec<LuaValue>,
}

impl ValueStack {
    pub fn new() -> Self {
        ValueStack {
            values: Vec::new(),
        }
    }

    pub fn push(&mut self, value: LuaValue) {
        self.values.push(value);
    }

    pub fn pop(&mut self) -> Option<LuaValue> {
        self.values.pop()
    }

    pub fn peek(&self) -> Option<&LuaValue> {
        self.values.last()
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl Default for ValueStack {
    fn default() -> Self {
        Self::new()
    }
}

/// The Lua interpreter with global state and execution context
pub struct LuaInterpreter {
    /// Global variables
    pub globals: HashMap<String, LuaValue>,
    /// Stack of local scopes
    pub scope_stack: Vec<HashMap<String, LuaValue>>,
    /// Call stack for function calls
    pub call_stack: Vec<CallFrame>,
    /// Value stack for temporary computation
    pub value_stack: ValueStack,
    /// Set of reachable objects for garbage collection (mark-and-sweep)
    pub reachable_objects: HashSet<usize>,
    /// Maximum recursion depth to prevent stack overflow
    pub max_call_depth: usize,
}

impl LuaInterpreter {
    /// Create a new Lua interpreter with standard library functions
    pub fn new() -> Self {
        Self::with_max_depth(1000)
    }

    /// Create a new interpreter with custom max recursion depth
    pub fn with_max_depth(max_depth: usize) -> Self {
        let mut interpreter = LuaInterpreter {
            globals: HashMap::new(),
            scope_stack: Vec::new(),
            call_stack: Vec::new(),
            value_stack: ValueStack::new(),
            reachable_objects: HashSet::new(),
            max_call_depth: max_depth,
        };

        // Initialize standard library
        interpreter.init_stdlib();

        interpreter
    }

    /// Initialize standard library functions
    fn init_stdlib(&mut self) {
        // Standard library functions will be added in Phase 6
        // Placeholder for now
    }

    /// Push a new scope for block statements or function calls
    pub fn push_scope(&mut self) {
        self.scope_stack.push(HashMap::new());
    }

    /// Pop the current scope
    pub fn pop_scope(&mut self) {
        self.scope_stack.pop();
    }

    /// Push a call frame for function call context
    pub fn push_call_frame(&mut self, func_name: String) -> Result<(), String> {
        if self.call_stack.len() >= self.max_call_depth {
            return Err(format!("Maximum call depth {} exceeded", self.max_call_depth));
        }
        self.call_stack.push(CallFrame::new(func_name));
        Ok(())
    }

    /// Push a call frame with expected return count
    pub fn push_call_frame_with_returns(&mut self, func_name: String, expected_returns: i32) -> Result<(), String> {
        if self.call_stack.len() >= self.max_call_depth {
            return Err(format!("Maximum call depth {} exceeded", self.max_call_depth));
        }
        self.call_stack.push(CallFrame::with_returns(func_name, expected_returns));
        Ok(())
    }

    /// Pop the current call frame and get its return values
    pub fn pop_call_frame(&mut self) -> Vec<LuaValue> {
        self.call_stack.pop()
            .map(|frame| frame.return_values)
            .unwrap_or_default()
    }

    /// Set return values for current call frame
    pub fn set_return_values(&mut self, values: Vec<LuaValue>) {
        if let Some(frame) = self.call_stack.last_mut() {
            frame.return_values = values;
        }
    }

    /// Push a value onto the value stack
    pub fn value_stack_push(&mut self, value: LuaValue) {
        self.value_stack.push(value);
    }

    /// Pop a value from the value stack
    pub fn value_stack_pop(&mut self) -> Option<LuaValue> {
        self.value_stack.pop()
    }

    /// Peek at the top of value stack
    pub fn value_stack_peek(&self) -> Option<&LuaValue> {
        self.value_stack.peek()
    }

    /// Clear the value stack
    pub fn value_stack_clear(&mut self) {
        self.value_stack.clear();
    }

    /// Define or update a variable in the current scope
    pub fn define(&mut self, name: String, value: LuaValue) {
        if let Some(scope) = self.scope_stack.last_mut() {
            scope.insert(name, value);
        } else {
            self.globals.insert(name, value);
        }
    }

    /// Look up a variable, checking scopes from innermost to outermost, then globals
    pub fn lookup(&self, name: &str) -> Option<LuaValue> {
        // Check scopes from innermost to outermost
        for scope in self.scope_stack.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        // Check globals
        self.globals.get(name).cloned()
    }

    /// Update an existing variable, searching scopes from innermost to outermost, then globals
    pub fn update(&mut self, name: &str, value: LuaValue) -> Result<(), String> {
        // Check scopes from innermost to outermost
        for scope in self.scope_stack.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        // Check globals
        if self.globals.contains_key(name) {
            self.globals.insert(name.to_string(), value);
            Ok(())
        } else {
            Err(format!("Undefined variable: {}", name))
        }
    }

    /// Create a new empty table
    pub fn create_table(&self) -> LuaValue {
        LuaValue::Table(Rc::new(RefCell::new(LuaTable {
            data: HashMap::new(),
            metatable: None,
        })))
    }

    /// Get the current call depth (for debugging/recursion limits)
    pub fn call_depth(&self) -> usize {
        self.call_stack.len()
    }

    /// Mark a table as reachable (for garbage collection)
    pub fn mark_reachable_table(&mut self, table: &LuaValue) {
        if let LuaValue::Table(t) = table {
            self.reachable_objects.insert(t.as_ptr() as usize);
        }
    }

    /// Mark all values in a scope as reachable
    pub fn mark_scope_reachable(&mut self, scope: &HashMap<String, LuaValue>) {
        for value in scope.values() {
            if let LuaValue::Table(t) = value {
                self.reachable_objects.insert(t.as_ptr() as usize);
            }
        }
    }

    /// Perform garbage collection (mark-and-sweep style)
    /// This is a simplified GC that marks all currently reachable objects
    pub fn collect_garbage(&mut self) {
        self.reachable_objects.clear();

        // Mark global values
        for value in self.globals.values() {
            if let LuaValue::Table(t) = value {
                self.reachable_objects.insert(t.as_ptr() as usize);
            }
        }

        // Mark values in all scopes
        for scope in &self.scope_stack {
            for value in scope.values() {
                if let LuaValue::Table(t) = value {
                    self.reachable_objects.insert(t.as_ptr() as usize);
                }
            }
        }

        // Mark values in call frames
        for frame in &self.call_stack {
            for value in frame.locals.values() {
                if let LuaValue::Table(t) = value {
                    self.reachable_objects.insert(t.as_ptr() as usize);
                }
            }
            for value in &frame.return_values {
                if let LuaValue::Table(t) = value {
                    self.reachable_objects.insert(t.as_ptr() as usize);
                }
            }
        }

        // Mark values in value stack
        for value in &self.value_stack.values {
            if let LuaValue::Table(t) = value {
                self.reachable_objects.insert(t.as_ptr() as usize);
            }
        }
    }

    /// Get current memory usage estimate
    pub fn memory_usage(&self) -> usize {
        let mut size = std::mem::size_of::<Self>();

        // Approximate size of globals
        size += self.globals.len() * (std::mem::size_of::<String>() + std::mem::size_of::<LuaValue>());

        // Approximate size of scopes
        for scope in &self.scope_stack {
            size += scope.len() * (std::mem::size_of::<String>() + std::mem::size_of::<LuaValue>());
        }

        // Size of call stack
        size += self.call_stack.len() * std::mem::size_of::<CallFrame>();

        // Size of value stack
        size += self.value_stack.values.len() * std::mem::size_of::<LuaValue>();

        size
    }
}

impl Default for LuaInterpreter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpreter_creation() {
        let interp = LuaInterpreter::new();
        assert_eq!(interp.globals.len(), 0);
        assert!(interp.scope_stack.is_empty());
        assert!(interp.call_stack.is_empty());
        assert!(interp.value_stack.is_empty());
        assert_eq!(interp.max_call_depth, 1000);
    }

    #[test]
    fn test_global_variable_definition() {
        let mut interp = LuaInterpreter::new();
        interp.define("x".to_string(), LuaValue::Number(42.0));
        assert_eq!(interp.lookup("x"), Some(LuaValue::Number(42.0)));
    }

    #[test]
    fn test_scope_stack() {
        let mut interp = LuaInterpreter::new();
        interp.define("x".to_string(), LuaValue::Number(1.0));

        interp.push_scope();
        interp.define("x".to_string(), LuaValue::Number(2.0));
        assert_eq!(interp.lookup("x"), Some(LuaValue::Number(2.0)));

        interp.pop_scope();
        assert_eq!(interp.lookup("x"), Some(LuaValue::Number(1.0)));
    }

    #[test]
    fn test_create_table() {
        let interp = LuaInterpreter::new();
        let table = interp.create_table();
        assert_eq!(table.type_name(), "table");
    }

    #[test]
    fn test_call_frame_tracking() {
        let mut interp = LuaInterpreter::new();
        assert_eq!(interp.call_depth(), 0);

        assert!(interp.push_call_frame("func1".to_string()).is_ok());
        assert_eq!(interp.call_depth(), 1);

        assert!(interp.push_call_frame("func2".to_string()).is_ok());
        assert_eq!(interp.call_depth(), 2);

        interp.pop_call_frame();
        assert_eq!(interp.call_depth(), 1);

        interp.pop_call_frame();
        assert_eq!(interp.call_depth(), 0);
    }

    #[test]
    fn test_value_stack() {
        let mut interp = LuaInterpreter::new();
        assert!(interp.value_stack_peek().is_none());

        interp.value_stack_push(LuaValue::Number(1.0));
        interp.value_stack_push(LuaValue::Number(2.0));
        assert_eq!(interp.value_stack.len(), 2);

        assert_eq!(interp.value_stack_peek(), Some(&LuaValue::Number(2.0)));
        assert_eq!(interp.value_stack_pop(), Some(LuaValue::Number(2.0)));
        assert_eq!(interp.value_stack_pop(), Some(LuaValue::Number(1.0)));
        assert!(interp.value_stack_pop().is_none());
    }

    #[test]
    fn test_call_frame_returns() {
        let mut interp = LuaInterpreter::new();
        assert!(interp.push_call_frame_with_returns("test_func".to_string(), 2).is_ok());

        interp.set_return_values(vec![LuaValue::Number(10.0), LuaValue::Number(20.0)]);
        let returns = interp.pop_call_frame();
        assert_eq!(returns.len(), 2);
        assert_eq!(returns[0], LuaValue::Number(10.0));
        assert_eq!(returns[1], LuaValue::Number(20.0));
    }

    #[test]
    fn test_max_recursion_depth() {
        let mut interp = LuaInterpreter::with_max_depth(3);
        assert!(interp.push_call_frame("f1".to_string()).is_ok());
        assert!(interp.push_call_frame("f2".to_string()).is_ok());
        assert!(interp.push_call_frame("f3".to_string()).is_ok());
        assert!(interp.push_call_frame("f4".to_string()).is_err()); // Should fail
    }

    #[test]
    fn test_garbage_collection() {
        let mut interp = LuaInterpreter::new();
        let table = interp.create_table();
        
        interp.define("my_table".to_string(), table.clone());
        interp.collect_garbage();
        
        // Table should be marked as reachable
        assert!(!interp.reachable_objects.is_empty());
    }

    #[test]
    fn test_memory_usage() {
        let mut interp = LuaInterpreter::new();
        let initial = interp.memory_usage();
        
        interp.define("x".to_string(), LuaValue::Number(42.0));
        let after_define = interp.memory_usage();
        
        assert!(after_define > initial);
    }
}
