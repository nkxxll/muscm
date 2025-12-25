//! Simple S-expression parser for Scheme
//! Converts tokens into an AST of nested S-expressions

use crate::ast::SExpr;
use crate::tokenizer::{tokenize_string, Token, TokenType};
use std::fmt;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error at line {}: {}", self.line, self.message)
    }
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            let token = self.tokens[self.pos].clone();
            self.pos += 1;
            Some(token)
        } else {
            None
        }
    }

    fn current_line(&self) -> usize {
        self.peek().map(|t| t.line).unwrap_or(0)
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            line: self.current_line(),
        }
    }

    fn parse_string(&mut self) -> Result<SExpr, ParseError> {
        // Opening quote already consumed
        // todo: this is not good string parsing because white space is thrown
        // this is because the tokenizer should have token string
        let mut content = String::new();
        let mut first = true;

        loop {
            match self.peek() {
                Some(Token {
                    token_type: TokenType::DQuote,
                    ..
                }) => {
                    self.consume();
                    return Ok(SExpr::String(content));
                }
                Some(token) => {
                    if !first {
                        content.push(' ');
                    }
                    content.push_str(&token.literal);
                    self.consume();
                    first = false;
                }
                None => return Err(self.error("Unterminated string")),
            }
        }
    }

    fn parse_list(&mut self) -> Result<SExpr, ParseError> {
        // Opening paren already consumed
        let mut items = Vec::new();

        loop {
            match self.peek() {
                Some(Token {
                    token_type: TokenType::RParen,
                    ..
                }) => {
                    self.consume();
                    return Ok(SExpr::List(items));
                }
                Some(Token {
                    token_type: TokenType::Dot,
                    ..
                }) => {
                    // Improper list (a . b)
                    if items.is_empty() {
                        return Err(self.error("Unexpected dot"));
                    }
                    self.consume();
                    let cdr = self.parse_expr()?;

                    // Convert (a . b) to (a . b) by creating improper list
                    match self.peek() {
                        Some(Token {
                            token_type: TokenType::RParen,
                            ..
                        }) => {
                            self.consume();
                            // Build improper list
                            items.push(cdr);
                            return Ok(SExpr::List(items));
                        }
                        _ => return Err(self.error("Expected ) after dot notation")),
                    }
                }
                _ => {
                    items.push(self.parse_expr()?);
                }
            }
        }
    }

    fn parse_vector(&mut self) -> Result<SExpr, ParseError> {
        // #( already consumed
        let mut items = Vec::new();

        loop {
            match self.peek() {
                Some(Token {
                    token_type: TokenType::RParen,
                    ..
                }) => {
                    self.consume();
                    return Ok(SExpr::Vector(items));
                }
                _ => {
                    items.push(self.parse_expr()?);
                }
            }
        }
    }

    fn parse_sharp_const(&self, literal: &str) -> Result<SExpr, ParseError> {
        match literal {
            "#t" => Ok(SExpr::Bool(true)),
            "#f" => Ok(SExpr::Bool(false)),
            s if s.starts_with("#\\") => {
                let char_part = &s[2..];
                let c = match char_part {
                    "space" => ' ',
                    "newline" => '\n',
                    "tab" => '\t',
                    "return" => '\r',
                    s if s.len() == 1 => s.chars().next().unwrap(),
                    _ => return Err(self.error(&format!("Unknown character literal: {}", s))),
                };
                Ok(SExpr::Char(c))
            }
            _ => Err(self.error(&format!("Unknown sharp constant: {}", literal))),
        }
    }

    fn parse_atom(&self, literal: &str) -> Result<SExpr, ParseError> {
        // Try to parse as number
        if let Ok(n) = literal.parse::<f64>() {
            return Ok(SExpr::Number(n));
        }

        // Otherwise it's an atom
        Ok(SExpr::Atom(literal.to_string()))
    }

    fn parse_expr(&mut self) -> Result<SExpr, ParseError> {
        match self.consume() {
            Some(Token {
                token_type: TokenType::LParen,
                ..
            }) => self.parse_list(),

            Some(Token {
                token_type: TokenType::DQuote,
                ..
            }) => self.parse_string(),

            Some(Token {
                token_type: TokenType::Quote,
                ..
            }) => {
                let expr = self.parse_expr()?;
                Ok(SExpr::Quote(Box::new(expr)))
            }

            Some(Token {
                token_type: TokenType::BQuote,
                ..
            }) => {
                let expr = self.parse_expr()?;
                Ok(SExpr::QuasiQuote(Box::new(expr)))
            }

            Some(Token {
                token_type: TokenType::Comma,
                ..
            }) => {
                let expr = self.parse_expr()?;
                Ok(SExpr::Unquote(Box::new(expr)))
            }

            Some(Token {
                token_type: TokenType::AtMark,
                ..
            }) => {
                let expr = self.parse_expr()?;
                Ok(SExpr::UnquoteSplicing(Box::new(expr)))
            }

            Some(Token {
                token_type: TokenType::Vec,
                ..
            }) => self.parse_vector(),

            Some(Token {
                token_type: TokenType::Atom,
                literal,
                ..
            }) => self.parse_atom(&literal),

            Some(Token {
                token_type: TokenType::SharpConst,
                literal,
                ..
            }) => self.parse_sharp_const(&literal),

            Some(Token {
                token_type: TokenType::Eof,
                ..
            }) => Err(self.error("Unexpected EOF")),

            _ => Err(self.error("Unexpected token")),
        }
    }

    pub fn parse(&mut self) -> Result<Vec<SExpr>, ParseError> {
        let mut exprs = Vec::new();

        loop {
            match self.peek() {
                Some(Token {
                    token_type: TokenType::Eof,
                    ..
                }) => break,
                None => break,
                _ => {
                    exprs.push(self.parse_expr()?);
                }
            }
        }

        Ok(exprs)
    }
}

