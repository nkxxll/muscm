use crate::ast::{Arena, NodeId, SExpr};
use std::fmt;

/// Runtime value representation for Scheme
#[derive(Debug, Clone)]
pub enum SVal {
    /// Numeric values (integers and floats)
    Number(f64),
    /// String values
    String(String),
    /// Boolean values
    Bool(bool),
    /// Symbols/atoms (quoted or identifiers)
    Atom(String),
    /// Character values
    Char(char),
    /// Proper and improper lists
    List(Vec<SVal>),
    /// Vector type
    Vector(Vec<SVal>),
    /// Nil/void value
    Nil,
    /// Built-in procedure
    BuiltinProc {
        name: String,
        arity: Option<usize>, // None for variable arity
    },
    /// User-defined procedure
    UserProc {
        params: Vec<String>,
        body: Box<SExpr>,
    },
}

impl fmt::Display for SVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SVal::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            SVal::String(s) => write!(f, "\"{}\"", s),
            SVal::Bool(b) => write!(f, "#{}", if *b { 't' } else { 'f' }),
            SVal::Atom(a) => write!(f, "{}", a),
            SVal::Char(c) => write!(f, "#\\{}", c),
            SVal::List(items) => {
                write!(f, "(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            SVal::Vector(items) => {
                write!(f, "#(")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        write!(f, " ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, ")")
            }
            SVal::Nil => write!(f, "'()"),
            SVal::BuiltinProc { name, .. } => write!(f, "#<builtin:{}>", name),
            SVal::UserProc { .. } => write!(f, "#<procedure>"),
        }
    }
}

impl PartialEq for SVal {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SVal::Number(a), SVal::Number(b)) => a == b,
            (SVal::String(a), SVal::String(b)) => a == b,
            (SVal::Bool(a), SVal::Bool(b)) => a == b,
            (SVal::Atom(a), SVal::Atom(b)) => a == b,
            (SVal::Char(a), SVal::Char(b)) => a == b,
            (SVal::Nil, SVal::Nil) => true,
            _ => false,
        }
    }
}

/// Environment for variable bindings and nested scopes
#[derive(Debug, Clone)]
pub struct Environment {
    /// Current scope's variable bindings
    bindings: Vec<(String, SVal)>,
    /// Reference to parent environment for nested scopes
    parent: Option<Box<Environment>>,
}

impl Environment {
    /// Create a new root environment with built-in functions
    pub fn new() -> Self {
        let mut env = Environment {
            bindings: Vec::new(),
            parent: None,
        };

        // Register built-in procedures
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
        ];

        for (name, val) in builtins {
            env.define(name.to_string(), val);
        }

        env
    }

    /// Create a new child environment with a parent reference
    pub fn child(&self) -> Self {
        Environment {
            bindings: Vec::new(),
            parent: Some(Box::new(self.clone())),
        }
    }

    /// Define a variable in the current scope
    pub fn define(&mut self, name: String, value: SVal) {
        // Check if variable already exists in current scope
        for (n, v) in &mut self.bindings {
            if n == &name {
                *v = value;
                return;
            }
        }
        // If not found, add new binding
        self.bindings.push((name, value));
    }

    /// Look up a variable's value, checking parent scopes recursively
    pub fn lookup(&self, name: &str) -> Option<SVal> {
        // Check current scope
        for (n, v) in &self.bindings {
            if n == name {
                return Some(v.clone());
            }
        }
        // Check parent scope recursively
        if let Some(parent) = &self.parent {
            parent.lookup(name)
        } else {
            None
        }
    }

    /// Update an existing variable (must exist in current or parent scope)
    pub fn set(&mut self, name: &str, value: SVal) -> Result<(), String> {
        // Check current scope
        for (n, v) in &mut self.bindings {
            if n == name {
                *v = value;
                return Ok(());
            }
        }
        // Check parent scope recursively
        if let Some(parent) = &mut self.parent {
            parent.set(name, value)
        } else {
            Err(format!("Unbound variable: {}", name))
        }
    }
}

pub struct Interpreter;

