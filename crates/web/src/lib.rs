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
    analyze, free_names, parse_graph_source, sample_data_plot, sample_spec, CurveKind, CurveSpec,
    DataPlot, InterestPoint, SampledCurve,
};
use epher_core::{history_expression, CatalogKind, DisplayPrefs, Notation, Session, Value};
use epher_i18n::Localizer;
use epher_shell::{classify, message, prepare};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{Element, HtmlElement, HtmlInputElement, HtmlTextAreaElement};
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
/// The slider span for a constant whose value is `v`: the base −10..10
/// window while the value lives inside it, otherwise a tight v±2 window.
/// A raw `min(-10, v-2) .. max(10, v+2)` union turns a Julian Date
/// (≈ 2.46e6) into a 2.46-million-wide slider no one can drag, and play
/// would wrap from v to −10 the first cycle. The window changes ONLY
/// outside the base span (ADR-0055): an earlier ±8 cutoff let the window
/// chase the value between 8 and 10, so the thumb jumped back mid-track
/// while dragging or playing instead of travelling to the end and
/// wrapping cleanly. The tight window keeps the ADR-0015 play cycle
/// honest for large constants: step 0.1 over v±2 loops in ≈ 5 s.
fn slider_span(v: f64) -> (f64, f64) {
    if (-10.0..=10.0).contains(&v) {
        (-10.0, 10.0)
    } else {
        (v - 2.0, v + 2.0)
    }
}

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
            CurveKind::Implicit(e) => visit(e),
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

/// The constants a 3D curve set references; the 3D pane's sliders
/// (ADR-0054).
fn curve3d_slider_names(
    curves: &[epher_core::graph::SpaceCurve],
    session: &Session,
) -> Vec<String> {
    let mut names = std::collections::BTreeSet::new();
    for c in curves {
        if let Ok((x, y, z, _)) = epher_core::graph::parse_space_curve_source(&c.source) {
            let mut found = std::collections::BTreeSet::new();
            free_names(&x, &mut found);
            free_names(&y, &mut found);
            free_names(&z, &mut found);
            for n in found {
                if session.const_sources().contains_key(&n) {
                    names.insert(n);
                }
            }
        }
    }
    names.into_iter().collect()
}

