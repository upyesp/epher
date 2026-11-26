//! epher-cli, native command-line frontend (ADR-0001).
//!
//! The library hosts every mode so the unified `epher` binary
//! (crates/tauri-app) can offer the same behavior without duplicating a
//! line:
//!
//! - [`run_one_shot`], evaluate a single expression, print the result;
//! - [`run_repl`], interactive REPL (prompt, persistent store);
//! - [`run_stdin_and_exit`], piped script mode (`epher -`): evaluate
//!   stdin line by line, no prompts, history untouched.
//!
//! All three share [`step`], the one-line-at-a-time seam that classifies a
//! line (shell command vs. language statement), runs it against the shared
//! session/store, and reports the printed output plus any language switch.
//!
//! The command-line conventions live here too (ADR-0013): [`dispatch`]
//! defines the argument surface, [`help`] the manual/help behavior, and
//! [`term`] the stdout/stderr/color policy, results on stdout, errors on
//! stderr with exit codes 0/1/2.

pub mod dispatch;
pub mod help;
pub mod term;

use std::io::{self, BufRead, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rustyline::error::ReadlineError;

use epher_core::{EpherError, Session};
use epher_i18n::Localizer;
use epher_shell::{classify, plain, run_command};
use epher_store::persist::{
    default_store_dir, load_language, load_session, save_history, save_session,
};
use epher_store::{DocStore, FsStore};

/// The outcome of processing one line: what to print (if anything),
/// whether it is a diagnostic (stderr, not data, ADR-0013), and a
/// language switch requested by a `lang` command (if any).
pub struct Step {
    pub output: Option<String>,
    pub error: bool,
    pub language: Option<String>,
}

/// Is this engine output an error line? The engine renders errors as
/// `error: …`: the only producer of that prefix on the result path.
fn is_engine_error(output: &str) -> bool {
    output.starts_with("error: ")
}

/// Process one input line against the session and store. Shell commands
/// (`save`, `lang`, …) run through epher-shell; anything else is a language
/// statement evaluated against the session. Errors come back as
/// `error: …` output marked `Step::error`: the session stays usable,
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
    let out = session.submit_entry(line);
    Step {
        error: is_engine_error(&out),
        output: if out.is_empty() { None } else { Some(out) },
        language: None,
    }
}

