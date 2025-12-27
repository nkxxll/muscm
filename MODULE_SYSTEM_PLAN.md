# Phase 9: Module System Implementation Plan

## Overview
Implement a module system that allows Lua code to import other Lua files via `require("<module>")`. This enables code organization, code reuse, and library development.

## Architecture

### 1. **Module Path Resolution**
**File:** `src/module_loader.rs` (new)

```rust
pub struct ModuleLoader {
    /// Search paths for modules (default: ['.', 'modules/'])
    search_paths: Vec<PathBuf>,
    /// Cached loaded modules to prevent reimporting
    loaded_modules: HashMap<String, LuaValue>,
}

impl ModuleLoader {
    /// Resolve a module name to a file path
    /// "mymodule" -> finds mymodule.lua in search paths
    /// "dir.subdir.module" -> finds dir/subdir/module.lua
    fn resolve_module(&self, module_name: &str) -> Result<PathBuf, String>
    
    /// Load and execute a module file
    fn load_module(&mut self, module_name: &str, executor: &mut Executor, interp: &mut LuaInterpreter) -> Result<LuaValue, String>
    
    /// Check if module already loaded (caching)
    fn is_cached(&self, module_name: &str) -> bool
    
    /// Add a search path for module discovery
    pub fn add_search_path(&mut self, path: PathBuf)
}
```

**Behavior:**
- Module name `"utils"` → searches for `./utils.lua`, `./modules/utils.lua`
- Module name `"config.server"` → searches for `./config/server.lua`
- Loaded modules are cached to avoid reimporting
- Returns error if module not found

### 2. **Core Require Function**
**File:** `src/stdlib.rs` (modified)

```rust
pub fn create_require(loader: Rc<RefCell<ModuleLoader>>) -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>>
```

**Behavior:**
- Takes module name as string argument
- Calls module loader to find and execute module file
- Module is executed in isolated scope (new environment)
- Module's returned value (or module.exports table) is returned
- Caches result so subsequent requires return same module object

### 3. **Module Execution Context**
**Design Decisions:**

1. **Module Scope Isolation**
   - Each module gets its own scope
   - Module code can access global stdlib functions
   - Module can define `exports` table or return values
   - Global scope modifications don't affect other modules

2. **Module Return Values**
   - Modules can return values (like CommonJS)
   - Or populate a global `exports` table
   - If module returns nothing, return the `exports` table

3. **Module.exports pattern**
   ```lua
   -- In mymodule.lua
   local exports = {}
   exports.foo = function() return "bar" end
   return exports
   
   -- In main.lua
   local m = require("mymodule")
   print(m.foo())  -- prints "bar"
   ```

### 4. **Integration with LuaInterpreter**
**File:** `src/lua_interpreter.rs` (modified)

Add module loader to interpreter:
```rust
pub struct LuaInterpreter {
    // ... existing fields ...
    
    /// Module loader for require() functionality
    pub module_loader: Rc<RefCell<ModuleLoader>>,
}

impl LuaInterpreter {
    pub fn new() -> Self {
        let mut loader = ModuleLoader::new();
        // Add default search paths
        loader.add_search_path(PathBuf::from("."));
        loader.add_search_path(PathBuf::from("modules"));
        loader.add_search_path(PathBuf::from("lib"));
        
        // ... initialize other fields ...
    }
    
    /// Add a custom search path for modules
    pub fn add_module_search_path(&mut self, path: PathBuf)
}
```

### 5. **Executor Enhancement**
**File:** `src/executor.rs` (modified)

Modify to support module loading without cycles:
```rust
impl Executor {
    /// Execute a require() call safely
    fn execute_require(
        &mut self,
        module_name: &str,
        interp: &mut LuaInterpreter,
    ) -> Result<LuaValue, String> {
        // Delegate to module loader
        interp.module_loader.borrow_mut().load_module(module_name, self, interp)
    }
}
```

### 6. **Handling Circular Dependencies**
**Strategy:** Load-then-execute with partial module objects

1. When require() is called, immediately add module to "loading" cache
2. Execute module code with reference to loading module
3. If module A requires B, and B requires A, B gets partial module state
4. Once loading complete, module is marked as "loaded"

```rust
enum ModuleState {
    Loading(Rc<RefCell<LuaValue>>),  // Placeholder table
    Loaded(LuaValue),                 // Final module value
}
```

## Implementation Steps

### Phase 1: Module Loader Infrastructure (2-3 hours)
1. Create `src/module_loader.rs`
2. Implement `ModuleLoader` struct
3. Implement path resolution logic (handle `.` notation)
4. Add caching mechanism
5. Write unit tests

**Tests:**
- Test path resolution (simple names, dot notation)
- Test caching (same module not reloaded)
- Test missing modules (error handling)

