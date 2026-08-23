//! epher-web — the Yew frontend compiled to `wasm32-unknown-unknown`, shared by
//! the PWA and the Tauri desktop shell (ADR-0001).
//!
//! A thin component over the shared [`Session`]: input line, result, history,
//! and the graph panel (ADR-0006/0014 — the core samples and analyzes, this
//! file is presentation glue: curves, trace, points of interest, sliders).
//! Inside the desktop shell, persistence goes through the native store via
//! the Tauri IPC bridge (ADR-0010); in the browser, the session is the whole
//! state.

mod bridge;
pub mod graph;

use crate::graph::{Graph, Graph3D};
use bridge::{Bridge, InitState};
use epher_core::graph::{
    analyze, free_names, parse_graph_source, sample_spec, CurveKind, CurveSpec, InterestPoint,
    SampledCurve,
};
use epher_core::{Session, Value};
use epher_i18n::Localizer;
use epher_shell::{classify, message, prepare};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlTextAreaElement};
use yew::events::{InputEvent, SubmitEvent};
use yew::prelude::*;

/// Live graph-interaction state. The SVG's event listeners are attached
/// once, at mount, so the callbacks they hold must read current values —
/// but a cloned `UseStateHandle` reads the snapshot it was created with
/// (Yew replaces the handle's inner `Rc` on every `set`). This cell is the
/// live copy; the Yew states mirror it for rendering.
#[derive(Default)]
struct GraphLive {
    curves: Vec<SampledCurve>,
    trace: Option<graph::TracePoint>,
}

/// A running parameter animation (ADR-0015): the constant `name` steps by
/// `step` between `lo` and `hi`, wrapping around; `value` is the next value
/// to apply on the coming tick.
#[derive(Debug, Clone, PartialEq)]
struct PlaySpec {
    name: String,
    lo: f64,
    hi: f64,
    step: f64,
    value: f64,
    /// The 3D viewBox frozen at play start: while playing, the plot keeps
    /// this box so the layout (and the pause button) stay put.
    freeze: Option<String>,
}

impl PlaySpec {
    fn ticked(&self) -> PlaySpec {
        let mut next = self.value + self.step;
        if next > self.hi {
            next = self.lo;
        }
        PlaySpec {
            value: next,
            ..self.clone()
        }
    }
}

/// The names of session constants any plotted expression references — each
/// becomes a live slider (ADR-0014). Surfaces count too (ADR-0015): their
/// constants animate the mesh the same way.
fn slider_names(
    curves: &[SampledCurve],
    surfaces: &[epher_core::graph::Surface],
    session: &Session,
) -> Vec<String> {
    let mut names = std::collections::BTreeSet::new();
    for c in curves {
        let mut visit = |expr: &epher_core::Expression| {
            let mut found = std::collections::BTreeSet::new();
            free_names(expr, &mut found);
            for n in found {
                if session.const_sources().contains_key(&n) {
                    names.insert(n);
                }
            }
        };
        match &c.kind {
            CurveKind::Cartesian(e) => visit(e),
            CurveKind::Parametric { x, y } => {
                visit(x);
                visit(y);
            }
            CurveKind::Polar(e) => visit(e),
        }
    }
    for surface in surfaces {
        if let Ok((expr, _)) = epher_core::graph::parse_surface_source(&surface.source) {
            let mut found = std::collections::BTreeSet::new();
            free_names(&expr, &mut found);
            for n in found {
                if session.const_sources().contains_key(&n) {
                    names.insert(n);
                }
            }
        }
    }
    names.into_iter().collect()
}

/// The spec that would reproduce a sampled curve (slider re-sampling).
fn curve_spec(c: &SampledCurve) -> CurveSpec {
    CurveSpec {
        kind: c.kind.clone(),
        domain: c.domain,
        fill: c.fill,
    }
}

/// Re-sample every curve against the (possibly changed) session environment.
fn resample(curves: &mut [SampledCurve], session: &Session) {
    for c in curves.iter_mut() {
        if let Ok(samples) = sample_spec(&curve_spec(c), 120, session.env()) {
            c.samples = samples;
        }
    }
}

/// Re-sample every surface against the current environment (a moved
/// constant changes the mesh).
fn resample_surfaces(surfaces: &mut [epher_core::graph::Surface], session: &Session) {
    for surface in surfaces.iter_mut() {
        if let Ok(fresh) = epher_core::graph::sample_surface(&surface.source, 30, session.env()) {
            *surface = fresh;
        }
    }
}

/// Localized renderer labels for the analyzed points of interest.
fn poi_labels(points: &[InterestPoint], localizer: &Localizer) -> Vec<graph::Poi> {
    points
        .iter()
        .map(|p| {
            let key = match p.kind {
                epher_core::graph::InterestKind::Root => "poi-root",
                epher_core::graph::InterestKind::Intersection => "poi-intersection",
                epher_core::graph::InterestKind::Maximum => "poi-maximum",
                epher_core::graph::InterestKind::Minimum => "poi-minimum",
            };
            graph::Poi {
                kind: p.kind,
                label: localizer.lookup(key),
                x: p.x,
                y: p.y,
            }
        })
        .collect()
}

/// The value of a session constant as an f64, when it is one.
fn const_value(session: &Session, name: &str) -> Option<f64> {
    match session.env().constant(name)? {
        Value::Float(v) => Some(*v),
        _ => None,
    }
}

/// The curve whose sampled points are the closest to a trace position.
fn curve_at(curves: &[SampledCurve], index: usize) -> Option<&SampledCurve> {
    curves.get(index)
}

// ===== keypad (ADR-0016) =====