pub fn parse(input: &str) -> Result<Vec<SExpr>, ParseError> {
    let tokens = tokenize_string(input);
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_list() {
        let result = parse("(+ 1 2)").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            SExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[1], SExpr::Number(1.0));
                assert_eq!(items[2], SExpr::Number(2.0));
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_parse_atom() {
        let result = parse("hello").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], SExpr::Atom("hello".to_string()));
    }

    #[test]
    fn test_parse_number() {
        let result = parse("42").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], SExpr::Number(42.0));
    }

    #[test]
    fn test_parse_quote() {
        let result = parse("'hello").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            SExpr::Quote(e) => {
                assert_eq!(**e, SExpr::Atom("hello".to_string()));
            }
            _ => panic!("Expected quote"),
        }
    }

    #[test]
    fn test_parse_bool() {
        let result = parse("#t #f").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], SExpr::Bool(true));
        assert_eq!(result[1], SExpr::Bool(false));
    }

    #[test]
    fn test_parse_string() {
        let result = parse("\"hello world\"").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], SExpr::String("hello world".to_string()));
    }

    #[test]
    fn test_parse_nested_list() {
        let result = parse("(define (square x) (* x x))").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            SExpr::List(items) => {
                assert_eq!(items.len(), 3);
                assert_eq!(items[0], SExpr::Atom("define".to_string()));
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_parse_vector() {
        let result = parse("#(1 2 3)").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            SExpr::Vector(items) => {
                assert_eq!(items.len(), 3);
            }
            _ => panic!("Expected vector"),
        }
    }

    #[test]
    fn test_parse_multiple_exprs() {
        let result = parse("42 hello (+ 1 2)").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], SExpr::Number(42.0));
        assert_eq!(result[1], SExpr::Atom("hello".to_string()));
    }

    #[test]
    fn test_parse_backquote() {
        let result = parse("`(a ,b)").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            SExpr::QuasiQuote(_) => {}
            _ => panic!("Expected quasi-quote"),
        }
    }

    #[test]
    fn test_parse_unquote_splicing() {
        let result = parse("(,@items)").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            SExpr::List(items) => {
                assert_eq!(items.len(), 1);
                match &items[0] {
                    SExpr::UnquoteSplicing(_) => {}
                    _ => panic!("Expected unquote-splicing"),
                }
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    fn test_parse_scheme_read_file() {
        let input = r#"(define (print-file filename)
          (call-with-input-file filename
            (lambda (port)
              (let loop ()
                (let ((line (read-line port)))
                  (if (eof-object? line)
                      'done
                      (begin
                        (display line)
                        (newline)
                        (loop))))))))

        ;; Example usage
        (print-file "example.txt")"#;
        let result = parse(input).unwrap();
        assert_eq!(result.len(), 2);
        // First expression is (define (print-file filename) ...)
        match &result[0] {
            SExpr::List(items) => {
                assert_eq!(items[0], SExpr::Atom("define".to_string()));
            }
            _ => panic!("Expected first expression to be a list"),
        }
        // Second expression is (print-file "example.txt")
        match &result[1] {
            SExpr::List(items) => {
                assert_eq!(items[0], SExpr::Atom("print-file".to_string()));
            }
            _ => panic!("Expected second expression to be a list"),
        }
    }
}
