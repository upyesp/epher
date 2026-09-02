//! epher-shell — the interactive-shell kernel shared by the CLI, TUI, and web
//! frontends (ADR-0010).
//!
//! One policy for shell commands: [`classify`] recognizes `save`,
//! `save script`, and `language` lines; [`prepare`] resolves them against
//! the session (validation and source lookups — `save name` finds functions
//! and constants); [`run_command`] additionally persists through the store
//! for native shells. The webview reuses classify/prepare and persists
//! through its IPC bridge instead.

pub mod plots;

use epher_core::Session;
use epher_i18n::Localizer;
use epher_store::persist;
use epher_store::{DocStore, Storage};

/// A shell command recognized in an input line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// `save name` — save the named function or constant.
    Save {
        name: String,
    },
    SaveScript {
        name: String,
    },
    Language {
        code: String,
    },
    /// `theme light|dark|night` — the UI theme (ADR-0017).
    Theme {
        name: String,
    },
    /// `table <expr> [from a to b] [points n]` — a table of values (ADR-0014).
    Table {
        source: String,
    },
}

/// A command resolved against the session, ready to persist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prepared {
    SaveFunction {
        name: String,
        source: String,
    },
    SaveConstant {
        name: String,
        source: String,
    },
    SaveScript {
        name: String,
        source: String,
    },
    Language {
        code: String,
    },
    Theme {
        name: String,
    },
    /// A preformatted table of values (monospace text, one row per line).
    Table {
        text: String,
    },
}

/// Recognize a shell command in an input line. Anything else (including
/// `save` or `language` without an argument) is `None` — the caller
/// evaluates it, exactly as the CLI always has.
pub fn classify(line: &str) -> Option<Command> {
    let line = line.trim();
    // Order matters: "save script " must win over the shorter "save ".
    if let Some(name) = line.strip_prefix("save script ") {
        let name = name.trim();
        if !name.is_empty() {
            return Some(Command::SaveScript {
                name: name.to_string(),
            });
        }
        return None;
    }
    if let Some(name) = line.strip_prefix("save ") {
        let name = name.trim();
        if !name.is_empty() {
            return Some(Command::Save {
                name: name.to_string(),
            });
        }
        return None;
    }
    if let Some(code) = line.strip_prefix("language ") {
        let code = code.trim();
        if !code.is_empty() {
            return Some(Command::Language {
                code: code.to_string(),
            });
        }
        return None;
    }
    if let Some(name) = line.strip_prefix("theme ") {
        let name = name.trim();
        if !name.is_empty() {
            return Some(Command::Theme {
                name: name.to_string(),
            });
        }
        return None;
    }
    if let Some(source) = line.strip_prefix("table ") {
        let source = source.trim();
        if !source.is_empty() {
            return Some(Command::Table {
                source: source.to_string(),
            });
        }
        return None;
    }
    None
}

/// A `last_line` qualifies as a savable script only if it was a real
/// evaluation, not another shell command.
fn savable(source: &str) -> bool {
    !source.starts_with("save")
        && !source.starts_with("language")
        && !source.starts_with("table")
        && !source.starts_with("quit")
}

