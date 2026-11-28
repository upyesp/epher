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

use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::io::{self, BufRead, IsTerminal, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use rustyline::error::ReadlineError;
use rustyline::history::{DefaultHistory, History, SearchDirection, SearchResult};

use epher_core::{EpherError, HistoryEntry, Session};
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
    open_store_with_session_at(&default_store_dir())
}

/// The same door rooted at an explicit directory, for the REPL (which
/// keeps reading the store while it runs) and the tests.
fn open_store_with_session_at(
    dir: &std::path::Path,
) -> (DocStore<FsStore>, Session, Localizer) {
    let store = DocStore::new(FsStore::new(dir));
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
///
/// Publish/subscribe (ADR-0010 amendment, 2026-11-28): the REPL is a
/// long-lived frontend like the TUI and the desktop app, not a transient
/// one. It subscribes by content rather than through a watcher thread:
/// each reader compares the store's history and session snapshot with
/// its own at every point of use (every line boundary, and every prompt
/// and arrow press in the interactive reader), so this REPL's own saves
/// never look like a foreign write. The 2026-11-27 watcher flag could
/// not tell them apart (a file event carries no origin): it ended the
/// read on every Up press that followed the REPL's own line, and on its
/// own startup, and a `load` that had not saved yet was lost to the
/// reload that followed.
pub fn run_repl() -> Result<(), EpherError> {
    let store_dir = default_store_dir();
    let (store, mut session, mut localizer) = open_store_with_session_at(&store_dir);
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
                // Publish at once (ADR-0010 amendment, 2026-11-28):
                // `load` is a submitted line like any other, so what it
                // changed reaches the store before the prompt returns,
                // like the TUI's publish on every submit. The run's
                // outcome propagates after the save, so a failed script
                // keeps the effects that did run.
                let ran = eval_whole_text(&text, session, store, localizer, plots);
                publish_line(store, session);
                let failed = ran?;
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
    publish_line(store, session);
    Ok(false)
}

/// Best-effort persistence of history and the session snapshot after a
/// line (atomic, last-write-wins): the publish half of ADR-0010.
fn publish_line(store: &DocStore<FsStore>, session: &Session) {
    let _ = save_history(store, session.history());
    let _ = save_session(store, session.bindings());
}

/// How the shared store stands to this session (ADR-0010 amendment,
/// 2026-11-28): the REPL tells a foreign write from its own saves
/// without a watcher by comparing the store's history and session
/// snapshot with the session's own, by content, at each line boundary.
/// This REPL's own saves leave the store equal to the session, so they
/// never count; a store with no session snapshot yet has nothing to
/// differ from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StoreComparison {
    /// The store holds exactly what the session does.
    Same,
    /// Another frontend wrote, or the session holds work the store
    /// lacks.
    Differs,
    /// The store cannot be read.
    Unreadable,
}

fn compare_with_store(store: &DocStore<FsStore>, session: &Session) -> StoreComparison {
    let (Ok(history), Ok(bindings)) = (
        epher_store::persist::history(store),
        epher_store::persist::session_bindings(store),
    ) else {
        return StoreComparison::Unreadable;
    };
    if history != session.history() || bindings.is_some_and(|saved| saved != *session.bindings()) {
        StoreComparison::Differs
    } else {
        StoreComparison::Same
    }
}

/// The store's session becomes the live session (ADR-0010 amendment,
/// 2026-11-27): history and bindings come from the store, then the
/// definitions this session created but has not `save`d yet replay over
/// it (the TUI's reload rule), so a foreign write (or our own echo)
/// cannot erase live work; the store's bindings win. Returns whether
/// the reload happened; a store that cannot be read leaves the session
/// as it was. Reloads never write, so no loop is possible.
fn reload_session_from_store(store: &DocStore<FsStore>, session: &mut Session) -> bool {
    let Ok(mut fresh) = load_session(store) else {
        return false;
    };
    for source in session.def_sources().values() {
        fresh.submit_quiet(source);
    }
    for source in session.const_sources().values() {
        fresh.submit_quiet(source);
    }
    *session = fresh;
    true
}

/// Reload the session and rebuild the editor's recall list from it
/// (ADR-0010 amendment, 2026-11-28): called at a line boundary when the
/// store differs from the session, so the next per-line save carries the
/// foreign write instead of erasing it, and the arrows walk the fresh
/// list. The list is rebuilt through the one recall door
/// ([`recall_lines`]), so recall stays expressions only.
fn refresh_repl_view(
    store: &DocStore<FsStore>,
    session: &mut Session,
    rl: &mut rustyline::Editor<(), StoreHistory>,
) -> bool {
    if !reload_session_from_store(store, session) {
        return false;
    }
    rl.history_mut().adopt(session.history());
    true
}

