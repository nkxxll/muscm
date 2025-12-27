use muscm::interpreter::{Interpreter, Environment, SVal};
use muscm::ast::{Arena, SExpr};

fn main() {
    let mut env = Environment::new();
    
    // Test that builtins are registered
    assert!(env.lookup("+").is_some());
    assert!(env.lookup("display").is_some());
    assert!(env.lookup("car").is_some());
    
    println!("All builtins registered successfully!");
}
