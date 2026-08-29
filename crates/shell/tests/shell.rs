//! epher-shell policy tests: classification, preparation (validation and
//! source resolution), and the native persist path.

use epher_core::Session;
use epher_i18n::Localizer;
use epher_shell::{classify, plain, prepare, run_command, Command, Prepared};
use epher_store::{DocStore, MemoryStore};

fn en() -> Localizer {
    Localizer::resolve(Some("en"), &[])
}

#[test]
fn classifies_the_three_commands() {
    assert_eq!(
        classify("save fib"),
        Some(Command::Save { name: "fib".into() })
    );
    assert_eq!(
        classify("  save script   count  "),
        Some(Command::SaveScript {
            name: "count".into()
        })
    );
    // "save script" must win over the shorter "save " prefix
    assert_eq!(
        classify("save script foo"),
        Some(Command::SaveScript { name: "foo".into() })
    );
    assert_eq!(
        classify("language fr"),
        Some(Command::Language { code: "fr".into() })
    );
}

#[test]
fn non_command_lines_are_none() {
    assert_eq!(classify("1 + 1"), None);
    assert_eq!(classify("saving"), None);
    assert_eq!(classify("save"), None);
    assert_eq!(classify("language"), None);
    assert_eq!(classify("def f(x) = x"), None);
    assert_eq!(classify(""), None);
}

#[test]
fn prepare_resolves_function_source_from_the_session() {
    let mut s = Session::new();
    s.submit("def f(x) = x ^ 2");
    let p = prepare(&Command::Save { name: "f".into() }, &s, &en()).unwrap();
    assert_eq!(
        p,
        epher_shell::Prepared::SaveFunction {
            name: "f".into(),
            source: "def f(x) = x ^ 2".into()
        }
    );
}

#[test]
fn prepare_resolves_constant_source_from_the_session() {
    let mut s = Session::new();
    s.submit("const tax = 0.2");
    let p = prepare(&Command::Save { name: "tax".into() }, &s, &en()).unwrap();
    assert_eq!(
        p,
        epher_shell::Prepared::SaveConstant {
            name: "tax".into(),
            source: "const tax = 0.2".into()
        }
    );
}

#[test]
fn run_command_persists_a_constant() {
    let mut s = Session::new();
    s.submit("const g = 9.81");
    let store = DocStore::new(MemoryStore::default());
    let out =
        plain(run_command(&Command::Save { name: "g".into() }, &mut s, &store, &en()).message);
    assert_eq!(out, "saved g");
    let docs = store.list_constants().unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].name, "g");
    assert_eq!(docs[0].source, "const g = 9.81");
}

#[test]
fn replay_lines_put_constants_between_functions_and_scripts() {
    use epher_store::persist::{replay_lines, save_constant, save_function, save_script};
    let store = DocStore::new(MemoryStore::default());
    save_function(&store, "f", "def f(x) = x + k").unwrap();
    save_script(&store, "use", "f(1)").unwrap();
    save_constant(&store, "k", "const k = 41").unwrap();
    let lines = replay_lines(&store).unwrap();
    assert_eq!(lines, vec!["def f(x) = x + k", "const k = 41", "f(1)"]);
}

#[test]
fn prepare_reports_missing_definition() {
    let s = Session::new();
    let err = plain(prepare(&Command::Save { name: "g".into() }, &s, &en()).unwrap_err());
    assert_eq!(err, "no definition for g in this session");
}

#[test]
fn prepare_uses_the_last_submitted_line_for_scripts() {
    let mut s = Session::new();
    s.submit("x = 0; while x < 5 do x = x + 1; x");
    let p = prepare(
        &Command::SaveScript {
            name: "count".into(),
        },
        &s,
        &en(),
    )
    .unwrap();
    assert!(matches!(p, epher_shell::Prepared::SaveScript { .. }));
}

#[test]
fn prepare_rejects_scripts_when_nothing_qualifies() {
    let s = Session::new();
    let err = plain(prepare(&Command::SaveScript { name: "x".into() }, &s, &en()).unwrap_err());
    assert_eq!(err, "nothing to save (no preceding script line)");

    let mut s = Session::new();
    s.submit("save fib"); // a previous command line is not a script
    let err = plain(prepare(&Command::SaveScript { name: "x".into() }, &s, &en()).unwrap_err());
    assert_eq!(err, "nothing to save (no preceding script line)");
}

