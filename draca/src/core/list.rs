use crate::{empty_quoted_list, num, parser::Expression};

pub fn car(args: &[Expression]) -> Result<Expression, String> {
    match args {
        [first, ..] => match first {
            Expression::List(lst) => Ok(lst.first().cloned().unwrap_or(Expression::Nil)),
            Expression::Quoted(quote) => match &**quote {
                Expression::List(lst) => Ok(lst.first().cloned().unwrap_or(Expression::Nil)),
                _ => Ok(empty_quoted_list!()),
            },
            _ => Ok(empty_quoted_list!()),
        },
        _ => Err("`car` requires one argument".into()),
    }
}

pub fn cdr(args: &[Expression]) -> Result<Expression, String> {
    match args {
        [Expression::List(lst), ..] => {
            if lst.is_empty() {
                Ok(Expression::Nil)
            } else {
                Ok(Expression::List(lst[1..].to_vec()))
            }
        }
        [Expression::Quoted(quote), ..] => match &**quote {
            Expression::List(lst) => {
                if lst.is_empty() {
                    Ok(Expression::Nil)
                } else {
                    Ok(Expression::List(lst[1..].to_vec()))
                }
            }
            _ => Ok(Expression::Nil),
        },
        _ => Err("`cdr` requires one argument".into()),
    }
}

pub fn cons(args: &[Expression]) -> Result<Expression, String> {
    if let [head, tail] = args {
        let mut new_lst = extract_list(tail)?;

        new_lst.insert(0, head.clone());
        Ok(Expression::List(new_lst))
    } else {
        Err("`cons` requires exactly two arguments".into())
    }
}

pub fn append(args: &[Expression]) -> Result<Expression, String> {
    if let [head, tail] = args {
        let mut head_lst = extract_list(head)?;
        let tail_lst = extract_list(tail)?;

        head_lst.extend(tail_lst);

        Ok(Expression::List(head_lst))
    } else {
        Err("`append` requires exactly two arguments".into())
    }
}

pub fn list(args: &[Expression]) -> Result<Expression, String> {
    Ok(Expression::List(args.to_vec()))
}

pub fn is_empty(args: &[Expression]) -> Result<Expression, String> {
    match args {
        [Expression::List(lst)] => Ok(Expression::Bool(lst.is_empty())),
        [Expression::Nil] => Ok(Expression::Bool(true)),
        [Expression::Quoted(q)] => match &**q {
            Expression::List(lst) => Ok(Expression::Bool(lst.is_empty())),
            Expression::Nil => Ok(Expression::Bool(true)),
            _ => Ok(Expression::Bool(false)),
        },
        _ => Ok(Expression::Bool(false)),
    }
}

pub fn len(args: &[Expression]) -> Result<Expression, String> {
    match args {
        [first] => Ok(num!(extract_list(first)?.len() as f64)),
        [_, rest @ ..] => Ok(num!(rest.len() as f64 + 1.0)),
        _ => Ok(num!(0.0)),
    }
}

pub(crate) fn extract_list(expr: &Expression) -> Result<Vec<Expression>, String> {
    match expr {
        Expression::List(lst) => Ok(lst.clone()),
        Expression::Quoted(boxed) => match &**boxed {
            Expression::List(lst) => Ok(lst.clone()),
            _ => Err("expected a list".into()),
        },
        _ => Err("expected a list".into()),
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Mutex, OnceLock};

    use crate::{env::Environment, lisp};

    use super::*;

    static ENV: OnceLock<Mutex<Environment>> = OnceLock::new();

    fn setup_env() -> &'static Mutex<Environment> {
        ENV.get_or_init(|| Mutex::new(Environment::empty().core().stdlib().build()))
    }

    #[test]
    fn car() {
        let env = setup_env();
        let mut env = env.lock().unwrap();
        let list = lisp!("(list 1 2 3)", &mut env);

        assert_eq!(super::car(&[list]).unwrap(), Expression::Number(1.0));
    }

    #[test]
    fn cdr() {
        let env = setup_env();
        let mut env = env.lock().unwrap();
        let list = lisp!("(list 1 2 3)", &mut env);

        assert_eq!(super::cdr(&[list]).unwrap(), lisp!("(list 2 3)", &mut env));
    }
}