impl Interpreter {
    /// Convert an SExpr to an SVal (for quoted expressions)
    fn sexpr_to_sval(expr: &SExpr, arena: &Arena) -> SVal {
        match expr {
            SExpr::Number(n) => SVal::Number(*n),
            SExpr::String(s) => SVal::String(s.clone()),
            SExpr::Bool(b) => SVal::Bool(*b),
            SExpr::Char(c) => SVal::Char(*c),
            SExpr::Atom(a) => SVal::Atom(a.clone()),
            SExpr::Quote(id) => {
                if let Some(node) = arena.get(*id) {
                    SVal::List(vec![
                        SVal::Atom("quote".to_string()),
                        Self::sexpr_to_sval(node, arena),
                    ])
                } else {
                    SVal::Nil
                }
            }
            SExpr::List(ids) => {
                let items: Vec<SVal> = ids.iter()
                    .filter_map(|id| arena.get(*id).map(|e| Self::sexpr_to_sval(e, arena)))
                    .collect();
                SVal::List(items)
            }
            SExpr::Vector(ids) => {
                let items: Vec<SVal> = ids.iter()
                    .filter_map(|id| arena.get(*id).map(|e| Self::sexpr_to_sval(e, arena)))
                    .collect();
                SVal::Vector(items)
            }
            _ => SVal::Nil, // Unquote, QuasiQuote, etc. become nil in simple implementation
        }
    }

    /// Check if value is truthy (everything except #f is truthy)
    fn is_truthy(val: &SVal) -> bool {
        !matches!(val, SVal::Bool(false))
    }

    /// Evaluate quote special form: (quote expr)
    fn eval_quote(ids: &[NodeId], arena: &Arena) -> Result<SVal, String> {
        if ids.len() != 2 {
            return Err("quote expects exactly 1 argument".to_string());
        }
        if let Some(expr) = arena.get(ids[1]) {
            Ok(Self::sexpr_to_sval(expr, arena))
        } else {
            Err("Invalid quote reference".to_string())
        }
    }

    /// Evaluate if special form: (if condition consequent alternative?)
    fn eval_if(ids: &[NodeId], env: &mut Environment, arena: &Arena) -> Result<SVal, String> {
        if ids.len() < 3 || ids.len() > 4 {
            return Err("if expects 2 or 3 arguments".to_string());
        }
        let cond_expr = arena.get(ids[1]).ok_or("Invalid if condition reference")?;
        let cond = Self::eval(cond_expr, env, arena)?;
        if Self::is_truthy(&cond) {
            let then_expr = arena.get(ids[2]).ok_or("Invalid if then reference")?;
            Self::eval(then_expr, env, arena)
        } else if ids.len() == 4 {
            let else_expr = arena.get(ids[3]).ok_or("Invalid if else reference")?;
            Self::eval(else_expr, env, arena)
        } else {
            Ok(SVal::Nil)
        }
    }

    /// Evaluate begin special form: (begin expr1 expr2 ... exprN)
    fn eval_begin(ids: &[NodeId], env: &mut Environment, arena: &Arena) -> Result<SVal, String> {
        let mut result = SVal::Nil;
        for id in &ids[1..] {
            if let Some(expr) = arena.get(*id) {
                result = Self::eval(expr, env, arena)?;
            }
        }
        Ok(result)
    }

    /// Evaluate define special form: (define name value) or (define (name params...) body)
    fn eval_define(ids: &[NodeId], env: &mut Environment, arena: &Arena) -> Result<SVal, String> {
        if ids.len() < 3 {
            return Err("define expects at least 2 arguments".to_string());
        }

        let name_expr = arena.get(ids[1]).ok_or("Invalid define name reference")?;
        match name_expr {
            // Simple variable definition: (define x 42)
            SExpr::Atom(name) => {
                let value_expr = arena.get(ids[2]).ok_or("Invalid define value reference")?;
                let value = Self::eval(value_expr, env, arena)?;
                env.define(name.clone(), value);
                Ok(SVal::Nil)
            }
            // Function definition: (define (name params...) body...)
            SExpr::List(sig_ids) if !sig_ids.is_empty() => {
                let func_expr = arena.get(sig_ids[0]).ok_or("Invalid function name reference")?;
                match func_expr {
                    SExpr::Atom(func_name) => {
                        let params: Result<Vec<String>, String> = sig_ids[1..]
                            .iter()
                            .filter_map(|id| arena.get(*id))
                            .map(|p| {
                                if let SExpr::Atom(s) = p {
                                    Ok(s.clone())
                                } else {
                                    Err("Invalid parameter".to_string())
                                }
                            })
                            .collect();
                        let params = params?;

                        // Combine remaining items as body (implicit begin)
                        let body = if ids.len() == 3 {
                            arena.get(ids[2]).ok_or("Invalid body reference")?.clone()
                        } else {
                            // Create a begin form with body expressions
                            // This is tricky - we need to create new SExpr nodes in the arena
                            let mut body_ids = vec![];
                            for body_id in &ids[2..] {
                                body_ids.push(*body_id);
                            }
                            SExpr::List(body_ids)
                        };

                        let func = SVal::UserProc {
                            params,
                            body: Box::new(body),
                        };
                        env.define(func_name.clone(), func);
                        Ok(SVal::Nil)
                    }
                    _ => Err("Invalid function definition".to_string()),
                }
            }
            _ => Err("Invalid define syntax".to_string()),
        }
    }

