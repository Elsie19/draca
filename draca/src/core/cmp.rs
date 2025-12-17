use std::ops::Not;

use crate::parser::Expression;

type Ret = Result<Expression, String>;

macro_rules! impl_cmp_ops {
    ($(($name:tt, $draca:expr)),* $(,)?) => {
        $(
            pub fn $name(args: &[Expression]) -> Ret {
                match args {
                    [first, second] => Ok(Expression::Bool(first.$name(second))),
                    _ => Err(format!("`{}` requires two arguments", stringify!($draca))),
                }
            }
        )*
    };
}

impl_cmp_ops![
    (eq, "="),
    (ne, "/="),
    (gt, ">"),
    (lt, "<"),
    (ge, ">="),
    (le, "<="),
];

pub fn not(args: &[Expression]) -> Ret {
    match args {
        [first] | [first, ..] => match first {
            Expression::Nil => Ok(Expression::Nil),
            Expression::Bool(b) => Ok(Expression::Bool(b.not())),
            _ => Err("`not` expects either a `nil` or a `bool`".into()),
        },
        _ => Err("`not` requires a single argument".into()),
    }
}
