//! epher-cli — native command-line frontend (ADR-0001).
//!
//! The dev/test binary for this crate: the same argument surface as the
//! unified `epher` executable (ADR-0011) — one-shot evaluation, `-` piped
//! scripts, `repl`, and `help` — through the shared [`epher_cli::dispatch`].
//! The `tui` and `gui` frontends live in the unified binary only; here
//! they are an explicit error instead of a silent difference.

use clap::Parser;

use epher_cli::dispatch::{action_from, Action, Args};

fn main() {
    let args = Args::parse_from(std::env::args_os());
    let result = match action_from(&args) {
        Action::OneShot(expr) => epher_cli::run_one_shot(&expr),
        Action::Stdin => epher_cli::run_stdin_and_exit(),
        Action::ScriptFile(path) => epher_cli::run_script_file(&path).and_then(|failed| {
            if failed {
                std::process::exit(1);
            }
            Ok(())
        }),
        Action::MissingScriptFile(path) => {
            epher_cli::term::error(&format!("error: no such script file: {path}"));
            std::process::exit(1);
        }
        Action::Repl => epher_cli::run_repl(),
        Action::HelpManual => std::process::exit(epher_cli::help::manual()),
        Action::HelpTopic(topic) => epher_cli::help::topic(&topic),
        Action::Tui | Action::Gui => {
            epher_cli::term::error(
                "the tui/gui frontends are part of the unified `epher` binary, not this dev binary",
            );
            std::process::exit(2);
        }
    };
    if let Err(e) = result {
        epher_cli::term::error(&format!("error: {e}"));
        std::process::exit(1);
    }
}
