# Module System - Quick Reference & Checklist

## Implementation Checklist

### Phase 1: Module Loader Infrastructure
- [ ] Create `src/module_loader.rs`
- [ ] Implement `ModuleLoader` struct with fields:
  - `search_paths: Vec<PathBuf>` (default: ['.', 'modules/', 'lib/'])
  - `loaded_modules: HashMap<String, LuaValue>`
  - `loading: HashSet<String>` (for circular dependency detection)
- [ ] Implement `resolve_module(&self, name) -> Result<PathBuf>`
  - "utils" → try "./utils.lua", "./modules/utils.lua", "./lib/utils.lua"
  - "config.server" → try "./config/server.lua", "./modules/config/server.lua"
- [ ] Implement `is_cached(&self, name) -> bool`
- [ ] Implement `add_search_path(&mut self, path)`
- [ ] Add unit tests

**Key Logic:**
```rust
fn resolve_module(&self, module_name: &str) -> Result<PathBuf> {
    let path_part = module_name.replace('.', "/");
    for search_path in &self.search_paths {
        let full_path = search_path.join(&format!("{}.lua", path_part));
        if full_path.exists() {
            return Ok(full_path);
        }
    }
    Err(format!("Module not found: {}", module_name))
}
```

### Phase 2: Require Function in Stdlib
- [ ] Add `create_require(loader: Rc<RefCell<ModuleLoader>>)` to `src/stdlib.rs`
- [ ] Function signature:
  ```rust
  pub fn create_require(loader: Rc<RefCell<ModuleLoader>>) 
    -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>>
  ```
