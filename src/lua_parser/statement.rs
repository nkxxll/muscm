//! Statement parsing - assignments, control flow, declarations

use nom::{
    branch::alt,
    combinator::opt,
    multi::many0,
    IResult, Parser,
};

use super::{Token, TokenSlice, Statement, Expression, Block, ReturnStatement, token_tag};
use super::expression;

/// Parse a single statement
pub fn parse_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    alt((
        parse_empty_statement,
        parse_break_statement,
        parse_label_statement,
        parse_goto_statement,
        parse_do_block,
        parse_while_loop,
        parse_repeat_until,
        parse_if_statement,
        parse_for_loop,
        parse_function_decl,
        parse_local_statement,
        parse_assignment_or_call,
    ))
    .parse(t)
}

fn parse_empty_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Semicolon)(t)?;
    Ok((rest, Statement::Empty))
}

fn parse_break_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Break)(t)?;
    Ok((rest, Statement::Break))
}

fn parse_label_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::DoubleColon)(t)?;
    if let Some(Token::Identifier(name)) = rest.0.first() {
        let name = name.clone();
        let rest = TokenSlice(&rest.0[1..]);
        let (rest, _) = token_tag(&Token::DoubleColon)(rest)?;
        Ok((rest, Statement::Label(name)))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            rest,
            nom::error::ErrorKind::Tag,
        )))
    }
}

fn parse_goto_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Goto)(t)?;
    if let Some(Token::Identifier(name)) = rest.0.first() {
        let name = name.clone();
        let rest = TokenSlice(&rest.0[1..]);
        Ok((rest, Statement::Goto(name)))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            rest,
            nom::error::ErrorKind::Tag,
        )))
    }
}

fn parse_do_block(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Do)(t)?;
    let (rest, block) = parse_block(rest)?;
    let (rest, _) = token_tag(&Token::End)(rest)?;
    Ok((rest, Statement::Do(Box::new(block))))
}

fn parse_while_loop(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::While)(t)?;
    let (rest, condition) = expression::parse_expression(rest)?;
    let (rest, _) = token_tag(&Token::Do)(rest)?;
    let (rest, body) = parse_block(rest)?;
    let (rest, _) = token_tag(&Token::End)(rest)?;
    Ok((
        rest,
        Statement::While {
            condition,
            body: Box::new(body),
        },
    ))
}

fn parse_repeat_until(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Repeat)(t)?;
    let (rest, body) = parse_block(rest)?;
    let (rest, _) = token_tag(&Token::Until)(rest)?;
    let (rest, condition) = expression::parse_expression(rest)?;
    Ok((
        rest,
        Statement::Repeat {
            body: Box::new(body),
            condition,
        },
    ))
}

fn parse_if_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::If)(t)?;
    let (rest, condition) = expression::parse_expression(rest)?;
    let (rest, _) = token_tag(&Token::Then)(rest)?;
    let (rest, then_block) = parse_block(rest)?;

    // Parse elseif parts
    let (rest, elseif_parts) = many0(|input| {
        let (r, _) = token_tag(&Token::Elseif)(input)?;
        let (r, cond) = expression::parse_expression(r)?;
        let (r, _) = token_tag(&Token::Then)(r)?;
        let (r, blk) = parse_block(r)?;
        Ok((r, (cond, blk)))
    })
    .parse(rest)?;

    // Parse optional else block
    let (rest, else_block) = opt(|input| {
        let (r, _) = token_tag(&Token::Else)(input)?;
        parse_block(r).map(|(r, b)| (r, Box::new(b)))
    })
    .parse(rest)?;

    let (rest, _) = token_tag(&Token::End)(rest)?;

    Ok((
        rest,
        Statement::If {
            condition,
            then_block: Box::new(then_block),
            elseif_parts,
            else_block,
        },
    ))
}

