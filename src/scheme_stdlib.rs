use crate::interpreter::{Environment, SVal};

/// Register all built-in Scheme functions in the environment
pub fn register_stdlib(env: &mut Environment) {
    let builtins = vec![
        // Arithmetic
        (
            "+",
            SVal::BuiltinProc {
                name: "+".to_string(),
                arity: None,
            },
        ),
        (
            "-",
            SVal::BuiltinProc {
                name: "-".to_string(),
                arity: None,
            },
        ),
        (
            "*",
            SVal::BuiltinProc {
                name: "*".to_string(),
                arity: None,
            },
        ),
        (
            "/",
            SVal::BuiltinProc {
                name: "/".to_string(),
                arity: None,
            },
        ),
        // Comparison
        (
            "=",
            SVal::BuiltinProc {
                name: "=".to_string(),
                arity: Some(2),
            },
        ),
        (
            "<",
            SVal::BuiltinProc {
                name: "<".to_string(),
                arity: Some(2),
            },
        ),
        (
            ">",
            SVal::BuiltinProc {
                name: ">".to_string(),
                arity: Some(2),
            },
        ),
        (
            "<=",
            SVal::BuiltinProc {
                name: "<=".to_string(),
                arity: Some(2),
            },
        ),
        (
            ">=",
            SVal::BuiltinProc {
                name: ">=".to_string(),
                arity: Some(2),
            },
        ),
        // Type predicates
        (
            "number?",
            SVal::BuiltinProc {
                name: "number?".to_string(),
                arity: Some(1),
            },
        ),
        (
            "symbol?",
            SVal::BuiltinProc {
                name: "symbol?".to_string(),
                arity: Some(1),
            },
        ),
        (
            "pair?",
            SVal::BuiltinProc {
                name: "pair?".to_string(),
                arity: Some(1),
            },
        ),
        (
            "null?",
            SVal::BuiltinProc {
                name: "null?".to_string(),
                arity: Some(1),
            },
        ),
        // List operations
        (
            "car",
            SVal::BuiltinProc {
                name: "car".to_string(),
                arity: Some(1),
            },
        ),
        (
            "cdr",
            SVal::BuiltinProc {
                name: "cdr".to_string(),
                arity: Some(1),
            },
        ),
        (
            "cons",
            SVal::BuiltinProc {
                name: "cons".to_string(),
                arity: Some(2),
            },
        ),
        (
            "list",
            SVal::BuiltinProc {
                name: "list".to_string(),
                arity: None,
            },
        ),
        (
            "length",
            SVal::BuiltinProc {
                name: "length".to_string(),
                arity: Some(1),
            },
        ),
        (
            "list?",
            SVal::BuiltinProc {
                name: "list?".to_string(),
                arity: Some(1),
            },
        ),
        (
            "append",
            SVal::BuiltinProc {
                name: "append".to_string(),
                arity: None,
            },
        ),
        (
            "atom?",
            SVal::BuiltinProc {
                name: "atom?".to_string(),
                arity: Some(1),
            },
        ),
        // I/O
        (
            "display",
            SVal::BuiltinProc {
                name: "display".to_string(),
                arity: None,
            },
        ),
        (
            "newline",
            SVal::BuiltinProc {
                name: "newline".to_string(),
                arity: Some(0),
            },
        ),
        // Mathematical functions
        (
            "abs",
            SVal::BuiltinProc {
                name: "abs".to_string(),
                arity: Some(1),
            },
        ),
        (
            "floor",
            SVal::BuiltinProc {
                name: "floor".to_string(),
                arity: Some(1),
            },
        ),
        (
            "ceiling",
            SVal::BuiltinProc {
                name: "ceiling".to_string(),
                arity: Some(1),
            },
        ),
        (
            "round",
            SVal::BuiltinProc {
                name: "round".to_string(),
                arity: Some(1),
            },
        ),
        (
            "truncate",
            SVal::BuiltinProc {
                name: "truncate".to_string(),
                arity: Some(1),
            },
        ),
        (
            "sqrt",
            SVal::BuiltinProc {
                name: "sqrt".to_string(),
                arity: Some(1),
            },
        ),
        (
            "sin",
            SVal::BuiltinProc {
                name: "sin".to_string(),
                arity: Some(1),
            },
        ),
        (
            "cos",
            SVal::BuiltinProc {
                name: "cos".to_string(),
                arity: Some(1),
            },
        ),
        (
            "tan",
            SVal::BuiltinProc {
                name: "tan".to_string(),
                arity: Some(1),
            },
        ),
        (
            "log",
            SVal::BuiltinProc {
                name: "log".to_string(),
                arity: Some(1),
            },
        ),
        (
            "exp",
            SVal::BuiltinProc {
                name: "exp".to_string(),
                arity: Some(1),
            },
        ),
        (
            "min",
            SVal::BuiltinProc {
                name: "min".to_string(),
                arity: None,
            },
        ),
        (
            "max",
            SVal::BuiltinProc {
                name: "max".to_string(),
                arity: None,
            },
        ),
        // String functions
        (
            "string?",
            SVal::BuiltinProc {
                name: "string?".to_string(),
                arity: Some(1),
            },
        ),
        (
            "string-length",
            SVal::BuiltinProc {
                name: "string-length".to_string(),
                arity: Some(1),
            },
        ),
        (
            "substring",
            SVal::BuiltinProc {
                name: "substring".to_string(),
                arity: Some(3),
            },
        ),
        (
            "string-upcase",
            SVal::BuiltinProc {
                name: "string-upcase".to_string(),
                arity: Some(1),
            },
        ),
        (
            "string-downcase",
            SVal::BuiltinProc {
                name: "string-downcase".to_string(),
                arity: Some(1),
            },
        ),
        (
            "string-append",
            SVal::BuiltinProc {
                name: "string-append".to_string(),
                arity: None,
            },
        ),
        (
            "string->number",
            SVal::BuiltinProc {
                name: "string->number".to_string(),
                arity: Some(1),
            },
        ),
        (
            "number->string",
            SVal::BuiltinProc {
                name: "number->string".to_string(),
                arity: Some(1),
            },
        ),
    ];

    for (name, val) in builtins {
        env.define(name.to_string(), val);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdlib_registration() {
        let env = Environment::new();

        // Verify key builtins are registered
        assert!(env.lookup("+").is_some());
        assert!(env.lookup("-").is_some());
        assert!(env.lookup("*").is_some());
        assert!(env.lookup("/").is_some());
        assert!(env.lookup("=").is_some());
        assert!(env.lookup("<").is_some());
        assert!(env.lookup(">").is_some());
        assert!(env.lookup("number?").is_some());
        assert!(env.lookup("car").is_some());
        assert!(env.lookup("cdr").is_some());
        assert!(env.lookup("cons").is_some());
        assert!(env.lookup("list").is_some());
        assert!(env.lookup("length").is_some());
        assert!(env.lookup("append").is_some());
        assert!(env.lookup("display").is_some());
        assert!(env.lookup("newline").is_some());

        // Verify math functions are registered
        assert!(env.lookup("abs").is_some());
        assert!(env.lookup("floor").is_some());
        assert!(env.lookup("ceiling").is_some());
        assert!(env.lookup("round").is_some());
        assert!(env.lookup("sqrt").is_some());
        assert!(env.lookup("sin").is_some());
        assert!(env.lookup("cos").is_some());
        assert!(env.lookup("min").is_some());
        assert!(env.lookup("max").is_some());

        // Verify string functions are registered
        assert!(env.lookup("string?").is_some());
        assert!(env.lookup("string-length").is_some());
        assert!(env.lookup("substring").is_some());
        assert!(env.lookup("string-upcase").is_some());
        assert!(env.lookup("string-downcase").is_some());
        assert!(env.lookup("string-append").is_some());
        assert!(env.lookup("string->number").is_some());
        assert!(env.lookup("number->string").is_some());
    }
}
