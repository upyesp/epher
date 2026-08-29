//! epher-cli — native command-line frontend (ADR-0001).
//!
//! The library hosts every mode so the unified `epher` binary
//! (crates/tauri-app) can offer the same behavior without duplicating a
//! line:
//!
//! - [`run_one_shot`] — evaluate a single expression, print the result;
//! - [`run_repl`] — interactive REPL (prompt, persistent store);
//! - [`run_stdin_and_exit`] — piped script mode (`epher -`): evaluate
//!   stdin line by line, no prompts, history untouched.
//!
//! All three share [`step`], the one-line-at-a-time seam that classifies a
//! line (shell command vs. language statement), runs it against the shared
//! session/store, and reports the printed output plus any language switch.
//!
//! The command-line conventions live here too (ADR-0013): [`dispatch`]
//! defines the argument surface, [`help`] the manual/help behavior, and
//! [`term`] the stdout/stderr/color policy — results on stdout, errors on
//! stderr with exit codes 0/1/2.

pub mod dispatch;
pub mod help;
pub mod term;

use std::io::{self, BufRead, Write};

use epher_core::{EpherError, Session};
use epher_i18n::Localizer;
use epher_shell::{classify, plain, run_command};
use epher_store::persist::{
    default_store_dir, load_language, load_session, save_history, save_session,
};
use epher_store::{DocStore, FsStore};

/// The outcome of processing one line: what to print (if anything),
/// whether it is a diagnostic (stderr, not data — ADR-0013), and a
/// language switch requested by a `lang` command (if any).
pub struct Step {
    pub output: Option<String>,
    pub error: bool,
    pub language: Option<String>,
}

/// Is this engine output an error line? The engine renders errors as
/// `error: …` — the only producer of that prefix on the result path.
fn is_engine_error(output: &str) -> bool {
    output.starts_with("error: ")
}

/// Process one input line against the session and store. Shell commands
/// (`save`, `lang`, …) run through epher-shell; anything else is a language
/// statement evaluated against the session. Errors come back as
/// `error: …` output marked `Step::error` — the session stays usable,
/// exactly like the REPL.
pub fn step(
    session: &mut Session,
    store: &DocStore<FsStore>,
    localizer: &Localizer,
    line: &str,
) -> Step {
    if let Some(cmd) = classify(line) {
        let handled = run_command(&cmd, session, store, localizer);
        let message = plain(handled.message);
        // Diagnostics get the same `error:` voice as engine errors (the
        // message itself stays prefix-free for the GUI/TUI inline lines).
        return if handled.error {
            Step {
                output: Some(format!("error: {message}")),
                error: true,
                language: handled.language,
            }
        } else {
            Step {
                output: Some(message),
                error: false,
                language: handled.language,
            }
        };
    }
    let out = session.submit(line);
    Step {
        error: is_engine_error(&out),
        output: if out.is_empty() { None } else { Some(out) },
        language: None,
    }
}

/// Open the shared native store (ADR-0002): `EPHER_STORE_DIR` override,
/// else `~/.epher`, and load the saved session (functions, scripts,
/// history) — warning and starting fresh if the store is unreadable.
fn open_store_with_session() -> (DocStore<FsStore>, Session, Localizer) {
    let store = DocStore::new(FsStore::new(default_store_dir()));
    let session = match load_session(&store) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("warning: could not load saved data ({e}); starting fresh");
            Session::new()
        }
    };
    let preference = load_language(&store).unwrap_or(None);
    let detected: Vec<String> = sys_locale::get_locales().collect();
    let localizer = Localizer::resolve(preference.as_deref(), &detected);
    (store, session, localizer)
}

/// Handle a `graph …`/`graph3d …`/`solar3d …` line against a run's plot state
/// (ADR-0020): returns the outcome to print, or `None` when the line is
/// not a graph line. Every CLI entry point (REPL, piped, one-shot) shares
/// this so the grammar behaves identically everywhere. Diagnostics carry
/// the same `error: ` voice as engine errors.
fn graph_line(
    line: &str,
    plots: &mut epher_shell::plots::Plots,
    env: &epher_core::Env,
    localizer: &Localizer,
) -> Option<Step> {
    if let Some(source) = line.strip_prefix("graph ") {
        let out = plots.submit_graph(source, env, localizer);
        return Some(step_from(out));
    }
    if let Some(source) = line.strip_prefix("graph3d ") {
        let out = plots.submit_surface(source, env, localizer);
        return Some(step_from(out));
    }
    if let Some(source) = line.strip_prefix("solar3d ") {
        let out = plots.submit_solar3d(source, env, localizer);
        return Some(step_from(out));
    }
    None
}

fn step_from(out: epher_shell::plots::PlotOutcome) -> Step {
    Step {
        output: Some(if out.error {
            format!("error: {}", out.message)
        } else {
            out.message
        }),
        error: out.error,
        language: None,
    }
}

