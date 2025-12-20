use muscm::parser::parse;
use muscm::interpreter::{Interpreter, Environment};

fn main() {
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
        Ok(exprs) => {
            let mut env = Environment::new();
            for expr in exprs {
                match Interpreter::eval(&expr, &mut env) {
                    Ok(val) => println!("{}", val),
                    Err(e) => println!("ERROR: {}", e),
                }
            }
        }
        Err(e) => println!("Parse error: {}", e),
    }
}
