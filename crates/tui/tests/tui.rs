use epher_core::Sample;
use epher_core::Session;
use epher_tui::{answer_fits, render_ascii, render_ascii3d, render_ascii_data, App};

fn app_session_constant(app: &App, name: &str) -> epher_core::Value {
    app.session()
        .env()
        .constant(name)
        .cloned()
        .unwrap_or(epher_core::Value::float(f64::NAN))
}

#[test]
fn answer_routing_follows_the_web_rule() {
    // ADR-0056: one short answer stays on the answer line; empty,
    // multi-line, and over-cap results go to the result pane.
    assert!(answer_fits("= 4", true));
    assert!(answer_fits(&"= 4".to_string(), false));
    assert!(!answer_fits("", true), "empty renders nowhere");
    assert!(
        !answer_fits("= 2\n= 4", true),
        "a transcript is never a one-line answer"
    );
    // The caps count the whole line, "= " prefix included, exactly as
    // the web's rule counts the result text.
    assert!(
        !answer_fits(&format!("= {}", "3".repeat(43)), true),
        "over the desktop cap the answer moves to the pane"
    );
    assert!(answer_fits(&format!("= {}", "3".repeat(42)), true));
    // The narrow stack uses the phone-like cap, like the web on mobile.
    assert!(answer_fits(&format!("= {}", "3".repeat(22)), false));
    assert!(!answer_fits(&format!("= {}", "3".repeat(23)), false));
}

#[test]
fn submit_evaluates_against_persistent_env() {
    let mut app = App::default();
    app.set_input("x = 5; x + 1");
    app.submit();
    // A script shows every answer it produced, in order (ADR-0052).
    assert_eq!(app.result(), "= 5\n= 6");
    app.set_input("x * 2");
    app.submit();
    assert_eq!(app.result(), "= 10");
    assert_eq!(app.history().len(), 2);
}

#[test]
fn app_with_session_starts_from_seeded_history() {
    let mut app = App::with_session(Session::with_history(vec!["old  = 1".to_string()]));
    assert_eq!(app.history().len(), 1);
    app.set_input("1 + 1");
    app.submit();
    assert_eq!(app.result(), "= 2");
    assert_eq!(app.history().len(), 2);
}

#[test]
fn errors_are_shown_not_crashing() {
    let mut app = App::default();
    app.set_input("1/0");
    app.submit();
    assert_eq!(app.result(), "error: division by zero");
}

#[test]
fn empty_input_does_nothing() {
    let mut app = App::default();
    app.submit();
    assert_eq!(app.result(), "");
    assert_eq!(app.history().len(), 0);
}

#[test]
fn graph_command_samples_expression() {
    let mut app = App::default();
    app.submit_graph("x ^ 2").expect("graph should sample");
    assert_eq!(app.graph().len(), 1);
    assert_eq!(app.graph()[0].samples.len(), 120);
    app.submit_graph("1 / x").expect("graph should sample");
    assert_eq!(app.graph().len(), 2, "curves overlay");
}

#[test]
fn graph_command_uses_session_functions() {
    let mut app = App::default();
    app.set_input("def f(x) = x ^ 3");
    app.submit();
    app.submit_graph("f(x)").expect("graph should sample");
    assert_eq!(app.graph().len(), 1);
    assert_eq!(app.graph()[0].samples.len(), 120);
}

#[test]
fn graph_command_records_source_for_caption() {
    let mut app = App::default();
    assert_eq!(app.graph().len(), 0);
    app.submit_graph("x ^ 2").expect("graph should sample");
    assert_eq!(app.graph()[0].source, "x ^ 2");
}

#[test]
fn graph_clear_empties_the_plot() {
    let mut app = App::default();
    app.submit_graph("x ^ 2").expect("graph should sample");
    app.submit_graph("clear").expect("clear should work");
    assert!(app.graph().is_empty());
    assert!(app.pois().is_empty());
}

#[test]
fn data_plots_own_the_pane_and_render_ascii() {
    let mut app = App::default();
    app.submit_graph("scatter({1, 2, 3}, {2, 4, 6})")
        .expect("scatter should plot");
    let data = app.data().expect("a data plot");
    assert_eq!(data.points, vec![(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)]);
    let fit = data.fit.expect("fit");
    assert!((fit.fit.a - 2.0).abs() < 1e-9 && (fit.fit.b).abs() < 1e-9);
    // the scatter draws glyphs
    let ascii = render_ascii_data(data, 40, 12);
    assert!(ascii.contains('o'), "{ascii}");
    // a plain curve command displaces the data plot
    app.submit_graph("x").expect("curve should sample");
    assert!(app.data().is_none());
    assert_eq!(app.graph().len(), 1);
    // and a data plot displaces the curves back
    app.submit_graph("histogram({1, 2, 2, 3, 3, 3, 4}, 4)")
        .expect("histogram should plot");
    assert!(app.graph().is_empty());
    let hist = app.data().expect("a histogram");
    let counts: Vec<f64> = hist.bins.iter().map(|(_, _, c)| *c).collect();
    assert_eq!(counts, vec![1.0, 2.0, 3.0, 1.0]);
    let ascii = render_ascii_data(hist, 40, 12);
    assert!(ascii.contains('█'), "{ascii}");
    app.submit_graph("boxplot({1, 2, 2, 3, 3, 3, 9})")
        .expect("boxplot should plot");
    let boxed = app.data().expect("a boxplot");
    assert_eq!(boxed.boxplot, Some([1.0, 2.0, 3.0, 3.0, 9.0]));
    let ascii = render_ascii_data(boxed, 40, 12);
    assert!(ascii.contains('┼'), "{ascii}");
    // clear empties the data plot too
    app.submit_graph("clear").expect("clear should work");
    assert!(app.data().is_none());
}

#[test]
fn graph_reports_points_of_interest() {
    let mut app = App::default();
    app.submit_graph("x ^ 2 - 1").expect("graph should sample");
    let pois = app.pois();
    assert!(
        pois.iter()
            .any(|p| p.kind == epher_core::graph::InterestKind::Root && (p.x - 1.0).abs() < 1e-3),
        "root near x=1 in {pois:?}"
    );
    app.submit_graph("2 - x").expect("graph should sample");
    assert!(app
        .pois()
        .iter()
        .any(|p| p.kind == epher_core::graph::InterestKind::Intersection));
}

#[test]
fn graph_parses_parametric_polar_and_domains() {
    let mut app = App::default();
    app.submit_graph("param t, t ^ 2 from 0 to 3")
        .expect("parametric should sample");
    app.submit_graph("polar 2").expect("polar should sample");
    assert_eq!(app.graph().len(), 2);
    assert!(app.submit_graph("x from 5 to -5").is_err());
}

fn curve_of(ys: &[f64]) -> epher_core::graph::SampledCurve {
    let samples = ys
        .iter()
        .enumerate()
        .map(|(i, y)| Sample { x: i as f64, y: *y })
        .collect::<Vec<_>>();
    let expr = epher_core::parse("0").unwrap();
    epher_core::graph::SampledCurve {
        source: "test".to_string(),
        kind: epher_core::graph::CurveKind::Cartesian(expr),
        domain: (0.0, (ys.len() - 1) as f64),
        samples,
        fill: None,
    }
}

#[test]
fn render_ascii_plots_a_diagonal() {
    let samples = vec![
        Sample { x: 0.0, y: 0.0 },
        Sample { x: 1.0, y: 1.0 },
        Sample { x: 2.0, y: 2.0 },
    ];
    let curves = [curve_of(&[0.0, 1.0, 2.0])];
    assert_eq!(render_ascii(&curves, 3, 3, None), "··o\n·o·\no··");
    let _ = samples;
}

#[test]
fn render_ascii_handles_empty_and_non_finite() {
    assert_eq!(render_ascii(&[], 3, 3, None), "");
    let expr = epher_core::parse("0").unwrap();
    let c = epher_core::graph::SampledCurve {
        source: "test".to_string(),
        kind: epher_core::graph::CurveKind::Cartesian(expr),
        domain: (0.0, 1.0),
        samples: vec![
            Sample {
                x: f64::NAN,
                y: 0.0,
            },
            Sample {
                x: 0.0,
                y: f64::INFINITY,
            },
            Sample { x: 1.0, y: 1.0 },
        ],
        fill: None,
    };
    let out = render_ascii(&[c], 3, 3, None);
    assert!(out.contains('o'));
    assert!(!out.contains("NaN"));
}