/// The editor's history is the store's (ADR-0010 amendment, 2026-11-28).
/// rustyline asks the list its length at every prompt and reads an
/// entry on every Up, Down, and Ctrl-R keystroke; each of those asks
/// first compares the shared store's history with what the list was
/// last built from or told, and rebuilds the list through the one
/// recall door when they differ. A write from another frontend is thus
/// one Up press away, with no interrupt, and this REPL's own saves
/// (which the loop reports through [`StoreHistory::note`]) never
/// rebuild anything: a line typed in this session stays in the walk,
/// as typed, until a foreign write really happens. The list holds
/// expressions only, like every other surface's recall.
///
/// rustyline keeps its own cursor and fixes it when the prompt starts,
/// at the length the list then reports: while nothing is walked that
/// index equals the length, and rustyline backs the typed line up only
/// on an Up that starts from there. A list that grew while the prompt
/// sat open would break both: the cursor would point into the list,
/// and the first Up would land on the entry before the new one with no
/// backup to come back to. So after a read rebuilds the list, the
/// length stays what it was until the first entry is read (the search
/// loop's own bounds need the real one after that), and that first
/// step is translated from the old end to the new one. The loop starts
/// each prompt with [`StoreHistory::begin_prompt`], which clears the
/// bookkeeping, since only it knows where a read starts.
struct StoreHistory {
    store: DocStore<FsStore>,
    /// rustyline's own list, behind a cell: the reads that may sync
    /// take `&self`, since rustyline walks the list through a shared
    /// reference.
    lines: RefCell<DefaultHistory>,
    /// The store history the list was built from, or that the session
    /// last wrote or read.
    seen: RefCell<Vec<HistoryEntry>>,
    /// The length last reported to rustyline: the cursor it fixed its
    /// prompt at.
    anchor: Cell<usize>,
    /// The anchor a read's rebuild left rustyline's cursor behind,
    /// until the first step is translated.
    grown_from: Cell<Option<usize>>,
    /// Whether an entry was read since the key handling began: the
    /// length reported stays the old one until then, so rustyline's
    /// "not walking" check passes and it backs the typed line up; the
    /// incremental search loops on the real length after its first read.
    entry_read: Cell<bool>,
}

impl StoreHistory {
    /// A list over `store`, synced once: a fresh REPL starts where the
    /// store is.
    fn new(store: DocStore<FsStore>) -> rustyline::Result<Self> {
        let history = Self {
            store,
            lines: RefCell::new(DefaultHistory::new()),
            seen: RefCell::new(Vec::new()),
            anchor: Cell::new(1),
            grown_from: Cell::new(None),
            entry_read: Cell::new(false),
        };
        history.sync();
        history.anchor.set(history.reported_len());
        Ok(history)
    }

    /// A prompt starts: take the store as it is now and forget the
    /// previous read's bookkeeping. rustyline fixes its cursor at the
    /// length reported right after this.
    fn begin_prompt(&self) {
        self.sync();
        self.grown_from.set(None);
        self.entry_read.set(false);
        self.anchor.set(self.reported_len());
    }

    /// The session just wrote or loaded exactly these records: the
    /// store holds them, so they are not a foreign write. The list is
    /// left alone (the loop added the typed line itself).
    fn note(&self, records: &[HistoryEntry]) {
        *self.seen.borrow_mut() = records.to_vec();
    }

    /// Make the list recall exactly these records, and take them as
    /// what the store holds (the loop's reload at a line boundary, or
    /// a `load` that recorded the script's lines).
    fn adopt(&self, records: &[HistoryEntry]) {
        let mut lines = self.lines.borrow_mut();
        let _ = lines.clear();
        for line in recall_lines(records) {
            let _ = lines.add_owned(line);
        }
        drop(lines);
        self.note(records);
    }

    /// Rebuild the list when the store's history is not what this
    /// session last saw. Returns whether it rebuilt. A store that
    /// cannot be read leaves the list as it is; the loop's reload is
    /// where that is reported.
    fn sync(&self) -> bool {
        let Ok(current) = epher_store::persist::history(&self.store) else {
            return false;
        };
        if *self.seen.borrow() == current {
            return false;
        }
        self.adopt(&current);
        true
    }

    /// Bring the list up to date for a read other than a length: a
    /// rebuild leaves rustyline's cursor (fixed at [`Self::anchor`])
    /// behind, and the first step after it is translated. Returns
    /// whether this is the first entry read since such a rebuild.
    fn sync_before_read(&self) -> bool {
        if self.sync() {
            self.grown_from.set(Some(self.anchor.get()));
            self.entry_read.set(false);
        }
        let first = self.grown_from.get().is_some() && !self.entry_read.replace(true);
        first
    }

