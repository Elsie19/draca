use crate::parser::Expression;

type Ret = Result<Expression, String>;

macro_rules! impl_cmp_ops {
    ($(($name:tt, $draca:expr)),* $(,)?) => {
        $(
            pub fn $name(args: &[Expression]) -> Ret {
                if args.len() != 2 {
                    return Err(format!("`{}` requires two arguments", stringify!($name)));
                }

                match args {
                    [first, second] => Ok(Expression::Bool(first.$name(second))),
                    _ => unreachable!("We checked above"),
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
