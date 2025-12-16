// Uses <https://github.com/chrischiedo/rustyscm> as a base.

mod env;
mod eval;
mod parser;
mod repl;
mod stdlib;

fn main() -> rustyline::Result<()> {
    repl::repl()
}
