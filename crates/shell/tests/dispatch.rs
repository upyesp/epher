//! The shells dispatch plot prefixes and store commands before the
//! evaluator; the language server masks exactly these (ADR-0069
//! command lines).

use epher_shell::{dispatched_piece_ranges, is_dispatched_piece, run_script};

#[test]
fn dispatch_table() {
    assert!(is_dispatched_piece("graph sin(x) from 0 to 6"));
    assert!(is_dispatched_piece("graph3d sin(x)*cos(y)"));
    assert!(is_dispatched_piece("solar3d"));
    assert!(is_dispatched_piece("solar3d 2 days"));
    assert!(is_dispatched_piece("save radius"));
    assert!(is_dispatched_piece("save script myscript"));
    assert!(is_dispatched_piece("language en"));
    // `graph = 42` is dispatched too: the run reads it as a broken
    // graph source, so the analysis mirrors that instead of seeing an
    // assignment the language would never run.
    assert!(is_dispatched_piece("graph = 42"));
    assert!(!is_dispatched_piece("x = graph"));
    assert!(!is_dispatched_piece("speed in km/hr"));
}

#[test]
fn ranges_land_on_the_command_pieces() {
    let text = "x = 1\ngraph sin(x) from 0 to 6\ny = x * 2";
    let ranges = dispatched_piece_ranges(text);
    assert_eq!(ranges.len(), 1);
    assert_eq!(&text[ranges[0].clone()], "graph sin(x) from 0 to 6");
}

#[test]
fn bare_solar3d_plots_the_default_scene() {
    let localizer = epher_i18n::Localizer::resolve(None, &[]);
    let run = run_script("solar3d", &localizer);
    assert_eq!(run.lines.len(), 1);
    assert!(!run.lines[0].error, "bare solar3d must plot, not error");
    assert!(!run.svgs.is_empty(), "the default scene renders an svg");
}

#[test]
fn a_variable_named_graph_stays_yours() {
    // Not a dispatched piece, so the evaluator sees the assignment.
    assert!(!is_dispatched_piece("graphed = 42"));
}
