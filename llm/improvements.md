# Lua Interpreter Code Improvements Plan

## Overview
This document outlines a prioritized roadmap for refactoring and improving the muscm Lua interpreter codebase. Current metrics: 11,240 lines of Rust, 216 passing tests, 9 modules.

## Phase 1: Error Handling Overhaul (High Priority)

### 1.1 Replace String Errors with Error Enum
**Current State:** All error handling uses `Result<T, String>`
**Problem:** Loss of error context, difficult to categorize, no source location tracking

**Tasks:**
- [ ] Create `src/error_types.rs` with comprehensive error enum:
  ```rust
  pub enum LuaError {
      ParseError { message: String, line: usize, column: usize },
      RuntimeError { message: String, context: String },
      TypeError { expected: String, got: String, function: String },
      ValueError { message: String },
      FileError { path: String, reason: String },
      ModuleError { module: String, reason: String },
  }
  ```
- [ ] Implement Display and Error traits
- [ ] Update all Result types: `Result<T, String>` → `Result<T, LuaError>`
- [ ] Add line/column tracking in tokenizer
- [ ] Propagate location info through parser
- [ ] Update executor error returns to include context
- [ ] Add error tests covering each variant

**Files to Modify:**
- Create: `src/error_types.rs`
- Update: All `src/*.rs` files
- Add tests: `tests/error_handling.rs`

**Benefits:**
- Better error messages with location info
- Type-safe error handling
- Easier debugging
- Path for future error recovery strategies

---

## Phase 2: Split Monolithic Executor (High Priority)

**Current State:** `executor.rs` is 2,405 lines
**Problem:** Hard to navigate, test individual components, understand responsibilities

### 2.1 Create Operator Module (`src/executor/operators.rs`)
**Scope:** ~250 lines
- [ ] Extract `eval_binary_op()` → `BinaryOpEvaluator::eval()`
- [ ] Extract `apply_binary_op()` → `BinaryOpEvaluator::apply()`
- [ ] Extract `eval_unary_op()` → `UnaryOpEvaluator::eval()`
- [ ] Move string concatenation logic to dedicated function
- [ ] Add operator precedence constants/helpers
- [ ] Create BinaryOpEvaluator struct with clear responsibilities

**Tests:**
- [ ] Binary operator tests (arithmetic, comparison, logical, bitwise)
- [ ] Unary operator tests (negation, not, bitwise not, length)
- [ ] Type coercion tests
- [ ] Short-circuit evaluation tests

### 2.2 Create Table Operations Module (`src/executor/tables.rs`)
**Scope:** ~150 lines
- [ ] Extract `table_get()`, `table_set()`, `create_table()`
- [ ] Create TableOperations struct
- [ ] Add metatable operation helpers
- [ ] Implement table iteration helpers

**Tests:**
- [ ] Table creation with various field types
- [ ] Table indexing (numeric, string, complex keys)
- [ ] Nested table operations
- [ ] Metatable interactions

### 2.3 Create Function Module (`src/executor/functions.rs`)
**Scope:** ~200 lines
- [ ] Extract `call_function()` into FunctionCaller struct
- [ ] Extract `create_function()` into FunctionFactory struct
- [ ] Add argument binding logic
- [ ] Handle varargs expansion
- [ ] Implement closure capture improvements

**Tests:**
- [ ] Function calls with various argument counts
- [ ] Return value handling
- [ ] Closure variable capture
- [ ] Varargs handling
- [ ] Method calls

### 2.4 Create Module Loader Integration (`src/executor/require.rs`)
**Scope:** ~100 lines
- [ ] Extract `execute_require()` to RequireHandler struct
- [ ] Clean up module loading error messages
- [ ] Add require caching verification

**Tests:**
- [ ] Module loading workflow
- [ ] Circular dependency detection
- [ ] Cache hits/misses
- [ ] Module not found errors

### 2.5 Refactor Executor Core (`src/executor.rs`)
**Scope:** ~200 lines
- [ ] Keep core execution flow: `execute_block()`, `execute_statement()`
- [ ] Create clean delegation to sub-modules
- [ ] Implement Executor as a coordinator

**Before/After:** 2405 lines → 200 core + 700 specialized

**Structure:**
```
executor/
  ├── mod.rs (coordinator, re-exports)
  ├── operators.rs (binary/unary ops)
  ├── tables.rs (table operations)
  ├── functions.rs (call/create)
  └── require.rs (module loading)
```

---

## Phase 3: Standard Library Refactoring (Medium Priority)

