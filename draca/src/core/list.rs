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

pub fn cons(args: &[Expression]) -> Result<Expression, String> {
    if let [head, tail] = args {
        let mut new_lst = match tail {
            Expression::List(lst) => lst.clone(),
            Expression::Quoted(quote) => match &**quote {
                Expression::List(lst) => lst.clone(),
                _ => return Err("`cons` expects a list as second argument".into()),
            },
            _ => return Err("`cons` expects a list as second argument".into()),
        };

        new_lst.insert(0, head.clone());
        Ok(Expression::List(new_lst))
    } else {
        Err("`cons` requires exactly two arguments".into())
    }
}
