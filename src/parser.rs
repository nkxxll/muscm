//! Simple S-expression parser for Scheme
//! Converts tokens into an AST of nested S-expressions

use crate::ast::{Arena, NodeId, SExpr};
use crate::tokenizer::{tokenize_string, Token, TokenType};
use std::fmt;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    arena: Arena,
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
        Parser {
            tokens,
            pos: 0,
            arena: Arena::new(),
        }
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

    fn parse_string(&mut self) -> Result<NodeId, ParseError> {
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
                    let expr = SExpr::String(content);
                    return Ok(self.arena.alloc(expr));
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

    fn parse_list(&mut self) -> Result<NodeId, ParseError> {
        // Opening paren already consumed
        let mut items = Vec::new();

        loop {
            match self.peek() {
                Some(Token {
                    token_type: TokenType::RParen,
                    ..
                }) => {
                    self.consume();
                    let expr = SExpr::List(items);
                    return Ok(self.arena.alloc(expr));
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
                    let cdr_id = self.parse_expr()?;
                    items.push(cdr_id);

                    // Convert (a . b) to (a . b) by creating improper list
                    match self.peek() {
                        Some(Token {
                            token_type: TokenType::RParen,
                            ..
                        }) => {
                            self.consume();
                            // Build improper list
                            let expr = SExpr::List(items);
                            return Ok(self.arena.alloc(expr));
                        }
                        _ => return Err(self.error("Expected ) after dot notation")),
                    }
                }
                _ => {
                    let node_id = self.parse_expr()?;
                    items.push(node_id);
                }
            }
        }
    }

    fn parse_vector(&mut self) -> Result<NodeId, ParseError> {
        // #( already consumed
        let mut items = Vec::new();

        loop {
            match self.peek() {
                Some(Token {
                    token_type: TokenType::RParen,
                    ..
                }) => {
                    self.consume();
                    let expr = SExpr::Vector(items);
                    return Ok(self.arena.alloc(expr));
                }
                _ => {
                    let node_id = self.parse_expr()?;
                    items.push(node_id);
                }
            }
        }
    }

    fn parse_sharp_const(&mut self, literal: &str) -> Result<NodeId, ParseError> {
        let expr = match literal {
            "#t" => SExpr::Bool(true),
            "#f" => SExpr::Bool(false),
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
                SExpr::Char(c)
            }
            _ => return Err(self.error(&format!("Unknown sharp constant: {}", literal))),
        };
        Ok(self.arena.alloc(expr))
    }

    fn parse_atom(&mut self, literal: &str) -> Result<NodeId, ParseError> {
        // Try to parse as number
        let expr = if let Ok(n) = literal.parse::<f64>() {
            SExpr::Number(n)
        } else {
            // Otherwise it's an atom
            SExpr::Atom(literal.to_string())
        };
        Ok(self.arena.alloc(expr))
    }

    fn parse_expr(&mut self) -> Result<NodeId, ParseError> {
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
                let node_id = self.parse_expr()?;
                let expr = SExpr::Quote(node_id);
                Ok(self.arena.alloc(expr))
            }

            Some(Token {
                token_type: TokenType::BQuote,
                ..
            }) => {
                let node_id = self.parse_expr()?;
                let expr = SExpr::QuasiQuote(node_id);
                Ok(self.arena.alloc(expr))
            }

            Some(Token {
                token_type: TokenType::Comma,
                ..
            }) => {
                let node_id = self.parse_expr()?;
                let expr = SExpr::Unquote(node_id);
                Ok(self.arena.alloc(expr))
            }

            Some(Token {
                token_type: TokenType::AtMark,
                ..
            }) => {
                let node_id = self.parse_expr()?;
                let expr = SExpr::UnquoteSplicing(node_id);
                Ok(self.arena.alloc(expr))
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

    pub fn parse(mut self) -> Result<(Arena, Vec<NodeId>), ParseError> {
        let mut node_ids = Vec::new();

        loop {
            match self.peek() {
                Some(Token {
                    token_type: TokenType::Eof,
                    ..
                }) => break,
                None => break,
                _ => {
                    node_ids.push(self.parse_expr()?);
                }
            }
        }

        Ok((self.arena, node_ids))
    }
}

pub fn parse(input: &str) -> Result<(Arena, Vec<NodeId>), ParseError> {
    let tokens = tokenize_string(input);
    let parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_list() {
        let (arena, node_ids) = parse("(+ 1 2)").unwrap();
        assert_eq!(node_ids.len(), 1);
        if let Some(SExpr::List(ids)) = arena.get(node_ids[0]) {
            assert_eq!(ids.len(), 3);
            assert_eq!(arena.get(ids[1]), Some(&SExpr::Number(1.0)));
            assert_eq!(arena.get(ids[2]), Some(&SExpr::Number(2.0)));
        } else {
            panic!("Expected list");
        }
    }

    #[test]
    fn test_parse_atom() {
        let (_arena, node_ids) = parse("hello").unwrap();
        assert_eq!(node_ids.len(), 1);
    }

    #[test]
    fn test_parse_number() {
        let (_arena, node_ids) = parse("42").unwrap();
        assert_eq!(node_ids.len(), 1);
    }

    #[test]
    fn test_parse_quote() {
        let (_arena, node_ids) = parse("'hello").unwrap();
        assert_eq!(node_ids.len(), 1);
    }

    #[test]
    fn test_parse_bool() {
        let (_arena, node_ids) = parse("#t #f").unwrap();
        assert_eq!(node_ids.len(), 2);
    }

    #[test]
    fn test_parse_string() {
        let (_arena, node_ids) = parse("\"hello world\"").unwrap();
        assert_eq!(node_ids.len(), 1);
    }

    #[test]
    fn test_parse_nested_list() {
        let (_arena, node_ids) = parse("(define (square x) (* x x))").unwrap();
        assert_eq!(node_ids.len(), 1);
    }

    #[test]
    fn test_parse_vector() {
        let (_arena, node_ids) = parse("#(1 2 3)").unwrap();
        assert_eq!(node_ids.len(), 1);
    }

    #[test]
    fn test_parse_multiple_exprs() {
        let (_arena, node_ids) = parse("42 hello (+ 1 2)").unwrap();
        assert_eq!(node_ids.len(), 3);
    }

    #[test]
    fn test_parse_backquote() {
        let (_arena, node_ids) = parse("`(a ,b)").unwrap();
        assert_eq!(node_ids.len(), 1);
    }

    #[test]
    fn test_parse_unquote_splicing() {
        let (_arena, node_ids) = parse("(,@items)").unwrap();
        assert_eq!(node_ids.len(), 1);
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
        let (_arena, node_ids) = parse(input).unwrap();
        assert_eq!(node_ids.len(), 2);
    }
}
