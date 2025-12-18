use muscm::parser::parse_tokens_to_ast;
use muscm::tokenizer::tokenize_string;

fn main() {
    // Example: tokenize a simple Scheme expression
    let input = "(define (factorial n) (if (= n 0) 1 (* n (factorial (- n 1)))))";
    let tokens = tokenize_string(input);
    println!("Tokens for: {}", input);
    println!("Count: {}", tokens.len());
    for token in tokens {
        println!("  {:?}", token);
    }
}
