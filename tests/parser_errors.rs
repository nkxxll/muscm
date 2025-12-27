use muscm::lua_parser::{tokenize, parse, TokenSlice};

// Helper function to tokenize and parse code
fn parse_code(code: &str) -> Result<(), String> {
    let tokens = tokenize(code)?;
    let token_slice = TokenSlice::from(tokens.as_slice());
    parse(token_slice).map(|_| ()).map_err(|e| format!("{:?}", e))
}

#[test]
fn test_unterminated_string_single_line() {
    let code = r#"local x = "hello"#;
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on unterminated string");
}

#[test]
fn test_unterminated_string_multiline() {
    let code = "local x = \"hello\nworld";
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on unterminated string across lines");
}

#[test]
fn test_invalid_number_literal_hex() {
    let code = r#"local x = 0x"#;
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on incomplete hex literal");
}

#[test]
fn test_missing_closing_paren() {
    let code = r#"local x = (1 + 2"#;
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on missing closing parenthesis");
}

#[test]
fn test_missing_closing_bracket() {
    let code = r#"local t = {1, 2, 3"#;
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on missing closing bracket");
}

#[test]
fn test_missing_end_keyword() {
    let code = r#"function test()
    return 42"#;
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on missing 'end' keyword");
}

#[test]
fn test_missing_then_keyword() {
    let code = r#"if x > 0
    return 42
end"#;
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on missing 'then' keyword");
}

#[test]
fn test_missing_do_keyword() {
    let code = r#"while x > 0
    x = x - 1
end"#;
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on missing 'do' keyword");
}

#[test]
fn test_incomplete_expression_binary_operator() {
    let code = r#"local x = 1 +"#;
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on incomplete binary operation");
}

#[test]
fn test_incomplete_table_definition() {
    let code = r#"local t = {x = "#;
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on incomplete table field");
}

#[test]
fn test_invalid_for_loop_syntax() {
    let code = r#"for i 1, 10 do
    print(i)
end"#;
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on invalid 'for' syntax");
}

#[test]
fn test_invalid_function_call_no_args() {
    let code = r#"print()"#;
    let result = parse_code(code);
    // This should actually parse fine - functions can have no args
    assert!(result.is_ok(), "Empty function args should parse");
}

#[test]
fn test_table_with_invalid_key() {
    let code = r#"local t = {[1+] = 5}"#;
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on invalid table key expression");
}

#[test]
fn test_nested_function_definition() {
    let code = r#"function outer()
    function inner()
        return 42
    end
    return inner()
end"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Nested functions should parse");
}

#[test]
fn test_lambda_with_missing_body() {
    let code = r#"local f = function() "#;
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on function with no body");
}

#[test]
fn test_invalid_local_declaration() {
    let code = r#"local 123"#;
    let result = parse_code(code);
    assert!(result.is_err(), "Should error on 'local' with number");
}

#[test]
fn test_operator_precedence_parentheses() {
    let code = r#"local x = (1 + 2) * 3"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Parentheses should parse");
}

#[test]
fn test_comment_in_string() {
    let code = r#"local s = "hello -- not a comment""#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Comments inside strings should not be treated as comments");
}

#[test]
fn test_string_escape_sequences() {
    let code = r#"local s = "hello\nworld\t!""#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Escape sequences in strings should parse");
}

#[test]
fn test_empty_function_block() {
    let code = r#"function empty()
end"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Empty function should parse");
}

#[test]
fn test_empty_if_block() {
    let code = r#"if true then
end"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Empty if block should parse");
}

#[test]
fn test_elseif_chain() {
    let code = r#"if x == 1 then
    print(1)
elseif x == 2 then
    print(2)
elseif x == 3 then
    print(3)
else
    print(0)
end"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "elseif chain should parse");
}

#[test]
fn test_global_function_definition() {
    let code = r#"function globalFunc(a, b, c)
    return a + b + c
end"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Global function definition should parse");
}

#[test]
fn test_method_definition() {
    let code = r#"function obj:method(x)
    return self.value + x
end"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Method definition with colon syntax should parse");
}

#[test]
fn test_table_field_expression() {
    let code = r#"local x = 5
local t = {x, [x] = 10, key = 20}"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Mixed table field types should parse");
}

#[test]
fn test_break_outside_loop() {
    let code = r#"break"#;
    let result = parse_code(code);
    // Parser may not validate context, but this should at least parse
    assert!(result.is_ok(), "Break should parse (validation happens at runtime)");
}

#[test]
fn test_comment_at_eof() {
    let code = r#"local x = 1 -- final comment"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Comment at end of file should parse");
}

#[test]
fn test_multiple_returns_in_function() {
    let code = r#"function test(x)
    if x > 0 then
        return 1
    end
    return 0
end"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Multiple returns should parse");
}

#[test]
fn test_nil_literal() {
    let code = r#"local x = nil"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "nil literal should parse");
}

#[test]
fn test_boolean_literals() {
    let code = r#"local t = true
local f = false"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Boolean literals should parse");
}

#[test]
fn test_negative_number() {
    let code = r#"local x = -42
local y = -3.14"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Negative numbers should parse");
}

#[test]
fn test_comparison_chain() {
    let code = r#"local x = a < b"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Comparison should parse");
}

#[test]
fn test_logical_operators() {
    let code = r#"local x = true and false or not true"#;
    let result = parse_code(code);
    assert!(result.is_ok(), "Logical operators should parse");
}
