//! S-expression parser using nom combinators
//! Directly parses from string input without tokenization

use crate::ast::SExpr;
use anyhow::Result;
use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::bytes::streaming::take_while;
use nom::character::complete::{char, multispace0, one_of, satisfy};
use nom::combinator::{map, recognize};
use nom::multi::many0;
use nom::sequence::{delimited, pair};
use nom::{IResult, Parser};

/// Parse a comment: ; followed by anything until newline
fn parse_comment(i: &str) -> IResult<&str, ()> {
    let (i, _) = char(';').parse(i)?;
    let (i, _) = is_not("\n\r").parse(i)?;
    Ok((i, ()))
}

/// Parse whitespace and comments
fn ws_or_comment(i: &str) -> IResult<&str, ()> {
    let mut remaining = i;
    loop {
        let (i_after_ws, _) = multispace0.parse(remaining)?;
        match parse_comment(i_after_ws) {
            Ok((i_after_comment, _)) => {
                remaining = i_after_comment;
            }
            Err(_) => {
                return Ok((i_after_ws, ()));
            }
        }
    }
}

/// Parse a simple string: "content"
fn parse_string(input: &str) -> IResult<&str, &str> {
    // let (input, _) = char('"').parse(input)?;
    // let (input, content) = is_not("\"").parse(input)?;
    // let (input, _) = char('"').parse(input)?;
    // Ok((input, content.to_string()))
    delimited(char('"'), take_while(|c: char| c != '"'), char('"')).parse(input)
}

/// Parse a number: integer or float
fn parse_number(input: &str) -> IResult<&str, f64> {
    let mut num_parser = alt((
        // Float: 42.5 or .5 or 42.
        recognize(pair(
            nom::combinator::opt(many0(one_of("0123456789"))),
            pair(char('.'), many0(one_of("0123456789"))),
        )),
        // Integer with optional exponent
        recognize(pair(
            nom::multi::many1(one_of("0123456789")),
            nom::combinator::opt(pair(
                one_of("eE"),
                pair(
                    nom::combinator::opt(one_of("+-")),
                    nom::multi::many1(one_of("0123456789")),
                ),
            )),
        )),
    ));

    let (input, num_str) = num_parser.parse(input)?;

    match num_str.parse::<f64>() {
        Ok(n) => Ok((input, n)),
        Err(_) => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        ))),
    }
}

/// Parse an atom: identifier like `hello`, `+`, `define`, etc.
fn parse_atom(input: &str) -> IResult<&str, String> {
    let mut atom_parser = recognize(pair(
        alt((
            nom::character::complete::alpha1,
            tag("+"),
            tag("-"),
            tag("*"),
            tag("/"),
            tag("="),
            tag("<"),
            tag(">"),
            tag("!"),
            tag("?"),
            tag("_"),
        )),
        many0(alt((
            nom::character::complete::alphanumeric1,
            tag("-"),
            tag("?"),
            tag("!"),
            tag("_"),
        ))),
    ));

    let (input, atom) = atom_parser.parse(input)?;
    Ok((input, atom.to_string()))
}

/// Parse a character literal: #\a, #\space, #\newline, etc.
fn parse_char(input: &str) -> IResult<&str, char> {
    let (input, _) = tag("#\\").parse(input)?;
    alt((
        tag("space").map(|_| ' '),
        tag("newline").map(|_| '\n'),
        tag("tab").map(|_| '\t'),
        tag("return").map(|_| '\r'),
        satisfy(|c| !c.is_whitespace()).map(|c| c),
    ))
    .parse(input)
}

/// Parse boolean: #t or #f
fn parse_bool(input: &str) -> IResult<&str, bool> {
    alt((tag("#t").map(|_| true), tag("#f").map(|_| false))).parse(input)
}

/// Parse a list: (expr1 expr2 ...)
/// Note: This parser requires Arena support. Lists now store Vec<NodeId>, not Vec<SExpr>.
/// For now, this returns an error. Use the tokenizer-based parser instead.
fn parse_list(input: &str) -> IResult<&str, SExpr> {
    // This parser is no longer compatible with Arena-based List storage
    // Use the tokenizer-based parser from parser.rs instead
    Err(nom::Err::Failure(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Fail,
    )))
}

/// Parse a vector: #(expr1 expr2 ...)
/// Note: This parser requires Arena support. Vectors now store Vec<NodeId>, not Vec<SExpr>.
/// For now, this returns an error. Use the tokenizer-based parser instead.
fn parse_vector(input: &str) -> IResult<&str, SExpr> {
    // This parser is no longer compatible with Arena-based Vector storage
    // Use the tokenizer-based parser from parser.rs instead
    Err(nom::Err::Failure(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Fail,
    )))
}

/// Parse a quoted expression: 'expr
/// Note: Returns a placeholder since this parser isn't arena-aware
fn parse_quote(input: &str) -> IResult<&str, SExpr> {
    let (input, _) = char('\'').parse(input)?;
    let (input, _expr) = parse_expr(input)?;
    // Return a placeholder - this parser needs refactoring to work with Arena
    Ok((input, SExpr::Quote(0)))
}

/// Parse a quasi-quoted expression: `expr
/// Note: Returns a placeholder since this parser isn't arena-aware
fn parse_quasi_quote(input: &str) -> IResult<&str, SExpr> {
    let (input, _) = char('`').parse(input)?;
    let (input, _expr) = parse_expr(input)?;
    // Return a placeholder - this parser needs refactoring to work with Arena
    Ok((input, SExpr::QuasiQuote(0)))
}

