// Uses <https://github.com/chrischiedo/rustyscm> as a base.

use std::error::Error;

mod core;
mod env;
mod eval;
mod parser;
mod repl;

const HELP: &str = "
draca --help

Usage: draca [-r] [file]

Options:
    -r      invoke the repl.
"
.trim_ascii();

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();

    match args.as_slice() {
        [repl] | [repl, ..] if repl == "-r" => Ok(repl::repl()?),
        [help] if help == "-h" => {
            println!("{HELP}");
            Ok(())
        }
        [file] if !file.starts_with('-') => env::run_file(file),
        _ => {
            eprintln!("{HELP}");
            std::process::exit(1)
        }
    }
}
