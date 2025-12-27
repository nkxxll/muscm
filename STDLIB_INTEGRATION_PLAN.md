# Plan: Integrating Lua Stdlib into Scheme Interpreter

## Overview
The Lua stdlib implementation (Phase 6) provides a modular, function-based architecture for standard library functions. This plan outlines how to adapt this approach for the Scheme interpreter, accounting for Scheme's different value system and semantic differences.

## Current State

### Lua Stdlib Architecture
- **stdlib.rs**: Contains factory functions for each stdlib feature
  - Individual `create_*()` functions (e.g., `create_print()`, `create_math_abs()`)
  - Table builders for grouped functions (e.g., `create_string_table()`, `create_math_table()`)
  - Builtin procedures wrapped in `Rc<dyn Fn()>`
- **lua_interpreter.rs**: Initializes stdlib in `init_stdlib()` method
  - Registers globals and tables in constructor
  - Clean separation of concerns
- **lua_value.rs**: `LuaFunction::Builtin` variant for native functions

### Scheme Interpreter Architecture  
- **interpreter.rs**: Contains `Environment` struct with builtin registration
  - Builtins hardcoded in `Environment::new()`
  - `SVal::BuiltinProc` with name and optional arity
  - Direct matching in `apply_builtin()` method
- Currently 25+ builtin procedures

## Integration Strategy

### 1. **Create Scheme-specific stdlib module** (NEW)
**File**: `src/scheme_stdlib.rs`

Create Scheme-native implementations of common stdlib functions:
- **I/O**: display (already exists), newline (exists), read (new)
- **Type checking**: additional predicates needed
- **Numeric**: arithmetic, comparison (exists), math functions (new)
- **String**: string operations (new)
- **List**: list operations (most exist, improve)

**Architecture**:
```rust
// Individual function implementations
pub fn builtin_display(args: Vec<SVal>) -> Result<SVal, String>
pub fn builtin_string_length(args: Vec<SVal>) -> Result<SVal, String>
pub fn builtin_string_substring(args: Vec<SVal>) -> Result<SVal, String>
pub fn builtin_math_abs(args: Vec<SVal>) -> Result<SVal, String>
// ... etc

// Registration function
pub fn register_stdlib(env: &mut Environment)
```

### 2. **Refactor Environment initialization**
**File**: `src/interpreter.rs`

Restructure `Environment::new()`:

```rust
impl Environment {
    pub fn new() -> Self {
        let mut env = Environment {
            bindings: Vec::new(),
            parent: None,
        };
        
        // Register all builtins via stdlib module
        scheme_stdlib::register_stdlib(&mut env);
        
        env
    }
}
```

Move hardcoded builtins to `scheme_stdlib.rs`.

### 3. **Update apply_builtin() dispatch**
**File**: `src/interpreter.rs`

Convert from switch statement to function dispatch:

```rust
fn apply_builtin(name: &str, args: Vec<SVal>, env: &mut Environment) -> Result<SVal, String> {
    // Try to find in environment (allows user-defined overrides)
    match scheme_stdlib::call_builtin(name, args, env) {
        Ok(result) => Ok(result),
        Err(UnknownFunction) => Err(format!("Unknown function: {}", name)),
        Err(e) => Err(e),
    }
}
```

### 4. **Map Lua stdlib to Scheme equivalents**

| Category | Lua Function | Scheme Equivalent | Notes |
|----------|--------------|-------------------|-------|
| **I/O** | print | display + newline | Need formatting options |
| | io.read | read | Not yet implemented |
| | io.write | display | Similar to write |
| **Type** | type() | Special: return symbol | Return 'number, 'string, etc. |
| | tonumber | string->number | Type conversion |
| | tostring | number->string | Type conversion |
| **String** | string.len | string-length | Direct map |
| | string.sub | substring | Direct map (1-based to 0-based) |
| | string.upper | string-upcase | Rename convention |
| | string.lower | string-downcase | Rename convention |
| **Math** | math.abs | abs | Already have + |
| | math.floor | floor | New |
| | math.ceil | ceiling | New |
| | math.min | min | Already have |
| | math.max | max | Already have |
| | math.random | random | New |
| **Table** | table.insert | list operations | Use cons/append/set-car! |
| | table.remove | list operations | Use filter/drop |
| | pairs | map/for-each | Higher-order functions |
| | ipairs | Similar to pairs | For vectors |
| | next | Not applicable | Scheme uses for-each |

### 5. **Handle semantic differences**

#### 1-based vs 0-based indexing
**Lua**: Strings and tables are 1-indexed  
**Scheme**: Strings and lists are 0-indexed

```rust
// Lua string.sub(s, 1, 3) -> first 3 chars
// Scheme (substring s 0 3) -> first 3 chars
// Need conversion in wrapper functions
```

#### Table vs Association List
**Lua**: Uses hashtable-like tables  
**Scheme**: Typically uses lists or vectors

```rust
// May not implement table functions directly
// Instead, provide:
// - list operations (car, cdr, cons, etc. - already have)
// - vector operations (new)
// - association lists via pairs
```

#### Type system
**Lua**: type() returns string  
**Scheme**: Should return symbol

