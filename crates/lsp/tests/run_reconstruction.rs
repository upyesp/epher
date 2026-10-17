//! The run reconstruction (ADR-0069): `epher/run` on already-analyzed
//! text must produce exactly what `epher_shell::run_script` produces,
//! without evaluating a second time. These tests walk a corpus of
//! documents through both paths and compare the lines field for field;
//! documents the reconstruction refuses (plots, tables, failed passes)
//! must come back as `None` so the caller falls through to the real
//! run.

use epher_core::{evaluation_trace, Session, StatementOutcome};
use epher_lsp::analysis::reconstruct_run;
use epher_shell::{run_script, ScriptLine};

fn trace(text: &str) -> Vec<StatementOutcome> {
    let mut session = Session::default();
    // The analysis masks dispatched pieces before parsing; the
    // reconstruction requires none, so an unmasked trace matches.
    evaluation_trace(&mut session, text)
}

fn assert_lines_match(text: &str) {
    let outcomes = trace(text);
    let rebuilt = reconstruct_run(text, &outcomes)
        .unwrap_or_else(|| panic!("reconstruction refused a clean document: {text:?}"));
    let localizer = epher_i18n::Localizer::resolve(None, &[]);
    let want = run_script(text, &localizer).lines;
    assert_eq!(
        normalize(rebuilt),
        normalize(want),
        "reconstruction diverged from the run for {text:?}"
    );
}

/// The localizer's message strings are the same in both paths (the
/// default locale); normalize nothing, compare everything.
fn normalize(lines: Vec<ScriptLine>) -> Vec<(u32, String, Option<String>, bool)> {
    lines
        .into_iter()
        .map(|l| (l.line, l.source, l.display, l.error))
        .collect()
}

#[test]
fn assignments_defs_and_consts_rebuild_exactly() {
    assert_lines_match("x = 40 + 2\ndef double(n) = n * 2\nconst k = double(x)\nk * 2\n");
}

#[test]
fn comments_and_blank_lines_keep_their_rows() {
    assert_lines_match(
        "/* a header block */\nx = 1\n\n// a line comment\ny = x + 1\n/* trailing */\n",
    );
}

#[test]
fn control_flow_and_multiline_bodies_rebuild_exactly() {
    assert_lines_match(
        "total = 0\nfor i in 1 to 5 do total = total + i\ntotal\ndef fact(n) do\n  if n <= 1 then 1 else n * fact(n - 1)\nend\nfact(6)\nwhile total > 100 do total = total - 50\ntotal\n",
    );
}

#[test]
fn semicolon_chains_and_multi_value_pieces_rebuild_exactly() {
    assert_lines_match("a = 1; b = 2\na + b; a * b\nc = a + b; c\n");
}

#[test]
fn unicode_and_strings_rebuild_exactly() {
    assert_lines_match("s = \"pi ≈ 3.14\"\nπ = 3.14159\nround(π * 100) / 100\n");
}

#[test]
fn plots_tables_and_failing_passes_are_refused() {
    // dispatched pieces: the run renders what the trace never sees
    let outcomes = trace("x = 1\ngraph sin(x) from 0 to 6\n");
    assert!(reconstruct_run("x = 1\ngraph sin(x) from 0 to 6\n", &outcomes).is_none());
    let outcomes = trace("x = 1\ntable sin(x) from 0 to 3\n");
    assert!(reconstruct_run("x = 1\ntable sin(x) from 0 to 3\n", &outcomes).is_none());
    // a failed pass is fail-fast: outcomes after the error are missing
    let text = "x = 40 + 2\n1 / 0\ny = 3\n";
    let outcomes = trace(text);
    assert!(reconstruct_run(text, &outcomes).is_none());
}