fn parse_for_loop(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::For)(t)?;

    // Parse the first variable name
    if let Some(Token::Identifier(var_name)) = rest.0.first() {
        let var_name = var_name.clone();
        let rest = TokenSlice(&rest.0[1..]);

        // Try numeric for: var = start, end [, step]
        if let Ok((r, _)) = token_tag(&Token::Equals)(rest) {
            let (r, start) = expression::parse_expression(r)?;
            let (r, _) = token_tag(&Token::Comma)(r)?;
            let (r, end) = expression::parse_expression(r)?;

            // Optional step
            let (r, step) = opt(|input| {
                let (r, _) = token_tag(&Token::Comma)(input)?;
                expression::parse_expression(r)
            })
            .parse(r)?;

            let (r, _) = token_tag(&Token::Do)(r)?;
            let (r, body) = parse_block(r)?;
            let (r, _) = token_tag(&Token::End)(r)?;

            return Ok((
                r,
                Statement::ForNumeric {
                    var: var_name,
                    start,
                    end,
                    step,
                    body: Box::new(body),
                },
            ));
        }

        // Try generic for: [var1,] var2 [, var3, ...] in iterables
        let (rest, more_vars) = opt(|input| {
            let (r, _) = token_tag(&Token::Comma)(input)?;
            parse_namelist(r)
        })
        .parse(rest)?;

        if let Ok((r, _)) = token_tag(&Token::In)(rest) {
            let (r, iterables) = expression::parse_expression_list(r)?;
            let (r, _) = token_tag(&Token::Do)(r)?;
            let (r, body) = parse_block(r)?;
            let (r, _) = token_tag(&Token::End)(r)?;

            let mut vars = vec![var_name];
            if let Some(mut extra_vars) = more_vars {
                vars.append(&mut extra_vars);
            }

            return Ok((
                r,
                Statement::ForGeneric {
                    vars,
                    iterables,
                    body: Box::new(body),
                },
            ));
        }
    }

    Err(nom::Err::Error(nom::error::Error::new(
        t,
        nom::error::ErrorKind::Alt,
    )))
}

fn parse_function_decl(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Function)(t)?;

    // Parse function name - can be simple (foo) or qualified (M.test, a.b.c, or a:method)
    if let Some(Token::Identifier(name)) = rest.0.first() {
        let mut full_name = name.clone();
        let mut rest = TokenSlice(&rest.0[1..]);

        // Handle qualified names like M.test or a:method
        loop {
            if let Some(Token::Dot) = rest.0.first() {
                rest = TokenSlice(&rest.0[1..]);
                if let Some(Token::Identifier(member)) = rest.0.first() {
                    full_name.push('.');
                    full_name.push_str(member);
                    rest = TokenSlice(&rest.0[1..]);
                } else {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        rest,
                        nom::error::ErrorKind::Tag,
                    )));
                }
            } else if let Some(Token::Colon) = rest.0.first() {
                // Method definition (a:b becomes a.b with self parameter)
                rest = TokenSlice(&rest.0[1..]);
                if let Some(Token::Identifier(method)) = rest.0.first() {
                    full_name.push(':');
                    full_name.push_str(method);
                    rest = TokenSlice(&rest.0[1..]);
                } else {
                    return Err(nom::Err::Error(nom::error::Error::new(
                        rest,
                        nom::error::ErrorKind::Tag,
                    )));
                }
                break;
            } else {
                break;
            }
        }

        let (rest, body) = expression::parse_funcbody(rest)?;
        Ok((
            rest,
            Statement::FunctionDecl {
                name: full_name,
                body: Box::new(body),
            },
        ))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            rest,
            nom::error::ErrorKind::Tag,
        )))
    }
}

fn parse_local_statement(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, _) = token_tag(&Token::Local)(t)?;

    // Check if it's local function
    if let Ok((r, _)) = token_tag(&Token::Function)(rest) {
        if let Some(Token::Identifier(name)) = r.0.first() {
            let name = name.clone();
            let r = TokenSlice(&r.0[1..]);
            let (r, body) = expression::parse_funcbody(r)?;
            return Ok((
                r,
                Statement::LocalFunction {
                    name,
                    body: Box::new(body),
                },
            ));
        }
    }

    // Otherwise it's local vars [= values]
    let (rest, names) = parse_namelist(rest)?;
    let (rest, values) = opt(|input| {
        let (r, _) = token_tag(&Token::Equals)(input)?;
        expression::parse_expression_list(r)
    })
    .parse(rest)?;

    Ok((rest, Statement::LocalVars { names, values }))
}

