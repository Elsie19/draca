use std::collections::HashMap;

use strfmt::strfmt;

use crate::parser::Expression;

pub fn panic(args: &[Expression]) -> Result<Expression, String> {
    let out = format(args)?;

    panic!("{}", out.fmt_string());
}

pub fn format(args: &[Expression]) -> Result<Expression, String> {
    match &args {
        [single] => Ok(Expression::String(single.fmt_string())),
        [fmt_string, rest @ ..] => {
            let Expression::String(fmt_string) = fmt_string else {
                return Err("format string must begin with string".into());
            };

            let fmted_args = rest.iter().map(Expression::fmt_string);

            let mut vars = HashMap::new();

            for (idx, arg) in fmted_args.enumerate() {
                vars.insert(idx, arg);
            }

            let ret = strfmt(fmt_string, &vars).map_err(|e| e.to_string())?;

            Ok(Expression::String(ret))
        }
        [] => Err("format requires at least one argument".into()),
    }
}

pub fn println(args: &[Expression]) -> Result<Expression, String> {
    let out = format(args)?;

    println!("{}", out.fmt_string());

    Ok(Expression::Bool(true))
}