/// Open the shared native store (ADR-0002): `EPHER_STORE_DIR` override,
/// else `~/.epher`, and load the saved session (functions, scripts,
/// history), warning and starting fresh if the store is unreadable.
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
    if line.trim() == "solar3d" {
        // Bare `solar3d` plots the solar system as of now, matching
        // the script runner and the hover hint.
        let out = plots.submit_solar3d("now()", env, localizer);
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
/// <file>` writes the SVG document, `epher "graph sin(x); graph save
/// plot.svg"` is a complete plot in one command.
pub fn run_one_shot(expr: &str) -> Result<(), EpherError> {
    // One-shot accepts a whole script (ADR-0001 seam unification):
    // statements separated by newlines or `;`, each result printed on its
    // own line, the piped mode's output without the `=` prefix.
    // `epher "2 + 3"` prints `5`, exactly as before. Graph statements
    // split out first; the rest keep the engine's exact script semantics.
    // The command runs against the shared store (ADR-0010 amendment): it
    // sees the saved session, functions, constants, variables, `ans`,
    // and records itself in the common history, so the CLI is part of the
    // same body of saved work as the REPL, TUI, and desktop app.
    let (store, mut session, localizer) = open_store_with_session();
    let mut plots = epher_shell::plots::Plots::new();
    // One invocation is one script (ADR-0012 amendment): its definitions
    // are the run's own, so a saved session's constants cannot break a
    // later example. Statements split where the tokenizer sees
    // separators: `;` inside a string literal or a comment is text, not
    // a separator (a pasted print("a; b") used to fragment and error).
    session.begin_script();
    // The entry records what came back (ADR-0097): the last value the
    // run produced, spelled the session's way (`= 5`, `display()`),
    // even though the piped stdout keeps the bare form (ADR-0001). A
    // multi-statement invocation is one command, so the entry carries
    // the last answer, the value `ans` holds after the run.
    let mut last_output = String::new();
    let outcome = (|| -> Result<(), EpherError> {
        for piece in epher_shell::split_statements(expr) {
            if let Some(out) = graph_line(piece, &mut plots, session.env(), &localizer) {
                if out.error {
                    // A graph failure records too, exactly like an
                    // engine failure; its message already carries the
                    // `error: ` voice (step_from).
                    let line = out.output.unwrap_or_else(|| "error".to_string());
                    session.record_with_output(expr.trim(), &line);
                    let _ = save_history(&store, session.history());
                    let _ = save_session(&store, session.bindings());
                    term::error(&line);
                    std::process::exit(1);
                }
                print_step(&out);
                continue;
            }
            let script = epher_core::parse_script(piece)?;
            for value in epher_core::run_all(&script, session.env_mut())? {
                println!(
                    "{}",
                    epher_core::format_value(&value, &epher_core::DisplayPrefs::default())
                );
                last_output =
                    format!("= {}", epher_core::format_value(&value, &session.display()));
            }
        }
        Ok(())
    })();
    session.end_script();
    // The command joins the shared history with what came back, a
    // failure included (`error: ...`, the REPL's spelling), and its
    // bindings (`ans` and any assignments) join the shared session
    // snapshot, best effort, exactly like the REPL's per-line saves.
    let spoken = match &outcome {
        Ok(()) => last_output.clone(),
        Err(e) => format!("error: {e}"),
    };
    session.record_with_output(expr.trim(), &spoken);
    let _ = save_history(&store, session.history());
    let _ = save_session(&store, session.bindings());
    outcome
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
    if io::stdin().is_terminal() {
        run_repl_interactive(&store, &mut session, &mut localizer, &mut plots)
    } else {
        run_repl_piped(&store, &mut session, &mut localizer, &mut plots)
    }
}

/// One REPL line, shared by both readers: trims, treats `quit`/`exit`
/// as the end of the session, dispatches `load`, `graph`, tables, and
/// plain statements. Returns `true` when the session ends. The
/// best-effort per-line persistence (history + session snapshot) lives
/// here, so both readers save identically.
fn repl_line(
    line: &str,
    store: &DocStore<FsStore>,
    session: &mut Session,
    localizer: &mut Localizer,
    plots: &mut epher_shell::plots::Plots,
) -> Result<bool, EpherError> {
    let line = line.trim();
    if line.is_empty() {
        return Ok(false);
    }
    if line == "quit" || line == "exit" {
        return Ok(true);
    }
    // `load <file-or-name>` (ADR-0040): run a script file, or a
    // script saved with `save script name`, as a whole program,
    // statements split where the tokenizer sees separators, so a
    // block comment may span lines, exactly like running the file
    // from a terminal. The `load` line itself joins the history;
    // the loaded statements print as they run.
    if let Some(arg) = line.strip_prefix("load ") {
        let arg = arg.trim();
        if arg.is_empty() {
            term::error("usage: load <file-or-name>");
            return Ok(false);
        }
        match load_script_text(store, arg) {
            Ok(text) => {
                session.record(line);
                let failed = eval_whole_text(&text, session, store, localizer, plots)?;
                if failed {
                    term::error(&localizer.lookup("load-failed"));
                }
            }
            Err(message) => term::error(&message),
        }
        return Ok(false);
    }
    let out = match graph_line(line, plots, session.env(), localizer) {
        Some(out) => {
            // the command joins the history like every other line
            session.record(line);
            out
        }
        None => {
            // A table is a computation: the line joins the history
            // like every other line (the graph precedent). The other
            // shell commands persist their effect and leave no
            // entry, and a `load`ed script's lines record nothing,
            // the `load` line itself recorded (ADR-0040), so the
            // recording stays here in the REPL loop, out of `step`.
            if matches!(classify(line), Some(epher_shell::Command::Table { .. })) {
                session.record(line);
            }
            step(session, store, localizer, line)
        }
    };
    print_step(&out);
    if let Some(code) = out.language {
        *localizer = Localizer::resolve(Some(&code), &[]);
    }
    // best-effort persistence of history and the session snapshot
    // (atomic, last-write-wins)
    let _ = save_history(store, session.history());
    let _ = save_session(store, session.bindings());
    Ok(false)
}

/// The lines the REPL's editor recalls (ADR-0086, amended 2026-11-26):
/// the persisted history as expressions, oldest first, exactly as the
/// TUI, the desktop app, and the PWA recall them. One door for every
/// surface ([`Session::history_expressions`]), so the editor is never
/// seeded with a display transcript: `2 + 2  = 4` on the Up arrow is a
/// parse error waiting to be recorded back into the store.
fn repl_history_lines(session: &Session) -> Vec<String> {
    session.history_expressions().collect()
}

/// The Ctrl-L key handler (ADR-0096): raises the flag the REPL loop
/// acts on, and ends the `readline` call at once with `AcceptLine` so
/// the clear happens on the key press, not on some later Enter. The
/// loop discards the buffer `AcceptLine` hands back: Ctrl-L clears the
/// history, it never submits whatever was half-typed. rustyline only
/// accepts `ConditionalEventHandler` as a custom binding, so the flag
/// lives on a struct rather than a closure.
struct ClearHistoryKey {
    requested: Arc<AtomicBool>,
}

impl rustyline::ConditionalEventHandler for ClearHistoryKey {
    fn handle(
        &self,
        _evt: &rustyline::Event,
        _n: rustyline::RepeatCount,
        _positive: bool,
        _ctx: &rustyline::EventContext<'_>,
    ) -> Option<rustyline::Cmd> {
        self.requested.store(true, Ordering::SeqCst);
        Some(rustyline::Cmd::AcceptLine)
    }
}

/// The terminal reader (ADR-0086): rustyline owns line editing and the
/// in-session history. Arrow keys move in the line, up and down walk
/// the history; the shared store seeds it at startup and receives every
/// line, so recall reaches previous sessions without a history file
/// (ADR-0027 amendment). Ctrl-C abandons the current line for a fresh
/// prompt; Ctrl-D (EOF) ends the session; Ctrl-L clears the history
/// (ADR-0096), the TUI's gesture, through the same `clear history`
/// path.
fn run_repl_interactive(
    store: &DocStore<FsStore>,
    session: &mut Session,
    localizer: &mut Localizer,
    plots: &mut epher_shell::plots::Plots,
) -> Result<(), EpherError> {
    let mut rl =
        rustyline::DefaultEditor::new().map_err(|e| EpherError::Io(e.to_string()))?;
    // Previous sessions are one up-press away: seed the editor from the
    // persisted history (in-memory only; the store is the archive), as
    // expressions, the same text the other frontends load into their
    // entry.
    for entry in repl_history_lines(session) {
        let _ = rl.add_history_entry(entry);
    }
    // Ctrl-L clears the history (ADR-0096), on the key press. rustyline
    // runs the key handler inside its editor, where the session is not
    // reachable: the handler raises a flag and ends the read, and the
    // loop acts on the flag right here, where the session and the store
    // are in reach.
    let clear_requested = Arc::new(AtomicBool::new(false));
    // No blanket `From<Box<T>>` exists for a conditional handler: build
    // the enum variant directly.
    rl.bind_sequence(
        rustyline::KeyEvent::ctrl('l'),
        rustyline::EventHandler::Conditional(Box::new(ClearHistoryKey {
            requested: Arc::clone(&clear_requested),
        })),
    );
    loop {
        let prompt = format!("{} ", localizer.lookup("prompt"));
        let read = match rl.readline(&prompt) {
            Ok(line) => Some(line),
            Err(ReadlineError::Interrupted) => None, // Ctrl-C: fresh prompt
            Err(ReadlineError::Eof) => break,        // Ctrl-D: quit
            Err(e) => return Err(EpherError::Io(e.to_string())),
        };
        // The flag means Ctrl-L ended the read: clear both lists (the
        // editor's in place, so the arrows stop recalling at once; the
        // session's, persisted to the store) and drop the half-typed
        // buffer. The line is never submitted.
        if clear_requested.swap(false, Ordering::SeqCst) {
            let _ = rl.clear_history();
            session.clear_history();
            let _ = save_history(store, session.history());
            // The confirmation goes to stdout like `reset`'s does.
            println!("{}", localizer.lookup("history-cleared"));
            continue;
        }
        let Some(line) = read else {
            continue;
        };
        if line.trim().is_empty() {
            continue;
        }
        // The typed line joins the editor's list as the user
        // typed it. A line recalled from the list is already an
        // expression, so re-submitting it never compounds a
        // second answer suffix; the editor also skips a line
        // identical to the previous one (rustyline's default),
        // which keeps a re-run from doubling the walk's length.
        let _ = rl.add_history_entry(line.trim());
        if repl_line(&line, store, session, localizer, plots)? {
            break;
        }
    }
    Ok(())
}

/// The piped reader, the old loop exactly: `printf '...' | epher repl`,
/// the CLI tests, and every scripted invocation read plain lines with no
/// raw-mode editing, so behavior off a terminal is byte-identical.
/// Ctrl-C on a pipe still terminates the process (SIGINT), as before.
fn run_repl_piped(
    store: &DocStore<FsStore>,
    session: &mut Session,
    localizer: &mut Localizer,
    plots: &mut epher_shell::plots::Plots,
) -> Result<(), EpherError> {
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    loop {
        // The prompt stays part of piped output: the CLI tests and every
        // scripted REPL transcript assert on it byte-for-byte.
        print!("{} ", localizer.lookup("prompt"));
        io::stdout()
            .flush()
            .map_err(|e| EpherError::Io(e.to_string()))?;
        let Some(line) = lines.next() else { break }; // EOF
        let line = line.map_err(|e| EpherError::Io(e.to_string()))?;
        if repl_line(&line, store, session, localizer, plots)? {
            break;
        }
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
/// write to) the shared store; interactive history is not written,
/// scripts are not interactive pasts.
pub fn run_stdin_and_exit() -> ! {
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
/// share one session, a function defined on an early line is available
/// later, and errors print (to stderr) while evaluation continues, like
/// the REPL. Returns whether any line failed.
pub fn run_stdin_from<R: BufRead>(input: R) -> Result<bool, EpherError> {
    let (store, mut session, localizer) = open_store_with_session();
    // Piped scripts plot too (ADR-0020): the plot state spans the script's
    // lines, `printf "graph sin(x)\ngraph save plot.svg\n" | epher -`.
    let mut plots = epher_shell::plots::Plots::new();
    eval_lines(input.lines(), &mut session, &store, &localizer, &mut plots)
}

/// Evaluate a script file as a one-shot run (ADR-0040): the same
/// statement semantics as a piped script (`epher -`), each result
/// prints, definitions and `save` commands go to the shared store,
/// interactive history is not written, reading the statements from
/// `path` instead of stdin. The file is a whole program: statements
/// split where the tokenizer sees separators, so a block comment may
/// span lines the way it does in the one-shot and web-paste paths.
/// `Ok(true)` when any statement failed.
pub fn run_script_file(path: &std::path::Path) -> Result<bool, EpherError> {
    let (store, mut session, localizer) = open_store_with_session();
    let mut plots = epher_shell::plots::Plots::new();
    let text = std::fs::read_to_string(path).map_err(|e| EpherError::Io(e.to_string()))?;
    eval_whole_text(&text, &mut session, &store, &localizer, &mut plots)
}

/// Evaluate a whole program (a script file, a `load`ed script) as one
/// piece of text (ADR-0040): statements split where the tokenizer sees
/// separators, `;` inside a string literal or comment is text, and the
/// newlines inside a block comment are comment text, so block comments
/// can span lines in files exactly as in the one-shot and web-paste
/// paths. Line-oriented input (REPL typing, piped mode) keeps the line
/// model: there a block comment closes on its own line. Every statement
/// evaluates against one live session, a function defined early is
/// available later, `graph …` statements plot into the run's plot
/// state, and errors print while evaluation continues. Returns whether
/// any statement failed.
fn eval_whole_text(
    text: &str,
    session: &mut Session,
    store: &DocStore<FsStore>,
    localizer: &Localizer,
    plots: &mut epher_shell::plots::Plots,
) -> Result<bool, EpherError> {
    let mut failed = false;
    // One file is one script (ADR-0012 amendment): its definitions are
    // the run's own, so re-running a file never collides with what an
    // earlier run (or the interactive session) left behind.
    session.begin_script();
    for piece in epher_shell::split_statements(text) {
        let piece = piece.trim();
        if piece.is_empty() {
            continue;
        }
        let out = match graph_line(piece, plots, session.env(), localizer) {
            Some(out) => out,
            None => step(session, store, localizer, piece),
        };
        failed |= out.error;
        print_step(&out);
    }
    session.end_script();
    Ok(failed)
}

/// The shared line loop of piped mode (ADR-0040): every line evaluates
/// against one live session, a function defined on an early line is
/// available later, `graph …` lines plot into the run's plot state, and
/// errors print while evaluation continues. Returns whether any line
/// failed.
fn eval_lines(
    lines: impl Iterator<Item = io::Result<String>>,
    session: &mut Session,
    store: &DocStore<FsStore>,
    localizer: &Localizer,
    plots: &mut epher_shell::plots::Plots,
) -> Result<bool, EpherError> {
    let mut failed = false;
    // One piped input is one script (ADR-0012 amendment), the same
    // scope a script file or a pasted submission gets.
    session.begin_script();
    for line in lines {
        let line = line
            .map_err(|e| EpherError::Io(e.to_string()))?
            .trim()
            .to_string();
        if line.is_empty() {
            continue;
        }
        let out = match graph_line(&line, plots, session.env(), localizer) {
            Some(out) => out,
            None => step(session, store, localizer, &line),
        };
        failed |= out.error;
        print_step(&out);
    }
    session.end_script();
    Ok(failed)
}

/// Resolve a `load` argument (ADR-0040): a path to an existing script
/// file wins; otherwise the name names a script saved with `save script
/// name` in the shared store. `Err` carries the user-facing message.
fn load_script_text(store: &DocStore<FsStore>, arg: &str) -> Result<String, String> {
    if let Ok(text) = std::fs::read_to_string(arg) {
        return Ok(text);
    }
    match store.get_script(arg) {
        Ok(Some(doc)) => Ok(doc.source),
        _ => Err(format!("error: no such file or saved script: {arg}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The REPL's recall is rustyline's, which a pipe cannot drive, so
    // the seed policy is tested here as a pure function: this is the
    // case the piped `repl_persists_*` test could never see (ADR-0086
    // amendment, 2026-11-26). The editor must recall expressions, the
    // same text the TUI and the desktop app load into their entry.
    #[test]
    fn repl_recall_seeds_the_editor_with_expressions_not_the_transcript() {
        let mut session = Session::default();
        session.submit("6+3");
        session.record("graph x ^ 2");
        session.submit("1/0");
        session.submit("x = 10; x + 5");
        assert_eq!(
            session.history(),
            [
                "6+3  = 9",
                "graph x ^ 2",
                "1/0  error: division by zero",
                "x = 10; x + 5  = 15",
            ],
            "the transcript keeps the suffixes for display"
        );
        assert_eq!(
            repl_history_lines(&session),
            ["6+3", "graph x ^ 2", "1/0", "x = 10; x + 5"],
            "every recalled line re-runs as recorded; no answer suffix"
        );
    }

    // The bug this replaces: a raw seed put `6+3  = 9` on the Up arrow,
    // Enter made it a parse error, and `repl_line` saved that error line
    // back into the shared history, where every other surface then saw
    // it. The seed must never carry the suffix forward.
    #[test]
    fn a_recalled_line_resubmitted_never_compounds_the_suffix() {
        let mut session = Session::default();
        session.submit("6+3");
        let recalled = repl_history_lines(&session).pop().unwrap();
        assert_eq!(recalled, "6+3");
        // What Enter does with the recalled line: evaluates cleanly, and
        // the transcript never gains an error line. Under ADR-0096 the
        // identical re-run collapses to the one entry it already had,
        // which is the better outcome this test was really after.
        assert_eq!(session.submit(&recalled), "= 9");
        assert_eq!(session.history(), ["6+3  = 9"]);
    }
}