```rust
// (type 42) -> 'number
// (type "str") -> 'string
// (type #t) -> 'boolean
```

### 6. **Implementation phases**

#### Phase 6A: Core functions (Week 1)
- Refactor `Environment` to use `scheme_stdlib` module
- Move existing builtins to stdlib
- Add mathematical functions (abs, floor, ceiling, sqrt, sin, cos)
- Add string functions (string-length, substring, string-upcase, string-downcase)

**Test files**:
```
tests/stdlib_math.rs
tests/stdlib_string.rs
tests/stdlib_io.rs
```

#### Phase 6B: Advanced features (Week 2)
- Add type conversion functions (string->number, number->string)
- Improve type checking predicates
- Add higher-order functions (map, filter, fold)
- Add vector operations

#### Phase 6C: I/O and metadata (Week 3)
- Implement read/write operations
- Add reflection functions (procedure?, ...)
- Add equality predicates (eq?, eqv?, equal?)

### 7. **File structure**

```
src/
├── interpreter.rs       (modify: refactor Environment::new())
├── scheme_stdlib.rs     (new: all stdlib implementations)
└── ... (rest unchanged)

tests/
├── stdlib_math.rs       (new: math function tests)
├── stdlib_string.rs     (new: string function tests)
├── stdlib_io.rs         (new: I/O function tests)
└── stdlib_integration.rs (new: cross-module tests)
```

### 8. **Naming conventions**

Follow Scheme naming conventions:
- Predicates end with `?`: `number?`, `string?`, `list?`
- Procedures returning booleans: use `?` suffix
- No snake_case in Scheme (use hyphens): `string-length`, not `string_length`
- Type conversion: `->` for "to": `string->number`

### 9. **Error handling**

Both Lua and Scheme stdlib use error results:

```rust
// Match Lua's pattern but with Scheme errors
pub fn builtin_string_length(args: Vec<SVal>) -> Result<SVal, String> {
    if args.len() != 1 {
        return Err("string-length: expects exactly 1 argument".to_string());
    }
    match &args[0] {
        SVal::String(s) => Ok(SVal::Number(s.len() as f64)),
        _ => Err("string-length: expects a string".to_string()),
    }
}
```

### 10. **Testing strategy**

#### Unit tests (in scheme_stdlib.rs)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_string_length() { ... }
    #[test]
    fn test_math_abs() { ... }
}
```

#### Integration tests (in tests/ directory)
```rust
// tests/stdlib_integration.rs
#[test]
fn test_stdlib_in_interpreter() {
    let mut env = Environment::new();
    let result = Interpreter::eval(..., &mut env, ...);
    // Verify builtin works through full interpreter
}
```

#### Regression tests
Ensure all existing Phase 1-5 tests still pass

### 11. **Example: String-length implementation**

From Lua:
```rust
pub fn create_string_len() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, String>> {
    Rc::new(|args| {
        if args.is_empty() {
            return Err("string.len() requires 1 argument".to_string());
        }
        match &args[0] {
            LuaValue::String(s) => Ok(LuaValue::Number(s.len() as f64)),
            _ => Err(format!("string.len() expects a string, got {}", args[0].type_name())),
        }
    })
}
```

To Scheme:
```rust
pub fn builtin_string_length(args: Vec<SVal>) -> Result<SVal, String> {
    if args.len() != 1 {
        return Err("string-length: expects exactly 1 argument".to_string());
    }
    match &args[0] {
        SVal::String(s) => Ok(SVal::Number(s.len() as f64)),
        _ => Err(format!("string-length: expects a string, got {}", type_of(&args[0]))),
    }
}
```

Registration in Environment:
```rust
env.define("string-length".to_string(), 
    SVal::BuiltinProc {
        name: "string-length".to_string(),
        arity: Some(1),
    }
);
```

Dispatch in apply_builtin:
```rust
"string-length" => builtin_string_length(args),
```

## Success Criteria

1. ✅ All Lua stdlib functions have Scheme equivalents (or documented reasons they don't)
2. ✅ Existing Phase 1-5 tests all pass
3. ✅ New stdlib module provides 30+ builtin procedures
4. ✅ Code follows Scheme naming conventions
5. ✅ Error messages are consistent and helpful
6. ✅ Documentation updated with all stdlib functions
7. ✅ Test coverage >80% for new stdlib code

## Risks & Mitigation

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Breaking existing tests | High | Keep refactoring backward compatible; run tests after each change |
| Semantic mismatch (1-based indexing) | Medium | Document differences; provide helper functions for conversion |
| Complexity of modules | Medium | Keep stdlib functions small and focused; use helper functions |
| Performance regression | Low | Benchmark before/after; optimize hot paths |

## Timeline

- **Day 1-2**: Refactor Environment, create scheme_stdlib.rs skeleton
- **Day 3-5**: Implement math and string functions
- **Day 6-7**: Implement I/O and type functions
- **Day 8-9**: Testing and documentation
- **Day 10**: Integration and polish

## Next Phase (Phase 7)

- Advanced list operations (map, filter, fold)
- Macros and meta-programming support
- Module system (if applicable)
- Optimization and performance improvements