/// What a keypad press does: insert text at the cursor, insert `name(`,
/// submit the form, clear the entry, or backspace. The scripting language
/// itself is untouched — the keypad is a second spelling of the same input.
#[derive(Clone, Copy, PartialEq)]
enum KeyAction {
    Text(&'static str),
    Call(&'static str),
    Submit,
    Clear,
    Backspace,
}

struct KeyDef {
    label: &'static str,
    act: KeyAction,
    cls: &'static str,
}

struct TabDef {
    id: &'static str,
    label: &'static str,
    i18n: &'static str,
    keys: &'static [KeyDef],
}

const fn key(label: &'static str, act: KeyAction, cls: &'static str) -> KeyDef {
    KeyDef { label, act, cls }
}

/// Every function, constant, and command the language supports, grouped
/// like a scientific calculator's key banks (ADR-0016). Labels are the
/// language tokens themselves (ADR-0007 — the language is never
/// localized); `÷`/`×`/`−` show the operator glyphs but insert the ASCII
/// tokens the language spells them with.
static TABS: &[TabDef] = &[
    TabDef {
        id: "digits",
        label: "123",
        i18n: "keypad-tab-digits",
        keys: &[
            key("C", KeyAction::Clear, "act"),
            key("⌫", KeyAction::Backspace, "act"),
            key("(", KeyAction::Text("("), "op"),
            key(")", KeyAction::Text(")"), "op"),
            key("÷", KeyAction::Text("/"), "op"),
            key("7", KeyAction::Text("7"), ""),
            key("8", KeyAction::Text("8"), ""),
            key("9", KeyAction::Text("9"), ""),
            key("×", KeyAction::Text("*"), "op"),
            key("−", KeyAction::Text("-"), "op"),
            key("4", KeyAction::Text("4"), ""),
            key("5", KeyAction::Text("5"), ""),
            key("6", KeyAction::Text("6"), ""),
            key("+", KeyAction::Text("+"), "op"),
            key("^", KeyAction::Text("^"), "op"),
            key("1", KeyAction::Text("1"), ""),
            key("2", KeyAction::Text("2"), ""),
            key("3", KeyAction::Text("3"), ""),
            key(";", KeyAction::Text(";"), "op"),
            key(",", KeyAction::Text(","), "op"),
            key("0", KeyAction::Text("0"), ""),
            key(".", KeyAction::Text("."), ""),
            key("ans", KeyAction::Text("ans"), "fn"),
            key("=", KeyAction::Submit, "eq"),
        ],
    },
    TabDef {
        id: "trig",
        label: "trig",
        i18n: "keypad-tab-trig",
        keys: &[
            key("sin", KeyAction::Call("sin"), "fn"),
            key("cos", KeyAction::Call("cos"), "fn"),
            key("tan", KeyAction::Call("tan"), "fn"),
            key("asin", KeyAction::Call("asin"), "fn"),
            key("acos", KeyAction::Call("acos"), "fn"),
            key("atan", KeyAction::Call("atan"), "fn"),
            key("sinh", KeyAction::Call("sinh"), "fn"),
            key("cosh", KeyAction::Call("cosh"), "fn"),
            key("tanh", KeyAction::Call("tanh"), "fn"),
            key("asinh", KeyAction::Call("asinh"), "fn"),
            key("acosh", KeyAction::Call("acosh"), "fn"),
            key("atanh", KeyAction::Call("atanh"), "fn"),
            key("deg", KeyAction::Call("deg"), "fn"),
            key("rad", KeyAction::Call("rad"), "fn"),
            key("atan2", KeyAction::Call("atan2"), "fn"),
        ],
    },
    TabDef {
        id: "func",
        label: "ƒ",
        i18n: "keypad-tab-func",
        keys: &[
            key("ln", KeyAction::Call("ln"), "fn"),
            key("log", KeyAction::Call("log"), "fn"),
            key("log2", KeyAction::Call("log2"), "fn"),
            key("logb", KeyAction::Call("logb"), "fn"),
            key("exp", KeyAction::Call("exp"), "fn"),
            key("sqrt", KeyAction::Call("sqrt"), "fn"),
            key("cbrt", KeyAction::Call("cbrt"), "fn"),
            key("root", KeyAction::Call("root"), "fn"),
            key("hypot", KeyAction::Call("hypot"), "fn"),
            key("abs", KeyAction::Call("abs"), "fn"),
            key("floor", KeyAction::Call("floor"), "fn"),
            key("ceil", KeyAction::Call("ceil"), "fn"),
            key("round", KeyAction::Call("round"), "fn"),
            key("trunc", KeyAction::Call("trunc"), "fn"),
            key("sign", KeyAction::Call("sign"), "fn"),
            key("min", KeyAction::Call("min"), "fn"),
            key("max", KeyAction::Call("max"), "fn"),
        ],
    },
    TabDef {
        id: "num",
        label: "nΣ",
        i18n: "keypad-tab-num",
        keys: &[
            key("gcd", KeyAction::Call("gcd"), "fn"),
            key("lcm", KeyAction::Call("lcm"), "fn"),
            key("mod", KeyAction::Call("mod"), "fn"),
            key("fact", KeyAction::Call("fact"), "fn"),
            key("ncr", KeyAction::Call("ncr"), "fn"),
            key("npr", KeyAction::Call("npr"), "fn"),
            key("sum", KeyAction::Call("sum"), "fn"),
            key("product", KeyAction::Call("product"), "fn"),
            key("mean", KeyAction::Call("mean"), "fn"),
            key("median", KeyAction::Call("median"), "fn"),
            key("variance", KeyAction::Call("variance"), "fn"),
            key("stdev", KeyAction::Call("stdev"), "fn"),
            key("frac", KeyAction::Call("frac"), "fn"),
            key("dec", KeyAction::Call("dec"), "fn"),
            key("big", KeyAction::Call("big"), "fn"),
            key("bin", KeyAction::Call("bin"), "fn"),
            key("oct", KeyAction::Call("oct"), "fn"),
            key("hex", KeyAction::Call("hex"), "fn"),
        ],
    },
    TabDef {
        id: "const",
        label: "π∇",
        i18n: "keypad-tab-const",
        keys: &[
            key("pi", KeyAction::Text("pi"), "fn"),
            key("e", KeyAction::Text("e"), "fn"),
            key("tau", KeyAction::Text("tau"), "fn"),
            key("phi", KeyAction::Text("phi"), "fn"),
            key("x", KeyAction::Text("x"), "fn"),
            key("t", KeyAction::Text("t"), "fn"),
            key("ans", KeyAction::Text("ans"), "fn"),
            key("graph", KeyAction::Text("graph "), "fn"),
            key("graph3d", KeyAction::Text("graph3d "), "fn"),
            key("table", KeyAction::Text("table "), "fn"),
            key("clear", KeyAction::Text("clear "), "fn"),
            key("history", KeyAction::Text("history "), "fn"),
        ],
    },
];

/// The name of a language in itself — the menu lists languages the way
/// their speakers write them, independent of the UI language.
fn native_language_name(code: &str) -> &str {
    match code {
        "en" => "English",
        "zh-CN" => "\u{7b80}\u{4f53}\u{4e2d}\u{6587}",
        "hi" => "\u{939}\u{93f}\u{928}\u{94d}\u{926}\u{940}",
        "es" => "Espa\u{f1}ol",
        "fr" => "Fran\u{e7}ais",
        "ar" => "\u{627}\u{644}\u{639}\u{631}\u{628}\u{64a}\u{629}",
        "de" => "Deutsch",
        "pt" => "Portugu\u{ea}s",
        _ => code,
    }
}

#[function_component(EpherApp)]
fn epher_app() -> Html {
    let session = use_state(Session::new);
    let input = use_state(String::new);
    let form_ref = use_node_ref();
    let input_ref = use_node_ref();
    let result = use_state(String::new);
    let localizer = use_state(|| Localizer::resolve(None, &[]));
    let graph = use_state(Vec::<SampledCurve>::new);
    let pois = use_state(Vec::<graph::Poi>::new);
    let trace = use_state(|| Option::<graph::TracePoint>::None);
    // Graph options (ADR-0019, on the pane itself since ADR-0020): whether
    // the pane lists the points of interest and marks them on the plot,
    // and the curve line width. Display-only — the analysis always runs,
    // so switching back is instant.
    let poi_list = use_state(|| true);
    let poi_markers = use_state(|| true);
    let line_width = use_state(|| graph::DEFAULT_STROKE_WIDTH);
    let live = use_state(|| Rc::new(RefCell::new(GraphLive::default())));
    let surface = use_state(Vec::<epher_core::graph::Surface>::new);
    let view = use_state(epher_core::graph::View3D::default);
    let play = use_state(|| Option::<PlaySpec>::None);
    // The live cell behind `play`: the animation loop reads and advances
    // it across ticks; Yew handles captured at spawn read stale snapshots.
    let play_cell = use_state(|| Rc::new(RefCell::new(Option::<PlaySpec>::None)));
    // The 3D viewBox from the latest render; play start freezes it.
    let rendered_box = use_state(|| Rc::new(RefCell::new(None::<String>)));
    let show_install_cli = use_state(|| false);
    // Keypad tab + which pane faces the user on mobile (ADR-0016).
    let key_tab = use_state(|| "digits".to_string());
    let active_pane = use_state(|| "calc".to_string());
    // The UI theme (ADR-0017): dark is the default; light and night are
    // set from the Settings menu or the `theme` command. The open menu
    // bar item (File/Edit/Settings) drives the dropdown (ADR-0017).
    let theme = use_state(|| "dark".to_string());
    let menu_open = use_state(|| Option::<&'static str>::None);
    // A live mirror of `menu_open` for long-lived closures (Yew handles
    // are render snapshots; the Rc cell updates every render).
    let menu_open_cell = use_state(|| Rc::new(RefCell::new(Option::<&'static str>::None)));
    *menu_open_cell.borrow_mut() = *menu_open;
    let hamburger_open = use_state(|| false);
    let guide_open = use_state(|| false);
    let guide_close_ref = use_node_ref();
    {
        // Focus the close button whenever the guide opens so Escape works
        // from the first keypress (autofocus only fires on first insert).
        let guide_open = guide_open.clone();
        let guide_close_ref = guide_close_ref.clone();
        use_effect_with(guide_open, move |open| {
            if **open {
                if let Some(el) = guide_close_ref.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
            }
            || {}
        });
    }
    let file_ref = use_node_ref();
    let bridge = Bridge::detect();

    // Clear history (the button next to the list): empty the session's
    // history, persist the cleared state in the desktop shell, and leave
    // definitions, constants, and plotted curves untouched.
    let on_clear_history = {
        let session = session.clone();
        let result = result.clone();
        Callback::from(move |_| {
            let mut s = (*session).clone();
            s.clear_history();
            session.set(s);
            bridge.save_history(&[]);
            result.set(String::new());
        })
    };

    // Inside the desktop shell: rebuild the session from the native store —
    // history plus saved functions and scripts replayed quietly, the exact
    // load_session recipe — and honor the stored language preference.
    {
        let session = session.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        let theme = theme.clone();
        let poi_list = poi_list.clone();
        let poi_markers = poi_markers.clone();
        let line_width = line_width.clone();
        use_effect_with((), move |_| {
            if bridge == Bridge::Tauri {
                spawn_local(async move {
                    match bridge.init().await {
                        Ok(InitState {
                            history,
                            replay,
                            language,
                            theme: theme_pref,
                        }) => {
                            let mut s = Session::with_history(history);
                            for line in &replay {
                                s.submit_quiet(line);
                            }
                            session.set(s);
                            if let Some(code) = language {
                                localizer.set(Localizer::resolve(Some(&code), &[]));
                            }
                            if let Some(name) = theme_pref {
                                theme.set(name);
                            }
                            // Graph pane options live in the webview's
                            // localStorage on desktop too (ADR-0020) — the
                            // native store carries only what must exist
                            // before mount.
                            if let Some(store) = web_sys::window()
                                .and_then(|w| w.local_storage().ok().flatten())
                            {
                                if let Ok(Some(v)) = store.get_item("epher-poi-list") {
                                    if v == "0" {
                                        poi_list.set(false);
                                    }
                                }
                                if let Ok(Some(v)) = store.get_item("epher-poi-markers") {
                                    if v == "0" {
                                        poi_markers.set(false);
                                    }
                                }
                                if let Ok(Some(v)) = store.get_item("epher-line-width") {
                                    if let Ok(w) = v.parse::<f64>() {
                                        if (0.5..=4.0).contains(&w) {
                                            line_width.set(w);
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            result.set(format!(
                                "warning: could not load saved data ({e}); starting fresh"
                            ));
                        }
                    }
                });
            } else {
                // The web app persists its theme and language overrides in
                // localStorage (no native store here, ADR-0010).
                let win = web_sys::window();
                if let Some(store) = win.as_ref().and_then(|w| w.local_storage().ok().flatten()) {
                    if let Ok(Some(t)) = store.get_item("epher-theme") {
                        if matches!(t.as_str(), "light" | "dark" | "night") {
                            theme.set(t);
                        }
                    }
                    if let Ok(Some(code)) = store.get_item("epher-language") {
                        localizer.set(Localizer::resolve(Some(&code), &[]));
                    }
                    if let Ok(Some(v)) = store.get_item("epher-poi-list") {
                        if v == "0" {
                            poi_list.set(false);
                        }
                    }
                    if let Ok(Some(v)) = store.get_item("epher-poi-markers") {
                        if v == "0" {
                            poi_markers.set(false);
                        }
                    }
                    if let Ok(Some(v)) = store.get_item("epher-line-width") {
                        if let Ok(w) = v.parse::<f64>() {
                            if (0.5..=4.0).contains(&w) {
                                line_width.set(w);
                            }
                        }
                    }
                }
            }
            || {}
        });
    }

    // Apply the theme token set (ADR-0017): the attribute swaps every
    // CSS custom property in one move, curves included.
    {
        let theme = theme.clone();
        use_effect_with((*theme).clone(), move |t| {
            let doc = web_sys::window().and_then(|w| w.document());
            if let Some(el) = doc.as_ref().and_then(|d| d.document_element()) {
                let _ = el.set_attribute("data-theme", t.as_str());
            }
            || {}
        });
    }

    // macOS desktop builds offer to install the `epher` terminal command
    // (ADR-0011): a one-click symlink into /usr/local/bin.
    {
        let show_install_cli = show_install_cli.clone();
        use_effect_with((), move |_| {
            if bridge == Bridge::Tauri {
                spawn_local(async move {
                    if let Ok(true) = bridge.cli_install_supported().await {
                        show_install_cli.set(true);
                    }
                });
            }
            || {}
        });
    }

    let on_install_cli = {
        let result = result.clone();
        let localizer = localizer.clone();
        Callback::from(move |_| {
            let result = result.clone();
            let localizer = localizer.clone();
            spawn_local(async move {
                let outcome = bridge.install_cli().await;
                let message = match outcome {
                    Ok(key) => localizer.lookup(&key),
                    Err(detail) => {
                        format!("{} {detail}", localizer.lookup("install-cli-failed"))
                    }
                };
                result.set(message);
            });
        })
    };

    // ---- menu actions (ADR-0017) ------------------------------------
    // Set the theme everywhere it lives: the render (state + attribute
    // effect above) and the persistence layer — the native store in the
    // desktop shell, localStorage in the browser.
    let on_set_theme = {
        let theme = theme.clone();
        Callback::from(move |name: String| {
            if matches!(name.as_str(), "light" | "dark" | "night") {
                if let Some(store) = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                {
                    let _ = store.set_item("epher-theme", &name);
                }
                bridge.save_theme(&name);
                theme.set(name);
            }
        })
    };

    // Set the UI language: re-resolve the localizer and persist it.
    let on_set_language = {
        let localizer = localizer.clone();
        Callback::from(move |code: String| {
            if epher_i18n::SUPPORTED_LOCALES.contains(&code.as_str()) {
                localizer.set(Localizer::resolve(Some(&code), &[]));
                if let Some(store) = web_sys::window()
                    .and_then(|w| w.local_storage().ok().flatten())
                {
                    let _ = store.set_item("epher-language", &code);
                }
                bridge.save_language(&code);
            }
        })
    };

    // Graph pane options (ADR-0019 → ADR-0020): the points-of-interest
    // list, the highlighted points on the plot, and the curve line width,
    // persisted like the theme.
    let on_set_poi_list = {
        let poi_list = poi_list.clone();
        Callback::from(move |on: bool| {
            if let Some(store) = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = store.set_item("epher-poi-list", if on { "1" } else { "0" });
            }
            poi_list.set(on);
        })
    };
    let on_set_poi_markers = {
        let poi_markers = poi_markers.clone();
        Callback::from(move |on: bool| {
            if let Some(store) = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = store.set_item("epher-poi-markers", if on { "1" } else { "0" });
            }
            poi_markers.set(on);
        })
    };
    // The line-width slider (ADR-0020): persisted like the POI toggles,
    // clamped to the slider's range so a stale stored value cannot
    // produce an invisible or absurd curve.
    let on_set_line_width = {
        let line_width = line_width.clone();
        Callback::from(move |w: f64| {
            let w = w.clamp(0.5, 4.0);
            if let Some(store) = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = store.set_item("epher-line-width", &format!("{w}"));
            }
            line_width.set(w);
        })
    };

    // File → Open: the hidden input's picker; the chosen file's text
    // lands in the entry field for review before running.
    let on_open = {
        let file_ref = file_ref.clone();
        Callback::from(move |_| {
            if let Some(el) = file_ref.cast::<web_sys::HtmlInputElement>() {
                el.click();
            }
        })
    };
    let on_file_chosen = {
        let input = input.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        Callback::from(move |e: Event| {
            let target = e.target_unchecked_into::<web_sys::HtmlInputElement>();
            let Some(files) = target.files() else {
                return;
            };
            let Some(file) = files.item(0) else {
                return;
            };
            let input = input.clone();
            let result = result.clone();
            let localizer = localizer.clone();
            spawn_local(async move {
                if let Ok(text) = wasm_bindgen_futures::JsFuture::from(file.text())
                    .await
                    .and_then(|v| v.as_string().ok_or(()).map_err(|()| wasm_bindgen::JsValue::NULL))
                {
                    input.set(text);
                    result.set(localizer.lookup("menu-loaded"));
                }
            });
            // Allow picking the same file twice in a row.
            target.set_value("");
        })
    };

    // File → Save: a Blob download. History lines, or the entry field's
    // script — the two things a user may want on disk.
    let save_text_file = |filename: &str, text: String| {
        let Some(win) = web_sys::window() else {
            return;
        };
        let parts = js_sys::Array::new();
        parts.push(&wasm_bindgen::JsValue::from_str(&text));
        let Ok(blob) = web_sys::Blob::new_with_str_sequence(&parts) else {
            return;
        };
        let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
            return;
        };
        // The anchor must live in the document for the download to start,
        // and the blob URL must outlive the click — revoke it later, not
        // synchronously.
        if let Some(doc) = win.document() {
            if let Some(a) = doc.create_element("a").ok().and_then(|el| {
                el.dyn_into::<web_sys::HtmlAnchorElement>().ok()
            }) {
                a.set_href(&url);
                a.set_download(filename);
                if let Some(body) = doc.body() {
                    let _ = body.append_child(&a);
                    a.click();
                    let _ = body.remove_child(&a);
                }
            }
        }
        let url_clone = url;
        spawn_local(async move {
            gloo_timers::future::TimeoutFuture::new(10_000).await;
            let _ = web_sys::Url::revoke_object_url(&url_clone);
        });
    };

    let on_save_history = {
        let session = session.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        let menu_open = menu_open.clone();
        Callback::from(move |_| {
            let text = session.history().join("\n");
            save_text_file("epher-history.epher", text);
            result.set(localizer.lookup("menu-saved"));
            menu_open.set(None);
        })
    };

    let on_save_script = {
        let input = input.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        let menu_open = menu_open.clone();
        Callback::from(move |_| {
            let text = (*input).clone();
            if !text.trim().is_empty() {
                save_text_file("epher-script.epher", text);
                result.set(localizer.lookup("menu-saved"));
            }
            menu_open.set(None);
        })
    };

    // Edit → Cut/Copy/Paste: the platform clipboard. Copy takes the last
    // result (or the entry when nothing ran yet); Cut moves the entry to
    // the clipboard; Paste reads the clipboard into the entry at the
    // cursor. When the browser withholds read access, say so — Ctrl+V
    // still works directly in the field.
    let on_copy = {
        let result = result.clone();
        let input = input.clone();
        let menu_open = menu_open.clone();
        Callback::from(move |_| {
            let text = if (*result).is_empty() {
                (*input).clone()
            } else {
                (*result).clone()
            };
            if let Some(clipboard) =
                web_sys::window().map(|w| w.navigator().clipboard())
            {
                spawn_local(async move {
                    let _ = wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&text)).await;
                });
            }
            menu_open.set(None);
        })
    };

    let on_cut = {
        let result = result.clone();
        let input = input.clone();
        let menu_open = menu_open.clone();
        Callback::from(move |_| {
            let text = (*input).clone();
            if !text.is_empty() {
                if let Some(clipboard) =
                    web_sys::window().map(|w| w.navigator().clipboard())
                {
                    let text_for_clip = text.clone();
                    spawn_local(async move {
                        let _ =
                            wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&text_for_clip)).await;
                    });
                }
                input.set(String::new());
            }
            let _ = result; // cut stays quiet: the emptied field is the feedback
            menu_open.set(None);
        })
    };

    let on_paste = {
        let input = input.clone();
        let input_ref = input_ref.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        let menu_open = menu_open.clone();
        Callback::from(move |_| {
            let Some(clipboard) =
                web_sys::window().map(|w| w.navigator().clipboard())
            else {
                return;
            };
            let input = input.clone();
            let input_ref = input_ref.clone();
            let result = result.clone();
            let localizer = localizer.clone();
            let menu_open = menu_open.clone();
            spawn_local(async move {
                match wasm_bindgen_futures::JsFuture::from(clipboard.read_text()).await {
                    Ok(v) => {
                        if let Some(text) = v.as_string() {
                            // Insert at the cursor, exactly like a keypad
                            // token: replace the selection if there is one.
                            if let Some(ta) =
                                input_ref.cast::<web_sys::HtmlTextAreaElement>()
                            {
                                let start =
                                    ta.selection_start().unwrap_or_default().unwrap_or(0);
                                let end = ta.selection_end().unwrap_or_default().unwrap_or(0);
                                let value = ta.value();
                                let start = (start as usize).min(value.len());
                                let end = (end as usize).max(start).min(value.len());
                                let mut spliced = String::with_capacity(value.len() + text.len());
                                spliced.push_str(&value[..start]);
                                spliced.push_str(&text);
                                spliced.push_str(&value[end..]);
                                ta.set_value(&spliced);
                                let _ = ta.focus();
                                input.set(spliced);
                            } else {
                                input.set(format!("{}{text}", *input));
                            }
                        }
                    }
                    Err(_) => result.set(localizer.lookup("paste-blocked")),
                }
                menu_open.set(None);
            });
        })
    };

    let on_input = {
        let input = input.clone();
        Callback::from(move |e: InputEvent| {
            let target = e.target_unchecked_into::<HtmlTextAreaElement>();
            input.set(target.value());
        })
    };

    // Enter submits (the textarea's own Enter would insert a newline);
    // Shift+Enter inserts a newline so multi-line scripts can be composed
    // by hand. Submitting goes through the form so the `=` button and the
    // keyboard share one path.
    let on_keydown = {
        let form_ref = form_ref.clone();
        Callback::from(move |e: web_sys::KeyboardEvent| {
            if e.key() == "Enter" && !e.shift_key() && !e.is_composing() {
                e.prevent_default();
                if let Some(form) = form_ref.cast::<web_sys::HtmlFormElement>() {
                    let _ = form.request_submit();
                }
            }
        })
    };

    let on_submit = {
        let session = session.clone();
        let input = input.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        let theme = theme.clone();
        let graph = graph.clone();
        let pois = pois.clone();
        let trace = trace.clone();
        let live = live.clone();
        let surface = surface.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            // A submitted entry may be several lines (pasted from the
            // guide, or composed with Shift+Enter). Each line runs in
            // order against one session snapshot — script semantics, like
            // the REPL and piped mode. Yew state handles do not expose
            // writes made earlier in the same callback, so the loop works
            // on locals and the states are published once, after the loop.
            let mut s = (*session).clone();
            let mut curves = (*graph).clone();
            let mut surfaces = (*surface).clone();
            // Statements join with newlines or `;` — the same separator
            // (ADR-0001). Each piece dispatches in order, exactly as if
            // typed one by one.
            for raw_line in (*input).split('\n') {
                for piece in raw_line.split(';') {
                let line = piece.trim().to_string();
                if line.is_empty() {
                    continue;
                }

                // Graphing (ADR-0006/0014: the core samples, the frontend renders).
                // Each `graph` line overlays one more curve; the command
                // itself joins the history list like every submitted line.
                if let Some(source) = line.strip_prefix("graph ") {
                    let source = source.trim();
                    s.record(&line);
                    if source == "clear" {
                        curves.clear();
                        continue;
                    }
                    match parse_graph_source(source)
                        .and_then(|spec| sample_spec(&spec, 120, s.env()).map(|samples| (spec, samples)))
                    {
                        Ok((spec, samples)) => {
                            curves.push(SampledCurve {
                                source: source.to_string(),
                                kind: spec.kind,
                                domain: spec.domain,
                                samples,
                                fill: spec.fill,
                            });
                            result.set(format!("graph: {source}"));
                        }
                        Err(e) => result.set(format!("error: {e}")),
                    }
                    continue;
                }

                // 3D surfaces (ADR-0015): z = f(x, y) over a square
                // domain, overlaid like curves. The command joins the
                // history list like every submitted line.
                if let Some(source) = line.strip_prefix("graph3d ") {
                    let source = source.trim();
                    s.record(&line);
                    if source == "clear" {
                        surfaces.clear();
                        continue;
                    }
                    match epher_core::graph::sample_surface(source, 30, s.env()) {
                        Ok(fresh) => {
                            surfaces.push(fresh);
                            result.set(format!("graph3d: {source}"));
                        }
                        Err(e) => result.set(format!("error: {e}")),
                    }
                    continue;
                }

                // Shell commands (epher-shell policy): persist through the
                // bridge in the desktop shell; explain the web app's limits
                // otherwise.
                if let Some(cmd) = classify(&line) {
                    match bridge {
                        Bridge::Tauri => match prepare(&cmd, &s, &localizer) {
                            Ok(prepared) => {
                                match &prepared {
                                    epher_shell::Prepared::SaveFunction { name, source } => {
                                        bridge.save_function(name, source);
                                    }
                                    epher_shell::Prepared::SaveConstant { name, source } => {
                                        bridge.save_constant(name, source);
                                    }
                                    epher_shell::Prepared::SaveScript { name, source } => {
                                        bridge.save_script(name, source);
                                    }
                                    epher_shell::Prepared::Language { code } => {
                                        bridge.save_language(code);
                                        localizer.set(Localizer::resolve(Some(code), &[]));
                                        if let Some(store) = web_sys::window()
                                            .and_then(|w| w.local_storage().ok().flatten())
                                        {
                                            let _ = store.set_item("epher-language", code);
                                        }
                                    }
                                    epher_shell::Prepared::Theme { name } => {
                                        bridge.save_theme(name);
                                        theme.set(name.clone());
                                        if let Some(store) = web_sys::window()
                                            .and_then(|w| w.local_storage().ok().flatten())
                                        {
                                            let _ = store.set_item("epher-theme", name);
                                        }
                                    }
                                    epher_shell::Prepared::Table { .. } => {}
                                }
                                result.set(message(&prepared, &localizer));
                            }
                            Err(msg) => result.set(msg),
                        },
                        Bridge::None => {
                            // Tables are pure computation — they work in the
                            // browser session just like an evaluation.
                            match &cmd {
                                epher_shell::Command::Table { .. } => {
                                    match prepare(&cmd, &s, &localizer) {
                                        Ok(prepared) => result.set(message(&prepared, &localizer)),
                                        Err(msg) => result.set(msg),
                                    }
                                }
                                // Themes apply for the session in the
                                // browser; the menu persists them properly.
                                epher_shell::Command::Theme { name } => {
                                    match prepare(&cmd, &s, &localizer) {
                                        Ok(prepared) => {
                                            result.set(message(&prepared, &localizer));
                                            theme.set(name.clone());
                                        }
                                        Err(msg) => result.set(msg),
                                    }
                                }
                                _ => result.set(localizer.lookup("web-session-only")),
                            }
                        }
                    }
                    continue;
                }

                let out = s.submit(&line);
                result.set(out);
                }
            }
            // Publish the loop's outcomes once: points of interest and the
            // slider set follow from the final curves and session.
            let found = analyze(&curves, s.env());
            let labels = poi_labels(&found, &localizer);
            {
                let mut l = (*live).borrow_mut();
                l.curves = curves.clone();
                l.trace = None;
            }
            graph.set(curves);
            surface.set(surfaces);
            pois.set(labels);
            trace.set(None);
            session.set(s.clone());
            input.set(String::new());
            // Desktop apps are killed, not exited: persist per line (ADR-0010).
            if bridge == Bridge::Tauri {
                bridge.save_history(s.history());
            }
        })
    };