#[test]
fn render_ascii_marks_axes_when_zero_is_inside() {
    // y = x on [-2, 2]: zero is strictly inside both ranges, so a vertical
    // and a horizontal axis must appear (the curve glyph wins on overlap).
    let expr = epher_core::parse("x").unwrap();
    let c = epher_core::graph::SampledCurve {
        source: "x".to_string(),
        kind: epher_core::graph::CurveKind::Cartesian(expr),
        domain: (-2.0, 2.0),
        samples: vec![
            Sample { x: -2.0, y: -2.0 },
            Sample { x: -1.0, y: -1.0 },
            Sample { x: 0.0, y: 0.0 },
            Sample { x: 1.0, y: 1.0 },
            Sample { x: 2.0, y: 2.0 },
        ],
        fill: None,
    };
    let out = render_ascii(&[c], 5, 5, None);
    assert!(out.contains('|'), "vertical axis: {out}");
    assert!(out.contains('-'), "horizontal axis: {out}");
}

#[test]
fn render_ascii_uses_distinct_glyphs_and_fills() {
    let expr = epher_core::parse("0").unwrap();
    let a = epher_core::graph::SampledCurve {
        source: "a".to_string(),
        kind: epher_core::graph::CurveKind::Cartesian(expr.clone()),
        domain: (0.0, 1.0),
        samples: vec![Sample { x: 0.0, y: 0.0 }, Sample { x: 1.0, y: 1.0 }],
        fill: Some(epher_core::graph::Fill::Below),
    };
    let b = epher_core::graph::SampledCurve {
        source: "b".to_string(),
        kind: epher_core::graph::CurveKind::Cartesian(expr),
        domain: (0.0, 1.0),
        samples: vec![Sample { x: 0.0, y: 1.0 }, Sample { x: 1.0, y: 0.0 }],
        fill: None,
    };
    let out = render_ascii(&[a, b], 4, 4, None);
    assert!(out.contains('o'), "first curve glyph: {out}");
    assert!(out.contains('x'), "second curve glyph: {out}");
    assert!(out.contains('.'), "fill shading: {out}");
}

// --- shell commands through the App seam (ADR-0010) ---

use epher_i18n::Localizer;
use epher_shell::plain;
use epher_store::persist::{history as load_history, load_language};
use epher_store::{DocStore, FsStore};

fn scratch_store() -> (DocStore<FsStore>, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let store = DocStore::new(FsStore::new(dir.path()));
    (store, dir)
}

#[test]
fn submit_line_dispatches_save_and_persists() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    app.set_input("def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)");
    app.submit();
    app.set_input("save fib");
    app.submit_line(
        &app.input().to_string(),
        &store,
        &Localizer::resolve(Some("en"), &[]),
    );
    assert_eq!(app.result(), "saved fib");
    assert_eq!(store.list_functions().unwrap().len(), 1);
    // commands must not enter history
    assert_eq!(app.history().len(), 1);
    assert!(app.input().is_empty());
}

#[test]
fn submit_line_evaluates_and_persists_history() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    app.set_input("2 + 3");
    app.submit_line("2 + 3", &store, &Localizer::resolve(Some("en"), &[]));
    assert_eq!(app.result(), "= 5");
    assert_eq!(
        load_history(&store).unwrap(),
        vec!["2 + 3  = 5".to_string()]
    );
}

#[test]
fn submit_line_keeps_multi_statement_scripts_as_one_history_entry() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    app.submit_line(
        "x = 10; x + 5",
        &store,
        &Localizer::resolve(Some("en"), &[]),
    );
    // One entry, semicolons intact, with the last answer appended;
    // the result area shows every answer in order (ADR-0052).
    assert_eq!(app.result(), "= 10\n= 15");
    assert_eq!(
        load_history(&store).unwrap(),
        vec!["x = 10; x + 5  = 15".to_string()]
    );
    // `save script` persists the whole script the user entered.
    app.submit_line(
        "save script demo",
        &store,
        &Localizer::resolve(Some("en"), &[]),
    );
    let saved = store.list_scripts().unwrap();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].name, "demo");
    assert_eq!(saved[0].source, "x = 10; x + 5");
}

#[test]
fn submit_line_keeps_mixed_graph_scripts_as_one_history_entry() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    app.submit_line(
        "graph x ^ 2; 2 + 2",
        &store,
        &Localizer::resolve(Some("en"), &[]),
    );
    assert_eq!(app.result(), "= 4");
    assert_eq!(
        load_history(&store).unwrap(),
        vec!["graph x ^ 2; 2 + 2  = 4".to_string()]
    );
}

#[test]
fn submit_line_reports_the_new_language() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    let new_lang = app.submit_line("language fr", &store, &Localizer::resolve(Some("en"), &[]));
    assert_eq!(new_lang, Some("fr".to_string()));
    assert_eq!(plain(app.result().to_string()), "language set to fr");
    assert_eq!(load_language(&store).unwrap(), Some("fr".to_string()));
}

#[test]
fn submit_line_keeps_graph_special_case() {
    let mut app = App::default();
    let (store, _keep) = scratch_store();
    app.submit_line("graph x ^ 2", &store, &Localizer::resolve(Some("en"), &[]));
    // ADR-0027: graphing prints nothing to the answer line.
    assert_eq!(app.result(), "");
    assert_eq!(app.graph().len(), 1);
    // Graph commands join the history list like every submitted line.
    assert_eq!(app.history(), ["graph x ^ 2".to_string()]);
}

// ===== 3D surfaces and animation (ADR-0015) =====

fn tui_store() -> epher_store::DocStore<epher_store::FsStore> {
    let dir = std::env::temp_dir().join(format!("epher-tui-test-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    epher_store::DocStore::new(epher_store::FsStore::new(dir))
}

#[test]
fn graph3d_samples_and_clears() {
    let store = tui_store();
    let mut app = App::with_session(epher_core::Session::new());
    app.submit_line(
        "graph3d x ^ 2 + y ^ 2",
        &store,
        &epher_i18n::Localizer::resolve(Some("en"), &[]),
    );
    assert_eq!(app.result(), "");
    assert_eq!(app.surfaces().len(), 1);
    assert_eq!(app.history(), ["graph3d x ^ 2 + y ^ 2".to_string()]);

    // A second surface overlays.
    app.submit_line(
        "graph3d x - y",
        &store,
        &epher_i18n::Localizer::resolve(Some("en"), &[]),
    );
    assert_eq!(app.surfaces().len(), 2);

    app.submit_line(
        "graph3d clear",
        &store,
        &epher_i18n::Localizer::resolve(Some("en"), &[]),
    );
    assert!(app.surfaces().is_empty());
}

#[test]
fn graph3d_rejects_nonsense() {
    let store = tui_store();
    let mut app = App::with_session(epher_core::Session::new());
    app.submit_line(
        "graph3d x +",
        &store,
        &epher_i18n::Localizer::resolve(Some("en"), &[]),
    );
    assert!(app.result().starts_with("error"), "got: {:?}", app.result());
    assert!(app.surfaces().is_empty());
}

#[test]
fn arrows_rotate_the_view_and_pitch_is_clamped() {
    let store = tui_store();
    let mut app = App::with_session(epher_core::Session::new());
    app.submit_line(
        "graph3d x ^ 2 + y ^ 2",
        &store,
        &epher_i18n::Localizer::resolve(Some("en"), &[]),
    );
    let before = *app.view();
    app.rotate_view(0.15, 0.0);
    assert!((app.view().yaw - before.yaw - 0.15).abs() < 1e-9);
    for _ in 0..50 {
        app.rotate_view(0.0, 0.3);
    }
    assert!(app.view().pitch <= 1.4 + 1e-9);
}

#[test]
fn space_toggles_play_and_tick_animates_the_constant() {
    let store = tui_store();
    // A constant referenced by a curve is the animation target.
    let mut s = epher_core::Session::new();
    s.submit("const a = 1");
    let mut app = App::with_session(s);
    app.submit_line(
        "graph a * x ^ 2",
        &store,
        &epher_i18n::Localizer::resolve(Some("en"), &[]),
    );
    assert!(app.toggle_play());
    let play = app.play().unwrap().clone();
    assert_eq!(play.name, "a");
    assert_eq!(play.step, 0.1);

    // Each tick advances the constant and re-samples the curve.
    let before = app.graph()[0].samples.clone();
    app.tick();
    let after = app.graph()[0].samples.clone();
    let v: f64 = match app_session_constant(&app, "a") {
        epher_core::Value::Float(f) => f,
        _ => panic!("constant a must be a float"),
    };
    assert!((v - 1.1).abs() < 1e-9);
    // The curve resampled: samples at a=1.1 differ from a=1.
    assert!(before
        .iter()
        .zip(&after)
        .any(|(x, y)| (x.y - y.y).abs() > 1e-9));

    // Toggling again stops.
    assert!(!app.toggle_play());
    assert!(app.play().is_none());
}

#[test]
fn tick_wraps_the_constant_within_its_play_bounds() {
    let store = tui_store();
    let mut s = epher_core::Session::new();
    s.submit("const a = 1");
    let mut app = App::with_session(s);
    app.submit_line(
        "graph a * x",
        &store,
        &epher_i18n::Localizer::resolve(Some("en"), &[]),
    );
    assert!(app.toggle_play());
    let lo = app.play().unwrap().lo;
    // 40 ticks of 0.1 from 1.0 with hi = 3.0: wrap back to lo = -1.0.
    for _ in 0..40 {
        app.tick();
    }
    let v = app_session_constant(&app, "a");
    let v = match v {
        epher_core::Value::Float(f) => f,
        _ => panic!(),
    };
    let hi = app.play().unwrap().hi;
    assert!(v >= lo && v <= hi);
}

#[test]
fn render_ascii3d_draws_the_wireframe() {
    use epher_core::graph::{sample_surface, View3D};
    let env = epher_core::Env::default();
    let s = sample_surface("x ^ 2 - y ^ 2", 20, &env).unwrap();
    let out = render_ascii3d(&[s], &View3D::default(), 40, 12);
    assert!(!out.is_empty());
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines.len(), 12);
    assert!(lines.iter().all(|l| l.chars().count() <= 40));
    // Depth glyphs and the frame are present.
    assert!(out.contains('*') || out.contains('+') || out.contains('.'));
    assert!(out.contains('o'));
}

#[test]
fn render_ascii3d_is_empty_without_surfaces() {
    assert_eq!(
        render_ascii3d(&[], &epher_core::graph::View3D::default(), 40, 12),
        ""
    );
}

#[test]
fn submit_line_splits_semicolon_statements() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    app.set_input("graph sin(x); graph cos(x)");
    app.submit_line(
        &app.input().to_string(),
        &store,
        &Localizer::resolve(Some("en"), &[]),
    );
    // both curves overlay the plot, each a separate statement (the event
    // loop, not submit_line, clears the input after Enter)
    assert_eq!(app.graph().len(), 2);
}