- [ ] Implementation:
  1. Get module name from args (validate it's a string)
  2. Call `loader.borrow_mut().load_module(name, ...)`
  3. Return loaded module value
  4. Handle errors: "Module not found", "Cannot load module"

**Key Logic:**
```rust
pub fn create_require(loader) -> Rc<dyn Fn(...)> {
    Rc::new(move |args| {
        if args.is_empty() { return Err("require() needs module name".into()); }
        let module_name = match &args[0] {
            LuaValue::String(s) => s.clone(),
            _ => return Err("Module name must be string".into()),
        };
        
        loader.borrow_mut().load_module(&module_name)
    })
}
```

### Phase 3: Module Execution Context
- [ ] Modify `ModuleLoader::load_module()` to:
  1. Check if module is cached → return cached
  2. Check if module is currently loading → return partial object (for circular deps)
  3. Mark module as "loading"
  4. Read module file content
  5. Parse content using Lua parser
  6. Create new scope for module execution
  7. Execute module code in that scope
  8. Extract return value OR `exports` table
  9. Cache result
  10. Mark as "loaded"
  11. Return module value

**Key Logic:**
```rust
pub fn load_module(&mut self, name: &str, executor: &mut Executor, 
                   interp: &mut LuaInterpreter) -> Result<LuaValue, String> {
    // Check cache
    if let Some(cached) = self.loaded_modules.get(name) {
        return Ok(cached.clone());
    }
    
    // Resolve path
    let path = self.resolve_module(name)?;
    
    // Read file
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Cannot read module: {}", e))?;
    
    // Parse (reuse existing parser)
    let ast = crate::lua_parser::parse(&content)
        .map_err(|e| format!("Parse error in {}: {}", name, e))?;
    
    // Execute in isolated scope
    interp.push_scope();
    let control_flow = executor.execute_block(&ast.block, interp)?;
    interp.pop_scope();
    
    // Extract result
    let result = match control_flow {
        ControlFlow::Return(values) if !values.is_empty() => values[0].clone(),
        _ => {
            // Return exports table if no return value
            interp.lookup("exports").unwrap_or(LuaValue::Nil)
        }
    };
    
    // Cache and return
    self.loaded_modules.insert(name.to_string(), result.clone());
    Ok(result)
}
```

### Phase 4: Integration with LuaInterpreter
- [ ] Add to `src/lua_interpreter.rs`:
  ```rust
  pub struct LuaInterpreter {
      // ... existing fields ...
      pub module_loader: Rc<RefCell<ModuleLoader>>,
  }
  ```
- [ ] Modify `new()` to initialize module loader:
  ```rust
  let mut loader = ModuleLoader::new();
  loader.add_search_path(".".into());
  loader.add_search_path("modules".into());
  loader.add_search_path("lib".into());
  ```
- [ ] Register `require` in `init_stdlib()`:
  ```rust
  let require_fn = crate::stdlib::create_require(
      Rc::clone(&self.module_loader)
  );
  self.define("require", 
      LuaValue::Function(Rc::new(LuaFunction::Builtin(require_fn))));
  ```
- [ ] Add `add_module_search_path(&mut self, path)` method

### Phase 5: Circular Dependency Handling
- [ ] Add `loading: HashSet<String>` to `ModuleLoader`
- [ ] Before parsing, add to "loading" set
- [ ] If module already in "loading", return partial module object
- [ ] After execution, remove from "loading" set
- [ ] Create partial module as empty table that can be populated

**Key Logic:**
```rust
fn load_module(&mut self, name: &str, ...) -> Result<LuaValue, String> {
    if self.loaded_modules.contains_key(name) {
        return Ok(self.loaded_modules[name].clone());
    }
    
    // Circular dependency: return empty table
    if self.loading.contains(name) {
        let partial = LuaValue::Table(Rc::new(RefCell::new(LuaTable {
            data: HashMap::new(),
            metatable: None,
        })));
        return Ok(partial);
    }
    
    self.loading.insert(name.to_string());
    
    // ... execute module ...
    
    self.loading.remove(name);
    self.loaded_modules.insert(name.to_string(), result.clone());
    Ok(result)
}
```

### Phase 6: Error Handling
- [ ] Module not found → clear error message
- [ ] Parse error in module → include module name in error
- [ ] Runtime error during require → propagate with context
- [ ] Type errors (non-string module name) → helpful message
- [ ] Tests for each error case

### Phase 7: Test Fixtures & Examples
- [ ] Create `fixtures/modules/` directory
- [ ] Create test modules:
  ```
  fixtures/modules/
  ├── simple.lua           # Single function
  ├── config.lua           # Configuration table
  ├── utils.lua            # Utility functions
  ├── utils/
  │   ├── math.lua         # Nested module
  │   └── string.lua
  ├── circular_a.lua       # Circular deps
  ├── circular_b.lua
  └── with_error.lua       # Error handling test
  ```

## Key Data Structures

### ModuleLoader
```rust
pub struct ModuleLoader {
    pub search_paths: Vec<PathBuf>,
    pub loaded_modules: HashMap<String, LuaValue>,
    pub loading: HashSet<String>,  // For circular dependency detection
}
```

### Module States
- `NotFound` → Error
- `Loading` → In progress (for circular deps)
- `Loaded` → Cached result available
- `Error` → Failed (cached error?)

## API Examples

### User API
```lua
-- Basic require
local utils = require("utils")

-- Dotted paths
local math = require("utils.math")

-- Module with explicit exports
local config = require("config")
```

### Builtin require() function
- Takes: module name (string)
- Returns: module value (whatever module returns or exports table)
- Errors on: missing module, parse error, runtime error in module

## Search Path Resolution

### Default search paths (in order)
1. `.` (current directory)
2. `modules/` (modules subdirectory)
3. `lib/` (lib subdirectory)

### How names are resolved
```
require("utils")          → "./utils.lua"
                          → "./modules/utils.lua"
                          → "./lib/utils.lua"

require("utils.math")     → "./utils/math.lua"
                          → "./modules/utils/math.lua"
                          → "./lib/utils/math.lua"

require("a.b.c")          → "./a/b/c.lua"
                          → "./modules/a/b/c.lua"
                          → "./lib/a/b/c.lua"
```

## Module Patterns

### Pattern 1: Return Table
```lua
-- mymodule.lua
return {
    foo = function() return "bar" end,
    value = 42,
}
```

### Pattern 2: Exports Table
```lua
-- mymodule.lua
local exports = {}
function exports.foo() return "bar" end
exports.value = 42
return exports
```

### Pattern 3: Global Exports (Not recommended but possible)
```lua
-- mymodule.lua
exports = {}
exports.foo = function() return "bar" end
return exports
```

### Pattern 4: Bare Functions (returns immediately)
```lua
-- mymodule.lua
return function()
    return "result"
end
```

## Testing Strategy

### Unit Tests (module_loader.rs)
- Path resolution for simple names
- Path resolution for dotted names
- Module caching verification
- Missing module error handling
- Circular dependency detection

### Integration Tests (fixtures/)
- Load simple module
- Load module with exports
- Load nested module (dotted path)
- Circular dependency (A↔B)
- Module not found error
- Parse error in module

## Common Issues & Solutions

### Issue: Module loaded multiple times
**Solution:** ModuleLoader caches modules in HashMap

### Issue: Circular requires cause infinite loop
**Solution:** Track "loading" modules, return partial object

### Issue: Module can't find relative modules
**Solution:** Add search paths, use dotted notation

### Issue: Module scope pollutes globals
**Solution:** Create isolated scope for each module execution

## Implementation Notes

1. **Scope Isolation**: Use `interp.push_scope()` / `interp.pop_scope()` to isolate module
2. **Caching**: Store in HashMap with module name as key
3. **Circular Dependencies**: Use HashSet to track modules currently loading
4. **Path Resolution**: Always try all search paths in order
5. **Error Context**: Include module name in all error messages
6. **Parser Reuse**: Use existing `lua_parser::parse()` for module content

## Validation Checklist Before Testing

- [ ] Compilation passes without errors
- [ ] All unit tests in module_loader pass
- [ ] require() is registered as global function
- [ ] Default search paths include ".", "modules/", "lib/"
- [ ] Path resolution handles both simple and dotted names
- [ ] Module caching prevents reimporting same module
- [ ] Circular dependencies return partial modules
- [ ] Error messages include module names
- [ ] Documentation is complete