    /// The length rustyline sees while its cursor predates a rebuild:
    /// the length it fixed the cursor at, until the first entry is
    /// read; then the real one.
    fn len_for_rustyline(&self) -> usize {
        match self.grown_from.get() {
            Some(before) if !self.entry_read.get() => before,
            _ => self.reported_len(),
        }
    }

    /// The length rustyline sees: never less than one. rustyline treats
    /// the index it holds as "not walking" when it equals the length,
    /// and as the oldest entry when it is zero; an empty list would make
    /// those the same position, and the first Up after the first entry
    /// ever arrived would never reach the list.
    fn reported_len(&self) -> usize {
        self.lines.borrow().len().max(1)
    }

    /// One read of rustyline's list, as owned data (the list sits
    /// behind a cell, so a borrowed entry cannot outlive the read).
    fn owned_read(
        &self,
        read: impl FnOnce(&DefaultHistory) -> rustyline::Result<Option<SearchResult<'_>>>,
    ) -> rustyline::Result<Option<SearchResult<'_>>> {
        read(&self.lines.borrow()).map(|hit| {
            hit.map(|hit| SearchResult {
                entry: Cow::Owned(hit.entry.into_owned()),
                idx: hit.idx,
                pos: hit.pos,
            })
        })
    }

    /// The index rustyline's step after a rebuild means. Before the
    /// rebuild its cursor was the old length; an Up from there is
    /// `index + 1 == anchor` and means the new end, and a reverse
    /// search from there is `start + 1 == anchor` and means the same.
    /// A step from a position the list no longer has (a `clear history`
    /// in another frontend while walking) means the new end too. Any
    /// other index is a position an append leaves where it was. `None`
    /// when there is nothing to walk into.
    fn translated(&self, index: usize, dir: SearchDirection) -> Option<usize> {
        let len = self.lines.borrow().len();
        match dir {
            SearchDirection::Reverse
                if Some(index + 1) == self.grown_from.get() || index >= len =>
            {
                len.checked_sub(1)
            }
            _ => Some(index),
        }
    }

    /// The walk took its translation (or needed none): rustyline's
    /// cursor now is a position in the list as it is, so that is the
    /// length its "not walking" check will be made against.
    fn take_translation(&self) {
        if self.grown_from.replace(None).is_some() {
            self.anchor.set(self.reported_len());
        }
    }
}

/// The lines an editor recalls for stored records (ADR-0086, amended
/// 2026-11-26): the persisted history as expressions, oldest first,
/// exactly as the TUI, the desktop app, and the PWA recall them. One
/// door for every surface ([`Session::history_expressions`], through a
/// session seeded with the records, so the seed policy governs every
/// rebuild), so the editor is never seeded with a display transcript:
/// `2 + 2  = 4` on the Up arrow is a parse error waiting to be recorded
/// back into the store.
fn recall_lines(records: &[HistoryEntry]) -> Vec<String> {
    Session::with_history(records.iter().cloned())
        .history_expressions()
        .collect()
}