/// The constants the solar pane's time expression references — the source
/// is stored as written (e.g. `t` or `now() + 10`), so its free names come
/// straight from the expression tree (ADR-0037).
fn solar_slider_names(source: &str, session: &Session) -> Vec<String> {
    let mut names = std::collections::BTreeSet::new();
    if let Ok(expr) = epher_core::parse(source) {
        let mut found = std::collections::BTreeSet::new();
        free_names(&expr, &mut found);
        for n in found {
            if session.const_sources().contains_key(&n) {
                names.insert(n);
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

/// Re-sample every 3D parametric curve against the current environment
/// (ADR-0054): a moved constant reshapes the curve.
fn resample_space_curves(curves: &mut [epher_core::graph::SpaceCurve], session: &Session) {
    for c in curves.iter_mut() {
        if let Ok(fresh) = epher_core::graph::sample_space_curve(&c.source, 240, session.env()) {
            *c = fresh;
        }
    }
}

/// Does a space curve's source reference this name? (The animation
/// tick re-samples only the curves that do.)
fn space_curve_references(curve: &epher_core::graph::SpaceCurve, name: &str) -> bool {
    match epher_core::graph::parse_space_curve_source(&curve.source) {
        Ok((x, y, z, _)) => {
            let mut found = std::collections::BTreeSet::new();
            free_names(&x, &mut found);
            free_names(&y, &mut found);
            free_names(&z, &mut found);
            found.contains(name)
        }
        Err(_) => false,
    }
}

/// Rebuild the solar system scene when its time expression references a
/// session constant - `const t = now(); solar3d t` replays through
/// the existing playback transport (ADR-0037). A scene whose expression
/// mentions no constant never rebuilds.
fn resample_solar(
    solar: &mut Option<epher_core::astro::SolarScene>,
    source: &Option<String>,
    session: &Session,
) {
    let Some(_scene) = solar.as_ref() else {
        return;
    };
    let Some(src) = source.as_deref() else {
        return;
    };
    let mut names = std::collections::BTreeSet::new();
    if let Ok(expr) = epher_core::parse(src) {
        epher_core::graph::free_names(&expr, &mut names);
    }
    if !names.iter().any(|n| session.env().constant(n).is_some()) {
        return;
    }
    if let Ok(jd) = epher_core::astro::eval_jd(src, session.env()) {
        if let Ok(fresh) = epher_core::astro::solar_scene(jd) {
            *solar = Some(fresh);
        }
    }
}

/// Does the curve's expression reference `name`? The animation tick only
/// re-samples what moves (ADR-0015): curves that do not mention the
/// animated constant keep their samples.
fn curve_references(c: &SampledCurve, name: &str) -> bool {
    let mut names = std::collections::BTreeSet::new();
    match &c.kind {
        epher_core::graph::CurveKind::Cartesian(e) => epher_core::graph::free_names(e, &mut names),
        epher_core::graph::CurveKind::Parametric { x, y } => {
            epher_core::graph::free_names(x, &mut names);
            epher_core::graph::free_names(y, &mut names);
        }
        epher_core::graph::CurveKind::Polar(e) => epher_core::graph::free_names(e, &mut names),
        epher_core::graph::CurveKind::Implicit(e) => epher_core::graph::free_names(e, &mut names),
    }
    names.contains(name)
}

/// The surface counterpart of [`curve_references`].
fn surface_references(surface: &epher_core::graph::Surface, name: &str) -> bool {
    let Ok((expr, _)) = epher_core::graph::parse_surface_source(&surface.source) else {
        return true; // unknown: re-sample to be safe
    };
    let mut names = std::collections::BTreeSet::new();
    epher_core::graph::free_names(&expr, &mut names);
    names.contains(name)
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
                curve: p.curve,
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

/// The FTL key of a key's meaning (ADR-0039): `key-hint-*`. Empty for
/// the self-evident digit keys, whose labels speak for themselves.
struct KeyDef {
    label: &'static str,
    act: KeyAction,
    cls: &'static str,
    hint: &'static str,
}

struct TabDef {
    id: &'static str,
    label: &'static str,
    i18n: &'static str,
    keys: &'static [KeyDef],
}

const fn key(label: &'static str, act: KeyAction, cls: &'static str, hint: &'static str) -> KeyDef {
    KeyDef {
        label,
        act,
        cls,
        hint,
    }
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
            key("C", KeyAction::Clear, "act", "key-hint-clear"),
            key("⌫", KeyAction::Backspace, "act", "key-hint-backspace"),
            key("(", KeyAction::Text("("), "op", "key-hint-lpar"),
            key(")", KeyAction::Text(")"), "op", "key-hint-rpar"),
            key("÷", KeyAction::Text("/"), "op", "key-hint-div"),
            key("7", KeyAction::Text("7"), "", ""),
            key("8", KeyAction::Text("8"), "", ""),
            key("9", KeyAction::Text("9"), "", ""),
            key("×", KeyAction::Text("*"), "op", "key-hint-mul"),
            key("−", KeyAction::Text("-"), "op", "key-hint-sub"),
            key("4", KeyAction::Text("4"), "", ""),
            key("5", KeyAction::Text("5"), "", ""),
            key("6", KeyAction::Text("6"), "", ""),
            key("+", KeyAction::Text("+"), "op", "key-hint-add"),
            key("^", KeyAction::Text("^"), "op", "key-hint-pow"),
            key("1", KeyAction::Text("1"), "", ""),
            key("2", KeyAction::Text("2"), "", ""),
            key("3", KeyAction::Text("3"), "", ""),
            key(";", KeyAction::Text(";"), "op", "key-hint-semi"),
            key(",", KeyAction::Text(","), "op", "key-hint-comma"),
            key("0", KeyAction::Text("0"), "", ""),
            key(".", KeyAction::Text("."), "", ""),
            // The newline key (ADR-0016 amendment): ans lives on the
            // pigreco tab, and a real newline in the entry is how
            // multi-line scripts are composed on touch.
            key("\u{23CE}", KeyAction::Text("\n"), "op", "key-hint-newline"),
            key("=", KeyAction::Submit, "eq", "key-hint-equals"),
        ],
    },
    TabDef {
        id: "trig",
        label: "trig",
        i18n: "keypad-tab-trig",
        keys: &[
            key("sin", KeyAction::Call("sin"), "fn", "key-hint-sin"),
            key("cos", KeyAction::Call("cos"), "fn", "key-hint-cos"),
            key("tan", KeyAction::Call("tan"), "fn", "key-hint-tan"),
            key("asin", KeyAction::Call("asin"), "fn", "key-hint-asin"),
            key("acos", KeyAction::Call("acos"), "fn", "key-hint-acos"),
            key("atan", KeyAction::Call("atan"), "fn", "key-hint-atan"),
            key("sinh", KeyAction::Call("sinh"), "fn", "key-hint-sinh"),
            key("cosh", KeyAction::Call("cosh"), "fn", "key-hint-cosh"),
            key("tanh", KeyAction::Call("tanh"), "fn", "key-hint-tanh"),
            key("asinh", KeyAction::Call("asinh"), "fn", "key-hint-asinh"),
            key("acosh", KeyAction::Call("acosh"), "fn", "key-hint-acosh"),
            key("atanh", KeyAction::Call("atanh"), "fn", "key-hint-atanh"),
            key("deg", KeyAction::Call("deg"), "fn", "key-hint-deg"),
            key("rad", KeyAction::Call("rad"), "fn", "key-hint-rad"),
            key("atan2", KeyAction::Call("atan2"), "fn", "key-hint-atan2"),
        ],
    },
    TabDef {
        id: "func",
        label: "ƒ",
        i18n: "keypad-tab-func",
        keys: &[
            key("ln", KeyAction::Call("ln"), "fn", "key-hint-ln"),
            key("log", KeyAction::Call("log"), "fn", "key-hint-log"),
            key("log2", KeyAction::Call("log2"), "fn", "key-hint-log2"),
            key("logb", KeyAction::Call("logb"), "fn", "key-hint-logb"),
            key("exp", KeyAction::Call("exp"), "fn", "key-hint-exp"),
            key("sqrt", KeyAction::Call("sqrt"), "fn", "key-hint-sqrt"),
            key("cbrt", KeyAction::Call("cbrt"), "fn", "key-hint-cbrt"),
            key("root", KeyAction::Call("root"), "fn", "key-hint-root"),
            key("hypot", KeyAction::Call("hypot"), "fn", "key-hint-hypot"),
            key("abs", KeyAction::Call("abs"), "fn", "key-hint-abs"),
            key("floor", KeyAction::Call("floor"), "fn", "key-hint-floor"),
            key("ceil", KeyAction::Call("ceil"), "fn", "key-hint-ceil"),
            key("round", KeyAction::Call("round"), "fn", "key-hint-round"),
            key("trunc", KeyAction::Call("trunc"), "fn", "key-hint-trunc"),
            key("sign", KeyAction::Call("sign"), "fn", "key-hint-sign"),
            key("min", KeyAction::Call("min"), "fn", "key-hint-min"),
            key("max", KeyAction::Call("max"), "fn", "key-hint-max"),
            // Complex parts and calculus (ADR-0043, ADR-0055 keypad).
            key("re", KeyAction::Call("re"), "fn", "key-hint-re"),
            key("im", KeyAction::Call("im"), "fn", "key-hint-im"),
            key("arg", KeyAction::Call("arg"), "fn", "key-hint-arg"),
            key("conj", KeyAction::Call("conj"), "fn", "key-hint-conj"),
            key(
                "derivative",
                KeyAction::Call("derivative"),
                "fn",
                "key-hint-derivative",
            ),
            key(
                "integral",
                KeyAction::Call("integral"),
                "fn",
                "key-hint-integral",
            ),
        ],
    },
    TabDef {
        id: "num",
        label: "nΣ",
        i18n: "keypad-tab-num",
        keys: &[
            key("gcd", KeyAction::Call("gcd"), "fn", "key-hint-gcd"),
            key("lcm", KeyAction::Call("lcm"), "fn", "key-hint-lcm"),
            key("mod", KeyAction::Call("mod"), "fn", "key-hint-mod"),
            key("fact", KeyAction::Call("fact"), "fn", "key-hint-fact"),
            key("ncr", KeyAction::Call("ncr"), "fn", "key-hint-ncr"),
            key("npr", KeyAction::Call("npr"), "fn", "key-hint-npr"),
            key("sum", KeyAction::Call("sum"), "fn", "key-hint-sum"),
            key(
                "product",
                KeyAction::Call("product"),
                "fn",
                "key-hint-product",
            ),
            key("mean", KeyAction::Call("mean"), "fn", "key-hint-mean"),
            key("median", KeyAction::Call("median"), "fn", "key-hint-median"),
            key(
                "variance",
                KeyAction::Call("variance"),
                "fn",
                "key-hint-variance",
            ),
            key("stdev", KeyAction::Call("stdev"), "fn", "key-hint-stdev"),
            // The percent key (ADR-0042): the transparent /100 suffix.
            // It lives here, not on the digits tab: that bank is exactly
            // full (the = key spans two cells of the five-row grid), so
            // any addition scrolls the 123 tab (ADR-0042 amendment).
            key("%", KeyAction::Text("%"), "op", "key-hint-percent"),
            // The seeded-random keys (ADR-0045, ADR-0055 keypad): the
            // generator family joins the statistics bank.
            key(
                "randint",
                KeyAction::Call("randint"),
                "fn",
                "key-hint-randint",
            ),
            key("random", KeyAction::Call("random"), "fn", "key-hint-random"),
            key(
                "randseed",
                KeyAction::Call("randseed"),
                "fn",
                "key-hint-randseed",
            ),
            key("randn", KeyAction::Call("randn"), "fn", "key-hint-randn"),
        ],
    },
    TabDef {
        id: "data",
        label: "data",
        i18n: "keypad-tab-data",
        keys: &[
            // The data-type keys (ADR-0055 keypad): matrices `[[1,2],[3,4]]`,
            // lists `{1,2,3}`, and strings `"text"` are typed from one bank
            // (the 123 tab is untouched and holds no brackets).
            key("[", KeyAction::Text("["), "op", "key-hint-lsqb"),
            key("]", KeyAction::Text("]"), "op", "key-hint-rsqb"),
            key("{", KeyAction::Text("{"), "op", "key-hint-lbrace"),
            key("}", KeyAction::Text("}"), "op", "key-hint-rbrace"),
            key("\"", KeyAction::Text("\""), "op", "key-hint-quote"),
            key("det", KeyAction::Call("det"), "fn", "key-hint-det"),
            key("inv", KeyAction::Call("inv"), "fn", "key-hint-inv"),
            key(
                "transpose",
                KeyAction::Call("transpose"),
                "fn",
                "key-hint-transpose",
            ),
            key("rref", KeyAction::Call("rref"), "fn", "key-hint-rref"),
            key("dim", KeyAction::Call("dim"), "fn", "key-hint-dim"),
            key("str", KeyAction::Call("str"), "fn", "key-hint-str"),
            key("len", KeyAction::Call("len"), "fn", "key-hint-len"),
        ],
    },
    TabDef {
        id: "dist",
        label: "dist",
        i18n: "keypad-tab-dist",
        keys: &[
            // The stats-class keys (ADR-0054, ADR-0055 keypad): fits,
            // tests, and the distribution family in one bank.
            key("linreg", KeyAction::Call("linreg"), "fn", "key-hint-linreg"),
            key(
                "quadreg",
                KeyAction::Call("quadreg"),
                "fn",
                "key-hint-quadreg",
            ),
            key("expreg", KeyAction::Call("expreg"), "fn", "key-hint-expreg"),
            key("powreg", KeyAction::Call("powreg"), "fn", "key-hint-powreg"),
            key("logreg", KeyAction::Call("logreg"), "fn", "key-hint-logreg"),
            key("anova", KeyAction::Call("anova"), "fn", "key-hint-anova"),
            key(
                "ttestpaired",
                KeyAction::Call("ttestpaired"),
                "fn",
                "key-hint-ttestpaired",
            ),
            key(
                "normcdf",
                KeyAction::Call("normcdf"),
                "fn",
                "key-hint-normcdf",
            ),
            key(
                "normpdf",
                KeyAction::Call("normpdf"),
                "fn",
                "key-hint-normpdf",
            ),
            key(
                "invnorm",
                KeyAction::Call("invnorm"),
                "fn",
                "key-hint-invnorm",
            ),
            key("tcdf", KeyAction::Call("tcdf"), "fn", "key-hint-tcdf"),
            key("tpdf", KeyAction::Call("tpdf"), "fn", "key-hint-tpdf"),
            key("invt", KeyAction::Call("invt"), "fn", "key-hint-invt"),
            key("ttest", KeyAction::Call("ttest"), "fn", "key-hint-ttest"),
            key(
                "tinterval",
                KeyAction::Call("tinterval"),
                "fn",
                "key-hint-tinterval",
            ),
            key("ztest", KeyAction::Call("ztest"), "fn", "key-hint-ztest"),
            key(
                "zinterval",
                KeyAction::Call("zinterval"),
                "fn",
                "key-hint-zinterval",
            ),
            key(
                "binomcdf",
                KeyAction::Call("binomcdf"),
                "fn",
                "key-hint-binomcdf",
            ),
            key(
                "binompdf",
                KeyAction::Call("binompdf"),
                "fn",
                "key-hint-binompdf",
            ),
            key(
                "poissoncdf",
                KeyAction::Call("poissoncdf"),
                "fn",
                "key-hint-poissoncdf",
            ),
            key(
                "poissonpdf",
                KeyAction::Call("poissonpdf"),
                "fn",
                "key-hint-poissonpdf",
            ),
            key(
                "chi2cdf",
                KeyAction::Call("chi2cdf"),
                "fn",
                "key-hint-chi2cdf",
            ),
            key(
                "chi2pdf",
                KeyAction::Call("chi2pdf"),
                "fn",
                "key-hint-chi2pdf",
            ),
            key(
                "invchi2",
                KeyAction::Call("invchi2"),
                "fn",
                "key-hint-invchi2",
            ),
        ],
    },
    TabDef {
        id: "fin",
        label: "$",
        i18n: "keypad-tab-fin",
        keys: &[
            // The finance keys (ADR-0050, ADR-0055 keypad).
            key("tvm_n", KeyAction::Call("tvm_n"), "fn", "key-hint-tvm_n"),
            key("tvm_i", KeyAction::Call("tvm_i"), "fn", "key-hint-tvm_i"),
            key("tvm_pv", KeyAction::Call("tvm_pv"), "fn", "key-hint-tvm_pv"),
            key(
                "tvm_pmt",
                KeyAction::Call("tvm_pmt"),
                "fn",
                "key-hint-tvm_pmt",
            ),
            key("tvm_fv", KeyAction::Call("tvm_fv"), "fn", "key-hint-tvm_fv"),
            key("npv", KeyAction::Call("npv"), "fn", "key-hint-npv"),
            key("irr", KeyAction::Call("irr"), "fn", "key-hint-irr"),
            key("amort", KeyAction::Call("amort"), "fn", "key-hint-amort"),
            key(
                "compound_interest",
                KeyAction::Call("compound_interest"),
                "fn",
                "key-hint-compound_interest",
            ),
            key(
                "simple_interest",
                KeyAction::Call("simple_interest"),
                "fn",
                "key-hint-simple_interest",
            ),
        ],
    },
    TabDef {
        id: "conv",
        label: "0x",
        i18n: "keypad-tab-conv",
        keys: &[
            key("frac", KeyAction::Call("frac"), "fn", "key-hint-frac"),
            key("dec", KeyAction::Call("dec"), "fn", "key-hint-dec"),
            key("big", KeyAction::Call("big"), "fn", "key-hint-big"),
            key("bin", KeyAction::Call("bin"), "fn", "key-hint-bin"),
            key("oct", KeyAction::Call("oct"), "fn", "key-hint-oct"),
            key("hex", KeyAction::Call("hex"), "fn", "key-hint-hex"),
            key("!", KeyAction::Text("!"), "fn", "key-hint-fact"),
        ],
    },
    TabDef {
        id: "const",
        label: "π∇",
        i18n: "keypad-tab-const",
        keys: &[
            key("pi", KeyAction::Text("pi"), "fn", "key-hint-pi"),
            key("e", KeyAction::Text("e"), "fn", "key-hint-e"),
            key("tau", KeyAction::Text("tau"), "fn", "key-hint-tau"),
            key("phi", KeyAction::Text("phi"), "fn", "key-hint-phi"),
            key("x", KeyAction::Text("x"), "fn", "key-hint-x"),
            key("t", KeyAction::Text("t"), "fn", "key-hint-t"),
            key("ans", KeyAction::Text("ans"), "fn", "key-hint-ans"),
            key("graph", KeyAction::Text("graph "), "fn", "key-hint-graph"),
            key(
                "graph3d",
                KeyAction::Text("graph3d "),
                "fn",
                "key-hint-graph3d",
            ),
            key(
                "solar3d",
                KeyAction::Text("solar3d "),
                "fn",
                "key-hint-solar3d",
            ),
            key("table", KeyAction::Text("table "), "fn", "key-hint-table"),
        ],
    },
    TabDef {
        id: "astro",
        label: "☉",
        i18n: "keypad-tab-astro",
        keys: &[
            // time (ADR-0037)
            key("jd", KeyAction::Call("jd"), "fn", "key-hint-jd"),
            key("mjd", KeyAction::Call("mjd"), "fn", "key-hint-mjd"),
            key("now", KeyAction::Text("now"), "fn", "key-hint-now"),
            key(
                "delta_t",
                KeyAction::Call("delta_t"),
                "fn",
                "key-hint-delta_t",
            ),
            key("lst", KeyAction::Call("lst"), "fn", "key-hint-lst"),
            // angles
            key(
                "hms2deg",
                KeyAction::Call("hms2deg"),
                "fn",
                "key-hint-hms2deg",
            ),
            key(
                "dms2deg",
                KeyAction::Call("dms2deg"),
                "fn",
                "key-hint-dms2deg",
            ),
            key(
                "deg2hms",
                KeyAction::Call("deg2hms"),
                "fn",
                "key-hint-deg2hms",
            ),
            key(
                "deg2dms",
                KeyAction::Call("deg2dms"),
                "fn",
                "key-hint-deg2dms",
            ),
            key("kepler", KeyAction::Call("kepler"), "fn", "key-hint-kepler"),
            // positions
            key("ra", KeyAction::Call("ra"), "fn", "key-hint-ra"),
            key("decl", KeyAction::Call("decl"), "fn", "key-hint-decl"),
            key("dist", KeyAction::Call("dist"), "fn", "key-hint-dist"),
            key("alt", KeyAction::Call("alt"), "fn", "key-hint-alt"),
            key("az", KeyAction::Call("az"), "fn", "key-hint-az"),
            // events and brightness
            key("rise", KeyAction::Call("rise"), "fn", "key-hint-rise"),
            key("set", KeyAction::Call("set"), "fn", "key-hint-set"),
            key(
                "transit",
                KeyAction::Call("transit"),
                "fn",
                "key-hint-transit",
            ),
            key("mag", KeyAction::Call("mag"), "fn", "key-hint-mag"),
            key("phase", KeyAction::Call("phase"), "fn", "key-hint-phase"),
            key("illum", KeyAction::Call("illum"), "fn", "key-hint-illum"),
            key("diam", KeyAction::Call("diam"), "fn", "key-hint-diam"),
            key(
                "airmass",
                KeyAction::Call("airmass"),
                "fn",
                "key-hint-airmass",
            ),
            key("dawes", KeyAction::Call("dawes"), "fn", "key-hint-dawes"),
            key(
                "dist_mod",
                KeyAction::Call("dist_mod"),
                "fn",
                "key-hint-dist_mod",
            ),
            // flux and seasons
            key("mag2jy", KeyAction::Call("mag2jy"), "fn", "key-hint-mag2jy"),
            key("jy2mag", KeyAction::Call("jy2mag"), "fn", "key-hint-jy2mag"),
            key(
                "march_equinox",
                KeyAction::Call("march_equinox"),
                "fn",
                "key-hint-march_equinox",
            ),
            key(
                "june_solstice",
                KeyAction::Call("june_solstice"),
                "fn",
                "key-hint-june_solstice",
            ),
            key(
                "september_equinox",
                KeyAction::Call("september_equinox"),
                "fn",
                "key-hint-september_equinox",
            ),
            key(
                "december_solstice",
                KeyAction::Call("december_solstice"),
                "fn",
                "key-hint-december_solstice",
            ),
            // constants
            key("au", KeyAction::Text("au"), "fn", "key-hint-au"),
            key("pc", KeyAction::Text("pc"), "fn", "key-hint-pc"),
            key("ly", KeyAction::Text("ly"), "fn", "key-hint-ly"),
            key("c", KeyAction::Text("c"), "fn", "key-hint-c"),
            key("g", KeyAction::Text("g"), "fn", "key-hint-g"),
            key("h", KeyAction::Text("h"), "fn", "key-hint-h"),
            key("h_bar", KeyAction::Text("h_bar"), "fn", "key-hint-h_bar"),
            key("k_b", KeyAction::Text("k_b"), "fn", "key-hint-k_b"),
            key(
                "sigma_sb",
                KeyAction::Text("sigma_sb"),
                "fn",
                "key-hint-sigma_sb",
            ),
            key("m_sun", KeyAction::Text("m_sun"), "fn", "key-hint-m_sun"),
            key("r_sun", KeyAction::Text("r_sun"), "fn", "key-hint-r_sun"),
            key("l_sun", KeyAction::Text("l_sun"), "fn", "key-hint-l_sun"),
            key(
                "m_earth",
                KeyAction::Text("m_earth"),
                "fn",
                "key-hint-m_earth",
            ),
            key(
                "r_earth",
                KeyAction::Text("r_earth"),
                "fn",
                "key-hint-r_earth",
            ),
            // unit suffixes (the leading space is part of the literal)
            key("AU", KeyAction::Text(" AU"), "fn", "key-hint-u-au"),
            key("pc", KeyAction::Text(" pc"), "fn", "key-hint-u-pc"),
            key("ly", KeyAction::Text(" ly"), "fn", "key-hint-u-ly"),
            key("deg", KeyAction::Text(" deg"), "fn", "key-hint-u-deg"),
            key(
                "arcmin",
                KeyAction::Text(" arcmin"),
                "fn",
                "key-hint-u-arcmin",
            ),
            key(
                "arcsec",
                KeyAction::Text(" arcsec"),
                "fn",
                "key-hint-u-arcsec",
            ),
            key("min", KeyAction::Text(" min"), "fn", "key-hint-u-min"),
            key("hr", KeyAction::Text(" hr"), "fn", "key-hint-u-hr"),
            key("d", KeyAction::Text(" d"), "fn", "key-hint-u-d"),
            key("yr", KeyAction::Text(" yr"), "fn", "key-hint-u-yr"),
            key("Jy", KeyAction::Text(" Jy"), "fn", "key-hint-u-jy"),
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

/// One active grab-bar drag (ADR-0060): where the gesture began, the
/// heights it works between, and the newest sample for the release
/// flick's velocity. Lives in a cell the pointer handlers read and
/// write without re-rendering — the dragged height lands on the DOM
/// directly.
struct KeypadDrag {
    pointer_id: i32,
    y0: f64,
    start_h: f64,
    open_h: f64,
    last_y: f64,
    last_t: f64,
    moved: bool,
}

/// The drag's release decision (ADR-0060): collapse when the bar moved
/// down past half the keypad's height, or when the last samples show a
/// downward flick faster than 0.5 px/ms; an upward flick opens it; a
/// slow release springs back to wherever the majority of the keypad
/// already is. Pure so the threshold contract is testable off-wasm.
fn keypad_snap(current_h: f64, open_h: f64, velocity_px_per_ms: f64) -> bool {
    if velocity_px_per_ms >= 0.5 {
        return true;
    }
    if velocity_px_per_ms <= -0.5 {
        return false;
    }
    current_h * 2.0 < open_h
}

/// The stored line widths (ADR-0035 amendment, ADR-0055 range): 2D and
/// 3D remember their values independently on every display — the 2D key
/// falls back to the legacy shared key and clamps into the 2D range
/// (0–4), the 3D key falls back to the same legacy key and clamps into
/// the 3D range of the layout in question (0–0.2 on the touch layout,
/// ADR-0035; 0–0.4 on the desktop, ADR-0055). The layout question
/// arrives as an argument so the clamp stays testable off-wasm. Each
/// kind's plot renders with its own value; the toolbar shows and edits
/// the kind in view.
fn stored_widths(store: &web_sys::Storage, mobile: bool) -> (Option<f64>, Option<f64>) {
    let read = |key: &str| {
        store
            .get_item(key)
            .ok()
            .flatten()
            .and_then(|v| v.parse::<f64>().ok())
    };
    let (w3d_max, _) = graph::three_d_width_range(mobile);
    let legacy = read("epher-line-width");
    let w2d = read("epher-line-width-2d")
        .or(legacy)
        .map(|w| w.clamp(0.0, 4.0));
    let w3d = read("epher-line-width-3d")
        .or(legacy)
        .map(|w| w.clamp(0.0, w3d_max));
    (w2d, w3d)
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

// ===== zoom (ADR-0038) =====

/// The window a wheel notch or pinch produces: `factor` scales the span
/// around the anchor (a data x), keeping the anchor's pixel spot fixed.
/// The span stays within `[base × 1e-9, base × 1e9]` - deep enough that
/// float sampling, not the clamp, ends the journey.
pub fn anchored_window(cur: (f64, f64), anchor: f64, factor: f64, base_span: f64) -> (f64, f64) {
    let factor = factor.clamp(0.2, 5.0);
    let span = ((cur.1 - cur.0) * factor).clamp(base_span * 1e-9, base_span * 1e9);
    let lo = anchor - (anchor - cur.0) * (span / (cur.1 - cur.0));
    (lo, lo + span)
}

/// The zoom slider position (−1..1) representing the current window: the
/// span relative to the base window, log-scaled. −1 is 100× wider (every
/// object fits), +1 is 100× narrower (a single object fills the pane).
pub fn zoom_slider_value(window: Option<(f64, f64)>, base: (f64, f64)) -> f64 {
    let Some((lo, hi)) = window else {
        return 0.0;
    };
    let ratio = (hi - lo) / (base.1 - base.0).max(1e-12);
    (-ratio.log10() / 2.0).clamp(-1.0, 1.0)
}

/// The window a slider value picks: the base span scaled by 10^(−2z)
/// around the current center.
pub fn slider_window(z: f64, base: (f64, f64), center: f64) -> (f64, f64) {
    let span = (base.1 - base.0) * 10f64.powf(-2.0 * z.clamp(-1.0, 1.0));
    (center - span / 2.0, center + span / 2.0)
}

/// Apple platforms (iOS, iPadOS, macOS) get the SF Symbols glyphs for
/// copy and share; everything else gets the squared Android/Windows
/// marks (ADR-0038). The share SHEET is always the OS's own -
/// `navigator.share` invokes it - the artwork just matches it.
fn is_apple_platform() -> bool {
    web_sys::window()
        .and_then(|w| w.navigator().user_agent().ok())
        .map(|ua| {
            let ua = ua.to_ascii_lowercase();
            ua.contains("mac") || ua.contains("iphone") || ua.contains("ipad")
        })
        .unwrap_or(false)
}

/// A 24×24 stroke icon in the menu-icon style.
fn platform_icon(class: &'static str, inner: &'static str) -> yew::Html {
    yew::Html::from_html_unchecked(
        format!(
            "<svg class=\"{class}\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\" focusable=\"false\">{inner}</svg>"
        )
        .into(),
    )
}

/// The standard copy icon for this device (ADR-0038): the points-of-
/// interests heading's copy button, and the graph pane's Copy SVG
/// button (ADR-0040, which turned the toolbar's text labels into
/// icons).
/// One autocomplete suggestion (ADR-0042): a builtin, a user-defined
/// function or constant, or a plain variable, with its localized hint
/// when one exists.
#[derive(Clone, PartialEq)]
struct Suggestion {
    name: String,
    kind: CatalogKind,
    hint: String,
}

/// The open state of the suggestion list: where the typed word starts in
/// the entry, the caret it grew to, the matches, and the highlighted one.
#[derive(Clone, PartialEq)]
struct AutocompleteState {
    word_start: usize,
    caret: usize,
    items: Vec<Suggestion>,
    selected: usize,
}

/// Convert a character index into a byte index for splicing Rust
/// strings (cursors and DOM selections count characters; slices index
/// bytes). The naive `&v[..i]` panicked whenever text before the caret
/// held a multi-byte letter like é (the ADR-0055 strings round).
fn char_byte(v: &str, chars: usize) -> usize {
    v.char_indices()
        .nth(chars.min(v.chars().count()))
        .map(|(b, _)| b)
        .unwrap_or(v.len())
}

/// The word being completed at `caret` in `value`: `Some((start, word))`
/// when the caret sits at the end of a name-shaped run. `caret` is a
/// character index; all slicing goes through [`char_byte`].
fn word_at(value: &str, caret: usize) -> Option<(usize, String)> {
    let caret = char_byte(value, caret);
    let head = &value[..caret];
    if !head
        .chars()
        .next_back()
        .is_some_and(|c| c.is_alphanumeric() || c == '_')
    {
        return None;
    }
    let start = head
        .char_indices()
        .rev()
        .take_while(|(_, c)| c.is_alphanumeric() || *c == '_')
        .last()
        .map(|(i, _)| i)
        .unwrap_or(caret);
    Some((start, head[start..].to_string()))
}

/// Suggestions for the word ending at `caret`: prefix matches over the
/// session's own names (which shadow) plus the builtin catalog, capped at
/// eight and sorted. `None` when there is nothing to suggest.
fn suggestions_for(
    value: &str,
    caret: usize,
    session: &Session,
    localizer: &Localizer,
) -> Option<(usize, Vec<Suggestion>)> {
    let (start, word) = word_at(value, caret)?;
    let mut items: Vec<Suggestion> = Vec::new();
    let mut push = |name: &str, kind: CatalogKind| {
        if !name.starts_with(&word) || items.iter().any(|s| s.name == name) {
            return;
        }
        let hint_key = format!("key-hint-{name}");
        let hint = localizer.lookup(&hint_key);
        let hint = if hint == hint_key {
            String::new()
        } else {
            hint
        };
        items.push(Suggestion {
            name: name.to_string(),
            kind,
            hint,
        });
    };
    for name in session.def_sources().keys() {
        push(name, CatalogKind::Function);
    }
    for name in session.const_sources().keys() {
        push(name, CatalogKind::Constant);
    }
    for name in session.bindings().keys() {
        if name != "ans" {
            push(name, CatalogKind::Constant);
        }
    }
    for entry in epher_core::catalog() {
        push(entry.name, entry.kind);
    }
    items.sort_by(|a, b| a.name.cmp(&b.name));
    items.truncate(8);
    if items.is_empty() {
        None
    } else {
        Some((start, items))
    }
}

/// Apply a suggestion: replace the typed prefix with the full name
/// (functions gain an open paren), and report the new value and caret.
fn apply_suggestion(value: &str, state: &AutocompleteState, item: &Suggestion) -> (String, usize) {
    let mut out = String::with_capacity(value.len() + item.name.len() + 1);
    out.push_str(&value[..char_byte(value, state.word_start)]);
    out.push_str(&item.name);
    let mut caret = state.word_start + item.name.len();
    if item.kind == CatalogKind::Function {
        out.push('(');
        caret += 1;
    }
    out.push_str(&value[char_byte(value, state.caret)..]);
    (out, caret)
}

/// Accept the suggestion at `selected`: splice it into the entry, put the
/// caret after it, and close the list.
fn accept_suggestion(
    state: &AutocompleteState,
    selected: usize,
    input: &UseStateHandle<String>,
    input_ref: &yew::NodeRef,
    autocomplete: &UseStateHandle<Option<AutocompleteState>>,
) {
    let Some(item) = state.items.get(selected) else {
        autocomplete.set(None);
        return;
    };
    let (new_value, caret) = apply_suggestion(input, state, item);
    input.set(new_value.clone());
    autocomplete.set(None);
    if let Some(ta) = input_ref.cast::<HtmlTextAreaElement>() {
        ta.set_value(&new_value);
        ta.set_selection_start(Some(caret as u32)).ok();
        ta.set_selection_end(Some(caret as u32)).ok();
    }
}

/// ADR-0042 auto-ans: an operator typed into an empty entry means
/// "continue from the previous answer", so `ans` is inserted first
/// (the SpeedCrunch/NumWorks behavior). Digits, names, and `(` start
/// fresh expressions and never trigger it.
/// The store spelling of a notation (ADR-0043): the same three values
/// the Settings menu and the TUI cycle through.
fn notation_of(notation: Notation) -> &'static str {
    match notation {
        Notation::Auto => "auto",
        Notation::Scientific => "scientific",
        Notation::Engineering => "engineering",
    }
}

fn wants_auto_ans(token: &str) -> bool {
    matches!(
        token.chars().next(),
        Some('+' | '-' | '*' | '/' | '^' | '%' | '!')
    )
}

fn copy_icon() -> yew::Html {
    if is_apple_platform() {
        platform_icon(
            "icon-svg",
            "<rect x=\"9\" y=\"9\" width=\"11\" height=\"11\" rx=\"2.5\"/><path d=\"M5.5 14.5H5a2.5 2.5 0 0 1-2.5-2.5V5A2.5 2.5 0 0 1 5 2.5h7A2.5 2.5 0 0 1 14.5 5v.5\"/>",
        )
    } else {
        platform_icon(
            "icon-svg",
            "<rect x=\"8\" y=\"8\" width=\"13\" height=\"13\" rx=\"1\"/><path d=\"M16 8V5a2 2 0 0 0-2-2H5a2 2 0 0 0-2 2v9a2 2 0 0 0 2 2h3\"/>",
        )
    }
}

/// The "copied" check (ADR-0057): the answer's copy button answers a
/// press with a check for a moment, then returns to the copy mark.
fn check_icon() -> yew::Html {
    platform_icon("icon-svg", "<path d=\"M4.5 12.5 10 18 19.5 6.5\"/>")
}

/// The Save PNG icon (ADR-0042): an arrow into a tray, the standard
/// download mark; the name stays available through the aria-label and
/// tooltip like the other icon buttons.
fn download_icon() -> yew::Html {
    platform_icon(
        "icon-svg",
        "<path d=\"M12 3v11\"/><path d=\"m7.5 9.5 4.5 4.5 4.5-4.5\"/><path d=\"M4.5 17.5V19a1.5 1.5 0 0 0 1.5 1.5h12a1.5 1.5 0 0 0 1.5-1.5v-1.5\"/>",
    )
}

/// The private separator between answers in the result state (ADR-0052,
/// ADR-0055 layout): the state joins a script's outputs with a unit
/// separator so the renderer can lay them out as separate items -
/// same line with `;` between them, never splitting one answer - while
/// messages and single outputs (which never contain the character)
/// render exactly as one item.
const ANSWER_SEP: char = '\u{1f}';

/// True when the answer line keeps a result (ADR-0056): exactly one
/// answer, no line breaks, and short enough to read without scrolling.
/// Anything longer - a pasted script's transcript, a table, a long
/// number - renders in the result pane instead, one answer per line.
pub fn answer_fits(text: &str) -> bool {
    answer_fits_at(text, mobile_layout())
}

/// The routing rule with the layout question abstracted, so tests can
/// run without a window: `narrow` is the mobile-layout answer.
pub fn answer_fits_at(text: &str, narrow: bool) -> bool {
    if text.is_empty() || text.contains(ANSWER_SEP) || text.contains('\n') {
        return false;
    }
    // Conservative caps keep the answer on one calm line: about 44
    // monospace characters fit a desktop answer line without
    // scrolling, about 24 fit a phone's. A borderline answer moving to
    // the result pane is a smaller fault than one the answer line
    // clips (ADR-0035's everything-visible contract).
    let cap = if narrow { 24 } else { 44 };
    text.chars().count() <= cap
}

/// The export palette for the app's theme name (ADR-0057): an exported
/// plot wears the same colors the pane wears.
fn export_palette(theme: &str) -> graph::SvgPalette {
    match theme {
        "light" => graph::SvgPalette::Light,
        "night" => graph::SvgPalette::Night,
        _ => graph::SvgPalette::Dark,
    }
}

/// The solar scene with the legend's unchecked bodies removed, the
/// exact filter the live pane renders through (ADR-0038 amendment);
/// exports go through it too so a hidden body stays hidden.
fn filter_solar_scene(
    scene: &epher_core::astro::SolarScene,
    hidden: &[i64],
) -> epher_core::astro::SolarScene {
    epher_core::astro::SolarScene {
        jd: scene.jd,
        orbits: scene
            .orbits
            .iter()
            .filter(|p| !hidden.contains(&p.body))
            .cloned()
            .collect(),
        trails: scene
            .trails
            .iter()
            .filter(|p| !hidden.contains(&p.body))
            .cloned()
            .collect(),
        dots: scene
            .dots
            .iter()
            .filter(|d| !hidden.contains(&d.body))
            .cloned()
            .collect(),
    }
}

/// Serialize a value to a JSON string for localStorage (ADR-0057).
/// serde_json, not serde_wasm_bindgen + JSON.stringify: the latter's
/// value carries a deserialization prototype that stringifies to an
/// empty object.
fn json_string<T: serde::Serialize>(v: &T) -> Option<String> {
    serde_json::to_string(v).ok()
}

/// The clipboard text for an answer (ADR-0057): the displayed answers
/// without the "= " voice, one per line, so a paste gives the values.
fn answer_clip(result: &str) -> String {
    result
        .split(ANSWER_SEP)
        .map(|p| {
            let t = p.trim();
            t.strip_prefix("= ").unwrap_or(t).to_string()
        })
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Render the result text as answer items (ADR-0055): short answers flow
/// on one line separated by semicolons; an answer that carries its own
/// line breaks (a table, a matrix) or cannot fit on the line is never
/// split - it moves to its own line whole.
fn answer_items(text: &str) -> Vec<Html> {
    let parts: Vec<&str> = text.split(ANSWER_SEP).collect();
    let mut out: Vec<Html> = Vec::new();
    // A separator joins two answers that share a line: it appears before
    // an inline answer whose predecessor was also inline. A multiline
    // answer (a table, a matrix) is its own block on its own line - the
    // layout wraps there, so nothing joins across the break.
    let mut prev_inline = false;
    for part in parts {
        if part.is_empty() {
            continue;
        }
        let multiline = part.contains('\n');
        if prev_inline && !multiline {
            out.push(html! { <span class="ans-sep" aria-hidden="true">{ ";" }</span> });
        }
        if multiline {
            out.push(html! { <span class="ans-block">{ (*part).to_string() }</span> });
        } else {
            out.push(html! { <span class="ans-inline">{ (*part).to_string() }</span> });
        }
        prev_inline = !multiline;
    }
    out
}

/// The clear icon (ADR-0040): the graph pane's Clear button reads as a
/// trash can, not a labelled button - the name stays available to
/// assistive tech through the button's aria-label and as a tooltip.
fn trash_icon() -> yew::Html {
    platform_icon(
        "icon-svg",
        "<path d=\"M3.5 6h17\"/><path d=\"M18.5 6v13a2 2 0 0 1-2 2h-9a2 2 0 0 1-2-2V6\"/><path d=\"M8.5 6V4a2 2 0 0 1 2-2h3a2 2 0 0 1 2 2v2\"/><path d=\"M10 10.5v6.5\"/><path d=\"M14 10.5v6.5\"/>",
    )
}

/// The tuning-strip icons (ADR-0041): each labels its slider in the
/// strip above the plot, with the tooltip carrying the words.
/// Line thickness: three strokes of increasing weight.
fn line_width_icon() -> yew::Html {
    platform_icon(
        "icon-svg",
        "<path stroke-width=\"1.2\" d=\"M3 6.5h18\"/><path stroke-width=\"2\" d=\"M3 12h18\"/><path stroke-width=\"3.2\" d=\"M3 17.5h18\"/>",
    )
}

/// Horizontal rotation: an arc with an arrowhead sweeping around the
/// vertical axis.
fn rot_h_icon() -> yew::Html {
    platform_icon(
        "icon-svg",
        "<polyline points=\"23 4 23 10 17 10\"/><path d=\"M20.49 15a9 9 0 1 1-2.12-9.36L23 10\"/>",
    )
}

/// Vertical rotation: the same arc turned a quarter-turn, sweeping
/// around the horizontal axis.
fn rot_v_icon() -> yew::Html {
    platform_icon(
        "icon-svg",
        "<g transform=\"rotate(90 12 12)\"><polyline points=\"23 4 23 10 17 10\"/><path d=\"M20.49 15a9 9 0 1 1-2.12-9.36L23 10\"/></g>",
    )
}

/// Zoom: a magnifier with a plus.
fn zoom_icon() -> yew::Html {
    platform_icon(
        "icon-svg",
        "<circle cx=\"11\" cy=\"11\" r=\"7\"/><path d=\"m21 21-4.3-4.3\"/><path d=\"M8 11h6\"/><path d=\"M11 8v6\"/>",
    )
}

/// The standard share icon for this device (ADR-0038): the arrow-out-of-
/// tray on Apple devices, the connected dots elsewhere.
fn share_icon() -> yew::Html {
    if is_apple_platform() {
        platform_icon(
            "icon-svg",
            "<path d=\"M12 15V3\"/><path d=\"m8 6.5 4-4 4 4\"/><path d=\"M5 11v8a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2v-8\"/>",
        )
    } else {
        platform_icon(
            "icon-svg",
            "<circle cx=\"18\" cy=\"5\" r=\"3\"/><circle cx=\"6\" cy=\"12\" r=\"3\"/><circle cx=\"18\" cy=\"19\" r=\"3\"/><path d=\"m8.6 13.5 6.8 4M15.4 6.5l-6.8 4\"/>",
        )
    }
}

/// Graph → Save PNG (ADR-0042): rasterize the exported SVG and save it
/// through the same channels a script save uses - the desktop dialog
/// over IPC, the browser's picker, else a plain download. Cancel stays
/// silent, like any native app.
fn save_png_with_dialog(
    bridge: Bridge,
    svg: String,
    default_name: &str,
    result: &UseStateHandle<String>,
    localizer: &UseStateHandle<Localizer>,
) {
    let default_name = default_name.to_string();
    let result = result.clone();
    let localizer = localizer.clone();
    spawn_local(async move {
        let bytes = match rasterize_svg(&svg).await {
            Ok(bytes) => bytes,
            Err(_) => {
                result.set(localizer.lookup("graph-png-failed"));
                return;
            }
        };
        if bridge == Bridge::Tauri {
            match bridge.save_png_dialog(&bytes, &default_name).await {
                Ok(Some(_)) => result.set(localizer.lookup("graph-png-saved")),
                Ok(None) => {}
                Err(_) => result.set(localizer.lookup("graph-png-failed")),
            }
        } else {
            match browser_save_png_dialog(&default_name, &bytes).await {
                Ok(Some(_)) => result.set(localizer.lookup("graph-png-saved")),
                Ok(None) => {}
                Err(_) => {
                    download_png_file(&default_name, &bytes);
                    result.set(localizer.lookup("graph-png-saved"));
                }
            }
        }
    });
}

/// Rasterize an SVG document to PNG bytes at twice its intrinsic size, so
/// curves stay crisp next to text. A transparent background exports as
/// transparency; a document that paints its own keeps it.
async fn rasterize_svg(svg: &str) -> Result<Vec<u8>, String> {
    let failed = || "png rasterization failed".to_string();
    let window = web_sys::window().ok_or_else(failed)?;
    let doc = window.document().ok_or_else(failed)?;
    let svg_bag = web_sys::BlobPropertyBag::new();
    svg_bag.set_type("image/svg+xml");
    let svg_blob = web_sys::Blob::new_with_str_sequence_and_options(
        &js_sys::Array::of1(&JsValue::from_str(svg)),
        &svg_bag,
    )
    .map_err(|_| failed())?;
    let url = web_sys::Url::create_object_url_with_blob(&svg_blob).map_err(|_| failed())?;
    let img = web_sys::HtmlImageElement::new().map_err(|_| failed())?;
    // the image's load event resolves the promise; an error rejects it
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let onload = Closure::once(move |_: JsValue| {
            let _ = resolve.call0(&JsValue::NULL);
        });
        let onerror = Closure::once(move |_: JsValue| {
            let _ = reject.call0(&JsValue::NULL);
        });
        img.set_onload(Some(onload.as_ref().unchecked_ref()));
        img.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onload.forget();
        onerror.forget();
    });
    img.set_src(&url);
    if JsFuture::from(promise).await.is_err() {
        let _ = web_sys::Url::revoke_object_url(&url);
        return Err(failed());
    }
    let w = img.natural_width().max(1);
    let h = img.natural_height().max(1);
    let canvas = doc
        .create_element("canvas")
        .map_err(|_| failed())?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| failed())?;
    canvas.set_width(w * 2);
    canvas.set_height(h * 2);
    let ctx = canvas
        .get_context("2d")
        .map_err(|_| failed())?
        .ok_or_else(failed)?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .map_err(|_| failed())?;
    ctx.draw_image_with_html_image_element_and_dw_and_dh(
        &img,
        0.0,
        0.0,
        (w * 2) as f64,
        (h * 2) as f64,
    )
    .map_err(|_| failed())?;
    let _ = web_sys::Url::revoke_object_url(&url);
    // to_blob is callback-based; wrap it in a promise
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let cb = Closure::once(move |blob: Option<web_sys::Blob>| match blob {
            Some(b) => {
                let _ = resolve.call1(&JsValue::NULL, &b);
            }
            None => {
                let _ = reject.call0(&JsValue::NULL);
            }
        });
        let _ = canvas.to_blob(cb.as_ref().unchecked_ref());
        cb.forget();
    });
    let blob: web_sys::Blob = JsFuture::from(promise)
        .await
        .map_err(|_| failed())?
        .dyn_into()
        .map_err(|_| failed())?;
    let buffer = JsFuture::from(blob.array_buffer())
        .await
        .map_err(|_| failed())?;
    Ok(js_sys::Uint8Array::new(&buffer).to_vec())
}

/// The browser's own save picker for a PNG (File System Access API,
/// Chromium); mirrors [`browser_save_dialog`] for binary content.
/// `Ok(None)` = cancelled; `Err` = no picker (fall back to a download).
async fn browser_save_png_dialog(
    default_name: &str,
    bytes: &[u8],
) -> Result<Option<String>, String> {
    let window = web_sys::window().ok_or_else(|| "no window".to_string())?;
    let unavailable = || "save picker unavailable".to_string();
    let picker = js_sys::Reflect::get(&window, &JsValue::from_str("showSaveFilePicker"))
        .map_err(|_| unavailable())?;
    if !picker.is_function() {
        return Err(unavailable());
    }
    let picker_fn = picker
        .dyn_into::<js_sys::Function>()
        .map_err(|_| unavailable())?;

    let accept = js_sys::Object::new();
    let extensions = js_sys::Array::new();
    extensions.push(&JsValue::from_str(".png"));
    js_sys::Reflect::set(&accept, &JsValue::from_str("image/png"), &extensions)
        .map_err(|_| unavailable())?;
    let type_entry = js_sys::Object::new();
    js_sys::Reflect::set(
        &type_entry,
        &JsValue::from_str("description"),
        &JsValue::from_str("PNG"),
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
            return if cancelled {
                Ok(None)
            } else {
                Err(unavailable())
            };
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
    let payload = js_sys::Uint8Array::from(bytes);
    let write_promise = write
        .call1(&writable, &payload)
        .map_err(|_| unavailable())?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| unavailable())?;
    JsFuture::from(write_promise)
        .await
        .map_err(|_| unavailable())?;
    let close = js_sys::Reflect::get(&writable, &JsValue::from_str("close"))
        .map_err(|_| unavailable())?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| unavailable())?;
    let close_promise = close
        .call0(&writable)
        .map_err(|_| unavailable())?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| unavailable())?;
    JsFuture::from(close_promise)
        .await
        .map_err(|_| unavailable())?;
    Ok(Some(name))
}

