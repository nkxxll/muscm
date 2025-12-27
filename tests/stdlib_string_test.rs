use muscm::interpreter::{Environment, Interpreter, SVal};
use muscm::parser::parse;

#[test]
fn test_string_predicate() {
    let mut env = Environment::new();

    let (arena, nodes) = parse("(string? \"hello\")").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Bool(true))));

    let (arena, nodes) = parse("(string? 42)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Bool(false))));
}

#[test]
fn test_string_length() {
    let mut env = Environment::new();

    let (arena, nodes) = parse("(string-length \"hello\")").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if n == 5.0));

    let (arena, nodes) = parse("(string-length \"\")").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if n == 0.0));
}

#[test]
fn test_substring() {
    let mut env = Environment::new();

    let (arena, nodes) = parse("(substring \"hello\" 0 5)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::String(ref s)) if s == "hello"));

    let (arena, nodes) = parse("(substring \"hello\" 1 4)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::String(ref s)) if s == "ell"));
}

#[test]
fn test_string_case() {
    let mut env = Environment::new();

    let (arena, nodes) = parse("(string-upcase \"Hello\")").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::String(ref s)) if s == "HELLO"));

    let (arena, nodes) = parse("(string-downcase \"Hello\")").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::String(ref s)) if s == "hello"));
}

#[test]
fn test_string_append() {
    let mut env = Environment::new();

    // Note: The tokenizer loses whitespace in strings, so we use a different separator
    let (arena, nodes) = parse("(string-append \"hello\" \"-\" \"world\")").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::String(ref s)) if s == "hello-world"));

    let (arena, nodes) = parse("(string-append \"foo\" \"bar\" \"baz\")").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::String(ref s)) if s == "foobarbaz"));

    let (arena, nodes) = parse("(string-append)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::String(ref s)) if s == ""));
}

#[test]
fn test_string_to_number() {
    let mut env = Environment::new();

    let (arena, nodes) = parse("(string->number \"42\")").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if n == 42.0));

    let (arena, nodes) = parse("(string->number \"3.14\")").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if (n - 3.14).abs() < 0.001));

    let (arena, nodes) = parse("(string->number \"invalid\")").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Bool(false))));
}

#[test]
fn test_number_to_string() {
    let mut env = Environment::new();

    let (arena, nodes) = parse("(number->string 42)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::String(ref s)) if s == "42"));

    let (arena, nodes) = parse("(number->string 3.14)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::String(ref s)) if s == "3.14"));
}