#[test]
fn submit_line_skips_empty_semicolon_pieces() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    app.set_input("2 + 3;;;");
    app.submit_line(
        &app.input().to_string(),
        &store,
        &Localizer::resolve(Some("en"), &[]),
    );
    assert_eq!(app.result(), "= 5");
    assert_eq!(app.history().len(), 1);
}

// --- keypad mode (ADR-0016) ---

#[test]
fn keypad_insert_appends_token() {
    let mut app = App::default();
    assert!(!app.keypad_focused());
    app.keypad_open();
    assert!(app.keypad_focused());
    // The digits bank is first, like the web's "123" tab.
    assert_eq!(app.keypad_bank(), "123");
    app.keypad_insert(); // (0,0) = C clears the input
    assert_eq!(app.input(), "");
    app.keypad_move(1, 0); // (1,0) = 7
    app.keypad_insert();
    assert_eq!(app.input(), "7");
    app.keypad_move(-1, 1); // (0,1) = ⌫
    app.keypad_insert();
    assert_eq!(app.input(), "");
}

#[test]
fn keypad_digits_bank_mirrors_the_web_tab() {
    let mut app = App::default();
    app.keypad_open();
    // The digits bank is frozen at its five rows (ADR-0042 amendment):
    // changes need the project owner's explicit approval.
    let digits = epher_tui::banks()[0].1;
    assert_eq!(digits.len(), 5);
    let flats: Vec<&str> = digits
        .iter()
        .flat_map(|r| r.iter())
        .map(|(d, _)| *d)
        .collect();
    assert_eq!(
        flats,
        [
            "C", "⌫", "(", ")", "÷", "7", "8", "9", "×", "−", "4", "5", "6", "+", "^", "1", "2",
            "3", ";", ",", "0", ".", "\u{23CE}", "="
        ]
    );
    // The newline key inserts a real newline (ADR-0016 amendment): the
    // entry composes multi-line scripts, and submit splits on them.
    app.keypad_set(4, 2);
    app.keypad_insert();
    assert_eq!(app.input(), "\n");
    app.clear_input();
    // % lives on the num bank (ADR-0042 amendment - the digits bank is
    // exactly full), and on an empty entry the auto-ans kicks in: an
    // operator continues from the answer.
    app.keypad_cycle(3); // 123 -> trig -> fn -> num
    assert_eq!(app.keypad_bank(), "num");
    app.keypad_set(3, 0);
    app.keypad_insert();
    assert_eq!(app.input(), "ans%");
    app.clear_input();
    app.keypad_cycle(-3); // back to digits for the keys below
                          // ÷/×/− display the glyphs but insert the language's ASCII tokens.
    assert_eq!(digits[0][4].1, "/");
    assert_eq!(digits[1][3].1, "*");
    assert_eq!(digits[1][4].1, "-");
    // ÷ on an empty entry continues from the answer (ADR-0042 auto-ans).
    app.keypad_set(0, 4);
    app.keypad_insert();
    assert_eq!(app.input(), "ans/");
    app.clear_input();
    app.push_char('x');
    app.keypad_set(0, 4);
    app.keypad_insert();
    assert_eq!(app.input(), "x/");
    // "=" marks the submit; C and ⌫ act, not insert.
    app.keypad_set(4, 3);
    assert!(app.keypad_is_submit());
    app.keypad_set(0, 0);
    app.keypad_insert();
    assert_eq!(app.input(), "");
}

#[test]
fn keypad_move_wraps_around_edges() {
    let mut app = App::default();
    app.keypad_open();
    app.keypad_move(0, -1); // from col 0 → col 4
    assert_eq!(app.keypad_col(), 4);
    app.keypad_move(-1, 0); // from row 0 → the digits bank's last row
    assert_eq!(app.keypad_row(), 4);
    assert_eq!(app.keypad_col(), 3, "the frozen last row has four keys");
    app.keypad_move(1, 0); // wraps back to row 0
    assert_eq!(app.keypad_row(), 0);
}

#[test]
fn keypad_banks_cycle_and_reset_the_highlight() {
    let mut app = App::default();
    app.keypad_open();
    assert_eq!(app.keypad_bank(), "123");
    app.keypad_move(2, 4); // somewhere inside
    app.keypad_cycle(1);
    assert_eq!(app.keypad_bank(), "trig");
    assert_eq!((app.keypad_row(), app.keypad_col()), (0, 0));
    app.keypad_cycle(-1); // back to digits, wrapping through the front
    assert_eq!(app.keypad_bank(), "123");
    app.keypad_cycle(-1); // …and on to the last bank
    assert_eq!(app.keypad_bank(), "var");
    app.keypad_insert(); // (0,0) of var = pi
    assert_eq!(app.input(), "pi");
}

#[test]
fn keypad_close_clears_focus_state() {
    let mut app = App::default();
    app.keypad_open();
    assert!(app.keypad_focused());
    app.keypad_close();
    assert!(!app.keypad_focused());
}

#[test]
fn keypad_has_the_graph_commands() {
    let mut app = App::default();
    app.keypad_open();
    for _ in 0..6 {
        app.keypad_cycle(1); // 123 → trig → fn → num → 0x → astro → var
    }
    app.keypad_move(1, 2); // row 1, col 2 of the var bank
    app.keypad_insert();
    assert_eq!(app.input(), "graph ");
}

