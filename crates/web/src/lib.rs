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
use epher_core::{history_expression, Session, Value};
use epher_i18n::Localizer;
use epher_shell::{classify, message, prepare};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
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
        ],
    },
    TabDef {
        id: "conv",
        label: "0x",
        i18n: "keypad-tab-conv",
        keys: &[
            key("frac", KeyAction::Call("frac"), "fn"),
            key("dec", KeyAction::Call("dec"), "fn"),
            key("big", KeyAction::Call("big"), "fn"),
            key("bin", KeyAction::Call("bin"), "fn"),
            key("oct", KeyAction::Call("oct"), "fn"),
            key("hex", KeyAction::Call("hex"), "fn"),
            key("!", KeyAction::Text("!"), "fn"),
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

/// File → Save: start a Blob download of `text` as `filename` (the
/// browser fallback when the save picker is unavailable).
/// The panes sit side by side from 880px up (ADR-0016); below that they
/// form a swipeable horizontal strip, so "show the graph" means sliding
/// the strip across. Mirrors the CSS breakpoint (879.98px).
fn mobile_layout() -> bool {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|v| v.as_f64())
        .map(|w| w < 880.0)
        .unwrap_or(false)
}

/// Whether a stored line width fits the current layout's slider range
/// (ADR-0031): mobile spans 0–0.2 step 0.01, desktop 0.1–4 step 0.1.
/// A stored value from the other layout is out of range and ignored, so
/// each layout starts at its own default.
fn width_in_range(w: f64) -> bool {
    if mobile_layout() {
        (0.0..=0.2).contains(&w)
    } else {
        (0.1..=4.0).contains(&w)
    }
}

/// Whether the user asked for reduced motion (WCAG 2.3.3). The rotation
/// sliders' continuous spin is motion: under reduced motion they keep
/// their v0.4.19 static-offset meaning instead (ADR-0032).
fn prefers_reduced() -> bool {
    web_sys::window()
        .and_then(|w| w.match_media("(prefers-reduced-motion: reduce)").ok())
        .flatten()
        .map(|m| m.matches())
        .unwrap_or(false)
}

/// The pose the 3D plot renders in (ADR-0031/0032): the orbit base plus
/// the spin phase and the static zoom offset. Under reduced motion no
/// phase accrues, so the rotation sliders keep their static-offset
/// meaning and the formula falls back to `with_offsets`.
fn effective_view(
    base: &epher_core::graph::View3D,
    h: f64,
    v: f64,
    z: f64,
    phase: (f64, f64),
) -> epher_core::graph::View3D {
    if prefers_reduced() {
        base.with_offsets(h, v, z)
    } else {
        base.with_spin_phase(phase.0, phase.1, z)
    }
}

/// One top-level menu icon (ADR-0032): the menu bar is a vertical rail of
/// icon buttons, so the names live in each button's aria-label (and its
/// native tooltip) while the SVG is aria-hidden.
fn menu_icon(inner: &'static str) -> yew::Html {
    yew::Html::from_html_unchecked(
        format!(
            "<svg class=\"menu-icon\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">{inner}</svg>"
        )
        .into(),
    )
}

// The stroke paths for the four rail icons (lucide's file, pencil,
// settings, and circle-help glyphs, MIT-licensed).
const ICON_FILE: &str =
    "<path d=\"M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z\"/><polyline points=\"14 2 14 8 20 8\"/>";
const ICON_EDIT: &str = "<path d=\"M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z\"/>";
const ICON_SETTINGS: &str = "<path d=\"M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z\"/><circle cx=\"12\" cy=\"12\" r=\"3\"/>";
const ICON_HELP: &str = "<circle cx=\"12\" cy=\"12\" r=\"10\"/><path d=\"M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3\"/><path d=\"M12 17h.01\"/>";

fn download_text_file(filename: &str, text: &str) {
    let Some(win) = web_sys::window() else {
        return;
    };
    let parts = js_sys::Array::new();
    parts.push(&wasm_bindgen::JsValue::from_str(text));
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
        if let Some(a) = doc
            .create_element("a")
            .ok()
            .and_then(|el| el.dyn_into::<web_sys::HtmlAnchorElement>().ok())
        {
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
}

/// File → Save (ADR-0024): ask the operating system where the file
/// should live. The desktop shell shows its native save dialog over IPC;
/// the PWA uses the browser's own save picker (File System Access API)
/// where it exists and falls back to a download elsewhere. Cancel stays
/// silent, like any native app; a written file is reported with its
/// path (desktop) or its name (browser).
fn save_with_dialog(
    bridge: Bridge,
    default_name: &str,
    text: String,
    script: bool,
    result: &UseStateHandle<String>,
    localizer: &UseStateHandle<Localizer>,
    menu_open: &UseStateHandle<Option<&'static str>>,
) {
    let default_name = default_name.to_string();
    let result = result.clone();
    let localizer = localizer.clone();
    let menu_open = menu_open.clone();
    spawn_local(async move {
        let report = |result: &UseStateHandle<String>,
                      localizer: &UseStateHandle<Localizer>,
                      name: &str| {
            let key = if script { "saved-script" } else { "saved" };
            result.set(localizer.lookup_args(key, &[("name", name)]));
        };
        if bridge == Bridge::Tauri {
            match bridge.save_file_dialog(&text, &default_name).await {
                Ok(Some(path)) => report(&result, &localizer, &path),
                Ok(None) => {}
                Err(e) => result.set(format!("error: {e}")),
            }
        } else {
            match browser_save_dialog(&default_name, &text).await {
                Ok(Some(name)) => report(&result, &localizer, &name),
                Ok(None) => {}
                Err(_) => {
                    download_text_file(&default_name, &text);
                    result.set(localizer.lookup("menu-saved"));
                }
            }
        }
        menu_open.set(None);
    });
}

/// The browser's own save dialog (File System Access API, Chromium).
/// `Ok(None)` = the user cancelled; `Err` = this browser has no picker
/// (the caller falls back to a download).
async fn browser_save_dialog(default_name: &str, text: &str) -> Result<Option<String>, String> {
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let unavailable = || "save picker unavailable".to_string();
    let picker = js_sys::Reflect::get(&window, &JsValue::from_str("showSaveFilePicker"))
        .map_err(|_| unavailable())?;
    if !picker.is_function() {
        return Err(unavailable());
    }
    let picker_fn = picker.dyn_into::<js_sys::Function>().map_err(|_| unavailable())?;

    let accept = js_sys::Object::new();
    let extensions = js_sys::Array::new();
    extensions.push(&JsValue::from_str(".epher"));
    js_sys::Reflect::set(
        &accept,
        &JsValue::from_str("text/plain"),
        &extensions,
    )
    .map_err(|_| unavailable())?;
    let type_entry = js_sys::Object::new();
    js_sys::Reflect::set(
        &type_entry,
        &JsValue::from_str("description"),
        &JsValue::from_str("epher"),
    )
    .map_err(|_| unavailable())?;
    js_sys::Reflect::set(&type_entry, &JsValue::from_str("accept"), &accept)
        .map_err(|_| unavailable())?;
    let types = js_sys::Array::new();
    types.push(&type_entry);
    let options = js_sys::Object::new();
    js_sys::Reflect::set(
        &options,
        &JsValue::from_str("suggestedName"),
        &JsValue::from_str(default_name),
    )
    .map_err(|_| unavailable())?;
    js_sys::Reflect::set(&options, &JsValue::from_str("types"), &types)
        .map_err(|_| unavailable())?;

    let window_js: JsValue = window.into();
    let promise = picker_fn
        .call1(&window_js, &options)
        .map_err(|_| unavailable())?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| unavailable())?;
    let handle = match JsFuture::from(promise).await {
        Ok(handle) => handle,
        Err(e) => {
            let cancelled = e
                .dyn_ref::<web_sys::DomException>()
                .map(|de| de.name() == "AbortError")
                .unwrap_or(false);
            return if cancelled { Ok(None) } else { Err(unavailable()) };
        }
    };
    let name = js_sys::Reflect::get(&handle, &JsValue::from_str("name"))
        .map(|v| v.as_string().unwrap_or_default())
        .unwrap_or_default();
    let writable = js_sys::Reflect::get(&handle, &JsValue::from_str("createWritable"))
        .map_err(|_| unavailable())?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| unavailable())?
        .call0(&handle)
        .map_err(|_| unavailable())?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| unavailable())?;
    let writable = JsFuture::from(writable).await.map_err(|_| unavailable())?;
    let write = js_sys::Reflect::get(&writable, &JsValue::from_str("write"))
        .map_err(|_| unavailable())?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| unavailable())?;
    let write_promise = write
        .call1(&writable, &JsValue::from_str(text))
        .map_err(|_| unavailable())?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| unavailable())?;
    JsFuture::from(write_promise).await.map_err(|_| unavailable())?;
    let close = js_sys::Reflect::get(&writable, &JsValue::from_str("close"))
        .map_err(|_| unavailable())?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| unavailable())?;
    let close_promise = close
        .call0(&writable)
        .map_err(|_| unavailable())?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| unavailable())?;
    JsFuture::from(close_promise).await.map_err(|_| unavailable())?;
    Ok(Some(name))
}