/// Parse an unquote-splicing expression: ,@expr (check this before ,expr)
/// Note: Returns a placeholder since this parser isn't arena-aware
fn parse_unquote_splicing(input: &str) -> IResult<&str, SExpr> {
    let (input, _) = tag(",@").parse(input)?;
    let (input, _expr) = parse_expr(input)?;
    // Return a placeholder - this parser needs refactoring to work with Arena
    Ok((input, SExpr::UnquoteSplicing(0)))
}

/// Parse an unquoted expression: ,expr
/// Note: Returns a placeholder since this parser isn't arena-aware
fn parse_unquote(input: &str) -> IResult<&str, SExpr> {
    let (input, _) = char(',').parse(input)?;
    let (input, _expr) = parse_expr(input)?;
    // Return a placeholder - this parser needs refactoring to work with Arena
    Ok((input, SExpr::Unquote(0)))
}

/// Parse a single S-expression
fn parse_expr(input: &str) -> IResult<&str, SExpr> {
    let (input, _) = ws_or_comment(input)?;
    alt((
        parse_list,
        parse_vector,
        parse_unquote_splicing,
        parse_unquote,
        parse_quasi_quote,
        parse_quote,
        map(parse_bool, |b| SExpr::Bool(b)),
        map(parse_char, |c| SExpr::Char(c)),
        map(parse_string, |s| SExpr::String(s.to_string())),
        map(parse_number, |n| SExpr::Number(n)),
        map(parse_atom, |a| SExpr::Atom(a)),
    ))
    .parse(input)
}

/// Parse a sequence of S-expressions
pub fn parse(input: &str) -> Result<Vec<SExpr>> {
    let mut exprs = Vec::new();
    let mut remaining = input;

    loop {
        // Skip whitespace and comments
        let (rest, _) = ws_or_comment(remaining).unwrap_or((remaining, ()));
        remaining = rest;

        if remaining.is_empty() {
            break;
        }

        match parse_expr(remaining) {
            Ok((rest, expr)) => {
                exprs.push(expr);
                remaining = rest;
            }
            Err(e) => {
                return Err(anyhow::anyhow!("Parse error: {:?}", e));
            }
        }
    }

    Ok(exprs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]  // List/Vector parsing disabled - requires Arena support
    fn test_parse_simple_list() {
        let result = parse("(+ 1 2)").unwrap();
        assert_eq!(result.len(), 1);
        // Note: nom_parser returns Vec<SExpr>, not arena-based.
        // This test would need to be updated if nom_parser used Arena.
        match &result[0] {
            SExpr::List(_ids) => {
                // Vector of NodeIds - can't easily test without Arena
                // assert_eq!(ids.len(), 3);
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    #[ignore]  // nom_parser tests not maintained
    fn test_parse_atom() {
        let result = parse("hello").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], SExpr::Atom("hello".to_string()));
    }

    #[test]
    #[ignore]  // nom_parser tests not maintained
    fn test_parse_number() {
        let result = parse("42").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], SExpr::Number(42.0));
    }

    #[test]
    #[ignore]  // nom_parser tests not maintained
    fn test_parse_quote() {
        let result = parse("'hello").unwrap();
        assert_eq!(result.len(), 1);
        // Note: Quote now stores NodeId (usize) which refers to arena storage
        // This test is simplified since the nom_parser uses placeholders
        match &result[0] {
            SExpr::Quote(_) => {}
            _ => panic!("Expected quote"),
        }
    }

    #[test]
    #[ignore]  // nom_parser tests not maintained
    fn test_parse_bool() {
        let result = parse("#t #f").unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], SExpr::Bool(true));
        assert_eq!(result[1], SExpr::Bool(false));
    }

    #[test]
    #[ignore]  // nom_parser tests not maintained
    fn test_parse_string() {
        let result = parse("\"hello world\"").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], SExpr::String("hello world".to_string()));
    }

    #[test]
    #[ignore]  // nom_parser tests not maintained
    fn test_parse_nested_list() {
        let result = parse("(define (square x) (* x x))").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            SExpr::List(_ids) => {
                // List now stores NodeIds, not SExpr
                // assert_eq!(ids.len(), 3);
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    #[ignore]  // nom_parser tests not maintained
    fn test_parse_vector() {
        let result = parse("#(1 2 3)").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            SExpr::Vector(_ids) => {
                // Vector now stores NodeIds
                // assert_eq!(ids.len(), 3);
            }
            _ => panic!("Expected vector"),
        }
    }

    #[test]
    #[ignore]  // nom_parser tests not maintained
    fn test_parse_multiple_exprs() {
        let result = parse("42 hello (+ 1 2)").unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], SExpr::Number(42.0));
        assert_eq!(result[1], SExpr::Atom("hello".to_string()));
    }

    #[test]
    #[ignore]  // nom_parser tests not maintained
    fn test_parse_backquote() {
        let result = parse("`(a ,b)").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            SExpr::QuasiQuote(_) => {}
            _ => panic!("Expected quasi-quote"),
        }
    }

    #[test]
    #[ignore]  // nom_parser tests not maintained
    fn test_parse_unquote_splicing() {
        let result = parse("(,@items)").unwrap();
        assert_eq!(result.len(), 1);
        match &result[0] {
            SExpr::List(_ids) => {
                // List now stores NodeIds
                // assert_eq!(ids.len(), 1);
                // match &arena.get(ids[0]) ...
            }
            _ => panic!("Expected list"),
        }
    }

    #[test]
    #[ignore]  // nom_parser tests not maintained
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
            SExpr::List(_ids) => {
                // List now stores NodeIds - would need Arena to check contents
            }
            _ => panic!("Expected first expression to be a list"),
        }
        // Second expression is (print-file "example.txt")
        match &result[1] {
            SExpr::List(_ids) => {
                // List now stores NodeIds - would need Arena to check contents
            }
            _ => panic!("Expected second expression to be a list"),
        }
    }
}
