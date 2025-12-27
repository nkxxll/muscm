//! Error handling tests for the Lua interpreter
//!
//! Tests all error variants and their properties

use muscm::error_types::*;

#[test]
fn test_parse_error_with_location() {
    let err = LuaError::parse("unexpected symbol", 42, 15);
    assert_eq!(err.category(), "parse");
    assert!(err.message().contains("42:15"));
    assert!(err.message().contains("unexpected symbol"));
}

#[test]
fn test_parse_error_display() {
    let err = LuaError::parse("unterminated string", 5, 10);
    let msg = format!("{}", err);
    assert!(msg.contains("5:10"));
    assert!(msg.contains("unterminated string"));
}

#[test]
fn test_runtime_error_with_context() {
    let err = LuaError::runtime("attempted to perform operation", "loop execution");
    assert_eq!(err.category(), "runtime");
    let msg = err.message();
    assert!(msg.contains("loop execution"));
    assert!(msg.contains("attempted to perform operation"));
}

#[test]
fn test_type_error_function_context() {
    let err = LuaError::type_error("string", "number", "string.len");
    assert_eq!(err.category(), "type");
    let msg = err.message();
    assert!(msg.contains("string.len"));
    assert!(msg.contains("string"));
    assert!(msg.contains("number"));
}

#[test]
fn test_value_error_simple() {
    let err = LuaError::value("invalid table key");
    assert_eq!(err.category(), "value");
    assert!(err.message().contains("invalid table key"));
}

#[test]
fn test_file_error_path_info() {
    let err = LuaError::file("modules/mylib.lua", "permission denied");
    assert_eq!(err.category(), "file");
    let msg = err.message();
    assert!(msg.contains("modules/mylib.lua"));
    assert!(msg.contains("permission denied"));
}

#[test]
fn test_module_error_circular_dependency() {
    let err = LuaError::module("moduleA", "circular dependency: moduleA -> moduleB -> moduleA");
    assert_eq!(err.category(), "module");
    assert!(err.message().contains("moduleA"));
    assert!(err.message().contains("circular"));
}

#[test]
fn test_module_error_not_found() {
    let err = LuaError::module("unknown_lib", "not found in search path");
    assert_eq!(err.category(), "module");
    assert!(err.message().contains("unknown_lib"));
}

#[test]
fn test_token_error_with_position() {
    let err = LuaError::token("invalid number format", 125);
    assert_eq!(err.category(), "token");
    let msg = err.message();
    assert!(msg.contains("125"));
    assert!(msg.contains("invalid number format"));
}

#[test]
fn test_user_error_with_level() {
    let err = LuaError::user("division by zero", 2);
    assert_eq!(err.category(), "user");
    match err {
        LuaError::UserError { message, level } => {
            assert_eq!(message, "division by zero");
            assert_eq!(level, 2);
        }
        _ => panic!("Expected UserError"),
    }
}

#[test]
fn test_user_error_display() {
    let err = LuaError::user("custom error from Lua code", 1);
    let msg = format!("{}", err);
    assert_eq!(msg, "custom error from Lua code");
}

#[test]
fn test_break_outside_loop() {
    let err = LuaError::BreakOutsideLoop;
    assert_eq!(err.category(), "control_flow");
    assert!(err.message().contains("break"));
    assert!(err.message().contains("outside loop"));
}

#[test]
fn test_undefined_label() {
    let err = LuaError::UndefinedLabel {
        label: "skip_ahead".to_string(),
    };
    assert_eq!(err.category(), "label");
    assert!(err.message().contains("skip_ahead"));
    assert!(err.message().contains("undefined"));
}

#[test]
fn test_argument_count_error_format() {
    let err = LuaError::arg_count("ipairs", 1, 5);
    assert_eq!(err.category(), "argument");
    let msg = err.message();
    assert!(msg.contains("ipairs"));
    assert!(msg.contains("1"));
    assert!(msg.contains("5"));
}

#[test]
fn test_division_by_zero_error() {
    let err = LuaError::DivisionByZero;
    assert_eq!(err.category(), "arithmetic");
    assert!(err.message().contains("division by zero"));
}

#[test]
fn test_index_error_table() {
    let err = LuaError::index("table", "number");
    assert_eq!(err.category(), "index");
    let msg = err.message();
    assert!(msg.contains("table"));
    assert!(msg.contains("number"));
}

