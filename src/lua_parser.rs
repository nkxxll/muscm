//! lua parser with nom
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
//!
//! label ::= '::' Name '::'
//!
//! funcname ::= Name {'.' Name} [':' Name]
//!
//! varlist ::= var {',' var}
//!
//! var ::=  Name | prefixexp '[' exp ']' | prefixexp '.' Name
//!
//! namelist ::= Name {',' Name}
//!
//! explist ::= exp {',' exp}
//!
//! exp ::=  nil | false | true | Numeral | LiteralString | '...' | functiondef |
//! prefixexp | tableconstructor | exp binop exp | unop exp
//!
//! prefixexp ::= var | functioncall | '(' exp ')'
//!
//! functioncall ::=  prefixexp args | prefixexp ':' Name args
//!
//! args ::=  '(' [explist] ')' | tableconstructor | LiteralString
//!
//! functiondef ::= function funcbody
//!
//! funcbody ::= '(' [parlist] ')' block end
//!
//! parlist ::= namelist [',' '...'] | '...'
//!
//! tableconstructor ::= '{' [fieldlist] '}'
//!
//! fieldlist ::= field {fieldsep field} [fieldsep]
//!
//! field ::= '[' exp ']' '=' exp | Name '=' exp | exp
//!
//! fieldsep ::= ',' | ';'
//!
//! binop ::=  '+' | '-' | '*' | '/' | '//' | '^' | '%' |
//! '&' | '~' | '|' | '>>' | '<<' | '..' |
//! '<' | '<=' | '>' | '>=' | '==' | '~=' |
//! and | or
//!
//! unop ::= '-' | not | '#' | '~'
//!
//!
use phf::phf_map;

use nom::{
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, satisfy},
    combinator::{opt, recognize},
    multi::many0,
    sequence::{pair, preceded},
    IResult, Input, Needed, Parser,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    And,
    Break,
    Do,
    Else,
    Elseif,
    End,
    False,
    For,
    Function,
    Goto,
    If,
    In,
    Local,
    Nil,
    Not,
    Or,
    Repeat,
    Return,
    Then,
    True,
    Until,
    While,
    // Symbols
    Semicolon,
    Equals,
    Comma,
    Dot,
    Colon,
    DoubleColon,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Plus,
    Minus,
    Star,
    Slash,
    DoubleSlash,
    Caret,
    Percent,
    Ampersand,
    Tilde,
    Pipe,
    RShift,
    LShift,
    Concat,
    Lt,
    Lte,
    Gt,
    Gte,
    Eq,
    Neq,
    Hash,
    Varargs,
    // Values
    Identifier(::std::string::String),
    Number(::std::string::String),
    String(::std::string::String),
}
use Token::*;

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
        self.0.iter().position(|item| predicate(item))
    }

    fn iter_elements(&self) -> Self::Iter {
        self.0.iter()
    }

    fn iter_indices(&self) -> Self::IterIndices {
        self.0.iter().enumerate()
    }

    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if count > self.0.len() {
            Err(Needed::Size(std::num::NonZeroUsize::new(count - self.0.len()).unwrap()))
        } else {
            Ok(count)
        }
    }
}

const KEYWORDS: phf::Map<&str, Token> = phf_map! {
    "and" => And,
    "break" => Break,
    "do" => Do,
    "else" => Else,
    "elseif" => Elseif,
    "end" => End,
    "false" => False,
    "for" => For,
    "function" => Function,
    "goto" => Goto,
    "if" => If,
    "in" => In,
    "local" => Local,
    "nil" => Nil,
    "not" => Not,
    "or" => Or,
    "repeat" => Repeat,
    "return" => Return,
    "then" => Then,
    "true" => True,
    "until" => Until,
    "while" => While,
};

