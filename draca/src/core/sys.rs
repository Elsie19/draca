use crate::parser::Expression;

pub fn exit(args: &[Expression]) -> Result<Expression, String> {
    let num: Result<i32, String> = match args {
        [arg] | [_, arg] | [_, arg, ..] => {
            if let Expression::Number(first) = arg {
                Ok(*first as i32)
            } else {
                Err("invalid arguments, not numbers".into())
            }
        }
        [] => Ok(0),
    };

    std::process::exit(num?)
}