impl History for StoreHistory {
    // The walk: the first step after a rebuild speaks the old index
    // space, every later one the new.
    fn get(
        &self,
        index: usize,
        dir: SearchDirection,
    ) -> rustyline::Result<Option<SearchResult<'_>>> {
        self.sync_before_read();
        let translated = self.translated(index, dir);
        self.take_translation();
        match translated {
            Some(index) => self.owned_read(|lines| lines.get(index, dir)),
            None => Ok(None),
        }
    }

    fn add(&mut self, line: &str) -> rustyline::Result<bool> {
        self.lines.borrow_mut().add(line)
    }

    fn add_owned(&mut self, line: String) -> rustyline::Result<bool> {
        self.lines.borrow_mut().add_owned(line)
    }

    fn len(&self) -> usize {
        // A rebuild seen here is reported as it is: the prompt start
        // fixes rustyline's cursor from this very call, and the search
        // loop's bound needs the real length. A write landing between
        // another read and this call, within one key handling, is the
        // one case that leaves no translation for the next step.
        if self.sync() {
            self.grown_from.set(None);
            self.entry_read.set(false);
            self.anchor.set(self.reported_len());
        }
        self.len_for_rustyline()
    }

    // rustyline calls this first in every key handling, so it is where
    // "an entry was read" starts over.
    fn is_empty(&self) -> bool {
        if self.sync() {
            self.grown_from.set(Some(self.anchor.get()));
        }
        self.entry_read.set(false);
        self.lines.borrow().len() == 0
    }

    fn set_max_len(&mut self, len: usize) -> rustyline::Result<()> {
        self.lines.borrow_mut().set_max_len(len)
    }

    fn ignore_dups(&mut self, yes: bool) -> rustyline::Result<()> {
        self.lines.borrow_mut().ignore_dups(yes)
    }

    fn ignore_space(&mut self, yes: bool) {
        self.lines.borrow_mut().ignore_space(yes)
    }

    // The REPL's archive is the store, never a history file, so the
    // file persistence rustyline offers is the inner list's own and is
    // not used by the loop.
    fn save(&mut self, path: &Path) -> rustyline::Result<()> {
        self.lines.borrow_mut().save(path)
    }

    fn append(&mut self, path: &Path) -> rustyline::Result<()> {
        self.lines.borrow_mut().append(path)
    }

    fn load(&mut self, path: &Path) -> rustyline::Result<()> {
        self.lines.borrow_mut().load(path)
    }

    fn clear(&mut self) -> rustyline::Result<()> {
        self.lines.borrow_mut().clear()
    }

    // The incremental search (Ctrl-R) starts from the length less one
    // and never writes rustyline's cursor: its first start is
    // translated like a first Up, and the walk's translation stays in
    // place, so an Up after an aborted search still starts from the
    // old end.
    fn search(
        &self,
        term: &str,
        start: usize,
        dir: SearchDirection,
    ) -> rustyline::Result<Option<SearchResult<'_>>> {
        let first_read = self.sync_before_read();
        let start = if first_read {
            self.translated(start, dir)
        } else {
            Some(start)
        };
        match start {
            Some(start) => self.owned_read(|lines| lines.search(term, start, dir)),
            None => Ok(None),
        }
    }

    fn starts_with(
        &self,
        term: &str,
        start: usize,
        dir: SearchDirection,
    ) -> rustyline::Result<Option<SearchResult<'_>>> {
        self.sync_before_read();
        let translated = self.translated(start, dir);
        self.take_translation();
        match translated {
            Some(start) => self.owned_read(|lines| lines.starts_with(term, start, dir)),
            None => Ok(None),
        }
    }
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
/// in-session history; the shared store seeds it at startup and
/// receives every line, so recall reaches previous sessions without a
/// history file (ADR-0027 amendment). The editor's list is the store's
/// (ADR-0010 amendment, 2026-11-28, [`StoreHistory`]): a line another
/// frontend writes while this prompt sits open is one Up press away,
/// with no interrupt. The session itself (bindings, definitions)
/// reloads at the line boundary when the store differs from it.
/// Ctrl-C abandons the current line for a fresh prompt; Ctrl-D (EOF)
/// ends the session; Ctrl-L clears the history (ADR-0096), the TUI's
/// gesture, through the same `clear history` path.
fn run_repl_interactive(
    store: &DocStore<FsStore>,
    session: &mut Session,
    localizer: &mut Localizer,
    plots: &mut epher_shell::plots::Plots,
) -> Result<(), EpherError> {
    // Previous sessions are one up-press away: the list starts where the
    // store is, as expressions, the same text the other frontends load
    // into their entry.
    let history = StoreHistory::new(DocStore::new(store.storage().clone()))
        .map_err(|e| EpherError::Io(e.to_string()))?;
    let mut rl = rustyline::Editor::with_history(rustyline::Config::default(), history)
        .map_err(|e| EpherError::Io(e.to_string()))?;
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
    // The unreadable-store warning prints once per failure episode, not
    // once per line.
    let mut reload_warned = false;
    loop {
        let prompt = format!("{} ", localizer.lookup("prompt"));
        rl.history_mut().begin_prompt();
        let read = match rl.readline(&prompt) {
            Ok(line) => Some(line),
            Err(ReadlineError::Interrupted) => None, // Ctrl-C: fresh prompt
            Err(ReadlineError::Eof) => break,        // Ctrl-D: quit
            Err(e) => return Err(EpherError::Io(e.to_string())),
        };
        // Subscribe (ADR-0010 amendment, 2026-11-28): the store is
        // compared with the session by content, right before anything
        // acts on the returned line. They differ when another frontend
        // wrote while this prompt sat open: reload then, replaying the
        // definitions this session created but has not `save`d (the
        // TUI's reload rule), so the arrows reach the foreign entries
        // and the per-line save carries them instead of erasing them.
        // This REPL's own saves leave the store equal to the session,
        // so they never reload. Reloads never write, so no loop is
        // possible; the Ctrl-L clear below still runs after the reload,
        // so a racing foreign write cannot resurrect what the clear
        // removes.
        match compare_with_store(store, session) {
            StoreComparison::Same => reload_warned = false,
            StoreComparison::Differs | StoreComparison::Unreadable => {
                if refresh_repl_view(store, session, &mut rl) {
                    reload_warned = false;
                } else if !reload_warned {
                    // The store could not be read. A store the REPL cannot
                    // read is one no frontend can read, so the session in
                    // memory stays authoritative and the per-line saves
                    // keep going (overwriting it is repair, not loss; the
                    // desktop's save gate guards the real hole, ADR-0010
                    // amendment 2026-11-27). Say the reload failed rather
                    // than degrade silently.
                    reload_warned = true;
                    eprintln!("warning: could not reload the shared store; keeping this session in memory");
                }
            }
        }
        // The flag means Ctrl-L ended the read: clear both lists (the
        // editor's in place, so the arrows stop recalling at once; the
        // session's, persisted to the store) and drop the half-typed
        // buffer. The line is never submitted.
        if clear_requested.swap(false, Ordering::SeqCst) {
            let _ = rl.clear_history();
            session.clear_history();
            let _ = save_history(store, session.history());
            rl.history_mut().note(session.history());
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
        let recorded = session.history().to_vec();
        let quit = repl_line(&line, store, session, localizer, plots)?;
        // What the session just published is not a foreign write. The
        // list already holds the typed line, so tell it and spare the
        // next prompt a rebuild; when the line recorded more than
        // itself (a `load` records the script's lines too), the list
        // takes the session's records instead.
        let history = rl.history_mut();
        let after = session.history();
        if after != recorded.as_slice() && after.len() != recorded.len() + 1 {
            history.adopt(after);
        } else {
            history.note(after);
        }
        if quit {
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
        // Subscribe like the interactive reader (ADR-0010 amendment,
        // 2026-11-28): a scripted session can run minutes, and its
        // per-line saves must carry what another frontend wrote in the
        // meantime, not overwrite it. The store is compared with the
        // session by content, so this session's own lines never cause
        // a reload. Nothing prints here, so piped transcripts stay
        // byte-identical.
        if compare_with_store(store, session) == StoreComparison::Differs {
            reload_session_from_store(store, session);
        }
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
    use rustyline::history::History;
    use rustyline::history::SearchDirection;

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
            recall_lines(session.history()),
            ["6+3", "graph x ^ 2", "1/0", "x = 10; x + 5"],
            "every recalled line re-runs as recorded; no answer suffix"
        );
    }

    // The store is the live view (ADR-0010 amendment, 2026-11-28):
    // between prompts a foreign write (TUI, desktop, one-shot) reloads
    // into the session and the editor's recall list at the next line
    // boundary, so the arrows reach it and the next per-line save
    // carries it instead of erasing it. rustyline's list is only
    // reachable in-process, so this is the seam for the reload the pty
    // harness proves end to end.
    #[test]
    fn repl_refresh_rebuilds_the_recall_list_from_the_store() {
        let dir = tempfile::tempdir().unwrap();
        let store = DocStore::new(FsStore::new(dir.path()));
        let mut session = Session::default();
        session.submit("1+1");
        publish_line(&store, &session);
        let history = StoreHistory::new(DocStore::new(FsStore::new(dir.path()))).unwrap();
        let mut rl =
            rustyline::Editor::with_history(rustyline::Config::default(), history).unwrap();
        assert_eq!(walk(rl.history()), ["1+1"], "startup: the store's lines");
        // Another frontend writes while the prompt sits open.
        save_history(
            &store,
            &[HistoryEntry {
                expr: "9*9".into(),
                out: Some("= 81".into()),
            }],
        )
        .unwrap();
        assert_eq!(
            compare_with_store(&store, &session),
            StoreComparison::Differs
        );
        assert!(refresh_repl_view(&store, &mut session, &mut rl));
        assert_eq!(
            walk(rl.history()),
            ["9*9"],
            "a foreign entry must be one up-press away"
        );
        // The session converged: the REPL's next save keeps the foreign
        // entry (the stale view used to overwrite the whole list).
        publish_line(&store, &session);
        let stored = epher_store::persist::history(&store).unwrap();
        assert!(
            stored.iter().any(|e| e.expr == "9*9"),
            "the converged save must not clobber: {stored:?}"
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
        let recalled = recall_lines(session.history()).pop().unwrap();
        assert_eq!(recalled, "6+3");
        // What Enter does with the recalled line: evaluates cleanly, and
        // the transcript never gains an error line. Under ADR-0096 the
        // identical re-run collapses to the one entry it already had,
        // which is the better outcome this test was really after.
        assert_eq!(session.submit(&recalled), "= 9");
        assert_eq!(session.history(), ["6+3  = 9"]);
    }

    // A `load` publishes at once (ADR-0010 amendment, 2026-11-28): the
    // `load` line, the script's lines, and its assignments reach the
    // store before the prompt returns. They used to live only in memory
    // until the next line saved, and the reload at that boundary (any
    // foreign write, or the watcher's echo of this REPL's own startup)
    // replaced the session with the store's copy and lost them:
    // `load setup.epher` then `rate * 100` answered `unknown name: rate`.
    #[test]
    fn repl_load_publishes_its_lines_and_assignments_at_once() {
        let dir = tempfile::tempdir().unwrap();
        let store = DocStore::new(FsStore::new(dir.path()));
        let script = dir.path().join("setup.epher");
        std::fs::write(&script, "rate = 7\nrate * 2\n").unwrap();
        let mut session = Session::default();
        let mut localizer = Localizer::resolve(None, &[]);
        let mut plots = epher_shell::plots::Plots::new();
        let line = format!("load {}", script.display());
        assert!(!repl_line(&line, &store, &mut session, &mut localizer, &mut plots).unwrap());
        let stored = epher_store::persist::history(&store).unwrap();
        let exprs: Vec<&str> = stored.iter().map(|e| e.expr.as_str()).collect();
        assert_eq!(exprs, [line.as_str(), "rate = 7", "rate * 2"]);
        // Another frontend (or this REPL after a restart) finds the work.
        let mut fresh = load_session(&store).unwrap();
        assert_eq!(fresh.submit("rate * 100"), "= 700");
    }

    // The REPL tells its own writes from another frontend's by content
    // (ADR-0010 amendment, 2026-11-28): the store differs from this
    // session exactly when its history or its session snapshot are not
    // what the session holds. The watcher flag it replaces could not
    // tell them apart (a file event carries no origin) and fired on
    // every line this REPL itself saved, and on its own startup.
    #[test]
    fn the_store_differs_from_the_session_only_after_a_foreign_write() {
        let dir = tempfile::tempdir().unwrap();
        let store = DocStore::new(FsStore::new(dir.path()));
        let mut session = Session::default();
        // A fresh profile: nothing stored yet is nothing foreign.
        assert_eq!(compare_with_store(&store, &session), StoreComparison::Same);
        session.submit("2+3");
        publish_line(&store, &session);
        assert_eq!(
            compare_with_store(&store, &session),
            StoreComparison::Same,
            "the store holds exactly what this session just wrote"
        );
        // Another frontend writes while the prompt sits open.
        save_history(
            &store,
            &[
                epher_core::HistoryEntry {
                    expr: "2+3".into(),
                    out: Some("= 5".into()),
                },
                epher_core::HistoryEntry {
                    expr: "9*9".into(),
                    out: Some("= 81".into()),
                },
            ],
        )
        .unwrap();
        assert_eq!(
            compare_with_store(&store, &session),
            StoreComparison::Differs
        );
        assert!(reload_session_from_store(&store, &mut session));
        assert_eq!(
            compare_with_store(&store, &session),
            StoreComparison::Same,
            "after the reload the session is the store, so the next line must not reload again"
        );
    }

    fn walk(history: &StoreHistory) -> Vec<String> {
        (0..history.len())
            .filter_map(|index| history.get(index, SearchDirection::Forward).unwrap())
            .map(|hit| hit.entry.into_owned())
            .collect()
    }

    // rustyline's key handlers, as it performs them with `cursor` as
    // the index it holds (`edit_history_next`): `None` when it leaves
    // the line as it is (an empty list, the oldest entry, or a step
    // back to the typed line).
    fn press_up<'a>(history: &'a StoreHistory, cursor: usize) -> Option<SearchResult<'a>> {
        if history.is_empty() {
            return None;
        }
        if cursor != history.len() && cursor == 0 {
            return None;
        }
        let index = cursor - 1;
        if index < history.len() {
            history.get(index, SearchDirection::Reverse).unwrap()
        } else {
            None
        }
    }

    fn press_down<'a>(history: &'a StoreHistory, cursor: usize) -> Option<SearchResult<'a>> {
        if history.is_empty() || cursor == history.len() {
            return None;
        }
        let index = cursor + 1;
        if index < history.len() {
            history.get(index, SearchDirection::Forward).unwrap()
        } else {
            None
        }
    }

    /// rustyline's Ctrl-R (`reverse_incremental_search`): the length
    /// less one is where a reverse search starts.
    fn press_ctrl_r<'a>(history: &'a StoreHistory, term: &str) -> Option<SearchResult<'a>> {
        if history.is_empty() {
            return None;
        }
        let start = history.len() - 1;
        history
            .search(term, start, SearchDirection::Reverse)
            .unwrap()
    }

    fn stored(exprs: &[&str]) -> Vec<HistoryEntry> {
        exprs
            .iter()
            .map(|expr| HistoryEntry {
                expr: (*expr).to_string(),
                out: None,
            })
            .collect()
    }

    // The editor's history is the store's (ADR-0010 amendment,
    // 2026-11-28): rustyline asks the list its length at every prompt
    // and reads it on every Up, and each ask first compares the store's
    // history with what the list was built from. A foreign write is
    // therefore one Up press away, with no interrupt and no second
    // press, and Ctrl-R finds it before any line boundary.
    #[test]
    fn the_editors_history_sees_a_foreign_write_on_the_next_press() {
        let dir = tempfile::tempdir().unwrap();
        let foreign = DocStore::new(FsStore::new(dir.path()));
        let history = StoreHistory::new(DocStore::new(FsStore::new(dir.path()))).unwrap();
        history.begin_prompt();
        let cursor = history.len();
        assert!(history.is_empty(), "a fresh store recalls nothing");
        // Another frontend writes while the prompt sits open.
        save_history(&foreign, &stored(&["9*9"])).unwrap();
        // The first Up press walks into the fresh list: no interrupt.
        let hit = press_up(&history, cursor).unwrap();
        assert_eq!((hit.entry.as_ref(), hit.idx), ("9*9", 0));
        // Ctrl-R before any line boundary finds it too.
        let found = press_ctrl_r(&history, "9*").unwrap();
        assert_eq!(found.entry, "9*9");
    }

    // The REPL's own saves never rebuild the list: the loop tells it
    // what the session just wrote (`note`), and a line typed in this
    // session stays one Up press away, as typed, until another frontend
    // writes. A reload the loop performs (`adopt`) makes the list the
    // store's at once.
    #[test]
    fn the_editors_history_is_not_rebuilt_by_the_repls_own_save() {
        let dir = tempfile::tempdir().unwrap();
        let own = DocStore::new(FsStore::new(dir.path()));
        let mut history = StoreHistory::new(DocStore::new(FsStore::new(dir.path()))).unwrap();
        history.begin_prompt();
        // A line submitted: it joins the list as typed, the session
        // records it, and the loop publishes it.
        assert!(history.add("1+1").unwrap());
        let first = stored(&["1+1"]);
        save_history(&own, &first).unwrap();
        history.note(&first);
        // A line that persists no entry (`save script x`) is still one
        // Up press away after the prompt returns: nothing rebuilt it.
        assert!(history.add("save script x").unwrap());
        history.begin_prompt();
        let cursor = history.len();
        let hit = press_up(&history, cursor).unwrap();
        assert_eq!(hit.entry, "save script x");
        // Another frontend writes: now the list is the store's, and the
        // unrecorded line is gone, as after any reload.
        let grown = stored(&["1+1", "4+5"]);
        save_history(&own, &grown).unwrap();
        history.begin_prompt();
        assert_eq!(walk(&history), ["1+1", "4+5"]);
        // The loop's boundary reload adopts the store's records.
        history.adopt(&grown);
        assert_eq!(walk(&history), ["1+1", "4+5"]);
    }

    // rustyline fixes its cursor at the prompt and steps from it, so a
    // list that grew while the prompt sat open must re-anchor the first
    // step to the new end: the first Up recalls the appended entry, not
    // the one before it, and the walk then proceeds from there, back to
    // the typed line included.
    #[test]
    fn the_first_up_after_the_list_grew_recalls_the_new_entry() {
        let dir = tempfile::tempdir().unwrap();
        let own = DocStore::new(FsStore::new(dir.path()));
        save_history(&own, &stored(&["9*9"])).unwrap();
        let history = StoreHistory::new(DocStore::new(FsStore::new(dir.path()))).unwrap();
        history.begin_prompt();
        let cursor = history.len();
        assert_eq!(cursor, 1, "the length rustyline fixes its cursor at");
        // Another frontend appends while the prompt sits open.
        save_history(&own, &stored(&["9*9", "8*8"])).unwrap();
        let hit = press_up(&history, cursor).unwrap();
        assert_eq!(
            (hit.entry.as_ref(), hit.idx),
            ("8*8", 1),
            "the first Up recalls the appended entry"
        );
        let hit = press_up(&history, hit.idx).unwrap();
        assert_eq!(
            (hit.entry.as_ref(), hit.idx),
            ("9*9", 0),
            "the next Up walks on from there"
        );
        let hit = press_down(&history, hit.idx).unwrap();
        assert_eq!(hit.entry, "8*8");
        assert!(
            press_down(&history, hit.idx).is_none(),
            "Down past the end restores the typed line"
        );
    }

    // The same for the first entry ever: an empty list reports one slot,
    // so rustyline's cursor is a position past the end rather than the
    // oldest entry (which it would refuse to leave).
    #[test]
    fn the_first_up_after_the_first_entry_ever_recalls_it() {
        let dir = tempfile::tempdir().unwrap();
        let own = DocStore::new(FsStore::new(dir.path()));
        let history = StoreHistory::new(DocStore::new(FsStore::new(dir.path()))).unwrap();
        history.begin_prompt();
        let cursor = history.len();
        assert_eq!(cursor, 1, "a position past the end, not the oldest entry");
        save_history(&own, &stored(&["9*9"])).unwrap();
        let hit = press_up(&history, cursor).unwrap();
        assert_eq!((hit.entry.as_ref(), hit.idx), ("9*9", 0));
        assert!(press_up(&history, hit.idx).is_none(), "at the oldest entry");
        assert!(
            press_down(&history, hit.idx).is_none(),
            "back to the typed line"
        );
    }

    // Ctrl-R as the first key after a foreign write starts its search
    // at the new end; it never writes rustyline's cursor, so the Up
    // that follows an aborted search still starts from the old end.
    #[test]
    fn the_first_search_after_the_list_grew_starts_at_the_new_end() {
        let dir = tempfile::tempdir().unwrap();
        let own = DocStore::new(FsStore::new(dir.path()));
        save_history(&own, &stored(&["9*9"])).unwrap();
        let history = StoreHistory::new(DocStore::new(FsStore::new(dir.path()))).unwrap();
        history.begin_prompt();
        let cursor = history.len();
        save_history(&own, &stored(&["9*9", "8*8"])).unwrap();
        let found = press_ctrl_r(&history, "8*").unwrap();
        assert_eq!((found.entry.as_ref(), found.idx), ("8*8", 1));
        // Ctrl-S inside the search asks for the length (its bound), which
        // must not move the length rustyline later checks its cursor
        // against.
        assert_eq!(history.len(), 2, "the bound is the real length");
        history.is_empty();
        assert_eq!(
            history.len(),
            1,
            "the cursor check still sees the old length"
        );
        // The search aborted (or was left with a move key): an Up next
        // recalls the appended entry, in one press.
        let hit = press_up(&history, cursor).unwrap();
        assert_eq!((hit.entry.as_ref(), hit.idx), ("8*8", 1));
    }

    // A second foreign write while the walk is under way: the cursor is
    // a position in the list by then, so the walk simply continues into
    // the new entries, and the typed line is still what Down comes back
    // to (the first write's translation is not applied twice).
    #[test]
    fn a_second_foreign_write_while_walking_continues_the_walk() {
        let dir = tempfile::tempdir().unwrap();
        let own = DocStore::new(FsStore::new(dir.path()));
        save_history(&own, &stored(&["a", "b"])).unwrap();
        let history = StoreHistory::new(DocStore::new(FsStore::new(dir.path()))).unwrap();
        history.begin_prompt();
        let cursor = history.len();
        assert_eq!(cursor, 2);
        save_history(&own, &stored(&["a", "b", "c"])).unwrap();
        let hit = press_up(&history, cursor).unwrap();
        assert_eq!((hit.entry.as_ref(), hit.idx), ("c", 2));
        save_history(&own, &stored(&["a", "b", "c", "d"])).unwrap();
        let hit = press_up(&history, hit.idx).unwrap();
        assert_eq!(
            (hit.entry.as_ref(), hit.idx),
            ("b", 1),
            "walking on, not back to the new end"
        );
        let hit = press_down(&history, hit.idx).unwrap();
        assert_eq!(hit.entry, "c");
        let hit = press_down(&history, hit.idx).unwrap();
        assert_eq!(hit.entry, "d", "the walk reaches the second write's entry");
        assert!(
            press_down(&history, hit.idx).is_none(),
            "back to the typed line"
        );
    }

    // A `clear history` in another frontend shrinks the list under an
    // open prompt: the arrows walk what is left, and Down comes back to
    // the typed line.
    #[test]
    fn a_foreign_clear_under_an_open_prompt_leaves_the_walk_working() {
        let dir = tempfile::tempdir().unwrap();
        let own = DocStore::new(FsStore::new(dir.path()));
        save_history(&own, &stored(&["a", "b", "c"])).unwrap();
        let history = StoreHistory::new(DocStore::new(FsStore::new(dir.path()))).unwrap();
        history.begin_prompt();
        let cursor = history.len();
        assert_eq!(cursor, 3);
        save_history(&own, &stored(&["x"])).unwrap();
        let hit = press_up(&history, cursor).unwrap();
        assert_eq!((hit.entry.as_ref(), hit.idx), ("x", 0));
        assert!(press_up(&history, hit.idx).is_none(), "at the oldest entry");
        assert!(
            press_down(&history, hit.idx).is_none(),
            "back to the typed line"
        );
    }
}
