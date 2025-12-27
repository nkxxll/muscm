use muscm::interpreter::{Interpreter, Environment, SVal};
use muscm::parser::parse;

#[test]
fn test_abs() {
    let mut env = Environment::new();
    
    let (arena, nodes) = parse("(abs -5)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if n == 5.0));
    
    let (arena, nodes) = parse("(abs 3.5)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if n == 3.5));
}

#[test]
fn test_floor_ceiling() {
    let mut env = Environment::new();
    
    let (arena, nodes) = parse("(floor 3.7)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if n == 3.0));
    
    let (arena, nodes) = parse("(ceiling 3.2)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if n == 4.0));
}

#[test]
fn test_round_truncate() {
    let mut env = Environment::new();
    
    let (arena, nodes) = parse("(round 3.6)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if n == 4.0));
    
    let (arena, nodes) = parse("(truncate 3.9)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if n == 3.0));
}

#[test]
fn test_sqrt() {
    let mut env = Environment::new();
    
    let (arena, nodes) = parse("(sqrt 16)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if (n - 4.0).abs() < 0.001));
}

#[test]
fn test_sqrt_negative_error() {
    let mut env = Environment::new();
    
    let (arena, nodes) = parse("(sqrt -1)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(result.is_err());
}

#[test]
fn test_trigonometric() {
    let mut env = Environment::new();
    
    let (arena, nodes) = parse("(sin 0)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if n.abs() < 0.001));
    
    let (arena, nodes) = parse("(cos 0)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if (n - 1.0).abs() < 0.001));
}

#[test]
fn test_min_max() {
    let mut env = Environment::new();
    
    let (arena, nodes) = parse("(min 3 1 4 1 5)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if n == 1.0));
    
    let (arena, nodes) = parse("(max 3 1 4 1 5)").unwrap();
    let result = Interpreter::eval(arena.get(nodes[0]).unwrap(), &mut env, &arena);
    assert!(matches!(result, Ok(SVal::Number(n)) if n == 5.0));
}