/// The plain-download fallback: a PNG blob handed to the browser's
/// downloader through an anchor click.
fn download_png_file(name: &str, bytes: &[u8]) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Ok(blob) = web_sys::Blob::new_with_buffer_source_sequence(&js_sys::Array::of1(
        &js_sys::Uint8Array::from(bytes).into(),
    )) else {
        return;
    };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
        return;
    };
    if let Ok(anchor) = doc.create_element("a") {
        let _ = anchor.set_attribute("href", &url);
        let _ = anchor.set_attribute("download", name);
        if let Some(body) = doc.body() {
            let _ = body.append_child(&anchor);
            if let Ok(a) = anchor.clone().dyn_into::<web_sys::HtmlAnchorElement>() {
                a.click();
            }
            let _ = body.remove_child(&anchor);
        }
    }
    // the browser has until the revoke timer below to start the download
    spawn_local(async move {
        gloo_timers::future::TimeoutFuture::new(10_000).await;
        let _ = web_sys::Url::revoke_object_url(&url);
    });
}

/// The share link for an expression (ADR-0038): this app's URL with the
/// expression in the `expr` parameter - the same contents a history
/// pick loads into the entry. Outside the web (the desktop shell's
/// `tauri://` origin) the link points at the PWA's home instead, so the
/// recipient's browser opens the app.
fn share_link(expr: &str) -> String {
    const PWA_URL: &str = "https://epher.org/pwa/";
    let base = web_sys::window().and_then(|w| {
        let origin = w.location().origin().ok()?;
        if origin.starts_with("http") {
            let path = w.location().pathname().ok()?;
            Some(format!("{origin}{path}"))
        } else {
            None
        }
    });
    let base = base.unwrap_or_else(|| PWA_URL.to_string());
    format!("{base}?expr={}", js_sys::encode_uri_component(expr))
}

// ===== guide search (ADR-0038) =====

/// One guide-search match: where to jump (the DOM order index among the
/// searched nodes) and the text around the match.
struct GuideHit {
    chapter: String,
    snippet: String,
    index: usize,
}

/// One locale's guide markdown (ADR-0053): fetched from the app's
/// static files the first time the guide opens, cached for the session;
/// the binary carries none of it.
enum GuideEntry {
    Loading,
    Ready(Rc<str>),
    Failed,
}

/// GET a same-origin text asset (the guide markdown, ADR-0053): the
/// response body on 200, None otherwise. The service worker
/// runtime-caches these files, so offline-after-first-view works.
async fn fetch_text(url: &str) -> Option<String> {
    let window = web_sys::window()?;
    let response = JsFuture::from(window.fetch_with_str(url))
        .await
        .ok()?
        .dyn_into::<web_sys::Response>()
        .ok()?;
    if !response.ok() {
        return None;
    }
    JsFuture::from(response.text().ok()?)
        .await
        .ok()?
        .as_string()
}

/// The guide overlay's body (ADR-0053): the fetched markdown rendered
/// as HTML, or the load state while it arrives / when it cannot be.
fn guide_body(localizer: &Localizer, cache: &HashMap<String, GuideEntry>) -> Html {
    match cache.get(localizer.locale()) {
        Some(GuideEntry::Ready(md)) => Html::from_html_unchecked(
            epher_guide::render_html(md, &localizer.lookup("guide-contents")).into(),
        ),
        Some(GuideEntry::Failed) => html! {
            <p class="guide-load-failed">
                { localizer.lookup("guide-unavailable") } { " " }
                <a href="https://epher.org/guide/" target="_blank" rel="noreferrer">
                    { "epher.org/guide" }
                </a>
            </p>
        },
        _ => html! {
            <p class="guide-loading" role="status">
                { localizer.lookup("guide-loading") }
            </p>
        },
    }
}

const GUIDE_SEARCH_NODES: &str = ".guide-body h2, .guide-body h3, .guide-body p, .guide-body li";

/// Search the rendered guide text (ADR-0038): a case-insensitive
/// substring scan over every heading, paragraph, and list entry, each hit
/// carrying its chapter's title and a snippet around the match.
fn guide_search(query: &str) -> Vec<GuideHit> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Vec::new();
    }
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return Vec::new();
    };
    let Ok(nodes) = doc.query_selector_all(GUIDE_SEARCH_NODES) else {
        return Vec::new();
    };
    let mut hits = Vec::new();
    for i in 0..nodes.length() {
        let Some(el) = nodes
            .item(i)
            .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
        else {
            continue;
        };
        let text = el.text_content().unwrap_or_default();
        let lower = text.to_lowercase();
        if let Some(pos) = lower.find(&q) {
            // The chapter is the nearest preceding h2 in document order.
            let mut chapter = String::new();
            let mut j = i;
            while j > 0 {
                j -= 1;
                if let Some(prev) = nodes
                    .item(j)
                    .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
                {
                    if prev.tag_name() == "H2" {
                        chapter = prev.text_content().unwrap_or_default();
                        break;
                    }
                }
            }
            let start = pos.saturating_sub(24);
            let end = (start + 96).min(text.len());
            let mut snippet = text[start..end].trim().to_string();
            if start > 0 {
                snippet = format!("\u{2026}{snippet}");
            }
            if end < text.len() {
                snippet.push('\u{2026}');
            }
            hits.push(GuideHit {
                chapter,
                snippet,
                index: i as usize,
            });
            if hits.len() >= 20 {
                break;
            }
        }
    }
    hits
}

