use crate::parser::Expression;

pub fn exit(args: &[Expression]) -> Result<Expression, String> {
    if args.is_empty() {
        return Err("`exit` requires at least one argument".into());
    }

    match args {
        [arg] | [_, arg] | [_, arg, ..] => {
            if let Expression::Number(first) = arg {
                std::process::exit(*first as i32)
            } else {
                Err("invalid arguments, not numbers".into())
            }
        }
        _ => unreachable!("Checked above"),
    }
}
