use std::{borrow::Cow, collections::HashSet};

use ansi_term::{Color, Style};
use rustyline::{
    CompletionType, Config, Context, EditMode, Editor, Helper, Hinter,
    completion::{Completer, Pair, extract_word},
    error::ReadlineError,
    highlight::{CmdKind, Highlighter, MatchingBracketHighlighter},
    history::MemHistory,
    validate::{ValidationContext, ValidationResult, Validator},
};

use crate::{env::Environment, eval::eval, lisp, parser::parse};

// TODO: Make this less spaghetti.

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

        if word.starts_with('(') {
            word = word.trim_start_matches('(');
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
                    }
                    .to_string();
                    Some(Pair {
                        display: hint.cmd.to_string(),
                        replacement,
                    })
                } else {
                    None
                }
            })
            .collect();
        Ok((start, matches))
    }
}

#[derive(Helper, Hinter)]
struct DracaHelper {
    cmd_completer: CommandCompleter,
    highligher: MatchingBracketHighlighter,
}

impl DracaHelper {
    pub fn from_env(env: &Environment) -> Self {
        Self {
            cmd_completer: CommandCompleter {
                cmds: cmd_sets(env),
            },
            highligher: MatchingBracketHighlighter::new(),
        }
    }
}

impl Highlighter for DracaHelper {
    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highligher.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, kind: CmdKind) -> bool {
        self.highligher.highlight_char(line, pos, kind)
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        self.highligher.highlight_hint(hint)
    }

    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        self.highligher.highlight_prompt(prompt, default)
    }

    fn highlight_candidate<'c>(
        &self,
        candidate: &'c str,
        completion: CompletionType,
    ) -> Cow<'c, str> {
        self.highligher.highlight_candidate(candidate, completion)
    }
}

impl Validator for DracaHelper {
    fn validate(&self, ctx: &mut ValidationContext) -> rustyline::Result<ValidationResult> {
        let parse_results = parse(ctx.input());

        match parse_results {
            Ok(_) => Ok(ValidationResult::Valid(None)),
            Err(_) => Ok(ValidationResult::Incomplete),
        }
    }

    fn validate_while_typing(&self) -> bool {
        false
    }
}

impl Completer for DracaHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> Result<(usize, Vec<Pair>), ReadlineError> {
        self.cmd_completer.find_matches(line, pos)
    }
}

fn cmd_sets(env: &Environment) -> HashSet<Command> {
    let mut set = HashSet::new();

    // Do internal calls first.
    for it in [
        "define",
        "define/in-namespace",
        "namespace/symbol",
        "namespace/as-list",
        "quote",
        "eval-file",
        "require",
        "deconst-fn",
        "if",
    ] {
        set.insert(Command::new(it, ""));
        set.insert(Command::new("", it));
    }

    // Then do user set things.
    for (qualified_path, it) in env.full_path_and_name() {
        set.insert(Command::new(&qualified_path, it));
        set.insert(Command::new(it, &qualified_path));

        set.insert(Command::new(it, ""));
        set.insert(Command::new(&qualified_path, ""));
    }

    set
}

pub fn repl() -> rustyline::Result<()> {
    let mut env = Environment::empty().rust_builtins().stdlib().build();

    let config = Config::builder()
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Vi)
        .build();

    let mut rl = Editor::<DracaHelper, _>::with_history(config, MemHistory::new())?;

    rl.set_helper(Some(DracaHelper::from_env(&env)));

    println!(
        "Draca REPL {}.\nTo exit, type `(std::sys::exit)` or press `^D`.",
        env!("CARGO_PKG_VERSION")
    );

    loop {
        let readline = rl.readline(&Color::White.bold().paint("\\> ").to_string());
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
                        Ok(val) => println!("{}", Style::new().dimmed().paint(val.to_string())),
                        Err(e) => eprintln!("==> Error: {e}"),
                    }
                }
                rl.add_history_entry(&line)?;
                // Make sure we update the environment because the user may have defined something!
                rl.set_helper(Some(DracaHelper::from_env(&env)));
            }
            Err(ReadlineError::Interrupted) => {
                eprintln!("^C");
            }
            Err(ReadlineError::Eof) => {
                eprintln!("^D");
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
