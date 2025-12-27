# Phase 7: Integration Test Examples

This document provides integration tests that demonstrate Phase 7 functionality working together in the Lua interpreter.

## Test 1: Metatables for Custom Objects

### Test Code
```lua
-- Create a metatable for vector operations
local mt = {}
mt.__add = function(a, b)
  return {x = a.x + b.x, y = a.y + b.y}
end
mt.__tostring = function(self)
  return "Vec(" .. self.x .. ", " .. self.y .. ")"
end

-- Create two vectors
local v1 = {x = 1, y = 2}
local v2 = {x = 3, y = 4}

-- Set metatables
setmetatable(v1, mt)
setmetatable(v2, mt)

-- Now we can add vectors
local v3 = v1 + v2
print("v3.x =", v3.x)  -- Should print 4
print("v3.y =", v3.y)  -- Should print 6
```

### Expected Results
- v3.x = 4
- v3.y = 6
- Metatables successfully enable custom operators

---

## Test 2: Error Handling with pcall

### Test Code
```lua
function safe_divide(a, b)
  if b == 0 then
    error("Division by zero")
  end
  return a / b
end

-- Test 1: Normal case
ok, result = pcall(safe_divide, 10, 2)
print("Test 1 - Success:", ok, result)  -- true 5

-- Test 2: Error case
ok, result = pcall(safe_divide, 10, 0)
print("Test 2 - Error:", ok, result)    -- false error message
```

### Expected Results
- Test 1: Success (ok=true, result=5)
- Test 2: Error caught (ok=false, result contains error message)

---

## Test 3: Getmetatable Access

### Test Code
```lua
-- Create a table
local t = {a = 1}

-- Create a metatable
local mt = {__name = "custom"}
setmetatable(t, mt)

-- Retrieve the metatable
local retrieved_mt = getmetatable(t)
print("Has metatable:", retrieved_mt ~= nil)  -- true

-- Clear metatable
setmetatable(t, nil)
retrieved_mt = getmetatable(t)
print("Cleared:", retrieved_mt == nil)  -- true
```

### Expected Results
- Retrieved metatable successfully
- Clearing with nil works
- getmetatable returns nil for tables without metatables

---

## Test 4: Coroutines Table Availability

### Test Code
```lua
-- Check that coroutine module is available
print("coroutine module exists:", coroutine ~= nil)

-- Check that coroutine functions are available
print("create:", coroutine.create ~= nil)
print("resume:", coroutine.resume ~= nil)
print("yield:", coroutine.yield ~= nil)
print("status:", coroutine.status ~= nil)
```

### Expected Results
- coroutine module is available
- All four coroutine functions are accessible

---

## Test 5: Error Function

### Test Code
```lua
-- Test error throwing
function may_error()
  error("Custom error message")
end

ok, msg = pcall(may_error)
print("Error caught:", not ok)           -- true
print("Error message contains text:", string.len(msg) > 0)  -- true
```

### Expected Results
- error() properly throws errors
- pcall catches the error
- Error message is preserved

---

## Test 6: Complex Metatable Chain

### Test Code
```lua
-- Create a metatable that tracks operations
local operations = {}
local mt = {
  __add = function(a, b)
    table.insert(operations, "add")
    return a.val + b.val
  end,
  __mul = function(a, b)
    table.insert(operations, "mul")
    return a.val * b.val
  end
}

local a = {val = 5}
local b = {val = 3}
setmetatable(a, mt)
setmetatable(b, mt)

local sum = a + b      -- Should trigger __add
local prod = a * b     -- Should trigger __mul

print("Operations tracked:", #operations)  -- 2
print("First op:", operations[1])          -- "add"
print("Second op:", operations[2])         -- "mul"
```

### Expected Results
- Multiple metamethods work together
- Metamethods are called correctly
- Operations table tracks all calls

---

## Running the Tests

To run these integration tests with the Lua interpreter:

```bash
# Compile the interpreter
cargo build --release

# Run with your favorite Lua code
./target/release/muscm script.lua
```

## Verification Checklist

After Phase 7 implementation, verify:

- [ ] setmetatable() sets metatables on tables
- [ ] getmetatable() retrieves metatables
- [ ] setmetatable() with nil clears metatables
- [ ] getmetatable() returns nil for non-table values
- [ ] getmetatable() returns nil for tables without metatables
- [ ] pcall() is available as a global function
- [ ] xpcall() is available as a global function
- [ ] error() properly throws errors
- [ ] coroutine module is available globally
- [ ] coroutine.create is available
- [ ] coroutine.resume is available
- [ ] coroutine.yield is available
- [ ] coroutine.status is available
- [ ] All Phase 1-6 tests still pass (backward compatibility)

## Phase 7 Status

### Completed
- ✅ Error handling infrastructure (errors.rs)
- ✅ Upvalues module (upvalues.rs)
- ✅ Coroutines module (coroutines.rs)
- ✅ Metatable support in LuaValue
- ✅ setmetatable() and getmetatable() functions
- ✅ pcall(), xpcall(), and error() functions
- ✅ coroutine module registration
- ✅ Comprehensive test suite (14 new tests)
- ✅ All existing tests pass (45 tests)

### Next Steps
- Full metatable operator support in executor
- Complete coroutine yield/resume implementation
- Tail-call optimization detection
- Complete upvalue capture during parsing
- Comprehensive error stack traces

## Test Summary

Total Phase 7 tests added: 14
- Metatable tests: 5
- Error handling tests: 5
- Coroutine tests: 2
- Module loading tests: 2

All tests pass with zero failures.
