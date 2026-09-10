//! Statement spans (ADR-0066): every statement parsed from a script
//! records the source range it was parsed from, and parse failures
//! carry the range of the offending token. These spans anchor inline
//! results, hover ranges, and ranged diagnostics in the editors.

use epher_core::{parse_script_with_spans, SpannedError};

fn spans_of(text: &str) -> Result<Vec<String>, SpannedError> {
    parse_script_with_spans(text).map(|stmts| {
        stmts
            .iter()
            .map(|s| text[s.span.start..s.span.end].to_string())
            .collect()
    })
}

#[test]
fn every_statement_records_its_source_span() {
    let text = "x = 1 + 2\ny = x * 10";
    assert_eq!(
        spans_of(text).expect("parses"),
        vec!["x = 1 + 2".to_string(), "y = x * 10".to_string()]
    );
}

#[test]
fn statements_on_one_line_get_distinct_spans() {
    let text = "a = 1; b = 2";
    assert_eq!(
        spans_of(text).expect("parses"),
        vec!["a = 1".to_string(), "b = 2".to_string()]
    );
}

#[test]
fn a_def_spans_its_whole_body() {
    let text = "def area(r) do\n  r^2 * pi\nend\narea(3)";
    let spans = spans_of(text).expect("parses");
    assert_eq!(spans.len(), 2);
    assert_eq!(spans[0], "def area(r) do\n  r^2 * pi\nend");
    assert_eq!(spans[1], "area(3)");
}

#[test]
fn leading_indentation_stays_outside_the_span() {
    let text = "  x = 5";
    assert_eq!(spans_of(text).expect("parses"), vec!["x = 5".to_string()]);
}

#[test]
fn a_parse_error_points_at_the_offending_token() {
    // `2` parses; `3` has no separator before it, so the error sits on `3`.
    let text = "2 3";
    let err = parse_script_with_spans(text).expect_err("does not parse");
    let span = err.span;
    assert_eq!(&text[span.start..span.end], "3");
}

#[test]
fn a_parse_error_at_the_end_points_at_the_last_token() {
    // `1 +` then a newline: parsing stops at the newline (a separator
    // token), so the span is that newline.
    let text = "1 +\n2";
    let err = parse_script_with_spans(text).expect_err("does not parse");
    let span = err.span;
    assert_eq!(&text[span.start..span.end], "\n");
}

#[test]
fn an_unterminated_string_points_at_itself() {
    let text = "x = \"oops";
    let err = parse_script_with_spans(text).expect_err("does not parse");
    let span = err.span;
    assert_eq!(&text[span.start..span.end], "\"oops");
}

#[test]
fn empty_text_yields_no_statements() {
    assert_eq!(spans_of("").expect("parses"), Vec::<String>::new());
}
