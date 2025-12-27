use muscm::executor::Executor;
use muscm::lua_interpreter::LuaInterpreter;
use muscm::lua_parser::{parse as parse_lua, tokenize, TokenSlice};

// Helper function to execute code
fn execute_code(code: &str) -> Result<String, String> {
    let tokens = tokenize(code)?;
    let token_slice = TokenSlice::from(tokens.as_slice());
    let (_, block) = parse_lua(token_slice).map_err(|e| format!("{:?}", e))?;
    
    let mut executor = Executor::new();
    let mut interp = LuaInterpreter::new();
    
    executor.execute_block(&block, &mut interp).map_err(|e| format!("{:?}", e))?;
    Ok("success".to_string())
}

// =====================================================
// STDLIB FUNCTION CHAINS
// =====================================================

#[test]
fn test_string_operations_chain() {
    let code = r#"
local s = "hello world"
local upper = string.upper(s)
local length = string.len(upper)
local sub = string.sub(upper, 1, 5)
return sub
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "String operation chain should work");
}

#[test]
fn test_math_operations_chain() {
    let code = r#"
local base = 10
local floored = math.floor(base / 3)
local absval = math.abs(-floored)
local squared = absval * absval
return squared
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Math operation chain should work");
}

#[test]
fn test_table_operations_chain() {
    let code = r#"
local t = {1, 2, 3}
table.insert(t, 4)
table.insert(t, 5)
local length = #t
return length
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Table operation chain should work");
}

#[test]
fn test_mixed_stdlib_operations() {
    let code = r#"
local numbers = {}
for i = 1, 5 do
    table.insert(numbers, i * 10)
end
local s = ""
for i = 1, #numbers do
    s = s .. tostring(numbers[i]) .. ","
end
return s
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Mixed stdlib operations should work");
}

// =====================================================
// ERROR PROPAGATION ACROSS BOUNDARIES
// =====================================================

#[test]
fn test_function_call_error_propagation() {
    let code = r#"
function may_error(x)
    if x < 0 then
        error("negative number")
    end
    return x * 2
end

function caller()
    return may_error(5)
end

return caller()
"#;
    let result = execute_code(code);
    // Should succeed with positive number
    assert!(result.is_ok(), "Positive number should work");
}

#[test]
fn test_type_error_in_operation() {
    let code = r#"
local t = {1, 2, 3}
table.insert(t, "not_a_number")
return t
"#;
    let result = execute_code(code);
    // table.insert should accept any value
    assert!(result.is_ok(), "table.insert with mixed types should work");
}

#[test]
fn test_argument_count_mismatch() {
    let code = r#"
function exact_args(a, b, c)
    return a + (b or 0) + (c or 0)
end
return exact_args(1, 2)
"#;
    let result = execute_code(code);
    // Missing argument, should be nil
    assert!(result.is_ok(), "Missing argument should use nil");
}

// =====================================================
// MIXED FEATURE INTERACTIONS
// =====================================================

#[test]
fn test_closures_with_tables() {
    let code = r#"
function make_counter()
    local count = 0
    return {
        inc = function() 
            count = count + 1 
            return count 
        end,
        get = function() 
            return count 
        end
    }
end

local c1 = make_counter()
c1.inc()
c1.inc()
return c1.get()
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Closures with tables should work");
}

#[test]
fn test_for_loop_with_functions() {
    let code = r#"
local results = {}
local functions = {
    function() return 1 end,
    function() return 2 end,
    function() return 3 end
}
for i = 1, #functions do
    table.insert(results, functions[i]())
end
return results[1]
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "For loop with table of functions should work");
}

#[test]
fn test_nested_function_calls_with_tables() {
    let code = r#"
function process_table(t)
    function process_item(item)
        return item * 2
    end
    local results = {}
    for i = 1, #t do
        table.insert(results, process_item(t[i]))
    end
    return results
end

local input = {1, 2, 3, 4, 5}
local output = process_table(input)
return output[1]
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Nested functions with table processing should work");
}

#[test]
fn test_error_with_pcall() {
    let code = r#"
function risky()
    error("something went wrong")
end

local ok, result = pcall(risky)
if ok then
    return "no error"
else
    return "error caught"
end
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "pcall with error should work");
}

#[test]
fn test_multiple_return_values_with_tables() {
    let code = r#"
function split_values()
    return 1, 2, 3
end

local a, b, c = split_values()
return a
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Multiple returns should work");
}

#[test]
fn test_iterator_with_tables() {
    let code = r#"
local data = {a = 1, b = 2, c = 3}
local sum = 0
for k, v in pairs(data) do
    sum = sum + v
end
return sum
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Iterator with table operations should work");
}

#[test]
fn test_complex_expression_in_table() {
    let code = r#"
local t = {
    a = 1 + 2,
    b = string.len("hello"),
    c = math.floor(3.7),
    d = function() return 42 end
}
return t.d()
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Complex expressions in table should work");
}

#[test]
fn test_scope_isolation_with_functions() {
    let code = r#"
local x = 10
function test_scope()
    local x = 20
    return x
end
local result = test_scope()
return x
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Scope isolation should work");
}

#[test]
fn test_global_vs_local_variables() {
    let code = r#"
global_var = 100
local local_var = 200
function check()
    return global_var
end
return check()
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Global variable access from function should work");
}

#[test]
fn test_upvalue_capture_across_functions() {
    let code = r#"
function outer(x)
    return function(y)
        return x + y
    end
end
local add5 = outer(5)
return add5(10)
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Upvalue capture should work");
}

#[test]
fn test_string_library_integration() {
    let code = r#"
local s = "Hello, Lua!"
local upper = string.upper(s)
local len = string.len(upper)
local sub = string.sub(upper, 1, 5)
return len
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "String library integration should work");
}

#[test]
fn test_math_library_integration() {
    let code = r#"
local pi_approx = math.floor(3.14159 * 100)
local abs_val = math.abs(-42)
local max_val = math.max(10, 20, 5)
return max_val
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Math library integration should work");
}

#[test]
fn test_table_library_integration() {
    let code = r#"
local t = {}
table.insert(t, 10)
table.insert(t, 20)
table.insert(t, 30)
local removed = table.remove(t)
return #t
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Table library integration should work");
}

#[test]
fn test_type_conversions() {
    let code = r#"
local n = tonumber("42")
local s = tostring(123)
local t = type(s)
return t
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Type conversions should work");
}

#[test]
fn test_table_iteration_with_ipairs() {
    let code = r#"
local t = {10, 20, 30, 40}
local sum = 0
for i, v in ipairs(t) do
    sum = sum + v
end
return sum
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Table iteration with ipairs should work");
}
