//! The argument surface of the `epher` command (ADR-0011, ADR-0013).
//!
//! One definition, used by every binary: the unified `epher` executable
//! and the epher-cli dev binary parse identically. This module owns only
//! the *decision* — parsing arguments into an [`Action`]; side effects
//! live with the frontends. Help copy follows clig.dev: concise `-h`
//! that leads with examples, full `--help`, and a `help` subcommand that
//! pages the installed man page when there is one.

use clap::{Parser, Subcommand};

/// The `epher` short help leads with examples (clig.dev: users reach for
/// examples first), then lists the commands — jq-style. The full text
/// lives behind `--help`.
const SHORT_HELP_TEMPLATE: &str = "\
{about-with-newline}
{usage-heading} {usage}

EXAMPLES:
  epher \"2 + 3 * 4\"          evaluate one expression (prints 14)
  epher \"-2 + 5\"             leading minus works (prints 3)
  printf \"x = 3\\nx * 10\\n\" | epher -
                         read a script from standard input
  epher repl              interactive session (quit or Ctrl-D ends it)

{all-args}{after-help}
";

/// epher: a programmable, scriptable calculator.
#[derive(Parser, Debug)]
#[command(
    name = "epher",
    version,
    about = "A programmable, scriptable calculator.",
    long_about = None,
    after_help = "Run `epher --help` for the full manual, or `epher help` to page it.",
    after_long_help = HELP_TAIL,
    help_template = SHORT_HELP_TEMPLATE,
    args_conflicts_with_subcommands = true,
    disable_help_subcommand = true,
)]
pub struct Args {
    /// A script to evaluate; each statement's result prints on its own
    /// line.
    ///
    /// Anything from the language works — `2 + 3 * 4`, `if 3 > 2 then 1
    /// else 0`, a leading minus — and statements join with `;` or
    /// newlines: `epher "x = 10; x + 5"` prints `10` then `15`. Use `-`
    /// to read a script from standard input, line by line, instead.
    /// `graph`/`graph3d` statements plot too; `graph save file.svg`
    /// writes the plot as an SVG image (`epher "graph sin(x); graph
    /// save plot.svg"`).
    #[arg(allow_hyphen_values = true, value_name = "EXPRESSION")]
    pub expression: Option<String>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, PartialEq)]
pub enum Command {
    /// Start an interactive session in the terminal.
    ///
    /// Lines evaluate one at a time and each answer prints as `= result`.
    /// Variables, functions, constants, and history persist between
    /// sessions in the epher store (~/.epher). Leave with `quit`, `exit`,
    /// or Ctrl-D.
    Repl,

    /// Start the full-screen terminal interface.
    ///
    /// A calculator with on-screen history and `graph` plotting. Ctrl-C
    /// (or `q` on an empty input line) leaves.
    Tui,

    /// Start the desktop app.
    ///
    /// The windowed application — the same thing a bare `epher` with no
    /// arguments starts.
    Gui,

    /// Print the manual, or help for a subcommand.
    ///
    /// Opens the installed man page (`man epher`) when the system has
    /// one; otherwise prints the full help text. `epher help repl`
    /// prints help for that subcommand.
    Help {
        /// The subcommand to describe: repl, tui, gui, or help.
        command: Option<String>,
    },
}

/// What the unified binary should do. Derived purely from [`Args`] so the
/// mapping is testable without launching anything.
#[derive(Debug, PartialEq)]
pub enum Action {
    /// Evaluate one expression and print the result.
    OneShot(String),
    /// Read a script from stdin, line by line.
    Stdin,
    /// Interactive REPL in the terminal.
    Repl,
    /// Full-screen terminal UI.
    Tui,
    /// Desktop GUI.
    Gui,
    /// Show the manual: `man epher` when installed, else the long help.
    HelpManual,
    /// Show help for one subcommand (may turn out to be an unknown name —
    /// [`crate::help`] reports that).
    HelpTopic(String),
}

/// Decide the mode. Subcommands win over the expression positional;
/// no arguments at all means GUI (that is what double-click and Start
/// Menu/Finder launches pass, and terminal users get the GUI with a bare
/// `epher` too).
pub fn action_from(args: &Args) -> Action {
    if let Some(command) = &args.command {
        return match command {
            Command::Repl => Action::Repl,
            Command::Tui => Action::Tui,
            Command::Gui => Action::Gui,
            Command::Help { command: None } => Action::HelpManual,
            Command::Help { command: Some(topic) } => Action::HelpTopic(topic.clone()),
        };
    }
    match args.expression.as_deref() {
        Some("-") => Action::Stdin,
        Some(expr) => Action::OneShot(expr.to_string()),
        None => Action::Gui,
    }
}

/// The tail of `--help` (and of the `epher help` fallback): where the
/// documentation lives and where to report bugs — the support path
/// clig.dev asks for. (Examples lead the page via the shared template.)
const HELP_TAIL: &str = "\
Saved functions, constants, scripts, and history live in ~/.epher;
set EPHER_STORE_DIR to relocate the store.

DOCUMENTATION:
  User guide:    https://epher.org/guide/
  Manual page:   man epher  (or `epher help`)
  Report issues: https://github.com/upyesp/epher/issues";