const SYMBOLS: phf::Map<&str, Token> = phf_map! {
    ";" => Semicolon,
    "=" => Equals,
    "," => Comma,
    "." => Dot,
    ":" => Colon,
    "::" => DoubleColon,
    "(" => LParen,
    ")" => RParen,
    "[" => LBracket,
    "]" => RBracket,
    "{" => LBrace,
    "}" => RBrace,
    "+" => Plus,
    "-" => Minus,
    "*" => Star,
    "/" => Slash,
    "//" => DoubleSlash,
    "^" => Caret,
    "%" => Percent,
    "&" => Ampersand,
    "~" => Tilde,
    "|" => Pipe,
    ">>" => RShift,
    "<<" => LShift,
    ".." => Concat,
    "<" => Lt,
    "<=" => Lte,
    ">" => Gt,
    ">=" => Gte,
    "==" => Eq,
    "~=" => Neq,
    "#" => Hash,
    "..." => Varargs,
};

// Helper parsers
fn identifier(input: &str) -> IResult<&str, &str> {
    recognize(pair(
        satisfy(|c| c.is_alphabetic() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
    ))
    .parse(input)
}

fn number(input: &str) -> IResult<&str, &str> {
    recognize(pair(digit1, opt(preceded(char('.'), digit1)))).parse(input)
}

fn string_literal(input: &str) -> IResult<&str, ::std::string::String> {
    let (input, _) = char('"').parse(input)?;
    let (input, content) = take_while1(|c: char| c != '"').parse(input)?;
    let (input, _) = char('"').parse(input)?;
    Ok((input, content.to_string()))
}

// Tokenizer
fn symbol(input: &str) -> IResult<&str, Token> {
    let symbols = vec![
        "...", "::", "//", ">>", "<<", "..", "<=", ">=", "==", "~=", ":", ".", "=", ",", ";", "(",
        ")", "[", "]", "{", "}", "+", "-", "*", "/", "^", "%", "&", "~", "|", "<", ">", "#",
    ];

    for sym in symbols {
        if let Ok((rest, _)) = tag::<_, _, nom::error::Error<_>>(sym)(input) {
            let token = SYMBOLS.get(sym).cloned().ok_or_else(|| {
                nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Tag))
            })?;
            return Ok((rest, token));
        }
    }

    Err(nom::Err::Error(nom::error::Error::new(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

fn token(input: &str) -> IResult<&str, Token> {
    if let Ok((rest, token)) = symbol(input) {
        return Ok((rest, token));
    }
    if let Ok((rest, content)) = string_literal(input) {
        return Ok((rest, Token::String(content)));
    }
    if let Ok((rest, num)) = number(input) {
        return Ok((rest, Token::Number(num.to_string())));
    }

    let (rest, ident) = identifier(input)?;
    let token = KEYWORDS
        .get(ident)
        .cloned()
        .unwrap_or_else(|| Token::Identifier(ident.to_string()));
    Ok((rest, token))
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, ::std::string::String> {
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

        let (rest, tok) = token(remaining).map_err(|e| format!("Tokenization error: {:?}", e))?;

        tokens.push(tok);
        remaining = rest;
    }

    Ok(tokens)
}

struct Block {
    statements: Vec<Statement>,
    return_statement: Option<ReturnStatement>,
}

enum Statement {}

struct ReturnStatement {
    expression_list: Vec<Expression>,
}

enum Expression {
}

fn parse_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    Err(nom::Err::Error(nom::error::Error::new(t, nom::error::ErrorKind::Tag)))
}

fn parse_return_statement(t: TokenSlice) -> IResult<TokenSlice, ReturnStatement> {
    Err(nom::Err::Error(nom::error::Error::new(t, nom::error::ErrorKind::Tag)))
}

pub fn parse(t: TokenSlice) -> IResult<TokenSlice, Block> {
    let (rest, statements) = many0(parse_statement).parse(t)?;
    let (_, return_statement) = opt(parse_return_statement).parse(rest)?;
    Ok((rest, Block {
        statements,
        return_statement,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let tokens = tokenize("if then else end").unwrap();
        assert_eq!(
            tokens,
            vec![Token::If, Token::Then, Token::Else, Token::End]
        );
    }

    #[test]
    fn test_identifiers() {
        let tokens = tokenize("local x = 42").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Local,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("42".to_string())
            ]
        );
    }

    #[test]
    fn test_operators() {
        let tokens = tokenize("a + b * c").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("a".to_string()),
                Token::Plus,
                Token::Identifier("b".to_string()),
                Token::Star,
                Token::Identifier("c".to_string())
            ]
        );
    }

    #[test]
    fn test_strings() {
        let tokens = tokenize(r#"local msg = "hello""#).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Local,
                Token::Identifier("msg".to_string()),
                Token::Equals,
                Token::String("hello".to_string())
            ]
        );
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize("x = 5 -- comment").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("5".to_string())
            ]
        );
    }

    #[test]
    fn test_if_statement() {
        let code = "if x > 5 then print(x) end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::If,
                Token::Identifier("x".to_string()),
                Token::Gt,
                Token::Number("5".to_string()),
                Token::Then,
                Token::Identifier("print".to_string()),
                Token::LParen,
                Token::Identifier("x".to_string()),
                Token::RParen,
                Token::End
            ]
        );
    }

    #[test]
    fn test_for_loop() {
        let code = "for i = 1, 10 do print(i) end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::For,
                Token::Identifier("i".to_string()),
                Token::Equals,
                Token::Number("1".to_string()),
                Token::Comma,
                Token::Number("10".to_string()),
                Token::Do,
                Token::Identifier("print".to_string()),
                Token::LParen,
                Token::Identifier("i".to_string()),
                Token::RParen,
                Token::End
            ]
        );
    }

    #[test]
    fn test_function_definition() {
        let code = "function add(a, b) return a + b end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Function,
                Token::Identifier("add".to_string()),
                Token::LParen,
                Token::Identifier("a".to_string()),
                Token::Comma,
                Token::Identifier("b".to_string()),
                Token::RParen,
                Token::Return,
                Token::Identifier("a".to_string()),
                Token::Plus,
                Token::Identifier("b".to_string()),
                Token::End
            ]
        );
    }

    #[test]
    fn test_table_constructor() {
        let code = "{1, 2, x = 3}";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LBrace,
                Token::Number("1".to_string()),
                Token::Comma,
                Token::Number("2".to_string()),
                Token::Comma,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("3".to_string()),
                Token::RBrace
            ]
        );
    }

    #[test]
    fn test_all_operators() {
        let code = "+ - * / // ^ % & | ~ >> << .. < <= > >= == ~=";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Plus,
                Token::Minus,
                Token::Star,
                Token::Slash,
                Token::DoubleSlash,
                Token::Caret,
                Token::Percent,
                Token::Ampersand,
                Token::Pipe,
                Token::Tilde,
                Token::RShift,
                Token::LShift,
                Token::Concat,
                Token::Lt,
                Token::Lte,
                Token::Gt,
                Token::Gte,
                Token::Eq,
                Token::Neq
            ]
        );
    }

    #[test]
    fn test_all_keywords() {
        let code = "and break do else elseif end false for function goto if in local nil not or repeat return then true until while";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::And,
                Token::Break,
                Token::Do,
                Token::Else,
                Token::Elseif,
                Token::End,
                Token::False,
                Token::For,
                Token::Function,
                Token::Goto,
                Token::If,
                Token::In,
                Token::Local,
                Token::Nil,
                Token::Not,
                Token::Or,
                Token::Repeat,
                Token::Return,
                Token::Then,
                Token::True,
                Token::Until,
                Token::While
            ]
        );
    }

    #[test]
    fn test_varargs() {
        let code = "function variadic(...) end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Function,
                Token::Identifier("variadic".to_string()),
                Token::LParen,
                Token::Varargs,
                Token::RParen,
                Token::End
            ]
        );
    }

    #[test]
    fn test_string_concatenation() {
        let code = r#"local msg = "hello" .. " " .. "world""#;
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Local,
                Token::Identifier("msg".to_string()),
                Token::Equals,
                Token::String("hello".to_string()),
                Token::Concat,
                Token::String(" ".to_string()),
                Token::Concat,
                Token::String("world".to_string())
            ]
        );
    }

    #[test]
    fn test_double_colon_label() {
        let code = "::start:: x = 1";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::DoubleColon,
                Token::Identifier("start".to_string()),
                Token::DoubleColon,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("1".to_string())
            ]
        );
    }

    #[test]
    fn test_method_call() {
        let code = "obj:method(arg)";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("obj".to_string()),
                Token::Colon,
                Token::Identifier("method".to_string()),
                Token::LParen,
                Token::Identifier("arg".to_string()),
                Token::RParen
            ]
        );
    }

    #[test]
    fn test_boolean_and_nil() {
        let code = "local x = true local y = false local z = nil";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Local,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::True,
                Token::Local,
                Token::Identifier("y".to_string()),
                Token::Equals,
                Token::False,
                Token::Local,
                Token::Identifier("z".to_string()),
                Token::Equals,
                Token::Nil
            ]
        );
    }

    #[test]
    fn test_while_loop() {
        let code = "while x < 10 do x = x + 1 end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::While,
                Token::Identifier("x".to_string()),
                Token::Lt,
                Token::Number("10".to_string()),
                Token::Do,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Identifier("x".to_string()),
                Token::Plus,
                Token::Number("1".to_string()),
                Token::End
            ]
        );
    }

    #[test]
    fn test_repeat_until_loop() {
        let code = "repeat x = x + 1 until x >= 10";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Repeat,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Identifier("x".to_string()),
                Token::Plus,
                Token::Number("1".to_string()),
                Token::Until,
                Token::Identifier("x".to_string()),
                Token::Gte,
                Token::Number("10".to_string())
            ]
        );
    }

    #[test]
    fn test_table_indexing() {
        let code = "t[1] t[\"key\"] t.field";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("t".to_string()),
                Token::LBracket,
                Token::Number("1".to_string()),
                Token::RBracket,
                Token::Identifier("t".to_string()),
                Token::LBracket,
                Token::String("key".to_string()),
                Token::RBracket,
                Token::Identifier("t".to_string()),
                Token::Dot,
                Token::Identifier("field".to_string())
            ]
        );
    }

    #[test]
    fn test_logical_operators() {
        let code = "x and y or not z";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::And,
                Token::Identifier("y".to_string()),
                Token::Or,
                Token::Not,
                Token::Identifier("z".to_string())
            ]
        );
    }

    #[test]
    fn test_complex_expression() {
        let code = "result = (a + b) * c - d / e ^ 2";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("result".to_string()),
                Token::Equals,
                Token::LParen,
                Token::Identifier("a".to_string()),
                Token::Plus,
                Token::Identifier("b".to_string()),
                Token::RParen,
                Token::Star,
                Token::Identifier("c".to_string()),
                Token::Minus,
                Token::Identifier("d".to_string()),
                Token::Slash,
                Token::Identifier("e".to_string()),
                Token::Caret,
                Token::Number("2".to_string())
            ]
        );
    }

    #[test]
    fn test_underscored_identifiers() {
        let code = "_var _G __private__ a_b_c";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("_var".to_string()),
                Token::Identifier("_G".to_string()),
                Token::Identifier("__private__".to_string()),
                Token::Identifier("a_b_c".to_string())
            ]
        );
    }

    #[test]
    fn test_decimal_numbers() {
        let code = "0 1 42 3.14 0.5 100.0";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number("0".to_string()),
                Token::Number("1".to_string()),
                Token::Number("42".to_string()),
                Token::Number("3.14".to_string()),
                Token::Number("0.5".to_string()),
                Token::Number("100.0".to_string())
            ]
        );
    }

    #[test]
    fn test_comment_variations() {
        let code = "x = 1 -- comment\ny = 2 -- another comment\nz = 3";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("1".to_string()),
                Token::Identifier("y".to_string()),
                Token::Equals,
                Token::Number("2".to_string()),
                Token::Identifier("z".to_string()),
                Token::Equals,
                Token::Number("3".to_string())
            ]
        );
    }

    #[test]
    fn test_local_function() {
        let code = "local function helper() return 42 end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Local,
                Token::Function,
                Token::Identifier("helper".to_string()),
                Token::LParen,
                Token::RParen,
                Token::Return,
                Token::Number("42".to_string()),
                Token::End
            ]
        );
    }

    #[test]
    fn test_goto_and_label() {
        let code = "goto skip ::skip:: x = 1";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Goto,
                Token::Identifier("skip".to_string()),
                Token::DoubleColon,
                Token::Identifier("skip".to_string()),
                Token::DoubleColon,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("1".to_string())
            ]
        );
    }

    #[test]
    fn test_unary_operators() {
        let code = "-x not y #z ~a";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Minus,
                Token::Identifier("x".to_string()),
                Token::Not,
                Token::Identifier("y".to_string()),
                Token::Hash,
                Token::Identifier("z".to_string()),
                Token::Tilde,
                Token::Identifier("a".to_string())
            ]
        );
    }

    #[test]
    fn test_bitwise_operators() {
        let code = "a & b | c ~ d a >> b a << b";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("a".to_string()),
                Token::Ampersand,
                Token::Identifier("b".to_string()),
                Token::Pipe,
                Token::Identifier("c".to_string()),
                Token::Tilde,
                Token::Identifier("d".to_string()),
                Token::Identifier("a".to_string()),
                Token::RShift,
                Token::Identifier("b".to_string()),
                Token::Identifier("a".to_string()),
                Token::LShift,
                Token::Identifier("b".to_string())
            ]
        );
    }

    #[test]
    fn test_mixed_whitespace() {
        let code = "x=1\t+\t2  \n  +   3";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("1".to_string()),
                Token::Plus,
                Token::Number("2".to_string()),
                Token::Plus,
                Token::Number("3".to_string())
            ]
        );
    }

    #[test]
    fn test_for_in_loop() {
        let code = "for k, v in pairs(t) do print(k, v) end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::For,
                Token::Identifier("k".to_string()),
                Token::Comma,
                Token::Identifier("v".to_string()),
                Token::In,
                Token::Identifier("pairs".to_string()),
                Token::LParen,
                Token::Identifier("t".to_string()),
                Token::RParen,
                Token::Do,
                Token::Identifier("print".to_string()),
                Token::LParen,
                Token::Identifier("k".to_string()),
                Token::Comma,
                Token::Identifier("v".to_string()),
                Token::RParen,
                Token::End
            ]
        );
    }

    #[test]
    fn test_empty_input() {
        let tokens = tokenize("").unwrap();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_only_whitespace() {
        let tokens = tokenize("   \n\t  \n  ").unwrap();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_only_comments() {
        let tokens = tokenize("-- comment\n-- another comment").unwrap();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_return_statement() {
        let code = "return a, b, c";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Return,
                Token::Identifier("a".to_string()),
                Token::Comma,
                Token::Identifier("b".to_string()),
                Token::Comma,
                Token::Identifier("c".to_string())
            ]
        );
    }

    #[test]
    fn test_break_statement() {
        let code = "while true do break end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::While,
                Token::True,
                Token::Do,
                Token::Break,
                Token::End
            ]
        );
    }

    #[test]
    fn test_do_block() {
        let code = "do local x = 1 end";
        let tokens = tokenize(code).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Do,
                Token::Local,
                Token::Identifier("x".to_string()),
                Token::Equals,
                Token::Number("1".to_string()),
                Token::End
            ]
        );
    }
}
