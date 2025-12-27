# Error Types Quick Reference

## Using LuaError in Your Code

### Import

```rust
use muscm::error_types::{LuaError, LuaResult};
// or use the re-exported types
use muscm::{LuaError, LuaResult};
```

## Creating Errors

### Parse Errors (with location info)

```rust
// Use when parsing fails at a specific location
let err = LuaError::parse("unexpected token", 42, 15);
// Produces: "Parse error at 42:15: unexpected token"
```

### Runtime Errors (with execution context)

```rust
// Use for runtime failures with execution context
let err = LuaError::runtime("invalid operation", "table assignment");
// Produces: "Runtime error (table assignment): invalid operation"
```

### Type Errors

```rust
// Use when type checking fails
let err = LuaError::type_error("number", "string", "math.abs");
// Produces: "Type error in math.abs: expected number, got string"
```

### Value Errors

```rust
// Use for value validation failures
let err = LuaError::value("invalid table key");
// Produces: "Value error: invalid table key"
```

### File Errors

```rust
// Use for file I/O failures
let err = LuaError::file("modules/lib.lua", "permission denied");
// Produces: "File error (modules/lib.lua): permission denied"
```

### Module Errors

```rust
// Use for module loading failures
let err = LuaError::module("mylib", "circular dependency detected");
// Produces: "Module error (mylib): circular dependency detected"
```

### Token Errors

```rust
// Use for tokenization failures
let err = LuaError::token("invalid number format", 125);
// Produces: "Token error at position 125: invalid number format"
```

### User Errors (from error() function)

```rust
// Use for user-raised errors
let err = LuaError::user("custom error message", 1);
// level: stack level for error reporting
```

### Argument Count Errors

```rust
// Use for function argument validation
let err = LuaError::arg_count("ipairs", 1, 5);
// Produces: "Function ipairs expects 1 argument(s), got 5"
```

### Index Errors

```rust
// Use when indexing fails
let err = LuaError::index("string", "function");
// Produces: "Cannot index string with function"
```

### Call Errors

```rust
// Use when attempting to call non-callable
let err = LuaError::call("number");
// Produces: "Attempt to call number (not a function)"
```

### Control Flow Errors

```rust
// Break outside loop
let err = LuaError::BreakOutsideLoop;
// Produces: "break statement outside loop"

// Undefined label
let err = LuaError::UndefinedLabel { label: "skip".to_string() };
// Produces: "undefined label: skip"

// Division by zero
let err = LuaError::DivisionByZero;
// Produces: "division by zero"
```

## Using LuaResult

```rust
// Type alias for Result<T, LuaError>
fn my_function() -> LuaResult<String> {
    if some_condition {
        Ok("success".to_string())
    } else {
        Err(LuaError::value("something went wrong"))
    }
}
```

## Error Handling Patterns

### Pattern Matching

```rust
match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(LuaError::TypeError { expected, got, function }) => {
        eprintln!("Type error in {}: expected {}, got {}", function, expected, got);
    }
    Err(LuaError::FileError { path, reason }) => {
        eprintln!("Failed to read {}: {}", path, reason);
    }
    Err(e) => eprintln!("Error: {}", e),
}
```

### Error Categorization

```rust
match error.category() {
    "parse" => handle_parse_error(),
    "runtime" => handle_runtime_error(),
    "type" => handle_type_error(),
    "file" => handle_file_error(),
    "module" => handle_module_error(),
    "argument" => handle_arg_error(),
    _ => handle_other(),
}
```

### Get Formatted Message

```rust
let error = LuaError::parse("unexpected EOF", 100, 42);
let msg = error.message();
println!("{}", msg);  // "Parse error at 100:42: unexpected EOF"
```

### Using ? Operator

```rust
fn validate_args(args: &[LuaValue]) -> LuaResult<()> {
    if args.is_empty() {
        return Err(LuaError::arg_count("my_func", 1, 0));
    }
    
    if let LuaValue::Number(n) = &args[0] {
        Ok(())
    } else {
        Err(LuaError::type_error("number", "other", "my_func"))
    }
}

fn process() -> LuaResult<i32> {
    validate_args(&args)?;  // Propagate error
    Ok(42)
}
```

### Error Conversion

```rust
// For legacy code returning Result<T, String>
fn legacy_code() -> Result<i32, String> {
    Err("old style error".to_string())
}

// Wrap in custom error
let result = legacy_code()
    .map_err(|msg| LuaError::runtime(msg, "legacy call"))?;
```

## Display and Formatting

