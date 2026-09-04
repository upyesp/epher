//! The `epher` command surface: argument parsing → [`Action`] (pure, no
//! side effects). The modes themselves are thin wrappers over the
//! frontends' library entry points. The same suite covers the unified
//! binary and the dev binary — they share this dispatch by construction.

use clap::error::ErrorKind;
use clap::Parser;

use epher_cli::dispatch::{action_from, Action, Args};
use epher_cli::help;

fn parse(args: &[&str]) -> Action {
    let parsed = Args::try_parse_from(std::iter::once("epher").chain(args.iter().copied()))
        .expect("args should parse");
    action_from(&parsed)
}

#[test]
fn bare_epher_opens_the_gui() {
    assert_eq!(parse(&[]), Action::Gui);
}

#[test]
fn gui_subcommand_is_explicit_gui() {
    assert_eq!(parse(&["gui"]), Action::Gui);
}

#[test]
fn expression_is_one_shot() {
    assert_eq!(parse(&["2 + 2"]), Action::OneShot("2 + 2".to_string()));
}

#[test]
fn dash_reads_stdin() {
    assert_eq!(parse(&["-"]), Action::Stdin);
}

#[test]
fn negative_numbers_are_expressions_not_flags() {
    assert_eq!(parse(&["-5"]), Action::OneShot("-5".to_string()));
}

#[test]
fn subcommands_select_their_modes() {
    assert_eq!(parse(&["repl"]), Action::Repl);
    assert_eq!(parse(&["tui"]), Action::Tui);
}

#[test]
fn help_subcommand_routes_to_manual_or_topic() {
    assert_eq!(parse(&["help"]), Action::HelpManual);
    assert_eq!(
        parse(&["help", "repl"]),
        Action::HelpTopic("repl".to_string())
    );
}

#[test]
fn help_and_version_still_work_despite_hyphen_values() {
    let help = Args::try_parse_from(["epher", "--help"]).unwrap_err();
    assert_eq!(help.kind(), ErrorKind::DisplayHelp);
    let version = Args::try_parse_from(["epher", "--version"]).unwrap_err();
    assert_eq!(version.kind(), ErrorKind::DisplayVersion);
}

#[test]
fn existing_files_are_scripts_not_expressions() {
    // A file with a path-ish character runs as a script (ADR-0040).
    let dir = tempfile::tempdir().unwrap();
    let script = dir.path().join("sine.es");
    std::fs::write(&script, "1 + 1\n").unwrap();
    assert_eq!(
        parse(&[script.display().to_string().as_str()]),
        Action::ScriptFile(script)
    );
}

#[test]
fn path_shaped_arguments_that_name_no_file_are_reported() {
    // ADR-0058 amendment: nothing the tokenizer could parse starts with
    // a slash, backslash, `./`, `../`, or a drive letter, so these are
    // missing scripts, not expressions.
    for path in [
        "/usr/lib/epher/scripts/astronomy/moon/full-moons.epher",
        "./sine.es",
        "../sine.es",
        "C:\\Users\\pete\\scripts\\full-moons.epher",
        "C:/Users/pete/scripts/full-moons.epher",
        "\\\\server\\share\\sine.es",
    ] {
        assert_eq!(
            parse(&[path]),
            Action::MissingScriptFile(path.to_string()),
            "{path:?}"
        );
    }
}

#[test]
fn dotted_typos_and_division_stay_expressions() {
    // ADR-0040: `1.5.5` keeps its parse error, and interior slashes are
    // division — `a/b` is a real expression, not a path.
    for expr in ["1.5.5", "1/2", "a/b", ".5", "-5"] {
        assert_eq!(
            parse(&[expr]),
            Action::OneShot(expr.to_string()),
            "{expr:?}"
        );
    }
}

#[test]
fn repl_after_an_expression_is_rejected_not_silently_merged() {
    // `epher "1+1" repl` must not quietly pick one meaning: the positional
    // expression and the subcommands are mutually exclusive (an error that
    // goes to stderr, unlike --help/--version which go to stdout).
    let err = Args::try_parse_from(["epher", "1 + 1", "repl"]).unwrap_err();
    assert!(err.use_stderr(), "unexpected kind: {:?}", err.kind());
}

#[test]
fn help_topics_are_validated_against_the_subcommands() {
    for topic in ["repl", "tui", "gui", "help"] {
        assert!(help::known_topic(topic), "{topic} should be known");
    }
    for bogus in ["bogus", "REPL", ""] {
        assert!(!help::known_topic(bogus), "{bogus:?} should not be known");
    }
}

#[test]
fn short_help_leads_with_examples_and_points_at_long_help() {
    use clap::CommandFactory;
    use std::io::Write;
    let mut cmd = Args::command();
    cmd.build();
    let mut out = Vec::new();
    cmd.write_help(&mut out).unwrap();
    let text = String::from_utf8(out).unwrap();
    assert!(text.contains("EXAMPLES:"), "short help leads with examples");
    assert!(
        text.contains("epher \"2 + 3 * 4\""),
        "first example is one-shot"
    );
    assert!(
        text.contains("--help"),
        "short help points at the full manual"
    );
    assert!(
        !text.contains("DOCUMENTATION:"),
        "short help stays concise; the tail lives in --help"
    );
}
