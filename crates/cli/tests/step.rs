//! The `step` seam: one line of input → output (+ optional language switch).
//! Both the REPL loop and the piped-stdin script mode run through it, so
//! behavior is defined once and tested here (logic exists once, ADR-0001).

use epher_cli::step;
use epher_core::Session;
use epher_i18n::Localizer;
use epher_store::DocStore;
use epher_store::FsStore;

fn temp_store() -> DocStore<FsStore> {
    let dir = std::env::temp_dir().join(format!(
        "epher-step-test-{}-{:?}",
        std::process::id(),
        std::time::Instant::now()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    DocStore::new(FsStore::new(dir))
}

#[test]
fn expression_evaluates() {
    let mut session = Session::new();
    let store = temp_store();
    let loc = Localizer::resolve(Some("en"), &[]);
    let out = step(&mut session, &store, &loc, "2 + 3 * 4");
    assert_eq!(out.output.as_deref(), Some("= 14"));
    assert!(out.language.is_none());
}

#[test]
fn session_persists_across_steps() {
    let mut session = Session::new();
    let store = temp_store();
    let loc = Localizer::resolve(Some("en"), &[]);
    step(&mut session, &store, &loc, "def f(x) = x ^ 2");
    let out = step(&mut session, &store, &loc, "f(9)");
    assert_eq!(out.output.as_deref(), Some("= 81"));
}

#[test]
fn error_line_reports_error_and_keeps_going() {
    let mut session = Session::new();
    let store = temp_store();
    let loc = Localizer::resolve(Some("en"), &[]);
    let bad = step(&mut session, &store, &loc, "1 / 0");
    assert!(bad.output.as_deref().unwrap().contains("error"));
    // The session is still usable afterwards.
    let good = step(&mut session, &store, &loc, "6 * 7");
    assert_eq!(good.output.as_deref(), Some("= 42"));
}

#[test]
fn assignment_prints_the_assigned_value() {
    let mut session = Session::new();
    let store = temp_store();
    let loc = Localizer::resolve(Some("en"), &[]);
    let out = step(&mut session, &store, &loc, "x = 5");
    assert_eq!(out.output.as_deref(), Some("= 5"));
}

#[test]
fn language_command_reports_the_new_language() {
    let mut session = Session::new();
    let store = temp_store();
    let loc = Localizer::resolve(Some("en"), &[]);
    let out = step(&mut session, &store, &loc, "language fr");
    assert_eq!(out.language.as_deref(), Some("fr"));
}

#[test]
fn save_command_round_trips_through_the_store() {
    let mut session = Session::new();
    let store = temp_store();
    let loc = Localizer::resolve(Some("en"), &[]);
    step(&mut session, &store, &loc, "def g(x) = x + 1");
    let out = step(&mut session, &store, &loc, "save g");
    assert!(out.output.as_deref().unwrap().contains("g"));
}
