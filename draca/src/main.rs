// Uses <https://github.com/chrischiedo/rustyscm> as a base.

use std::error::Error;

mod env;
mod eval;
mod parser;
mod repl;
mod stdlib;

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    match args.as_slice() {
        [repl] | [repl, ..] if repl == "-r" => Ok(repl::repl()?),
        [file] => env::run_file(file),
        _ => unimplemented!("Working on it"),
    }
}