    // Sliders: adjusting a constant re-samples every curve against the new
    // environment and re-runs the analysis (ADR-0014).
    let on_slider = {
        let session = session.clone();
        let graph = graph.clone();
        let pois = pois.clone();
        let localizer = localizer.clone();
        let surface = surface.clone();
        Callback::from(move |(name, value): (String, f64)| {
            let mut s = (*session).clone();
            s.set_constant(
                name.clone(),
                Value::float(value),
                format!("const {name} = {value}"),
            );
            let mut curves = (*graph).clone();
            resample(&mut curves, &s);
            let mut surfaces = (*surface).clone();
            resample_surfaces(&mut surfaces, &s);
            let found = analyze(&curves, s.env());
            session.set(s);
            graph.set(curves);
            surface.set(surfaces);
            pois.set(poi_labels(&found, &localizer));
        })
    };

    // The same resample logic, shared with the animation loop through a
    // live cell (Yew handles captured by the loop would go stale). The
    // cell is refreshed after every render.
    let live_apply =
        use_state(|| Rc::new(RefCell::new(None::<Rc<dyn Fn(String, f64)>>)));
    {
        let live_apply = live_apply.clone();
        let on_slider = on_slider.clone();
        use_effect(move || {
            let apply: Rc<dyn Fn(String, f64)> = Rc::new(move |name: String, value: f64| {
                on_slider.emit((name, value));
            });
            *live_apply.borrow_mut() = Some(apply);
            || {}
        });
    }

