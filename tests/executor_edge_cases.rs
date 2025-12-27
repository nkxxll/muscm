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

#[test]
fn test_division_by_zero() {
    let code = "return 10 / 0";
    let result = execute_code(code);
    assert!(result.is_ok(), "Division by zero should not panic");
}

#[test]
fn test_deeply_nested_tables() {
    let code = r#"
local t = {}
local current = t
for i = 1, 50 do
    current.next = {}
    current = current.next
end
current.value = 42
return t
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Deeply nested tables should execute");
}

#[test]
fn test_table_self_reference() {
    let code = r#"
local t = {}
t.self = t
return t
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Table can contain reference to itself");
}

#[test]
fn test_recursive_function() {
    let code = r#"
function factorial(n)
    if n <= 1 then return 1 end
    return n * factorial(n - 1)
end
return factorial(5)
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Recursive function should execute");
}

#[test]
fn test_deep_recursion() {
    let code = r#"
function deep(n)
    if n <= 0 then return 0 end
    return deep(n - 1) + 1
end
return deep(50)
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Deep recursion should execute");
}

#[test]
fn test_mutual_recursion() {
    let code = r#"
function even(n)
    if n == 0 then return true end
    return odd(n - 1)
end

function odd(n)
    if n == 0 then return false end
    return even(n - 1)
end

return even(10)
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Mutual recursion should execute");
}

#[test]
fn test_string_concatenation_chain() {
    let code = r#"
local s = ""
for i = 1, 100 do
    s = s .. "x"
end
return s
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "String concatenation chains should work");
}

#[test]
fn test_large_table_operations() {
    let code = r#"
local t = {}
for i = 1, 500 do
    t[i] = i
end
return t[250]
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Large table operations should work");
}

#[test]
fn test_table_with_many_string_keys() {
    let code = r#"
local t = {}
for i = 1, 100 do
    t["key" .. i] = i
end
return t["key50"]
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Table with many string keys should work");
}

#[test]
fn test_nil_comparison() {
    let code = r#"
local x = nil
if x == nil then
    return true
else
    return false
end
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "nil comparison should work");
}

#[test]
fn test_function_as_table_value() {
    let code = r#"
local t = {}
t.func = function() return 42 end
return t.func()
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Functions as table values should work");
}

#[test]
fn test_function_returning_multiple_values() {
    let code = r#"
function multi_return()
    return 1, 2, 3
end
local a, b, c = multi_return()
return a
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Multi-return functions should work");
}

#[test]
fn test_empty_table() {
    let code = r#"
local t = {}
return t
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Empty table should work");
}

#[test]
fn test_table_length_operator() {
    let code = r#"
local t = {1, 2, 3}
return #t
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Length operator should work");
}

#[test]
fn test_unary_minus() {
    let code = r#"return -5"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Unary minus should work");
}

#[test]
fn test_unary_not() {
    let code = r#"return not true"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Unary not should work");
}

#[test]
fn test_unary_length() {
    let code = r#"return #"hello""#;
    let result = execute_code(code);
    assert!(result.is_ok(), "String length operator should work");
}

#[test]
fn test_all_arithmetic_operators() {
    let code = r#"
local a = 10
local add = a + 5
local sub = a - 3
local mul = a * 2
local div = a / 2
local mod = a % 3
return add
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "All arithmetic operators should work");
}

#[test]
fn test_short_circuit_and() {
    let code = r#"
if false and true then
    return "executed"
else
    return "short-circuited"
end
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Short-circuit AND should work");
}

#[test]
fn test_short_circuit_or() {
    let code = r#"
if true or false then
    return "short-circuited"
else
    return "executed"
end
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Short-circuit OR should work");
}

#[test]
fn test_nested_table_indexing() {
    let code = r#"
local t = {a = {b = {c = 42}}}
return t.a.b.c
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Nested table indexing should work");
}

#[test]
fn test_table_numeric_and_string_keys_mixed() {
    let code = r#"
local t = {10, 20, 30, name = "test", value = 42}
return t[1]
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Mixed table keys should work");
}

#[test]
fn test_for_loop_with_step() {
    let code = r#"
local sum = 0
for i = 1, 10, 2 do
    sum = sum + i
end
return sum
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "For loop with step should work");
}

#[test]
fn test_while_loop() {
    let code = r#"
local i = 0
while i < 10 do
    i = i + 1
end
return i
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "While loop should work");
}

#[test]
fn test_repeat_until_loop() {
    let code = r#"
local i = 0
repeat
    i = i + 1
until i >= 10
return i
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Repeat-until loop should work");
}

#[test]
fn test_nested_loops_with_tables() {
    let code = r#"
local matrix = {}
for i = 1, 20 do
    matrix[i] = {}
    for j = 1, 20 do
        matrix[i][j] = i * j
    end
end
return matrix[10][10]
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "Nested loops with table operations should work");
}

#[test]
fn test_if_elseif_else() {
    let code = r#"
local x = 5
if x > 10 then
    return "big"
elseif x > 0 then
    return "positive"
else
    return "negative"
end
"#;
    let result = execute_code(code);
    assert!(result.is_ok(), "If-elseif-else should work");
}
