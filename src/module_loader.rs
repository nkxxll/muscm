/// Phase 9: Module System
/// 
/// This module implements a module loading system for Lua code.
/// Allows code organization and reuse via `require("<module>")`.

use crate::lua_value::LuaValue;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;



/// Manages module loading and caching
pub struct ModuleLoader {
    /// Search paths for modules (e.g., ['.', 'modules/', 'lib/'])
    pub search_paths: Vec<PathBuf>,
    /// Cache of loaded modules
    pub loaded_modules: HashMap<String, LuaValue>,
    /// Tracks modules currently being loaded (for circular dependency detection)
    pub loading: HashSet<String>,
}

impl ModuleLoader {
    /// Create a new module loader with default search paths
    pub fn new() -> Self {
        ModuleLoader {
            search_paths: vec![
                PathBuf::from("."),
                PathBuf::from("modules"),
                PathBuf::from("lib"),
            ],
            loaded_modules: HashMap::new(),
            loading: HashSet::new(),
        }
    }

    /// Add a search path for module discovery
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Resolve a module name to a file path
    /// 
    /// "mymodule" → finds mymodule.lua in search paths
    /// "config.server" → finds config/server.lua in search paths
    pub fn resolve_module(&self, module_name: &str) -> Result<PathBuf, String> {
        // Convert dot notation to path notation
        let path_part = module_name.replace('.', "/");
        let filename = format!("{}.lua", path_part);

        for search_path in &self.search_paths {
            let full_path = search_path.join(&filename);
            if full_path.exists() && full_path.is_file() {
                return Ok(full_path);
            }
        }

        Err(format!("Module not found: {}", module_name))
    }

    /// Check if a module is already cached
    pub fn is_cached(&self, module_name: &str) -> bool {
        self.loaded_modules.contains_key(module_name)
    }



    /// Clear the module cache
    pub fn clear_cache(&mut self) {
        self.loaded_modules.clear();
        self.loading.clear();
    }

    /// Get number of cached modules
    pub fn cached_count(&self) -> usize {
        self.loaded_modules.len()
    }
}

impl Default for ModuleLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_loader_creation() {
        let loader = ModuleLoader::new();
        assert_eq!(loader.search_paths.len(), 3);
        assert!(loader.loaded_modules.is_empty());
        assert!(loader.loading.is_empty());
    }

    #[test]
    fn test_add_search_path() {
        let mut loader = ModuleLoader::new();
        loader.add_search_path(PathBuf::from("custom"));
        assert_eq!(loader.search_paths.len(), 4);
    }

    #[test]
    fn test_resolve_simple_module() {
        let loader = ModuleLoader::new();
        // This will fail since file doesn't exist, but shows the logic
        let result = loader.resolve_module("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_dotted_module() {
        let loader = ModuleLoader::new();
        // Test path resolution logic (file won't exist)
        let result = loader.resolve_module("config.server");
        assert!(result.is_err());
        // Should have tried paths like config/server.lua
    }

    #[test]
    fn test_is_cached() {
        let mut loader = ModuleLoader::new();
        assert!(!loader.is_cached("mymodule"));

        loader.loaded_modules.insert(
            "mymodule".to_string(),
            LuaValue::Nil,
        );
        assert!(loader.is_cached("mymodule"));
    }

    #[test]
    fn test_clear_cache() {
        let mut loader = ModuleLoader::new();
        loader.loaded_modules.insert(
            "module1".to_string(),
            LuaValue::Number(42.0),
        );
        loader.loading.insert("module2".to_string());

        loader.clear_cache();
        assert!(loader.loaded_modules.is_empty());
        assert!(loader.loading.is_empty());
    }
}
