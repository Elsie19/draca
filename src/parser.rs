use std::fmt::Display;

use crate::{env::Environment, lexer::Token};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Expression {
    Bool(bool),
    Number(f64),
    Symbol(String),
    List(Vec<Expression>),
    Func(fn(&[Expression]) -> Expression),
    Function(Procedure),
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expression::Bool(b) => write!(f, "{b}"),
            Expression::Number(n) => write!(f, "{n}"),
            Expression::Symbol(s) => write!(f, "{s}"),
            Expression::List(list) => {
                let formatted_list: Vec<_> = list.iter().map(ToString::to_string).collect();
                write!(f, "({})", formatted_list.join(" "))
            }
            Expression::Func(_) | Self::Function(_) => write!(f, "<function>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Procedure {
    pub params: Vec<Expression>,
    pub body: Vec<Expression>,
    pub env: Environment,
}

pub fn parse(input: &str) -> Result<Expression, String> {
    let token_result = Token::tokenize(input)?;

    let mut tokens = token_result.into_iter().rev().collect();

    parse_token_list(&mut tokens)
}

fn parse_token_list(tokens: &mut Vec<Token>) -> Result<Expression, String> {
    let token = tokens.pop();

    if token != Some(Token::OpenParen) {
        return Err(format!("Error: Expected OpenParen, found {:?}", token));
    }

    let mut list = vec![];

    while !tokens.is_empty() {
        let token = tokens.pop();

        if token.is_none() {
            return Err("Error: Did not find enough tokens".to_string());
        }

        let tok = token.unwrap();

        match tok {
            Token::Number(n) => list.push(Expression::Number(n)),
            Token::Symbol(s) => list.push(Expression::Symbol(s)),
            Token::OpenParen => {
                tokens.push(Token::OpenParen);
                let sub_list = parse_token_list(tokens)?;
                list.push(sub_list);
            }
            Token::CloseParen => {
                return Ok(Expression::List(list));
            }
        }
    }

    Ok(Expression::List(list))
}
