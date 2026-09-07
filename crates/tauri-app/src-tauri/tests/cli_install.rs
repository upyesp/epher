//! The macOS "install the epher command" seam (ADR-0011): the symlink plan
//! is pure path logic, and the osascript/shell escaping must be exact —
//! both testable on any platform. The install itself runs only on macOS.

use app_lib::cli_install::{
    applescript_escape, cli_symlink_plan, manual_instructions, osascript_line, shell_quote,
};
use std::path::Path;

#[test]
fn bundle_executable_yields_a_plan() {
    let exe = Path::new("/Applications/epher.app/Contents/MacOS/epher");
    let (link, target) = cli_symlink_plan(exe).expect("bundle exe should plan");
    assert_eq!(link, Path::new("/usr/local/bin/epher"));
    assert_eq!(target, exe);
}

#[test]
fn non_bundle_executables_have_no_plan() {
    // Dev builds and Linux/Windows paths never offer the action.
    assert!(cli_symlink_plan(Path::new("/home/dev/code/target/debug/epher")).is_none());
    assert!(cli_symlink_plan(Path::new("/usr/bin/epher")).is_none());
    // A bundle whose binary is not named epher, or a broken layout.
    assert!(cli_symlink_plan(Path::new("/Applications/other.app/Contents/MacOS/x")).is_none());
    assert!(cli_symlink_plan(Path::new("/Applications/epher.dmg/Contents/MacOS/epher")).is_none());
    // not .app
}

#[test]
fn shell_quote_surrounds_and_escapes_single_quotes() {
    assert_eq!(
        shell_quote("/Applications/epher.app"),
        "'/Applications/epher.app'"
    );
    assert_eq!(shell_quote("it's"), "'it'\\''s'");
}

#[test]
fn applescript_string_escapes_backslash_and_double_quote() {
    assert_eq!(applescript_escape("ln -sf 'a' 'b'"), "ln -sf 'a' 'b'");
    assert_eq!(applescript_escape("a\"b"), "a\\\"b");
    assert_eq!(applescript_escape("a\\b"), "a\\\\b");
}

#[test]
fn osascript_line_wraps_with_administrator_privileges() {
    assert_eq!(
        osascript_line("ln -sf -- 'x' 'y'"),
        "do shell script \"ln -sf -- 'x' 'y'\" with administrator privileges"
    );
}

#[test]
fn manual_instructions_are_copyable_shell() {
    let msg = manual_instructions(
        Path::new("/Applications/epher.app/Contents/MacOS/epher"),
        Path::new("/usr/local/bin/epher"),
    );
    assert_eq!(
        msg,
        "run: sudo ln -sf -- '/Applications/epher.app/Contents/MacOS/epher' '/usr/local/bin/epher'"
    );
}
