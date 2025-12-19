use crate::parser::Expression;

macro_rules! binary_op {
    ($name:ident, $op:tt) => {
        pub fn $name(args: &[Expression]) -> Result<Expression, String> {
            match args {
                [Expression::Number(first), Expression::Number(second)] => {
                    Ok(Expression::Number(first $op second))
                }
                [_, _] => Err("invalid arguments, not numbers".into()),
                _ => Err(format!("`{}` requires at least two arguments", stringify!($name))),
            }
        }
    };
}

macro_rules! method_op {
    ($name:ident, $op:tt) => {
        pub fn $name(args: &[Expression]) -> Result<Expression, String> {
            match args {
                [Expression::Number(first), Expression::Number(second)] => {
                    Ok(Expression::Number(first.$op(*second)))
                }
                [_, _] => Err("invalid arguments, not numbers".into()),
                _ => Err(format!(
                    "`{}` requires at least two arguments",
                    stringify!($name)
                )),
            }
        }
    };
}

macro_rules! def_ops {
    ( $( ($name:ident, $op:tt) ),* $(,)? ) => {
        $(
            pub fn $name(args: &[Expression]) -> Result<Expression, String> {
                match args {
                    [Expression::Number(start), tail @ ..] => {
                        let mut base = *start;
                        for arg in tail {
                            match arg {
                                Expression::Number(n) => base $op n,
                                _ => return Err(String::from("Expected a number")),
                            }
                        }
                        Ok(Expression::Number(base))
                    }
                    [_, _] => Err(String::from("Expected a number")),
                    _ => Err(format!("`{}` requires at least two arguments", stringify!($name))),
                }
            }
        )*
    };
}

// Sub has a special case with just one argument which inverts it.
pub fn sub(args: &[Expression]) -> Result<Expression, String> {
    match args {
        [single] => {
            if let Expression::Number(n) = single {
                Ok(Expression::Number(n.abs()))
            } else {
                Err(String::from("Expected a number"))
            }
        }
        [start, tail @ ..] => {
            let mut base = if let Expression::Number(n) = start {
                *n
            } else {
                return Err(String::from("Expected a number"));
            };
            for arg in tail {
                match arg {
                    Expression::Number(n) => base -= n,
                    _ => return Err(String::from("Expected a number")),
                }
            }
            Ok(Expression::Number(base))
        }
        _ => panic!("oops"),
    }
}

pub fn ash(args: &[Expression]) -> Result<Expression, String> {
    match args {
        [Expression::Number(first), Expression::Number(second)] => {
            if second.is_sign_negative() {
                Ok(Expression::Number(
                    ((first.round() as u64) >> second.round() as u64) as f64,
                ))
            } else {
                Ok(Expression::Number(
                    ((first.round() as u64) << second.round() as u64) as f64,
                ))
            }
        }
        [_, _] => Err("invalid arguments, not numbers".into()),
        _ => Err("`ash` requires at least two arguments".into()),
    }
}

def_ops![
    (add, +=),
    (mul, *=),
    (div, /=),
];

binary_op!(rem, %);
method_op!(pow, powf);
