//! The statement commands (`graph`, `graph3d`, `solar3d`, `save`) are
//! dispatched by the shells, not the parser: the analysis must not
//! squiggle them (ADR-0069 command lines regression).

use epher_lsp::analysis::Document;

const SCRIPT: &str = "\
// Earth from its radius
const radius = 6371 km
circumference = 2 * pi * radius -> km

// The discriminant of a quadratic
def disc(a, b, c) = b^2 - 4*a*c
disc(2, 7, 3)

// Plots and store commands among the answers
graph sin(x) from 0 to 6
graph3d sin(x)*cos(y)
solar3d
save radius
speed = 55 mile/hr
speed in km/hr
";

#[test]
fn dispatched_command_lines_do_not_squiggle() {
    let doc = Document::new(SCRIPT.to_string(), 1);
    let diagnostics = doc.diagnostics();
    assert!(
        diagnostics.is_empty(),
        "expected a clean script, got: {diagnostics:#?}"
    );
}

#[test]
fn command_lines_do_not_swallow_the_neighbouring_answers() {
    let doc = Document::new(SCRIPT.to_string(), 1);
    let hints = doc.inlay_hints(lsp_types::Range {
        start: lsp_types::Position { line: 0, character: 0 },
        end: lsp_types::Position { line: u32::MAX, character: u32::MAX },
    });
    let labels: Vec<&str> = hints
        .iter()
        .map(|h| match &h.label {
            lsp_types::InlayHintLabel::String(s) => s.as_str(),
            _ => "",
        })
        .collect();
    assert_eq!(
        labels,
        vec![
            "= 6371 km",
            "= 40030.173592 km",
            "= 25",
            "= 55 mile/hr",
            "= 88.51392 km/hr",
        ],
        "the plot and store lines stay silent; the value lines answer"
    );
}

#[test]
fn real_errors_still_squiggle() {
    let doc = Document::new("x = 12\nnope + 1".to_string(), 1);
    let diagnostics = doc.diagnostics();
    assert_eq!(diagnostics.len(), 1, "unknown names must still error");
}

#[test]
fn assignment_after_a_command_word_is_still_the_shells_business() {
    // `graph = 5` is dispatched as a broken graph source by the run
    // (the results pane shows the plot error); the analysis stays
    // quiet on it, which is the closest an inlay pass can be.
    let doc = Document::new("graph = 5".to_string(), 1);
    assert!(doc.diagnostics().is_empty());
}
