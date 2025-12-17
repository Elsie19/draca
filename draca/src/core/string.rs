use crate::{core::list::extract_list, parser::Expression};

pub fn as_list(args: &[Expression]) -> Result<Expression, String> {
    match args {
        [first, ..] => {
            if let Expression::String(str) = first {
                Ok(Expression::List(
                    str.chars()
                        .map(|c| Expression::String(String::from(c)))
                        .collect(),
                ))
            } else {
                Err("`string->list` accepts a string".into())
            }
        }
        _ => Err("`string->list` requires one argument".into()),
    }
}

pub fn from_list(args: &[Expression]) -> Result<Expression, String> {
    match args {
        [first, ..] => {
            let lst = extract_list(first)?;
            Ok(Expression::String(String::from_iter(
                lst.iter().map(Expression::fmt_string),
            )))
        }
        _ => Err("`list->string` requires one argument".into()),
    }
}