```rust
let error = LuaError::type_error("string", "number", "tostring");

// Using Display trait
println!("Error: {}", error);

// In log messages
log::error!("Operation failed: {}", error);

// As String
let error_msg = error.to_string();
```

## Extracting Error Information

```rust
match error {
    LuaError::ParseError { message, line, column } => {
        println!("Parse error at {}:{}: {}", line, column, message);
    }
    LuaError::RuntimeError { message, context } => {
        println!("Runtime error during {}: {}", context, message);
    }
    LuaError::TypeError { expected, got, function } => {
        println!("{}: expected {}, got {}", function, expected, got);
    }
    _ => {}
}
```

## Testing with Errors

```rust
#[test]
fn test_error_creation() {
    let err = LuaError::value("test error");
    assert_eq!(err.category(), "value");
    assert!(err.message().contains("test error"));
}

#[test]
fn test_error_matching() {
    let err = LuaError::arg_count("func", 2, 5);
    match err {
        LuaError::ArgumentCountError { function, expected, got } => {
            assert_eq!(function, "func");
            assert_eq!(expected, 2);
            assert_eq!(got, 5);
        }
        _ => panic!("Wrong error type"),
    }
}

#[test]
fn test_result_propagation() {
    fn returns_error() -> LuaResult<()> {
        Err(LuaError::value("test"))
    }
    
    let result = returns_error();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.category(), "value");
}
```

## Common Patterns

### Argument Validation

```rust
fn validate_func_args(name: &str, args: &[LuaValue], min: usize, max: usize) -> LuaResult<()> {
    if args.len() < min || args.len() > max {
        return Err(LuaError::arg_count(name, min, args.len()));
    }
    Ok(())
}

fn my_builtin(args: Vec<LuaValue>) -> LuaResult<LuaValue> {
    validate_func_args("my_builtin", &args, 1, 2)?;
    // Continue processing
    Ok(LuaValue::Nil)
}
```

### Type Checking

```rust
fn get_number(args: &[LuaValue], index: usize) -> LuaResult<f64> {
    if index >= args.len() {
        return Err(LuaError::arg_count("get_number", index + 1, args.len()));
    }
    
    match &args[index] {
        LuaValue::Number(n) => Ok(*n),
        _ => Err(LuaError::type_error("number", args[index].type_name(), "get_number")),
    }
}
```

### Chainable Operations

```rust
fn process_data(args: Vec<LuaValue>) -> LuaResult<LuaValue> {
    let value = get_number(&args, 0)?;
    if value < 0.0 {
        return Err(LuaError::value("value must be non-negative"));
    }
    Ok(LuaValue::Number(value * 2.0))
}
```

## Migration from String Errors

### Before (Old Style)

```rust
fn old_function(args: Vec<LuaValue>) -> Result<LuaValue, String> {
    if args.is_empty() {
        return Err("Expected at least 1 argument".to_string());
    }
    match &args[0] {
        LuaValue::Number(n) => Ok(LuaValue::Number(n * 2.0)),
        _ => Err(format!("Expected number, got {}", args[0].type_name())),
    }
}
```

### After (New Style)

```rust
fn new_function(args: Vec<LuaValue>) -> LuaResult<LuaValue> {
    if args.is_empty() {
        return Err(LuaError::arg_count("new_function", 1, 0));
    }
    match &args[0] {
        LuaValue::Number(n) => Ok(LuaValue::Number(n * 2.0)),
        _ => Err(LuaError::type_error("number", args[0].type_name(), "new_function")),
    }
}
```

## Best Practices

1. **Use specific error variants** - Don't just use `ValueError`
2. **Include context** - Help users understand where the error occurred
3. **Be consistent** - Use the same error type across similar situations
4. **Test error paths** - Add tests for error cases
5. **Document errors** - Note what errors a function can return
6. **Propagate with ?** - Use the ? operator for clean error propagation

## Error Variants Reference

| Variant | Use Case | Example |
|---------|----------|---------|
| ParseError | Parse failures with location | Unterminated string |
| RuntimeError | Runtime failures with context | Invalid operation |
| TypeError | Type mismatches | Wrong argument type |
| ValueError | Value validation failures | Out of range |
| FileError | File I/O failures | File not found |
| ModuleError | Module loading failures | Circular dependency |
| TokenError | Tokenization failures | Invalid token |
| UserError | User-raised errors | error() function |
| BreakOutsideLoop | Control flow error | break outside loop |
| UndefinedLabel | Label not found | goto undefined |
| ArgumentCountError | Wrong arg count | Too few arguments |
| DivisionByZero | Arithmetic error | x / 0 |
| IndexError | Indexing failure | Index non-table |
| CallError | Calling non-function | Call non-callable |
