//! Lua parser with nom
//!
//! chunk ::= block
//! block ::= {stat} [retstat]
//!
//! stat ::=  ';' |
//! varlist '=' explist |
//! functioncall |
//! label |
//! break |
//! goto Name |
//! do block end |
//! while exp do block end |
//! repeat block until exp |
//! if exp then block {elseif exp then block} [else block] end |
//! for Name '=' exp ',' exp [',' exp] do block end |
//! for namelist in explist do block end |
//! function funcname funcbody |
//! local function Name funcbody |
//! local namelist ['=' explist]
//!
//! retstat ::= return [explist] [';']

mod helpers;
mod expression;
mod statement;

pub use helpers::{tokenize_single, KEYWORDS, SYMBOLS};
pub use expression::{parse_expression, parse_expression_list, parse_prefix_exp};
pub use statement::parse_block;

use nom::{IResult, Input, Needed};

use crate::lua_parser_types as types;

// Re-export main AST types
pub use types::{
    Block, Expression, Statement, Token, Token::*, ReturnStatement,
    BinaryOp, UnaryOp, Field, FieldKey, FunctionBody,
};

#[derive(Debug, Clone, Copy)]
pub struct TokenSlice<'a>(&'a [Token]);

impl<'a> From<&'a [Token]> for TokenSlice<'a> {
    fn from(slice: &'a [Token]) -> Self {
        TokenSlice(slice)
    }
}

impl<'a> Input for TokenSlice<'a> {
    type Item = &'a Token;
    type Iter = std::slice::Iter<'a, Token>;
    type IterIndices = std::iter::Enumerate<std::slice::Iter<'a, Token>>;

    fn input_len(&self) -> usize {
        self.0.len()
    }

    fn take(&self, index: usize) -> Self {
        TokenSlice(&self.0[..index.min(self.0.len())])
    }

    fn take_from(&self, index: usize) -> Self {
        TokenSlice(&self.0[index.min(self.0.len())..])
    }

    fn take_split(&self, index: usize) -> (Self, Self) {
        let index = index.min(self.0.len());
        (TokenSlice(&self.0[index..]), TokenSlice(&self.0[..index]))
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.0.iter().position(predicate)
    }

    fn iter_elements(&self) -> Self::Iter {
        self.0.iter()
    }

    fn iter_indices(&self) -> Self::IterIndices {
        self.0.iter().enumerate()
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if count > self.0.len() {
            Err(Needed::Size(
                std::num::NonZeroUsize::new(count - self.0.len()).unwrap(),
            ))
        } else {
            Ok(count)
        }
    }
}

/// Helper to match a specific token
pub fn token_tag(expected: &Token) -> impl Fn(TokenSlice) -> IResult<TokenSlice, &Token> {
    let expected = expected.clone();
    move |input: TokenSlice| {
        if let Some(tok) = input.0.first() {
            if tok == &expected {
                Ok((TokenSlice(&input.0[1..]), tok))
            } else {
                Err(nom::Err::Error(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Tag,
                )))
            }
        } else {
            Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Eof,
            )))
        }
    }
}

/// Tokenize Lua source code into a vector of tokens
pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut remaining = input;

    loop {
        // Skip whitespace and comments
        while !remaining.is_empty() {
            if remaining.starts_with("--") {
                if let Some(newline) = remaining.find('\n') {
                    remaining = &remaining[newline + 1..];
                } else {
                    remaining = "";
                }
            } else if remaining.chars().next().is_some_and(char::is_whitespace) {
                remaining = &remaining[1..];
            } else {
                break;
            }
        }

        if remaining.is_empty() {
            break;
        }

        let (rest, tok) = tokenize_single(remaining)
            .map_err(|e| format!("Tokenization error: {:?}", e))?;

        tokens.push(tok);
        remaining = rest;
    }

    Ok(tokens)
}

/// Parse tokenized Lua code into an AST
pub fn parse(t: TokenSlice) -> IResult<TokenSlice, Block> {
    parse_block(t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_assignment() {
        let code = "x = 5";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_function_definition() {
        let code = "function foo(a, b) return a + b end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_if_statement() {
        let code = "if x > 0 then print('positive') elseif x < 0 then print('negative') else print('zero') end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_table_construction() {
        let code = "t = {1, 2, 3, x = 10, y = 20}";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_while_loop() {
        let code = "while x < 10 do x = x + 1 end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_multiple_statements() {
        let code = "x = 1; y = 2; z = x + y";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 5); // 3 assignments + 2 semicolons as empty statements
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_nested_function_calls() {
        let code = "print(foo(bar(1, 2), 3))";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_binary_operations() {
        let code = "z = a + b * c - d / e ^ f % g";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_for_loop_numeric() {
        let code = "for i = 1, 10, 2 do print(i) end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_for_loop_generic() {
        let code = "for k, v in pairs(t) do print(k, v) end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_local_variables() {
        let code = "local x, y, z = 1, 2, 3";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_return_statement() {
        let code = "function test() return 42 end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_method_call() {
        let code = "obj:method(arg1, arg2)";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_anonymous_function() {
        let code = "f = function(x) return x * 2 end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_string_concatenation() {
        let code = "s = 'hello' .. ' ' .. 'world'";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_comments() {
        let code = "-- This is a comment
        x = 5 -- inline comment
        -- Another comment
        y = 10";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 2);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_repeat_until_loop() {
        let code = "repeat x = x + 1 until x > 10";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_do_block() {
        let code = "do local x = 5 print(x) end";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_label_and_goto() {
        let code = "
        ::start::
        print(1)
        goto finish
        print(2)
        ::finish::
        print(3)
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 6); // label, print, goto, print, label, print
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_table_indexing_chains() {
        let code = "x = t[1][2][3]; y = t.a.b.c; z = t[k].field[m]";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        // x = t[1][2][3] (1) + ; (1) + y = t.a.b.c (1) + ; (1) + z = t[k].field[m] (1) = 5
        assert_eq!(block.statements.len(), 5);
        assert!(rest.0.is_empty());
    }

    #[test]
    fn test_varargs_in_function() {
        let code = "
        function print_all(...)
            return ...
        end
        ";
        let tokens = tokenize(code).unwrap();
        let ts = TokenSlice::from(tokens.as_slice());
        let (rest, block) = parse(ts).unwrap();

        assert_eq!(block.statements.len(), 1);
        assert!(rest.0.is_empty());
    }
}
