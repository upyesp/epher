//! epher-cli, native command-line frontend (ADR-0001).
//!
//! The dev/test binary for this crate: the same argument surface as the
//! unified `epher` executable (ADR-0011), one-shot evaluation, `-` piped
//! scripts, `repl`, and `help`, through the shared [`epher_cli::dispatch`].
//! The `tui` and `gui` frontends live in the unified binary only; here
//! they are an explicit error instead of a silent difference.

use clap::Parser;

use epher_cli::dispatch::{action_from, Action, Args};

fn main() {
    // Script evaluation recurses per expression node, and collection
    // searches legitimately reach hundreds of thousands of frames; the
    // default 8 MiB main-thread stack overflows first. Run the frontend
    // on a dedicated fat-stack thread: the reservation is virtual only,
    // pages commit as frames actually nest.
    let child = std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(real_main)
        .expect("spawn the main thread with a fat stack");
    let code = child.join().expect("main thread panicked");
    std::process::exit(code);
}

fn real_main() -> i32 {
    let args = Args::parse_from(std::env::args_os());
    let result = match action_from(&args) {
        Action::OneShot(expr) => epher_cli::run_one_shot(&expr),
        Action::Stdin => epher_cli::run_stdin_and_exit(),
        Action::ScriptFile(path) => match epher_cli::run_script_file(&path) {
            Ok(false) => Ok(()),
            Ok(true) => return 1,
            Err(e) => Err(e),
        },
        Action::MissingScriptFile(path) => {
            epher_cli::term::error(&format!("error: no such script file: {path}"));
            return 1;
        }
        Action::Repl => epher_cli::run_repl(),
        Action::HelpManual => return epher_cli::help::manual(),
        Action::HelpTopic(topic) => return epher_cli::help::topic(&topic),
        Action::Tui | Action::Gui(_) => {
            epher_cli::term::error(
                "the tui/gui frontends are part of the unified `epher` binary, not this dev binary",
            );
            return 2;
        }
    };
    match result {
        Ok(()) => 0,
        Err(e) => {
            epher_cli::term::error(&format!("error: {e}"));
            1
        }
    }
}