#[test]
fn test_index_error_string() {
    let err = LuaError::index("string", "function");
    assert_eq!(err.category(), "index");
    assert!(err.message().contains("string"));
    assert!(err.message().contains("function"));
}

#[test]
fn test_call_error() {
    let err = LuaError::call("number");
    assert_eq!(err.category(), "call");
    let msg = err.message();
    assert!(msg.contains("number"));
    assert!(msg.contains("call"));
}

#[test]
fn test_call_error_various_types() {
    let types = vec!["nil", "string", "number", "table", "boolean"];
    for typ in types {
        let err = LuaError::call(typ);
        let msg = err.message();
        assert!(msg.contains(typ));
    }
}

#[test]
fn test_error_equality() {
    let err1 = LuaError::value("test");
    let err2 = LuaError::value("test");
    assert_eq!(err1, err2);
}

#[test]
fn test_error_inequality() {
    let err1 = LuaError::value("test1");
    let err2 = LuaError::value("test2");
    assert_ne!(err1, err2);
}

#[test]
fn test_error_clone() {
    let err1 = LuaError::parse("test", 1, 1);
    let err2 = err1.clone();
    assert_eq!(err1, err2);
}

#[test]
fn test_lua_result_ok() {
    let result: LuaResult<i32> = Ok(42);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_lua_result_err() {
    let result: LuaResult<i32> = Err(LuaError::value("oops"));
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.category(), "value");
}

#[test]
fn test_result_map_ok() {
    let result: LuaResult<i32> = Ok(10);
    let mapped = result.map(|x| x * 2);
    assert_eq!(mapped.unwrap(), 20);
}

#[test]
fn test_result_map_err() {
    let result: LuaResult<i32> = Err(LuaError::value("error"));
    let mapped = result.map(|x| x * 2);
    assert!(mapped.is_err());
}

#[test]
fn test_error_conversion_from_string() {
    // Helper for legacy code migration
    fn legacy_error_to_lua(msg: &str) -> LuaError {
        LuaError::value(msg)
    }

    let err = legacy_error_to_lua("legacy error message");
    assert!(err.message().contains("legacy error message"));
}

#[test]
fn test_all_error_categories() {
    let errors = vec![
        (LuaError::parse("test", 0, 0), "parse"),
        (LuaError::runtime("test", "ctx"), "runtime"),
        (LuaError::type_error("a", "b", "c"), "type"),
        (LuaError::value("test"), "value"),
        (LuaError::file("f", "r"), "file"),
        (LuaError::module("m", "r"), "module"),
        (LuaError::token("test", 0), "token"),
        (LuaError::user("test", 0), "user"),
        (LuaError::BreakOutsideLoop, "control_flow"),
        (LuaError::DivisionByZero, "arithmetic"),
    ];

    for (err, expected_cat) in errors {
        assert_eq!(err.category(), expected_cat);
    }
}

#[test]
fn test_error_message_consistency() {
    let err = LuaError::type_error("number", "string", "tonumber");
    let msg1 = err.message();
    let msg2 = err.message();
    assert_eq!(msg1, msg2); // Consistent across multiple calls
}

#[test]
fn test_complex_error_scenario() {
    // Simulates error propagation through function calls
    let parse_err = LuaError::parse("unexpected token", 10, 5);
    assert_eq!(parse_err.category(), "parse");

    // Later converted to runtime context if caught
    let runtime_context = format!("while parsing: {}", parse_err.message());
    let err = LuaError::runtime("parse failed", runtime_context);
    assert!(err.message().contains("parse failed"));
}

#[test]
fn test_error_display_impl() {
    let errors: Vec<(&str, Box<dyn std::fmt::Display>)> = vec![
        (
            "parse",
            Box::new(LuaError::parse("test", 1, 1)) as Box<dyn std::fmt::Display>,
        ),
        ("runtime", Box::new(LuaError::runtime("test", "ctx"))),
        ("value", Box::new(LuaError::value("test"))),
        ("file", Box::new(LuaError::file("f", "r"))),
    ];

    for (expected_word, err) in errors {
        let display_str = format!("{}", err);
        assert!(!display_str.is_empty());
        // Each should contain some meaningful information
        assert!(display_str.len() > 5);
    }
}

#[test]
fn test_lua_error_std_error_trait() {
    let err: Box<dyn std::error::Error> = Box::new(LuaError::value("test error"));
    let msg = err.to_string();
    assert!(msg.contains("test error"));
}