fn parse_assignment_or_call(t: TokenSlice) -> IResult<TokenSlice, Statement> {
    let (rest, first_expr) = expression::parse_prefix_exp(t)?;

    // Check if this is an assignment by looking for more variables or =
    if let Ok((r, _)) = token_tag(&Token::Comma)(rest) {
        // Multiple variables: var1, var2, ... = ...
        let (r, rest_vars) = many0(|input| {
            let (r, expr) = expression::parse_prefix_exp(input)?;
            let (r, _) = token_tag(&Token::Comma)(r)?;
            Ok((r, expr))
        })
        .parse(r)?;

        // Now we must have = and values
        let (r, final_expr) = expression::parse_prefix_exp(r)?;
        let (r, _) = token_tag(&Token::Equals)(r)?;
        let (r, values) = expression::parse_expression_list(r)?;

        let mut variables = vec![first_expr];
        variables.extend(rest_vars);
        variables.push(final_expr);

        return Ok((r, Statement::Assignment { variables, values }));
    }

    // Try assignment: varlist = explist
    if let Ok((r, _)) = token_tag(&Token::Equals)(rest) {
        let (r, values) = expression::parse_expression_list(r)?;
        // Collect first_expr as a variable
        return Ok((
            r,
            Statement::Assignment {
                variables: vec![first_expr],
                values,
            },
        ));
    }

    // Try function call (prefix expression that is a function call)
    match &first_expr {
        Expression::FunctionCall { .. } | Expression::MethodCall { .. } => {
            Ok((rest, Statement::FunctionCall(first_expr)))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            t,
            nom::error::ErrorKind::Alt,
        ))),
    }
}

pub fn parse_return_statement(t: TokenSlice) -> IResult<TokenSlice, ReturnStatement> {
    let (rest, _) = token_tag(&Token::Return).parse(t)?;
    let (rest, list) = opt(expression::parse_expression_list).parse(rest)?;
    let (rest, _) = opt(token_tag(&Token::Semicolon)).parse(rest)?;
    Ok((
        rest,
        ReturnStatement {
            expression_list: list.unwrap_or_default(),
        },
    ))
}

/// Parse name list: `name {',' name}`
fn parse_namelist(t: TokenSlice) -> IResult<TokenSlice, Vec<String>> {
    let (rest, first_name) = if let Some(Token::Identifier(name)) = t.0.first() {
        (TokenSlice(&t.0[1..]), name.clone())
    } else {
        return Err(nom::Err::Error(nom::error::Error::new(
            t,
            nom::error::ErrorKind::Tag,
        )));
    };

    let (rest, rest_names) = many0(|input| {
        let (r, _) = token_tag(&Token::Comma)(input)?;
        if let Some(Token::Identifier(name)) = r.0.first() {
            Ok((TokenSlice(&r.0[1..]), name.clone()))
        } else {
            Err(nom::Err::Error(nom::error::Error::new(
                r,
                nom::error::ErrorKind::Tag,
            )))
        }
    })
    .parse(rest)?;

    let mut result = vec![first_name];
    result.extend(rest_names);
    Ok((rest, result))
}

/// Parse a block of statements, stopping at block-terminating tokens
/// Block terminators: 'end', 'else', 'elseif', 'until', EOF
pub fn parse_block(t: TokenSlice) -> IResult<TokenSlice, Block> {
    let mut statements = Vec::new();
    let mut current = t;

    // Parse statements until we hit a block terminator
    loop {
        // Check if we've hit a block terminator or EOF
        if current.0.is_empty() {
            break;
        }

        // Check for block terminating tokens
        if let Some(token) = current.0.first() {
            match token {
                Token::End | Token::Else | Token::Elseif | Token::Until => {
                    break;
                }
                _ => {}
            }
        }

        // Try to parse a return statement first (since it can be followed by anything)
        if let Ok((rest, ret_stmt)) = parse_return_statement(current) {
            return Ok((
                rest,
                Block {
                    statements,
                    return_statement: Some(ret_stmt),
                },
            ));
        }

        // Try to parse a regular statement
        match parse_statement(current) {
            Ok((rest, stmt)) => {
                statements.push(stmt);
                current = rest;
            }
            Err(_) => {
                // If we can't parse a statement, we're done with the block
                break;
            }
        }
    }

    Ok((
        current,
        Block {
            statements,
            return_statement: None,
        },
    ))
}