/// Resolve a command against the session: validation and source lookups.
/// `Err` carries the localized message to show the user.
pub fn prepare(
    cmd: &Command,
    session: &Session,
    localizer: &Localizer,
) -> Result<Prepared, String> {
    match cmd {
        Command::Save { name } => {
            // a function first, then a constant (ADR-0012)
            if let Some(source) = session.def_sources().get(name) {
                return Ok(Prepared::SaveFunction {
                    name: name.clone(),
                    source: source.clone(),
                });
            }
            match session.const_sources().get(name) {
                Some(source) => Ok(Prepared::SaveConstant {
                    name: name.clone(),
                    source: source.clone(),
                }),
                None => Err(localizer.lookup_args("no-definition", &[("name", name)])),
            }
        }
        Command::SaveScript { name } => match session.last_line() {
            Some(source) if savable(source) => Ok(Prepared::SaveScript {
                name: name.clone(),
                source: source.to_string(),
            }),
            _ => Err(localizer.lookup("nothing-to-save")),
        },
        Command::Language { code } => {
            if epher_i18n::SUPPORTED_LOCALES.contains(&code.as_str()) {
                Ok(Prepared::Language { code: code.clone() })
            } else {
                Err(localizer.lookup_args(
                    "unsupported-language",
                    &[
                        ("code", code),
                        ("supported", &epher_i18n::SUPPORTED_LOCALES.join(", ")),
                    ],
                ))
            }
        }
        Command::Theme { name } => {
            if matches!(name.as_str(), "light" | "dark" | "night") {
                Ok(Prepared::Theme { name: name.clone() })
            } else {
                Err(localizer
                    .lookup_args("unsupported-theme", &[("supported", "light, dark, night")]))
            }
        }
        Command::Table { source } => {
            let spec =
                epher_core::graph::parse_table_source(source).map_err(|e| format!("error: {e}"))?;
            let rows = match &spec.values {
                // The `values <list>` column mode (ADR-0054): rows at
                // the list's x values instead of an even grid.
                Some(vexpr) => {
                    let evaluated = epher_core::eval(vexpr, session.env())
                        .map_err(|e| format!("error: {e}"))?;
                    let epher_core::Value::List(items) = &evaluated else {
                        return Err(format!(
                            "error: {}",
                            epher_core::EpherError::Type(
                                "the values argument must be a list".to_string()
                            )
                        ));
                    };
                    let mut xs = Vec::with_capacity(items.len());
                    for item in items {
                        match item {
                            epher_core::Value::Float(x) => xs.push(*x),
                            other => {
                                return Err(format!(
                                    "error: {}",
                                    epher_core::EpherError::Type(format!(
                                        "the values list holds numbers, got {other:?}"
                                    ))
                                ));
                            }
                        }
                    }
                    epher_core::graph::table_rows_at(
                        &spec.expr,
                        spec.derivative.as_ref(),
                        &xs,
                        session.env(),
                    )
                    .map_err(|e| format!("error: {e}"))?
                }
                None => epher_core::graph::table_rows(
                    &spec.expr,
                    spec.derivative.as_ref(),
                    spec.x_min,
                    spec.x_max,
                    spec.points,
                    session.env(),
                ),
            };
            // The `exact`/`approx` suffix overrides the session's
            // exact-fraction display for this one table (ADR-0054).
            let exact = spec.exact.unwrap_or(session.display().exact_fractions);
            Ok(Prepared::Table {
                text: format_table(&rows, exact),
            })
        }
    }
}

/// A trimmed, monospace-readable number for table columns (the same
/// graph-scale formatting the renderers use).
fn fmt(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".to_string()
    } else {
        s.to_string()
    }
}

/// Render table rows as aligned monospace text; undefined y cells show an
/// em dash (TI-style blank rows). `exact` switches the cells to the
/// session's exact-fraction display (ADR-0044): a cell whose value
/// reconstructs as a small-denominator fraction shows `1/3` instead of
/// `0.333`.
fn format_table(rows: &[(f64, Option<f64>, Option<f64>)], exact: bool) -> String {
    const W: usize = 10;
    let has_derivative = rows.iter().any(|(_, _, d)| d.is_some());
    let mut out = if has_derivative {
        String::from(format!("{:>W$}  {:>W$}  {:>W$}\n", "x", "y", "y'"))
    } else {
        String::from(format!("{:>W$}  {:>W$}\n", "x", "y"))
    };
    let cell = |v: Option<f64>| -> String {
        match v {
            Some(v) => {
                if exact {
                    if let Some(r) = epher_core::reconstruct_fraction(v, 1000, 5e-13) {
                        // Same rule as the result line (ADR-0051): a
                        // terminating decimal stays a decimal.
                        if !epher_core::terminating_decimal(&r) {
                            return format!("{r}");
                        }
                    }
                }
                fmt(v)
            }
            None => "—".to_string(),
        }
    };
    for (x, y, d) in rows {
        if has_derivative {
            out.push_str(&format!(
                "{:>W$}  {:>W$}  {:>W$}\n",
                fmt(*x),
                cell(*y),
                cell(*d)
            ));
        } else {
            out.push_str(&format!("{:>W$}  {:>W$}\n", fmt(*x), cell(*y)));
        }
    }
    out.pop();
    out
}