#[test]
fn prepare_validates_language_codes() {
    let s = Session::new();
    let p = prepare(&Command::Language { code: "fr".into() }, &s, &en()).unwrap();
    assert_eq!(p, epher_shell::Prepared::Language { code: "fr".into() });

    let err = plain(prepare(&Command::Language { code: "xx".into() }, &s, &en()).unwrap_err());
    assert!(err.starts_with("unsupported language xx"));
}

#[test]
fn run_command_persists_a_function() {
    let mut s = Session::new();
    s.submit("def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)");
    let store = DocStore::new(MemoryStore::default());
    let out =
        plain(run_command(&Command::Save { name: "fib".into() }, &mut s, &store, &en()).message);
    assert_eq!(out, "saved fib");
    let docs = store.list_functions().unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].name, "fib");
    assert!(docs[0].source.starts_with("def fib(n)"));
}

#[test]
fn run_command_persists_language_and_answers() {
    let mut s = Session::new();
    let store = DocStore::new(MemoryStore::default());
    let handled = run_command(
        &Command::Language { code: "es".into() },
        &mut s,
        &store,
        &en(),
    );
    let out = plain(handled.message);
    assert_eq!(handled.language, Some("es".into()));
    assert_eq!(out, "language set to es");
    assert_eq!(
        epher_store::persist::load_language(&store).unwrap(),
        Some("es".into())
    );
}

#[test]
fn run_command_surfaces_prepare_errors_without_persisting() {
    let mut s = Session::new();
    let store = DocStore::new(MemoryStore::default());
    let out = plain(
        run_command(
            &Command::Save {
                name: "nope".into(),
            },
            &mut s,
            &store,
            &en(),
        )
        .message,
    );
    assert_eq!(out, "no definition for nope in this session");
    assert!(store.list_functions().unwrap().is_empty());
}

#[test]
fn classify_recognizes_table() {
    assert_eq!(
        classify("table x ^ 2 from -2 to 2 points 5"),
        Some(Command::Table {
            source: "x ^ 2 from -2 to 2 points 5".into()
        })
    );
    assert_eq!(classify("table"), None);
}

#[test]
fn prepare_formats_a_table_with_blank_rows() {
    let mut s = Session::new();
    let out = prepare(
        &Command::Table {
            source: "x ^ 2 from -1 to 1 points 3".into(),
        },
        &s,
        &en(),
    )
    .unwrap();
    let Prepared::Table { text } = out else {
        panic!("expected a table");
    };
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 4, "header + 3 rows");
    assert!(lines[0].contains('x') && lines[0].contains('y'));
    assert!(lines[1].contains("-1") && lines[1].contains('1'));
    assert!(lines[3].contains('1'));

    // Undefined rows show the blank marker, not a crash or a number.
    s.submit("const k = 2");
    let out = prepare(
        &Command::Table {
            source: "1 / x from -1 to 1 points 5".into(),
        },
        &s,
        &en(),
    )
    .unwrap();
    let Prepared::Table { text } = out else {
        panic!("expected a table");
    };
    assert!(text.lines().nth(3).unwrap().contains('—'));
}

#[test]
fn prepare_reports_table_errors() {
    let s = Session::new();
    assert!(prepare(
        &Command::Table {
            source: "x from 5 to 2".into()
        },
        &s,
        &en(),
    )
    .is_err());
    assert!(prepare(
        &Command::Table {
            source: "x points 0".into()
        },
        &s,
        &en(),
    )
    .is_err());
    assert!(prepare(
        &Command::Table {
            source: "x points 1.5".into()
        },
        &s,
        &en(),
    )
    .is_err());
}

// --- plot state and SVG export (ADR-0020) ---

use epher_shell::plots::Plots;
use std::io::Write;

