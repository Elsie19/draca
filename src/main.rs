use std::io::{self, Write};

use crate::{env::Environment, eval::eval, lexer::Token};

mod env;
mod eval;
mod lexer;
mod parser;
mod stdlib;

pub fn repl() {
    let mut math_env = Environment::math_std_env();

    loop {
        print!("draca-math> ");
        io::stdout().flush().unwrap();
        let expr = read_input().unwrap();

        match eval(&expr, &mut math_env) {
            Ok(val) => println!(" ==> {val}"),
            Err(e) => eprintln!(" ==> Error: {e}"),
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