#[test]
fn keypad_covers_every_function_that_was_missing() {
    // The banks grew to the full language (ADR-0019): every function,
    // constant, and command from the guide's reference must be
    // reachable from the keypad.
    let mut tokens = Vec::new();
    for bank in epher_tui::banks() {
        for row in bank.1 {
            for (disp, token) in *row {
                let _ = disp;
                tokens.push(token.trim_end_matches('(').trim());
            }
        }
    }
    for name in [
        "asin", "acos", "atan", "sinh", "cosh", "tanh", "asinh", "acosh", "atanh", "deg", "rad",
        "atan2", "exp", "log2", "logb", "cbrt", "root", "hypot", "trunc", "sign", "min", "max",
        "gcd", "lcm", "mod", "fact", "ncr", "npr", "sum", "product", "mean", "median", "variance",
        "stdev", "frac", "dec", "big", "bin", "oct", "hex", "phi", "x", "t", "ans", "graph",
        "graph3d", "table", "sin", "cos", "tan", "ln", "log", "sqrt", "abs", "floor", "ceil",
        "round", "pi", "e", "tau",
        // the keypad's clear/history KEYS are gone (ADR-0016 amendment);
        // the commands stay in the language, tested in submit tests
        // the condensed astro bank (ADR-0037; the full set lives on the
        // web keypad's Astro tab)
        "jd", "now", "lst", "kepler", "ra", "decl", "dist", "alt", "mag", "rise", "set", "illum",
        "diam", "delta_t", "airmass", "dawes", "dist_mod", "mag2jy", "hms2deg", "solar3d", "az",
        "transit", "phase", "mjd", "deg2hms",
    ] {
        assert!(tokens.contains(&name), "the keypad is missing {name}");
    }
}

// ===== menus, themes, and file prompts (ADR-0017) =====

#[test]
fn poi_list_setting_toggles_and_persists() {
    let mut app = App::with_session(epher_core::Session::new());
    let (store, _keep) = scratch_store();
    assert!(app.poi_list(), "shown by default");
    app.toggle_pois();
    assert!(!app.poi_list());
    epher_store::persist::save_pois(&store, app.poi_list()).unwrap();
    assert_eq!(
        epher_store::persist::load_pois(&store).unwrap(),
        Some(false)
    );
    app.set_pois(true);
    assert!(app.poi_list());
}

#[test]
fn theme_command_sets_and_persists_the_theme() {
    let mut app = App::with_session(epher_core::Session::new());
    let (store, _keep) = scratch_store();
    let localizer = epher_i18n::Localizer::resolve(Some("en"), &[]);
    app.submit_line("theme night", &store, &localizer);
    assert_eq!(app.theme(), epher_tui::Theme::Night);
    assert_eq!(
        epher_store::persist::load_theme(&store).unwrap().as_deref(),
        Some("night")
    );
    app.submit_line("theme bogus", &store, &localizer);
    assert_eq!(
        app.theme(),
        epher_tui::Theme::Night,
        "bad theme must not change the palette"
    );
}

#[test]
fn menu_navigation_and_actions() {
    let mut app = App::with_session(epher_core::Session::new());
    app.menu_open(0);
    assert_eq!(app.menu_active(), Some((0, 0)));
    app.menu_move(0, 1);
    assert_eq!(app.menu_active(), Some((0, 1)));
    assert_eq!(app.menu_activate(), Some(epher_tui::MenuAction::OpenScript));
    assert_eq!(app.menu_active(), None);

    // File ends with Quit (ADR-0023, ADR-0025): five items now — open
    // history, open script, save history, save script, quit.
    assert_eq!(app.menu_len(0), 5);
    app.menu_open(0);
    app.menu_move(0, 4);
    assert_eq!(app.menu_activate(), Some(epher_tui::MenuAction::Quit));
    assert_eq!(app.menu_active(), None);

    // Right arrow from the last menu (Settings) wraps to the first.
    app.menu_open(4);
    app.menu_move(1, 0);
    assert_eq!(app.menu_active(), Some((0, 0)));
    // Graph menu: clear graph, then copy points of interest (ADR-0038
    // amendment - the menu spelling of the web heading's copy button).
    app.menu_open(2);
    assert_eq!(app.menu_len(2), 2);
    assert_eq!(app.menu_activate(), Some(epher_tui::MenuAction::ClearGraph));
    app.menu_open(2);
    app.menu_move(0, 1);
    assert_eq!(app.menu_activate(), Some(epher_tui::MenuAction::CopyPois));
    // Help sits ABOVE Settings (ADR-0038 amendment): slot 3 is the
    // guide plus the keypad key help (ADR-0039), slot 4 the settings.
    app.menu_open(3);
    assert_eq!(app.menu_len(3), 3);
    assert_eq!(app.menu_activate(), Some(epher_tui::MenuAction::OpenGuide));
    // Settings moved to slot 4: the POI-list checkbox (ADR-0019), then
    // theme radios, then languages, then the result-display rows
    // (ADR-0043).
    app.menu_open(4);
    assert_eq!(app.menu_len(4), 15);
    assert_eq!(app.menu_activate(), Some(epher_tui::MenuAction::TogglePois));
    app.menu_open(4);
    app.menu_move(0, 1);
    assert_eq!(
        app.menu_activate(),
        Some(epher_tui::MenuAction::SetTheme("light"))
    );
    app.menu_open(4);
    for _ in 0..5 {
        app.menu_move(0, 1);
    }
    assert_eq!(
        app.menu_activate(),
        Some(epher_tui::MenuAction::SetLanguage("zh-CN"))
    );

    // Typing a character dismisses the menu (the event loop calls
    // menu_close before push_char; here we check the state transitions
    // the loop relies on).
    app.menu_open(0);
    app.menu_close();
    assert_eq!(app.menu_active(), None);
}