/// The browser's own open picker (File System Access API, Chromium) —
/// the device's file explorer — or `None` when this browser has none
/// (the caller falls back to the hidden file input).
fn browser_open_picker() -> Option<js_sys::Function> {
    let window = web_sys::window()?;
    let picker = js_sys::Reflect::get(&window, &JsValue::from_str("showOpenFilePicker")).ok()?;
    picker.dyn_into::<js_sys::Function>().ok()
}

/// What `showOpenFilePicker` resolved to: a file, a user cancellation
/// (stay silent, like the save dialog — ADR-0024), or a failure the
/// caller falls back from.
enum OpenOutcome {
    File(web_sys::File),
    Cancelled,
    Failed,
}

/// Run the open picker. The file-type filter is deliberately empty
/// (ADR-0028): the user navigates their files freely.
async fn browser_open_dialog() -> OpenOutcome {
    let Some(picker) = browser_open_picker() else {
        return OpenOutcome::Failed;
    };
    let types = js_sys::Array::new();
    let opts = js_sys::Object::new();
    if js_sys::Reflect::set(&opts, &JsValue::from_str("types"), &types).is_err() {
        return OpenOutcome::Failed;
    }
    let Ok(promise) = picker.call1(&JsValue::UNDEFINED, &opts) else {
        return OpenOutcome::Failed;
    };
    match JsFuture::from(js_sys::Promise::from(promise)).await {
        Err(e) => {
            let cancelled = js_sys::Reflect::get(&e, &JsValue::from_str("name"))
                .ok()
                .and_then(|v| v.as_string())
                .as_deref()
                == Some("AbortError");
            if cancelled {
                OpenOutcome::Cancelled
            } else {
                OpenOutcome::Failed
            }
        }
        Ok(handles) => {
            let handles = js_sys::Array::from(&handles);
            let handle = handles.get(0);
            let Ok(get_file) = js_sys::Reflect::get(&handle, &JsValue::from_str("getFile")) else {
                return OpenOutcome::Failed;
            };
            let Ok(file_fn) = get_file.dyn_into::<js_sys::Function>() else {
                return OpenOutcome::Failed;
            };
            let Ok(file) = file_fn.call0(&handle) else {
                return OpenOutcome::Failed;
            };
            match JsFuture::from(js_sys::Promise::from(file)).await {
                Ok(v) => v
                    .dyn_into::<web_sys::File>()
                    .map(OpenOutcome::File)
                    .unwrap_or(OpenOutcome::Failed),
                Err(_) => OpenOutcome::Failed,
            }
        }
    }
}

