//! Tokenizer for Scheme expressions
//! Based on TinyScheme tokenization logic

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    LParen,
    RParen,
    Dot,
    Atom,
    Quote,
    DQuote,
    BQuote,
    Comma,
    AtMark,
    Sharp,
    SharpConst,
    Vec,
    Eof,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::LParen => write!(f, "LParen"),
            TokenType::RParen => write!(f, "RParen"),
            TokenType::Dot => write!(f, "Dot"),
            TokenType::Atom => write!(f, "Atom"),
            TokenType::Quote => write!(f, "Quote"),
            TokenType::DQuote => write!(f, "DQuote"),
            TokenType::BQuote => write!(f, "BQuote"),
            TokenType::Comma => write!(f, "Comma"),
            TokenType::AtMark => write!(f, "AtMark"),
            TokenType::Sharp => write!(f, "Sharp"),
            TokenType::SharpConst => write!(f, "SharpConst"),
            TokenType::Vec => write!(f, "Vec"),
            TokenType::Eof => write!(f, "Eof"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub literal: String,
}

pub struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
    line: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Tokenizer {
            input,
            pos: 0,
            line: 1,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.pos).copied()
    }

    fn consume(&mut self) -> Option<u8> {
        let c = self.input.as_bytes().get(self.pos).copied();
        if let Some(ch) = c {
            if ch == b'\n' {
                self.line += 1;
            }
            self.pos += 1;
        }
        c
    }

    fn is_whitespace(c: u8) -> bool {
        matches!(c, b' ' | b'\t' | b'\n' | b'\r' | b'\x0b' | b'\x0c')
    }

    fn is_one_of(chars: &str, c: u8) -> bool {
        chars.as_bytes().contains(&c)
    }

    fn skip_whitespace(&mut self) -> bool {
        while let Some(c) = self.peek() {
            if Self::is_whitespace(c) {
                self.consume();
            } else {
                return true;
            }
        }
        false
    }

    fn skip_atom(&mut self) {
        while let Some(c) = self.peek() {
            if Self::is_whitespace(c) || Self::is_one_of("()\"';,#", c) {
                break;
            }
            self.consume();
        }
    }

    pub fn next_token(&mut self) -> Token {
        loop {
            // Skip leading whitespace
            if !self.skip_whitespace() {
                return Token {
                    token_type: TokenType::Eof,
                    start: self.pos,
                    end: self.pos,
                    line: self.line,
                    literal: String::new(),
                };
            }

            let start_pos = self.pos;
            let start_line = self.line;

            match self.peek() {
                Some(b'(') => {
                    self.consume();
                    return Token {
                        token_type: TokenType::LParen,
                        start: start_pos,
                        end: self.pos,
                        line: start_line,
                        literal: "(".to_string(),
                    };
                }
                Some(b')') => {
                    self.consume();
                    return Token {
                        token_type: TokenType::RParen,
                        start: start_pos,
                        end: self.pos,
                        line: start_line,
                        literal: ")".to_string(),
                    };
                }
                Some(b'\'') => {
                    self.consume();
                    return Token {
                        token_type: TokenType::Quote,
                        start: start_pos,
                        end: self.pos,
                        line: start_line,
                        literal: "'".to_string(),
                    };
                }
                Some(b'`') => {
                    self.consume();
                    return Token {
                        token_type: TokenType::BQuote,
                        start: start_pos,
                        end: self.pos,
                        line: start_line,
                        literal: "`".to_string(),
                    };
                }
                Some(b'"') => {
                    self.consume();
                    return Token {
                        token_type: TokenType::DQuote,
                        start: start_pos,
                        end: self.pos,
                        line: start_line,
                        literal: "\"".to_string(),
                    };
                }
                Some(b'.') => {
                    self.consume();
                    // Check if this is a dot token or start of an atom
                    match self.peek() {
                        Some(c) if Self::is_whitespace(c) || Self::is_one_of("()\"';,#", c) => {
                            return Token {
                                token_type: TokenType::Dot,
                                start: start_pos,
                                end: self.pos,
                                line: start_line,
                                literal: ".".to_string(),
                            };
                        }
                        Some(_) => {
                            // It's an atom starting with dot
                            self.skip_atom();
                            let literal = self.input[start_pos..self.pos].to_string();
                            return Token {
                                token_type: TokenType::Atom,
                                start: start_pos,
                                end: self.pos,
                                line: start_line,
                                literal,
                            };
                        }
                        None => {
                            return Token {
                                token_type: TokenType::Dot,
                                start: start_pos,
                                end: self.pos,
                                line: start_line,
                                literal: ".".to_string(),
                            };
                        }
                    }
                }
                Some(b';') => {
                    // Comment: skip until newline or EOF
                    self.consume(); // consume the semicolon
                    while let Some(c) = self.consume() {
                        if c == b'\n' {
                            break;
                        }
                    }
                    // Loop to get next token after comment
                    continue;
                }
                Some(b',') => {
                    self.consume();
                    // Check for ,@
                    if self.peek() == Some(b'@') {
                        self.consume();
                        return Token {
                            token_type: TokenType::AtMark,
                            start: start_pos,
                            end: self.pos,
                            line: start_line,
                            literal: ",@".to_string(),
                        };
                    } else {
                        return Token {
                            token_type: TokenType::Comma,
                            start: start_pos,
                            end: self.pos,
                            line: start_line,
                            literal: ",".to_string(),
                        };
                    }
                }
                Some(b'#') => {
                    self.consume();
                    match self.peek() {
                        Some(b'(') => {
                            self.consume();
                            return Token {
                                token_type: TokenType::Vec,
                                start: start_pos,
                                end: self.pos,
                                line: start_line,
                                literal: "#(".to_string(),
                            };
                        }
                        Some(b'!') => {
                            // Shebang comment: skip until newline or EOF
                            self.consume(); // consume the !
                            while let Some(c) = self.consume() {
                                if c == b'\n' {
                                    break;
                                }
                            }
                            // Loop to get next token after shebang
                            continue;
                        }
                        Some(c) if Self::is_one_of(" tfodxb\\", c) => {
                            // Sharp constant: consume the special char and any following atom chars
                            self.consume();
                            if c != b' ' {
                                self.skip_atom();
                            }
                            let literal = self.input[start_pos..self.pos].to_string();
                            return Token {
                                token_type: TokenType::SharpConst,
                                start: start_pos,
                                end: self.pos,
                                line: start_line,
                                literal,
                            };
                        }
                        _ => {
                            return Token {
                                token_type: TokenType::Sharp,
                                start: start_pos,
                                end: self.pos,
                                line: start_line,
                                literal: "#".to_string(),
                            };
                        }
                    }
                }
                Some(_) => {
                    // Any other character starts an atom
                    self.skip_atom();
                    let literal = self.input[start_pos..self.pos].to_string();
                    return Token {
                        token_type: TokenType::Atom,
                        start: start_pos,
                        end: self.pos,
                        line: start_line,
                        literal,
                    };
                }
                None => {
                    return Token {
                        token_type: TokenType::Eof,
                        start: self.pos,
                        end: self.pos,
                        line: self.line,
                        literal: String::new(),
                    };
                }
            }
        }
    }

    /// Get all tokens until EOF
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        loop {
            let token = self.next_token();
            if token.token_type == TokenType::Eof {
                break;
            }
            tokens.push(token);
        }
        tokens
    }
}