    // 3D orbit: drag or arrow keys rotate the view (ADR-0015).
    let on_orbit = {
        let view = view.clone();
        Callback::from(move |(dyaw, dpitch): (f64, f64)| {
            let v = *view;
            view.set(epher_core::graph::View3D {
                yaw: v.yaw + dyaw,
                pitch: (v.pitch + dpitch).clamp(-1.4, 1.4),
                camera: v.camera,
            });
        })
    };

    // Parameter animation (ADR-0015): the play button on a slider starts a
    // loop that steps the constant within the slider's bounds and re-runs
    // the same resample path as a drag. The loop talks to the live cell so
    // it never reads stale state; `play` mirrors it for rendering.
    {
        let play_cell = play_cell.clone();
        let live_apply = live_apply.clone();
        // The loop must be spawned once, not per render: use_effect (no
        // deps) re-runs after every render, so a bare use_effect here
        // would add a new loop on every tick, each tick re-rendering and
        // spawning another — playback would accelerate to a crash.
        use_effect_with((), move |_| {
            spawn_local(async move {
                loop {
                    if (*play_cell).borrow().is_none() {
                        gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
                        continue;
                    }
                    // One step per 120 ms: a fresh constant's slider spans
                    // ±10 (200 steps), so one full cycle takes 24 s — the
                    // vendor norm for playback speed.
                    gloo_timers::future::sleep(std::time::Duration::from_millis(120)).await;
                    let Some(spec) = (*play_cell).borrow().clone() else {
                        continue;
                    };
                    let next = spec.ticked();
                    *play_cell.borrow_mut() = Some(next.clone());
                    if let Some(apply) = (*live_apply).borrow().as_ref() {
                        apply(next.name.clone(), next.value);
                    }
                }
            });
            || {}
        });
    }
    let start_play = {
        let play = play.clone();
        let play_cell = play_cell.clone();
        let rendered_box = rendered_box.clone();
        let live_apply = live_apply.clone();
        Callback::from(move |(name, value): (String, f64)| {
            let reduce = web_sys::window()
                .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok())
                .flatten()
                .map(|m| m.matches())
                .unwrap_or(false);
            if reduce {
                // No looping playback under reduced motion: each press
                // steps the parameter once (WCAG 2.3.3).
                let lo = f64::min(-10.0, value - 2.0);
                let hi = f64::max(10.0, value + 2.0);
                let mut next = value + 0.1;
                if next > hi {
                    next = lo;
                }
                if let Some(apply) = (*live_apply).borrow().as_ref() {
                    apply(name.clone(), next);
                }
                return;
            }
            let lo = f64::min(-10.0, value - 2.0);
            let hi = f64::max(10.0, value + 2.0);
            let spec = PlaySpec {
                name,
                lo,
                hi,
                step: 0.1,
                value,
                freeze: (*rendered_box).borrow().clone(),
            };
            play.set(Some(spec.clone()));
            *play_cell.borrow_mut() = Some(spec);
        })
    };
    let stop_play = {
        let play = play.clone();
        let play_cell = play_cell.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            play.set(None);
            *play_cell.borrow_mut() = None;
        })
    };

    // Trace: pointer moves/taps find the nearest sampled point; arrow keys
    // step along the traced curve (up/down switch curves). These callbacks
    // are bound to the SVG once at mount, so they read the live cell.
    let on_trace = {
        let live = live.clone();
        let trace = trace.clone();
        Callback::from(move |(px, py): (f64, f64)| {
            let found = {
                let l = (*live).borrow();
                graph::geometry(&l.curves)
                    .and_then(|geom| graph::trace_nearest(&l.curves, &geom, px, py))
            };
            (*live).borrow_mut().trace = found;
            trace.set(found);
        })
    };
    let on_trace_leave = {
        let live = live.clone();
        let trace = trace.clone();
        Callback::from(move |()| {
            (*live).borrow_mut().trace = None;
            trace.set(None);
        })
    };
    let on_trace_key = {
        let live = live.clone();
        let trace = trace.clone();
        Callback::from(move |e: web_sys::KeyboardEvent| {
            let Some(current) = (*live).borrow().trace else {
                return;
            };
            let data = (*live).borrow().curves.clone();
            let Some(curve) = curve_at(&data, current.curve) else {
                return;
            };
            let last = curve.samples.len().saturating_sub(1);
            match e.key().as_str() {
                "ArrowRight" => {
                    if current.index < last {
                        let s = &curve.samples[current.index + 1];
                        let next = Some(graph::TracePoint {
                            index: current.index + 1,
                            x: s.x,
                            y: s.y,
                            ..current
                        });
                        (*live).borrow_mut().trace = next;
                        trace.set(next);
                    }
                    e.prevent_default();
                }
                "ArrowLeft" => {
                    if current.index > 0 {
                        let s = &curve.samples[current.index - 1];
                        let next = Some(graph::TracePoint {
                            index: current.index - 1,
                            x: s.x,
                            y: s.y,
                            ..current
                        });
                        (*live).borrow_mut().trace = next;
                        trace.set(next);
                    }
                    e.prevent_default();
                }
                "ArrowDown" if !data.is_empty() => {
                    let ci = (current.curve + 1) % data.len();
                    let c = &data[ci];
                    if let Some(s) = c.samples.get(current.index.min(last)) {
                        let next = Some(graph::TracePoint {
                            curve: ci,
                            index: current.index.min(last),
                            x: s.x,
                            y: s.y,
                        });
                        (*live).borrow_mut().trace = next;
                        trace.set(next);
                    }
                    e.prevent_default();
                }
                "ArrowUp" if !data.is_empty() => {
                    let ci = (current.curve + data.len() - 1) % data.len();
                    let c = &data[ci];
                    if let Some(s) = c.samples.get(current.index.min(last)) {
                        let next = Some(graph::TracePoint {
                            curve: ci,
                            index: current.index.min(last),
                            x: s.x,
                            y: s.y,
                        });
                        (*live).borrow_mut().trace = next;
                        trace.set(next);
                    }
                    e.prevent_default();
                }
                _ => {}
            }
        })
    };

    // Copy the plot as standalone SVG (the same string renderer the tests
    // exercise), with a localized outcome message.
    let on_copy_svg = {
        let curves = graph.clone();
        let pois = pois.clone();
        let trace = trace.clone();
        let poi_markers = poi_markers.clone();
        let line_width = line_width.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        Callback::from(move |_| {
            let svg = graph::graph_svg(&curves, &pois, *trace, *poi_markers, *line_width);
            if svg.is_empty() {
                return;
            }
            let result = result.clone();
            let localizer = localizer.clone();
            spawn_local(async move {
                if let Some(clipboard) = web_sys::window().map(|w| w.navigator().clipboard()) {
                    match clipboard.write_text(&svg).await {
                        Ok(_) => result.set(localizer.lookup("graph-copied")),
                        Err(_) => result.set(localizer.lookup("graph-copy-failed")),
                    }
                } else {
                    result.set(localizer.lookup("graph-copy-failed"));
                }
            });
        })
    };

    let is_error = result.starts_with("error:") || result.starts_with("warning:");

    // Keypad presses (ADR-0016): insert text at the textarea cursor —
    // selection-replacing, cursor after the inserted text — or act like
    // the pocket calculator keys they are. Focus returns to the input so
    // typing continues right after a press. The language itself is
    // untouched: the keypad only spells input the evaluator already reads.
    let on_keypad = {
        let input = input.clone();
        let input_ref = input_ref.clone();
        let form_ref = form_ref.clone();
        Callback::from(move |act: KeyAction| {
            let Some(ta) = input_ref.cast::<HtmlTextAreaElement>() else {
                return;
            };
            let cursor = |v: &str| -> (usize, usize) {
                let s = ta
                    .selection_start()
                    .ok()
                    .flatten()
                    .unwrap_or(0) as usize;
                let e = ta
                    .selection_end()
                    .ok()
                    .flatten()
                    .unwrap_or(0) as usize;
                (s.min(v.len()), e.min(v.len()))
            };
            match act {
                KeyAction::Submit => {
                    if let Some(form) = form_ref.cast::<web_sys::HtmlFormElement>() {
                        let _ = form.request_submit();
                    }
                }
                KeyAction::Clear => {
                    input.set(String::new());
                    ta.set_value("");
                    ta.set_selection_start(Some(0)).ok();
                    ta.set_selection_end(Some(0)).ok();
                }
                KeyAction::Backspace => {
                    let mut v = (*input).clone();
                    let (s, e) = cursor(&v);
                    let (lo, hi) = if s == e { (s.saturating_sub(1), s) } else { (s, e) };
                    v.replace_range(lo..hi, "");
                    input.set(v.clone());
                    ta.set_value(&v);
                    ta.set_selection_start(Some(lo as u32)).ok();
                    ta.set_selection_end(Some(lo as u32)).ok();
                }
                KeyAction::Text(t) => {
                    let mut v = (*input).clone();
                    let (s, e) = cursor(&v);
                    v.replace_range(s..e, t);
                    input.set(v.clone());
                    ta.set_value(&v);
                    let pos = s + t.len();
                    ta.set_selection_start(Some(pos as u32)).ok();
                    ta.set_selection_end(Some(pos as u32)).ok();
                }
                KeyAction::Call(name) => {
                    let mut v = (*input).clone();
                    let (s, e) = cursor(&v);
                    let t = format!("{name}(");
                    v.replace_range(s..e, &t);
                    input.set(v.clone());
                    ta.set_value(&v);
                    let pos = s + t.len();
                    ta.set_selection_start(Some(pos as u32)).ok();
                    ta.set_selection_end(Some(pos as u32)).ok();
                }
            }
            let _ = ta.focus();
        })
    };

    // Pane switching (ADR-0016): mobile swipes horizontally between the
    // calculator and the graph; these buttons are the non-swipe spelling.
    // The jump is instant — one discrete step, which is also the
    // reduced-motion behavior (WCAG 2.3.3).
    let scroll_pane = Callback::from(|id: &'static str| {
        let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
            return;
        };
        let (Some(panes), Some(pane)) = (
            doc.get_element_by_id("panes"),
            doc.get_element_by_id(id),
        ) else {
            return;
        };
        let offset = pane
            .dyn_ref::<web_sys::HtmlElement>()
            .map(|el| el.offset_left())
            .unwrap_or(0);
        let target = offset.saturating_sub(panes.client_left());
        panes.set_scroll_left(target);
    });
    let on_panes_scroll = {
        let active_pane = active_pane.clone();
        Callback::from(move |e: Event| {
            let panes = e.target_unchecked_into::<web_sys::HtmlElement>();
            let next = if panes.scroll_left() > panes.client_width() / 2 {
                "graph"
            } else {
                "calc"
            };
            if *active_pane != next {
                active_pane.set(next.to_string());
            }
        })
    };

    // The trace announcement: coordinates in the current UI language-free
    // numeric form, announced politely (the plot itself is an image).
    let trace_text = (*trace).map(|t| format!("x = {:.3}, y = {:.3}", t.x, t.y));

    // Slider rows for a list of constant names — the 2D plot gets the
    // constants its curves reference, the 3D plot the constants its
    // surfaces reference (ADR-0014/0015). Dragging the animated slider
    // stops playback; the play button (re)starts it.
    let build_rows = |names: &[String]| -> Vec<Html> {
        names
            .iter()
            .filter_map(|name| {
                let v = const_value(&session, name)?;
                let lo = f64::min(-10.0, v - 2.0);
                let hi = f64::max(10.0, v + 2.0);
                let on_slider = on_slider.clone();
                let playing_this = (*play).as_ref().is_some_and(|p| p.name == *name);
                let stop_on_drag = {
                    let play = play.clone();
                    let play_cell = play_cell.clone();
                    let name = name.clone();
                    let on_slider = on_slider.clone();
                    Callback::from(move |e: InputEvent| {
                        let target = e.target_unchecked_into::<HtmlInputElement>();
                        let Ok(value) = target.value().parse::<f64>() else {
                            return;
                        };
                        if play.as_ref().is_some_and(|p| p.name == name) {
                            play.set(None);
                            *play_cell.borrow_mut() = None;
                        }
                        on_slider.emit((name.clone(), value));
                    })
                };
                let name_for_play = name.clone();
                let start_play = start_play.clone();
                let stop_play = stop_play.clone();
                let animate_label = if playing_this {
                    localizer.lookup("animate-stop")
                } else {
                    localizer.lookup("animate")
                };
                Some(html! {
                    <div class="slider">
                        <span class="slider-name">{ name.clone() }</span>
                        <input
                            type="range"
                            min={lo.to_string()}
                            max={hi.to_string()}
                            step="0.1"
                            value={v.to_string()}
                            oninput={stop_on_drag}
                        />
                        <span class="slider-value">{ format!("{v:.3}") }</span>
                        <button
                            type="button"
                            class="play-btn"
                            aria-pressed={playing_this.to_string()}
                            aria-label={format!("{animate_label} {name}")}
                            onclick={if playing_this {
                                stop_play
                            } else {
                                Callback::from(move |_: web_sys::MouseEvent| {
                                    start_play.emit((name_for_play.clone(), v))
                                })
                            }}
                        >
                            <span aria-hidden="true">{ if playing_this { "⏸" } else { "▶" } }</span>
                        </button>
                    </div>
                })
            })
            .collect()
    };
    let curve_sliders = slider_names(&graph, &[], &session);
    let surface_sliders = slider_names(&[], &surface, &session);
    let curve_rows = build_rows(&curve_sliders);
    let surface_rows = build_rows(&surface_sliders);

    let legend_items: Vec<Html> = (*graph)
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let caption = graph::curve_caption(c);
            html! {
                <li>
                    <span class={format!("swatch curve-{i}")} aria-hidden="true"></span>
                    { caption }
                </li>
            }
        })
        .collect();

    let poi_items: Vec<Html> = (*pois)
        .iter()
        .map(|p| {
            let text = format!("{} ({}, {})", p.label, graph::label(p.x), graph::label(p.y));
            html! { <li>{ text }</li> }
        })
        .collect();

    // ADR-0017: on mobile the menu bar folds into a hamburger whose panel
    // lists the same three menus as labeled groups. Items share the
    // desktop handlers and close the panel when activated.
    let close_hamburger = {
        let hamburger_open = hamburger_open.clone();
        let menu_open = menu_open.clone();
        Callback::from(move |_| {
            hamburger_open.set(false);
            menu_open.set(None);
        })
    };
    let mobile_item = |action: Callback<web_sys::MouseEvent>| {
        let close = close_hamburger.clone();
        Callback::from(move |e: web_sys::MouseEvent| {
            action.emit(e);
            close.emit(());
        })
    };

    // ADR-0018: clear the graph pane — curves, points of interest, 3D
    // surfaces, and any trace/animation state (the same as the `graph
    // clear` / `graph3d clear` commands, in one button).
    let on_graph_clear = {
        let graph = graph.clone();
        let pois = pois.clone();
        let surface = surface.clone();
        let trace = trace.clone();
        let play = play.clone();
        let play_cell = play_cell.clone();
        let live = live.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        Callback::from(move |_| {
            graph.set(Vec::new());
            pois.set(Vec::new());
            surface.set(Vec::new());
            trace.set(None);
            play.set(None);
            *play_cell.borrow_mut() = None;
            *live.borrow_mut() = GraphLive::default();
            result.set(localizer.lookup("graph-cleared"));
        })
    };

    // ADR-0018: the in-app user guide — the same markdown the website
    // guide pages are built from, rendered for the current language.
    let on_open_guide = {
        let guide_open = guide_open.clone();
        Callback::from(move |_: web_sys::MouseEvent| guide_open.set(true))
    };
    let on_close_guide = {
        let guide_open = guide_open.clone();
        Callback::from(move |_: web_sys::MouseEvent| guide_open.set(false))
    };

    // File → Quit (ADR-0023): the desktop shell exits its process; a
    // browser tab can only ask the browser, which refuses for tabs it did
    // not open — after a moment still on screen, say so honestly.
    let on_quit = {
        let result = result.clone();
        let localizer = localizer.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            if bridge == Bridge::Tauri {
                spawn_local(async move { bridge.quit().await });
            } else {
                if let Some(w) = web_sys::window() {
                    let _ = w.close();
                }
                let result = result.clone();
                let localizer = localizer.clone();
                spawn_local(async move {
                    gloo_timers::future::sleep(std::time::Duration::from_millis(300)).await;
                    result.set(localizer.lookup("quit-tab-hint"));
                });
            }
        })
    };

    // Native menu behavior (ADR-0023): a click anywhere outside the menu
    // bar closes the open menu, like every desktop menubar. Inside the
    // bar, the button handlers own the toggle, so this only watches the
    // outside.
    let menubar_ref = use_node_ref();
    {
        let menu_open = menu_open.clone();
        let menubar_ref = menubar_ref.clone();
        use_effect_with((), move |_| {
            let menu_open = menu_open.clone();
            let menu_open_cell = menu_open_cell.clone();
            let menubar_ref = menubar_ref.clone();
            let window = web_sys::window().expect("window");
            let callback: Rc<Closure<dyn FnMut(web_sys::Event)>> =
                Rc::new(Closure::new(move |e: web_sys::Event| {
                    if menu_open_cell.borrow().is_none() {
                        return;
                    }
                    let inside = e
                        .target()
                        .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                        .and_then(|el| {
                            menubar_ref
                                .cast::<web_sys::Element>()
                                .map(|bar| bar.contains(Some(&el)))
                        })
                        .unwrap_or(false);
                    if !inside {
                        menu_open.set(None);
                    }
                }));
            window
                .add_event_listener_with_callback(
                    "mousedown",
                    callback.as_ref().as_ref().unchecked_ref(),
                )
                .expect("mousedown listener");
            // The destructor keeps the closure alive and removes it on
            // unmount. Raw web-sys rather than gloo-events: the app's
            // unified gloo build does not deliver raw DOM events to
            // EventListener callbacks (ADR-0023).
            let window = window.clone();
            let cb = callback.clone();
            move || {
                let _ = window.remove_event_listener_with_callback(
                    "mousedown",
                    cb.as_ref().as_ref().unchecked_ref(),
                );
            }
        });
    }

    html! {
        <main class="epher">
            <h1 class="visually-hidden">{ localizer.lookup("app-name") }</h1>
            <header class="topbar">
                {
                    // File → Open: a real file picker, hidden from the tab
                    // order (it is reached through the menu item).
                    html! {
                        <input
                            type="file"
                            accept=".epher,.txt,text/plain"
                            class="visually-hidden-file"
                            ref={file_ref.clone()}
                            onchange={on_file_chosen}
                            tabindex="-1"
                            aria-hidden="true"
                        />
                    }
                }
                <nav
                    class="menubar"
                    role="menubar"
                    ref={menubar_ref.clone()}
                    onkeydown={{
                        let menu_open = menu_open.clone();
                        Callback::from(move |e: web_sys::KeyboardEvent| {
                            if e.key() == "Escape" {
                                menu_open.set(None);
                            }
                        })
                    }}
                >
                    <div class="menu">
                        <button
                            type="button"
                            role="menuitem"
                            aria-haspopup="menu"
                            aria-expanded={(*menu_open == Some("file")).to_string()}
                            class={if *menu_open == Some("file") { "menu-top open" } else { "menu-top" }}
                            onclick={{
                                let menu_open = menu_open.clone();
                                Callback::from(move |_| menu_open.set(if *menu_open == Some("file") { None } else { Some("file") }))
                            }}
                        >
                            { localizer.lookup("menu-file") }
                        </button>
                        {
                            if *menu_open == Some("file") {
                                html! {
                                    <div class="menu-drop" role="menu" aria-label={localizer.lookup("menu-file")}>
                                        <button type="button" role="menuitem" class="menu-item"
                                            onclick={Callback::from({
                                                let menu_open = menu_open.clone();
                                                let on_open = on_open.clone();
                                                move |_| { menu_open.set(None); on_open.emit(()); }
                                            })}
                                        >
                                            { localizer.lookup("menu-open") }
                                        </button>
                                        <button type="button" role="menuitem" class="menu-item" onclick={on_save_history.clone()}>
                                            { localizer.lookup("menu-save-history") }
                                        </button>
                                        <button type="button" role="menuitem" class="menu-item" onclick={on_save_script.clone()}>
                                            { localizer.lookup("menu-save-script") }
                                        </button>
                                        <div class="menu-sep" role="separator"></div>
                                        <button type="button" role="menuitem" class="menu-item"
                                            onclick={Callback::from({
                                                let menu_open = menu_open.clone();
                                                let on_quit = on_quit.clone();
                                                move |e: web_sys::MouseEvent| { menu_open.set(None); on_quit.emit(e); }
                                            })}
                                        >
                                            { localizer.lookup("menu-quit") }
                                        </button>
                                    </div>
                                }
                            } else { html! {} }
                        }
                    </div>
                    <div class="menu">
                        <button
                            type="button"
                            role="menuitem"
                            aria-haspopup="menu"
                            aria-expanded={(*menu_open == Some("edit")).to_string()}
                            class={if *menu_open == Some("edit") { "menu-top open" } else { "menu-top" }}
                            onclick={{
                                let menu_open = menu_open.clone();
                                Callback::from(move |_| menu_open.set(if *menu_open == Some("edit") { None } else { Some("edit") }))
                            }}
                        >
                            { localizer.lookup("menu-edit") }
                        </button>
                        {
                            if *menu_open == Some("edit") {
                                html! {
                                    <div class="menu-drop" role="menu" aria-label={localizer.lookup("menu-edit")}>
                                        <button type="button" role="menuitem" class="menu-item" onclick={Callback::from({
                                            let menu_open = menu_open.clone();
                                            let on_cut = on_cut.clone();
                                            move |e: web_sys::MouseEvent| { menu_open.set(None); on_cut.emit(e); }
                                        })}>
                                            { localizer.lookup("menu-cut") }
                                        </button>
                                        <button type="button" role="menuitem" class="menu-item" onclick={Callback::from({
                                            let menu_open = menu_open.clone();
                                            let on_copy = on_copy.clone();
                                            move |e: web_sys::MouseEvent| { menu_open.set(None); on_copy.emit(e); }
                                        })}>
                                            { localizer.lookup("menu-copy") }
                                        </button>
                                        <button type="button" role="menuitem" class="menu-item" onclick={Callback::from({
                                            let menu_open = menu_open.clone();
                                            let on_paste = on_paste.clone();
                                            move |e: web_sys::MouseEvent| { menu_open.set(None); on_paste.emit(e); }
                                        })}>
                                            { localizer.lookup("menu-paste") }
                                        </button>
                                    </div>
                                }
                            } else { html! {} }
                        }
                    </div>
                    <div class="menu">
                        <button
                            type="button"
                            role="menuitem"
                            aria-haspopup="menu"
                            aria-expanded={(*menu_open == Some("settings")).to_string()}
                            class={if *menu_open == Some("settings") { "menu-top open" } else { "menu-top" }}
                            onclick={{
                                let menu_open = menu_open.clone();
                                Callback::from(move |_| menu_open.set(if *menu_open == Some("settings") { None } else { Some("settings") }))
                            }}
                        >
                            { localizer.lookup("menu-settings") }
                        </button>
                        {
                            if *menu_open == Some("settings") {
                                html! {
                                    <div class="menu-drop wide" role="menu" aria-label={localizer.lookup("menu-settings")}>
                                        <p class="menu-group" aria-hidden="true">{ localizer.lookup("menu-theme") }</p>
                                        { for ["light", "dark", "night"].map(|name| {
                                            let label = match name {
                                                "light" => localizer.lookup("theme-light"),
                                                "night" => localizer.lookup("theme-night"),
                                                _ => localizer.lookup("theme-dark"),
                                            };
                                            let checked = *theme == name;
                                            html! {
                                                <button type="button" role="menuitemradio" class="menu-item"
                                                    aria-checked={checked.to_string()}
                                                    onclick={Callback::from({
                                                        let menu_open = menu_open.clone();
                                                        let on_set_theme = on_set_theme.clone();
                                                        let name = name.to_string();
                                                        move |_| { on_set_theme.emit(name.clone()); menu_open.set(None); }
                                                    })}
                                                >
                                                    <span class="menu-check" aria-hidden="true">{ if checked { "\u{2713}" } else { "" } }</span>
                                                    { label }
                                                </button>
                                            }
                                        }) }
                                        <div class="menu-sep" role="separator"></div>
                                        <p class="menu-group" aria-hidden="true">{ localizer.lookup("menu-language") }</p>
                                        { for epher_i18n::SUPPORTED_LOCALES.iter().map(|code| {
                                            let checked = localizer.locale() == *code;
                                            html! {
                                                <button type="button" role="menuitemradio" class="menu-item"
                                                    aria-checked={checked.to_string()}
                                                    onclick={Callback::from({
                                                        let menu_open = menu_open.clone();
                                                        let on_set_language = on_set_language.clone();
                                                        let code = code.to_string();
                                                        move |_| { on_set_language.emit(code.clone()); menu_open.set(None); }
                                                    })}
                                                >
                                                    <span class="menu-check" aria-hidden="true">{ if checked { "\u{2713}" } else { "" } }</span>
                                                    { native_language_name(code) }
                                                </button>
                                            }
                                        }) }
                                    </div>
                                }
                            } else { html! {} }
                        }
                    </div>
                    <div class="menu">
                        <button
                            type="button"
                            role="menuitem"
                            aria-haspopup="menu"
                            aria-expanded={(*menu_open == Some("help")).to_string()}
                            class={if *menu_open == Some("help") { "menu-top open" } else { "menu-top" }}
                            onclick={{
                                let menu_open = menu_open.clone();
                                Callback::from(move |_| menu_open.set(if *menu_open == Some("help") { None } else { Some("help") }))
                            }}
                        >
                            { localizer.lookup("menu-help") }
                        </button>
                        {
                            if *menu_open == Some("help") {
                                html! {
                                    <div class="menu-drop" role="menu" aria-label={localizer.lookup("menu-help")}>
                                        <button type="button" role="menuitem" class="menu-item"
                                            onclick={Callback::from({
                                                let menu_open = menu_open.clone();
                                                let on_open_guide = on_open_guide.clone();
                                                move |e: web_sys::MouseEvent| { menu_open.set(None); on_open_guide.emit(e); }
                                            })}
                                        >
                                            { localizer.lookup("menu-guide") }
                                        </button>
                                    </div>
                                }
                            } else { html! {} }
                        }
                    </div>
                </nav>
                <nav class="pane-switch">
                    <button
                        type="button"
                        aria-pressed={(*active_pane == "calc").to_string()}
                        aria-label={localizer.lookup("calc-pane")}
                        onclick={{
                            let scroll_pane = scroll_pane.clone();
                            Callback::from(move |_| scroll_pane.emit("calc-pane"))
                        }}
                    >
                        { localizer.lookup("calc-pane") }
                    </button>
                    <button
                        type="button"
                        aria-pressed={(*active_pane == "graph").to_string()}
                        aria-label={localizer.lookup("graph-pane")}
                        onclick={{
                            let scroll_pane = scroll_pane.clone();
                            Callback::from(move |_| scroll_pane.emit("graph-pane"))
                        }}
                    >
                        { localizer.lookup("graph-pane") }
                    </button>
                </nav>
                <button
                    type="button"
                    class="hamburger-btn"
                    aria-label={localizer.lookup("menu")}
                    aria-haspopup="menu"
                    aria-controls="mobile-menu"
                    aria-expanded={hamburger_open.to_string()}
                    onclick={{
                        let hamburger_open = hamburger_open.clone();
                        Callback::from(move |_| hamburger_open.set(!*hamburger_open))
                    }}
                >
                    {"\u{2630}"}
                </button>
                {
                    if *hamburger_open {
                        html! {
                            <div
                                id="mobile-menu"
                                class="mobile-menu"
                                role="menu"
                                aria-label={localizer.lookup("menu")}
                                tabindex="0"
                                onkeydown={{
                                    let hamburger_open = hamburger_open.clone();
                                    Callback::from(move |e: web_sys::KeyboardEvent| {
                                        if e.key() == "Escape" {
                                            hamburger_open.set(false);
                                        }
                                    })
                                }}
                            >
                                <p class="menu-group" aria-hidden="true">{ localizer.lookup("menu-file") }</p>
                                <button type="button" role="menuitem" class="menu-item" onclick={mobile_item(Callback::from({
                                    let on_open = on_open.clone();
                                    move |_: web_sys::MouseEvent| on_open.emit(())
                                }))}>
                                    { localizer.lookup("menu-open") }
                                </button>
                                <button type="button" role="menuitem" class="menu-item" onclick={mobile_item(on_save_history.clone())}>
                                    { localizer.lookup("menu-save-history") }
                                </button>
                                <button type="button" role="menuitem" class="menu-item" onclick={mobile_item(on_save_script.clone())}>
                                    { localizer.lookup("menu-save-script") }
                                </button>
                                <button type="button" role="menuitem" class="menu-item" onclick={mobile_item(on_quit.clone())}>
                                    { localizer.lookup("menu-quit") }
                                </button>
                                <div class="menu-sep" role="separator"></div>
                                <p class="menu-group" aria-hidden="true">{ localizer.lookup("menu-edit") }</p>
                                <button type="button" role="menuitem" class="menu-item" onclick={mobile_item(on_cut.clone())}>
                                    { localizer.lookup("menu-cut") }
                                </button>
                                <button type="button" role="menuitem" class="menu-item" onclick={mobile_item(on_copy.clone())}>
                                    { localizer.lookup("menu-copy") }
                                </button>
                                <button type="button" role="menuitem" class="menu-item" onclick={mobile_item(on_paste.clone())}>
                                    { localizer.lookup("menu-paste") }
                                </button>
                                <div class="menu-sep" role="separator"></div>
                                <p class="menu-group" aria-hidden="true">{ localizer.lookup("menu-theme") }</p>
                                { for ["light", "dark", "night"].map(|name| {
                                    let label = match name {
                                        "light" => localizer.lookup("theme-light"),
                                        "night" => localizer.lookup("theme-night"),
                                        _ => localizer.lookup("theme-dark"),
                                    };
                                    let checked = *theme == name;
                                    html! {
                                        <button type="button" role="menuitemradio" class="menu-item"
                                            aria-checked={checked.to_string()}
                                            onclick={Callback::from({
                                                let on_set_theme = on_set_theme.clone();
                                                let close = close_hamburger.clone();
                                                let name = name.to_string();
                                                move |_| { on_set_theme.emit(name.clone()); close.emit(()); }
                                            })}
                                        >
                                            <span class="menu-check" aria-hidden="true">{ if checked { "\u{2713}" } else { "" } }</span>
                                            { label }
                                        </button>
                                    }
                                }) }
                                <div class="menu-sep" role="separator"></div>
                                <p class="menu-group" aria-hidden="true">{ localizer.lookup("menu-language") }</p>
                                { for epher_i18n::SUPPORTED_LOCALES.iter().map(|code| {
                                    let checked = localizer.locale() == *code;
                                    html! {
                                        <button type="button" role="menuitemradio" class="menu-item"
                                            aria-checked={checked.to_string()}
                                            onclick={Callback::from({
                                                let on_set_language = on_set_language.clone();
                                                let close = close_hamburger.clone();
                                                let code = code.to_string();
                                                move |_| { on_set_language.emit(code.clone()); close.emit(()); }
                                            })}
                                        >
                                            <span class="menu-check" aria-hidden="true">{ if checked { "\u{2713}" } else { "" } }</span>
                                            { native_language_name(code) }
                                        </button>
                                    }
                                }) }
                                <div class="menu-sep" role="separator"></div>
                                <p class="menu-group" aria-hidden="true">{ localizer.lookup("menu-help") }</p>
                                <button type="button" role="menuitem" class="menu-item" onclick={mobile_item(Callback::from({
                                    let on_open_guide = on_open_guide.clone();
                                    move |e: web_sys::MouseEvent| on_open_guide.emit(e)
                                }))}>
                                    { localizer.lookup("menu-guide") }
                                </button>
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </header>
            <div class="panes" id="panes" onscroll={on_panes_scroll}>
                <section class="pane" id="calc-pane">
                    {
                        if *show_install_cli {
                            html! {
                                <button
                                    type="button"
                                    class="install-cli"
                                    onclick={on_install_cli}
                                >
                                    { localizer.lookup("install-cli") }
                                </button>
                            }
                        } else {
                            html! {}
                        }
                    }
                    <form ref={form_ref.clone()} onsubmit={on_submit}>
                        <textarea
                            ref={input_ref.clone()}
                            rows="1"
                            placeholder={"expression or script"}
                            value={(*input).clone()}
                            oninput={on_input}
                            onkeydown={on_keydown}
                            autofocus={true}
                            aria-label="expression"
                            aria-invalid={if is_error { "true" } else { "false" }}
                            aria-describedby={if is_error { "epher-result" } else { "" }}
                        />
                    </form>
                    <div class="answer">
                        <span class="visually-hidden" id="answer-label">
                            { localizer.lookup("answer") }
                        </span>
                        <div
                            id="epher-result"
                            class="result"
                            role="status"
                            aria-live="polite"
                            aria-labelledby="answer-label"
                            tabindex="0"
                        >
                            { (*result).clone() }
                        </div>
                    </div>
                    <section class="history-box" tabindex="0" aria-label={localizer.lookup("history")}>
                        <div class="history-head">
                            <h2>{ localizer.lookup("history") }</h2>
                            <button type="button" class="clear-history" onclick={on_clear_history}>
                                { localizer.lookup("clear-history") }
                            </button>
                        </div>
                        <ul class="history">
                            { for session.history().iter().rev().map(|h| html! { <li>{ h.clone() } </li> }) }
                        </ul>
                    </section>
                    <section class="keypad" aria-label={localizer.lookup("keypad")}>
                        <div class="keypad-tabs" role="tablist" aria-label={localizer.lookup("keypad")}>
                            { for TABS.iter().map(|t| {
                                let on_tab = {
                                    let key_tab = key_tab.clone();
                                    let id = t.id;
                                    Callback::from(move |_| key_tab.set(id.to_string()))
                                };
                                html! {
                                    <button
                                        type="button"
                                        role="tab"
                                        id={format!("keypad-tab-{}", t.id)}
                                        aria-selected={(*key_tab == t.id).to_string()}
                                        aria-controls="keypad-panel"
                                        aria-label={localizer.lookup(t.i18n)}
                                        onclick={on_tab}
                                    >
                                        { t.label }
                                    </button>
                                }
                            }) }
                        </div>
                        <div
                            class="keypad-grid"
                            role="tabpanel"
                            id="keypad-panel"
                            aria-labelledby={format!("keypad-tab-{}", (*key_tab).as_str())}
                        >
                            { for TABS.iter()
                                .find(|t| t.id == *key_tab)
                                .map(|t| t.keys.iter().map(|k| {
                                    let on_key = {
                                        let on_keypad = on_keypad.clone();
                                        let act = k.act;
                                        Callback::from(move |_| on_keypad.emit(act))
                                    };
                                    html! {
                                        <button
                                            type="button"
                                            class={format!("keypad-btn {}", k.cls)}
                                            aria-label={k.label}
                                            onclick={on_key}
                                        >
                                            { k.label }
                                        </button>
                                    }
                                }).collect::<Vec<Html>>())
                                .unwrap_or_default() }
                        </div>
                    </section>
                </section>
                <section class="pane" id="graph-pane" aria-label={localizer.lookup("graph-pane")}>
                    {
                        if !(*graph).is_empty() || !(*surface).is_empty() {
                            html! {
                                // The pane toolbar (ADR-0023): commands and
                                // settings sit above the plot — Clear and
                                // Copy SVG as equal buttons, the graph
                                // options beside them — not scattered under
                                // it. Everything is a real labelled control.
                                <div class="graph-head">
                                    <button type="button" class="pane-btn" onclick={on_graph_clear.clone()}>
                                        { localizer.lookup("graph-clear") }
                                    </button>
                                    {
                                        if !(*graph).is_empty() {
                                            html! {
                                                <>
                                                    <button type="button" class="pane-btn" onclick={on_copy_svg}>
                                                        { localizer.lookup("graph-copy") }
                                                    </button>
                                                    // The graph options row (ADR-0020): the two
                                                    // points-of-interest toggles that lived in
                                                    // Settings (ADR-0019) plus the line-width
                                                    // slider. Real form controls — focusable and
                                                    // labelled — not menu items, because they
                                                    // are adjustments, not commands.
                                                    <div class="graph-options">
                                                        <label class="graph-option">
                                                            <input type="checkbox" checked={*poi_list} onchange={Callback::from({
                                                                let on_set_poi_list = on_set_poi_list.clone();
                                                                move |e: web_sys::Event| {
                                                                    if let Some(el) = e
                                                                        .target()
                                                                        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                                                    {
                                                                        on_set_poi_list.emit(el.checked());
                                                                    }
                                                                }
                                                            })} />
                                                            { localizer.lookup("graph-points") }
                                                        </label>
                                                        <label class="graph-option">
                                                            <input type="checkbox" checked={*poi_markers} onchange={Callback::from({
                                                                let on_set_poi_markers = on_set_poi_markers.clone();
                                                                move |e: web_sys::Event| {
                                                                    if let Some(el) = e
                                                                        .target()
                                                                        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                                                    {
                                                                        on_set_poi_markers.emit(el.checked());
                                                                    }
                                                                }
                                                            })} />
                                                            { localizer.lookup("settings-markers") }
                                                        </label>
                                                        <label class="graph-option graph-width">
                                                            <span class="graph-width-label">{ localizer.lookup("graph-width") }</span>
                                                            <input type="range" class="graph-width-slider"
                                                                min="0.5" max="4" step="0.25" value={line_width.to_string()}
                                                                oninput={Callback::from({
                                                                    let on_set_line_width = on_set_line_width.clone();
                                                                    move |e: web_sys::InputEvent| {
                                                                        if let Some(el) = e
                                                                            .target()
                                                                            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                                                        {
                                                                            if let Ok(w) = el.value().parse::<f64>() {
                                                                                on_set_line_width.emit(w);
                                                                            }
                                                                        }
                                                                    }
                                                                })}
                                                            />
                                                            <span class="graph-width-value" aria-hidden="true">{ format!("{:.2}", *line_width) }</span>
                                                        </label>
                                                    </div>
                                                </>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                    {
                        if !(*graph).is_empty() {
                            html! {
                                <section class="graph">
                                    <ul class="legend">
                                        { for legend_items }
                                    </ul>
                                    <div class="plot-box">
                                        <Graph
                                            curves={(*graph).clone()}
                                            pois={(*pois).clone()}
                                            trace={*trace}
                                            markers={*poi_markers}
                                            line_width={*line_width}
                                            on_trace={on_trace}
                                            on_key={on_trace_key}
                                            on_leave={on_trace_leave}
                                        />
                                    </div>
                                    <p class="trace" role="status" aria-live="polite">
                                        { trace_text }
                                    </p>
                                    {
                                        if !(*pois).is_empty() && *poi_list {
                                            html! {
                                                <>
                                                    <p class="poi-heading">{ localizer.lookup("graph-points") }</p>
                                                    <ul class="poi-list">
                                                        { for poi_items }
                                                    </ul>
                                                </>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    <div class="sliders">
                                        { for curve_rows }
                                    </div>
                                </section>
                            }
                        } else {
                            html! {}
                        }
                    }
                    {
                        if !(*surface).is_empty() {
                            let rendered = graph::surface_svg(&surface, &view);
                            let aria = format!(
                                "{}: {}",
                                "3D",
                                (*surface)
                                    .iter()
                                    .map(|s| format!("z = {}", s.source.trim()))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                            if let Some((view_box, content)) = rendered {
                                // Record for play-freeze; while playing, keep the
                                // frozen box so the layout stays put.
                                *rendered_box.borrow_mut() = Some(view_box.clone());
                                let shown_box = (*play)
                                    .as_ref()
                                    .and_then(|p| p.freeze.clone())
                                    .unwrap_or(view_box);
                                html! {
                                    <section class="graph graph3d">
                                        <h2 class="graph3d-title">{ "3D" }</h2>
                                        <div class="plot-box">
                                            <Graph3D
                                                view_box={shown_box}
                                                content={content}
                                                aria_label={aria}
                                                on_orbit={on_orbit}
                                            />
                                        </div>
                                        <p class="graph3d-hint">{ localizer.lookup("graph3d-hint") }</p>
                                        <div class="sliders">
                                            { for surface_rows }
                                        </div>
                                    </section>
                                }
                            } else {
                                html! {}
                            }
                        } else {
                            html! {}
                        }
                    }
                </section>
            </div>
            {
                if *guide_open {
                    html! {
                        <div
                            class="guide-overlay"
                            role="dialog"
                            aria-modal="true"
                            aria-label={localizer.lookup("menu-guide")}
                            onkeydown={{
                                let guide_open = guide_open.clone();
                                Callback::from(move |e: web_sys::KeyboardEvent| {
                                    if e.key() == "Escape" {
                                        guide_open.set(false);
                                    }
                                })
                            }}
                        >
                            <div class="guide-head">
                                <h2>{ localizer.lookup("menu-guide") }</h2>
                                <button type="button" class="guide-close-btn" ref={guide_close_ref.clone()} onclick={on_close_guide.clone()}>
                                    { localizer.lookup("guide-close") }
                                </button>
                            </div>
                            <p class="guide-insert-hint">{ localizer.lookup("guide-insert-hint") }</p>
                            <div class="guide-body" tabindex="0" onclick={Callback::from({
                                let input = input.clone();
                                let input_ref = input_ref.clone();
                                let guide_open = guide_open.clone();
                                let scroll_pane = scroll_pane.clone();
                                move |e: web_sys::MouseEvent| {
                                    // Clicking an example loads its code into
                                    // the entry field and returns to the
                                    // calculator (ADR-0018).
                                    if let Some(target) =
                                        e.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                                    {
                                        if let Some(btn) =
                                            target.closest(".guide-example-btn").ok().flatten()
                                        {
                                            if let Some(code) = btn.get_attribute("data-code") {
                                                input.set(code);
                                                guide_open.set(false);
                                                scroll_pane.emit("calc-pane");
                                                if let Some(ta) =
                                                    input_ref.cast::<web_sys::HtmlTextAreaElement>()
                                                {
                                                    let _ = ta.focus();
                                                }
                                            }
                                        }
                                    }
                                }
                            })}>
                                {
                                    Html::from_html_unchecked(
                                        epher_guide::render_html(epher_guide::guide(localizer.locale())).into(),
                                    )
                                }
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </main>

    }
}

#[wasm_bindgen(start)]
pub fn start() {
    yew::Renderer::<EpherApp>::new().render();
}
