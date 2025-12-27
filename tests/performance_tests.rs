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
// LARGE TABLE OPERATIONS
// =====================================================

#[test]
fn test_large_table_creation() {
    let code = r#"
local t = {}
for i = 1, 1000 do
    t[i] = i * 2
end
return t[500]
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Creating table with 1000 elements should succeed");
}

#[test]
fn test_large_table_with_string_keys() {
    let code = r#"
local t = {}
for i = 1, 500 do
    t["key_" .. i] = i
end
return t["key_250"]
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Large table with string keys should succeed");
}

#[test]
fn test_large_table_modification() {
    let code = r#"
local t = {}
for i = 1, 500 do
    t[i] = i
end
for i = 1, 500 do
    t[i] = t[i] * 2
end
return t[250]
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Modifying large table should succeed");
}

#[test]
fn test_nested_table_creation() {
    let code = r#"
local t = {}
for i = 1, 50 do
    t[i] = {}
    for j = 1, 30 do
        t[i][j] = i * j
    end
end
return t[25][15]
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Nested table creation should succeed");
}

#[test]
fn test_table_with_mixed_types() {
    let code = r#"
local t = {}
for i = 1, 300 do
    if i % 3 == 0 then
        t[i] = "string"
    elseif i % 2 == 0 then
        t[i] = 3.14
    else
        t[i] = true
    end
end
return t[300]
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Table with mixed types should succeed");
}

#[test]
fn test_table_iteration_large() {
    let code = r#"
local t = {}
for i = 1, 500 do
    t[i] = i
end
local sum = 0
for i = 1, #t do
    sum = sum + t[i]
end
return sum
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Iterating large table should succeed");
}

// =====================================================
// STRING CONCATENATION OPERATIONS
// =====================================================

#[test]
fn test_string_concatenation_simple() {
    let code = r#"
local s = ""
for i = 1, 100 do
    s = s .. "x"
end
return s
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Simple string concatenation should work");
}

#[test]
fn test_string_concatenation_large() {
    let code = r#"
local s = ""
for i = 1, 300 do
    s = s .. "hello_"
end
return s
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Large string concatenation should work");
}

#[test]
fn test_string_concatenation_with_numbers() {
    let code = r#"
local s = ""
for i = 1, 100 do
    s = s .. tostring(i) .. ","
end
return s
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "String concatenation with number conversion should work");
}

#[test]
fn test_string_operations_on_large_string() {
    let code = r#"
local s = ""
for i = 1, 500 do
    s = s .. "a"
end
local len = string.len(s)
return len
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "String operations on large string should work");
}

// =====================================================
// DEEP RECURSION
// =====================================================

#[test]
fn test_deep_recursion_factorial() {
    let code = r#"
function fact(n)
    if n <= 1 then return 1 end
    return n * fact(n - 1)
end
return fact(15)
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Deep recursion (15 levels) should work");
}

#[test]
fn test_deep_recursion_fibonacci() {
    let code = r#"
function fib(n)
    if n <= 1 then return n end
    return fib(n - 1) + fib(n - 2)
end
return fib(10)
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Deep recursion with branching should work");
}

#[test]
fn test_deep_recursion_tree_traversal() {
    let code = r#"
function sum_tree(t)
    local sum = 0
    for i = 1, #t do
        if type(t[i]) == "table" then
            sum = sum + sum_tree(t[i])
        else
            sum = sum + t[i]
        end
    end
    return sum
end

local tree = {1, {2, {3, {4}}}}
return sum_tree(tree)
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Deep recursion with tree traversal should work");
}

#[test]
fn test_mutual_recursion_performance() {
    let code = r#"
function even(n)
    if n == 0 then return true end
    return odd(n - 1)
end

function odd(n)
    if n == 0 then return false end
    return even(n - 1)
end

return even(30)
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Mutual recursion should work");
}

// =====================================================
// COMPLEX OPERATIONS COMBINATIONS
// =====================================================

#[test]
fn test_nested_loops_with_table_operations() {
    let code = r#"
local matrix = {}
for i = 1, 30 do
    matrix[i] = {}
    for j = 1, 30 do
        matrix[i][j] = i * j
    end
end
local sum = 0
for i = 1, 30 do
    for j = 1, 30 do
        sum = sum + matrix[i][j]
    end
end
return sum
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Nested loops with table operations should work");
}

#[test]
fn test_function_calls_in_loop() {
    let code = r#"
function process(x)
    return x * 2 + 1
end

local results = {}
for i = 1, 300 do
    table.insert(results, process(i))
end
return results[150]
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Function calls in loop should work");
}

#[test]
fn test_string_manipulation_in_loop() {
    let code = r#"
local results = {}
for i = 1, 100 do
    local s = "item_" .. i
    local upper = string.upper(s)
    table.insert(results, upper)
end
return results[50]
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "String manipulation in loop should work");
}

#[test]
fn test_conditional_table_building() {
    let code = r#"
local primes = {}
for i = 2, 100 do
    local is_prime = true
    for j = 2, i - 1 do
        if i % j == 0 then
            is_prime = false
            break
        end
    end
    if is_prime then
        table.insert(primes, i)
    end
end
return #primes
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Conditional table building should work");
}

#[test]
fn test_map_reduce_pattern() {
    let code = r#"
local data = {}
for i = 1, 100 do
    table.insert(data, i)
end

local doubled = {}
for i = 1, #data do
    table.insert(doubled, data[i] * 2)
end

local sum = 0
for i = 1, #doubled do
    sum = sum + doubled[i]
end
return sum
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Map-reduce pattern should work");
}

#[test]
fn test_higher_order_functions() {
    let code = r#"
function apply_twice(f, x)
    return f(f(x))
end

function double(x)
    return x * 2
end

local result = apply_twice(double, 5)
return result
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Higher-order functions should work");
}

#[test]
fn test_large_expression_evaluation() {
    let code = r#"
local a = 5
local b = 10
local c = 15
local d = 20
local e = 25
local result = (a + b) * (c - d) + (e / a)
return result
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Large expression evaluation should work");
}
