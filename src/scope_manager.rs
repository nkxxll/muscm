/// Phase 5: Abstraction Layer for Scope Management
///
/// This module encapsulates scope management logic, providing a clean interface
/// for scope operations. Instead of directly manipulating a Vec<HashMap> throughout
/// the codebase, all scope operations go through this abstraction.
///
/// Benefits:
/// - Encapsulation of scope logic
/// - Easier to add scope features (e.g., upvalue tracking)
/// - Reduced borrowing complexity
/// - Single point for scope manipulation
use crate::lua_value::LuaValue;
use std::collections::HashMap;

/// Manages a stack of scopes for local variables in nested blocks and functions
pub struct ScopeManager {
    /// Stack of scopes, innermost scope is at the end
    stack: Vec<HashMap<String, LuaValue>>,
}

impl ScopeManager {
    /// Create a new empty scope manager
    pub fn new() -> Self {
        ScopeManager {
            stack: Vec::new(),
        }
    }

    /// Push a new scope onto the stack
    /// Returns the depth after pushing (1-based for convenience)
    pub fn push(&mut self) -> usize {
        self.stack.push(HashMap::new());
        self.stack.len()
    }

    /// Pop the current scope from the stack
    /// Returns the popped scope, or an error if stack is empty
    pub fn pop(&mut self) -> Result<HashMap<String, LuaValue>, String> {
        self.stack.pop()
            .ok_or_else(|| "Cannot pop from empty scope stack".to_string())
    }

    /// Define a variable in the current (innermost) scope
    /// If no scopes exist, returns an error
    pub fn define(&mut self, name: String, value: LuaValue) -> Result<(), String> {
        if let Some(scope) = self.stack.last_mut() {
            scope.insert(name, value);
            Ok(())
        } else {
            Err("Cannot define variable: no active scope".to_string())
        }
    }

    /// Look up a variable, searching from innermost to outermost scope
    /// Returns a clone of the value if found, or None
    pub fn lookup(&self, name: &str) -> Option<LuaValue> {
        for scope in self.stack.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value.clone());
            }
        }
        None
    }

    /// Update an existing variable, searching from innermost to outermost scope
    /// Returns error if variable not found in any scope
    pub fn update(&mut self, name: &str, value: LuaValue) -> Result<(), String> {
        for scope in self.stack.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        Err(format!("Undefined variable in scope stack: {}", name))
    }

    /// Get the current depth of the scope stack
    pub fn depth(&self) -> usize {
        self.stack.len()
    }

    /// Check if the scope stack is empty
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// Get a reference to the current (innermost) scope, if it exists
    pub fn current_scope(&self) -> Option<&HashMap<String, LuaValue>> {
        self.stack.last()
    }

    /// Get a mutable reference to the current (innermost) scope, if it exists
    pub fn current_scope_mut(&mut self) -> Option<&mut HashMap<String, LuaValue>> {
        self.stack.last_mut()
    }

    /// Clear all scopes from the stack
    pub fn clear(&mut self) {
        self.stack.clear();
    }

    /// Get the raw scope stack (for advanced operations or migration)
    pub fn as_ref(&self) -> &Vec<HashMap<String, LuaValue>> {
        &self.stack
    }

    /// Get a mutable reference to the raw scope stack
    pub fn as_mut(&mut self) -> &mut Vec<HashMap<String, LuaValue>> {
        &mut self.stack
    }
}