/// Jump to a search hit: scroll it into view inside the guide body and
/// flash it, so the match is findable again after the jump.
fn guide_jump_to(index: usize) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Ok(nodes) = doc.query_selector_all(GUIDE_SEARCH_NODES) else {
        return;
    };
    if let Some(el) = nodes
        .item(index as u32)
        .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
    {
        el.scroll_into_view();
        let Ok(el) = el.dyn_into::<web_sys::HtmlElement>() else {
            return;
        };
        let _ = el.class_list().add_1("guide-hit");
        let el = el.clone();
        gloo_timers::callback::Timeout::new(1_600, move || {
            let _ = el.class_list().remove_1("guide-hit");
        })
        .forget();
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
        let report =
            |result: &UseStateHandle<String>, localizer: &UseStateHandle<Localizer>, name: &str| {
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
    let picker_fn = picker
        .dyn_into::<js_sys::Function>()
        .map_err(|_| unavailable())?;

    let accept = js_sys::Object::new();
    let extensions = js_sys::Array::new();
    extensions.push(&JsValue::from_str(".epher"));
    js_sys::Reflect::set(&accept, &JsValue::from_str("text/plain"), &extensions)
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
            return if cancelled {
                Ok(None)
            } else {
                Err(unavailable())
            };
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
    JsFuture::from(write_promise)
        .await
        .map_err(|_| unavailable())?;
    let close = js_sys::Reflect::get(&writable, &JsValue::from_str("close"))
        .map_err(|_| unavailable())?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| unavailable())?;
    let close_promise = close
        .call0(&writable)
        .map_err(|_| unavailable())?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| unavailable())?;
    JsFuture::from(close_promise)
        .await
        .map_err(|_| unavailable())?;
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
/// Multi-line entries travel as one line with `\n` escapes (ADR-0027
/// amendment): the two-character sequence becomes the entry's newline
/// on load.
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
        s.record(&line.replace("\\n", "\n"));
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
    // The live session cell (stale-deref rule): the store-changed apply
    // and the submit both read and write this Rc<RefCell<Session>> in
    // lockstep with the render state, so a callback never observes the
    // pre-set value of the Yew handle.
    let session_live = use_state(|| Rc::new(RefCell::new(Session::new())));
    let input = use_state(String::new);
    let form_ref = use_node_ref();
    let input_ref = use_node_ref();
    let result = use_state(String::new);
    // Flips true for a moment after the answer's copy button is pressed
    // (ADR-0057): the icon answers as a check and the label as
    // "Copied", then the timer resets both.
    let answer_copied = use_state(|| false);
    // The answer on screen is part of the PWA's recent activity
    // (ADR-0057). An effect keyed on the result state persists it with
    // the fresh value: a submit closure's handle still derefs to the
    // previous render's answer, the exact trap the session-live cell
    // exists to avoid.
    {
        let result = result.clone();
        use_effect_with(result.clone(), move |r| {
            // The empty mount value never writes: the init restore below
            // reads this same key, and a mount-time wipe would erase the
            // answer before the restore could see it.
            if !r.is_empty() {
                if let Some(store) =
                    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
                {
                    let _ = store.set_item("epher-result", r);
                }
            }
            || {}
        });
    }
    let localizer = use_state(|| Localizer::resolve(None, &[]));
    let graph = use_state(Vec::<SampledCurve>::new);
    let data = use_state(|| Option::<DataPlot>::None);
    let pois = use_state(Vec::<graph::Poi>::new);
    let trace = use_state(|| Option::<graph::TracePoint>::None);
    // Graph options (ADR-0019, on the pane itself since ADR-0020): whether
    // the pane lists the points of interest and marks them on the plot,
    // and the curve line width. Display-only — the analysis always runs,
    // so switching back is instant. Mobile remembers each graph kind's
    // width independently (ADR-0035): 3D starts at 0.1, thin lines for
    // the small screen (ADR-0031); 2D starts at the desktop default.
    // Desktop keeps one shared width for both kinds (ADR-0020).
    let poi_list = use_state(|| true);
    let poi_markers = use_state(|| true);
    let width_2d = use_state(|| graph::DEFAULT_STROKE_WIDTH);
    // The 3D wireframe width (ADR-0055 desktop, ADR-0035 mobile): the
    // desktop range is 0–0.4 step 0.05 with 0.2 the default; the touch
    // layout keeps ADR-0035's 0–0.2 step 0.01 with 0.1. The width is a
    // screen-px measure (vector-effect), so the defaults draw a 2 px
    // (desktop) or 1 px (mobile) line on any display and in the exports
    // — the two kinds own separate sliders, and a 3D surface keeps its
    // own default rather than inheriting the 2D curve's.
    let width_3d = use_state(|| graph::three_d_default_width(mobile_layout()));
    // Per-curve visibility (ADR-0015 amendment): each legend entry has a
    // checkbox, checked by default; unchecking hides that curve from the
    // plot, its points of interest, and the SVG export. Reset whenever a
    // new plot replaces the curve list.
    let hidden = use_state(|| Vec::<bool>::new());
    // 3D per-element visibility (ADR-0055): the surface and space-curve
    // legends carry the same checkboxes as the 2D curve legend. Hidden
    // indices keep their palette slots, so a hidden neighbour never
    // shifts the remaining elements' colours.
    let hidden_surfaces = use_state(Vec::<usize>::new);
    let hidden_curves3d = use_state(Vec::<usize>::new);
    // Which side of the 880px breakpoint the window is on (ADR-0016): the
    // width slider's range is a mobile/desktop decision (0–0.2 step 0.01
    // vs 0.1–4 step 0.1), and it tracks window resizes.
    let is_mobile = use_state(mobile_layout);
    let live = use_state(|| Rc::new(RefCell::new(GraphLive::default())));
    let surface = use_state(Vec::<epher_core::graph::Surface>::new);
    // The 3D parametric curves (`graph3d param ...`, ADR-0054): the
    // curve sibling of the surface set; the pane shows one kind at a
    // time.
    let curve3ds = use_state(Vec::<epher_core::graph::SpaceCurve>::new);
    // The solar system scene (`solar3d`, ADR-0037) plus the source of
    // its time expression, so playback can rebuild the scene per tick.
    let solar = use_state(|| Option::<epher_core::astro::SolarScene>::None);
    let solar_source = use_state(|| Option::<String>::None);
    let view = use_state(epher_core::graph::View3D::default);
    // The live cell behind `view`: orbit emissions mutate it in place, so
    // a burst of drag/keyboard events accumulates instead of each event
    // reading the same stale handle snapshot and overwriting the last
    // (the v0.4.13 "shivering" — the render-snapshot rule, ADR-0026).
    let view_cell = use_state(|| Rc::new(RefCell::new(epher_core::graph::View3D::default())));
    // Whether a 3D surface is currently plotted — the live spelling of
    // `!surface.is_empty()` for the width-slider's range decision, which
    // must read the kind at emit time (ADR-0035), not a stale handle
    // snapshot.
    let surface3d_cell = use_state(|| Rc::new(RefCell::new(false)));
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
    // The 2D plot's zoom window (ADR-0038): wheel, pinch, and the zoom
    // slider set an explicit x window; `None` is the auto-fit around the
    // samples. The live cell feeds the mount-time plot listeners, the
    // state mirrors it for rendering - the render-snapshot rule
    // (ADR-0026).
    let view2d = use_state(|| Option::<(f64, f64)>::None);
    let view2d_cell = use_state(|| Rc::new(RefCell::new(Option::<(f64, f64)>::None)));
    // The solar legend (ADR-0038): the bodies whose orbit, trail, and dot
    // are unchecked. A fresh scene starts with everything visible.
    let solar_hidden = use_state(|| Vec::<i64>::new());
    // Live mirrors of the rotation slider values for the spin loop.
    let view_h_cell = use_state(|| Rc::new(RefCell::new(0.0_f64)));
    let view_v_cell = use_state(|| Rc::new(RefCell::new(0.0_f64)));
    // The zoom's live cell (ADR-0057): a pinch delivers a burst of
    // pointer moves within one render, and the state handle's deref
    // trails the render - the cell keeps every step composing on the
    // true value.
    let view_z_cell = use_state(|| Rc::new(RefCell::new(0.0_f64)));
    let play = use_state(|| Option::<PlaySpec>::None);
    // The playback analysis throttle: a persistent counter shared by the
    // per-render on_tick closures (each render rebuilds the closure for
    // the live_apply cell, so a captured Cell would reset every tick).
    let tick_no = use_state(|| Rc::new(std::cell::RefCell::new(0u32)));
    // The live cell behind `play`: the animation loop reads and advances
    // it across ticks; Yew handles captured at spawn read stale snapshots.
    let play_cell = use_state(|| Rc::new(RefCell::new(Option::<PlaySpec>::None)));
    // The 3D viewBox from the latest render; play start freezes it.
    let rendered_box = use_state(|| Rc::new(RefCell::new(None::<String>)));
    let show_install_cli = use_state(|| false);
    // Keypad tab + which pane faces the user on mobile (ADR-0016).
    let key_tab = use_state(|| "digits".to_string());
    // Keypad key hints (ADR-0039): the bar text while a key rests under
    // the pointer or holds focus (empty shows the idle prompt), and the
    // toggle that captions every key for touch screens, where hover and
    // focus do not exist.
    let key_hint_bar = use_state(String::new);
    let show_key_hints = use_state(|| false);
    // The keypad drawer (ADR-0060): false docks the keypad away, the
    // history list grows into its place, and the grab bar stays for the
    // drag, tap, or Enter that brings the keypad back to its spot.
    let keypad_open = use_state(|| true);
    let keypad_drawer_ref = use_node_ref();
    // The pending snap animation's timer: a new gesture (or toggle)
    // cancels it, or its cleanup would clear the height a live drag just
    // froze.
    let keypad_anim =
        use_state(|| Rc::new(RefCell::new(Option::<gloo_timers::callback::Timeout>::None)));
    let keypad_drag = use_state(|| Rc::new(RefCell::new(Option::<KeypadDrag>::None)));
    // The time of the last pointer release on the grab bar: the browser
    // synthesizes a click right after pointerup, and that click must not
    // toggle the drawer a second time (the release already acted). A
    // click within half a second of a gesture is that echo; a keyboard
    // click (Enter/Space) comes without a gesture and toggles. A
    // timestamp, not a latch, so a cancelled gesture can never eat a
    // later keyboard toggle.
    let keypad_last_gesture = use_state(|| Rc::new(RefCell::new(0.0f64)));
    // The drawer's helper (ADR-0060): animate to a final state from
    // wherever the drawer is now — frozen inline height, one forced
    // reflow, target height with the transition on, then the inline
    // height clears so the resting class rule takes over. Shared by the
    // keyboard toggle and the drag's snap, so both get the same motion.
    let keypad_animate = {
        let keypad_drawer_ref = keypad_drawer_ref.clone();
        let keypad_open = keypad_open.clone();
        let keypad_anim = keypad_anim.clone();
        move |open: bool| {
            let Some(drawer) = keypad_drawer_ref.cast::<Element>() else {
                return;
            };
            let _ = drawer.class_list().remove_1("dragging");
            let Some(clip) = drawer
                .last_element_child()
                .and_then(|c| c.dyn_into::<HtmlElement>().ok())
            else {
                return;
            };
            let Some(body) = clip
                .first_element_child()
                .and_then(|b| b.dyn_into::<HtmlElement>().ok())
            else {
                return;
            };
            let open_h = body.offset_height() as f64;
            let start = clip.get_bounding_client_rect().height();
            let _ = clip.style().set_property("height", &format!("{start}px"));
            // The read forces the layout flush so the transition runs
            // from the frozen height, not from the rule it just left.
            let _ = clip.offset_height();
            let _ = clip
                .style()
                .set_property("height", &format!("{}", if open { open_h } else { 0.0 }));
            keypad_open.set(open);
            if let Some(t) = keypad_anim.borrow_mut().take() {
                t.cancel();
            }
            let clip = clip.clone();
            // The cell owns the timer: it stays alive until it fires (or
            // until the next gesture takes and cancels it — gloo's
            // Timeout cancels on drop). The fired timer sitting in the
            // cell is harmless; dropping it cancels nothing that runs.
            *keypad_anim.borrow_mut() = Some(gloo_timers::callback::Timeout::new(300, move || {
                let _ = clip.style().remove_property("height");
            }));
        }
    };
    let on_grab_down = {
        let keypad_drawer_ref = keypad_drawer_ref.clone();
        let keypad_drag = keypad_drag.clone();
        let keypad_anim = keypad_anim.clone();
        Callback::from(move |e: web_sys::PointerEvent| {
            // A second pointer never steals the gesture.
            if keypad_drag.borrow().is_some() {
                return;
            }
            if let Some(t) = keypad_anim.borrow_mut().take() {
                t.cancel();
            }
            let Some(drawer) = keypad_drawer_ref.cast::<Element>() else {
                return;
            };
            let Some(clip) = drawer
                .last_element_child()
                .and_then(|c| c.dyn_into::<HtmlElement>().ok())
            else {
                return;
            };
            let Some(body) = clip
                .first_element_child()
                .and_then(|b| b.dyn_into::<HtmlElement>().ok())
            else {
                return;
            };
            let _ = drawer.class_list().add_1("dragging");
            // Capture on the pressed element: moves keep arriving even
            // when the finger or cursor leaves the 24px strip.
            if let Some(el) = e.target().and_then(|t| t.dyn_into::<Element>().ok()) {
                let _ = el.set_pointer_capture(e.pointer_id());
            }
            let y = e.client_y() as f64;
            *keypad_drag.borrow_mut() = Some(KeypadDrag {
                pointer_id: e.pointer_id(),
                y0: y,
                start_h: clip.get_bounding_client_rect().height(),
                open_h: body.offset_height() as f64,
                last_y: y,
                last_t: js_sys::Date::now(),
                moved: false,
            });
            e.prevent_default();
        })
    };
    let on_grab_move = {
        let keypad_drawer_ref = keypad_drawer_ref.clone();
        let keypad_drag = keypad_drag.clone();
        Callback::from(move |e: web_sys::PointerEvent| {
            let mut drag = keypad_drag.borrow_mut();
            let Some(d) = drag.as_mut() else {
                return;
            };
            if d.pointer_id != e.pointer_id() {
                return;
            }
            let y = e.client_y() as f64;
            if (y - d.y0).abs() > 4.0 {
                d.moved = true;
            }
            // The keypad follows the pointer: dragging down shrinks it,
            // and the history list grows into the freed space live.
            let h = (d.start_h - (y - d.y0)).clamp(0.0, d.open_h);
            if let Some(drawer) = keypad_drawer_ref.cast::<Element>() {
                if let Some(clip) = drawer
                    .last_element_child()
                    .and_then(|c| c.dyn_into::<HtmlElement>().ok())
                {
                    let _ = clip.style().set_property("height", &format!("{h}px"));
                }
            }
            d.last_y = y;
            d.last_t = js_sys::Date::now();
        })
    };
    let on_grab_end = {
        let keypad_drawer_ref = keypad_drawer_ref.clone();
        let keypad_drag = keypad_drag.clone();
        let keypad_last_gesture = keypad_last_gesture.clone();
        let keypad_animate = keypad_animate.clone();
        let keypad_open = keypad_open.clone();
        Callback::from(move |e: web_sys::PointerEvent| {
            let Some(d) = keypad_drag.borrow_mut().take() else {
                return;
            };
            if d.pointer_id != e.pointer_id() {
                return;
            }
            let _ = keypad_drawer_ref
                .cast::<Element>()
                .map(|drawer| drawer.class_list().remove_1("dragging"));
            *keypad_last_gesture.borrow_mut() = js_sys::Date::now();
            if !d.moved {
                // A tap toggles — the same as Enter on the button. The
                // click that follows this release is ignored by the
                // gesture timestamp.
                keypad_animate(!*keypad_open);
                return;
            }
            let dt = js_sys::Date::now() - d.last_t;
            let v = if dt > 0.0 {
                (e.client_y() as f64 - d.last_y) / dt
            } else {
                0.0
            };
            let current = keypad_drawer_ref
                .cast::<Element>()
                .and_then(|drawer| drawer.last_element_child())
                .and_then(|c| c.dyn_into::<HtmlElement>().ok())
                .map(|clip| clip.get_bounding_client_rect().height())
                .unwrap_or(0.0);
            keypad_animate(!keypad_snap(current, d.open_h, v));
        })
    };
    let on_grab_click = {
        let keypad_last_gesture = keypad_last_gesture.clone();
        let keypad_animate = keypad_animate.clone();
        let keypad_open = keypad_open.clone();
        Callback::from(move |_| {
            // The click a pointer gesture just synthesized, not a press:
            // the release already acted. A keyboard click (Enter/Space)
            // has no gesture behind it and toggles.
            if js_sys::Date::now() - *keypad_last_gesture.borrow() < 500.0 {
                return;
            }
            keypad_animate(!*keypad_open);
        })
    };
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
    // Result rendering (ADR-0043): exact fractions (default on), the
    // notation, and thousands separators, from the Settings menu.
    let display_prefs = use_state(DisplayPrefs::default);
    let menu_open = use_state(|| Option::<&'static str>::None);
    // A live mirror of `menu_open` for long-lived closures (Yew handles
    // are render snapshots; the Rc cell updates every render).
    let menu_open_cell = use_state(|| Rc::new(RefCell::new(Option::<&'static str>::None)));
    *menu_open_cell.borrow_mut() = *menu_open;
    let hamburger_open = use_state(|| false);
    let guide_open = use_state(|| false);
    // The guide's per-locale markdown cache (ADR-0053): the overlay
    // reads it each render; the effect below starts the fetch. The tick
    // bumps when a fetch lands so the open overlay re-renders.
    let guide_cache: Rc<RefCell<HashMap<String, GuideEntry>>> = use_mut_ref(HashMap::new);
    let guide_tick = use_state(|| 0u32);
    {
        let guide_open = guide_open.clone();
        let guide_cache = guide_cache.clone();
        let guide_tick = guide_tick.clone();
        let locale = (*localizer).locale().to_string();
        use_effect_with((guide_open, locale), move |(open, locale)| {
            if **open {
                let known = guide_cache.borrow().contains_key(locale.as_str());
                if !known {
                    guide_cache
                        .borrow_mut()
                        .insert(locale.clone(), GuideEntry::Loading);
                    let guide_cache = guide_cache.clone();
                    let locale = locale.clone();
                    let file = epher_guide::file_name(&locale);
                    spawn_local(async move {
                        let md = fetch_text(&format!("guide/{file}")).await;
                        guide_cache.borrow_mut().insert(
                            locale,
                            match md {
                                Some(text) => GuideEntry::Ready(Rc::from(text.as_str())),
                                None => GuideEntry::Failed,
                            },
                        );
                        guide_tick.set((*guide_tick).wrapping_add(1));
                    });
                }
            }
            move || {}
        });
    }
    // The constants browser (ADR-0045): Help menu -> Constants, the
    // grouped builtin list that inserts a name into the entry field.
    let constants_open = use_state(|| false);
    let constants_query = use_state(String::new);
    let constants_close_ref = use_node_ref();
    {
        // Focus the close button whenever the browser opens, like the
        // guide dialog, so Escape works from the first keypress.
        let constants_open = constants_open.clone();
        let constants_close_ref = constants_close_ref.clone();
        use_effect_with(constants_open, move |open| {
            if **open {
                if let Some(el) = constants_close_ref.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
            }
            || {}
        });
    }
    let on_close_constants = {
        let constants_open = constants_open.clone();
        Callback::from(move |_: web_sys::MouseEvent| constants_open.set(false))
    };
    // Insert a constant name at the entry's cursor — the same splice a
    // keypad press does (ADR-0045): selection-replacing, cursor after
    // the name, and the browser stays open for the next pick.
    let insert_constant = {
        let input = input.clone();
        let input_ref = input_ref.clone();
        let cursor_cell = cursor_cell.clone();
        Callback::from(move |name: String| {
            let Some(ta) = input_ref.cast::<HtmlTextAreaElement>() else {
                return;
            };
            let mut v = (*input).clone();
            let (s, e) = *cursor_cell.borrow();
            let (s, e) = (s.min(v.chars().count()), e.min(v.chars().count()));
            v.replace_range(char_byte(&v, s)..char_byte(&v, e), &name);
            input.set(v.clone());
            ta.set_value(&v);
            let pos = s + name.chars().count();
            ta.set_selection_start(Some(pos as u32)).ok();
            ta.set_selection_end(Some(pos as u32)).ok();
            *cursor_cell.borrow_mut() = (pos, pos);
        })
    };
    // The in-app guide's search box (ADR-0038): the query drives a scan
    // over the rendered guide text; hits jump to their match.
    let guide_query = use_state(String::new);
    let guide_hits = use_state(Vec::<GuideHit>::new);
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
    // The history list's section - the `history` command (ADR-0038) and
    // keyboard cycling focus it.
    let history_box_ref = use_node_ref();
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
            if let Some(store) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
                let _ = store.set_item("epher-history", "[]");
                let _ = store.remove_item("epher-result");
            }
            result.set(String::new());
        })
    };

    // Inside the desktop shell: rebuild the session from the native store —
    // history plus saved functions and scripts replayed quietly, the exact
    // load_session recipe — and honor the stored language preference.
    // The same apply path serves the live store-changed broadcasts
    // (ADR-0010 amendment): another frontend's write arrives as a fresh
    // InitState and this applies it in place, so the open app always
    // mirrors the shared store.
    let apply_store_state = {
        let session = session.clone();
        let session_live = session_live.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        let theme = theme.clone();
        let display_prefs = display_prefs.clone();
        move |state: InitState| {
            let mut s = Session::with_history(state.history);
            for line in &state.replay {
                s.submit_quiet(line);
            }
            // Keep the definitions the user created in THIS session but
            // has not `save`d yet: a store write from another frontend
            // (or our own echo) must not erase live work. The store's
            // bindings win over anything the replay re-applied. The
            // sources come from the live cell, never from the Yew state
            // handle: a deref of the handle can still hold the pre-set
            // value when this callback runs right after a submit, which
            // silently dropped freshly typed constants on reload
            // (v0.4.34: `const a = 1` then `graph3d …a…` errored with
            // "unknown name: a" on the desktop app).
            {
                let live = session_live.borrow();
                for source in live.def_sources().values() {
                    s.submit_quiet(source);
                }
                for source in live.const_sources().values() {
                    s.submit_quiet(source);
                }
            }
            // The shared session snapshot (ADR-0010 amendment): bindings
            // saved by whichever CLI/REPL/TUI/desktop frontend ran last —
            // `ans` and every user assignment carry over.
            s.restore_bindings(&state.session);
            let display = *display_prefs;
            s.set_display(display);
            session.set(s.clone());
            *session_live.borrow_mut() = s;
            if let Some(code) = state.language {
                localizer.set(Localizer::resolve(Some(&code), &[]));
            }
            if let Some(name) = state.theme {
                theme.set(name);
            }
        }
    };
    {
        let result = result.clone();
        let localizer = localizer.clone();
        let theme = theme.clone();
        let poi_list = poi_list.clone();
        let poi_markers = poi_markers.clone();
        let width_2d = width_2d.clone();
        let width_3d = width_3d.clone();
        let input = input.clone();
        let input_ref = input_ref.clone();
        let cursor_cell = cursor_cell.clone();
        let display_prefs = display_prefs.clone();
        let session_live = session_live.clone();
        let session = session.clone();
        use_effect_with((), move |_| {
            if bridge == Bridge::Tauri {
                let apply = apply_store_state.clone();
                spawn_local(async move {
                    match bridge.init().await {
                        Ok(state) => {
                            apply(state);
                            // Graph pane options live in the webview's
                            // localStorage on desktop too (ADR-0020) — the
                            // native store carries only what must exist
                            // before mount.
                            if let Some(store) =
                                web_sys::window().and_then(|w| w.local_storage().ok().flatten())
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
                                let (w2d, w3d) = stored_widths(&store, mobile_layout());
                                if let Some(w) = w2d {
                                    width_2d.set(w);
                                }
                                if let Some(w) = w3d {
                                    width_3d.set(w);
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
                // Live sync: every store write — this window's own or the
                // TUI's, the REPL's, a one-shot CLI run's — reapplies the
                // shared state immediately (ADR-0010 amendment).
                Bridge::listen_store_changed(apply_store_state.clone());
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
                    // Result rendering (ADR-0043): the stored display
                    // preferences; defaults are exact fractions on, Auto
                    // notation, no separators.
                    if let Ok(Some(v)) = store.get_item("epher-exact") {
                        if v == "0" {
                            display_prefs.set(DisplayPrefs {
                                exact_fractions: false,
                                ..*display_prefs
                            });
                        }
                    }
                    if let Ok(Some(v)) = store.get_item("epher-format") {
                        let notation = match v.as_str() {
                            "scientific" => Notation::Scientific,
                            "engineering" => Notation::Engineering,
                            _ => Notation::Auto,
                        };
                        display_prefs.set(DisplayPrefs {
                            notation,
                            ..*display_prefs
                        });
                    }
                    if let Ok(Some(v)) = store.get_item("epher-separators") {
                        if v == "1" {
                            display_prefs.set(DisplayPrefs {
                                separators: true,
                                ..*display_prefs
                            });
                        }
                    }
                    let (w2d, w3d) = stored_widths(&store, mobile_layout());
                    if let Some(w) = w2d {
                        width_2d.set(w);
                    }
                    if let Some(w) = w3d {
                        width_3d.set(w);
                    }
                    // The PWA's recent activity (ADR-0057): history, the
                    // session bindings (`ans` and every assignment), and the
                    // answer on screen — reopening shows where the user left
                    // off instead of a blank slate.
                    let mut restored: Option<Session> = None;
                    if let Ok(Some(json)) = store.get_item("epher-history") {
                        if let Ok(hist) = serde_json::from_str::<Vec<String>>(&json) {
                            let mut s = Session::with_history(hist);
                            if let Ok(Some(json)) = store.get_item("epher-bindings") {
                                if let Ok(bindings) = serde_json::from_str::<
                                    std::collections::HashMap<String, epher_core::Value>,
                                >(&json)
                                {
                                    s.restore_bindings(&bindings);
                                }
                            }
                            s.set_display(*display_prefs);
                            restored = Some(s);
                        }
                    }
                    if let Some(s) = restored {
                        session.set(s.clone());
                        *session_live.borrow_mut() = s;
                    } else {
                        // The loaded display preferences (ADR-0043) shape the
                        // live session from the first submit on.
                        session_live.borrow_mut().set_display(*display_prefs);
                    }
                    if let Ok(Some(text)) = store.get_item("epher-result") {
                        if !text.is_empty() {
                            result.set(text);
                        }
                    }
                }
                // The site's Examples page hands an example over via
                // localStorage on touch devices (ADR-0035 amendment):
                // tapping an example there copies it and opens the app
                // with it staged under `epher-example`. Consume it into
                // the entry with the cursor at its end; on mobile the
                // keypad composes from it without summoning the device
                // keyboard (the same rule as guide code loads).
                if let Some(store) =
                    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
                {
                    if let Ok(Some(text)) = store.get_item("epher-example") {
                        let _ = store.remove_item("epher-example");
                        input.set(text.clone());
                        *cursor_cell.borrow_mut() = (text.chars().count(), text.chars().count());
                        if !mobile_layout() {
                            if let Some(ta) = input_ref.cast::<web_sys::HtmlTextAreaElement>() {
                                let _ = ta.focus();
                            }
                        }
                    }
                }
            }
            // A shared link (ADR-0038): `?expr=...` stages the expression
            // in the entry - the same contents a history pick loads. The
            // parameter is consumed so a reload starts clean.
            if let Some(window) = web_sys::window() {
                if let Ok(search) = window.location().search() {
                    if !search.is_empty() {
                        if let Ok(Some(expr)) = web_sys::UrlSearchParams::new_with_str(&search)
                            .map(|params| params.get("expr"))
                        {
                            if !expr.is_empty() {
                                input.set(expr.clone());
                                *cursor_cell.borrow_mut() =
                                    (expr.chars().count(), expr.chars().count());
                                if !mobile_layout() {
                                    if let Some(ta) =
                                        input_ref.cast::<web_sys::HtmlTextAreaElement>()
                                    {
                                        let _ = ta.focus();
                                    }
                                }
                            }
                        }
                        if let Ok(history) = window.history() {
                            let path = window.location().pathname().unwrap_or_default();
                            let url = format!(
                                "{}{}",
                                window.location().origin().unwrap_or_default(),
                                path
                            );
                            let _ = history.replace_state_with_url(
                                &wasm_bindgen::JsValue::NULL,
                                "",
                                Some(&url),
                            );
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
                if let Some(store) =
                    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
                {
                    let _ = store.set_item("epher-theme", &name);
                }
                bridge.save_theme(&name);
                theme.set(name);
            }
        })
    };

    // Result rendering (ADR-0043): exact fractions, the notation, and
    // thousands separators. Applied to the live session so every later
    // submit formats accordingly, and persisted like the theme.
    let on_set_display = {
        let display_prefs = display_prefs.clone();
        let session_live = session_live.clone();
        let bridge = bridge;
        Callback::from(move |prefs: DisplayPrefs| {
            display_prefs.set(prefs);
            session_live.borrow_mut().set_display(prefs);
            if let Some(store) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
                let _ =
                    store.set_item("epher-exact", if prefs.exact_fractions { "1" } else { "0" });
                let _ = store.set_item("epher-format", notation_of(prefs.notation));
                let _ =
                    store.set_item("epher-separators", if prefs.separators { "1" } else { "0" });
            }
            bridge.save_exact(prefs.exact_fractions);
            bridge.save_format(notation_of(prefs.notation));
            bridge.save_separators(prefs.separators);
        })
    };

    // Set the UI language: re-resolve the localizer and persist it.
    let on_set_language = {
        let localizer = localizer.clone();
        Callback::from(move |code: String| {
            if epher_i18n::SUPPORTED_LOCALES.contains(&code.as_str()) {
                localizer.set(Localizer::resolve(Some(&code), &[]));
                if let Some(store) =
                    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
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
            if let Some(store) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
                let _ = store.set_item("epher-poi-list", if on { "1" } else { "0" });
            }
            poi_list.set(on);
        })
    };
    let on_set_poi_markers = {
        let poi_markers = poi_markers.clone();
        Callback::from(move |on: bool| {
            if let Some(store) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
                let _ = store.set_item("epher-poi-markers", if on { "1" } else { "0" });
            }
            poi_markers.set(on);
        })
    };
    // The line-width sliders (ADR-0020, ADR-0035 amendment, ADR-0055):
    // one slider per graph kind — 2D 0–4 step 0.1, 3D 0–0.4 step 0.05
    // (default 0.2) — and only the kind in view is shown, so the range
    // always matches the plot the user is adjusting. Each kind remembers
    // its own width under its own key (the legacy shared key still seeds
    // both), and each kind's plot renders with its own value. Persisted
    // like the POI toggles, clamped to the slider's range so a stale
    // stored value cannot re-enter.
    let on_set_line_width = {
        let width_2d = width_2d.clone();
        let width_3d = width_3d.clone();
        let surface3d_cell = surface3d_cell.clone();
        Callback::from(move |w: f64| {
            let persist = |key: &str, v: f64| {
                if let Some(store) =
                    web_sys::window().and_then(|w| w.local_storage().ok().flatten())
                {
                    let _ = store.set_item(key, &format!("{v}"));
                }
            };
            if *surface3d_cell.borrow() {
                let w = w.clamp(0.0, 0.4);
                persist("epher-line-width-3d", w);
                width_3d.set(w);
            } else {
                let w = w.clamp(0.0, 4.0);
                persist("epher-line-width-2d", w);
                width_2d.set(w);
            }
        })
    };

    // Crossing the 880px breakpoint re-clamps the width to the new
    // slider range (ADR-0031): the slider element's value is always
    // current, and the setter clamps to whatever layout the window is
    // in now. The `is_mobile` state flips the slider's placement.
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
                    // The flip may have changed the kind's range (ADR-0035
                    // mobile 0-0.2 vs ADR-0055 desktop 0-0.4); keep the
                    // remembered value inside the slider now in view.
                    if let Ok(w) = el.value().parse::<f64>() {
                        if let (Ok(lo), Ok(hi)) = (el.min().parse::<f64>(), el.max().parse::<f64>())
                        {
                            on_set_line_width.emit(w.clamp(lo, hi));
                        }
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
                    .and_then(|v| {
                        v.as_string()
                            .ok_or(())
                            .map_err(|()| wasm_bindgen::JsValue::NULL)
                    })
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
                    .and_then(|v| {
                        v.as_string()
                            .ok_or(())
                            .map_err(|()| wasm_bindgen::JsValue::NULL)
                    })
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
            // Multi-line entries become one escaped line each, so the
            // file's one-line-per-entry shape survives (ADR-0027).
            let text = session
                .history()
                .iter()
                .map(|h| h.replace('\n', "\\n"))
                .collect::<Vec<_>>()
                .join("\n");
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
            if let Some(clipboard) = web_sys::window().map(|w| w.navigator().clipboard()) {
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
                if let Some(clipboard) = web_sys::window().map(|w| w.navigator().clipboard()) {
                    let text_for_clip = text.clone();
                    spawn_local(async move {
                        let _ = wasm_bindgen_futures::JsFuture::from(
                            clipboard.write_text(&text_for_clip),
                        )
                        .await;
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
            let Some(clipboard) = web_sys::window().map(|w| w.navigator().clipboard()) else {
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
                            if let Some(ta) = input_ref.cast::<web_sys::HtmlTextAreaElement>() {
                                let start = ta.selection_start().unwrap_or_default().unwrap_or(0);
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

    // The open suggestion list (ADR-0042), kept in step with the entry by
    // on_input and driven from on_keydown.
    let autocomplete = use_state(|| None::<AutocompleteState>);

    let on_input = {
        let input = input.clone();
        let autocomplete = autocomplete.clone();
        let session_live = session_live.clone();
        let localizer = localizer.clone();
        Callback::from(move |e: InputEvent| {
            let target = e.target_unchecked_into::<HtmlTextAreaElement>();
            let value = target.value();
            input.set(value.clone());
            let caret = target.selection_start().ok().flatten().unwrap_or(0) as usize;
            let next = {
                let session = session_live.borrow();
                suggestions_for(&value, caret, &session, &localizer)
            }
            .map(|(word_start, items)| AutocompleteState {
                word_start,
                caret,
                items,
                selected: 0,
            });
            autocomplete.set(next);
        })
    };

    // Enter submits (the textarea's own Enter would insert a newline);
    // Shift+Enter inserts a newline so multi-line scripts can be composed
    // by hand. Submitting goes through the form so the `=` button and the
    // keyboard share one path. When the suggestion list is open (ADR-0042)
    // the arrows and Tab navigate/accept it first, and Enter accepts the
    // highlighted suggestion instead of submitting. F1 shows the function
    // help for the word under the cursor in the hint bar, and an operator
    // typed into an empty entry auto-inserts `ans` (ADR-0042).
    let on_keydown = {
        let form_ref = form_ref.clone();
        let input = input.clone();
        let input_ref = input_ref.clone();
        let autocomplete = autocomplete.clone();
        let key_hint_bar = key_hint_bar.clone();
        let localizer = localizer.clone();
        Callback::from(move |e: web_sys::KeyboardEvent| {
            let key = e.key();
            if key == "F1" && !e.ctrl_key() && !e.alt_key() && !e.meta_key() {
                e.prevent_default();
                let Some(ta) = input_ref.cast::<HtmlTextAreaElement>() else {
                    return;
                };
                let value = ta.value();
                let caret =
                    (ta.selection_start().ok().flatten().unwrap_or(0) as usize).min(value.len());
                match word_at(&value, caret) {
                    Some((_, word)) => {
                        let hint_key = format!("key-hint-{word}");
                        let hint = localizer.lookup(&hint_key);
                        if hint == hint_key {
                            key_hint_bar.set(localizer.lookup("help-no-description"));
                        } else {
                            key_hint_bar.set(format!("{word}: {hint}"));
                        }
                    }
                    None => key_hint_bar.set(localizer.lookup("help-no-description")),
                }
                return;
            }
            let value = (*input).clone();
            // auto-ans (ADR-0042): an operator typed into an empty entry
            // continues from the previous answer.
            if value.is_empty()
                && key.chars().count() == 1
                && wants_auto_ans(&key)
                && !e.ctrl_key()
                && !e.alt_key()
                && !e.meta_key()
            {
                e.prevent_default();
                let spliced = format!("ans{key}");
                input.set(spliced.clone());
                if let Some(ta) = input_ref.cast::<HtmlTextAreaElement>() {
                    ta.set_value(&spliced);
                    ta.set_selection_start(Some(4)).ok();
                    ta.set_selection_end(Some(4)).ok();
                }
                return;
            }
            // the suggestion list steers the keys while it is open
            let mut fall_through_to_submit = false;
            if let Some(ac) = (*autocomplete).clone() {
                match key.as_str() {
                    "ArrowDown" if !e.shift_key() => {
                        e.prevent_default();
                        let n = ac.items.len().max(1);
                        autocomplete.set(Some(AutocompleteState {
                            selected: (ac.selected + 1) % n,
                            ..ac.clone()
                        }));
                    }
                    "ArrowUp" if !e.shift_key() => {
                        e.prevent_default();
                        let n = ac.items.len().max(1);
                        autocomplete.set(Some(AutocompleteState {
                            selected: (ac.selected + n - 1) % n,
                            ..ac.clone()
                        }));
                    }
                    "Tab" if !e.shift_key() => {
                        e.prevent_default();
                        // A unit word (ADR-0046) never completes: `5 m`
                        // is five metres, not a prefix of `m_P(`.
                        let word = &value[ac.word_start.min(value.len())..];
                        if epher_core::is_unit_token(word) {
                            autocomplete.set(None);
                        } else {
                            accept_suggestion(&ac, ac.selected, &input, &input_ref, &autocomplete);
                        }
                    }
                    "Enter" if !e.shift_key() && !e.is_composing() => {
                        // Enter accepts the highlighted suggestion when that
                        // changes the entry (functions gain their paren); a
                        // fully typed constant accepts as a no-op, so Enter
                        // falls through and evaluates - typing `pi` and
                        // pressing Enter must not feel like a dead key.
                        // A unit word evaluates instead of completing
                        // (ADR-0046): `5 m + 3 m` stays `5 m + 3 m`.
                        let word = &value[ac.word_start.min(value.len())..];
                        if epher_core::is_unit_token(word) {
                            autocomplete.set(None);
                            fall_through_to_submit = true;
                            e.prevent_default();
                        } else {
                            let changes = ac
                                .items
                                .get(ac.selected)
                                .map(|item| apply_suggestion(&value, &ac, item).0 != value)
                                .unwrap_or(false);
                            e.prevent_default();
                            if changes {
                                accept_suggestion(
                                    &ac,
                                    ac.selected,
                                    &input,
                                    &input_ref,
                                    &autocomplete,
                                );
                            } else {
                                autocomplete.set(None);
                                fall_through_to_submit = true;
                            }
                        }
                    }
                    "Escape" => {
                        autocomplete.set(None);
                    }
                    _ => {}
                }
                if matches!(
                    key.as_str(),
                    "Tab" | "Enter" | "ArrowDown" | "ArrowUp" | "Escape"
                ) && !fall_through_to_submit
                {
                    return;
                }
            }
            if key == "Enter" && !e.shift_key() && !e.is_composing() {
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
        let (Some(panes), Some(pane)) = (doc.get_element_by_id("panes"), doc.get_element_by_id(id))
        else {
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
        let session_live = session_live.clone();
        let input = input.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        let theme = theme.clone();
        let graph = graph.clone();
        let pois = pois.clone();
        let trace = trace.clone();
        let hidden = hidden.clone();
        let hidden_surfaces = hidden_surfaces.clone();
        let hidden_curves3d = hidden_curves3d.clone();
        let live = live.clone();
        let surface = surface.clone();
        let curve3ds = curve3ds.clone();
        let solar_handle = solar.clone();
        let solar_source_handle = solar_source.clone();
        let view = view.clone();
        let surface3d_cell = surface3d_cell.clone();
        let scroll_pane = scroll_pane.clone();
        let input_ref = input_ref.clone();
        let view_h = view_h.clone();
        let view_v = view_v.clone();
        let view_h_cell = view_h_cell.clone();
        let view_v_cell = view_v_cell.clone();
        let view_z_cell = view_z_cell.clone();
        let view_z = view_z.clone();
        let spin_phase = spin_phase.clone();
        let spin_phase_cell = spin_phase_cell.clone();
        let view2d = view2d.clone();
        let view2d_cell = view2d_cell.clone();
        let solar_hidden = solar_hidden.clone();
        let data = data.clone();
        let view_cell = view_cell.clone();
        let history_box_ref = history_box_ref.clone();
        let scroll_pane_for_submit = scroll_pane.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            // A submitted entry may be several lines (pasted from the
            // guide, or composed with Shift+Enter). Each line runs in
            // order against one session snapshot — script semantics, like
            // the REPL and piped mode. Yew state handles do not expose
            // writes made earlier in the same callback, so the loop works
            // on locals and the states are published once, after the loop.
            let mut s = session_live.borrow().clone();
            let mut curves = (*graph).clone();
            let mut surfaces = (*surface).clone();
            let mut curve3ds_local = (*curve3ds).clone();
            let mut solar = (*solar_handle).clone();
            let mut solar_source = (*solar_source_handle).clone();
            let mut data_local = (*data).clone();
            // Mobile: a submit that empties the graph pane slides the
            // view back to the calculator (ADR-0035) — the mirror of the
            // draw slide. Tracked before the loop so only a pane that
            // HAD content moves.
            let had_graph = !curves.is_empty()
                || !surfaces.is_empty()
                || !curve3ds_local.is_empty()
                || solar.is_some()
                || data.is_some();
            // Statements join with newlines or `;` — the same separator
            // (ADR-0001). Each piece dispatches in order, exactly as if
            // typed one by one — but the history keeps the script the way
            // the user entered it: a single-line submission is one entry
            // per line (semicolons intact, last answer appended), and a
            // multi-line submission is ONE entry carrying the whole
            // script verbatim (ADR-0027 amendment) — the pieces below
            // must not record their own lines then.
            let raw = (*input).clone();
            let multiline = raw
                .split('\n')
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .count()
                > 1;
            let script_verbatim = raw
                .split('\n')
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            // Every answer the submitted script produced, in order, one
            // per line (ADR-0052): the result area shows the whole
            // transcript, not only the final value. Plots and commands
            // set their own message and are not evaluations.
            let mut script_outputs: Vec<String> = Vec::new();
            let mut last_was_eval = false;
            // `;` and newlines separate statements only where the
            // tokenizer sees them: inside a string literal or a comment
            // they are text, and inside a block comment a newline is
            // comment text. The whole submission splits at once, so a
            // pasted script parses exactly like a script file: the
            // spanning block-comment banner is one comment, not a
            // screenful of phantom parse errors (the per-line split this
            // replaces also bit on semicolons in strings; epher-shell
            // split_statements mirrors the tokenizer).
            let pieces = epher_shell::split_statements(&raw);
            let single = pieces.len() == 1;
            let line = raw.trim().to_string();
            // The output of the last evaluation, for the combined
            // history entry of a multi-statement script.
            let mut last_eval_output: Option<String> = None;
            for piece in &pieces {
                let piece = piece.trim();
                last_eval_output = None;
                last_was_eval = false;

                // The keypad's command keys (ADR-0038): `clear` empties
                // the plot like the Clear graph button; `history`
                // focuses the history list. Both previously fell
                // through to the evaluator and errored as unknown
                // names.
                if piece == "clear" {
                    curves.clear();
                    pois.set(Vec::new());
                    surfaces.clear();
                    curve3ds_local.clear();
                    solar = None;
                    solar_source = None;
                    solar_hidden.set(Vec::new());
                    hidden_surfaces.set(Vec::new());
                    hidden_curves3d.set(Vec::new());
                    view_h.set(0.0);
                    view_v.set(0.0);
                    *view_z_cell.borrow_mut() = 0.0;
                    view_z.set(0.0);
                    spin_phase.set((0.0, 0.0));
                    *spin_phase_cell.borrow_mut() = (0.0, 0.0);
                    *view2d_cell.borrow_mut() = None;
                    view2d.set(None);
                    result.set(String::new());
                    continue;
                }
                if piece == "history" {
                    if let Some(el) = history_box_ref.cast::<web_sys::HtmlElement>() {
                        let _ = el.focus();
                    }
                    if mobile_layout() {
                        scroll_pane_for_submit.emit("calc-pane");
                    }
                    continue;
                }

                // Graphing (ADR-0006/0014: the core samples, the frontend renders).
                // Each `graph` line overlays one more curve; the command
                // itself joins the history list like every submitted line.
                if let Some(source) = piece.strip_prefix("graph ") {
                    let source = source.trim();
                    if single && !multiline {
                        s.record(piece);
                    }
                    if source == "clear" {
                        curves.clear();
                        data_local = None;
                        pois.set(Vec::new());
                        *view2d_cell.borrow_mut() = None;
                        view2d.set(None);
                        continue;
                    }
                    // Data plots (ADR-0044): a scatter, histogram, or
                    // boxplot owns the pane like a solar scene does.
                    if epher_core::graph::is_data_plot_source(source) {
                        match sample_data_plot(source, s.env()) {
                            Ok(plot) => {
                                curves.clear();
                                pois.set(Vec::new());
                                surfaces.clear();
                                curve3ds_local.clear();
                                solar = None;
                                solar_source = None;
                                solar_hidden.set(Vec::new());
                                *view2d_cell.borrow_mut() = None;
                                view2d.set(None);
                                data_local = Some(plot);
                                result.set(String::new());
                                if mobile_layout() {
                                    scroll_pane.emit("graph-pane");
                                }
                            }
                            Err(e) => result.set(format!("error: {e}")),
                        }
                        continue;
                    }
                    match parse_graph_source(source).and_then(|spec| {
                        sample_spec(&spec, 120, s.env()).map(|samples| (spec, samples))
                    }) {
                        Ok((spec, samples)) => {
                            // The pane shows one kind at a time (ADR-0015
                            // amendment): drawing a curve clears any 3D
                            // surfaces and any solar scene, so the two
                            // never share the pane and each plot keeps its
                            // full size.
                            surfaces.clear();
                            curve3ds_local.clear();
                            data_local = None;
                            solar = None;
                            solar_source = None;
                            solar_hidden.set(Vec::new());
                            // A fresh plot re-fits the zoom window (ADR-0038).
                            *view2d_cell.borrow_mut() = None;
                            view2d.set(None);
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
                                if let Some(ta) = input_ref.cast::<web_sys::HtmlTextAreaElement>() {
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
                    if single && !multiline {
                        s.record(piece);
                    }
                    if source == "clear" {
                        surfaces.clear();
                        curve3ds_local.clear();
                        hidden_surfaces.set(Vec::new());
                        hidden_curves3d.set(Vec::new());
                        view_h.set(0.0);
                        view_v.set(0.0);
                        *view_z_cell.borrow_mut() = 0.0;
                        view_z.set(0.0);
                        // The spin loop reads the live cells (not the
                        // states): stale non-zero cells here kept a
                        // fresh graph spinning with the sliders at 0
                        // (the ADR-0038 amendment's animation fix).
                        *view_h_cell.borrow_mut() = 0.0;
                        *view_v_cell.borrow_mut() = 0.0;
                        spin_phase.set((0.0, 0.0));
                        *spin_phase_cell.borrow_mut() = (0.0, 0.0);
                        continue;
                    }
                    // A `param` body is a space curve (ADR-0054);
                    // anything else is a surface.
                    if source.starts_with("param ") {
                        match epher_core::graph::sample_space_curve(source, 240, s.env()) {
                            Ok(fresh) => {
                                // The newest command owns the pane.
                                curves.clear();
                                surfaces.clear();
                                data_local = None;
                                solar = None;
                                solar_source = None;
                                solar_hidden.set(Vec::new());
                                *view2d_cell.borrow_mut() = None;
                                view2d.set(None);
                                if curve3ds_local.is_empty() {
                                    view_h.set(0.0);
                                    view_v.set(0.0);
                                    *view_z_cell.borrow_mut() = 0.0;
                                    view_z.set(0.0);
                                    spin_phase.set((0.0, 0.0));
                                    *spin_phase_cell.borrow_mut() = (0.0, 0.0);
                                }
                                curve3ds_local.push(fresh);
                                result.set(String::new());
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
                    match epher_core::graph::sample_surface(source, 30, s.env()) {
                        Ok(fresh) => {
                            // The pane shows one kind at a time (ADR-0015
                            // amendment): drawing a surface clears any 2D
                            // curves, their points of interest, and any
                            // solar scene — the newest command owns the pane.
                            curves.clear();
                            curve3ds_local.clear();
                            data_local = None;
                            solar = None;
                            solar_source = None;
                            solar_hidden.set(Vec::new());
                            *view2d_cell.borrow_mut() = None;
                            view2d.set(None);
                            // A 3D graph drawn into an empty pane brings
                            // fresh fine-control sliders at their default
                            // 0 (ADR-0031); overlays keep the current pose.
                            if surfaces.is_empty() {
                                view_h.set(0.0);
                                view_v.set(0.0);
                                *view_z_cell.borrow_mut() = 0.0;
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
                                if let Some(ta) = input_ref.cast::<web_sys::HtmlTextAreaElement>() {
                                    let _ = ta.blur();
                                }
                            }
                        }
                        Err(e) => result.set(format!("error: {e}")),
                    }
                    continue;
                }

                // The solar system (ADR-0037): one scene at the
                // evaluated time expression, rendered as orbit
                // curves, trails, and positioned dots (the ADR-0015
                // amendment). The command joins the history list.
                if let Some(source) = piece.strip_prefix("solar3d ") {
                    let source = source.trim();
                    if single && !multiline {
                        s.record(piece);
                    }
                    if source == "clear" {
                        solar = None;
                        solar_source = None;
                        solar_hidden.set(Vec::new());
                        view_h.set(0.0);
                        view_v.set(0.0);
                        *view_z_cell.borrow_mut() = 0.0;
                        view_z.set(0.0);
                        // The spin loop reads the live cells (not the
                        // states): stale non-zero cells here kept a
                        // fresh graph spinning with the sliders at 0
                        // (the ADR-0038 amendment's animation fix).
                        *view_h_cell.borrow_mut() = 0.0;
                        *view_v_cell.borrow_mut() = 0.0;
                        spin_phase.set((0.0, 0.0));
                        *spin_phase_cell.borrow_mut() = (0.0, 0.0);
                        continue;
                    }
                    let jd = match epher_core::astro::eval_jd(source, s.env()) {
                        Ok(jd) => jd,
                        Err(e) => {
                            result.set(format!("error: {e}"));
                            continue;
                        }
                    };
                    match epher_core::astro::solar_scene(jd) {
                        Ok(scene) => {
                            // A fresh scene brings the fine controls
                            // to their defaults, like a fresh 3D graph:
                            // the camera starts above the ecliptic.
                            let home = scene.default_view();
                            // The pane shows one kind at a time.
                            curves.clear();
                            data_local = None;
                            surfaces.clear();
                            solar = Some(scene);
                            solar_source = Some(source.to_string());
                            solar_hidden.set(Vec::new());
                            view.set(home);
                            // The orbit cell follows the fresh pose,
                            // or the first drag would start from the
                            // stale default instead of this view.
                            *view_cell.borrow_mut() = home;
                            view_h.set(0.0);
                            view_v.set(0.0);
                            *view_z_cell.borrow_mut() = 0.0;
                            view_z.set(0.0);
                            // The spin loop reads the live cells: stale
                            // non-zero cells kept a fresh graph spinning
                            // with the sliders at 0 (ADR-0038 amendment).
                            *view_h_cell.borrow_mut() = 0.0;
                            *view_v_cell.borrow_mut() = 0.0;
                            spin_phase.set((0.0, 0.0));
                            *spin_phase_cell.borrow_mut() = (0.0, 0.0);
                            result.set(String::new());
                            if mobile_layout() {
                                scroll_pane.emit("graph-pane");
                                if let Some(ta) = input_ref.cast::<web_sys::HtmlTextAreaElement>() {
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
                    // A table is a computation: the command joins the
                    // history list like every submitted line (the graph
                    // precedent, ADR-0027) — picking it loads the
                    // command, and re-running it regenerates the table.
                    // A multi-statement line records once at the tail,
                    // so only a single statement records here.
                    if matches!(cmd, epher_shell::Command::Table { .. }) && single && !multiline {
                        s.record(piece);
                    }
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

                let out = if single && !multiline {
                    s.submit(piece)
                } else {
                    s.submit_quiet(piece)
                };
                last_was_eval = true;
                if !out.is_empty() {
                    script_outputs.push(out.clone());
                }
                last_eval_output = Some(out);
            }
            if !multiline && !single {
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
            if multiline {
                // One history entry for the whole multi-line script:
                // the script verbatim, no answer suffix (the lines above
                // recorded nothing) — and `save script` persists the
                // script, not its last statement.
                s.record(&script_verbatim);
                s.set_last_line(&script_verbatim);
            }
            // The result area: every answer the script produced, in
            // order, joined with the private separator the renderer
            // turns into the same-line semicolon layout. A plot or
            // command finishing the script keeps its own message
            // (graphs print nothing, ADR-0027).
            if last_was_eval {
                let joined = script_outputs.join(&ANSWER_SEP.to_string());
                // A long result renders in the result pane (ADR-0056):
                // on mobile it slides into view, and the entry drops the
                // focus so the keyboard closes for it (ADR-0035's slide
                // contract, now for transcripts as well as plots).
                let to_pane = !answer_fits(&joined);
                result.set(joined);
                if to_pane && mobile_layout() {
                    scroll_pane_for_submit.emit("graph-pane");
                    if let Some(ta) = input_ref.cast::<web_sys::HtmlTextAreaElement>() {
                        let _ = ta.blur();
                    }
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
            // Mobile (ADR-0035): once the graph pane has been cleared,
            // slide back to the calculator — there is nothing left to
            // look at over there. Computed before the moves below.
            let cleared = mobile_layout()
                && had_graph
                && curves.is_empty()
                && surfaces.is_empty()
                && curve3ds_local.is_empty()
                && solar.is_none();
            // A fresh plot: every legend checkbox returns to checked.
            hidden.set(vec![false; curves.len()]);
            hidden_surfaces.set(Vec::new());
            hidden_curves3d.set(Vec::new());
            graph.set(curves);
            data.set(data_local);
            surface.set(surfaces.clone());
            curve3ds.set(curve3ds_local.clone());
            solar_handle.set(solar);
            solar_source_handle.set(solar_source);
            *surface3d_cell.borrow_mut() =
                !surfaces.is_empty() || !curve3ds_local.is_empty() || solar_handle.is_some();
            pois.set(labels);
            trace.set(None);
            session.set(s.clone());
            *session_live.borrow_mut() = s.clone();
            input.set(String::new());
            if cleared {
                scroll_pane.emit("calc-pane");
            }
            // Desktop apps are killed, not exited: persist per line (ADR-0010).
            if bridge == Bridge::Tauri {
                bridge.save_history(s.history());
                // The shared session snapshot travels with it: `ans` and
                // user assignments survive into the next frontend.
                bridge.save_session_state(s.bindings());
            } else if let Some(store) =
                web_sys::window().and_then(|w| w.local_storage().ok().flatten())
            {
                // The PWA's recent activity (ADR-0057): history, bindings,
                // and the answer on screen, saved per line like the
                // desktop store.
                if let Some(json) = json_string(&s.history()) {
                    let _ = store.set_item("epher-history", &json);
                }
                if let Some(json) = json_string(&s.bindings()) {
                    let _ = store.set_item("epher-bindings", &json);
                }
            }
        })
    };

    // Sliders: adjusting a constant re-samples every curve against the new
    // environment and re-runs the analysis (ADR-0014).
    let on_slider = {
        let session = session.clone();
        let session_live = session_live.clone();
        let graph = graph.clone();
        let pois = pois.clone();
        let localizer = localizer.clone();
        let surface = surface.clone();
        let curve3ds = curve3ds.clone();
        let solar_handle = solar.clone();
        let solar_source_handle = solar_source.clone();
        let surface3d_cell = surface3d_cell.clone();
        let hidden = hidden.clone();
        Callback::from(move |(name, value): (String, f64)| {
            let mut s = session_live.borrow().clone();
            s.set_constant(
                name.clone(),
                Value::float(value),
                format!("const {name} = {value}"),
            );
            let mut curves = (*graph).clone();
            resample(&mut curves, &s);
            let mut surfaces = (*surface).clone();
            resample_surfaces(&mut surfaces, &s);
            let mut curve3ds_local = (*curve3ds).clone();
            resample_space_curves(&mut curve3ds_local, &s);
            let mut solar = (*solar_handle).clone();
            resample_solar(&mut solar, &*solar_source_handle, &s);
            let found = analyze(&curves, s.env());
            session.set(s.clone());
            *session_live.borrow_mut() = s;
            hidden.set(vec![false; curves.len()]);
            graph.set(curves);
            surface.set(surfaces.clone());
            curve3ds.set(curve3ds_local.clone());
            solar_handle.set(solar);
            *surface3d_cell.borrow_mut() =
                !surfaces.is_empty() || !curve3ds_local.is_empty() || solar_handle.is_some();
            pois.set(poi_labels(&found, &localizer));
        })
    };

    // The playback tick: what the animation loop applies every step. It
    // is deliberately lighter than a slider drag — the loop runs at a
    // fixed cadence and must not fall behind on weak devices:
    //   - only curves/surfaces that reference the animated constant are
    //     re-sampled (the rest keep their samples),
    //   - the points-of-interest analysis runs at 2 Hz (every 4th tick),
    //     not per tick — the markers track the moving curve, but the
    //     bisection/golden-section work does not gate every frame,
    //   - the visibility checkboxes are never rewritten mid-playback.
    // No storage is touched: the shared store only sees user actions
    // (submit, play/pause, slider drags commit on release via submit;
    // the animation itself never persists).
    let on_tick = {
        let session = session.clone();
        let session_live = session_live.clone();
        let graph = graph.clone();
        let pois = pois.clone();
        let localizer = localizer.clone();
        let surface = surface.clone();
        let curve3ds = curve3ds.clone();
        let solar_handle = solar.clone();
        let solar_source_handle = solar_source.clone();
        let surface3d_cell = surface3d_cell.clone();
        let hidden = hidden.clone();
        let tick_no = tick_no.clone();
        let result = result.clone();
        let live = live.clone();
        Callback::from(move |(name, value): (String, f64)| {
            // Two steps: a borrow_mut must never be alive while the
            // increment reads (that panics and kills the loop task).
            let n = tick_no.borrow().wrapping_add(1);
            *tick_no.borrow_mut() = n;
            let mut s = session_live.borrow().clone();
            s.set_constant(
                name.clone(),
                Value::float(value),
                format!("const {name} = {value}"),
            );
            let mut curves = (*graph).clone();
            for c in curves.iter_mut() {
                if curve_references(c, &name) {
                    if let Ok(samples) = sample_spec(&curve_spec(c), 120, s.env()) {
                        c.samples = samples;
                    }
                }
            }
            let mut surfaces = (*surface).clone();
            for sf in surfaces.iter_mut() {
                if surface_references(sf, &name) {
                    if let Ok(fresh) = epher_core::graph::sample_surface(&sf.source, 30, s.env()) {
                        *sf = fresh;
                    }
                }
            }
            // A space curve replays through the same constant animation.
            let mut curve3ds_local = (*curve3ds).clone();
            for c in curve3ds_local.iter_mut() {
                if space_curve_references(c, &name) {
                    if let Ok(fresh) =
                        epher_core::graph::sample_space_curve(&c.source, 240, s.env())
                    {
                        *c = fresh;
                    }
                }
            }
            // The solar system replays through the same transport: the
            // scene rebuilds only when its time expression references the
            // animated constant (ADR-0037).
            let mut solar = (*solar_handle).clone();
            resample_solar(&mut solar, &*solar_source_handle, &s);
            if n % 4 == 0 {
                let found = analyze(&curves, s.env());
                pois.set(poi_labels(&found, &localizer));
            }
            session.set(s.clone());
            *session_live.borrow_mut() = s;
            if (*hidden).len() != curves.len() {
                hidden.set(vec![false; curves.len()]);
            }
            {
                let mut l = (*live).borrow_mut();
                l.curves = curves.clone();
            }
            graph.set(curves);
            surface.set(surfaces.clone());
            curve3ds.set(curve3ds_local.clone());
            solar_handle.set(solar);
            *surface3d_cell.borrow_mut() =
                !surfaces.is_empty() || !curve3ds_local.is_empty() || solar_handle.is_some();
        })
    };

    // The same playback logic, shared with the animation loop through a
    // live cell (Yew handles captured by the loop would go stale). The
    // cell is refreshed after every render.
    let live_apply = use_state(|| Rc::new(RefCell::new(None::<Rc<dyn Fn(String, f64)>>)));
    {
        let live_apply = live_apply.clone();
        let on_tick = on_tick.clone();
        use_effect(move || {
            let apply: Rc<dyn Fn(String, f64)> = Rc::new(move |name: String, value: f64| {
                on_tick.emit((name, value));
            });
            *live_apply.borrow_mut() = Some(apply);
            || {}
        });
    }

    let on_set_view = {
        let view_h = view_h.clone();
        let view_v = view_v.clone();
        let view_h_cell = view_h_cell.clone();
        let view_v_cell = view_v_cell.clone();
        let view_z_cell = view_z_cell.clone();
        let view_z = view_z.clone();
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
                _ => {
                    *view_z_cell.borrow_mut() = v;
                    view_z.set(v);
                }
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
                // One step per 120 ms: a fresh constant's slider spans
                // ±10 (200 steps), so one full cycle takes 24 s — the
                // vendor norm for playback speed.
                // wasm32 has no std clock: the deadlines ride on
                // js_sys::Date::now() (like the spin loop below).
                let step = 120.0; // ms
                let mut next = js_sys::Date::now() + step;
                loop {
                    if (*play_cell).borrow().is_none() {
                        gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
                        next = js_sys::Date::now() + step;
                        continue;
                    }
                    let Some(spec) = (*play_cell).borrow().clone() else {
                        continue;
                    };
                    let stepped = spec.ticked();
                    *play_cell.borrow_mut() = Some(stepped.clone());
                    if let Some(apply) = (*live_apply).borrow().as_ref() {
                        apply(stepped.name.clone(), stepped.value);
                    }
                    // Work first, then rest until the deadline: the
                    // period stays 120 ms whenever the tick fits inside
                    // it, and an overrunning tick never compounds (the
                    // next deadline is still one step away).
                    let now = js_sys::Date::now();
                    if now < next {
                        gloo_timers::future::sleep(std::time::Duration::from_millis(
                            (next - now) as u64,
                        ))
                        .await;
                    }
                    next += step;
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
                let (lo, hi) = slider_span(value);
                let mut next = value + 0.1;
                if next > hi {
                    next = lo;
                }
                if let Some(apply) = (*live_apply).borrow().as_ref() {
                    apply(name.clone(), next);
                }
                return;
            }
            let (lo, hi) = slider_span(value);
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
        let view2d_cell = view2d_cell.clone();
        Callback::from(move |(px, py): (f64, f64)| {
            let found = {
                let l = (*live).borrow();
                let geom = match (*view2d_cell).borrow().clone() {
                    Some((lo, hi)) => graph::geometry_in(&l.curves, lo, hi),
                    None => graph::geometry(&l.curves),
                };
                geom.and_then(|geom| graph::trace_nearest(&l.curves, &geom, px, py))
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
        let data = data.clone();
        let pois = pois.clone();
        let hidden = hidden.clone();
        let hidden_surfaces = hidden_surfaces.clone();
        let hidden_curves3d = hidden_curves3d.clone();
        let trace = trace.clone();
        let poi_markers = poi_markers.clone();
        let width_2d = width_2d.clone();
        let width_3d = width_3d.clone();
        let surface = surface.clone();
        let curve3ds = curve3ds.clone();
        let solar = solar.clone();
        let view = view.clone();
        let view_h = view_h.clone();
        let view_v = view_v.clone();
        let view_z = view_z.clone();
        let spin_phase = spin_phase.clone();
        let view2d = view2d.clone();
        let solar_hidden = solar_hidden.clone();
        let theme = theme.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        Callback::from(move |_| {
            // The export shows what the pane shows (ADR-0015 amendment):
            // curves hidden by their legend checkboxes stay out of the
            // SVG, and so do their points of interest.
            let visible: Vec<(usize, epher_core::graph::SampledCurve)> = (*curves)
                .iter()
                .enumerate()
                .filter(|(i, _)| !(*hidden).get(*i).copied().unwrap_or(false))
                .map(|(i, c)| (i, c.clone()))
                .collect();
            let pois_visible: Vec<graph::Poi> = (*pois)
                .iter()
                .filter(|p| !(*hidden).get(p.curve).copied().unwrap_or(false))
                .cloned()
                .collect();
            // 3D scene elements hidden through their legends stay out of
            // the export too, each keeping its palette index (ADR-0055).
            let visible_curves3d: Vec<(usize, epher_core::graph::SpaceCurve)> = (*curve3ds)
                .iter()
                .enumerate()
                .filter(|(i, _)| !(*hidden_curves3d).contains(i))
                .map(|(i, c)| (i, c.clone()))
                .collect();
            let visible_surfaces3d: Vec<(usize, epher_core::graph::Surface)> = (*surface)
                .iter()
                .enumerate()
                .filter(|(i, _)| !(*hidden_surfaces).contains(i))
                .map(|(i, s)| (i, s.clone()))
                .collect();
            // The export wears the app theme's palette and carries the
            // pane's legend entries (ADR-0057): the picture answers to the
            // same colors, widths, zoom, and captions the pane shows.
            let palette = export_palette(&theme);
            let view3d = effective_view(&view, *view_h, *view_v, *view_z, *spin_phase);
            let svg = if let Some(data) = (*data).as_ref() {
                // The data plot's zoom window clips the export, exactly
                // as the pane shows it (ADR-0055).
                let mut legend = vec![graph::LegendEntry {
                    color: graph::palette_curve(palette, 0).to_string(),
                    caption: data.source.trim().to_string(),
                }];
                if let Some(f) = data.fit {
                    legend[0].caption.push(' ');
                    legend[0].caption.push_str(&graph::fit_legend(&f));
                }
                graph::data_svg_styled(data, *view2d, *width_2d, palette, &legend)
            } else if !visible.is_empty() {
                let legend: Vec<graph::LegendEntry> = visible
                    .iter()
                    .map(|(i, c)| graph::LegendEntry {
                        color: graph::palette_curve(palette, *i).to_string(),
                        caption: graph::curve_caption(c),
                    })
                    .collect();
                graph::graph_svg_styled(
                    &visible,
                    &pois_visible,
                    *trace,
                    *poi_markers,
                    *width_2d,
                    palette,
                    &legend,
                )
            } else if let Some(scene) = (*solar).as_ref() {
                // The legend's unchecked bodies stay out of the export too,
                // through the same filtered scene the pane renders.
                let shown = filter_solar_scene(scene, &solar_hidden);
                let legend: Vec<graph::LegendEntry> = shown
                    .dots
                    .iter()
                    .map(|d| graph::LegendEntry {
                        color: epher_core::graph_svg::solar_color(d.body).to_string(),
                        caption: epher_core::astro::body_name(d.body).to_string(),
                    })
                    .collect();
                graph::solar3d_styled(&shown, &view3d, *width_3d, palette, &legend)
                    .unwrap_or_default()
            } else if !visible_curves3d.is_empty() {
                let legend: Vec<graph::LegendEntry> = visible_curves3d
                    .iter()
                    .map(|(i, c)| graph::LegendEntry {
                        color: graph::palette_curve(palette, *i).to_string(),
                        caption: format!("param {}", c.source.trim()),
                    })
                    .collect();
                graph::graph3d_curve_svg_styled(
                    &visible_curves3d,
                    &view3d,
                    *width_3d,
                    palette,
                    &legend,
                )
                .unwrap_or_default()
            } else {
                let legend: Vec<graph::LegendEntry> = visible_surfaces3d
                    .iter()
                    .map(|(i, s)| graph::LegendEntry {
                        color: graph::palette_curve(palette, *i).to_string(),
                        caption: format!("z = {}", s.source.trim()),
                    })
                    .collect();
                graph::graph3d_svg_styled(&visible_surfaces3d, &view3d, *width_3d, palette, &legend)
                    .unwrap_or_default()
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

    // Save PNG (ADR-0042): the same document Copy SVG produces, rasterized
    // at twice its size so curves stay crisp, and saved through the same
    // flow a script save uses - the desktop dialog over IPC, the browser's
    // picker, else a plain download.
    let on_save_png = {
        let curves = graph.clone();
        let data = data.clone();
        let pois = pois.clone();
        let hidden = hidden.clone();
        let hidden_surfaces = hidden_surfaces.clone();
        let hidden_curves3d = hidden_curves3d.clone();
        let trace = trace.clone();
        let poi_markers = poi_markers.clone();
        let width_2d = width_2d.clone();
        let width_3d = width_3d.clone();
        let surface = surface.clone();
        let curve3ds = curve3ds.clone();
        let solar = solar.clone();
        let view = view.clone();
        let view_h = view_h.clone();
        let view_v = view_v.clone();
        let view_z = view_z.clone();
        let spin_phase = spin_phase.clone();
        let view2d = view2d.clone();
        let solar_hidden = solar_hidden.clone();
        let theme = theme.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        Callback::from(move |_| {
            // Exactly the Copy SVG export (ADR-0015 amendment): what the
            // pane shows is what saves.
            let visible: Vec<(usize, epher_core::graph::SampledCurve)> = (*curves)
                .iter()
                .enumerate()
                .filter(|(i, _)| !(*hidden).get(*i).copied().unwrap_or(false))
                .map(|(i, c)| (i, c.clone()))
                .collect();
            let pois_visible: Vec<graph::Poi> = (*pois)
                .iter()
                .filter(|p| !(*hidden).get(p.curve).copied().unwrap_or(false))
                .cloned()
                .collect();
            // 3D scene elements hidden through their legends stay out of
            // the export too, each keeping its palette index (ADR-0055).
            let visible_curves3d: Vec<(usize, epher_core::graph::SpaceCurve)> = (*curve3ds)
                .iter()
                .enumerate()
                .filter(|(i, _)| !(*hidden_curves3d).contains(i))
                .map(|(i, c)| (i, c.clone()))
                .collect();
            let visible_surfaces3d: Vec<(usize, epher_core::graph::Surface)> = (*surface)
                .iter()
                .enumerate()
                .filter(|(i, _)| !(*hidden_surfaces).contains(i))
                .map(|(i, s)| (i, s.clone()))
                .collect();
            // The export wears the app theme's palette and carries the
            // pane's legend entries (ADR-0057): the picture answers to the
            // same colors, widths, zoom, and captions the pane shows.
            let palette = export_palette(&theme);
            let view3d = effective_view(&view, *view_h, *view_v, *view_z, *spin_phase);
            let svg = if let Some(data) = (*data).as_ref() {
                // The data plot's zoom window clips the export, exactly
                // as the pane shows it (ADR-0055).
                let mut legend = vec![graph::LegendEntry {
                    color: graph::palette_curve(palette, 0).to_string(),
                    caption: data.source.trim().to_string(),
                }];
                if let Some(f) = data.fit {
                    legend[0].caption.push(' ');
                    legend[0].caption.push_str(&graph::fit_legend(&f));
                }
                graph::data_svg_styled(data, *view2d, *width_2d, palette, &legend)
            } else if !visible.is_empty() {
                let legend: Vec<graph::LegendEntry> = visible
                    .iter()
                    .map(|(i, c)| graph::LegendEntry {
                        color: graph::palette_curve(palette, *i).to_string(),
                        caption: graph::curve_caption(c),
                    })
                    .collect();
                graph::graph_svg_styled(
                    &visible,
                    &pois_visible,
                    *trace,
                    *poi_markers,
                    *width_2d,
                    palette,
                    &legend,
                )
            } else if let Some(scene) = (*solar).as_ref() {
                // The legend's unchecked bodies stay out of the export too,
                // through the same filtered scene the pane renders.
                let shown = filter_solar_scene(scene, &solar_hidden);
                let legend: Vec<graph::LegendEntry> = shown
                    .dots
                    .iter()
                    .map(|d| graph::LegendEntry {
                        color: epher_core::graph_svg::solar_color(d.body).to_string(),
                        caption: epher_core::astro::body_name(d.body).to_string(),
                    })
                    .collect();
                graph::solar3d_styled(&shown, &view3d, *width_3d, palette, &legend)
                    .unwrap_or_default()
            } else if !visible_curves3d.is_empty() {
                let legend: Vec<graph::LegendEntry> = visible_curves3d
                    .iter()
                    .map(|(i, c)| graph::LegendEntry {
                        color: graph::palette_curve(palette, *i).to_string(),
                        caption: format!("param {}", c.source.trim()),
                    })
                    .collect();
                graph::graph3d_curve_svg_styled(
                    &visible_curves3d,
                    &view3d,
                    *width_3d,
                    palette,
                    &legend,
                )
                .unwrap_or_default()
            } else {
                let legend: Vec<graph::LegendEntry> = visible_surfaces3d
                    .iter()
                    .map(|(i, s)| graph::LegendEntry {
                        color: graph::palette_curve(palette, *i).to_string(),
                        caption: format!("z = {}", s.source.trim()),
                    })
                    .collect();
                graph::graph3d_svg_styled(&visible_surfaces3d, &view3d, *width_3d, palette, &legend)
                    .unwrap_or_default()
            };
            if svg.is_empty() {
                return;
            }
            save_png_with_dialog(bridge, svg, "epher-plot.png", &result, &localizer);
        })
    };

    // Zoom the 2D plot (ADR-0038): a wheel notch, a pinch, or the zoom
    // slider lands here. Cartesian curves re-sample over the new window
    // (param and polar keep their parameter samples - the view clips
    // them), the trace clears, and the window cell leads so the
    // mount-time listeners read fresh values.
    let apply_zoom_window = {
        let live = live.clone();
        let session_live = session_live.clone();
        let graph = graph.clone();
        let trace = trace.clone();
        let data_state = data.clone();
        let view2d = view2d.clone();
        let view2d_cell = view2d_cell.clone();
        // `None` is the auto-fit (the reset action): curves re-sample
        // over their own stored domains, data plots drop the clip.
        Rc::new(move |window: Option<(f64, f64)>| {
            if data_state.is_some() {
                // A data plot clips at render time (ADR-0055): the window
                // state alone drives the picture.
                *view2d_cell.borrow_mut() = window;
                view2d.set(window);
                return;
            }
            let s = session_live.borrow().clone();
            let mut curves = (*live).borrow().curves.clone();
            if curves.is_empty() {
                return;
            }
            for c in curves.iter_mut() {
                if matches!(c.kind, epher_core::graph::CurveKind::Cartesian(_)) {
                    // Zoomed: sample inside the window; reset: back to
                    // the curve's own stored domain.
                    let domain = match window {
                        Some((lo, hi)) => (lo, hi),
                        None => c.domain,
                    };
                    let spec = epher_core::graph::CurveSpec {
                        kind: c.kind.clone(),
                        domain,
                        fill: c.fill,
                    };
                    if let Ok(samples) = epher_core::graph::sample_spec(&spec, 120, s.env()) {
                        c.samples = samples;
                    }
                }
            }
            (*live).borrow_mut().trace = None;
            trace.set(None);
            *view2d_cell.borrow_mut() = window;
            view2d.set(window);
            graph.set(curves);
        })
    };
    // The base window of whichever kind owns the pane: the 2D curves'
    // shared geometry, or the data plot's own ranges.
    let current_zoom_base = {
        let live = live.clone();
        let data_state = data.clone();
        move || -> Option<(f64, f64)> {
            if let Some(d) = data_state.as_ref() {
                graph::data_geometry(d, None).map(|g| (g.x_min, g.x_max))
            } else {
                let l = live.borrow();
                graph::geometry(&l.curves).map(|g| (g.x_min, g.x_max))
            }
        }
    };
    let on_reset_zoom2d = {
        let apply = apply_zoom_window.clone();
        Callback::from(move |_: web_sys::MouseEvent| apply(None))
    };
    let on_zoom2d = {
        let base_of = current_zoom_base.clone();
        let view2d_cell = view2d_cell.clone();
        let apply = apply_zoom_window.clone();
        Callback::from(move |(px, py, factor): (f64, f64, f64)| {
            let _ = py;
            let Some(base) = base_of() else {
                return;
            };
            let cur = (*view2d_cell).borrow().unwrap_or(base);
            // The anchor's data x under the current window.
            let anchor =
                cur.0 + (px - graph::LEFT) / (graph::RIGHT - graph::LEFT) * (cur.1 - cur.0);
            apply(Some(anchored_window(cur, anchor, factor, base.1 - base.0)));
        })
    };
    let on_set_zoom2d = {
        let base_of = current_zoom_base.clone();
        let view2d_cell = view2d_cell.clone();
        let apply = apply_zoom_window.clone();
        Callback::from(move |z: f64| {
            let Some(base) = base_of() else {
                return;
            };
            let cur = (*view2d_cell).borrow().unwrap_or(base);
            let center = (cur.0 + cur.1) / 2.0;
            apply(Some(slider_window(z, base, center)));
        })
    };
    // Wheel and pinch zoom the 3D scene (ADR-0038, ADR-0055): the zoom
    // state lives in the zoom slider's value, so a wheel notch moves the
    // zoom slider exactly as it does on the 2D pane - the consistency the
    // user asked for. The state may leave the slider's ±1 range (the
    // slider pins at its end while the zoom goes deeper), mirroring the
    // 2D pane's wheel-past-the-ends rule. The camera base itself never
    // moves: the effective camera is base × 10^(−2z).
    let on_zoom3d = {
        let view_z = view_z.clone();
        let view_z_cell = view_z_cell.clone();
        Callback::from(move |factor: f64| {
            let z = *view_z_cell.borrow();
            // camera ∝ 10^(−2z): a camera factor f is a shift of
            // −log10(f)/2 in z. The clamp keeps the effective camera in
            // the band the old direct-camera zoom used (0.01..1e7).
            let zmin = -0.5 * (1e7_f64 / 30.0).log10();
            let zmax = -0.5 * (0.01_f64 / 30.0).log10();
            let next = (z - 0.5 * factor.log10()).clamp(zmin, zmax);
            *view_z_cell.borrow_mut() = next;
            view_z.set(next);
        })
    };
    // Reset a 3D fine-control slider to its default (ADR-0055): pressing
    // the icon beside the slider. The h/v live cells reset too, or a
    // stale non-zero cell would keep a "zeroed" plot spinning.
    let on_reset_view = {
        let view_h = view_h.clone();
        let view_v = view_v.clone();
        let view_z_cell = view_z_cell.clone();
        let view_z = view_z.clone();
        let view_h_cell = view_h_cell.clone();
        let view_v_cell = view_v_cell.clone();
        Callback::from(move |axis: &'static str| match axis {
            "h" => {
                *view_h_cell.borrow_mut() = 0.0;
                view_h.set(0.0);
            }
            "v" => {
                *view_v_cell.borrow_mut() = 0.0;
                view_v.set(0.0);
            }
            _ => {
                *view_z_cell.borrow_mut() = 0.0;
                *view_z_cell.borrow_mut() = 0.0;
                view_z.set(0.0);
            }
        })
    };
    let on_reset_width_2d = {
        let on_set_line_width = on_set_line_width.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            on_set_line_width.emit(graph::DEFAULT_STROKE_WIDTH)
        })
    };
    let on_reset_width_3d = {
        let on_set_line_width = on_set_line_width.clone();
        Callback::from(move |_: web_sys::MouseEvent| {
            on_set_line_width.emit(graph::three_d_default_width(mobile_layout()))
        })
    };
    // The solar legend's per-body checkboxes (ADR-0038): like the curve
    // legend, unchecking hides the body from the plot and the export.
    let on_toggle_solar_body = {
        let solar_hidden = solar_hidden.clone();
        Callback::from(move |(body, on): (i64, bool)| {
            let mut hidden = (*solar_hidden).clone();
            if on {
                hidden.retain(|b| *b != body);
            } else if !hidden.contains(&body) {
                hidden.push(body);
            }
            solar_hidden.set(hidden);
        })
    };
    // Copy the answer (ADR-0057): the values behind what's on screen —
    // the answer line's single answer or the result pane's whole
    // transcript — one per line, onto the clipboard. The icon answers
    // with a check for a moment; the answer itself never moves.
    let on_copy_answer = {
        let result = result.clone();
        let answer_copied = answer_copied.clone();
        let localizer = localizer.clone();
        Callback::from(move |_| {
            let text = answer_clip(&result);
            if text.is_empty() {
                return;
            }
            answer_copied.set(true);
            if let Some(window) = web_sys::window() {
                let answer_copied = answer_copied.clone();
                let reset = Closure::once(move || answer_copied.set(false));
                let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                    reset.as_ref().unchecked_ref(),
                    1600,
                );
                reset.forget();
            }
            let result = result.clone();
            let localizer = localizer.clone();
            spawn_local(async move {
                let ok = match web_sys::window().map(|w| w.navigator().clipboard()) {
                    Some(clipboard) => clipboard.write_text(&text).await.is_ok(),
                    None => false,
                };
                if !ok {
                    result.set(localizer.lookup("answer-copy-failed"));
                }
            });
        })
    };

    // Copy the points of interest (ADR-0038): the same lines the list
    // shows, one per line, onto the clipboard.
    let on_copy_pois = {
        let pois = pois.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        Callback::from(move |_| {
            let text = (*pois)
                .iter()
                .map(|p| format!("{} ({}, {})", p.label, graph::label(p.x), graph::label(p.y)))
                .collect::<Vec<_>>()
                .join("\n");
            let result = result.clone();
            let localizer = localizer.clone();
            spawn_local(async move {
                match web_sys::window().map(|w| w.navigator().clipboard()) {
                    Some(clipboard) => match clipboard.write_text(&text).await {
                        Ok(_) => result.set(localizer.lookup("poi-copied")),
                        Err(_) => result.set(localizer.lookup("graph-copy-failed")),
                    },
                    None => result.set(localizer.lookup("graph-copy-failed")),
                }
            });
        })
    };
    // Share a history line (ADR-0038, amended by ADR-0040): the OS
    // share sheet (the web share API) carries the app link with the
    // line's expression in the text; without the API the message, the
    // expression, and the link land on the clipboard together.
    let on_share = {
        let result = result.clone();
        let localizer = localizer.clone();
        Callback::from(move |line: String| {
            let expr = history_expression(&line).to_string();
            let link = share_link(&expr);
            // The share reads as three lines: the message, the
            // expression, the link (ADR-0040). The sheet keeps the link
            // as its own field; the clipboard fallback writes the three
            // lines together.
            let text = format!("{}\n{}", localizer.lookup("share-text"), expr);
            let text_link = format!("{text}\n{link}");
            let result = result.clone();
            let localizer = localizer.clone();
            let shared = web_sys::window().and_then(|w| {
                let navigator = w.navigator();
                let nav: &wasm_bindgen::JsValue = navigator.as_ref();
                js_sys::Reflect::has(nav, &JsValue::from_str("share"))
                    .ok()
                    .filter(|has| *has)
                    .map(|_| navigator)
            });
            if let Some(navigator) = shared {
                let obj = js_sys::Object::new();
                let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("title"), &"epher".into());
                let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("text"), &text.into());
                let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("url"), &link.clone().into());
                let nav: &wasm_bindgen::JsValue = navigator.as_ref();
                let promise = js_sys::Reflect::get(nav, &JsValue::from_str("share"))
                    .ok()
                    .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
                    .and_then(|f| f.call1(nav, &obj.into()).ok())
                    .and_then(|v| v.dyn_into::<js_sys::Promise>().ok());
                if let Some(promise) = promise {
                    spawn_local(async move {
                        // A cancel (AbortError) is the user's own choice,
                        // not an error; the sheet already showed it.
                        let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
                    });
                    return;
                }
            }
            spawn_local(async move {
                match web_sys::window().map(|w| w.navigator().clipboard()) {
                    Some(clipboard) => match clipboard.write_text(&text_link).await {
                        Ok(_) => result.set(localizer.lookup("share-copied")),
                        Err(_) => result.set(localizer.lookup("share-failed")),
                    },
                    None => result.set(localizer.lookup("share-failed")),
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
                (s.min(v.chars().count()), e.min(v.chars().count()))
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
                    let (lo, hi) = if s == e {
                        (s.saturating_sub(1), s)
                    } else {
                        (s, e)
                    };
                    v.replace_range(char_byte(&v, lo)..char_byte(&v, hi), "");
                    input.set(v.clone());
                    ta.set_value(&v);
                    ta.set_selection_start(Some(lo as u32)).ok();
                    ta.set_selection_end(Some(lo as u32)).ok();
                    new_cursor = (lo, lo);
                }
                KeyAction::Text(t) => {
                    let mut v = (*input).clone();
                    let (s, e) = cursor(&v);
                    // ADR-0042 auto-ans: an operator on an empty entry
                    // continues from the previous answer.
                    let owned;
                    let token: &str = if v.is_empty() && wants_auto_ans(t) {
                        owned = format!("ans{t}");
                        &owned
                    } else {
                        t
                    };
                    v.replace_range(char_byte(&v, s)..char_byte(&v, e), token);
                    input.set(v.clone());
                    ta.set_value(&v);
                    let pos = s + token.chars().count();
                    ta.set_selection_start(Some(pos as u32)).ok();
                    ta.set_selection_end(Some(pos as u32)).ok();
                    new_cursor = (pos, pos);
                }
                KeyAction::Call(name) => {
                    let mut v = (*input).clone();
                    let (s, e) = cursor(&v);
                    let t = format!("{name}(");
                    v.replace_range(char_byte(&v, s)..char_byte(&v, e), &t);
                    input.set(v.clone());
                    ta.set_value(&v);
                    let pos = s + t.chars().count();
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
                if let (Some(ta), Some(active)) =
                    (input_ref.cast::<HtmlTextAreaElement>(), active_ta)
                {
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
                // While this constant plays, the slider window is the play
                // span fixed at press time: the thumb travels the whole
                // track and wraps cleanly from the right end back to the
                // left. A window that followed the value would stall the
                // thumb mid-track once the value left the −8..8 core and
                // the span began sliding (the ADR-0055 animation fix).
                let playing_this = (*play).as_ref().is_some_and(|p| p.name == *name);
                let (lo, hi) = match (*play).as_ref().filter(|p| p.name == *name) {
                    Some(p) => (p.lo, p.hi),
                    None => slider_span(v),
                };
                let on_slider = on_slider.clone();
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
                // Grabbing the slider indicator while playback runs stops
                // the animation on the pointer-down, before the thumb can
                // run away under the finger; the drag then moves the
                // value normally (ADR-0055).
                let stop_on_grab = {
                    let play = play.clone();
                    let play_cell = play_cell.clone();
                    let name = name.clone();
                    Callback::from(move |_: web_sys::PointerEvent| {
                        if play.as_ref().is_some_and(|p| p.name == name) {
                            play.set(None);
                            *play_cell.borrow_mut() = None;
                        }
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
                            onpointerdown={stop_on_grab}
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
    let mut surface_sliders = slider_names(&[], &surface, &session);
    for n in curve3d_slider_names(&curve3ds, &session) {
        if !surface_sliders.contains(&n) {
            surface_sliders.push(n);
        }
    }
    let solar_sliders = match (*solar_source).as_deref() {
        Some(src) => solar_slider_names(src, &session),
        None => Vec::new(),
    };
    let curve_rows = build_rows(&curve_sliders);
    let surface_rows = build_rows(&surface_sliders);
    let solar_rows = build_rows(&solar_sliders);

    // Per-curve visibility toggle (ADR-0015 amendment): the legend's
    // checkbox. Unchecking hides the curve from the plot, its points of
    // interest, and the SVG export; the checkbox itself stays so the
    // curve can be brought back.
    let on_toggle_curve = {
        let hidden = hidden.clone();
        Callback::from(move |(i, on): (usize, bool)| {
            let mut h = (*hidden).clone();
            if i < h.len() {
                h[i] = !on;
            }
            hidden.set(h);
        })
    };

    // The same gesture for 3D scene elements (ADR-0055): the surface and
    // space-curve legends' checkboxes hide one element (its mesh lines)
    // from the plot and the export.
    let toggle_hidden_index = |hidden: UseStateHandle<Vec<usize>>| {
        let hidden = hidden.clone();
        Callback::from(move |(i, on): (usize, bool)| {
            let mut h = (*hidden).clone();
            if on {
                h.retain(|x| *x != i);
            } else if !h.contains(&i) {
                h.push(i);
            }
            hidden.set(h);
        })
    };
    let on_toggle_surface = toggle_hidden_index(hidden_surfaces.clone());
    let on_toggle_curve3d = toggle_hidden_index(hidden_curves3d.clone());

    let legend_items: Vec<Html> = (*graph)
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let caption = graph::curve_caption(c);
            let checked = !(*hidden).get(i).copied().unwrap_or(false);
            let on_toggle_curve = on_toggle_curve.clone();
            // The curve class lives on the label so `--curve` (ADR-0014)
            // inherits into both the swatch line and the checkbox, which
            // then renders in the line's own colour.
            html! {
                <li class="legend-item">
                    <label class={format!("legend-check curve-{i}")}>
                        <input type="checkbox" checked={checked} aria-label={caption.clone()}
                            onchange={Callback::from(move |e: web_sys::Event| {
                                if let Some(el) = e
                                    .target()
                                    .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                {
                                    on_toggle_curve.emit((i, el.checked()));
                                }
                            })}
                        />
                        <span class="swatch" aria-hidden="true"></span>
                        { caption }
                    </label>
                </li>
            }
        })
        .collect();

    // The curves actually drawn: hidden ones stay out of the plot, the
    // points of interest, and the SVG export (ADR-0015 amendment). Each
    // curve keeps its ORIGINAL palette index — a hidden neighbour must
    // not shift the remaining lines' colours (they must always match
    // their legend entries).
    let visible_curves: Vec<(usize, epher_core::graph::SampledCurve)> = (*graph)
        .iter()
        .enumerate()
        .filter(|(i, _)| !(*hidden).get(*i).copied().unwrap_or(false))
        .map(|(i, c)| (i, c.clone()))
        .collect();
    let visible_pois: Vec<graph::Poi> = (*pois)
        .iter()
        .filter(|p| !(*hidden).get(p.curve).copied().unwrap_or(false))
        .cloned()
        .collect();

    let poi_items: Vec<Html> = (*pois)
        .iter()
        .map(|p| {
            let text = format!("{} ({}, {})", p.label, graph::label(p.x), graph::label(p.y));
            html! { <li>{ text }</li> }
        })
        .collect();

    // The 2D zoom slider's position and input handler (ADR-0038/0055):
    // -1 fits every object, +1 shows a single one; wheel and pinch move
    // past the ends and the slider pins there. The base window comes
    // from whichever kind owns the pane: the curves' geometry, or the
    // data plot's ranges (their zoom works the same way).
    let zoom2d_slider = if (*data).is_some() {
        (*data)
            .as_ref()
            .and_then(|d| graph::data_geometry(d, None))
            .map(|g| (g.x_min, g.x_max))
            .map(|base| zoom_slider_value(*view2d, base))
            .unwrap_or(0.0)
    } else {
        graph::geometry(&graph)
            .map(|g| (g.x_min, g.x_max))
            .map(|base| zoom_slider_value(*view2d, base))
            .unwrap_or(0.0)
    };
    let on_zoom2d_input = {
        let on_set_zoom2d = on_set_zoom2d.clone();
        Callback::from(move |e: web_sys::InputEvent| {
            if let Some(el) = e
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
            {
                if let Ok(v) = el.value().parse::<f64>() {
                    on_set_zoom2d.emit(v);
                }
            }
        })
    };

    // The tuning strip (ADR-0041, ADR-0055 amendment): the graph's
    // adjustment sliders - line thickness plus the view controls - as
    // one compact row above the plot in every kind, each slider named by
    // an icon. Pressing the icon resets that slider to its default (the
    // icon is a real button, so the reset is keyboard-accessible); the
    // words live in the tooltip/aria-label. The numeric readouts are
    // gone; the slider IS the control. 2D carries thickness + zoom; 3D
    // and solar carry thickness + the two rotation speeds + zoom.
    let tune_reset = {
        let localizer = localizer.clone();
        move |name_key: &str, icon: yew::Html, reset: Callback<web_sys::MouseEvent>| {
            let tip = localizer.lookup(name_key);
            let label = localizer.lookup_args("tune-reset", &[("name", &tip)]);
            html! {
                <button
                    type="button"
                    class="icon-btn tune-reset"
                    title={label.clone()}
                    aria-label={label}
                    onclick={reset}
                >
                    { icon }
                </button>
            }
        }
    };
    let tuning_2d = {
        let on_set_line_width = on_set_line_width.clone();
        let on_zoom2d_input = on_zoom2d_input.clone();
        let on_reset_width_2d = on_reset_width_2d.clone();
        let on_reset_zoom2d = on_reset_zoom2d.clone();
        let localizer = localizer.clone();
        html! {
            <>
                <span class="tune">
                    { tune_reset("tune-line-width", line_width_icon(), on_reset_width_2d) }
                    <input
                        type="range"
                        class="graph-width-slider"
                        min="0" max="4" step="0.1"
                        value={width_2d.to_string()}
                        title={localizer.lookup("tune-line-width")}
                        aria-label={localizer.lookup("tune-line-width")}
                        oninput={Callback::from(move |e: web_sys::InputEvent| {
                            if let Some(w) = e
                                .target()
                                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                .and_then(|el| el.value().parse::<f64>().ok())
                            {
                                on_set_line_width.emit(w);
                            }
                        })}
                    />
                </span>
                <span class="tune">
                    { tune_reset("tune-zoom", zoom_icon(), on_reset_zoom2d) }
                    <input
                        type="range"
                        class="view3d-slider"
                        min="-1" max="1" step="0.1"
                        value={zoom2d_slider.to_string()}
                        title={localizer.lookup("tune-zoom")}
                        aria-label={localizer.lookup("tune-zoom")}
                        oninput={on_zoom2d_input}
                    />
                </span>
            </>
        }
    };
    let tuning_3d = {
        let on_set_line_width = on_set_line_width.clone();
        let on_set_view = on_set_view.clone();
        let on_reset_width_3d = on_reset_width_3d.clone();
        let on_reset_view = on_reset_view.clone();
        let localizer = localizer.clone();
        html! {
            <>
                <span class="tune">
                    { tune_reset("tune-line-width", line_width_icon(), on_reset_width_3d) }
                    <input
                        type="range"
                        class="graph-width-slider"
                        min="0"
                        max={graph::three_d_width_range(*is_mobile).0.to_string()}
                        step={graph::three_d_width_range(*is_mobile).1.to_string()}
                        value={width_3d.to_string()}
                        title={localizer.lookup("tune-line-width")}
                        aria-label={localizer.lookup("tune-line-width")}
                        oninput={Callback::from(move |e: web_sys::InputEvent| {
                            if let Some(w) = e
                                .target()
                                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                .and_then(|el| el.value().parse::<f64>().ok())
                            {
                                on_set_line_width.emit(w);
                            }
                        })}
                    />
                </span>
                { for [("h", "tune-rot-h"), ("v", "tune-rot-v"), ("z", "tune-zoom")].iter().map(|(axis, tip)| {
                    let value = match *axis {
                        "h" => *view_h,
                        "v" => *view_v,
                        // The zoom state may sit past the slider ends (a
                        // wheel zoomed deeper); the slider pins there.
                        _ => (*view_z).clamp(-1.0, 1.0),
                    };
                    let on_input = {
                        let on_set_view = on_set_view.clone();
                        Callback::from(move |e: web_sys::InputEvent| {
                            if let Some(el) = e
                                .target()
                                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                            {
                                if let Ok(v) = el.value().parse::<f64>() {
                                    on_set_view.emit((*axis, v));
                                }
                            }
                        })
                    };
                    let reset = {
                        let on_reset_view = on_reset_view.clone();
                        let axis = *axis;
                        Callback::from(move |_: web_sys::MouseEvent| on_reset_view.emit(axis))
                    };
                    html! {
                        <span class="tune">
                            { tune_reset(tip, match *axis {
                                "h" => rot_h_icon(),
                                "v" => rot_v_icon(),
                                _ => zoom_icon(),
                            }, reset) }
                            <input
                                type="range"
                                class="view3d-slider"
                                min="-1" max="1" step="0.1"
                                value={value.to_string()}
                                title={localizer.lookup(tip)}
                                aria-label={localizer.lookup(tip)}
                                oninput={on_input}
                            />
                        </span>
                    }
                })}
            </>
        }
    };

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
        let hidden_surfaces = hidden_surfaces.clone();
        let hidden_curves3d = hidden_curves3d.clone();
        let surface = surface.clone();
        let curve3ds = curve3ds.clone();
        let solar = solar.clone();
        let solar_source = solar_source.clone();
        let surface3d_cell = surface3d_cell.clone();
        let scroll_pane = scroll_pane.clone();
        let trace = trace.clone();
        let play = play.clone();
        let play_cell = play_cell.clone();
        let live = live.clone();
        let result = result.clone();
        let localizer = localizer.clone();
        let view_h = view_h.clone();
        let view_v = view_v.clone();
        let view_h_cell = view_h_cell.clone();
        let view_v_cell = view_v_cell.clone();
        let view_z = view_z.clone();
        let spin_phase = spin_phase.clone();
        let spin_phase_cell = spin_phase_cell.clone();
        let hidden = hidden.clone();
        let view2d = view2d.clone();
        let view2d_cell = view2d_cell.clone();
        let solar_hidden = solar_hidden.clone();
        Callback::from(move |_| {
            hidden.set(Vec::new());
            hidden_surfaces.set(Vec::new());
            hidden_curves3d.set(Vec::new());
            graph.set(Vec::new());
            pois.set(Vec::new());
            surface.set(Vec::new());
            curve3ds.set(Vec::new());
            solar.set(None);
            solar_source.set(None);
            solar_hidden.set(Vec::new());
            *view2d_cell.borrow_mut() = None;
            view2d.set(None);
            *surface3d_cell.borrow_mut() = false;
            trace.set(None);
            // Mobile (ADR-0035): the pane is empty now — slide the view
            // back to the calculator.
            if mobile_layout() {
                scroll_pane.emit("calc-pane");
            }
            play.set(None);
            *play_cell.borrow_mut() = None;
            *live.borrow_mut() = GraphLive::default();
            view_h.set(0.0);
            view_v.set(0.0);
            *view_z_cell.borrow_mut() = 0.0;
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
                                        <button type="button" role="menuitem" class="menu-item"
                                            onclick={Callback::from({
                                                let menu_open = menu_open.clone();
                                                let constants_open = constants_open.clone();
                                                move |_| {
                                                    menu_open.set(None);
                                                    constants_open.set(true);
                                                }
                                            })}
                                        >
                                            { localizer.lookup("menu-constants") }
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
                                        <div class="menu-sep" role="separator"></div>
                                        <p class="menu-group" aria-hidden="true">{ localizer.lookup("menu-results") }</p>
                                        { for [true, false].map(|on| {
                                            let checked = display_prefs.exact_fractions == on;
                                            let label = if on { localizer.lookup("results-on") } else { localizer.lookup("results-off") };
                                            html! {
                                                <button type="button" role="menuitemradio" class="menu-item"
                                                    aria-checked={checked.to_string()}
                                                    onclick={Callback::from({
                                                        let menu_open = menu_open.clone();
                                                        let on_set_display = on_set_display.clone();
                                                        let display_prefs = display_prefs.clone();
                                                        move |_| {
                                                            let mut p = *display_prefs;
                                                            p.exact_fractions = on;
                                                            on_set_display.emit(p);
                                                            menu_open.set(None);
                                                        }
                                                    })}
                                                >
                                                    <span class="menu-check" aria-hidden="true">{ if checked { "\u{2713}" } else { "" } }</span>
                                                    { format!("{}: {}", localizer.lookup("results-fractions"), label) }
                                                </button>
                                            }
                                        }) }
                                        { for [Notation::Auto, Notation::Scientific, Notation::Engineering].map(|notation| {
                                            let checked = display_prefs.notation == notation;
                                            let label = match notation {
                                                Notation::Auto => localizer.lookup("results-auto"),
                                                Notation::Scientific => localizer.lookup("results-scientific"),
                                                Notation::Engineering => localizer.lookup("results-engineering"),
                                            };
                                            html! {
                                                <button type="button" role="menuitemradio" class="menu-item"
                                                    aria-checked={checked.to_string()}
                                                    onclick={Callback::from({
                                                        let menu_open = menu_open.clone();
                                                        let on_set_display = on_set_display.clone();
                                                        let display_prefs = display_prefs.clone();
                                                        move |_| {
                                                            let mut p = *display_prefs;
                                                            p.notation = notation;
                                                            on_set_display.emit(p);
                                                            menu_open.set(None);
                                                        }
                                                    })}
                                                >
                                                    <span class="menu-check" aria-hidden="true">{ if checked { "\u{2713}" } else { "" } }</span>
                                                    { label }
                                                </button>
                                            }
                                        }) }
                                        { for [true, false].map(|on| {
                                            let checked = display_prefs.separators == on;
                                            let label = if on { localizer.lookup("results-on") } else { localizer.lookup("results-off") };
                                            html! {
                                                <button type="button" role="menuitemradio" class="menu-item"
                                                    aria-checked={checked.to_string()}
                                                    onclick={Callback::from({
                                                        let menu_open = menu_open.clone();
                                                        let on_set_display = on_set_display.clone();
                                                        let display_prefs = display_prefs.clone();
                                                        move |_| {
                                                            let mut p = *display_prefs;
                                                            p.separators = on;
                                                            on_set_display.emit(p);
                                                            menu_open.set(None);
                                                        }
                                                    })}
                                                >
                                                    <span class="menu-check" aria-hidden="true">{ if checked { "\u{2713}" } else { "" } }</span>
                                                    { format!("{}: {}", localizer.lookup("results-separators"), label) }
                                                </button>
                                            }
                                        }) }
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
                    </button>                    <button
                        type="button"
                        aria-pressed={(*active_pane == "graph").to_string()}
                        aria-label={localizer.lookup("result-pane")}
                        onclick={{
                            let scroll_pane = scroll_pane.clone();
                            Callback::from(move |_| scroll_pane.emit("graph-pane"))
                        }}
                    >
                        { localizer.lookup("result-pane") }
                    </button>
                </nav>
                <button
                    type="button"
                    class="hamburger-btn"
                    aria-label={localizer.lookup("menu")}
                    aria-haspopup="menu"
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
                                <p class="menu-group" aria-hidden="true">{ localizer.lookup("menu-help") }</p>
                                <button type="button" role="menuitem" class="menu-item" onclick={mobile_item(Callback::from({
                                    let on_open_guide = on_open_guide.clone();
                                    move |e: web_sys::MouseEvent| on_open_guide.emit(e)
                                }))}>
                                    { localizer.lookup("menu-guide") }
                                </button>
                                <button type="button" role="menuitem" class="menu-item" onclick={mobile_item(Callback::from({
                                    let constants_open = constants_open.clone();
                                    move |_: web_sys::MouseEvent| constants_open.set(true)
                                }))}>
                                    { localizer.lookup("menu-constants") }
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
                                <p class="menu-group" aria-hidden="true">{ localizer.lookup("menu-results") }</p>
                                { for [true, false].map(|on| {
                                    let checked = display_prefs.exact_fractions == on;
                                    let label = if on { localizer.lookup("results-on") } else { localizer.lookup("results-off") };
                                    html! {
                                        <button type="button" role="menuitemradio" class="menu-item"
                                            aria-checked={checked.to_string()}
                                            onclick={Callback::from({
                                                let display_prefs = display_prefs.clone();
                                                let on_set_display = on_set_display.clone();
                                                let close = close_hamburger.clone();
                                                move |_| {
                                                    let mut p = *display_prefs;
                                                    p.exact_fractions = on;
                                                    on_set_display.emit(p);
                                                    close.emit(());
                                                }
                                            })}
                                        >
                                            <span class="menu-check" aria-hidden="true">{ if checked { "\u{2713}" } else { "" } }</span>
                                            { format!("{}: {}", localizer.lookup("results-fractions"), label) }
                                        </button>
                                    }
                                }) }
                                { for [Notation::Auto, Notation::Scientific, Notation::Engineering].map(|notation| {
                                    let checked = display_prefs.notation == notation;
                                    let label = match notation {
                                        Notation::Auto => localizer.lookup("results-auto"),
                                        Notation::Scientific => localizer.lookup("results-scientific"),
                                        Notation::Engineering => localizer.lookup("results-engineering"),
                                    };
                                    html! {
                                        <button type="button" role="menuitemradio" class="menu-item"
                                            aria-checked={checked.to_string()}
                                            onclick={Callback::from({
                                                let display_prefs = display_prefs.clone();
                                                let on_set_display = on_set_display.clone();
                                                let close = close_hamburger.clone();
                                                move |_| {
                                                    let mut p = *display_prefs;
                                                    p.notation = notation;
                                                    on_set_display.emit(p);
                                                    close.emit(());
                                                }
                                            })}
                                        >
                                            <span class="menu-check" aria-hidden="true">{ if checked { "\u{2713}" } else { "" } }</span>
                                            { label }
                                        </button>
                                    }
                                }) }
                                { for [true, false].map(|on| {
                                    let checked = display_prefs.separators == on;
                                    let label = if on { localizer.lookup("results-on") } else { localizer.lookup("results-off") };
                                    html! {
                                        <button type="button" role="menuitemradio" class="menu-item"
                                            aria-checked={checked.to_string()}
                                            onclick={Callback::from({
                                                let display_prefs = display_prefs.clone();
                                                let on_set_display = on_set_display.clone();
                                                let close = close_hamburger.clone();
                                                move |_| {
                                                    let mut p = *display_prefs;
                                                    p.separators = on;
                                                    on_set_display.emit(p);
                                                    close.emit(());
                                                }
                                            })}
                                        >
                                            <span class="menu-check" aria-hidden="true">{ if checked { "\u{2713}" } else { "" } }</span>
                                            { format!("{}: {}", localizer.lookup("results-separators"), label) }
                                        </button>
                                    }
                                }) }
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
                            aria-controls={if autocomplete.is_some() { "epher-autocomplete" } else { "" }}
                            aria-activedescendant={(*autocomplete)
                                .as_ref()
                                .map(|ac| format!("epher-ac-{}", ac.selected))
                                .unwrap_or_default()}
                        />
                        {
                            // The suggestion list (ADR-0042): a combobox under
                            // the entry. Focus never leaves the textarea - the
                            // arrows move the highlight, Enter/Tab accept,
                            // Escape closes, and a click accepts without
                            // stealing focus (the mousedown is suppressed for
                            // that reason). It lives inside the form so the
                            // CSS can anchor it just below the entry.
                            if let Some(ac) = (*autocomplete).clone() {
                                html! {
                                    <ul
                                        class="autocomplete"
                                        id="epher-autocomplete"
                                        role="listbox"
                                        aria-label={localizer.lookup("autocomplete-label")}
                                    >
                                        { for ac.items.iter().enumerate().map(|(i, item)| {
                                            let selected = i == ac.selected;
                                            let item_name = item.name.clone();
                                            let autocomplete_for_click = autocomplete.clone();
                                            let input_for_click = input.clone();
                                            let input_ref_for_click = input_ref.clone();
                                            let on_pick = Callback::from(move |e: MouseEvent| {
                                                e.prevent_default();
                                                if let Some(state) = (*autocomplete_for_click).clone() {
                                                    accept_suggestion(
                                                        &state,
                                                        state
                                                            .items
                                                            .iter()
                                                            .position(|s| s.name == item_name)
                                                            .unwrap_or(state.selected),
                                                        &input_for_click,
                                                        &input_ref_for_click,
                                                        &autocomplete_for_click,
                                                    );
                                                }
                                            });
                                            html! {
                                                <li
                                                    key={item.name.clone()}
                                                    id={format!("epher-ac-{i}")}
                                                    role="option"
                                                    class={if selected { "selected" } else { "" }}
                                                    aria-selected={selected.to_string()}
                                                    onmousedown={Callback::from(
                                                        |e: MouseEvent| e.prevent_default(),
                                                    )}
                                                    onclick={on_pick}
                                                >
                                                    <span class="ac-name">{ item.name.clone() }</span>
                                                    if !item.hint.is_empty() {
                                                        <span class="ac-hint">{ item.hint.clone() }</span>
                                                    }
                                                </li>
                                            }
                                        }) }
                                    </ul>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </form>
                    <div class="answer">
                        <span class="visually-hidden" id="answer-label">
                            { localizer.lookup("answer") }
                        </span>
                        <div class="answer-row">
                        {
                            // Copy the answer (ADR-0057): one copy icon
                            // just left of the answer, present only while
                            // an answer is on screen. It answers a press
                            // with a check, then returns to the copy mark.
                            if answer_fits(&result) && !result.is_empty() {
                                html! {
                                    <button
                                        type="button"
                                        class="icon-btn copy-answer"
                                        title={ if *answer_copied { localizer.lookup("answer-copied") } else { localizer.lookup("answer-copy") } }
                                        aria-label={ if *answer_copied { localizer.lookup("answer-copied") } else { localizer.lookup("answer-copy") } }
                                        onclick={on_copy_answer.clone()}
                                    >
                                        { if *answer_copied { check_icon() } else { copy_icon() } }
                                    </button>
                                }
                            } else {
                                html! {}
                            }
                        }
                        <div
                            id="epher-result"
                            class="result"
                            role="status"
                            aria-live="polite"
                            aria-labelledby="answer-label"
                            tabindex="0"
                        >
                            {
                                // The answer line keeps only what reads well
                                // on one line (ADR-0056): a short single
                                // answer. Anything longer renders in the
                                // result pane instead, one answer per line.
                                if answer_fits(&result) {
                                    html! { { for answer_items(&result) } }
                                } else {
                                    html! {}
                                }
                            }
                        </div>
                        </div>
                    </div>
                    <section class="history-box" tabindex="0" aria-label={localizer.lookup("history")} ref={history_box_ref.clone()}>
                        <div class="history-head">
                            // The trash (ADR-0041, ADR-0055 layout): the
                            // icon sits LEFT of the heading, the same
                            // command the icon's tooltip names (Ctrl+L in
                            // the terminal).
                            <button
                                type="button"
                                class="icon-btn clear-history"
                                title={localizer.lookup("clear-history")}
                                aria-label={localizer.lookup("clear-history")}
                                onclick={on_clear_history}
                            >
                                { trash_icon() }
                            </button>
                            <h2>{ localizer.lookup("history") }</h2>
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
                                        *cursor_cell.borrow_mut() =
                                    (expr.chars().count(), expr.chars().count());
                                        // ADR-0035: on touch layouts the pick
                                        // loads the line without summoning
                                        // the device keyboard — only a touch
                                        // inside the entry opens it. Desktop
                                        // keeps ADR-0016's focus return.
                                        if !mobile_layout() {
                                            if let Some(ta) =
                                                input_ref.cast::<web_sys::HtmlTextAreaElement>()
                                            {
                                                let _ = ta.focus();
                                            }
                                        }
                                    })
                                };
                                // Share (ADR-0038): the OS share sheet with
                                // an app link carrying this line's
                                // expression - the same contents a pick
                                // loads. The icon sits left of the item.
                                let on_share = on_share.clone();
                                let line = h.clone();
                                html! {
                                    <li class="history-row">
                                        <button
                                            type="button"
                                            class="icon-btn share-btn"
                                            aria-label={localizer.lookup("share-item")}
                                            title={localizer.lookup("share-item")}
                                            onclick={Callback::from(move |_| on_share.emit(line.clone()))}
                                        >
                                            { share_icon() }
                                        </button>
                                        <button type="button" class="history-item" onclick={on_pick}>{ h.clone() }</button>
                                    </li>
                                }
                            }) }
                        </ul>
                    </section>
                    // The drawer (ADR-0060): the grab bar rides the rule
                    // above the keypad; dragging it down (or a tap, or
                    // Enter — it is a real button, aria-expanded) docks
                    // the keypad away and hands its height to the
                    // history list; dragging up brings it back to this
                    // exact place. The clip animates the height; the
                    // section below is untouched.
                    <div class="keypad-drawer" data-open={(*keypad_open).to_string()} ref={keypad_drawer_ref.clone()}>
                        <button
                            type="button"
                            class="keypad-grab"
                            aria-expanded={(*keypad_open).to_string()}
                            aria-controls="keypad-panel"
                            aria-label={if *keypad_open {
                                localizer.lookup("keypad-grab-hide")
                            } else {
                                localizer.lookup("keypad-grab-show")
                            }}
                            title={if *keypad_open {
                                localizer.lookup("keypad-grab-hide")
                            } else {
                                localizer.lookup("keypad-grab-show")
                            }}
                            onpointerdown={on_grab_down}
                            onpointermove={on_grab_move}
                            onpointerup={on_grab_end.clone()}
                            onpointercancel={on_grab_end}
                            onclick={on_grab_click}
                        >
                            <span class="keypad-grab-pill" aria-hidden="true"></span>
                        </button>
                        <div class="keypad-clip">
                    <section class="keypad" aria-label={localizer.lookup("keypad")}>
                        // The hints row (ADR-0039): the tab list plus the
                        // hints toggle, a sibling OUTSIDE the tablist (a
                        // non-tab button inside role="tablist" breaks the
                        // ARIA tabs pattern).
                        <div class="keypad-top">
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
                            <button
                                type="button"
                                class="keypad-hints-btn"
                                aria-pressed={(*show_key_hints).to_string()}
                                aria-label={localizer.lookup("keypad-hints")}
                                title={localizer.lookup("keypad-hints")}
                                onclick={
                                    let show_key_hints = show_key_hints.clone();
                                    Callback::from(move |_| show_key_hints.set(!*show_key_hints))
                                }
                            >{ "?" }</button>
                        </div>
                        // The hint bar (ADR-0039): pointer hover and
                        // keyboard focus both speak here, one line, no
                        // floating tooltip to clip inside the scrolling
                        // grid. aria-hidden: screen readers announce the
                        // key's aria-label directly instead.
                        <div class="keypad-hint-bar" aria-hidden="true">
                            { if key_hint_bar.is_empty() {
                                localizer.lookup("keypad-hint-idle")
                            } else {
                                (*key_hint_bar).clone()
                            } }
                        </div>
                        <div
                            class={if *show_key_hints { "keypad-grid show-hints" } else { "keypad-grid" }}
                            role="tabpanel"
                            id="keypad-panel"
                            // The key recreates the panel per tab, so a
                            // bank change starts scrolled to the top
                            // instead of inheriting the old scroll.
                            key={(*key_tab).clone()}
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
                                    // The hint text joins the aria-label, so
                                    // screen readers hear "jd: Julian Day of
                                    // a moment" when the key takes focus;
                                    // the caption span is aria-hidden (the
                                    // label already says it).
                                    let hint = if k.hint.is_empty() {
                                        String::new()
                                    } else {
                                        localizer.lookup(k.hint)
                                    };
                                    let aria = if hint.is_empty() {
                                        k.label.to_string()
                                    } else {
                                        format!("{}: {}", k.label, hint)
                                    };
                                    // The bar callbacks are explicit per event:
                                    // hover is a MouseEvent, focus a
                                    // FocusEvent.
                                    let bar_hover = {
                                        let key_hint_bar = key_hint_bar.clone();
                                        let hint = hint.clone();
                                        let label = k.label.to_string();
                                        Callback::from(move |_: web_sys::MouseEvent| {
                                            if !hint.is_empty() {
                                                key_hint_bar.set(format!("{}: {}", label, hint));
                                            }
                                        })
                                    };
                                    let bar_focus = {
                                        let key_hint_bar = key_hint_bar.clone();
                                        let hint = hint.clone();
                                        let label = k.label.to_string();
                                        Callback::from(move |_: web_sys::FocusEvent| {
                                            if !hint.is_empty() {
                                                key_hint_bar.set(format!("{}: {}", label, hint));
                                            }
                                        })
                                    };
                                    let bar_leave = {
                                        let key_hint_bar = key_hint_bar.clone();
                                        Callback::from(
                                            move |_: web_sys::MouseEvent| key_hint_bar.set(String::new()),
                                        )
                                    };
                                    let bar_blur = {
                                        let key_hint_bar = key_hint_bar.clone();
                                        Callback::from(
                                            move |_: web_sys::FocusEvent| key_hint_bar.set(String::new()),
                                        )
                                    };
                                    let class = format!(
                                        "keypad-btn{}{}",
                                        if *show_key_hints { " show-hint" } else { "" },
                                        if k.cls.is_empty() {
                                            String::new()
                                        } else {
                                            format!(" {}", k.cls)
                                        }
                                    );
                                    html! {
                                        <button
                                            type="button"
                                            class={class}
                                            aria-label={aria}
                                            onmousedown={on_key_capture.clone()}
                                            onclick={on_key}
                                            onmouseover={bar_hover}
                                            onfocus={bar_focus}
                                            onmouseleave={bar_leave}
                                            onblur={bar_blur}
                                        >
                                            { k.label }
                                            if *show_key_hints && !hint.is_empty() {
                                                <span class="keypad-hint" aria-hidden="true">{ hint }</span>
                                            }
                                        </button>
                                    }
                                }).collect::<Vec<Html>>())
                                .unwrap_or_default() }
                        </div>
                    </section>
                        </div>
                    </div>
                </section>
                <section class="pane" id="graph-pane" aria-label={localizer.lookup("result-pane")}>
                    {
                        // The result transcript (ADR-0056): a long answer -
                        // a pasted script's transcript, a table, a long
                        // number - renders here, one answer per line, where
                        // the pane can give it room. Short single answers
                        // stay on the answer line under the entry; graphs
                        // and curves share this pane as before.
                        if !result.is_empty() && !answer_fits(&result) {
                            html! {
                                <section class="pane-result" role="status" aria-live="polite">
                                    <button
                                        type="button"
                                        class="icon-btn copy-answer"
                                        title={ if *answer_copied { localizer.lookup("answer-copied") } else { localizer.lookup("answer-copy") } }
                                        aria-label={ if *answer_copied { localizer.lookup("answer-copied") } else { localizer.lookup("answer-copy") } }
                                        onclick={on_copy_answer.clone()}
                                    >
                                        { if *answer_copied { check_icon() } else { copy_icon() } }
                                    </button>
                                    <div class="pane-answers">
                                        { for result
                                            .split(ANSWER_SEP)
                                            .filter(|p| !p.is_empty())
                                            .map(|p| html! { <div class="pane-answer">{ p }</div> }) }
                                    </div>
                                </section>
                            }
                        } else {
                            html! {}
                        }
                    }
                    {
                        // The toolbar shows for every plotting pane: 2D,
                        // 3D surfaces, 3D parametric curves (ADR-0055:
                        // the space-curve pane was missing it), and the
                        // solar system (whose sliders drive the same
                        // shared view state).
                        if !(*graph).is_empty()
                            || !(*surface).is_empty()
                            || !(*curve3ds).is_empty()
                            || (*solar).is_some()
                        {
                            html! {
                                // The pane toolbar (ADR-0023): commands and
                                // settings sit above the plot — Clear and
                                // Copy SVG as equal buttons, the graph
                                // options beside them — not scattered under
                                // it. Everything is a real labelled control.
                                <div class="graph-head">
                                    // The toolbar commands read as icons
                                    // (ADR-0040): a trash can and the copy
                                    // mark instead of two long text labels.
                                    // The names stay available to assistive
                                    // tech (aria-label) and as tooltips
                                    // (title); 44px targets keep the icon
                                    // buttons finger-friendly.
                                    <button
                                        type="button"
                                        class="icon-btn"
                                        title={localizer.lookup("graph-clear")}
                                        aria-label={localizer.lookup("graph-clear")}
                                        onclick={on_graph_clear.clone()}
                                    >
                                        { trash_icon() }
                                    </button>
                                    <button
                                        type="button"
                                        class="icon-btn"
                                        title={localizer.lookup("graph-copy")}
                                        aria-label={localizer.lookup("graph-copy")}
                                        onclick={on_copy_svg}
                                    >
                                        { copy_icon() }
                                    </button>
                                    // Save PNG (ADR-0042): the same document Copy
                                    // SVG produces, rasterized and saved through
                                    // the platform's save flow.
                                    <button
                                        type="button"
                                        class="icon-btn"
                                        title={localizer.lookup("graph-save-png")}
                                        aria-label={localizer.lookup("graph-save-png")}
                                        onclick={on_save_png.clone()}
                                    >
                                        { download_icon() }
                                    </button>
                                    // The graph options row (ADR-0020, ADR-0025):
                                    // the two points-of-interest toggles
                                    // (ADR-0019) belong to the 2D plot only, so
                                    // they render just when curves exist. Real form
                                    // controls — focusable and labelled — not menu
                                    // items, because they are adjustments, not
                                    // commands.
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
                                    </div>
                                    // The 3D fine controls (ADR-0031): three sliders above
                                    // the plot, visible while any 3D pane is displayed -
                                    // surfaces and the solar system alike (the solar pane
                                    // renders from the same shared view state, so it
                                    // inherits the same controls).
                                    // Each spans −1..1, step 0.1, default 0, and updates the
                                    // plot in real time — on top of the orbit gesture.
                                </div>
                            }
                        } else {
                            html! {}
                        }
                    }
                    {
                        if let Some(data) = (*data).as_ref() {
                            let fit_caption = data.fit.map(|f| graph::fit_legend(&f));
                            let caption = data.source.trim();
                            let extra = fit_caption.as_deref();
                            html! {
                                <section class="graph">
                                    <ul class="legend">
                                        <li>
                                            <span class="swatch" aria-hidden="true"></span>
                                            { caption }
                                            {
                                                if let Some(extra) = extra {
                                                    html! { <span class="fit-caption">{ " " }{ extra }</span> }
                                                } else {
                                                    html! {}
                                                }
                                            }
                                        </li>
                                    </ul>
                                    <div class="graph-tuning">
                                        { tuning_2d.clone() }
                                    </div>
                                    <div class="plot-box">
                                        <Graph
                                            curves={Vec::new()}
                                            data={(*data).clone()}
                                            pois={Vec::new()}
                                            trace={None}
                                            markers={false}
                                            line_width={*width_2d}
                                            window={*view2d}
                                            on_trace={on_trace.clone()}
                                            on_zoom={on_zoom2d.clone()}
                                            on_key={on_trace_key.clone()}
                                            on_leave={on_trace_leave.clone()}
                                        />
                                    </div>
                                </section>
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
                                    <div class="graph-tuning">
                                        { tuning_2d }
                                    </div>
                                    <div class="plot-box">
                                        <Graph
                                            curves={visible_curves.clone()}
                                            data={(*data).clone()}
                                            pois={visible_pois.clone()}
                                            trace={*trace}
                                            markers={*poi_markers}
                                            line_width={*width_2d}
                                            window={*view2d}
                                            on_trace={on_trace}
                                            on_zoom={on_zoom2d}
                                            on_key={on_trace_key}
                                            on_leave={on_trace_leave}
                                        />
                                    </div>
                                    <p class="trace" role="status" aria-live="polite">
                                        { trace_text }
                                    </p>
                                    <div class="sliders">
                                        { for curve_rows }
                                    </div>
                                    {
                                        if !(*pois).is_empty() && *poi_list {
                                            let on_copy_pois = on_copy_pois.clone();
                                            html! {
                                                <>
                                                    <div class="poi-head">
                                                        <button
                                                            type="button"
                                                            class="icon-btn"
                                                            aria-label={localizer.lookup("poi-copy")}
                                                            title={localizer.lookup("poi-copy")}
                                                            onclick={on_copy_pois}
                                                        >
                                                            { copy_icon() }
                                                        </button>
                                                        <p class="poi-heading">{ localizer.lookup("graph-points") }</p>
                                                    </div>
                                                    <ul class="poi-list">
                                                        { for poi_items }
                                                    </ul>
                                                </>
                                            }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </section>
                            }
                        } else {
                            html! {}
                        }
                    }
                    {
                        if let Some(scene) = (*solar).as_ref() {
                            // The solar system pane (ADR-0037): orbit
                            // curves, trails, and positioned dots through
                            // the same 3D pane, orbit and fine controls
                            // inherited (the ADR-0015 amendment). The
                            // legend's unchecked bodies (ADR-0038) stay out
                            // of the render and the aria name.
                            let shown = epher_core::astro::SolarScene {
                                jd: scene.jd,
                                orbits: scene
                                    .orbits
                                    .iter()
                                    .filter(|p| !solar_hidden.contains(&p.body))
                                    .cloned()
                                    .collect(),
                                trails: scene
                                    .trails
                                    .iter()
                                    .filter(|p| !solar_hidden.contains(&p.body))
                                    .cloned()
                                    .collect(),
                                dots: scene
                                    .dots
                                    .iter()
                                    .filter(|d| !solar_hidden.contains(&d.body))
                                    .cloned()
                                    .collect(),
                            };
                            let effective =
                                effective_view(&view, *view_h, *view_v, *view_z, *spin_phase);
                            // The frame comes from the FULL scene (ADR-0038
                            // amendment): hiding a body through the legend
                            // must never rescale or jump the view. The parts
                            // come from the filtered scene - and when every
                            // body is hidden the pane still renders (the
                            // ADR-0039 amendment): the legend stays so the
                            // bodies can be brought back.
                            let rendered = graph::solar_view_box(&scene, &effective).map(
                                |view_box| {
                                    let content = graph::solar_parts_in(
                                        &shown,
                                        &effective,
                                        *width_3d,
                                        &view_box,
                                    )
                                    .map(|(_, content)| content)
                                    .unwrap_or_default();
                                    (view_box, content)
                                },
                            );
                            let aria = format!(
                                "{}: {}",
                                localizer.lookup("solar3d-title"),
                                scene
                                    .dots
                                    .iter()
                                    .filter(|d| !solar_hidden.contains(&d.body))
                                    .map(|d| epher_core::astro::body_name(d.body))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                            // The legend (ADR-0038): one entry per body, the
                            // checkbox hiding its orbit, trail, and dot - the
                            // curve legend's gesture on the solar pane.
                            let solar_legend: Vec<Html> = scene
                                .dots
                                .iter()
                                .map(|d| {
                                    let body = d.body;
                                    let name = epher_core::astro::body_name(body);
                                    let color = epher_core::graph_svg::solar_color(body);
                                    let checked = !solar_hidden.contains(&body);
                                    let on_toggle_solar_body = on_toggle_solar_body.clone();
                                    html! {
                                        <li class="legend-item">
                                            <label
                                                class="legend-check"
                                                style={format!("--curve: {color}")}
                                            >
                                                <input
                                                    type="checkbox"
                                                    checked={checked}
                                                    aria-label={name.to_string()}
                                                    onchange={Callback::from(
                                                        move |e: web_sys::Event| {
                                                            if let Some(el) = e
                                                                .target()
                                                                .and_then(|t| {
                                                                    t.dyn_into::<
                                                                        web_sys::HtmlInputElement,
                                                                    >()
                                                                    .ok()
                                                                })
                                                            {
                                                                on_toggle_solar_body
                                                                    .emit((body, el.checked()));
                                                            }
                                                        },
                                                    )}
                                                />
                                                <span class="swatch" aria-hidden="true"></span>
                                                { name }
                                            </label>
                                        </li>
                                    }
                                })
                                .collect();
                            if let Some((view_box, content)) = rendered {
                                *rendered_box.borrow_mut() = Some(view_box.clone());
                                let shown_box = (*play)
                                    .as_ref()
                                    .and_then(|p| p.freeze.clone())
                                    .unwrap_or(view_box);
                                html! {
                                    <section class="graph graph3d">
                                        <h2 class="graph3d-title">{ localizer.lookup("solar3d-title") }</h2>
                                        <ul class="legend legend-solar">
                                            { for solar_legend }
                                        </ul>
                                        <div class="graph-tuning">
                                            { tuning_3d.clone() }
                                        </div>
                                        <div class="plot-box">
                                            <Graph3D
                                                view_box={shown_box}
                                                content={content}
                                                aria_label={aria}
                                                stroke_px={graph::THREE_D_PX_PER_WIDTH * *width_3d}
                                                on_orbit={on_orbit}
                                                on_zoom={on_zoom3d.clone()}
                                            />
                                        </div>
                                        <p class="graph3d-hint">{ localizer.lookup("graph3d-hint") }</p>
                                        <div class="sliders">
                                            { for solar_rows }
                                        </div>
                                    </section>
                                }
                            } else {
                                html! {}
                            }
                        } else if !(*curve3ds).is_empty() {
                            // 3D parametric curves (ADR-0054): the same
                            // pose, sliders, controls, and legend as the
                            // surface pane (ADR-0055): every curve gets a
                            // checkbox, and hidden curves stay out of the
                            // plot and the export while keeping their
                            // palette index.
                            let effective =
                                effective_view(&view, *view_h, *view_v, *view_z, *spin_phase);
                            let visible3d: Vec<(usize, epher_core::graph::SpaceCurve)> =
                                (*curve3ds)
                                    .iter()
                                    .enumerate()
                                    .filter(|(i, _)| !(*hidden_curves3d).contains(i))
                                    .map(|(i, c)| (i, c.clone()))
                                    .collect();
                            let rendered =
                                graph::scene_parts_indexed(&[], &visible3d, &effective, *width_3d);
                            let curve3d_legend: Vec<Html> = (*curve3ds)
                                .iter()
                                .enumerate()
                                .map(|(i, c)| {
                                    let caption = format!("param {}", c.source.trim());
                                    let checked = !(*hidden_curves3d).contains(&i);
                                    let on_toggle_curve3d = on_toggle_curve3d.clone();
                                    html! {
                                        <li class="legend-item">
                                            <label class={format!("legend-check curve-{i}")}>
                                                <input type="checkbox" checked={checked} aria-label={caption.clone()}
                                                    onchange={Callback::from(move |e: web_sys::Event| {
                                                        if let Some(el) = e
                                                            .target()
                                                            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                                        {
                                                            on_toggle_curve3d.emit((i, el.checked()));
                                                        }
                                                    })}
                                                />
                                                <span class="swatch" aria-hidden="true"></span>
                                                { caption }
                                            </label>
                                        </li>
                                    }
                                })
                                .collect();
                            let aria = format!(
                                "{}: {}",
                                "3D",
                                (*curve3ds)
                                    .iter()
                                    .map(|c| format!("param {}", c.source.trim()))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                            html! {
                                <section class="graph graph3d">
                                    <ul class="legend">
                                        { for curve3d_legend }
                                    </ul>
                                    <div class="graph-tuning">
                                        { tuning_3d.clone() }
                                    </div>
                                    {
                                        if let Some((view_box, content)) = rendered {
                                            // Record for play-freeze; while playing,
                                            // keep the frozen box so the layout stays
                                            // put.
                                            *rendered_box.borrow_mut() = Some(view_box.clone());
                                            let shown_box = (*play)
                                                .as_ref()
                                                .and_then(|p| p.freeze.clone())
                                                .unwrap_or(view_box);
                                            html! {
                                                <>
                                                    <div class="plot-box">
                                                        <Graph3D
                                                            view_box={shown_box}
                                                            content={content}
                                                            aria_label={aria}
                                                stroke_px={graph::THREE_D_PX_PER_WIDTH * *width_3d}
                                                            on_orbit={on_orbit}
                                                            on_zoom={on_zoom3d.clone()}
                                                        />
                                                    </div>
                                                    <p class="graph3d-hint">{ localizer.lookup("graph3d-hint") }</p>
                                                    <div class="sliders">
                                                        { for surface_rows }
                                                    </div>
                                                </>
                                            }
                                        } else {
                                            html! { <div class="plot-box"></div> }
                                        }
                                    }
                                </section>
                            }
                        } else if !(*surface).is_empty() {
                            // The fine-control sliders ride on the orbit
                            // base (ADR-0031); the rotation sliders spin
                            // the pose (ADR-0032). The pane renders the
                            // effective pose. The legend (ADR-0055) is the
                            // 2D curve legend's gesture: one checkbox per
                            // surface, hiding its mesh from the plot and
                            // the export.
                            let effective =
                                effective_view(&view, *view_h, *view_v, *view_z, *spin_phase);
                            let visible3d: Vec<(usize, epher_core::graph::Surface)> =
                                (*surface)
                                    .iter()
                                    .enumerate()
                                    .filter(|(i, _)| !(*hidden_surfaces).contains(i))
                                    .map(|(i, s)| (i, s.clone()))
                                    .collect();
                            let rendered =
                                graph::scene_parts_indexed(&visible3d, &[], &effective, *width_3d);
                            let surface_legend: Vec<Html> = (*surface)
                                .iter()
                                .enumerate()
                                .map(|(i, s)| {
                                    let caption = format!("z = {}", s.source.trim());
                                    let checked = !(*hidden_surfaces).contains(&i);
                                    let on_toggle_surface = on_toggle_surface.clone();
                                    html! {
                                        <li class="legend-item">
                                            <label class={format!("legend-check curve-{i}")}>
                                                <input type="checkbox" checked={checked} aria-label={caption.clone()}
                                                    onchange={Callback::from(move |e: web_sys::Event| {
                                                        if let Some(el) = e
                                                            .target()
                                                            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                                        {
                                                            on_toggle_surface.emit((i, el.checked()));
                                                        }
                                                    })}
                                                />
                                                <span class="swatch" aria-hidden="true"></span>
                                                { caption }
                                            </label>
                                        </li>
                                    }
                                })
                                .collect();
                            let aria = format!(
                                "{}: {}",
                                "3D",
                                (*surface)
                                    .iter()
                                    .map(|s| format!("z = {}", s.source.trim()))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            );
                            html! {
                                <section class="graph graph3d">
                                    <ul class="legend">
                                        { for surface_legend }
                                    </ul>
                                    <div class="graph-tuning">
                                        { tuning_3d.clone() }
                                    </div>
                                    {
                                        if let Some((view_box, content)) = rendered {
                                            // Record for play-freeze; while playing,
                                            // keep the frozen box so the layout stays
                                            // put.
                                            *rendered_box.borrow_mut() = Some(view_box.clone());
                                            let shown_box = (*play)
                                                .as_ref()
                                                .and_then(|p| p.freeze.clone())
                                                .unwrap_or(view_box);
                                            html! {
                                                <>
                                                    <div class="plot-box">
                                                        <Graph3D
                                                            view_box={shown_box}
                                                            content={content}
                                                            aria_label={aria}
                                                stroke_px={graph::THREE_D_PX_PER_WIDTH * *width_3d}
                                                            on_orbit={on_orbit}
                                                            on_zoom={on_zoom3d.clone()}
                                                        />
                                                    </div>
                                                    <p class="graph3d-hint">{ localizer.lookup("graph3d-hint") }</p>
                                                    <div class="sliders">
                                                        { for surface_rows }
                                                    </div>
                                                </>
                                            }
                                        } else {
                                            html! { <div class="plot-box"></div> }
                                        }
                                    }
                                </section>
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
                            <div class="guide-search">
                                <input
                                    type="search"
                                    aria-label={localizer.lookup("guide-search")}
                                    placeholder={localizer.lookup("guide-search-placeholder")}
                                    value={(*guide_query).clone()}
                                    oninput={Callback::from({
                                        let guide_query = guide_query.clone();
                                        let guide_hits = guide_hits.clone();
                                        move |e: web_sys::InputEvent| {
                                            let q = e
                                                .target()
                                                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                                .map(|el| el.value())
                                                .unwrap_or_default();
                                            guide_hits.set(guide_search(&q));
                                            guide_query.set(q);
                                        }
                                    })}
                                    onkeydown={Callback::from({
                                        let guide_query = guide_query.clone();
                                        let guide_hits = guide_hits.clone();
                                        move |e: web_sys::KeyboardEvent| {
                                            match e.key().as_str() {
                                                // Enter jumps to the first hit.
                                                "Enter" => {
                                                    if let Some(hit) = guide_hits.first() {
                                                        guide_jump_to(hit.index);
                                                    }
                                                    e.prevent_default();
                                                }
                                                "Escape" => {
                                                    guide_query.set(String::new());
                                                    guide_hits.set(Vec::new());
                                                }
                                                _ => {}
                                            }
                                        }
                                    })}
                                />
                                {
                                    if !(*guide_query).trim().is_empty() {
                                        if guide_hits.is_empty() {
                                            html! { <p class="guide-no-results" role="status">{ localizer.lookup("guide-no-results") }</p> }
                                        } else {
                                            html! {
                                                <ul class="guide-results" role="listbox" aria-label={localizer.lookup("guide-search")}>
                                                    { for guide_hits.iter().map(|hit| {
                                                        let index = hit.index;
                                                        let chapter = if hit.chapter.is_empty() { hit.snippet.clone() } else { hit.chapter.clone() };
                                                        html! {
                                                            <li>
                                                                <button
                                                                    type="button"
                                                                    role="option"
                                                                    aria-selected="false"
                                                                    class="guide-result"
                                                                    onclick={Callback::from(move |_| {
                                                                        guide_jump_to(index);
                                                                    })}
                                                                >
                                                                    <span class="guide-result-chapter">{ chapter }</span>
                                                                    <span class="guide-result-snippet">{ hit.snippet.clone() }</span>
                                                                </button>
                                                            </li>
                                                        }
                                                    }) }
                                                </ul>
                                            }
                                        }
                                    } else {
                                        html! {}
                                    }
                                }
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
                                    // calculator (ADR-0018). Clicking a
                                    // table-of-contents item scrolls the
                                    // guide body to its chapter (ADR-0018
                                    // amendment).
                                    if let Some(target) =
                                        e.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                                    {
                                        if let Some(btn) =
                                            target.closest(".guide-toc-btn").ok().flatten()
                                        {
                                            if let Some(n) = btn.get_attribute("data-jump") {
                                                if let Some(el) = web_sys::window()
                                                    .and_then(|w| w.document())
                                                    .and_then(|d| {
                                                        d.query_selector(&format!(
                                                            ".guide-body #guide-ch-{}",
                                                            n
                                                        ))
                                                        .ok()
                                                        .flatten()
                                                    })
                                                {
                                                    let _ = el.scroll_into_view();
                                                }
                                            }
                                        }
                                        if let Some(btn) =
                                            target.closest(".guide-example-btn").ok().flatten()
                                        {
                                            if let Some(code) = btn.get_attribute("data-code") {
                                                input.set(code.clone());
                                                // The load puts the cursor at the
                                                // end of the code (ADR-0035).
                                                *cursor_cell.borrow_mut() =
                                                    (code.chars().count(), code.chars().count());
                                                guide_open.set(false);
                                                scroll_pane.emit("calc-pane");
                                                // ADR-0035: on touch layouts the
                                                // load does not summon the device
                                                // keyboard — only a touch inside
                                                // the entry opens it. Desktop
                                                // keeps ADR-0016's focus return.
                                                if !mobile_layout() {
                                                    if let Some(ta) =
                                                        input_ref.cast::<web_sys::HtmlTextAreaElement>()
                                                    {
                                                        let _ = ta.focus();
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            })}>
                                {
                                    guide_body(&localizer, &guide_cache.borrow())
                                }
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
            {
                if *constants_open {
                    let q = (*constants_query).trim().to_lowercase();
                    let group_key = |g: epher_core::ConstGroup| match g {
                        epher_core::ConstGroup::Math => "constants-group-math",
                        epher_core::ConstGroup::Astronomy => "constants-group-astronomy",
                        epher_core::ConstGroup::Physics => "constants-group-physics",
                        epher_core::ConstGroup::Chemistry => "constants-group-chemistry",
                    };
                    // The catalog is name-sorted, so cluster by group
                    // explicitly: four headed sections in a fixed order.
                    let mut groups: Vec<(String, Vec<(&'static str, Option<f64>)>)> = Vec::new();
                    for g in [
                        epher_core::ConstGroup::Math,
                        epher_core::ConstGroup::Astronomy,
                        epher_core::ConstGroup::Physics,
                        epher_core::ConstGroup::Chemistry,
                    ] {
                        let key = group_key(g).to_string();
                        let rows: Vec<_> = epher_core::builtin_constant_groups()
                            .iter()
                            .filter(|(_, group)| *group == g)
                            .filter(|(name, _)| q.is_empty() || name.to_lowercase().contains(&q))
                            .map(|(name, _)| (*name, epher_core::builtin_constant_value(name)))
                            .collect();
                        if !rows.is_empty() {
                            groups.push((key, rows));
                        }
                    }
                    html! {
                        <div
                            class="guide-overlay"
                            role="dialog"
                            aria-modal="true"
                            aria-label={localizer.lookup("menu-constants")}
                            onkeydown={{
                                let constants_open = constants_open.clone();
                                Callback::from(move |e: web_sys::KeyboardEvent| {
                                    if e.key() == "Escape" {
                                        constants_open.set(false);
                                    }
                                })
                            }}
                        >
                            <div class="guide-head">
                                <h2>{ localizer.lookup("menu-constants") }</h2>
                                <button type="button" class="guide-close-btn" ref={constants_close_ref.clone()} onclick={on_close_constants.clone()}>
                                    { localizer.lookup("guide-close") }
                                </button>
                            </div>
                            <p class="guide-insert-hint">{ localizer.lookup("constants-insert-hint") }</p>
                            <div class="guide-search">
                                <input
                                    type="search"
                                    aria-label={localizer.lookup("constants-search")}
                                    placeholder={localizer.lookup("constants-search")}
                                    value={(*constants_query).clone()}
                                    oninput={Callback::from({
                                        let constants_query = constants_query.clone();
                                        move |e: web_sys::InputEvent| {
                                            let v = e
                                                .target()
                                                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
                                                .map(|el| el.value())
                                                .unwrap_or_default();
                                            constants_query.set(v);
                                        }
                                    })}
                                />
                            </div>
                            <div class="constants-list" role="list">
                                {
                                    if groups.is_empty() {
                                        html! { <p class="guide-no-results" role="status">{ localizer.lookup("constants-no-results") }</p> }
                                    } else {
                                        html! {
                                            <>
                                                { for groups.into_iter().map(|(key, rows)| {
                                                    let heading = localizer.lookup(&key);
                                                    html! {
                                                        <>
                                                            <h3 class="constants-group">{ heading }</h3>
                                                            { for rows.into_iter().map(|(name, value)| {
                                                                let hint_key = format!("key-hint-{name}");
                                                                let hint = {
                                                                    let h = localizer.lookup(&hint_key);
                                                                    if h == hint_key { String::new() } else { h }
                                                                };
                                                                let shown = match value {
                                                                    Some(v) => epher_core::format_value(&Value::float(v), &session.display()),
                                                                    None => String::new(),
                                                                };
                                                                let name_owned = name.to_string();
                                                                let label = if shown.is_empty() { name_owned.clone() } else { format!("{name}, {shown}") };
                                                                html! {
                                                                    <button
                                                                        type="button"
                                                                        class="constants-row"
                                                                        role="listitem"
                                                                        aria-label={label}
                                                                        onclick={{
                                                                            let insert_constant = insert_constant.clone();
                                                                            let name = name_owned.clone();
                                                                            move |_: web_sys::MouseEvent| insert_constant.emit(name.clone())
                                                                        }}
                                                                    >
                                                                        <span class="constants-name">{ name_owned }</span>
                                                                        {
                                                                            if shown.is_empty() {
                                                                                html! {}
                                                                            } else {
                                                                                html! { <span class="constants-value">{ shown }</span> }
                                                                            }
                                                                        }
                                                                        {
                                                                            if hint.is_empty() {
                                                                                html! {}
                                                                            } else {
                                                                                html! { <span class="constants-hint">{ hint }</span> }
                                                                            }
                                                                        }
                                                                    </button>
                                                                }
                                                            }) }
                                                        </>
                                                    }
                                                }) }
                                            </>
                                        }
                                    }
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
    console_error_panic_hook::set_once();
    yew::Renderer::<EpherApp>::new().render();
}

#[cfg(test)]
mod tests {
    use super::{keypad_snap, slider_span, TABS};

    /// The snap contract (ADR-0060): a downward flick collapses, an
    /// upward flick opens, and a slow release keeps the keypad wherever
    /// the majority of it already is.
    #[test]
    fn keypad_snap_flicks_beat_distance() {
        // Mostly open, but flicked down fast: away.
        assert!(keypad_snap(240.0, 260.0, 0.8));
        // Mostly closed, but flicked up fast: back.
        assert!(!keypad_snap(20.0, 260.0, -0.8));
        // The exact flick threshold (0.5 px/ms) belongs to the flick.
        assert!(keypad_snap(240.0, 260.0, 0.5));
        assert!(!keypad_snap(20.0, 260.0, -0.5));
    }

    #[test]
    fn keypad_snap_slow_release_follows_the_majority() {
        let open_h = 260.0;
        // Slower than half stays open.
        assert!(!keypad_snap(open_h * 0.5, open_h, 0.0));
        assert!(!keypad_snap(open_h - 1.0, open_h, 0.0));
        // Past half (any slow speed) goes away.
        assert!(keypad_snap(open_h * 0.5 - 0.5, open_h, 0.1));
        assert!(keypad_snap(0.0, open_h, 0.0));
        // Exactly half stays: the tie belongs to the keypad.
        assert!(!keypad_snap(open_h * 0.5, open_h, -0.1));
    }

    #[test]
    fn small_values_keep_the_base_window() {
        // The ADR-0014 default: a constant near zero plays across −10..10.
        assert_eq!(slider_span(0.0), (-10.0, 10.0));
        assert_eq!(slider_span(2.5), (-10.0, 10.0));
        assert_eq!(slider_span(-7.0), (-10.0, 10.0));
        assert_eq!(slider_span(8.0), (-10.0, 10.0));
    }

    /// The digits tab is frozen (ADR-0042 amendment): it is exactly full
    /// (24 keys, where = spans two cells of the five-row, five-column
    /// grid), so any addition scrolls the bank. Changes need the project
    /// owner's explicit approval; this test holds the line.
    #[test]
    fn the_digits_tab_is_frozen_and_full() {
        let digits = &TABS[0];
        assert_eq!(digits.id, "digits");
        assert_eq!(digits.keys.len(), 24);
        assert_eq!(digits.keys[23].label, "=");
    }

    /// Every tab except astronomy must fit the fixed five-row keypad
    /// (ADR-0039): five columns, and a spanning = (class eq) counts as
    /// two cells. An overflowing tab scrolls; the long astronomy bank is
    /// the one tab allowed to, everything else must fit like 123 does.
    #[test]
    fn every_tab_fits_the_five_row_grid() {
        for tab in TABS {
            let spans: usize = tab
                .keys
                .iter()
                .map(|k| if k.cls == "eq" { 2 } else { 1 })
                .sum();
            if tab.id == "astro" {
                continue; // ADR-0039: the astronomy bank scrolls by design
            }
            assert!(
                spans <= 25,
                "tab {} needs {spans} cells, the grid holds 25",
                tab.id
            );
        }
    }

    #[test]
    fn large_values_get_a_tight_window() {
        // A Julian Date (or any large-magnitude constant) gets a v±2
        // window: draggable, and play's 0.1 step loops in ≈ 5 s —
        // not a multi-million-wide slider that wraps v to −10.
        let (lo, hi) = slider_span(2_461_282.762);
        assert!((lo - 2_461_280.762).abs() < 1e-9);
        assert!((hi - 2_461_284.762).abs() < 1e-9);
        let (lo, hi) = slider_span(-1e6);
        assert_eq!(lo, -1e6 - 2.0);
        assert_eq!(hi, -1e6 + 2.0);
    }

    #[test]
    fn the_window_always_contains_the_value() {
        for v in [-100.0, -9.0, 8.5, 42.0, 5e5] {
            let (lo, hi) = slider_span(v);
            assert!(lo <= v && v <= hi, "span {lo}..{hi} misses {v}");
        }
    }

    /// Every keypad hint key (ADR-0039) must resolve in every locale -
    /// the hint is an accessibility surface, a missing translation must
    /// fail here rather than fall back to the key name on screen.
    #[test]
    fn keypad_hints_resolve_in_every_locale() {
        for locale in epher_i18n::SUPPORTED_LOCALES {
            let l = epher_i18n::Localizer::resolve(Some(locale), &[]);
            for tab in TABS {
                for k in tab.keys {
                    if k.hint.is_empty() {
                        // only the self-evident digits are bare
                        assert!(
                            k.label.chars().all(|c| c.is_ascii_digit()) || k.label == ".",
                            "key {} needs a hint",
                            k.label
                        );
                    } else {
                        assert_ne!(l.lookup(k.hint), k.hint, "hint {} missing", k.hint);
                    }
                }
            }
        }
    }
}