**Current State:** `stdlib.rs` is 789 lines with high code duplication
**Problem:** 
- Repetitive argument validation (20+ similar error checks)
- Duplicate type conversion logic across functions
- Inconsistent error messages
- Hard to extend with new stdlib functions
- No clear separation of concerns

**Metrics:**
- ~150 lines of duplicated validation code
- ~8 different error message patterns
- 50+ builtin functions mixed in one file

### 3.1 Create Argument Validation Module (`src/stdlib/validation.rs`)
**Scope:** ~80 lines

```rust
/// Validate argument count with optional bounds
pub fn require_args(name: &str, args: &[LuaValue], min: usize, max: Option<usize>) -> Result<(), LuaError> {
    if args.len() < min {
        return Err(LuaError::ArgumentError { 
            function: name.to_string(),
            expected: format!("at least {}", min),
            got: args.len(),
        });
    }
    if let Some(max) = max {
        if args.len() > max {
            return Err(LuaError::ArgumentError { 
                function: name.to_string(),
                expected: format!("at most {}", max),
                got: args.len(),
            });
        }
    }
    Ok(())
}

/// Require specific type for argument
pub fn require_type(name: &str, index: usize, arg: &LuaValue, expected: &str) -> Result<(), LuaError> {
    if arg.type_name() != expected {
        return Err(LuaError::TypeError {
            function: name.to_string(),
            position: index + 1,  // 1-based for error reporting
            expected: expected.to_string(),
            got: arg.type_name().to_string(),
        });
    }
    Ok(())
}

/// Extract number with type checking
pub fn get_number(name: &str, index: usize, arg: &LuaValue) -> Result<f64, LuaError> {
    match arg {
        LuaValue::Number(n) => Ok(*n),
        _ => Err(LuaError::TypeError {
            function: name.to_string(),
            position: index + 1,
            expected: "number".to_string(),
            got: arg.type_name().to_string(),
        }),
    }
}

/// Extract string with type checking
pub fn get_string(name: &str, index: usize, arg: &LuaValue) -> Result<String, LuaError> {
    match arg {
        LuaValue::String(s) => Ok(s.clone()),
        _ => Err(LuaError::TypeError { ... }),
    }
}

/// Extract table with type checking
pub fn get_table(name: &str, index: usize, arg: &LuaValue) 
    -> Result<Rc<RefCell<LuaTable>>, LuaError> {
    match arg {
        LuaValue::Table(t) => Ok(t.clone()),
        _ => Err(LuaError::TypeError { ... }),
    }
}
```

**Current Duplication Examples (to be replaced):**
- Lines 54-55: `type()` arg check
- Lines 109-112: `string.len()` arg + type check
- Lines 124-127: `string.sub()` arg count check
- Lines 178-180: `string.upper()` arg + type check
- Lines 203-206: `math.abs()` arg + type check
- Pattern repeats 40+ more times

**Benefit:** Eliminates ~150 lines of duplicated boilerplate.

### 3.2 Organize Stdlib into Submodules ✅ COMPLETE
**Status:** DONE - Stdlib module successfully reorganized into focused submodules
**Implementation:**
```
stdlib/
  ├── mod.rs (97 lines - public API, function registration, re-exports)
  ├── validation.rs (153 lines - arg validation helpers)
  ├── string.rs (114 lines - string.len, sub, upper, lower)
  ├── math.rs (141 lines - math.abs, floor, ceil, min, max, random)
  ├── table.rs (100 lines - table.insert, remove)
  ├── types.rs (59 lines - type, tonumber, tostring)
  ├── metatables.rs (170 lines - setmetatable, getmetatable, pcall, xpcall, error, coroutine)
  └── iterators.rs (59 lines - pairs, ipairs, next)
```

**Completed Tasks:**
- [x] Created `src/stdlib/string.rs` with all string functions
- [x] Created `src/stdlib/math.rs` with all math functions
- [x] Created `src/stdlib/table.rs` with table operations
- [x] Created `src/stdlib/types.rs` with type conversion functions
- [x] Created `src/stdlib/iterators.rs` with iterator functions
- [x] Created `src/stdlib/metatables.rs` with metatable/error/coroutine functions
- [x] Refactored `src/stdlib/mod.rs` to coordinate submodules with re-exports
- [x] Maintained backward compatibility via public re-exports
- [x] All 227 tests passing after refactoring
- [x] Zero compilation errors

**Results:**
- Reduced mod.rs from 709 lines → 97 lines (86% reduction)
- 8 focused modules instead of 1 monolithic file
- Each module has clear single responsibility
- Easier to extend with new stdlib functions
- Public API unchanged (seamless integration)

