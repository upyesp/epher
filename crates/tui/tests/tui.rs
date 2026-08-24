use epher_core::Sample;
use epher_core::Session;
use epher_tui::{render_ascii, render_ascii3d, App};

fn app_session_constant(app: &App, name: &str) -> epher_core::Value {
    app.session()
        .env()
        .constant(name)
        .cloned()
        .unwrap_or(epher_core::Value::float(f64::NAN))
}

#[test]
fn submit_evaluates_against_persistent_env() {
    let mut app = App::default();
    app.set_input("x = 5; x + 1");
    app.submit();
    assert_eq!(app.result(), "= 6");
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
        .map(|(i, y)| Sample {
            x: i as f64,
            y: *y,
        })
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
    assert_eq!(render_ascii(&curves, 3, 3), "··o\n·o·\no··");
    let _ = samples;
}

#[test]
fn render_ascii_handles_empty_and_non_finite() {
    assert_eq!(render_ascii(&[], 3, 3), "");
    let expr = epher_core::parse("0").unwrap();
    let c = epher_core::graph::SampledCurve {
        source: "test".to_string(),
        kind: epher_core::graph::CurveKind::Cartesian(expr),
        domain: (0.0, 1.0),
        samples: vec![
            Sample { x: f64::NAN, y: 0.0 },
            Sample {
                x: 0.0,
                y: f64::INFINITY,
            },
            Sample { x: 1.0, y: 1.0 },
        ],
        fill: None,
    };
    let out = render_ascii(&[c], 3, 3);
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
    let out = render_ascii(&[c], 5, 5);
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
    let out = render_ascii(&[a, b], 4, 4);
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
    app.submit_line(&app.input().to_string(), &store, &Localizer::resolve(Some("en"), &[]));
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
    assert_eq!(load_history(&store).unwrap(), vec!["2 + 3  = 5".to_string()]);
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
    assert_eq!(app.result(), "graph: x ^ 2");
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
    app.submit_line("graph3d x ^ 2 + y ^ 2", &store, &epher_i18n::Localizer::resolve(Some("en"), &[]));
    assert_eq!(app.result(), "graph3d: x ^ 2 + y ^ 2");
    assert_eq!(app.surfaces().len(), 1);
    assert_eq!(app.history(), ["graph3d x ^ 2 + y ^ 2".to_string()]);

    // A second surface overlays.
    app.submit_line("graph3d x - y", &store, &epher_i18n::Localizer::resolve(Some("en"), &[]));
    assert_eq!(app.surfaces().len(), 2);

    app.submit_line("graph3d clear", &store, &epher_i18n::Localizer::resolve(Some("en"), &[]));
    assert!(app.surfaces().is_empty());
}

#[test]
fn graph3d_rejects_nonsense() {
    let store = tui_store();
    let mut app = App::with_session(epher_core::Session::new());
    app.submit_line("graph3d x +", &store, &epher_i18n::Localizer::resolve(Some("en"), &[]));
    assert!(app.result().starts_with("error"), "got: {:?}", app.result());
    assert!(app.surfaces().is_empty());
}

#[test]
fn arrows_rotate_the_view_and_pitch_is_clamped() {
    let store = tui_store();
    let mut app = App::with_session(epher_core::Session::new());
    app.submit_line("graph3d x ^ 2 + y ^ 2", &store, &epher_i18n::Localizer::resolve(Some("en"), &[]));
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
    app.submit_line("graph a * x ^ 2", &store, &epher_i18n::Localizer::resolve(Some("en"), &[]));
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
    assert!(before.iter().zip(&after).any(|(x, y)| (x.y - y.y).abs() > 1e-9));

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
    app.submit_line("graph a * x", &store, &epher_i18n::Localizer::resolve(Some("en"), &[]));
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
    assert_eq!(render_ascii3d(&[], &epher_core::graph::View3D::default(), 40, 12), "");
}

#[test]
fn submit_line_splits_semicolon_statements() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    app.set_input("graph sin(x); graph cos(x)");
    app.submit_line(&app.input().to_string(), &store, &Localizer::resolve(Some("en"), &[]));
    // both curves overlay the plot, each a separate statement (the event
    // loop, not submit_line, clears the input after Enter)
    assert_eq!(app.graph().len(), 2);
}

#[test]
fn submit_line_skips_empty_semicolon_pieces() {
    let (store, _keep) = scratch_store();
    let mut app = App::default();
    app.set_input("2 + 3;;;");
    app.submit_line(&app.input().to_string(), &store, &Localizer::resolve(Some("en"), &[]));
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
    app.keypad_insert(); // (0,0) = sin(
    assert_eq!(app.input(), "sin(");
    app.keypad_move(0, 1);
    app.keypad_insert(); // cos(
    assert_eq!(app.input(), "sin(cos(");
}

#[test]
fn keypad_move_wraps_around_edges() {
    let mut app = App::default();
    app.keypad_open();
    app.keypad_move(0, -1); // from col 0 → col 4
    assert_eq!(app.keypad_col(), 4);
    app.keypad_move(-1, 0); // from row 0 → the trig bank's last row
    assert_eq!(app.keypad_row(), 2);
    assert_eq!(app.keypad_col(), 4, "clamped to the ragged row's length");
    app.keypad_move(1, 0); // wraps back to row 0
    assert_eq!(app.keypad_row(), 0);
}

