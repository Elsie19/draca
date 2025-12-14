// Uses <https://github.com/chrischiedo/rustyscm> as a base.

use std::io::{self, Write};

use crate::{env::Environment, eval::eval, parser::parse};

mod env;
mod eval;
mod parser;
mod stdlib;

pub fn repl() {
    let mut math_env = Environment::empty()
        .sys_plugin()
        .math_plugin()
        .cmp_plugin()
        .build();

    loop {
        print!("draca-math> ");
        io::stdout().flush().unwrap();
        let expr = read_input().unwrap();

        let parsed_list = match parse(&expr) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Oops, {:?}", e);
                std::process::exit(1);
            }
        };

        for expr in parsed_list {
            match eval(expr, &mut math_env) {
                Ok(val) => println!(" ==> Ok: {val}"),
                Err(e) => eprintln!(" ==> Error: {e}"),
            }
        }
    }
}

fn read_input() -> io::Result<String> {
    let mut raw_input = String::new();

    io::stdin().read_line(&mut raw_input)?;

    Ok(raw_input)
}

fn main() {
    repl();
}
