// Uses <https://github.com/chrischiedo/rustyscm> as a base.

use std::collections::HashSet;

use rustyline::{
    CompletionType, Config, Context, EditMode, Editor, Helper, Highlighter, Hinter, Validator,
    completion::{Completer, Pair, extract_word},
    error::ReadlineError,
    history::MemHistory,
};

use crate::{env::Environment, eval::eval, parser::parse};

mod env;
mod eval;
mod parser;
mod stdlib;

fn is_break_char(c: char) -> bool {
    [' ', '\t', '\n'].contains(&c)
}

#[derive(Hash, Debug, PartialEq, Eq)]
struct Command {
    cmd: String,
    pre_cmd: String,
}

impl Command {
    fn new(cmd: &str, pre_cmd: &str) -> Self {
        Self {
            cmd: cmd.into(),
            pre_cmd: pre_cmd.into(),
        }
    }
}

struct CommandCompleter {
    cmds: HashSet<Command>,
}

impl CommandCompleter {
    pub fn find_matches(&self, line: &str, pos: usize) -> rustyline::Result<(usize, Vec<Pair>)> {
        let mut starts_with_paren = false;
        let (start, mut word) = extract_word(line, pos, None, is_break_char);
        let pre_cmd = line[..start].trim();

        if word.starts_with("(") {
            word = word.trim_start_matches("(");
            starts_with_paren = true;
        }

        let matches = self
            .cmds
            .iter()
            .filter_map(|hint| {
                if hint.cmd.starts_with(word) && pre_cmd == hint.pre_cmd {
                    let replacement = if starts_with_paren {
                        format!("({}", hint.cmd.clone())
                    } else {
                        hint.cmd.clone()
                    };
                    // replacement += " ";
                    Some(Pair {
                        display: hint.cmd.to_string(),
                        replacement: replacement.to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok((start, matches))
    }
}

#[derive(Helper, Hinter, Validator, Highlighter)]
struct DracaHelper {
    cmd_completer: CommandCompleter,
}

impl Completer for DracaHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        match self.cmd_completer.find_matches(line, pos) {
            Ok((start, matches)) => Ok((start, matches)),
            Err(e) => Err(e),
        }
    }
}

fn cmd_sets(env: &Environment) -> HashSet<Command> {
    let mut set = HashSet::new();
    for (qualified_path, it) in env.full_path_and_name() {
        set.insert(Command::new(&qualified_path, it));
        set.insert(Command::new(it, &qualified_path));

        set.insert(Command::new(it, ""));
        set.insert(Command::new(&qualified_path, ""));
    }

    set
}

pub fn repl() -> rustyline::Result<()> {
    let mut env = Environment::empty()
        .macros_plugin()
        .sys_plugin()
        .math_plugin()
        .cmp_plugin()
        .build();

    let h = DracaHelper {
        cmd_completer: CommandCompleter {
            cmds: cmd_sets(&env),
        },
    };

    let config = Config::builder()
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .build();

    let mut rl = Editor::<DracaHelper, _>::with_history(config, MemHistory::new())?;

    rl.set_helper(Some(h));

    println!(
        "Draca REPL {}.\nTo exit, type `(std::sys::exit)` or press `^D`.",
        env!("CARGO_PKG_VERSION")
    );

    loop {
        let readline = rl.readline("\\> ");
        match readline {
            Ok(line) => {
                let parsed_list = match parse(line.trim()) {
                    Ok(val) => val,
                    Err(e) => {
                        eprintln!("{e}");
                        continue;
                    }
                };

                for expr in parsed_list {
                    match eval(expr, &mut env) {
                        Ok(val) => println!("{val}"),
                        Err(e) => eprintln!("==> Error: {e}"),
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