### Phase 2: Parser Integration (1 hour)
1. Verify parser already handles `require()` as function calls
2. No parser changes needed (it's just a function)

### Phase 3: Require Function (2 hours)
1. Create `create_require()` in stdlib
2. Integrate with module loader
3. Handle return value extraction
4. Add to global scope

**Tests:**
- Basic require of simple module
- Module with exports table
- Module with explicit return
- Caching verification

### Phase 4: Module Execution & Scope (2-3 hours)
1. Create isolated scope for module execution
2. Parse and execute module file content
3. Handle module-local variables
4. Extract return value or exports

**Tests:**
- Module scope isolation
- Global access from modules
- Return vs exports patterns
- Module-level variable scoping

### Phase 5: Circular Dependency Handling (2-3 hours)
1. Implement loading state tracking
2. Handle A → B → A cycles
3. Test with actual circular modules

**Tests:**
- Simple circular dependency (A ↔ B)
- Multi-level circular (A → B → C → A)
- Partial module access during loading

### Phase 6: Error Handling & Edge Cases (1-2 hours)
1. Module not found errors
2. Parse errors in module
3. Runtime errors in module initialization
4. Syntax errors

**Tests:**
- Missing modules
- Syntax errors in modules
- Runtime errors during require
- Permission denied

### Phase 7: Documentation & Examples (1 hour)
1. Create module system documentation
2. Write example modules
3. Create test fixtures

## File Structure

```
src/
├── module_loader.rs     (NEW: ~300 lines)
│   ├── ModuleLoader struct
│   ├── Module path resolution
│   ├── Module caching
│   └── Circular dependency handling
├── stdlib.rs            (MODIFIED: +30 lines)
│   └── create_require() function
├── lua_interpreter.rs   (MODIFIED: +20 lines)
│   └── Module loader integration
├── executor.rs          (MODIFIED: +15 lines)
│   └── Module execution support
└── lib.rs               (MODIFIED: +1 line)
    └── Declare module_loader module

fixtures/
├── modules/             (NEW: test modules)
│   ├── simple.lua
│   ├── config.lua
│   ├── utils/
│   │   ├── math.lua
│   │   └── string.lua
│   └── circular_a.lua
│       circular_b.lua
└── test_require_*.lua   (integration tests)
```

## Example Usage

### Example 1: Simple Module Export
```lua
-- modules/utils.lua
local exports = {}

function exports.add(a, b)
    return a + b
end

function exports.multiply(a, b)
    return a * b
end

return exports

-- main.lua
local utils = require("utils")
print(utils.add(2, 3))          -- 5
print(utils.multiply(4, 5))     -- 20
```

### Example 2: Configuration Module
```lua
-- config.lua
return {
    host = "localhost",
    port = 8080,
    debug = true,
    db = {
        name = "mydb",
        user = "admin"
    }
}

-- main.lua
local config = require("config")
print("Server: " .. config.host .. ":" .. config.port)
```

### Example 3: Nested Module Path
```lua
-- utils/math.lua
local exports = {}
exports.abs = function(n) return n < 0 and -n or n end
exports.sign = function(n) return n < 0 and -1 or (n > 0 and 1 or 0) end
return exports

-- main.lua
local math_utils = require("utils.math")
print(math_utils.abs(-5))  -- 5
```

### Example 4: Module with Dependencies
```lua
-- models/user.lua
local validation = require("utils.validation")
local db = require("services.database")

local User = {}

function User:validate()
    return validation.is_string(self.name)
end

function User:save()
    db.insert("users", self)
end

return User

-- main.lua
local User = require("models.user")
local user = {name = "Alice"}
if user:validate() then
    user:save()
end
```

## Testing Strategy

### Unit Tests (in `module_loader.rs`)
```rust
#[cfg(test)]
mod tests {
    // Path resolution tests
    test_resolve_simple_module()      // "utils" -> "./utils.lua"
    test_resolve_dotted_module()      // "utils.math" -> "./utils/math.lua"
    test_missing_module()             // Error on not found
    
    // Caching tests
    test_module_cached_on_reload()    // Second require returns cached
    test_module_cache_isolation()     // Different modules have different caches
    
    // Circular dependency tests
    test_circular_dependency_ab()     // A requires B, B requires A
    test_circular_dependency_abc()    // A->B->C->A
}
```

### Integration Tests
```lua
-- tests/require_simple.lua
local m = require("simple_module")
assert(m.value == 42)

-- tests/require_exports.lua
local m = require("module_with_exports")
assert(type(m) == "table")
assert(m.func() == "result")

-- tests/require_circular.lua
local a = require("circular_a")
local b = require("circular_b")
assert(a.b_ref == b)
assert(b.a_ref == a)
```

## Future Enhancements (Phase 10+)

1. **Module Metadata**
   - `module.name` - module name
   - `module.version` - version string
   - `module.dependencies` - list of dependencies

2. **Module.export Alternative**
   - Support setting `module.exports` directly
   - Support exports as global

3. **Package System**
   - `package.path` - Lua module search path
   - `package.cpath` - C module search path
   - `package.loaded` - table of loaded modules

4. **Library Search Paths**
   - Environment variable `LUA_PATH`
   - System library directories
   - Local library dependencies

5. **Module Metadata & Version**
   - Simple version checking
   - Module info/metadata
   - Dependency declaration

6. **Lua Package Manager (Luarocks-like)**
   - Dependency resolution
   - Package installation
   - Version compatibility

## Success Criteria

✓ Module loading via `require("name")`
✓ Module path resolution (`.` notation → `/` paths)
✓ Module caching (same module not reloaded)
✓ Module scope isolation (local variables per module)
✓ Return value & exports table support
✓ Circular dependency handling
✓ Error reporting for missing modules
✓ Integration tests with multiple modules
✓ Documentation with examples

## Timeline
- Total estimated time: 12-16 hours
- Best implemented in 2-3 sessions of 4-6 hours each
- Start with Phase 1-2 (infrastructure + require function)
- Then Phase 3-4 (execution context)
- Finally Phase 5-7 (circular deps, errors, docs)
