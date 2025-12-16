// Uses <https://github.com/chrischiedo/rustyscm> as a base.

use std::io::{self, Write};

use rustyline::{Config, DefaultEditor, Editor, error::ReadlineError, history::MemHistory};

use crate::{env::Environment, eval::eval, parser::parse};

mod env;
mod eval;
mod parser;
mod stdlib;

pub fn repl() -> rustyline::Result<()> {
    let mut env = Environment::empty()
        .macros_plugin()
        .sys_plugin()
        .math_plugin()
        .cmp_plugin()
        .build();

    let mut rl = Editor::<(), _>::with_history(Config::builder().build(), MemHistory::new())?;

    loop {
        let readline = rl.readline("\\> ");
        match readline {
            Ok(line) => {
                let parsed_list = match parse(line.trim()) {
                    Ok(val) => val,
                    Err(e) => {
                        eprintln!("{e}");
                        std::process::exit(1);
                    }
                };

                for expr in parsed_list {
                    match eval(expr, &mut env) {
                        Ok(val) => println!(" ==> Ok: {val}"),
                        Err(e) => eprintln!(" ==> Error: {e}"),
                    }
                }
                rl.add_history_entry(&line)?;
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("{err:?}");
                break;
            }
        }
    }

    Ok(())
}

fn main() -> rustyline::Result<()> {
    repl()
}
