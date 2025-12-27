# Module System - Quick Start Guide

## Basic Usage

### Loading a Module
```lua
local mymodule = require("mymodule")
```

### Module Search Paths
The module loader searches for `mymodule.lua` in these locations (in order):
1. `./mymodule.lua`
2. `./modules/mymodule.lua`
3. `./lib/mymodule.lua`

### Nested Modules
Use dot notation for nested modules:
```lua
local config = require("config.server")
```

This searches for `config/server.lua` in the search paths.

## Creating Modules

### Pattern 1: Return a Table
```lua
-- mymodule.lua
return {
    greet = function(name)
        return "Hello, " .. name
    end,
    version = "1.0"
}
```

Usage:
```lua
local m = require("mymodule")
print(m.greet("Alice"))   -- prints: Hello, Alice
print(m.version)          -- prints: 1.0
```

### Pattern 2: Use Exports Table
```lua
-- config.lua
local exports = {}

exports.host = "localhost"
exports.port = 8080

function exports.get_url()
    return "http://" .. exports.host .. ":" .. exports.port
end

return exports
```

Usage:
```lua
local cfg = require("config")
print(cfg.host)         -- prints: localhost
print(cfg.get_url())    -- prints: http://localhost:8080
```

## Module Behavior

### Caching
Modules are loaded once and cached. Subsequent requires return the same module object:

```lua
local m1 = require("utils")
local m2 = require("utils")
-- m1 and m2 are the same object
```

### Circular Dependencies
If module A requires B, and B requires A, the system handles it gracefully:
- When A is being loaded and tries to load B
- B sees that A is being loaded and gets an empty table
- This prevents infinite loops

### Module Scope
Each module executes in its own local scope:
- Module variables don't leak to caller
- Module can access global stdlib functions
- Clean isolation between modules

## Advanced Configuration

### Adding Custom Search Paths
In Rust:
```rust
let mut interp = LuaInterpreter::new();
interp.add_module_search_path(PathBuf::from("custom_lib"));
interp.add_module_search_path(PathBuf::from("/usr/local/lua"));
```

The module loader will now search these paths first.

## Examples

### Example 1: Math Library
```lua
-- math_utils.lua
local M = {}

M.square = function(x)
    return x * x
end

M.cube = function(x)
    return x * x * x
end

return M
```

```lua
-- main.lua
local math = require("math_utils")
print(math.square(5))   -- 25
print(math.cube(3))     -- 27
```

### Example 2: Database Connection
```lua
-- db.lua
local db = {}
local connection = nil

function db.connect(host, port)
    connection = {host = host, port = port}
    print("Connected to " .. host .. ":" .. port)
end

function db.is_connected()
    return connection ~= nil
end

function db.disconnect()
    connection = nil
end

return db
```

```lua
-- app.lua
local db = require("db")
db.connect("localhost", 5432)
print(db.is_connected())  -- true
db.disconnect()
print(db.is_connected())  -- false
```

### Example 3: Nested Utilities
```lua
-- utils/string.lua
local S = {}

S.trim = function(str)
    return str:match("^%s*(.-)%s*$")
end

S.split = function(str, delim)
    local result = {}
    for part in str:gmatch("[^" .. delim .. "]+") do
        table.insert(result, part)
    end
    return result
end

return S
```

```lua
-- app.lua
local string_utils = require("utils.string")
local trimmed = string_utils.trim("  hello world  ")
print(trimmed)  -- "hello world"

local words = string_utils.split("one,two,three", ",")
-- words = {"one", "two", "three"}
```

## Common Patterns

### Initialization Module
```lua
-- init.lua
local init = {}

function init.setup()
    print("Setting up application...")
    -- initialization code
end

function init.cleanup()
    print("Cleaning up...")
    -- cleanup code
end

return init
```

### Configuration Module
```lua
-- config.lua
local config = {
    debug = true,
    max_retries = 3,
    timeout = 30
}

function config.get(key)
    return config[key]
end

function config.set(key, value)
    config[key] = value
end

return config
```

### Utility Module
```lua
-- util.lua
local util = {}

util.merge_tables = function(t1, t2)
    local result = {}
    for k, v in pairs(t1) do
        result[k] = v
    end
    for k, v in pairs(t2) do
        result[k] = v
    end
    return result
end

util.filter = function(tbl, predicate)
    local result = {}
    for _, v in ipairs(tbl) do
        if predicate(v) then
            table.insert(result, v)
        end
    end
    return result
end

return util
```

## Troubleshooting

### Module Not Found
```
Module not found: mymodule
```
- Check that `mymodule.lua` exists in one of the search paths
- Use `./` prefix if the file is in the current directory
- Check spelling and case sensitivity

### Parse Error in Module
```
Parse error in module 'mymodule': ...
```
- Check for syntax errors in the module file
- Ensure the module returns a value (table or function)

### Runtime Error in Module
```
Runtime error in module 'mymodule': ...
```
- The module code has an error when executed
- Check for undefined variables or invalid function calls
- Use `pcall()` for error handling in modules

## Best Practices

1. **Organize Related Code**: Group related functions in a module
   ```lua
   -- utils/array.lua
   local M = {}
   M.reverse = function(t) ... end
   M.flatten = function(t) ... end
   return M
   ```

2. **Use Meaningful Names**: Choose clear, descriptive module names
   ```lua
   local user_service = require("services.user")
   ```

3. **Document Exports**: Comment what the module exports
   ```lua
   -- config.lua
   -- Exports: host (string), port (number), get_url() -> string
   ```

4. **Minimize Global Impact**: Keep modules self-contained
   ```lua
   local exports = {}
   -- Don't modify global tables unless necessary
   return exports
   ```

5. **Use Local Variables**: Avoid polluting module scope
   ```lua
   local helper = function() ... end  -- not exported
   exports.public_func = function() ... end
   ```

## See Also
- [MODULE_SYSTEM_PLAN.md](./MODULE_SYSTEM_PLAN.md) - Detailed architecture
- [PHASE_9_SUMMARY.md](./PHASE_9_SUMMARY.md) - Implementation summary
