//! Help and manual behavior (ADR-0013).
//!
//! `epher help` follows the git/npm convention: when the system has an
//! installed man page, open it (`man epher`) so the user gets the real,
//! paged manual; otherwise print the full `--help` text — which is what
//! macOS app installs, Windows, and any system without the page get.
//! `epher help <command>` prints clap's help for that subcommand.

use clap::{CommandFactory, Parser};

use crate::dispatch::Args;
use crate::term;

/// Show the manual for a bare `epher help`. Returns the process exit
/// code (man's, when it ran).
pub fn manual() -> i32 {
    match run_man_page() {
        Some(code) => code,
        None => {
            print_long_help();
            0
        }
    }
}

/// `man epher` if the page is installed. `Some(exit_code)` when man ran;
/// `None` when there is no man or no page and the caller should fall back
/// to the long help.
fn run_man_page() -> Option<i32> {
    // `man -w epher` locates the page without displaying it: exit 0 with a
    // printed path means the page exists on this system. Anything else —
    // no man binary, no page — falls back to the built-in help.
    let located = std::process::Command::new("man")
        .arg("-w")
        .arg("epher")
        .output()
        .ok()?;
    if !located.status.success() || located.stdout.is_empty() {
        return None;
    }
    let shown = std::process::Command::new("man")
        .arg("epher")
        .status()
        .ok()?;
    Some(shown.code().unwrap_or(0))
}

/// Print the full `--help` text to stdout (the `epher help` fallback).
pub fn print_long_help() {
    let mut cmd = Args::command();
    cmd.build();
    let _ = cmd.print_long_help();
}

/// Is `name` a subcommand of epher? (Exposed for `epher help <name>`
/// validation and tests.)
pub fn known_topic(name: &str) -> bool {
    let mut cmd = Args::command();
    cmd.build();
    cmd.find_subcommand(name).is_some()
}

/// Print help for one subcommand (from `epher help <name>`), or a
/// clap-style usage error for an unknown name. Exits the process.
pub fn topic(name: &str) -> ! {
    if !known_topic(name) {
        term::error(&format!("unrecognized subcommand '{name}'"));
        eprintln!();
        eprintln!("Usage: epher help [COMMAND]");
        eprintln!();
        eprintln!("For more information, try '--help'.");
        std::process::exit(2);
    }
    // Re-parse with --help so clap renders the subcommand's help exactly
    // as `epher <name> --help` would — right usage line, colors, and
    // stdout (not stderr).
    match Args::try_parse_from(["epher", name, "--help"]) {
        Err(e) => e.exit(),
        Ok(_) => unreachable!("--help always exits through clap"),
    }
}