#[test]
fn clear_graph_empties_the_pane() {
    let dir = std::env::temp_dir().join(format!("epher-tui-clear-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let store = epher_store::DocStore::new(epher_store::FsStore::new(&dir));
    let localizer = epher_i18n::Localizer::resolve(None, &[]);
    let mut app = App::with_session(epher_core::Session::new());
    app.submit_line("graph x ^ 2", &store, &localizer);
    assert!(!app.graph().is_empty());
    app.clear_graph();
    assert!(app.graph().is_empty());
    assert!(app.surfaces().is_empty());
    // The menu spelling and the typed commands agree.
    app.submit_line("graph x ^ 3", &store, &localizer);
    assert!(!app.graph().is_empty());
    app.submit_line("graph clear", &store, &localizer);
    assert!(app.graph().is_empty());
}

#[test]
fn guide_loads_from_disk_when_opened_and_miss_reports_paths() {
    // The guide text comes from the installed files at open (ADR-0053);
    // the binary carries none of it. EPHER_GUIDE_DIR points the loader
    // at the repo's site/guide for the test.
    let src = guide_source_dir();
    std::env::set_var("EPHER_GUIDE_DIR", &src);
    let mut app = App::with_session(epher_core::Session::new());
    app.guide_open();
    app.guide_load("en");
    let (_, md) = app.guide_text().expect("guide text loaded");
    assert!(md.as_deref().unwrap().contains("epher user guide"));
    // A language change reloads from that locale's file.
    app.guide_load("de");
    let (locale, md) = app.guide_text().expect("german guide loaded");
    assert_eq!(locale, "de");
    assert!(!md.as_deref().unwrap().is_empty());
    // No installed files: the pager can say where it looked.
    let empty = std::env::temp_dir().join(format!("epher-tui-guide-miss-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&empty);
    std::env::set_var("EPHER_GUIDE_DIR", &empty);
    let mut app = App::with_session(epher_core::Session::new());
    app.guide_open();
    app.guide_load("en");
    let (_, md) = app.guide_text().expect("the miss is recorded, not a panic");
    assert!(md.is_err());
    std::env::remove_var("EPHER_GUIDE_DIR");
}

#[test]
fn guide_view_opens_scrolls_and_closes() {
    let mut app = App::with_session(epher_core::Session::new());
    assert!(!app.guide_active());
    app.guide_open();
    assert!(app.guide_active());
    assert_eq!(app.guide_offset(), Some(0));
    app.guide_scroll(5);
    assert_eq!(app.guide_offset(), Some(5));
    app.guide_scroll(-20); // clamps at zero
    assert_eq!(app.guide_offset(), Some(0));
    app.guide_scroll_to(usize::MAX); // End: clamped to content at draw time
    assert_eq!(app.guide_offset(), Some(usize::MAX));
    app.guide_close();
    assert!(!app.guide_active());
    // The installed guide renders in every interface language: the
    // loader reads the repo's site/guide/*.md when EPHER_GUIDE_DIR
    // points at it (the content is not compiled in, ADR-0053).
    let src = guide_source_dir();
    for l in epher_i18n::SUPPORTED_LOCALES {
        let md = std::fs::read_to_string(src.join(epher_guide::file_name(l)))
            .expect("guide source next to the repo");
        let lines = epher_guide::render_text(&md);
        assert!(!lines.is_empty());
        assert!(lines
            .iter()
            .any(|t| matches!(t, epher_guide::TLine::Heading(1, _))));
    }
}

/// The repo's site/guide directory from the test's working directory
/// (crates/tui during cargo test).
fn guide_source_dir() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("../../site/guide");
    p
}

#[test]
fn file_prompt_saves_and_opens() {
    let mut app = App::with_session(epher_core::Session::new());
    app.set_input("1+1");
    let dir = std::env::temp_dir().join(format!("epher-tui-menu-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("script.epher");

    let localizer = epher_i18n::Localizer::resolve(None, &[]);
    app.prompt_start(epher_tui::PromptKind::SaveScript);
    // ADR-0027: the save prompt pre-fills the default file name; the
    // buffer stays fully editable.
    assert_eq!(
        app.prompt_active(),
        Some((epher_tui::PromptKind::SaveScript, "epher-script.esr"))
    );
    while app.prompt_active().is_some_and(|(_, b)| !b.is_empty()) {
        app.prompt_pop();
    }
    for c in path.to_string_lossy().chars() {
        app.prompt_push(c);
    }
    assert_eq!(
        app.prompt_submit(&localizer),
        None,
        "save must succeed and close the prompt"
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "1+1");

    // Opening a script loads the file into the input.
    app.set_input("");
    app.prompt_start(epher_tui::PromptKind::OpenScript);
    for c in path.to_string_lossy().chars() {
        app.prompt_push(c);
    }
    assert_eq!(app.prompt_submit(&localizer), None);
    assert_eq!(app.input(), "1+1");

    // Opening a history file replaces the history section without
    // executing anything (ADR-0025).
    let hist_path = dir.join("history.epher");
    std::fs::write(&hist_path, "2 + 2\n\nhello = 5\n").unwrap();
    app.set_input("keep me");
    app.prompt_start(epher_tui::PromptKind::OpenHistory);
    for c in hist_path.to_string_lossy().chars() {
        app.prompt_push(c);
    }
    assert_eq!(app.prompt_submit(&localizer), None);
    assert_eq!(app.history(), &["2 + 2", "hello = 5"]);
    assert_eq!(
        app.input(),
        "keep me",
        "history load must not touch the entry"
    );
    assert!(
        app.result().contains('2'),
        "loaded count in: {}",
        app.result()
    );

    // A missing path fails and keeps the prompt (with its text) open.
    app.prompt_start(epher_tui::PromptKind::OpenScript);
    for c in "/nonexistent/nope.epher".chars() {
        app.prompt_push(c);
    }
    assert_eq!(
        app.prompt_submit(&localizer),
        Some(epher_tui::PromptKind::OpenScript)
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn osc52_copy_frames_base64() {
    let encoded = epher_tui::base64_for_osc52(b"graph x ^ 2");
    // RFC 4648 of that payload, sanity-checked against the known value.
    assert_eq!(encoded, "Z3JhcGggeCBeIDI=");
}

#[test]
fn save_history_writes_the_session_history() {
    let mut app = App::with_session(epher_core::Session::new());
    let (store, _keep) = scratch_store();
    let localizer = epher_i18n::Localizer::resolve(Some("en"), &[]);
    app.submit_line("1+1", &store, &localizer);
    app.submit_line("graph x", &store, &localizer);
    let dir = std::env::temp_dir().join(format!("epher-tui-hist-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("history.epher");
    app.prompt_start(epher_tui::PromptKind::SaveHistory);
    assert_eq!(
        app.prompt_active(),
        Some((epher_tui::PromptKind::SaveHistory, "epher-history.ehs"))
    );
    while app.prompt_active().is_some_and(|(_, b)| !b.is_empty()) {
        app.prompt_pop();
    }
    for c in path.to_string_lossy().chars() {
        app.prompt_push(c);
    }
    assert_eq!(app.prompt_submit(&localizer), None);
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("1+1  = 2"), "history file: {saved:?}");
    assert!(saved.contains("graph x"), "history file: {saved:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn multiline_history_entries_survive_save_open_and_pick() {
    // A multi-line script is ONE history entry (ADR-0027 amendment);
    // record it exactly as the web submit path does.
    let script = "x = 10\ny  =  x + 5\ny ^ 2";
    let mut session = epher_core::Session::new();
    session.record(script);
    let mut app = App::with_session(session);
    let (store, _keep) = scratch_store();
    let localizer = epher_i18n::Localizer::resolve(Some("en"), &[]);
    app.submit_line("1+1", &store, &localizer);
    assert_eq!(app.history(), &[script, "1+1  = 2"]);

    // The pick loads the whole script as one `; `-joined line (the TUI
    // input is one row; `;` is the same separator).
    let picked = app.history_pick_display(1).expect("multi-line entry");
    assert_eq!(picked, "x = 10; y  =  x + 5; y ^ 2");

    // Save → open round trip: the newline is escaped so the file keeps
    // its one-line-per-entry shape, and loading restores it.
    let dir = std::env::temp_dir().join(format!("epher-tui-ml-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("history.epher");
    app.prompt_start(epher_tui::PromptKind::SaveHistory);
    while app.prompt_active().is_some_and(|(_, b)| !b.is_empty()) {
        app.prompt_pop();
    }
    for c in path.to_string_lossy().chars() {
        app.prompt_push(c);
    }
    assert_eq!(app.prompt_submit(&localizer), None);
    let saved = std::fs::read_to_string(&path).unwrap();
    assert_eq!(saved, "x = 10\\ny  =  x + 5\\ny ^ 2\n1+1  = 2");

    app.clear_history();
    app.prompt_start(epher_tui::PromptKind::OpenHistory);
    for c in path.to_string_lossy().chars() {
        app.prompt_push(c);
    }
    assert_eq!(app.prompt_submit(&localizer), None);
    assert_eq!(app.history(), &[script, "1+1  = 2"]);
    let _ = std::fs::remove_dir_all(&dir);
}

// ===== SVG export from the TUI (ADR-0020) =====

#[test]
fn graph_save_writes_the_same_document_as_the_web_app() {
    let mut app = App::default();
    let (store, _keep) = scratch_store();
    let en = Localizer::resolve(Some("en"), &[]);
    app.submit_line("graph x ^ 2 - 1", &store, &en);
    let svg = _keep.path().join("tui.svg");
    app.submit_line(&format!("graph save {}", svg.display()), &store, &en);
    assert!(
        app.result().contains(svg.display().to_string().as_str()),
        "{}",
        app.result()
    );
    let doc = std::fs::read_to_string(&svg).unwrap();
    assert!(doc.starts_with("<svg "), "{doc}");
    assert!(doc.contains("y = x ^ 2 - 1"), "{doc}");
    // the shared renderer, self-contained like the copy button's output
    assert!(doc.contains("<style>"), "{doc}");
    assert!(doc.contains("viewBox=\"0 0 640 400\""), "{doc}");
}

#[test]
fn graph_save_without_a_plot_or_path_names_the_problem() {
    let mut app = App::default();
    let (store, _keep) = scratch_store();
    let en = Localizer::resolve(Some("en"), &[]);
    app.submit_line("graph save", &store, &en);
    assert_eq!(app.result(), "Name a file to save to");
    let svg = _keep.path().join("empty.svg");
    app.submit_line(&format!("graph save {}", svg.display()), &store, &en);
    assert_eq!(app.result(), "Nothing is plotted");
    assert!(!svg.exists());
}

#[test]
fn graph3d_save_writes_the_orbit_pose() {
    let mut app = App::default();
    let (store, _keep) = scratch_store();
    let en = Localizer::resolve(Some("en"), &[]);
    app.submit_line("graph3d x ^ 2 - y ^ 2", &store, &en);
    app.rotate_view(0.5, 0.2);
    let svg = _keep.path().join("tui3d.svg");
    app.submit_line(&format!("graph3d save {}", svg.display()), &store, &en);
    assert!(
        app.result().contains(svg.display().to_string().as_str()),
        "{}",
        app.result()
    );
    let doc = std::fs::read_to_string(&svg).unwrap();
    assert!(doc.contains("viewBox=\"0 0 640 400\""), "{doc}");
    assert!(doc.contains("transform=\"translate("), "{doc}");
}

// ===== History focus (ADR-0027) =====

#[test]
fn history_focus_moves_and_picks_without_running() {
    let mut app = App::with_session(epher_core::Session::with_history(vec![
        "1 + 1".to_string(),
        "2 + 2".to_string(),
        "graph x ^ 2".to_string(),
    ]));
    // Not focused by default; Tab's first stop is the keypad, and the
    // second opens the history list.
    assert!(!app.history_focused());
    app.history_open();
    assert!(app.history_focused());
    assert_eq!(app.history_sel(), 0);
    // Display order is newest-first; Down wraps to the oldest line and
    // Up wraps back around.
    app.history_move(-1);
    assert_eq!(app.history_sel(), 2);
    app.history_move(1);
    assert_eq!(app.history_sel(), 0);
    app.history_move(1);
    assert_eq!(app.history_sel(), 1);
    // Picking loads the line into the input (replacing), leaves focus,
    // and does NOT evaluate it.
    app.set_input("leftover");
    let picked = app.history_pick().unwrap();
    assert_eq!(picked, "2 + 2");
    // The event loop applies the pick to the input.
    app.set_input(&picked);
    assert_eq!(app.input(), "2 + 2");
    assert!(!app.history_focused());
    assert!(app.result().is_empty(), "picking must not run the line");

    // Empty history: picking is None and focus ends.
    app.clear_history();
    app.history_open();
    assert!(app.history_pick().is_none());
    assert!(!app.history_focused());
}

/// ADR-0031: the Settings menu grows the three 3D fine-control rows only
/// while surfaces are displayed, and the rows are adjusted with
/// Left/Right ±0.1, clamped to −1..1 (the web sliders' range and step).
#[test]
fn view_fine_controls_appear_with_3d_and_nudge_within_range() {
    let store = tui_store();
    let mut app = App::with_session(epher_core::Session::new());
    assert_eq!(app.menu_len(4), 15);
    app.submit_line(
        "graph3d x ^ 2 - y ^ 2",
        &store,
        &epher_i18n::Localizer::resolve(Some("en"), &[]),
    );
    // a 3D surface, the fine controls add three rows
    assert_eq!(app.menu_len(4), 18);
    app.menu_open(4);
    app.menu_move(0, 15);
    assert_eq!(app.menu_view_item(), Some(15));
    assert_eq!(app.view_offsets(), (0.0, 0.0, 0.0));
    app.nudge_view_offset(epher_tui::ViewAxis::Horizontal, 0.1);
    assert_eq!(app.view_offsets(), (0.1, 0.0, 0.0));
    // The effective pose folds the offsets into the orbit base: +1
    // horizontal = a full π of yaw, +1 zoom halves the camera distance.
    let eff = app.effective_view();
    let base = epher_core::graph::View3D::default();
    assert!((eff.yaw - (base.yaw + 0.1 * std::f64::consts::PI)).abs() < 1e-12);
    // Clamp at both ends.
    for _ in 0..30 {
        app.nudge_view_offset(epher_tui::ViewAxis::Zoom, 0.1);
    }
    assert_eq!(app.view_offsets().2, 1.0);
    for _ in 0..30 {
        app.nudge_view_offset(epher_tui::ViewAxis::Zoom, -0.1);
    }
    assert_eq!(app.view_offsets().2, -1.0);
    // Activating a fine-control row keeps the menu open (Enter is not an
    // action for these rows; Left/Right is the gesture).
    app.menu_open(4);
    app.menu_move(0, 15);
    assert_eq!(app.menu_activate(), None);
    assert_eq!(app.menu_active(), Some((4, 15)));
    // A fresh 3D graph drawn into an empty pane resets the controls.
    app.reset_view_offsets();
    assert_eq!(app.view_offsets(), (0.0, 0.0, 0.0));
    // Clearing the graph resets them too.
    app.nudge_view_offset(epher_tui::ViewAxis::Vertical, 0.5);
    app.clear_graph();
    assert_eq!(app.view_offsets(), (0.0, 0.0, 0.0));
}

/// ADR-0031: a history pick loads the expression, without the recorded
/// answer suffix — the user can edit and re-run it directly.
#[test]
fn history_pick_drops_the_answer_suffix() {
    let mut app = App::with_session(Session::with_history(vec![
        "2 + 2  = 4".to_string(),
        "x = 10; x + 5  = 15".to_string(),
        "graph x ^ 2".to_string(),
    ]));
    app.history_open();
    // Newest first: the graph line, then the script, then 2 + 2.
    app.history_move(1);
    assert_eq!(app.history_pick(), Some("x = 10; x + 5".to_string()));
    app.history_open();
    app.history_move(2);
    assert_eq!(app.history_pick(), Some("2 + 2".to_string()));
    app.history_open();
    assert_eq!(app.history_pick(), Some("graph x ^ 2".to_string()));
}

// --- mouse support (ADR-0034) ---

#[test]
fn mouse_2d_pan_zoom_and_reset_follow_the_viewport() {
    let mut app = App::default();
    app.submit_graph("x").expect("graph x");
    let auto = app.graph2d_effective().expect("auto ranges");
    assert_eq!(app.graph2d_view(), None);
    // Panning right by one cell moves the window left through the data:
    // the span is unchanged, both bounds shift by -span / width.
    let (x_min, x_max, y_min, y_max) = auto;
    app.graph2d_pan(1.0, 0.0, 30, 10);
    let panned = app.graph2d_view().expect("panned view");
    let dx = -(x_max - x_min) / 30.0;
    assert!((panned.0 - (x_min + dx)).abs() < 1e-9);
    assert!((panned.1 - (x_max + dx)).abs() < 1e-9);
    assert_eq!(panned.2, y_min);
    assert_eq!(panned.3, y_max);
    // Zooming halves the spans around the center.
    let (px_min, px_max, py_min, py_max) = panned;
    app.graph2d_zoom(0.5);
    let zoomed = app.graph2d_view().expect("zoomed view");
    let cx = (px_min + px_max) / 2.0;
    let cy = (py_min + py_max) / 2.0;
    assert!((zoomed.0 - (cx - (px_max - px_min) / 4.0)).abs() < 1e-9);
    assert!((zoomed.2 - (cy - (py_max - py_min) / 4.0)).abs() < 1e-9);
    // Reset drops the override; a new graph re-fits too.
    app.graph2d_reset();
    assert_eq!(app.graph2d_view(), None);
    app.graph2d_pan(1.0, 0.0, 30, 10);
    assert!(app.graph2d_view().is_some());
    app.submit_graph("x + 1").expect("graph x + 1");
    assert_eq!(app.graph2d_view(), None);
    // Clearing the graph drops the override as well.
    app.graph2d_pan(1.0, 0.0, 30, 10);
    app.submit_graph("clear").expect("clear");
    assert_eq!(app.graph2d_view(), None);
}

#[test]
fn mouse_keypad_clicks_select_banks_and_cells() {
    let mut app = App::default();
    app.keypad_select_bank(6);
    assert_eq!(app.keypad_bank(), "var");
    assert_eq!((app.keypad_row(), app.keypad_col()), (0, 0));
    // A click on (1, 2) of the var bank inserts exactly that token.
    let expected = epher_tui::banks()[6].1[1][2].1;
    app.keypad_set(1, 2);
    app.keypad_insert();
    assert_eq!(app.input(), expected);
    // Clicks clamp to the clicked bank's grid: row 99 lands on the last
    // row, column 99 on that row's last cell.
    app.keypad_set(99, 99);
    let bank = epher_tui::banks()[6].1;
    assert_eq!(app.keypad_row(), bank.len() - 1);
    let last_len = bank[bank.len() - 1].len();
    assert_eq!(app.keypad_col(), last_len - 1);
}

#[test]
fn mouse_history_click_picks_the_expression_only() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    app.submit_line("1/0", &store, &Localizer::resolve(Some("en"), &[]));
    app.submit_line("2 + 2", &store, &Localizer::resolve(Some("en"), &[]));
    // Display row 0 is the newest line; its answer suffix must stay out
    // of the input (ADR-0031 + ADR-0034).
    let picked = app.history_pick_display(0).expect("newest line");
    assert_eq!(picked, "2 + 2");
    // The older line (display row 1) keeps its suffix stripped too.
    let older = app.history_pick_display(1).expect("older line");
    assert!(!older.contains("error"));
    // Out of range: nothing picked, input unchanged.
    assert_eq!(app.history_pick_display(99), None);
}

#[test]
fn mouse_double_click_resets_the_3d_pose() {
    let mut app = App::default();
    use epher_core::graph::View3D;
    app.submit_surface("x ^ 2 - y ^ 2").expect("surface");
    app.rotate_view(1.0, -0.5);
    app.view_set_camera(12.0);
    assert_ne!(app.view(), &View3D::default());
    app.view_reset_pose();
    assert_eq!(app.view(), &View3D::default());
}

/// ADR-0034: panning/zooming moves samples outside the window — the
/// renderer must clip them instead of indexing out of bounds.
#[test]
fn render_ascii_clips_samples_outside_the_viewport() {
    let expr = epher_core::parse("x").unwrap();
    let c = epher_core::graph::SampledCurve {
        source: "x".to_string(),
        kind: epher_core::graph::CurveKind::Cartesian(expr),
        domain: (0.0, 2.0),
        samples: vec![
            Sample { x: 0.0, y: 0.0 },
            Sample { x: 1.0, y: 1.0 },
            Sample { x: 2.0, y: 2.0 },
        ],
        fill: None,
    };
    // A viewport far from the data draws an empty grid, no panic.
    let far = render_ascii(&[c.clone()], 5, 5, Some((100.0, 101.0, 100.0, 101.0)));
    assert!(!far.is_empty());
    // A zoomed window around the middle draws the curve inside it.
    let near = render_ascii(&[c], 5, 5, Some((0.8, 1.2, 0.8, 1.2)));
    assert!(near.contains('o'), "zoomed window: {near}");
}

// ===== The pane shows one kind at a time (ADR-0015 amendment) =====

#[test]
fn the_pane_shows_one_kind_at_a_time() {
    let store = tui_store();
    let localizer = epher_i18n::Localizer::resolve(Some("en"), &[]);
    let mut app = App::with_session(epher_core::Session::new());

    // 2D first, then 3D: the surface replaces the curves.
    app.submit_line("graph x ^ 2", &store, &localizer);
    assert_eq!(app.graph().len(), 1);
    app.submit_line("graph3d x ^ 2 + y ^ 2", &store, &localizer);
    assert!(app.graph().is_empty(), "drawing 3D clears the 2D curves");
    assert!(app.pois().is_empty());
    assert_eq!(app.surfaces().len(), 1);

    // and back: a 2D curve replaces the surfaces.
    app.submit_line("graph sin(x)", &store, &localizer);
    assert!(app.surfaces().is_empty(), "drawing 2D clears the surfaces");
    assert_eq!(app.graph().len(), 1);

    // same-kind overlays still accumulate.
    app.submit_line("graph cos(x)", &store, &localizer);
    assert_eq!(app.graph().len(), 2);
    app.submit_line("graph3d x - y", &store, &localizer);
    assert_eq!(app.surfaces().len(), 1);
    app.submit_line("graph3d x + y", &store, &localizer);
    assert_eq!(app.surfaces().len(), 2, "3D overlays still accumulate");

    // explicit clears stay kind-specific.
    app.submit_line("graph3d clear", &store, &localizer);
    assert!(app.surfaces().is_empty());
    app.submit_line("graph3d x - y", &store, &localizer);
    app.submit_line("graph clear", &store, &localizer);
    assert!(app.graph().is_empty());
    assert_eq!(app.surfaces().len(), 1, "graph clear leaves the 3D alone");
}

#[test]
fn a_failed_switch_keeps_the_previous_kind() {
    let store = tui_store();
    let localizer = epher_i18n::Localizer::resolve(Some("en"), &[]);
    let mut app = App::with_session(epher_core::Session::new());
    app.submit_line("graph x ^ 2", &store, &localizer);
    assert_eq!(app.graph().len(), 1);
    // a broken graph3d must not clear the 2D curves
    app.submit_line("graph3d x +", &store, &localizer);
    assert!(app.result().starts_with("error"), "got: {:?}", app.result());
    assert_eq!(app.graph().len(), 1, "a failed 3D keeps the curves");
}

// ===== The shared session snapshot (ADR-0010 amendment) =====

#[test]
fn session_bindings_persist_to_the_shared_store() {
    use epher_store::persist::session_bindings;
    let store = tui_store();
    let localizer = epher_i18n::Localizer::resolve(Some("en"), &[]);
    let mut app = App::with_session(epher_core::Session::new());
    app.set_input("x = 5");
    app.submit_line("x = 5", &store, &localizer);
    assert_eq!(app.result(), "= 5");
    let saved = session_bindings(&store).unwrap().unwrap();
    assert_eq!(saved.len(), 2, "x and ans: {saved:?}");
    assert_eq!(saved["x"], epher_core::Value::float(5.0));
    assert_eq!(saved["ans"], epher_core::Value::float(5.0));
}

// --- the entry's caret (ADR-0035 amendment, TUI) ----------------------

#[test]
fn set_input_puts_the_caret_at_the_end() {
    let mut app = App::default();
    app.set_input("2 + 3");
    assert_eq!(app.cursor(), 5);
}

#[test]
fn arrows_move_the_caret_without_touching_the_text() {
    let mut app = App::default();
    app.set_input("abc");
    app.cursor_move(-1);
    assert_eq!(app.cursor(), 2);
    app.cursor_move(-1);
    assert_eq!(app.cursor(), 1);
    app.cursor_move(1);
    assert_eq!(app.cursor(), 2);
    app.cursor_move(1);
    app.cursor_move(1);
    assert_eq!(app.cursor(), 3);
    app.cursor_move(-1);
    app.cursor_move(-1);
    app.cursor_move(-1);
    assert_eq!(app.cursor(), 0);
}

#[test]
fn typed_characters_insert_at_the_caret() {
    let mut app = App::default();
    app.set_input("ab");
    app.cursor_move(-1);
    app.push_char('X');
    assert_eq!(app.input(), "aXb");
    assert_eq!(app.cursor(), 2);
    // a selected-range press is not a TUI concept; the caret just moves on
    app.cursor_move(1);
    app.push_char('!');
    assert_eq!(app.input(), "aXb!");
}

#[test]
fn backspace_deletes_before_the_caret() {
    let mut app = App::default();
    app.set_input("abc");
    app.cursor_move(-1);
    app.pop_char();
    assert_eq!(app.input(), "ac");
    assert_eq!(app.cursor(), 1);
    // at the very start there is nothing to delete
    app.cursor_move(-1);
    app.pop_char();
    assert_eq!(app.input(), "ac");
}

#[test]
fn shift_enter_composes_a_multiline_script_in_the_entry() {
    let mut app = App::default();
    app.set_input("x = 1");
    app.push_char('\n');
    app.push_char('y');
    app.push_char(' ');
    app.push_char('=');
    app.push_char(' ');
    app.push_char('2');
    assert_eq!(app.input(), "x = 1\ny = 2");
    assert_eq!(app.cursor(), 11);
    // the caret rides the line structure
    assert_eq!(app.cursor_line_index(), 1);
}

#[test]
fn cursor_lines_move_between_lines_keeping_the_column() {
    let mut app = App::default();
    app.set_input("ab\ncde\nf");
    app.cursor_move(-1); // end of the last line ("f")
    assert_eq!(app.cursor_line_index(), 2);
    app.cursor_line(-1);
    assert_eq!(app.cursor_line_index(), 1);
    assert_eq!(app.cursor(), 3); // "cde" column 0
    app.cursor_line(-1);
    assert_eq!(app.cursor_line_index(), 0);
    assert_eq!(app.cursor(), 0); // "ab" column 0
    app.cursor_line(1);
    assert_eq!(app.cursor_line_index(), 1);
    app.cursor_line(1);
    assert_eq!(app.cursor_line_index(), 2);
}

#[test]
fn home_and_end_jump_to_the_lines_edges() {
    let mut app = App::default();
    app.set_input("12\n345");
    app.cursor_move(-1);
    app.cursor_move(-1);
    app.cursor_line_edge(-1);
    assert_eq!(app.cursor(), 3); // start of "345"
    app.cursor_line_edge(1);
    assert_eq!(app.cursor(), 6); // end of the input
    app.cursor_line_edge(-1);
    app.cursor_move(-1);
    app.cursor_line_edge(-1);
    assert_eq!(app.cursor(), 0); // start of "12"
}

#[test]
fn a_mouse_click_places_the_caret_by_line_and_column() {
    let mut app = App::default();
    app.set_input("alpha\nbeta");
    app.cursor_to(0, 2);
    assert_eq!(app.cursor(), 2);
    app.cursor_to(1, 3);
    assert_eq!(app.cursor(), 9);
    app.cursor_to(1, 99); // beyond the line: clamps to its end
    assert_eq!(app.cursor(), 10);
    app.cursor_to(99, 0); // beyond the text: end of the input
    assert_eq!(app.cursor(), 10);
}

#[test]
fn submitting_clears_the_caret_with_the_input() {
    let mut app = App::default();
    app.set_input("1 + 1");
    app.cursor_move(-1);
    app.submit();
    assert_eq!(app.cursor(), 0);
}

// ===== solar3d (ADR-0037 + the ADR-0015 amendment) =====

#[test]
fn solar3d_draws_the_scene_and_clears_other_kinds() {
    let (store, _keep) = scratch_store();
    let localizer = Localizer::resolve(Some("en"), &[]);
    let mut app = App::default();
    // a 2D curve first: drawing the solar system yields the pane
    app.set_input("graph sin(x)");
    app.submit_line(&app.input().to_string(), &store, &localizer);
    assert!(!app.graph().is_empty());
    app.set_input("solar3d jd(2020, 7, 1)");
    app.submit_line(&app.input().to_string(), &store, &localizer);
    assert!(app.graph().is_empty(), "one kind at a time");
    assert!(app.solar().is_some());
    // the scene's positioned dots appear in the ASCII render
    let text =
        epher_tui::render_solar_ascii(app.solar().expect("scene"), &app.effective_view(), 60, 20);
    assert!(text.contains('O'), "a positioned dot is stamped: {text}");
    // clear through the same grammar
    app.set_input("solar3d clear");
    app.submit_line(&app.input().to_string(), &store, &localizer);
    assert!(app.solar().is_none());
}

#[test]
fn solar3d_errors_and_save_follow_the_graph_voice() {
    let (store, _keep) = scratch_store();
    let localizer = Localizer::resolve(Some("en"), &[]);
    let mut app = App::default();
    app.set_input("solar3d bogus_name");
    app.submit_line(&app.input().to_string(), &store, &localizer);
    assert!(app.result().starts_with("error:"), "{}", app.result());
    app.set_input("solar3d jd(2020, 13, 1)");
    app.submit_line(&app.input().to_string(), &store, &localizer);
    assert!(app.result().starts_with("error:"));
    // the save path reports the empty pane before any scene is drawn
    app.set_input("solar3d save /tmp/never-epher-tui-solar.svg");
    app.submit_line(&app.input().to_string(), &store, &localizer);
    assert_eq!(app.result(), "Nothing is plotted");
}

// ===== keypad key help (ADR-0039) =====

#[test]
fn every_bank_token_has_a_hint_or_speaks_for_itself() {
    // Every non-digit key in every bank maps to a key-hint FTL message;
    // the digit keys 0-9 and "." are the self-evident exceptions.
    for (_label, rows) in epher_tui::banks() {
        for row in rows.iter() {
            for (disp, _insert) in row.iter() {
                let hint = epher_tui::keypad_hint_key(disp);
                if disp.chars().all(|c| c.is_ascii_digit()) || *disp == "." {
                    assert!(hint.is_none(), "{disp} needs no hint");
                } else {
                    assert!(hint.is_some(), "{disp} is missing a hint");
                    assert!(hint.unwrap().starts_with("key-hint-"));
                }
            }
        }
    }
}

#[test]
fn key_help_opens_scrolls_and_closes() {
    let mut app = App::default();
    app.keypad_open();
    app.keypad_cycle(1); // move off the digits bank
    assert_eq!(app.keypad_bank(), "trig");
    assert!(!app.key_help_active());
    app.key_help_open();
    assert!(app.key_help_active());
    assert_eq!(app.key_help_offset(), Some(0));
    app.key_help_scroll(3);
    assert_eq!(app.key_help_offset(), Some(3));
    app.key_help_scroll(-10);
    assert_eq!(app.key_help_offset(), Some(0));
    app.key_help_close();
    assert!(!app.key_help_active());
    assert_eq!(app.key_help_offset(), None);
}

#[test]
fn help_menu_lists_guide_and_key_help() {
    let mut app = App::default();
    assert_eq!(app.menu_len(3), 3);
    app.menu_open(3);
    app.menu_move(0, 1);
    // The activation runs through the loop's perform step; here the
    // action itself is the contract.
    assert_eq!(
        app.menu_activate(),
        Some(epher_tui::MenuAction::OpenKeyHelp)
    );
    app.key_help_open();
    assert!(app.key_help_active());
}

// --- the constants browser (ADR-0045) ---

#[test]
fn constants_browser_opens_selects_and_inserts() {
    let mut app = App::default();
    let localizer = epher_i18n::Localizer::resolve(Some("en"), &[]);
    assert!(!app.constants_active());
    app.constants_open(&localizer);
    assert!(app.constants_active());
    // The first group is Math, so the first row is the alphabetically
    // first builtin: e (Euler's number, groups are ordered, not the
    // byte-sorted catalog order).
    assert_eq!(app.constants_row_name(0), Some("e"));
    // Selection moves over the constant rows and clamps.
    app.constants_select(-5);
    assert_eq!(app.constants_selection(), Some(0));
    app.constants_select(1);
    assert_eq!(app.constants_selection(), Some(1));
    // Enter inserts the selected name into the input line.
    app.set_input("2 * ");
    for _ in 0..app.constants_selection().unwrap() {
        app.constants_select(-1);
    }
    app.constants_insert();
    assert_eq!(app.input(), "2 * e");
    assert!(!app.constants_active(), "inserting closes the browser");
}

#[test]
fn help_menu_lists_constants_browser() {
    let mut app = App::default();
    assert_eq!(app.menu_len(3), 3, "Help: guide, key help, constants");
    app.menu_open(3);
    app.menu_move(0, 2);
    assert_eq!(
        app.menu_activate(),
        Some(epher_tui::MenuAction::BrowseConstants)
    );
}

#[test]
fn paste_lands_whole_text_in_the_entry_as_one_unit() {
    let mut app = App::default();
    app.paste_text("1 +\n2");
    assert_eq!(app.input(), "1 +\n2");
    // The cursor sits after what was pasted, so a follow-up paste
    // appends at the caret like typing would.
    app.paste_text(" + 3");
    assert_eq!(app.input(), "1 +\n2 + 3");
}

#[test]
fn paste_normalizes_windows_line_endings() {
    let mut app = App::default();
    app.paste_text("2 + 2\r\n3 * 3\r4 ^ 2");
    assert_eq!(app.input(), "2 + 2\n3 * 3\n4 ^ 2");
}

#[test]
fn paste_does_not_inject_ans_for_a_leading_operator() {
    let mut app = App::default();
    // Typing "+" into an empty entry means "continue from the previous
    // answer" (ADR-0042); a paste is verbatim.
    app.paste_text("+1");
    assert_eq!(app.input(), "+1");
}

#[test]
fn pasted_script_runs_on_one_enter_like_the_gui_entry() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    // A miniature of the website's script text: banner comment, line
    // comment, blank lines, a print group. Pasted whole, one Enter
    // (the entry's submit path) runs it like the desktop and web
    // entries do — not line by line, which would die on the opening
    // unterminated `/*`.
    let script = "/* === banner ===\n   a comment block, newlines and all\n   === */\n\n// a line comment\nprint(\"half of ten:\", 10 / 2)\nprint(\"double of three:\", 3 * 2)\n";
    app.paste_text(script);
    let pasted = app.input().to_string();
    app.submit_line(&pasted, &store, &Localizer::resolve(Some("en"), &[]));
    assert!(
        app.result().contains("= half of ten: 5"),
        "{}",
        app.result()
    );
    assert!(
        app.result().contains("= double of three: 6"),
        "{}",
        app.result()
    );
    assert_eq!(
        app.history().len(),
        1,
        "one history entry for the whole paste"
    );
    assert!(app.input().is_empty(), "the entry is clean after the run");
}