/// Read an opened file's text.
async fn open_file_text(file: web_sys::File) -> Option<String> {
    JsFuture::from(file.text())
        .await
        .ok()
        .and_then(|v| v.as_string())
}

/// Load opened history text (ADR-0025): the current history clears,
/// then each non-empty line is recorded — nothing executes — and the
/// new history persists through the bridge. The answer names the count.
fn apply_open_history(
    text: String,
    session: &UseStateHandle<Session>,
    bridge: Bridge,
    result: &UseStateHandle<String>,
    localizer: &Localizer,
) {
    // The double deref matters: UseStateHandle itself is Clone, so a
    // single deref would clone the handle, not the session.
    let mut s = (**session).clone();
    s.clear_history();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty()) {
        s.record(line);
    }
    let loaded = s.history().len();
    let count = loaded.to_string();
    // The handle deref stays a render snapshot (the play_cell pattern,
    // ADR-0023): read the slice from `s` before moving it into the state.
    let saved: Vec<String> = s.history().to_vec();
    session.set(s);
    bridge.save_history(&saved);
    result.set(localizer.lookup_args("history-loaded", &[("count", &count)]));
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
    // so switching back is instant. Mobile starts at 0.1: thin lines for
    // the small screen (ADR-0031), the desktop default stays 1.0.
    let poi_list = use_state(|| true);
    let poi_markers = use_state(|| true);
    let line_width = use_state(|| if mobile_layout() { 0.1 } else { graph::DEFAULT_STROKE_WIDTH });
    // Which side of the 880px breakpoint the window is on (ADR-0016): the
    // width slider's range is a mobile/desktop decision (0–0.2 step 0.01
    // vs 0.1–4 step 0.1), and it tracks window resizes.
    let is_mobile = use_state(mobile_layout);
    let live = use_state(|| Rc::new(RefCell::new(GraphLive::default())));
    let surface = use_state(Vec::<epher_core::graph::Surface>::new);
    let view = use_state(epher_core::graph::View3D::default);
    // The live cell behind `view`: orbit emissions mutate it in place, so
    // a burst of drag/keyboard events accumulates instead of each event
    // reading the same stale handle snapshot and overwriting the last
    // (the v0.4.13 "shivering" — the render-snapshot rule, ADR-0026).
    let view_cell = use_state(|| Rc::new(RefCell::new(epher_core::graph::View3D::default())));
    // The 3D fine-control sliders (ADR-0031): horizontal rotation,
    // vertical rotation, and zoom offsets — each −1..1, step 0.1, with 0
    // the default. They ride on top of the orbit base view, applied via
    // View3D::with_offsets, and reset whenever a 3D graph is drawn into
    // an empty pane.
    // ADR-0032: the two rotation sliders SPIN while non-zero — horizontal
    // around the vertical axis, vertical around the horizontal axis,
    // roughly one revolution in six seconds at full deflection. The phase
    // accumulates in a live cell per frame; `spin_phase` mirrors it for
    // rendering. At zero the phase freezes where the spin stopped; only a
    // fresh 3D graph or Clear graph resets it.
    let view_h = use_state(|| 0.0_f64);
    let view_v = use_state(|| 0.0_f64);
    let view_z = use_state(|| 0.0_f64);
    let spin_phase = use_state(|| (0.0_f64, 0.0_f64));
    let spin_phase_cell = use_state(|| Rc::new(RefCell::new((0.0_f64, 0.0_f64))));
    // Live mirrors of the rotation slider values for the spin loop.
    let view_h_cell = use_state(|| Rc::new(RefCell::new(0.0_f64)));
    let view_v_cell = use_state(|| Rc::new(RefCell::new(0.0_f64)));
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
    // The entry's selection, mirrored while it owns focus and refreshed
    // at each keypad mousedown (ADR-0035): keypad presses read it, because
    // the button's mousedown default action blurs the entry — and the blur
    // that closes the mobile keyboard also zeroes the DOM selection in
    // Chromium. The tuple carries ranges, so replacing a selection works.
    let cursor_cell = use_state(|| Rc::new(RefCell::new((0usize, 0usize))));
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
    let history_ref = use_node_ref();
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
                                        if width_in_range(w) {
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
                            if width_in_range(w) {
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
    // The line-width slider (ADR-0020): persisted like the POI toggles,
    // clamped to the slider's range so a stale stored value cannot
    // re-enter. The range itself is the layout's (ADR-0031): mobile
    // 0–0.2 step 0.01 (thin lines on the small screen, floor included),
    // desktop 0.1–4 step 0.1 (the ADR-0028 hairline floor stays).
    let on_set_line_width = {
        let line_width = line_width.clone();
        Callback::from(move |w: f64| {
            let (lo, hi) = if mobile_layout() {
                (0.0, 0.2)
            } else {
                (0.1, 4.0)
            };
            let w = w.clamp(lo, hi);
            if let Some(store) = web_sys::window()
                .and_then(|w| w.local_storage().ok().flatten())
            {
                let _ = store.set_item("epher-line-width", &format!("{w}"));
            }
            line_width.set(w);
        })
    };

    // Crossing the 880px breakpoint re-clamps the width to the new
    // slider range (ADR-0031): the slider element's value is always
    // current, and the setter clamps to whatever layout the window is
    // in now. The `is_mobile` state flips the slider's attributes.
    {
        let is_mobile = is_mobile.clone();
        let on_set_line_width = on_set_line_width.clone();
        use_effect(move || {
            let window = web_sys::window().expect("window");
            let listener = gloo_events::EventListener::new(&window, "resize", move |_| {
                is_mobile.set(mobile_layout());
                if let Some(el) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.query_selector(".graph-width-slider").ok().flatten())
                    .and_then(|el| el.dyn_into::<web_sys::HtmlInputElement>().ok())
                {
                    if let Ok(w) = el.value().parse::<f64>() {
                        on_set_line_width.emit(w);
                    }
                }
            });
            move || drop(listener)
        });
    }

    // File → Open script (ADR-0025, ADR-0031): on the PWA the browser's
    // own open picker (File System Access API) runs when available — the
    // device's file explorer, straight from the menu tap. Browsers
    // without it, and the desktop shell, fall back to the hidden input's
    // picker, and a picker failure falls back the same way.
    let on_open_script = {
        let file_ref = file_ref.clone();
        let input = input.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        let bridge = bridge;
        Callback::from(move |_| {
            if !matches!(bridge, Bridge::Tauri) && browser_open_picker().is_some() {
                let file_ref = file_ref.clone();
                let input = input.clone();
                let result = result.clone();
                let localizer = localizer.clone();
                spawn_local(async move {
                    match browser_open_dialog().await {
                        OpenOutcome::File(file) => {
                            if let Some(text) = open_file_text(file).await {
                                input.set(text);
                                result.set(localizer.lookup("menu-loaded"));
                            }
                        }
                        OpenOutcome::Cancelled => {}
                        OpenOutcome::Failed => {
                            if let Some(el) = file_ref.cast::<web_sys::HtmlInputElement>() {
                                el.click();
                            }
                        }
                    }
                });
            } else if let Some(el) = file_ref.cast::<web_sys::HtmlInputElement>() {
                el.click();
            }
        })
    };
    let on_script_chosen = {
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

    // File → Open history (ADR-0025): the hidden input's picker; the
    // chosen file's lines REPLACE the history section — the current
    // history clears first, then each non-empty line is recorded — and
    // the new history persists through the same store save every submit
    // uses. Nothing executes: the lines display exactly as saved.
    let on_open_history = {
        let history_ref = history_ref.clone();
        let session = session.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        let bridge = bridge;
        Callback::from(move |_| {
            if !matches!(bridge, Bridge::Tauri) && browser_open_picker().is_some() {
                let history_ref = history_ref.clone();
                let session = session.clone();
                let result = result.clone();
                let localizer = localizer.clone();
                let bridge = bridge;
                spawn_local(async move {
                    match browser_open_dialog().await {
                        OpenOutcome::File(file) => {
                            if let Some(text) = open_file_text(file).await {
                                apply_open_history(text, &session, bridge, &result, &localizer);
                            }
                        }
                        OpenOutcome::Cancelled => {}
                        OpenOutcome::Failed => {
                            if let Some(el) = history_ref.cast::<web_sys::HtmlInputElement>() {
                                el.click();
                            }
                        }
                    }
                });
            } else if let Some(el) = history_ref.cast::<web_sys::HtmlInputElement>() {
                el.click();
            }
        })
    };
    let on_history_chosen = {
        let session = session.clone();
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
            let session = session.clone();
            let result = result.clone();
            let localizer = localizer.clone();
            spawn_local(async move {
                if let Ok(text) = wasm_bindgen_futures::JsFuture::from(file.text())
                    .await
                    .and_then(|v| v.as_string().ok_or(()).map_err(|()| wasm_bindgen::JsValue::NULL))
                {
                    apply_open_history(text, &session, bridge, &result, &localizer);
                }
            });
            // Allow picking the same file twice in a row.
            target.set_value("");
        })
    };

    // File → Save: a Blob download. History lines, or the entry field's
    // script — the two things a user may want on disk.
    let on_save_history = {
        let session = session.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        let menu_open = menu_open.clone();
        Callback::from(move |_| {
            let text = session.history().join("\n");
            save_with_dialog(
                bridge,
                "epher-history.ehs",
                text,
                false,
                &result,
                &localizer,
                &menu_open,
            );
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
                save_with_dialog(
                    bridge,
                    "epher-script.esr",
                    text,
                    true,
                    &result,
                    &localizer,
                    &menu_open,
                );
            } else {
                result.set(localizer.lookup("save-empty"));
                menu_open.set(None);
            }
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

    // Pane switching (ADR-0016): mobile swipes horizontally between the
    // calculator and the graph; these buttons are the non-swipe spelling.
    // The jump is instant — one discrete step, which is also the
    // reduced-motion behavior (WCAG 2.3.3). Defined before on_submit so
    // the submit path can slide the view to a freshly drawn graph.
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
        let scroll_pane = scroll_pane.clone();
        let input_ref = input_ref.clone();
        let view_h = view_h.clone();
        let view_v = view_v.clone();
        let view_z = view_z.clone();
        let spin_phase = spin_phase.clone();
        let spin_phase_cell = spin_phase_cell.clone();
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
            // typed one by one — but the history keeps the script the way
            // the user entered it: one entry per line, semicolons intact.
            for raw_line in (*input).split('\n') {
                let line = raw_line.trim().to_string();
                if line.is_empty() {
                    continue;
                }
                let pieces: Vec<&str> = line
                    .split(';')
                    .map(str::trim)
                    .filter(|p| !p.is_empty())
                    .collect();
                if pieces.is_empty() {
                    continue;
                }
                let single = pieces.len() == 1;
                // The output of the last evaluation, for the combined
                // history entry of a multi-statement script.
                let mut last_eval_output: Option<String> = None;
                for piece in &pieces {
                let piece = piece.trim();
                last_eval_output = None;

                // Graphing (ADR-0006/0014: the core samples, the frontend renders).
                // Each `graph` line overlays one more curve; the command
                // itself joins the history list like every submitted line.
                if let Some(source) = piece.strip_prefix("graph ") {
                    let source = source.trim();
                    if single {
                        s.record(piece);
                    }
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
                            // Graphing prints nothing to the answer area
                            // (ADR-0027): the command joins the history
                            // list, and the plot itself is the result.
                            result.set(String::new());
                            // Mobile convenience: the graph pane is one
                            // horizontal slide away in the stacked-pane
                            // layout — a drawn plot slides the view
                            // across so the curve is visible immediately.
                            if mobile_layout() {
                                scroll_pane.emit("graph-pane");
                                // Mobile keyboards stay open while the
                                // entry keeps focus; a drawn plot wants
                                // the screen. Drop the focus so the
                                // keyboard closes and the pane is ready
                                // for touch rotation.
                                if let Some(ta) =
                                    input_ref.cast::<web_sys::HtmlTextAreaElement>()
                                {
                                    let _ = ta.blur();
                                }
                            }
                        }
                        Err(e) => result.set(format!("error: {e}")),
                    }
                    continue;
                }

                // 3D surfaces (ADR-0015): z = f(x, y) over a square
                // domain, overlaid like curves. The command joins the
                // history list like every submitted line.
                if let Some(source) = piece.strip_prefix("graph3d ") {
                    let source = source.trim();
                    if single {
                        s.record(piece);
                    }
                    if source == "clear" {
                        surfaces.clear();
                        view_h.set(0.0);
                        view_v.set(0.0);
                        view_z.set(0.0);
                        spin_phase.set((0.0, 0.0));
                        *spin_phase_cell.borrow_mut() = (0.0, 0.0);
                        continue;
                    }
                    match epher_core::graph::sample_surface(source, 30, s.env()) {
                        Ok(fresh) => {
                            // A 3D graph drawn into an empty pane brings
                            // fresh fine-control sliders at their default
                            // 0 (ADR-0031); overlays keep the current pose.
                            if surfaces.is_empty() {
                                view_h.set(0.0);
                                view_v.set(0.0);
                                view_z.set(0.0);
                                spin_phase.set((0.0, 0.0));
                                *spin_phase_cell.borrow_mut() = (0.0, 0.0);
                            }
                            surfaces.push(fresh);
                            // Same as 2D: no answer echo, the surface is
                            // the result (ADR-0027).
                            result.set(String::new());
                            // Mobile convenience, as for 2D graphs: slide
                            // the view across to the freshly drawn pane.
                            if mobile_layout() {
                                scroll_pane.emit("graph-pane");
                                if let Some(ta) =
                                    input_ref.cast::<web_sys::HtmlTextAreaElement>()
                                {
                                    let _ = ta.blur();
                                }
                            }
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

                let out = if single {
                    s.submit(piece)
                } else {
                    s.submit_quiet(piece)
                };
                result.set(out.clone());
                last_eval_output = Some(out);
                }
                if !single {
                    // One history entry for the whole script: the line as
                    // typed, semicolons intact, with the last answer
                    // appended exactly as single statements record theirs.
                    let entry = match &last_eval_output {
                        Some(out) if !out.is_empty() => format!("{line}  {out}"),
                        _ => line.clone(),
                    };
                    s.record(&entry);
                    // `save script` persists the whole script the user
                    // entered, not just its last statement.
                    s.set_last_line(&line);
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

    let on_set_view = {
        let view_h = view_h.clone();
        let view_v = view_v.clone();
        let view_z = view_z.clone();
        let view_h_cell = view_h_cell.clone();
        let view_v_cell = view_v_cell.clone();
        Callback::from(move |(axis, v): (&'static str, f64)| {
            let v = v.clamp(-1.0, 1.0);
            match axis {
                "h" => {
                    *view_h_cell.borrow_mut() = v;
                    view_h.set(v);
                }
                "v" => {
                    *view_v_cell.borrow_mut() = v;
                    view_v.set(v);
                }
                _ => view_z.set(v),
            }
        })
    };

    // 3D orbit: drag or arrow keys rotate the view (ADR-0015). Each event
    // mutates the live cell first, so consecutive events accumulate on top
    // of each other; the state write only mirrors it for rendering.
    let on_orbit = {
        let view = view.clone();
        let view_cell = view_cell.clone();
        Callback::from(move |(dyaw, dpitch): (f64, f64)| {
            let v = *view_cell.borrow();
            let next = epher_core::graph::View3D {
                yaw: v.yaw + dyaw,
                pitch: (v.pitch + dpitch).clamp(-1.4, 1.4),
                camera: v.camera,
            };
            *view_cell.borrow_mut() = next;
            view.set(next);
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
    // The spin loop (ADR-0032): one spawned task advances the phase while
    // either rotation slider is non-zero. Under reduced motion it skips
    // entirely — the sliders then keep their static-offset meaning.
    {
        let spin_phase = spin_phase.clone();
        let spin_phase_cell = spin_phase_cell.clone();
        let view_h_cell = view_h_cell.clone();
        let view_v_cell = view_v_cell.clone();
        use_effect_with((), move |_| {
            let reduce = prefers_reduced();
            spawn_local(async move {
                let mut last: Option<f64> = None;
                loop {
                    gloo_timers::future::sleep(std::time::Duration::from_millis(33)).await;
                    let h = *view_h_cell.borrow();
                    let v = *view_v_cell.borrow();
                    if reduce || (h == 0.0 && v == 0.0) {
                        last = None;
                        continue;
                    }
                    let now = js_sys::Date::now();
                    let dt = match last {
                        Some(t) => ((now - t) / 1000.0).min(0.1),
                        None => 0.033,
                    };
                    last = Some(now);
                    let mut phase = *spin_phase_cell.borrow();
                    phase.0 += h * dt * 1.05;
                    phase.1 += v * dt * 1.05;
                    *spin_phase_cell.borrow_mut() = phase;
                    spin_phase.set(phase);
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
    // exercise), with a localized outcome message. When the pane shows 2D
    // curves they win; a surface-only pane exports the 3D document at the
    // current orbit pose (ADR-0025: the pane controls serve both kinds of
    // graph).
    let on_copy_svg = {
        let curves = graph.clone();
        let pois = pois.clone();
        let trace = trace.clone();
        let poi_markers = poi_markers.clone();
        let line_width = line_width.clone();
        let surface = surface.clone();
        let view = view.clone();
        let view_h = view_h.clone();
        let view_v = view_v.clone();
        let view_z = view_z.clone();
        let spin_phase = spin_phase.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        Callback::from(move |_| {
            let svg = if !(*curves).is_empty() {
                graph::graph_svg(&curves, &pois, *trace, *poi_markers, *line_width)
            } else if let Some(doc) = graph::graph3d_svg(
                &surface,
                &effective_view(&view, *view_h, *view_v, *view_z, *spin_phase),
                *line_width,
            ) {
                doc
            } else {
                String::new()
            };
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
    // the pocket calculator keys they are. The language itself is
    // untouched: the keypad only spells input the evaluator already reads.
    // ADR-0035: on touch layouts a press never refocuses the entry, so
    // the blur that closes the mobile keyboard also zeroes the DOM
    // selection (Chromium). The selection therefore lives in a cell —
    // mirrored by selectionchange while the entry is focused and
    // refreshed at each keypad mousedown — and a press reads the cell,
    // so the next insertion point is always immediately after what was
    // just inserted. Desktop keeps ADR-0016's focus return.
    let on_keypad = {
        let input = input.clone();
        let input_ref = input_ref.clone();
        let form_ref = form_ref.clone();
        let cursor_cell = cursor_cell.clone();
        Callback::from(move |act: KeyAction| {
            let Some(ta) = input_ref.cast::<HtmlTextAreaElement>() else {
                return;
            };
            let cursor = |v: &str| -> (usize, usize) {
                let (s, e) = *cursor_cell.borrow();
                (s.min(v.len()), e.min(v.len()))
            };
            let mut new_cursor = (0usize, 0usize);
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
                    new_cursor = (lo, lo);
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
                    new_cursor = (pos, pos);
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
                    new_cursor = (pos, pos);
                }
            }
            *cursor_cell.borrow_mut() = new_cursor;
            // ADR-0035: on touch layouts a keypad press must never
            // summon the device keyboard — the tap itself closes it,
            // and blurring makes that explicit for browsers that keep
            // focus on the entry through the tap. Desktop keeps
            // ADR-0016's rule: focus returns to the input so typing
            // continues.
            if mobile_layout() {
                let _ = ta.blur();
            } else {
                let _ = ta.focus();
            }
        })
    };

    // Mirror the entry's caret while it owns focus (ADR-0035). Keypad
    // presses read this cell when the entry is unfocused, because the
    // blur that closes the mobile keyboard also zeroes the DOM
    // selection in Chromium — without the mirror, the next press would
    // lose the insertion point the user left.
    {
        let input_ref = input_ref.clone();
        let cursor_cell = cursor_cell.clone();
        use_effect(move || {
            let window = web_sys::window().expect("window");
            let doc = window.document().expect("document");
            let value = doc.clone();
            let listener = gloo_events::EventListener::new(&doc, "selectionchange", move |_| {
                let active_ta = value
                    .active_element()
                    .and_then(|a| a.dyn_into::<HtmlTextAreaElement>().ok());
                if let (Some(ta), Some(active)) = (
                    input_ref.cast::<HtmlTextAreaElement>(),
                    active_ta,
                ) {
                    if active.as_ref() as &web_sys::Element == ta.as_ref() as &web_sys::Element {
                        let s = ta.selection_start().ok().flatten().unwrap_or(0) as usize;
                        let e = ta.selection_end().ok().flatten().unwrap_or(0) as usize;
                        *cursor_cell.borrow_mut() = (s, e);
                    }
                }
            });
            move || drop(listener)
        });
    }

    // Refresh the selection cell at each keypad mousedown (ADR-0035).
    // The mousedown handler runs before the button's default action moves
    // focus, so the entry still owns the true DOM selection — the last
    // moment it is readable. Keyboard activation (Tab to a keypad button,
    // Enter) skips mousedown and relies on the selectionchange mirror.
    let on_key_capture = {
        let input_ref = input_ref.clone();
        let cursor_cell = cursor_cell.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            let Some(ta) = input_ref.cast::<HtmlTextAreaElement>() else {
                return;
            };
            let focused = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.active_element())
                .and_then(|a| a.dyn_into::<HtmlTextAreaElement>().ok())
                .as_ref()
                == Some(&ta);
            if focused {
                let s = ta.selection_start().ok().flatten().unwrap_or(0) as usize;
                let e = ta.selection_end().ok().flatten().unwrap_or(0) as usize;
                *cursor_cell.borrow_mut() = (s, e);
            }
        })
    };

    // (The scroll_pane callback itself is defined above on_submit, so
    // the submit path can slide the view to a freshly drawn graph.)
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
        let view_h = view_h.clone();
        let view_v = view_v.clone();
        let view_z = view_z.clone();
        let spin_phase = spin_phase.clone();
        let spin_phase_cell = spin_phase_cell.clone();
        Callback::from(move |_| {
            graph.set(Vec::new());
            pois.set(Vec::new());
            surface.set(Vec::new());
            trace.set(None);
            play.set(None);
            *play_cell.borrow_mut() = None;
            *live.borrow_mut() = GraphLive::default();
            view_h.set(0.0);
            view_v.set(0.0);
            view_z.set(0.0);
            spin_phase.set((0.0, 0.0));
            *spin_phase_cell.borrow_mut() = (0.0, 0.0);
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
                    // File → Open history / Open script: two real file
                    // pickers, hidden from the tab order (reached through
                    // the menu items).
                    html! {
                        <>
                            <input
                                type="file"
                                class="visually-hidden-file"
                                ref={file_ref.clone()}
                                onchange={on_script_chosen}
                                tabindex="-1"
                                aria-hidden="true"
                            />
                            <input
                                type="file"
                                class="visually-hidden-file"
                                ref={history_ref.clone()}
                                onchange={on_history_chosen}
                                tabindex="-1"
                                aria-hidden="true"
                            />
                        </>
                    }
                }
                <nav
                    class="menubar"
                    role="menubar"
                    aria-orientation="vertical"
                    aria-label={localizer.lookup("menu")}
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
                            aria-label={localizer.lookup("menu-file")}
                            title={localizer.lookup("menu-file")}
                            class={if *menu_open == Some("file") { "menu-top open" } else { "menu-top" }}
                            onclick={{
                                let menu_open = menu_open.clone();
                                Callback::from(move |_| menu_open.set(if *menu_open == Some("file") { None } else { Some("file") }))
                            }}
                        >
                            { menu_icon(ICON_FILE) }
                        </button>
                        {
                            if *menu_open == Some("file") {
                                html! {
                                    <div class="menu-drop" role="menu" aria-label={localizer.lookup("menu-file")}>
                                        <button type="button" role="menuitem" class="menu-item"
                                            onclick={Callback::from({
                                                let menu_open = menu_open.clone();
                                                let on_open_history = on_open_history.clone();
                                                move |_| { menu_open.set(None); on_open_history.emit(()); }
                                            })}
                                        >
                                            { localizer.lookup("menu-open-history") }
                                        </button>
                                        <button type="button" role="menuitem" class="menu-item"
                                            onclick={Callback::from({
                                                let menu_open = menu_open.clone();
                                                let on_open_script = on_open_script.clone();
                                                move |_| { menu_open.set(None); on_open_script.emit(()); }
                                            })}
                                        >
                                            { localizer.lookup("menu-open-script") }
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
                            aria-label={localizer.lookup("menu-edit")}
                            title={localizer.lookup("menu-edit")}
                            class={if *menu_open == Some("edit") { "menu-top open" } else { "menu-top" }}
                            onclick={{
                                let menu_open = menu_open.clone();
                                Callback::from(move |_| menu_open.set(if *menu_open == Some("edit") { None } else { Some("edit") }))
                            }}
                        >
                            { menu_icon(ICON_EDIT) }
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
                            aria-label={localizer.lookup("menu-settings")}
                            title={localizer.lookup("menu-settings")}
                            class={if *menu_open == Some("settings") { "menu-top open" } else { "menu-top" }}
                            onclick={{
                                let menu_open = menu_open.clone();
                                Callback::from(move |_| menu_open.set(if *menu_open == Some("settings") { None } else { Some("settings") }))
                            }}
                        >
                            { menu_icon(ICON_SETTINGS) }
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
                            aria-label={localizer.lookup("menu-help")}
                            title={localizer.lookup("menu-help")}
                            class={if *menu_open == Some("help") { "menu-top open" } else { "menu-top" }}
                            onclick={{
                                let menu_open = menu_open.clone();
                                Callback::from(move |_| menu_open.set(if *menu_open == Some("help") { None } else { Some("help") }))
                            }}
                        >
                            { menu_icon(ICON_HELP) }
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
                                    let on_open_history = on_open_history.clone();
                                    move |_: web_sys::MouseEvent| on_open_history.emit(())
                                }))}>
                                    { localizer.lookup("menu-open-history") }
                                </button>
                                <button type="button" role="menuitem" class="menu-item" onclick={mobile_item(Callback::from({
                                    let on_open_script = on_open_script.clone();
                                    move |_: web_sys::MouseEvent| on_open_script.emit(())
                                }))}>
                                    { localizer.lookup("menu-open-script") }
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
                            { for session.history().iter().rev().map(|h| {
                                // Clickable history (ADR-0027): picking a
                                // line loads it into the entry, replacing
                                // whatever is there, for editing and
                                // re-running — the same gesture the TUI's
                                // history focus mode offers.
                                let on_pick = {
                                    let input = input.clone();
                                    let input_ref = input_ref.clone();
                                    let cursor_cell = cursor_cell.clone();
                                    let line = h.clone();
                                    Callback::from(move |_| {
                                        // ADR-0031: the pick loads the
                                        // expression — the recorded
                                        // answer suffix stays out of the
                                        // input so the user can edit and
                                        // re-run it.
                                        let expr = history_expression(&line).to_string();
                                        input.set(expr.clone());
                                        // The load puts the cursor at the
                                        // end of the expression (ADR-0035).
                                        *cursor_cell.borrow_mut() = (expr.len(), expr.len());
                                        if let Some(ta) =
                                            input_ref.cast::<web_sys::HtmlTextAreaElement>()
                                        {
                                            let _ = ta.focus();
                                        }
                                    })
                                };
                                html! { <li><button type="button" class="history-item" onclick={on_pick}>{ h.clone() }</button></li> }
                            }) }
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
                                            onmousedown={on_key_capture.clone()}
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
                                    <button type="button" class="pane-btn" onclick={on_copy_svg}>
                                        { localizer.lookup("graph-copy") }
                                    </button>
                                    // The graph options row (ADR-0020, ADR-0025): the
                                    // line-width slider serves both 2D curves and 3D
                                    // surfaces; the two points-of-interest toggles
                                    // (ADR-0019) belong to the 2D plot only, so they
                                    // render just when curves exist. Real form
                                    // controls — focusable and labelled — not menu
                                    // items, because they are adjustments, not commands.
                                    <div class="graph-options">
                                        {
                                            if !(*graph).is_empty() {
                                                html! {
                                                    <>
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
                                                    </>
                                                }
                                            } else {
                                                html! {}
                                            }
                                        }
                                        <label class="graph-option graph-width">
                                            <span class="graph-width-label">{ localizer.lookup("graph-width") }</span>
                                            {
                                                // ADR-0031: the slider's range is the layout's —
                                                // mobile 0–0.2 step 0.01 (thin lines), desktop
                                                // 0.1–4 step 0.1.
                                                if *is_mobile {
                                                    html! {
                                                        <input type="range" class="graph-width-slider"
                                                            min="0" max="0.2" step="0.01" value={line_width.to_string()}
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
                                                    }
                                                } else {
                                                    html! {
                                                        <input type="range" class="graph-width-slider"
                                                            min="0.1" max="4" step="0.1" value={line_width.to_string()}
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
                                                    }
                                                }
                                            }
                                            <span class="graph-width-value" aria-hidden="true">{ format!("{:.2}", *line_width) }</span>
                                        </label>
                                    </div>
                                    // The 3D fine controls (ADR-0031): three sliders above
                                    // the plot, visible only while surfaces are displayed.
                                    // Each spans −1..1, step 0.1, default 0, and updates the
                                    // plot in real time — on top of the orbit gesture.
                                    if !(*surface).is_empty() {
                                        <div class="view3d-options">
                                            { for [("h", "view-horizontal"), ("v", "view-vertical"), ("z", "view-zoom")].iter().map(|(axis, key)| {
                                                let value = match *axis {
                                                    "h" => *view_h,
                                                    "v" => *view_v,
                                                    _ => *view_z,
                                                };
                                                let on_input = {
                                                    let on_set_view = on_set_view.clone();
                                                    let axis = *axis;
                                                    Callback::from(move |e: web_sys::InputEvent| {
                                                        if let Some(el) = e
                                                            .target()
                                                            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                                        {
                                                            if let Ok(v) = el.value().parse::<f64>() {
                                                                on_set_view.emit((axis, v));
                                                            }
                                                        }
                                                    })
                                                };
                                                html! {
                                                    <label class="graph-option view3d-option">
                                                        <span>{ localizer.lookup(key) }</span>
                                                        <input type="range" class="view3d-slider"
                                                            min="-1" max="1" step="0.1" value={value.to_string()}
                                                            oninput={on_input}
                                                        />
                                                        <span class="graph-width-value" aria-hidden="true">{ format!("{value:.1}") }</span>
                                                    </label>
                                                }
                                            }) }
                                        </div>
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
                            // The fine-control sliders ride on the orbit
                            // base (ADR-0031); the rotation sliders spin
                            // the pose (ADR-0032). The pane renders the
                            // effective pose.
                            let effective =
                                effective_view(&view, *view_h, *view_v, *view_z, *spin_phase);
                            let rendered = graph::surface_svg(&surface, &effective, *line_width);
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
                                let cursor_cell = cursor_cell.clone();
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
                                                input.set(code.clone());
                                                // The load puts the cursor at the
                                                // end of the code (ADR-0035).
                                                *cursor_cell.borrow_mut() = (code.len(), code.len());
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