### 3.3 Replace Error Strings with LuaError Usage
**Current State:** All functions return `Result<LuaValue, String>`
**Changes:**
- [ ] Update all `Err("...")` to use `LuaError` variants (enabled by Phase 1)
- [ ] Use `validation::require_args()` and `validation::require_type()` consistently
- [ ] Add context: function name and argument positions
- [ ] Standardize error message format: "function_name() expects X as argument N, got Y"

**Before:**
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

**After:**
```rust
pub fn create_string_len() -> Rc<dyn Fn(Vec<LuaValue>) -> Result<LuaValue, LuaError>> {
    Rc::new(|args| {
        validation::require_args("string.len", args, 1, Some(1))?;
        let s = validation::get_string("string.len", 0, &args[0])?;
        Ok(LuaValue::Number(s.len() as f64))
    })
}
```

**Impact:** ~200 lines of error handling code condensed to 3-line validation chains.

**Files to Modify/Create:**
- Create: `src/stdlib/` directory with all submodules
- Create: `src/stdlib/validation.rs` with helper functions
- Move functions: Organize into thematic submodules
- Delete: Old `src/stdlib.rs`
- Update: `src/lib.rs` and `src/main.rs` imports

**Success Criteria:**
- [ ] All stdlib functions under 15 lines each
- [ ] Zero duplicated validation logic
- [ ] `validation::require_args()` called in 50+ functions
- [ ] `validation::get_*()` eliminates type-check boilerplate
- [ ] All errors use LuaError enum (dependency on Phase 1)
- [ ] Each submodule has dedicated unit tests
- [ ] No changes to public API (register_stdlib unchanged)

---

## Phase 4: Parser Code Organization ✅ COMPLETE

**Previous State:** `lua_parser.rs` was 3,330 lines
**Status:** Successfully split into modular structure

### Completed Implementation

**Structure:**
```
lua_parser_types.rs (210 lines - type definitions)
lua_parser/
  ├── mod.rs (410 lines - public API, integration, tests)
  ├── helpers.rs (184 lines - tokenization, keywords, symbols)
  ├── expression.rs (568 lines - expr parsing, operators)
  └── statement.rs (447 lines - statement parsing, control flow)
```

**Results:**
- 3,330 lines → 1,609 lines (52% reduction)
- 4 focused modules with clear responsibilities
- 140/142 tests passing (2 pre-existing failures with single-quote strings)
- Zero breaking changes to public API
- Comprehensive test coverage maintained

**Benefits Realized:**
- Each module under 600 lines (vs 3,330)
- Clear separation: types, parsing, expressions, statements
- Easier to extend (add new operators, statements)
- Better code reusability and maintenance
- Prepared foundation for Phase 5 (ScopeManager)

---

## Phase 5: Abstraction Layer for Scope Management (Medium Priority)

**Current State:** `scope_stack: Vec<HashMap<String, LuaValue>>` scattered throughout
**Problem:** No encapsulation, borrowing issues, difficult to add features

### 5.1 Create ScopeManager Struct
**Location:** `src/scope_manager.rs`
**Scope:** ~150 lines

```rust
pub struct ScopeManager {
    stack: Vec<HashMap<String, LuaValue>>,
    closure_captures: HashMap<String, LuaValue>,
}

impl ScopeManager {
    pub fn push(&mut self) -> usize
    pub fn pop(&mut self) -> Result<(), String>
    pub fn define(&mut self, name: String, value: LuaValue)
    pub fn lookup(&self, name: &str) -> Option<LuaValue>
    pub fn update(&mut self, name: String, value: LuaValue) -> Result<(), String>
    pub fn depth(&self) -> usize
    pub fn current_scope(&self) -> &HashMap<String, LuaValue>
    pub fn clear(&mut self)
}
```

### 5.2 Update LuaInterpreter
- [ ] Replace `scope_stack: Vec<HashMap<...>>` with `ScopeManager`
- [ ] Replace direct scope access with ScopeManager calls
- [ ] Add scope_manager tests

**Benefits:**
- Encapsulation of scope logic
- Easier to add scope features (e.g., upvalue tracking)
- Reduced borrowing complexity
- Single point for scope manipulation

---

## Phase 6: Enhanced Testing Coverage (Medium Priority)

### 6.1 Parser Error Cases
**Location:** `tests/parser_errors.rs`
- [ ] Unterminated strings
- [ ] Invalid number literals
- [ ] Missing closing delimiters
- [ ] Invalid statements
- [ ] Incomplete expressions

