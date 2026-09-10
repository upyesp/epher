//! The evaluation trace (ADR-0066): a document evaluated statement by
//! statement, each outcome carrying its span and what it produced.
//! This is the data behind inline results and ranged diagnostics.

use epher_core::{evaluation_trace, Session, StatementOutcome};

fn displays(outcomes: &[StatementOutcome]) -> Vec<Option<&str>> {
    outcomes
        .iter()
        .map(|o| o.display.as_deref())
        .collect()
}

#[test]
fn every_statement_records_what_it_produced() {
    let mut session = Session::default();
    let outcomes = evaluation_trace(&mut session, "x = 40 + 2\nx + 1");
    assert_eq!(outcomes.len(), 2);
    assert_eq!(displays(&outcomes), vec![Some("42"), Some("43")]);
    assert!(outcomes.iter().all(|o| o.error.is_none()));
}

#[test]
fn outcomes_carry_their_statement_spans() {
    let mut session = Session::default();
    let text = "x = 40 + 2\nx + 1";
    let outcomes = evaluation_trace(&mut session, text);
    assert_eq!(&text[outcomes[0].span.start..outcomes[0].span.end], "x = 40 + 2");
    assert_eq!(&text[outcomes[1].span.start..outcomes[1].span.end], "x + 1");
}

#[test]
fn statements_without_a_value_produce_no_display() {
    let mut session = Session::default();
    let outcomes = evaluation_trace(&mut session, "def f(a) = a * 2\nf(21)");
    assert_eq!(displays(&outcomes), vec![None, Some("42")]);
    assert!(outcomes.iter().all(|o| o.error.is_none()));
}

#[test]
fn state_flows_across_statements() {
    let mut session = Session::default();
    let outcomes = evaluation_trace(&mut session, "base = 2\nheight = 5\nbase * height / 2");
    assert_eq!(displays(&outcomes), vec![Some("2"), Some("5"), Some("5")]);
}

#[test]
fn the_first_error_ends_the_pass() {
    let mut session = Session::default();
    let outcomes = evaluation_trace(&mut session, "x = 1 / 0\ny = 2");
    assert_eq!(outcomes.len(), 1);
    assert!(outcomes[0].error.is_some());
    assert_eq!(&outcomes[0].error.as_ref().expect("set").to_string(), "division by zero");
}

#[test]
fn statements_before_the_error_still_stand() {
    let mut session = Session::default();
    let text = "a = 1\nb = 1 / 0\nc = 3";
    let outcomes = evaluation_trace(&mut session, text);
    assert_eq!(outcomes.len(), 2);
    assert_eq!(displays(&outcomes), vec![Some("1"), None]);
    assert_eq!(&text[outcomes[1].span.start..outcomes[1].span.end], "b = 1 / 0");
}

#[test]
fn a_parse_failure_yields_one_error_outcome() {
    let mut session = Session::default();
    let text = "2 3";
    let outcomes = evaluation_trace(&mut session, text);
    assert_eq!(outcomes.len(), 1);
    let err = outcomes[0].error.as_ref().expect("a parse error");
    assert!(err.to_string().starts_with("parse error:"));
    let span = outcomes[0].span;
    assert_eq!(&text[span.start..span.end], "3");
}

#[test]
fn definitions_reach_the_session() {
    let mut session = Session::default();
    evaluation_trace(&mut session, "def double(a) = a * 2\nconst tax = 21\nv = double(3)");
    assert!(session.def_sources().contains_key("double"));
    assert!(session.const_sources().contains_key("tax"));
}

#[test]
fn defined_names_surface_for_completion() {
    let mut session = Session::default();
    evaluation_trace(&mut session, "alpha = 1\nbeta = 2\ndef gem(a) = a\nconst k = 9");
    // `ans` rides along: every value-producing statement records it,
    // and the keypad carries an `ans` key.
    assert_eq!(session.env().binding_names(), vec!["alpha", "ans", "beta"]);
    assert_eq!(session.env().constant_names(), vec!["k"]);
    assert_eq!(session.env().function_names(), vec!["gem"]);
}

#[test]
fn the_loop_budget_is_one_per_document() {
    // 200k passes split across two loops overruns the shared budget;
    // either loop alone would fit. The pass must stop with an error.
    let mut session = Session::default();
    let text = "n = 0\nfor k in 1 to 100000 do n = n + 1 end\nfor k in 1 to 100000 do n = n + 1 end";
    let outcomes = evaluation_trace(&mut session, text);
    let last = outcomes.last().expect("at least one outcome");
    assert!(last.error.is_some());
}
