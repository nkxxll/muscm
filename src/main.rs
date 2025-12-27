use muscm::parser::parse;
use muscm::interpreter::{Interpreter, Environment};
use muscm::lua_interpreter::LuaInterpreter;
use muscm::lua_parser::{tokenize, parse as parse_lua, TokenSlice};
use muscm::executor::Executor;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        run_scheme_default();
        return;
    }
    
    match args[1].as_str() {
        "lua" => {
            if args.len() < 3 {
                eprintln!("Usage: {} lua <file>", args[0]);
                std::process::exit(1);
            }
            run_lua(&args[2]);
        }
        _ => {
            run_scheme_default();
        }
    }
}

fn run_scheme_default() {
    // Test Phase 3: List Operations
    let input = r#"
(display "=== Phase 3: List Operations ===")
(newline)

(display "list? '(1 2 3): ")
(display (list? '(1 2 3)))
(newline)

(display "list? '(): ")
(display (list? '()))
(newline)

(display "list? 42: ")
(display (list? 42))
(newline)

(display "atom? 'hello: ")
(display (atom? 'hello))
(newline)

(display "atom? '(1 2): ")
(display (atom? '(1 2)))
(newline)

(display "pair? '(1 2): ")
(display (pair? '(1 2)))
(newline)

(display "null? '(): ")
(display (null? '()))
(newline)

(display "null? '(1): ")
(display (null? '(1)))
(newline)

(display "car '(1 2 3): ")
(display (car '(1 2 3)))
(newline)

(display "cdr '(1 2 3): ")
(display (cdr '(1 2 3)))
(newline)

(display "cons 0 '(1 2): ")
(display (cons 0 '(1 2)))
(newline)

(display "list 1 2 3: ")
(display (list 1 2 3))
(newline)

(display "length '(a b c d): ")
(display (length '(a b c d)))
(newline)

(display "append '(1 2) '(3 4): ")
(display (append '(1 2) '(3 4)))
(newline)

(display "append '() '(5 6): ")
(display (append '() '(5 6)))
(newline)

(display "append '(a) '(b) '(c): ")
(display (append '(a) '(b) '(c)))
(newline)
"#;

    match parse(input) {
        Ok((arena, node_ids)) => {
            let mut env = Environment::new();
            for node_id in node_ids {
                if let Some(expr) = arena.get(node_id) {
                    match Interpreter::eval(expr, &mut env, &arena) {
                        Ok(val) => println!("{}", val),
                        Err(e) => println!("ERROR: {}", e),
                    }
                }
            }
        }
        Err(e) => println!("Parse error: {}", e),
    }
}

fn run_lua(file_path: &str) {
    // Read the Lua file
    let code = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", file_path, e);
            std::process::exit(1);
        }
    };
    
    // Tokenize the code
    let tokens = match tokenize(&code) {
        Ok(tokens) => tokens,
        Err(e) => {
            eprintln!("Tokenize error: {}", e);
            std::process::exit(1);
        }
    };
    
    // Parse the code
    let token_slice = TokenSlice::from(tokens.as_slice());
    let block = match parse_lua(token_slice) {
        Ok((_, block)) => block,
        Err(e) => {
            eprintln!("Parse error: {:?}", e);
            std::process::exit(1);
        }
    };
    
    // Create a Lua interpreter and executor
    let mut interpreter = LuaInterpreter::new();
    
    // Add the script's directory to the module search paths
    let script_dir = std::path::Path::new(file_path)
        .canonicalize()
        .ok()
        .and_then(|p| p.parent().map(|parent| parent.to_path_buf()))
        .or_else(|| {
            // Fallback: use parent of the path, or current dir if no parent
            std::path::Path::new(file_path)
                .parent()
                .map(|p| std::path::PathBuf::from(p))
        });
    
    if let Some(dir) = script_dir {
        interpreter.add_module_search_path(dir);
    }
    
    let mut executor = Executor::new();
    
    // Execute the block
    match executor.execute_block(&block, &mut interpreter) {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Runtime error: {}", e);
            std::process::exit(1);
        }
    }
}