    /// Evaluate lambda special form: (lambda (params...) body...)
    fn eval_lambda(ids: &[NodeId], arena: &Arena) -> Result<SVal, String> {
        if ids.len() < 3 {
            return Err("lambda expects at least 2 arguments".to_string());
        }
        let params_expr = arena.get(ids[1]).ok_or("Invalid lambda params reference")?;
        let params = match params_expr {
            SExpr::List(ps_ids) => ps_ids
                .iter()
                .filter_map(|id| arena.get(*id))
                .map(|p| {
                    if let SExpr::Atom(s) = p {
                        Ok(s.clone())
                    } else {
                        Err("Invalid parameter".to_string())
                    }
                })
                .collect::<Result<Vec<String>, String>>()?,
            _ => return Err("lambda expects a parameter list".to_string()),
        };

        // Combine remaining items as body (implicit begin)
        let body = if ids.len() == 3 {
            arena.get(ids[2]).ok_or("Invalid lambda body reference")?.clone()
        } else {
            // Create list of body ids
            let mut body_ids = vec![];
            for body_id in &ids[2..] {
                body_ids.push(*body_id);
            }
            SExpr::List(body_ids)
        };

        Ok(SVal::UserProc {
            params,
            body: Box::new(body),
        })
    }

    /// Call a function value with arguments
    fn call_function(func: SVal, args: Vec<SVal>, env: &mut Environment, arena: &Arena) -> Result<SVal, String> {
        match func {
            SVal::BuiltinProc { name: fname, .. } => Self::apply_builtin(&fname, args, env),
            SVal::UserProc { params, body } => {
                if params.len() != args.len() {
                    return Err(format!(
                        "Function expects {} arguments, got {}",
                        params.len(),
                        args.len()
                    ));
                }

                // Create new environment for function call
                let mut call_env = env.child();
                for (param, arg) in params.iter().zip(args.iter()) {
                    call_env.define(param.clone(), arg.clone());
                }

                Self::eval(&body, &mut call_env, arena)
            }
            _ => Err(format!("Cannot call non-function value: {}", func)),
        }
    }