### 6.2 Executor Edge Cases
**Location:** `tests/executor_edge_cases.rs`
- [ ] Division by zero handling
- [ ] Type coercion edge cases
- [ ] Deeply nested tables
- [ ] Recursive function limits
- [ ] Very large strings/tables

### 6.3 Integration Tests
**Location:** `tests/integration/`
- [ ] Module loading with errors
- [ ] Circular dependencies
- [ ] Stdlib function chains
- [ ] Error propagation across boundaries
- [ ] Mixed feature interactions (coroutines + modules + tables)

### 6.4 Performance Tests
**Location:** `tests/performance.rs`
- [ ] Large table operations
- [ ] String concatenation (100+ operations)
- [ ] Deep recursion
- [ ] Module loading overhead

---

## Phase 7: Performance Optimizations (Low Priority)

### 7.1 String Concatenation
**Problem:** Each concat allocates new string
**Solution:** Buffer string operations, lazy concatenation
- [ ] Implement StringBuffer helper
- [ ] Modify `apply_binary_op` to use buffer for concat chains
- [ ] Add benchmarks

### 7.2 Value Stack Utilization
**Current:** ValueStack exists but underutilized
**Improvement:**
- [ ] Use for intermediate expression results
- [ ] Reduce Vec allocations in expression eval
- [ ] Profile and measure impact

### 7.3 Table Lookup Optimization
**Ideas:**
- [ ] Cache recent lookups
- [ ] Add string interning for table keys
- [ ] Profile table access patterns

---

## Phase 8: Documentation & Architecture (Low Priority)

### 8.1 Add Architecture Document (`doc/ARCHITECTURE.md`)
- [ ] Module responsibilities diagram
- [ ] Data flow diagram (input → parse → execute → output)
- [ ] Call stack examples
- [ ] Scope/closure mechanism explanation

### 8.2 Module Documentation
- [ ] Document each module's public API
- [ ] Add examples to key functions
- [ ] Document design decisions (e.g., why RefCell for tables)
- [ ] Error handling strategy

### 8.3 Contribution Guide (`CONTRIBUTING.md`)
- [ ] Code style guidelines
- [ ] Test expectations
- [ ] How to add new stdlib functions
- [ ] How to add new language features

---

## Implementation Priority Order

### **Sprint 1 (High Impact, Essential):**
1. Phase 1: Error Handling Overhaul (enables better testing)
2. Phase 2.1-2.2: Split executor (operators + tables modules)
3. Phase 6.1: Parser error tests

### **Sprint 2 (High Impact, Important):**
1. Phase 2.3-2.5: Complete executor split
2. Phase 3.1: Stdlib argument validation
3. Phase 5: ScopeManager abstraction

### **Sprint 3 (Medium Impact, Nice-to-Have):**
1. Phase 3.2-3.3: Organize stdlib
2. Phase 4: Parser code organization
3. Phase 6.2-6.3: Additional tests
4. Phase 8: Documentation

### **Sprint 4+ (Low Priority):**
1. Phase 7: Performance optimizations
2. Phase 8.1-8.3: Architecture documentation

---

## Metrics & Success Criteria

### After Phase 1 (Error Handling):
- All functions use LuaError instead of String
- Error tests achieve 90%+ coverage
- Error messages include location info

### After Phase 2 (Executor Split):
- executor.rs < 300 lines
- 4 focused sub-modules total
- Each module has dedicated tests
- Test count increases to 250+

### After Phase 3 (Stdlib):
- Code duplication reduced by 40%
- Stdlib functions < 400 lines
- Consistent error reporting

### After Phase 5 (ScopeManager):
- Borrowing issues in interpreter/executor reduced
- ScopeManager tests at 100%
- No unsafe code introduced

### Overall Goals:
- Increase test count to 350+
- Reduce cyclomatic complexity
- Improve code organization (clear module boundaries)
- Add comprehensive documentation
- No performance regression

---

## Risk Mitigation

- **Large refactors:** Maintain git history, test frequently, don't refactor multiple areas simultaneously
- **Backward compatibility:** Executor API stays same, internal changes only
- **Performance:** Add benchmarks before/after optimizations, measure with `cargo bench`
- **Testing:** Increase test coverage before major changes, ensure all tests pass between phases

---

## Notes

- All error handling changes should be done first—they enable better testing
- Keep executor split changes small and incremental
- Scope manager can be introduced gradually (one method at a time)
- Document rationale for design decisions as you implement
- Consider creating a CHANGELOG for tracking improvements