#[test]
fn keypad_banks_cycle_and_reset_the_highlight() {
    let mut app = App::default();
    app.keypad_open();
    assert_eq!(app.keypad_bank(), "trig");
    app.keypad_move(2, 4); // somewhere inside
    app.keypad_cycle(1);
    assert_eq!(app.keypad_bank(), "fn");
    assert_eq!((app.keypad_row(), app.keypad_col()), (0, 0));
    app.keypad_cycle(-1); // back to trig, wrapping through the front
    assert_eq!(app.keypad_bank(), "trig");
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
    for _ in 0..4 {
        app.keypad_cycle(1); // trig → fn → num → 0x → var
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
        "asin", "acos", "atan", "sinh", "cosh", "tanh", "asinh", "acosh", "atanh", "deg",
        "rad", "atan2", "exp", "log2", "logb", "cbrt", "root", "hypot", "trunc", "sign",
        "min", "max", "gcd", "lcm", "mod", "fact", "ncr", "npr", "sum", "product", "mean",
        "median", "variance", "stdev", "frac", "dec", "big", "bin", "oct", "hex", "phi",
        "x", "t", "ans", "graph", "graph3d", "table", "clear", "history", "sin", "cos",
        "tan", "ln", "log", "sqrt", "abs", "floor", "ceil", "round", "pi", "e", "tau",
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
    assert_eq!(epher_store::persist::load_pois(&store).unwrap(), Some(false));
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
    assert_eq!(epher_store::persist::load_theme(&store).unwrap().as_deref(), Some("night"));
    app.submit_line("theme bogus", &store, &localizer);
    assert_eq!(app.theme(), epher_tui::Theme::Night, "bad theme must not change the palette");
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
    assert_eq!(epher_tui::App::menu_len(0), 5);
    app.menu_open(0);
    app.menu_move(0, 4);
    assert_eq!(app.menu_activate(), Some(epher_tui::MenuAction::Quit));
    assert_eq!(app.menu_active(), None);

    // Right arrow from the last menu (Help) wraps to the first.
    app.menu_open(4);
    app.menu_move(1, 0);
    assert_eq!(app.menu_active(), Some((0, 0)));
    // Graph menu: exactly one item, clearing the graph (ADR-0018).
    app.menu_open(2);
    assert_eq!(epher_tui::App::menu_len(2), 1);
    assert_eq!(app.menu_activate(), Some(epher_tui::MenuAction::ClearGraph));
    // Settings moved to slot 3: the POI-list checkbox (ADR-0019), then
    // theme radios, then languages.
    app.menu_open(3);
    assert_eq!(epher_tui::App::menu_len(3), 12);
    assert_eq!(app.menu_activate(), Some(epher_tui::MenuAction::TogglePois));
    app.menu_open(3);
    app.menu_move(0, 1);
    assert_eq!(app.menu_activate(), Some(epher_tui::MenuAction::SetTheme("light")));
    app.menu_open(3);
    for _ in 0..5 {
        app.menu_move(0, 1);
    }
    assert_eq!(app.menu_activate(), Some(epher_tui::MenuAction::SetLanguage("zh-CN")));
    // Help menu: one item, the user guide.
    app.menu_open(4);
    assert_eq!(epher_tui::App::menu_len(4), 1);
    assert_eq!(app.menu_activate(), Some(epher_tui::MenuAction::OpenGuide));

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
    // The embedded guide renders in every interface language (the TUI's
    // own crate embeds the same site/guide/*.md as the website).
    for l in epher_i18n::SUPPORTED_LOCALES {
        let lines = epher_guide::render_text(epher_guide::guide(l));
        assert!(!lines.is_empty());
        assert!(lines.iter().any(|t| matches!(t, epher_guide::TLine::Heading(1, _))));
    }
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
    for c in path.to_string_lossy().chars() {
        app.prompt_push(c);
    }
    assert_eq!(app.prompt_submit(&localizer), None, "save must succeed and close the prompt");
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
    assert_eq!(app.input(), "keep me", "history load must not touch the entry");
    assert!(app.result().contains('2'), "loaded count in: {}", app.result());

    // A missing path fails and keeps the prompt (with its text) open.
    app.prompt_start(epher_tui::PromptKind::OpenScript);
    for c in "/nonexistent/nope.epher".chars() {
        app.prompt_push(c);
    }
    assert_eq!(app.prompt_submit(&localizer), Some(epher_tui::PromptKind::OpenScript));
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
    for c in path.to_string_lossy().chars() {
        app.prompt_push(c);
    }
    assert_eq!(app.prompt_submit(&localizer), None);
    let saved = std::fs::read_to_string(&path).unwrap();
    assert!(saved.contains("1+1  = 2"), "history file: {saved:?}");
    assert!(saved.contains("graph x"), "history file: {saved:?}");
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
    assert!(app.result().contains(svg.display().to_string().as_str()), "{}", app.result());
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
    assert!(app.result().contains(svg.display().to_string().as_str()), "{}", app.result());
    let doc = std::fs::read_to_string(&svg).unwrap();
    assert!(doc.contains("viewBox=\"0 0 640 400\""), "{doc}");
    assert!(doc.contains("transform=\"translate("), "{doc}");
}