    /// Apply a built-in function
    fn apply_builtin(name: &str, args: Vec<SVal>, _env: &mut Environment) -> Result<SVal, String> {
        match name {
            // Arithmetic
            "+" => {
                let mut sum = 0.0;
                for arg in args {
                    match arg {
                        SVal::Number(n) => sum += n,
                        _ => return Err("+ expects numbers".to_string()),
                    }
                }
                Ok(SVal::Number(sum))
            }
            "-" => {
                if args.is_empty() {
                    return Err("- expects at least one argument".to_string());
                }
                match args[0] {
                    SVal::Number(first) => {
                        let mut result = first;
                        for arg in &args[1..] {
                            match arg {
                                SVal::Number(n) => result -= n,
                                _ => return Err("- expects numbers".to_string()),
                            }
                        }
                        Ok(SVal::Number(result))
                    }
                    _ => Err("- expects numbers".to_string()),
                }
            }
            "*" => {
                let mut product = 1.0;
                for arg in args {
                    match arg {
                        SVal::Number(n) => product *= n,
                        _ => return Err("* expects numbers".to_string()),
                    }
                }
                Ok(SVal::Number(product))
            }
            "/" => {
                if args.is_empty() {
                    return Err("/ expects at least one argument".to_string());
                }
                match args[0] {
                    SVal::Number(first) => {
                        let mut result = first;
                        for arg in &args[1..] {
                            match arg {
                                SVal::Number(n) => {
                                    if *n == 0.0 {
                                        return Err("Division by zero".to_string());
                                    }
                                    result /= n;
                                }
                                _ => return Err("/ expects numbers".to_string()),
                            }
                        }
                        Ok(SVal::Number(result))
                    }
                    _ => Err("/ expects numbers".to_string()),
                }
            }

            // Comparison
            "=" => {
                if args.len() != 2 {
                    return Err("= expects exactly 2 arguments".to_string());
                }
                match (&args[0], &args[1]) {
                    (SVal::Number(a), SVal::Number(b)) => Ok(SVal::Bool(a == b)),
                    (a, b) => Ok(SVal::Bool(a == b)),
                }
            }
            "<" => {
                if args.len() != 2 {
                    return Err("< expects exactly 2 arguments".to_string());
                }
                match (&args[0], &args[1]) {
                    (SVal::Number(a), SVal::Number(b)) => Ok(SVal::Bool(a < b)),
                    _ => Err("< expects numbers".to_string()),
                }
            }
            ">" => {
                if args.len() != 2 {
                    return Err("> expects exactly 2 arguments".to_string());
                }
                match (&args[0], &args[1]) {
                    (SVal::Number(a), SVal::Number(b)) => Ok(SVal::Bool(a > b)),
                    _ => Err("> expects numbers".to_string()),
                }
            }
            "<=" => {
                if args.len() != 2 {
                    return Err("<= expects exactly 2 arguments".to_string());
                }
                match (&args[0], &args[1]) {
                    (SVal::Number(a), SVal::Number(b)) => Ok(SVal::Bool(a <= b)),
                    _ => Err("<= expects numbers".to_string()),
                }
            }
            ">=" => {
                if args.len() != 2 {
                    return Err(">= expects exactly 2 arguments".to_string());
                }
                match (&args[0], &args[1]) {
                    (SVal::Number(a), SVal::Number(b)) => Ok(SVal::Bool(a >= b)),
                    _ => Err(">= expects numbers".to_string()),
                }
            }

            // Type predicates
            "number?" => {
                if args.len() != 1 {
                    return Err("number? expects exactly 1 argument".to_string());
                }
                Ok(SVal::Bool(matches!(args[0], SVal::Number(_))))
            }
            "symbol?" => {
                if args.len() != 1 {
                    return Err("symbol? expects exactly 1 argument".to_string());
                }
                Ok(SVal::Bool(matches!(args[0], SVal::Atom(_))))
            }
            "pair?" => {
                if args.len() != 1 {
                    return Err("pair? expects exactly 1 argument".to_string());
                }
                match &args[0] {
                    SVal::List(items) => Ok(SVal::Bool(!items.is_empty())),
                    _ => Ok(SVal::Bool(false)),
                }
            }
            "null?" => {
                if args.len() != 1 {
                    return Err("null? expects exactly 1 argument".to_string());
                }
                Ok(SVal::Bool(matches!(args[0], SVal::Nil)))
            }
            "list?" => {
                if args.len() != 1 {
                    return Err("list? expects exactly 1 argument".to_string());
                }
                match &args[0] {
                    SVal::List(_) | SVal::Nil => Ok(SVal::Bool(true)),
                    _ => Ok(SVal::Bool(false)),
                }
            }
            "atom?" => {
                if args.len() != 1 {
                    return Err("atom? expects exactly 1 argument".to_string());
                }
                match &args[0] {
                    SVal::Atom(_) => Ok(SVal::Bool(true)),
                    _ => Ok(SVal::Bool(false)),
                }
            }

            // List operations
            "car" => {
                if args.len() != 1 {
                    return Err("car expects exactly 1 argument".to_string());
                }
                match &args[0] {
                    SVal::List(items) if !items.is_empty() => Ok(items[0].clone()),
                    _ => Err("car expects a non-empty list".to_string()),
                }
            }
            "cdr" => {
                if args.len() != 1 {
                    return Err("cdr expects exactly 1 argument".to_string());
                }
                match &args[0] {
                    SVal::List(items) if !items.is_empty() => {
                        if items.len() == 1 {
                            Ok(SVal::Nil)
                        } else {
                            Ok(SVal::List(items[1..].to_vec()))
                        }
                    }
                    _ => Err("cdr expects a non-empty list".to_string()),
                }
            }
            "cons" => {
                if args.len() != 2 {
                    return Err("cons expects exactly 2 arguments".to_string());
                }
                match &args[1] {
                    SVal::List(items) => {
                        let mut new_list = vec![args[0].clone()];
                        new_list.extend(items.clone());
                        Ok(SVal::List(new_list))
                    }
                    SVal::Nil => Ok(SVal::List(vec![args[0].clone()])),
                    _ => Err("cons expects a list as second argument".to_string()),
                }
            }
            "list" => Ok(SVal::List(args)),
            "length" => {
                if args.len() != 1 {
                    return Err("length expects exactly 1 argument".to_string());
                }
                match &args[0] {
                    SVal::List(items) => Ok(SVal::Number(items.len() as f64)),
                    SVal::Nil => Ok(SVal::Number(0.0)),
                    _ => Err("length expects a list".to_string()),
                }
            }
            "append" => {
                // append: concatenate multiple lists
                // (append '(1 2) '(3 4)) -> (1 2 3 4)
                // (append '() '(1)) -> (1)
                if args.is_empty() {
                    return Ok(SVal::Nil);
                }

                let mut result = Vec::new();
                for (i, arg) in args.iter().enumerate() {
                    match arg {
                        SVal::List(items) => {
                            result.extend(items.clone());
                        }
                        SVal::Nil => {
                            // nil contributes nothing to append
                        }
                        _ => {
                            // Last argument can be non-list for improper lists
                            if i == args.len() - 1 {
                                if !result.is_empty() {
                                    // This would create improper list, not standard
                                    return Err("append: improper list not supported".to_string());
                                }
                                return Ok(arg.clone());
                            }
                            return Err("append expects lists as arguments".to_string());
                        }
                    }
                }

                if result.is_empty() {
                    Ok(SVal::Nil)
                } else {
                    Ok(SVal::List(result))
                }
            }

            // I/O
            "display" => {
                for arg in args {
                    print!("{}", arg);
                }
                Ok(SVal::Nil)
            }
            "newline" => {
                println!();
                Ok(SVal::Nil)
            }

            _ => Err(format!("Unknown function: {}", name)),
        }
    }