pub fn tokenize_string(input: &str) -> Vec<Token> {
    let mut tokenizer = Tokenizer::new(input);
    tokenizer.tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_list() {
        let tokens = tokenize_string("(+ 1 2)");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_type, TokenType::LParen);
        assert_eq!(tokens[1].token_type, TokenType::Atom);
        assert_eq!(tokens[1].literal, "+");
        assert_eq!(tokens[2].token_type, TokenType::Atom);
        assert_eq!(tokens[2].literal, "1");
        assert_eq!(tokens[3].token_type, TokenType::Atom);
        assert_eq!(tokens[3].literal, "2");
        assert_eq!(tokens[4].token_type, TokenType::RParen);
    }

    #[test]
    fn test_nested_lists() {
        let tokens =
            tokenize_string("(define (factorial n) (if (= n 0) 1 (* n (factorial (- n 1)))))");
        assert_eq!(tokens[0].token_type, TokenType::LParen);
        assert_eq!(tokens[1].token_type, TokenType::Atom); // define
        assert_eq!(tokens[1].literal, "define");
        assert_eq!(tokens[2].token_type, TokenType::LParen); // (factorial
        assert_eq!(tokens[3].token_type, TokenType::Atom); // factorial
        assert_eq!(tokens[3].literal, "factorial");
        assert_eq!(tokens[4].token_type, TokenType::Atom); // n
        assert_eq!(tokens[4].literal, "n");
        assert_eq!(tokens[5].token_type, TokenType::RParen); // )
        assert_eq!(tokens[tokens.len() - 1].token_type, TokenType::RParen); // final )
    }

    #[test]
    fn test_quote() {
        let tokens = tokenize_string("'(a b c)");
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].token_type, TokenType::Quote);
        assert_eq!(tokens[1].token_type, TokenType::LParen);
        assert_eq!(tokens[2].token_type, TokenType::Atom);
        assert_eq!(tokens[2].literal, "a");
        assert_eq!(tokens[3].token_type, TokenType::Atom);
        assert_eq!(tokens[3].literal, "b");
        assert_eq!(tokens[4].token_type, TokenType::Atom);
        assert_eq!(tokens[4].literal, "c");
        assert_eq!(tokens[5].token_type, TokenType::RParen);
    }

    #[test]
    fn test_backquote() {
        let tokens = tokenize_string("`(a ,b)");
        assert_eq!(tokens.len(), 6);
        assert_eq!(tokens[0].token_type, TokenType::BQuote);
        assert_eq!(tokens[1].token_type, TokenType::LParen);
        assert_eq!(tokens[2].token_type, TokenType::Atom);
        assert_eq!(tokens[2].literal, "a");
        assert_eq!(tokens[3].token_type, TokenType::Comma);
        assert_eq!(tokens[4].token_type, TokenType::Atom);
        assert_eq!(tokens[4].literal, "b");
        assert_eq!(tokens[5].token_type, TokenType::RParen);
    }

    #[test]
    fn test_unquote_splicing() {
        let tokens = tokenize_string("(list ,@items)");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_type, TokenType::LParen);
        assert_eq!(tokens[1].token_type, TokenType::Atom);
        assert_eq!(tokens[1].literal, "list");
        assert_eq!(tokens[2].token_type, TokenType::AtMark);
        assert_eq!(tokens[2].literal, ",@");
        assert_eq!(tokens[3].token_type, TokenType::Atom);
        assert_eq!(tokens[3].literal, "items");
        assert_eq!(tokens[4].token_type, TokenType::RParen);
    }

    #[test]
    fn test_dot_notation() {
        let tokens = tokenize_string("(a . b)");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_type, TokenType::LParen);
        assert_eq!(tokens[1].token_type, TokenType::Atom);
        assert_eq!(tokens[1].literal, "a");
        assert_eq!(tokens[2].token_type, TokenType::Dot);
        assert_eq!(tokens[2].literal, ".");
        assert_eq!(tokens[3].token_type, TokenType::Atom);
        assert_eq!(tokens[3].literal, "b");
        assert_eq!(tokens[4].token_type, TokenType::RParen);
    }

    #[test]
    fn test_vector() {
        let tokens = tokenize_string("#(1 2 3)");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_type, TokenType::Vec);
        assert_eq!(tokens[0].literal, "#(");
        assert_eq!(tokens[1].token_type, TokenType::Atom);
        assert_eq!(tokens[1].literal, "1");
        assert_eq!(tokens[2].token_type, TokenType::Atom);
        assert_eq!(tokens[2].literal, "2");
        assert_eq!(tokens[3].token_type, TokenType::Atom);
        assert_eq!(tokens[3].literal, "3");
        assert_eq!(tokens[4].token_type, TokenType::RParen);
    }

    #[test]
    fn test_sharp_const() {
        let tokens = tokenize_string("#t #f #\\n #x1F");
        assert_eq!(tokens.len(), 4);
        for i in 0..4 {
            assert_eq!(tokens[i].token_type, TokenType::SharpConst);
        }
    }

    #[test]
    fn test_string_literal() {
        let tokens = tokenize_string("\"hello world\"");
        // Tokenizer returns DQuote, then atoms for the content, then another DQuote
        // String parsing is left to the parser
        assert_eq!(tokens[0].token_type, TokenType::DQuote);
        assert_eq!(tokens[0].literal, "\"");
        assert!(tokens.len() >= 3); // at least opening quote, content, closing quote
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize_string("(+ 1 2) ; this is a comment\n(* 3 4)");
        assert_eq!(tokens.len(), 10);
        assert_eq!(tokens[0].token_type, TokenType::LParen);
        assert_eq!(tokens[1].token_type, TokenType::Atom);
        assert_eq!(tokens[1].literal, "+");
        assert_eq!(tokens[2].token_type, TokenType::Atom);
        assert_eq!(tokens[2].literal, "1");
        assert_eq!(tokens[3].token_type, TokenType::Atom);
        assert_eq!(tokens[3].literal, "2");
        assert_eq!(tokens[4].token_type, TokenType::RParen);
        assert_eq!(tokens[5].token_type, TokenType::LParen);
        assert_eq!(tokens[6].token_type, TokenType::Atom);
        assert_eq!(tokens[6].literal, "*");
        assert_eq!(tokens[7].token_type, TokenType::Atom);
        assert_eq!(tokens[7].literal, "3");
        assert_eq!(tokens[8].token_type, TokenType::Atom);
        assert_eq!(tokens[8].literal, "4");
        assert_eq!(tokens[9].token_type, TokenType::RParen);
    }

    #[test]
    fn test_lambda() {
        let tokens = tokenize_string("(lambda (x) (* x x))");
        assert_eq!(tokens[0].token_type, TokenType::LParen);
        assert_eq!(tokens[1].token_type, TokenType::Atom); // lambda
        assert_eq!(tokens[1].literal, "lambda");
        assert_eq!(tokens[2].token_type, TokenType::LParen); // (x)
        assert_eq!(tokens[3].token_type, TokenType::Atom); // x
        assert_eq!(tokens[3].literal, "x");
        assert_eq!(tokens[4].token_type, TokenType::RParen);
        assert_eq!(tokens[tokens.len() - 1].token_type, TokenType::RParen);
    }

    #[test]
    fn test_whitespace_handling() {
        let tokens = tokenize_string("  (  +   1   2  )  ");
        assert_eq!(tokens.len(), 5);
        assert_eq!(tokens[0].token_type, TokenType::LParen);
        assert_eq!(tokens[1].token_type, TokenType::Atom);
        assert_eq!(tokens[1].literal, "+");
        assert_eq!(tokens[2].token_type, TokenType::Atom);
        assert_eq!(tokens[2].literal, "1");
        assert_eq!(tokens[3].token_type, TokenType::Atom);
        assert_eq!(tokens[3].literal, "2");
        assert_eq!(tokens[4].token_type, TokenType::RParen);
    }

    #[test]
    fn test_atom_with_numbers() {
        let tokens = tokenize_string("(list 42 3.14 -17)");
        let atom_count = tokens
            .iter()
            .filter(|t| t.token_type == TokenType::Atom)
            .count();
        assert_eq!(atom_count, 4); // list, 42, 3.14, -17
    }

    #[test]
    fn test_let_form() {
        let tokens = tokenize_string("(let ((x 10) (y 20)) (+ x y))");
        assert_eq!(tokens[0].token_type, TokenType::LParen);
        assert_eq!(tokens[1].token_type, TokenType::Atom); // let
        assert_eq!(tokens[1].literal, "let");
        assert_eq!(tokens[2].token_type, TokenType::LParen); // ((x 10)
        assert_eq!(tokens[3].token_type, TokenType::LParen); // (x 10)
        assert_eq!(tokens[tokens.len() - 1].token_type, TokenType::RParen);
    }

    #[test]
    fn test_cond_form() {
        let tokens = tokenize_string("(cond ((> x 0) 1) ((< x 0) -1) (else 0))");
        assert_eq!(tokens[0].token_type, TokenType::LParen);
        assert_eq!(tokens[1].token_type, TokenType::Atom); // cond
        assert_eq!(tokens[1].literal, "cond");
        assert_eq!(tokens[2].token_type, TokenType::LParen); // ((> x 0)
    }

    #[test]
    fn test_empty_list() {
        let tokens = tokenize_string("()");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::LParen);
        assert_eq!(tokens[1].token_type, TokenType::RParen);
    }

    #[test]
    fn test_multiple_expressions() {
        let tokens = tokenize_string("42 hello (+ 1 2)");
        assert_eq!(tokens[0].token_type, TokenType::Atom); // 42
        assert_eq!(tokens[0].literal, "42");
        assert_eq!(tokens[1].token_type, TokenType::Atom); // hello
        assert_eq!(tokens[1].literal, "hello");
        assert_eq!(tokens[2].token_type, TokenType::LParen);
        assert_eq!(tokens[3].token_type, TokenType::Atom); // +
        assert_eq!(tokens[3].literal, "+");
        assert_eq!(tokens[4].token_type, TokenType::Atom); // 1
        assert_eq!(tokens[4].literal, "1");
        assert_eq!(tokens[5].token_type, TokenType::Atom); // 2
        assert_eq!(tokens[5].literal, "2");
        assert_eq!(tokens[6].token_type, TokenType::RParen);
    }

    #[test]
    fn test_boolean_constants() {
        let tokens = tokenize_string("#t #f");
        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].token_type, TokenType::SharpConst);
        assert_eq!(tokens[1].token_type, TokenType::SharpConst);
    }

    #[test]
    fn test_character_literals() {
        let tokens = tokenize_string("#\\a #\\space #\\newline");
        assert_eq!(tokens[0].token_type, TokenType::SharpConst);
        assert_eq!(tokens[1].token_type, TokenType::SharpConst);
        assert_eq!(tokens[2].token_type, TokenType::SharpConst);
    }
}
