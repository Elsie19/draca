use crate::parser::Expression;

pub fn eq(args: &[Expression]) -> Result<Expression, String> {
    if args.len() != 2 {
        return Err(String::from("`=` requires two arguments"));
    }

    match args {
        [first, second] => Ok(Expression::Bool(first.eq(second))),
        _ => unreachable!("We checked above"),
    }
}