    /// Evaluate an S-expression in the given environment
    pub fn eval(expr: &SExpr, env: &mut Environment, arena: &Arena) -> Result<SVal, String> {
        match expr {
            // Literals evaluate to themselves
            SExpr::Number(n) => Ok(SVal::Number(*n)),
            SExpr::Bool(b) => Ok(SVal::Bool(*b)),
            SExpr::String(s) => Ok(SVal::String(s.clone())),
            SExpr::Char(c) => Ok(SVal::Char(*c)),

            // Atoms are looked up in the environment
            SExpr::Atom(name) => env
                .lookup(name)
                .ok_or_else(|| format!("Unbound variable: {}", name)),

            // Quote: return the expression as a literal value
            SExpr::Quote(id) => {
                if let Some(node) = arena.get(*id) {
                    Ok(Self::sexpr_to_sval(node, arena))
                } else {
                    Err("Invalid quote reference".to_string())
                }
            }

            // Non-empty lists: function calls and special forms
            SExpr::List(ids) => {
                if ids.is_empty() {
                    return Ok(SVal::Nil);
                }
                let first_expr = arena.get(ids[0]).ok_or("Invalid list head reference")?;
                match first_expr {
                    SExpr::Atom(name) => {
                        // Special forms
                        match name.as_str() {
                            "quote" => Self::eval_quote(&ids, arena),
                            "if" => Self::eval_if(&ids, env, arena),
                            "define" => Self::eval_define(&ids, env, arena),
                            "begin" => Self::eval_begin(&ids, env, arena),
                            "lambda" => Self::eval_lambda(&ids, arena),

                            // Regular function call
                            _ => {
                                let func = Self::eval(first_expr, env, arena)?;
                                let args: Result<Vec<SVal>, String> =
                                    ids[1..].iter()
                                        .filter_map(|id| arena.get(*id))
                                        .map(|arg| Self::eval(arg, env, arena))
                                        .collect();
                                let args = args?;

                                Self::call_function(func, args, env, arena)
                            }
                        }
                    }
                    // If the first element is not an atom, evaluate it
                    _ => {
                        let func = Self::eval(first_expr, env, arena)?;
                        let args: Result<Vec<SVal>, String> =
                            ids[1..].iter()
                                .filter_map(|id| arena.get(*id))
                                .map(|arg| Self::eval(arg, env, arena))
                                .collect();
                        let args = args?;

                        Self::call_function(func, args, env, arena)
                    }
                }
            }

            // Not yet supported
            SExpr::Vector(_) => Err("Vectors not yet supported".to_string()),
            SExpr::QuasiQuote(_) => Err("Quasi-quote not yet supported".to_string()),
            SExpr::Unquote(_) => Err("Unquote not in quote context".to_string()),
            SExpr::UnquoteSplicing(_) => Err("Unquote-splicing not in quote context".to_string()),
        }
    }
}