/// Evaluate a single expression and print the result (no UI, no store).
/// A `graph …`/`graph3d …` statement may appear among the statements
/// (ADR-0020): curves accumulate over the statements and `graph save
/// <file>` writes the SVG document — `epher "graph sin(x); graph save
/// plot.svg"` is a complete plot in one command.
pub fn run_one_shot(expr: &str) -> Result<(), EpherError> {
    // One-shot accepts a whole script (ADR-0001 seam unification):
    // statements separated by newlines or `;`, each result printed on its
    // own line — the piped mode's output without the `=` prefix.
    // `epher "2 + 3"` prints `5`, exactly as before. Graph statements
    // split out first; the rest keep the engine's exact script semantics.
    // The command runs against the shared store (ADR-0010 amendment): it
    // sees the saved session — functions, constants, variables, `ans` —
    // and records itself in the common history, so the CLI is part of the
    // same body of saved work as the REPL, TUI, and desktop app.
    let (store, mut session, localizer) = open_store_with_session();
    let mut plots = epher_shell::plots::Plots::new();
    for piece in expr.split(['\n', ';']) {
        let piece = piece.trim();
        if piece.is_empty() {
            continue;
        }
        if let Some(out) = graph_line(piece, &mut plots, session.env(), &localizer) {
            if out.error {
                term::error(out.output.as_deref().unwrap_or("error"));
                std::process::exit(1);
            }
            print_step(&out);
            continue;
        }
        let script = epher_core::parse_script(piece)?;
        for value in epher_core::run_all(&script, session.env_mut())? {
            println!("{value}");
        }
    }
    // The command joins the shared history and its bindings (`ans` and
    // any assignments) join the shared session snapshot — best effort,
    // exactly like the REPL's per-line saves.
    session.record(expr.trim());
    let _ = save_history(&store, session.history());
    let _ = save_session(&store, session.bindings());
    Ok(())
}

/// Interactive REPL: scripts run against a persistent environment; history,
/// saved functions, and the language preference survive restarts via the
/// shared store. The UI language is the store preference if set, else the
/// detected device locales (ADR-0008). Diagnostics print to stderr
/// (red on a terminal) and the conversation continues; quitting exits 0.
pub fn run_repl() -> Result<(), EpherError> {
    let (store, mut session, mut localizer) = open_store_with_session();
    // The run's plot state (ADR-0020): `graph` lines accumulate here and
    // `graph save <file>` writes the SVG the desktop and PWA produce.
    let mut plots = epher_shell::plots::Plots::new();
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        print!("{} ", localizer.lookup("prompt"));
        io::stdout()
            .flush()
            .map_err(|e| EpherError::Io(e.to_string()))?;
        let Some(line) = lines.next() else { break }; // EOF
        let line = line
            .map_err(|e| EpherError::Io(e.to_string()))?
            .trim()
            .to_string();
        if line.is_empty() {
            continue;
        }
        if line == "quit" || line == "exit" {
            break;
        }
        let out = match graph_line(&line, &mut plots, session.env(), &localizer) {
            Some(out) => {
                // the command joins the history like every other line
                session.record(&line);
                out
            }
            None => step(&mut session, &store, &localizer, &line),
        };
        print_step(&out);
        if let Some(code) = out.language {
            localizer = Localizer::resolve(Some(&code), &[]);
        }
        // best-effort persistence of history and the session snapshot
        // (atomic, last-write-wins)
        let _ = save_history(&store, session.history());
        let _ = save_session(&store, session.bindings());
    }
    Ok(())
}

/// Print one step's output on the right stream (ADR-0013): results to
/// stdout, diagnostics to stderr in red.
fn print_step(out: &Step) {
    match (&out.output, out.error) {
        (Some(text), true) => term::error(text),
        (Some(text), false) => println!("{text}"),
        _ => {}
    }
}

/// The message shown when `epher -` is run with a terminal on stdin
/// instead of a piped script (clig.dev: don't hang waiting for input the
/// user never promised).
pub const STDIN_IS_TERMINAL_MSG: &str =
    "`epher -` reads a script from standard input, but standard input is a terminal.";

/// Piped script mode (`epher -`) as an entry point: refuse to hang on an
/// interactive terminal, evaluate stdin line by line printing each result,
/// and exit 0 when every line succeeded, 1 when any line failed (per-line
/// errors have already printed). Sessions load from (and `save` commands
/// write to) the shared store; interactive history is not written —
/// scripts are not interactive pasts.
pub fn run_stdin_and_exit() -> ! {
    use std::io::IsTerminal;
    if io::stdin().is_terminal() {
        term::error(STDIN_IS_TERMINAL_MSG);
        eprintln!("Pipe a script in, or run `epher repl` for an interactive session.");
        std::process::exit(2);
    }
    match run_stdin() {
        Ok(false) => std::process::exit(0),
        Ok(true) => std::process::exit(1),
        Err(e) => {
            term::error(&format!("error: {e}"));
            std::process::exit(1);
        }
    }
}

/// Piped script mode (`epher -`) reading real stdin: `Ok(true)` when any
/// line failed (the caller decides the exit code).
pub fn run_stdin() -> Result<bool, EpherError> {
    let stdin = io::stdin();
    run_stdin_from(stdin.lock())
}

/// The testable core of [`run_stdin`]: any line-oriented reader. Lines
/// share one session — a function defined on an early line is available
/// later — and errors print (to stderr) while evaluation continues, like
/// the REPL. Returns whether any line failed.
pub fn run_stdin_from<R: BufRead>(input: R) -> Result<bool, EpherError> {
    let (store, mut session, localizer) = open_store_with_session();
    // Piped scripts plot too (ADR-0020): the plot state spans the script's
    // lines — `printf "graph sin(x)\ngraph save plot.svg\n" | epher -`.
    let mut plots = epher_shell::plots::Plots::new();
    let mut failed = false;
    for line in input.lines() {
        let line = line
            .map_err(|e| EpherError::Io(e.to_string()))?
            .trim()
            .to_string();
        if line.is_empty() {
            continue;
        }
        let out = match graph_line(&line, &mut plots, session.env(), &localizer) {
            Some(out) => out,
            None => step(&mut session, &store, &localizer, &line),
        };
        failed |= out.error;
        print_step(&out);
    }
    Ok(failed)
}