fn temp_svg(name: &str) -> String {
    let p = std::env::temp_dir().join(format!(
        "epher-shell-test-{name}-{}.svg",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&p);
    p.to_string_lossy().into_owned()
}

#[test]
fn graph_lines_accumulate_and_clear() {
    let mut plots = Plots::new();
    let env = epher_core::Env::default();
    assert_eq!(
        plots.submit_graph("sin(x)", &env, &en()).message,
        "graph: sin(x)"
    );
    assert_eq!(
        plots.submit_graph("x ^ 2", &env, &en()).message,
        "graph: x ^ 2"
    );
    assert_eq!(plots.curves().len(), 2);
    let out = plots.submit_graph("clear", &env, &en());
    assert_eq!(out.message, "Graph cleared");
    assert!(plots.curves().is_empty());
}

#[test]
fn graph_save_writes_the_same_document_the_web_app_copies() {
    let mut plots = Plots::new();
    let env = epher_core::Env::default();
    plots.submit_graph("x ^ 2 - 1", &env, &en());
    let path = temp_svg("2d");
    let out = plots.submit_graph(&format!("save {path}"), &env, &en());
    assert!(!out.error, "{}", out.message);
    let doc = std::fs::read_to_string(&path).unwrap();
    assert!(doc.starts_with("<svg "), "{doc}");
    assert!(doc.contains("viewBox=\"0 0 640 400\""));
    // self-contained: the embedded style is the app's default palette
    assert!(doc.contains("<style>"));
    assert!(doc.contains("y = x ^ 2 - 1"));
    // the points of interest are labeled in the file
    assert!(doc.contains("root"), "{doc}");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn graph3d_save_writes_a_standalone_document() {
    let mut plots = Plots::new();
    let env = epher_core::Env::default();
    plots.submit_surface("x ^ 2 - y ^ 2", &env, &en());
    let path = temp_svg("3d");
    let out = plots.submit_surface(&format!("save {path}"), &env, &en());
    assert!(!out.error, "{}", out.message);
    let doc = std::fs::read_to_string(&path).unwrap();
    assert!(doc.contains("viewBox=\"0 0 640 400\""));
    assert!(doc.contains("transform=\"translate("));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn saving_an_empty_plot_is_a_diagnostic() {
    let plots = Plots::new();
    let out = plots.save_svg(
        "/tmp/never-epher.svg",
        true,
        &epher_core::Env::default(),
        &en(),
    );
    assert!(out.error);
    assert_eq!(out.message, "Nothing is plotted");
    // save without a path names the problem too
    let mut plots = Plots::new();
    let out = plots.submit_graph("save", &epher_core::Env::default(), &en());
    assert!(out.error);
    assert_eq!(out.message, "Name a file to save to");
}

#[test]
fn a_bad_expression_is_a_diagnostic_not_a_crash() {
    let mut plots = Plots::new();
    let out = plots.submit_graph("sin(", &epher_core::Env::default(), &en());
    assert!(out.error);
    assert!(
        !out.message.is_empty(),
        "the diagnostic names the parse error"
    );
    let _ = std::io::stdout().flush();
}

// ===== solar3d (ADR-0037 + the ADR-0015 amendment) =====

#[test]
fn solar3d_builds_a_scene_and_save_writes_it() {
    let mut plots = Plots::new();
    let env = epher_core::Env::default();
    let out = plots.submit_solar3d("jd(2020, 7, 1)", &env, &en());
    assert!(!out.error, "{}", out.message);
    let path = temp_svg("solar");
    let out = plots.submit_solar3d(&format!("save {path}"), &env, &en());
    assert!(!out.error, "{}", out.message);
    let doc = std::fs::read_to_string(&path).unwrap();
    assert!(doc.contains("viewBox=\"0 0 640 400\""), "{doc}");
    // eleven labelled dots, orbit polylines
    assert_eq!(doc.matches("<circle").count(), 11, "{doc}");
    assert!(doc.contains("<title>Jupiter</title>"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn solar3d_clear_and_errors_follow_the_graph_grammar() {
    let mut plots = Plots::new();
    let env = epher_core::Env::default();
    let out = plots.submit_solar3d("now()", &env, &en());
    assert!(!out.error);
    let out = plots.submit_solar3d("clear", &env, &en());
    assert!(!out.error);
    // saving after a clear is the empty-plot diagnostic
    let out = plots.submit_solar3d("save /tmp/never-epher-solar.svg", &env, &en());
    assert!(out.error);
    // a time expression that does not evaluate is the engine's error voice
    let out = plots.submit_solar3d("nope", &env, &en());
    assert!(out.error);
    // a jd with a bad month is a domain error
    let out = plots.submit_solar3d("jd(2020, 13, 1)", &env, &en());
    assert!(out.error);
}
