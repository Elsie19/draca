use crate::{empty_quoted_list, parser::Expression};

pub fn car(args: &[Expression]) -> Result<Expression, String> {
    match args {
        [first, ..] => match first {
            Expression::List(lst) => {
                Ok(lst.first().cloned().unwrap_or_else(|| empty_quoted_list!()))
            }
            Expression::Quoted(quote) => match &**quote {
                Expression::List(lst) => {
                    Ok(lst.first().cloned().unwrap_or_else(|| empty_quoted_list!()))
                }
                _ => Ok(empty_quoted_list!()),
            },
            _ => Ok(empty_quoted_list!()),
        },
        _ => Err("`car` requires one argument".into()),
    }
}

pub fn cdr(args: &[Expression]) -> Result<Expression, String> {
    match args {
        [first, ..] => match first {
            Expression::List(lst) => {
                Ok(lst.last().cloned().unwrap_or_else(|| empty_quoted_list!()))
            }
            Expression::Quoted(quote) => match &**quote {
                Expression::List(lst) => {
                    Ok(lst.last().cloned().unwrap_or_else(|| empty_quoted_list!()))
                }
                _ => Ok(empty_quoted_list!()),
            },
            _ => Ok(empty_quoted_list!()),
        },
        _ => Err("`cdr` requires one argument".into()),
    }
}