impl Default for ScopeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_manager_creation() {
        let manager = ScopeManager::new();
        assert!(manager.is_empty());
        assert_eq!(manager.depth(), 0);
    }

    #[test]
    fn test_push_scope() {
        let mut manager = ScopeManager::new();
        assert_eq!(manager.push(), 1);
        assert_eq!(manager.depth(), 1);
        assert_eq!(manager.push(), 2);
        assert_eq!(manager.depth(), 2);
    }

    #[test]
    fn test_pop_scope() {
        let mut manager = ScopeManager::new();
        manager.push();
        manager.push();
        
        assert!(manager.pop().is_ok());
        assert_eq!(manager.depth(), 1);
        
        assert!(manager.pop().is_ok());
        assert_eq!(manager.depth(), 0);
        
        // Pop from empty stack should fail
        assert!(manager.pop().is_err());
    }

    #[test]
    fn test_define_variable() {
        let mut manager = ScopeManager::new();
        manager.push();
        
        assert!(manager.define("x".to_string(), LuaValue::Number(42.0)).is_ok());
        assert_eq!(manager.lookup("x"), Some(LuaValue::Number(42.0)));
    }

    #[test]
    fn test_define_without_scope() {
        let mut manager = ScopeManager::new();
        // No scope pushed yet
        let result = manager.define("x".to_string(), LuaValue::Number(42.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_lookup_across_scopes() {
        let mut manager = ScopeManager::new();
        
        // Define in outer scope
        manager.push();
        manager.define("x".to_string(), LuaValue::Number(1.0)).unwrap();
        
        // Define in inner scope
        manager.push();
        manager.define("y".to_string(), LuaValue::Number(2.0)).unwrap();
        
        // Can lookup both
        assert_eq!(manager.lookup("x"), Some(LuaValue::Number(1.0)));
        assert_eq!(manager.lookup("y"), Some(LuaValue::Number(2.0)));
        
        // Pop inner scope
        manager.pop().unwrap();
        
        // Can still lookup x (from outer scope)
        assert_eq!(manager.lookup("x"), Some(LuaValue::Number(1.0)));
        // But not y
        assert_eq!(manager.lookup("y"), None);
    }

    #[test]
    fn test_shadow_variable() {
        let mut manager = ScopeManager::new();
        
        // Define in outer scope
        manager.push();
        manager.define("x".to_string(), LuaValue::Number(1.0)).unwrap();
        
        // Shadow in inner scope
        manager.push();
        manager.define("x".to_string(), LuaValue::Number(2.0)).unwrap();
        
        // Lookup returns inner value
        assert_eq!(manager.lookup("x"), Some(LuaValue::Number(2.0)));
        
        // Pop inner scope
        manager.pop().unwrap();
        
        // Now returns outer value
        assert_eq!(manager.lookup("x"), Some(LuaValue::Number(1.0)));
    }

    #[test]
    fn test_update_variable() {
        let mut manager = ScopeManager::new();
        
        manager.push();
        manager.define("x".to_string(), LuaValue::Number(1.0)).unwrap();
        
        // Update existing variable
        assert!(manager.update("x", LuaValue::Number(2.0)).is_ok());
        assert_eq!(manager.lookup("x"), Some(LuaValue::Number(2.0)));
    }

    #[test]
    fn test_update_nonexistent_variable() {
        let mut manager = ScopeManager::new();
        manager.push();
        
        // Update nonexistent variable
        let result = manager.update("nonexistent", LuaValue::Number(1.0));
        assert!(result.is_err());
    }

    #[test]
    fn test_update_across_scopes() {
        let mut manager = ScopeManager::new();
        
        // Define in outer scope
        manager.push();
        manager.define("x".to_string(), LuaValue::Number(1.0)).unwrap();
        
        // Enter inner scope
        manager.push();
        
        // Update should modify outer scope's variable
        assert!(manager.update("x", LuaValue::Number(2.0)).is_ok());
        assert_eq!(manager.lookup("x"), Some(LuaValue::Number(2.0)));
        
        // Pop inner scope
        manager.pop().unwrap();
        
        // Outer scope should have updated value
        assert_eq!(manager.lookup("x"), Some(LuaValue::Number(2.0)));
    }

    #[test]
    fn test_current_scope() {
        let mut manager = ScopeManager::new();
        
        // No scope
        assert!(manager.current_scope().is_none());
        
        // Add scope
        manager.push();
        assert!(manager.current_scope().is_some());
        
        // Define variable in current scope
        manager.define("x".to_string(), LuaValue::Number(42.0)).unwrap();
        
        // Can access through current_scope
        assert_eq!(manager.current_scope().unwrap().get("x"), Some(&LuaValue::Number(42.0)));
    }

    #[test]
    fn test_clear() {
        let mut manager = ScopeManager::new();
        manager.push();
        manager.push();
        manager.push();
        
        assert_eq!(manager.depth(), 3);
        
        manager.clear();
        assert_eq!(manager.depth(), 0);
        assert!(manager.is_empty());
    }

    #[test]
    fn test_default_trait() {
        let manager: ScopeManager = Default::default();
        assert!(manager.is_empty());
    }
}
