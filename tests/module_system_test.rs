use muscm::lua_parser::{tokenize, parse as parse_lua, TokenSlice};
use muscm::executor::Executor;
use muscm::lua_interpreter::LuaInterpreter;
use muscm::lua_value::LuaValue;
use std::path::PathBuf;

#[test]
fn test_require_simple_module() {
    let mut executor = Executor::new();
    let mut interp = LuaInterpreter::new();
    
    // Add fixtures directory to module search paths
    interp.add_module_search_path(PathBuf::from("fixtures/modules"));
    
    let code = r#"
        local simple = require("simple")
        result = simple.add(2, 3)
    "#;
    
    let tokens = tokenize(code).expect("Failed to tokenize");
    let token_slice = TokenSlice::from(tokens.as_slice());
    let (_, block) = parse_lua(token_slice).expect("Failed to parse");
    
    let result = executor.execute_block(&block, &mut interp);
    assert!(result.is_ok(), "Execution failed: {:?}", result);
    
    // Check that the result variable was set correctly
    let result_val = interp.lookup("result").expect("result variable not found");
    assert_eq!(result_val, LuaValue::Number(5.0));
}

#[test]
fn test_require_with_exports() {
    let mut executor = Executor::new();
    let mut interp = LuaInterpreter::new();
    
    interp.add_module_search_path(PathBuf::from("fixtures/modules"));
    
    let code = r#"
        local config = require("config")
        host = config.host
        port = config.port
    "#;
    
    let tokens = tokenize(code).expect("Failed to tokenize");
    let token_slice = TokenSlice::from(tokens.as_slice());
    let (_, block) = parse_lua(token_slice).expect("Failed to parse");
    
    let result = executor.execute_block(&block, &mut interp);
    assert!(result.is_ok(), "Execution failed: {:?}", result);
    
    let host = interp.lookup("host").expect("host variable not found");
    assert_eq!(host, LuaValue::String("localhost".to_string()));
    
    let port = interp.lookup("port").expect("port variable not found");
    assert_eq!(port, LuaValue::Number(8080.0));
}

#[test]
fn test_require_nested_module() {
    let mut executor = Executor::new();
    let mut interp = LuaInterpreter::new();
    
    interp.add_module_search_path(PathBuf::from("fixtures/modules"));
    
    let code = r#"
        local math_utils = require("utils.math")
        result = math_utils.square(5)
    "#;
    
    let tokens = tokenize(code).expect("Failed to tokenize");
    let token_slice = TokenSlice::from(tokens.as_slice());
    let (_, block) = parse_lua(token_slice).expect("Failed to parse");
    
    let result = executor.execute_block(&block, &mut interp);
    assert!(result.is_ok(), "Execution failed: {:?}", result);
    
    let result_val = interp.lookup("result").expect("result variable not found");
    assert_eq!(result_val, LuaValue::Number(25.0));
}

#[test]
fn test_require_caching() {
    let mut executor = Executor::new();
    let mut interp = LuaInterpreter::new();
    
    interp.add_module_search_path(PathBuf::from("fixtures/modules"));
    
    let code = r#"
        local m1 = require("simple")
        local m2 = require("simple")
        -- Both should be the same object
        same = (m1 == m2)
    "#;
    
    let tokens = tokenize(code).expect("Failed to tokenize");
    let token_slice = TokenSlice::from(tokens.as_slice());
    let (_, block) = parse_lua(token_slice).expect("Failed to parse");
    
    let result = executor.execute_block(&block, &mut interp);
    assert!(result.is_ok(), "Execution failed: {:?}", result);
    
    // Note: Lua table equality is based on identity when comparing tables
    // so two references to the same cached module should be equal
    let same = interp.lookup("same").expect("same variable not found");
    assert_eq!(same, LuaValue::Boolean(true));
}

#[test]
fn test_module_loader_cached_count() {
    let interp = LuaInterpreter::new();
    
    // Before loading, cache should be empty
    let loader = interp.module_loader.borrow();
    assert_eq!(loader.cached_count(), 0);
}