/// The localized success message for a prepared command.
pub fn message(prepared: &Prepared, localizer: &Localizer) -> String {
    match prepared {
        Prepared::SaveFunction { name, .. } => localizer.lookup_args("saved", &[("name", name)]),
        Prepared::SaveConstant { name, .. } => localizer.lookup_args("saved", &[("name", name)]),
        Prepared::SaveScript { name, .. } => {
            localizer.lookup_args("saved-script", &[("name", name)])
        }
        Prepared::Language { code, .. } => localizer.lookup_args("language-set", &[("code", code)]),
        Prepared::Theme { name, .. } => {
            let label = match name.as_str() {
                "light" => localizer.lookup("theme-light"),
                "night" => localizer.lookup("theme-night"),
                _ => localizer.lookup("theme-dark"),
            };
            localizer.lookup_args("theme-set", &[("name", &label)])
        }
        Prepared::Table { text } => text.clone(),
    }
}

/// Strip the bidi isolating characters Fluent wraps around interpolated
/// values (U+2068/U+2069). Browsers want them (they keep RTL fragments
/// readable); terminals render them as invisible-but-annoying gaps, so the
/// CLI and TUI pass every message through here.
pub fn plain(message: String) -> String {
    message
        .chars()
        .filter(|c| *c != '\u{2068}' && *c != '\u{2069}')
        .collect()
}

/// The outcome of handling a command: the message to show, plus the new
/// language preference when it changed (shells re-resolve their Localizer).
/// `error` marks the message as a diagnostic — the CLI prints those to
/// stderr (ADR-0013), while successful messages stay on stdout.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Handled {
    pub message: String,
    pub error: bool,
    pub language: Option<String>,
    /// The new theme preference when a `theme` command changed it
    /// (frontends re-apply their palette).
    pub theme: Option<String>,
}

impl Handled {
    /// A successful answer: message to stdout, no language switch.
    fn ok(message: String) -> Self {
        Handled {
            message,
            error: false,
            language: None,
            theme: None,
        }
    }

    /// A rejected command: message to stderr, store untouched.
    fn err(message: String) -> Self {
        Handled {
            message,
            error: true,
            language: None,
            theme: None,
        }
    }
}

/// Handle a command for a native shell: prepare, persist, and answer with
/// the localized message (or the prepare error, without touching the store).
pub fn run_command<S: Storage>(
    cmd: &Command,
    session: &mut Session,
    store: &DocStore<S>,
    localizer: &Localizer,
) -> Handled {
    let prepared = match prepare(cmd, session, localizer) {
        Ok(p) => p,
        Err(msg) => return Handled::err(msg),
    };
    let result = match &prepared {
        Prepared::SaveFunction { name, source } => persist::save_function(store, name, source),
        Prepared::SaveConstant { name, source } => persist::save_constant(store, name, source),
        Prepared::SaveScript { name, source } => persist::save_script(store, name, source),
        Prepared::Language { code } => persist::save_language(store, code),
        Prepared::Theme { name } => persist::save_theme(store, name),
        // A table is pure computation — nothing to persist.
        Prepared::Table { .. } => Ok(()),
    };
    match result {
        Ok(()) => {
            let language = if let Prepared::Language { code } = &prepared {
                Some(code.clone())
            } else {
                None
            };
            let theme = if let Prepared::Theme { name } = &prepared {
                Some(name.clone())
            } else {
                None
            };
            let mut handled = Handled::ok(message(&prepared, localizer));
            handled.language = language;
            handled.theme = theme;
            handled
        }
        Err(e) => Handled::err(e.to_string()),
    }
}
