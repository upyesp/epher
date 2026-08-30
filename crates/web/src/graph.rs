//! The web graph renderer (ADR-0006, ADR-0014): the core's sampler and
//! analyzer provide the data; this module turns multiple curves, points of
//! interest, and the trace cursor into SVG. Pure math in [`geometry`],
//! [`segments`], [`ticks`], and [`trace_nearest`] (unit-tested natively),
//! then two renderers over it: [`graph_svg`] (string form — tests and the
//! copy-to-clipboard button) and [`graph_html`] (Yew VNodes — the
//! production renderer, so the SVG lands in the proper namespace;
//! innerHTML-parsed `<svg>` is HTML-namespaced and invisible in WebKit).
//!
//! Accessibility (ADR-0009): the SVG is `role="img"` with a `title` and an
//! `aria-label` naming every plotted expression; the visible caption and
//! legend live next to it. Curve colors are CSS classes (`curve-0` …
//! `curve-3`, contrast-verified in `index.html`); curves are all solid
//! (ADR-0023) and stay distinguishable without color through the legend
//! and captions (WCAG 1.4.1). Axes/gridlines inherit `currentColor` at
//! recorded opacities (1.4.11).

use epher_core::graph::{SampledCurve, Surface, View3D};
use wasm_bindgen::JsCast;
use yew::prelude::*;

/// Everything pure lives in `epher_core::graph_svg` (ADR-0020): the
/// terminal frontends and the copy button render through the same code,
/// so an SVG saved from the TUI is byte-for-byte the app's plot. The
/// re-exports keep this module's long-standing surface.
pub use epher_core::graph_svg::{
    aria_label, curve_caption, escape, fill_points, geometry, geometry_in, graph3d_svg, graph_svg,
    graph_svg_indexed, label, layers_svg, polyline_points, segments, solar_parts_in,
    solar_view_box, ticks, trace_nearest, Geometry, Poi, TracePoint, BOTTOM,
    DEFAULT_STROKE_WIDTH, HEIGHT, LEFT, RIGHT, TOP, WIDTH,
};
/// The live 3D renderer's content (view box + mesh markup).
pub fn surface_svg(
    surfaces: &[Surface],
    view: &View3D,
    stroke_width: f64,
) -> Option<(String, String)> {
    epher_core::graph_svg::surface_parts(surfaces, view, stroke_width)
}

/// The solar system scene as (view box, part markup) — the same live
/// contract as [`surface_svg`], fed to the same `Graph3D` component.
pub fn solar_svg(
    scene: &epher_core::astro::SolarScene,
    view: &View3D,
    stroke_width: f64,
) -> Option<(String, String)> {
    epher_core::graph_svg::solar_parts(scene, view, stroke_width)
}

/// The solar system scene as a standalone SVG document — the clipboard
/// and export path.
pub fn solar3d_doc(
    scene: &epher_core::astro::SolarScene,
    view: &View3D,
    stroke_width: f64,
) -> Option<String> {
    epher_core::graph_svg::solar3d_svg(scene, view, stroke_width)
}

/// Render the same layers as Yew SVG VNodes — the production renderer.
/// Yew creates SVG elements in the SVG namespace, so the plot actually
/// paints in every engine (innerHTML-parsed SVG does not, in WebKit).
/// Pointer/keyboard interaction uses native listeners (gloo-events) bound
/// to the element through a NodeRef — Yew's synthetic event delegation
/// does not reach SVG children.
/// Map an element-local pixel position to viewBox coordinates, accounting
/// for the letterbox bands of `preserveAspectRatio="xMidYMid meet"` — the
/// SVG now fits a fixed-size pane instead of owning its aspect ratio
/// (ADR-0016), so edge pixels lie outside the plotted area.
fn to_viewbox(el: &web_sys::Element, offset_x: f64, offset_y: f64) -> (f64, f64) {
    let w = el.client_width().max(1) as f64;
    let h = el.client_height().max(1) as f64;
    let content_w = w.min(h * WIDTH / HEIGHT);
    let content_h = content_w * HEIGHT / WIDTH;
    let ox = (w - content_w) / 2.0;
    let oy = (h - content_h) / 2.0;
    let px = (offset_x - ox).max(0.0).min(content_w) * WIDTH / content_w.max(1.0);
    let py = (offset_y - oy).max(0.0).min(content_h) * HEIGHT / content_h.max(1.0);
    (px, py)
}

#[derive(Properties, PartialEq)]
pub struct GraphProps {
    /// The plotted curves with their original palette indices: the pane
    /// filters hidden curves out but each keeps its own colour class
    /// (ADR-0015 amendment), so the drawn lines always match the legend.
    pub curves: Vec<(usize, SampledCurve)>,
    pub pois: Vec<Poi>,
    pub trace: Option<TracePoint>,
    /// Settings → Graph (ADR-0019): draw the highlighted points on the
    /// plot itself. The list below the plot is a separate toggle.
    pub markers: bool,
    /// Curve line width in viewBox units (ADR-0020): the slider at the
    /// bottom of the graph pane sets it; the CSS default equals
    /// [`DEFAULT_STROKE_WIDTH`].
    pub line_width: f64,
    /// Mouse move/tap over the plot: viewBox coordinates.
    pub on_trace: Callback<(f64, f64)>,
    /// Keyboard input while the plot has focus (arrow-key tracing).
    pub on_key: Callback<web_sys::KeyboardEvent>,
    /// Wheel notch or pinch over the plot (ADR-0038): the anchor in
    /// viewBox coordinates and the window-scale factor - `> 1` widens the
    /// window (zoom out), `< 1` narrows it (zoom in). Wheel anchors at
    /// the pointer, a pinch at the midpoint between the fingers.
    pub on_zoom: Callback<(f64, f64, f64)>,
    /// The pane's zoom window in data coordinates (ADR-0038): `None` is
    /// the auto-fit around the samples, `Some((x_min, x_max))` the window
    /// wheel, pinch, or the zoom slider set. The y range stays data-fit.
    pub window: Option<(f64, f64)>,
    /// End of pointer interaction: hide the trace cursor.
    pub on_leave: Callback<()>,
}

#[function_component(Graph)]
pub fn graph_html(props: &GraphProps) -> Html {
    let svg_ref = use_node_ref();
    // The active pointers, for pinch-zoom (ADR-0038): two fingers on the
    // plot zoom around their midpoint; one finger keeps tracing.
    let pointers = use_state(|| std::rc::Rc::new(std::cell::RefCell::new(
        Vec::<(i32, f64, f64)>::new(),
    )));

    // Attach the interaction listeners once, directly on the SVG element.
    {
        let svg_ref = svg_ref.clone();
        let on_trace = props.on_trace.clone();
        let on_key = props.on_key.clone();
        let on_leave = props.on_leave.clone();
        let on_zoom = props.on_zoom.clone();
        let pointers = pointers.clone();
        let listeners = use_state(Vec::<gloo_events::EventListener>::new);
        use_effect_with((), move |_| {
            let Some(el) = svg_ref.cast::<web_sys::Element>() else {
                return;
            };
            let mut bound = Vec::new();
            {
                let el_closure = el.clone();
                let on_trace = on_trace.clone();
                bound.push(gloo_events::EventListener::new(
                    &el,
                    "pointermove",
                    move |e| {
                        let Some(me) = e.dyn_ref::<web_sys::PointerEvent>() else {
                            return;
                        };
                        // offsetX/Y are relative to the event TARGET — a
                        // path or axis line when the pointer is over a
                        // curve, not the SVG. Use clientX/Y minus the
                        // SVG's rect: element-local regardless of target.
                        let r = el_closure.get_bounding_client_rect();
                        let (px, py) = to_viewbox(
                            &el_closure,
                            me.client_x() as f64 - r.left(),
                            me.client_y() as f64 - r.top(),
                        );
                        on_trace.emit((px, py));
                    },
                ));
            }
            {
                let el_closure = el.clone();
                let on_trace = on_trace.clone();
                bound.push(gloo_events::EventListener::new(&el, "click", move |e| {
                    let Some(me) = e.dyn_ref::<web_sys::MouseEvent>() else {
                        return;
                    };
                    let r = el_closure.get_bounding_client_rect();
                    let (px, py) = to_viewbox(
                        &el_closure,
                        me.client_x() as f64 - r.left(),
                        me.client_y() as f64 - r.top(),
                    );
                    on_trace.emit((px, py));
                }));
            }
            {
                let el = el.clone();
                let on_key = on_key.clone();
                bound.push(gloo_events::EventListener::new(&el, "keydown", move |e| {
                    if let Some(ke) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                        on_key.emit(ke.clone());
                    }
                }));
            }
            {
                let el_closure = el.clone();
                let on_zoom = on_zoom.clone();
                bound.push(gloo_events::EventListener::new(&el, "wheel", move |e| {
                    let Some(we) = e.dyn_ref::<web_sys::WheelEvent>() else {
                        return;
                    };
                    // The page must not scroll while the plot zooms.
                    we.prevent_default();
                    let r = el_closure.get_bounding_client_rect();
                    let (px, py) = to_viewbox(
                        &el_closure,
                        we.client_x() as f64 - r.left(),
                        we.client_y() as f64 - r.top(),
                    );
                    // One notch = 1.15× the window; scrolling up zooms in.
                    let factor = (we.delta_y() / 300.0).exp().clamp(0.5, 2.0);
                    on_zoom.emit((px, py, factor));
                }));
            }
            {
                let el_closure = el.clone();
                let pointers = pointers.clone();
                bound.push(gloo_events::EventListener::new(&el, "pointerdown", move |e| {
                    let Some(pe) = e.dyn_ref::<web_sys::PointerEvent>() else {
                        return;
                    };
                    let mut pts = pointers.borrow_mut();
                    pts.retain(|(id, _, _)| *id != pe.pointer_id());
                    pts.push((pe.pointer_id(), pe.client_x() as f64, pe.client_y() as f64));
                    drop(pts);
                    let _ = el_closure.set_pointer_capture(pe.pointer_id());
                }));
            }
            {
                let el_closure = el.clone();
                let on_zoom = on_zoom.clone();
                let pointers = pointers.clone();
                bound.push(gloo_events::EventListener::new(&el, "pointermove", move |e| {
                    let Some(pe) = e.dyn_ref::<web_sys::PointerEvent>() else {
                        return;
                    };
                    let mut pts = pointers.borrow_mut();
                    if let Some(slot) = pts.iter_mut().find(|(id, _, _)| *id == pe.pointer_id()) {
                        slot.1 = pe.client_x() as f64;
                        slot.2 = pe.client_y() as f64;
                    }
                    if pts.len() < 2 {
                        return;
                    }
                    // Two fingers: the span scales by the distance ratio
                    // between moves, anchored at the fingers' midpoint.
                    let dx = pts[1].1 - pts[0].1;
                    let dy = pts[1].2 - pts[0].2;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let mid_x = (pts[0].1 + pts[1].1) / 2.0;
                    let mid_y = (pts[0].2 + pts[1].2) / 2.0;
                    drop(pts);
                    let last = el_closure
                        .get_attribute("data-pinch-dist")
                        .and_then(|v| v.parse::<f64>().ok());
                    let _ = el_closure
                        .set_attribute("data-pinch-dist", &format!("{dist}"));
                    let Some(last) = last else { return };
                    if last < 1.0 || dist < 1.0 {
                        return;
                    }
                    let r = el_closure.get_bounding_client_rect();
                    let (px, py) = to_viewbox(
                        &el_closure,
                        mid_x - r.left(),
                        mid_y - r.top(),
                    );
                    on_zoom.emit((px, py, last / dist));
                }));
            }
            {
                let el = el.clone();
                let pointers = pointers.clone();
                for event_name in ["pointerup", "pointercancel", "pointerleave"] {
                    let el_inner = el.clone();
                    let pointers = pointers.clone();
                    bound.push(gloo_events::EventListener::new(&el, event_name, move |e| {
                        if let Some(pe) = e.dyn_ref::<web_sys::PointerEvent>() {
                            pointers.borrow_mut().retain(|(id, _, _)| *id != pe.pointer_id());
                        }
                        if pointers.borrow().len() < 2 {
                            let _ = el_inner.remove_attribute("data-pinch-dist");
                        }
                    }));
                }
            }
            {
                let el = el.clone();
                let on_leave = on_leave.clone();
                bound.push(gloo_events::EventListener::new(&el, "blur", move |_| {
                    on_leave.emit(());
                }));
            }
            listeners.set(bound);
        });
    }
    let all: Vec<epher_core::graph::SampledCurve> =
        props.curves.iter().map(|(_, c)| c.clone()).collect();
    // The zoom window (ADR-0038) picks the x range; the y range still
    // fits the samples inside it. No window: the classic auto-fit.
    let geom = match props.window {
        Some((lo, hi)) => geometry_in(&all, lo, hi),
        None => geometry(&all),
    };
    let Some(geom) = geom else {
        return html! {};
    };
    let y_span = geom.y_max - geom.y_min;

    let mut curve_layers = Vec::new();
    for (i, c) in &props.curves {
        let segs = segments(&c.samples, y_span);
        if let Some(fill) = c.fill {
            let below = matches!(fill, epher_core::graph::Fill::Below);
            for seg in &segs {
                curve_layers.push(html! {
                    <polygon class={format!("fill curve-{i}")} points={fill_points(seg, below, &geom)} fill="currentColor" fill-opacity="0.18" />
                });
            }
        }
        for seg in &segs {
            curve_layers.push(html! {
                <polyline class={format!("curve curve-{i}")} points={polyline_points(seg, &geom)} fill="none" />
            });
        }
    }

    let mut grid_lines = Vec::new();
    for v in ticks(geom.x_min, geom.x_max, geom.step_x) {
        if v.abs() > (geom.x_max - geom.x_min) * 1e-9 {
            let x = geom.sx(v);
            grid_lines.push(html! {
                <line class="grid" x1={x.to_string()} y1={TOP.to_string()} x2={x.to_string()} y2={BOTTOM.to_string()} />
            });
        }
    }
    for v in ticks(geom.y_min, geom.y_max, geom.step_y) {
        if v.abs() > (geom.y_max - geom.y_min) * 1e-9 {
            let y = geom.sy(v);
            grid_lines.push(html! {
                <line class="grid" x1={LEFT.to_string()} y1={y.to_string()} x2={RIGHT.to_string()} y2={y.to_string()} />
            });
        }
    }

    let x_axis = (geom.x_min <= 0.0 && geom.x_max >= 0.0).then(|| {
        let x = geom.sx(0.0);
        html! {
            <line class="axis" x1={x.to_string()} y1={TOP.to_string()} x2={x.to_string()} y2={BOTTOM.to_string()} />
        }
    });
    let y_axis = geom.zero_axis.then(|| {
        let y = geom.sy(0.0);
        html! {
            <line class="axis" x1={LEFT.to_string()} y1={y.to_string()} x2={RIGHT.to_string()} y2={y.to_string()} />
        }
    });

    let mut x_labels = Vec::new();
    for v in ticks(geom.x_min, geom.x_max, geom.step_x) {
        let x = geom.sx(v);
        x_labels.push(html! {
            <text class="tick" x={x.to_string()} y={(HEIGHT - 6.0).to_string()} text-anchor="middle">{ label(v) }</text>
        });
    }
    let mut y_labels = Vec::new();
    for v in ticks(geom.y_min, geom.y_max, geom.step_y) {
        let y = geom.sy(v);
        y_labels.push(html! {
            <text class="tick" x={(LEFT - 4.0).to_string()} y={(y + 4.0).to_string()} text-anchor="end">{ label(v) }</text>
        });
    }

    let mut poi_nodes = Vec::new();
    if props.markers {
        for p in &props.pois {
            let (x, y) = (geom.sx(p.x), geom.sy(p.y));
            let text = format!("{} ({}, {})", p.label, label(p.x), label(p.y));
            poi_nodes.push(html! {
                <circle class="poi" cx={x.to_string()} cy={y.to_string()} r="4" />
            });
            poi_nodes.push(html! {
                <text class="poi-label" x={(x + 7.0).to_string()} y={(y - 7.0).to_string()}>{ text }</text>
            });
        }
    }

    let trace_node = props.trace.map(|t| {
        let x = geom.sx(t.x);
        let y = geom.sy(t.y);
        html! {
            <circle class="trace" cx={x.to_string()} cy={y.to_string()} r="5" />
        }
    });

    html! {
        <svg ref={svg_ref} viewBox={format!("0 0 {WIDTH} {HEIGHT}")} preserveAspectRatio="xMidYMid meet" role="img" aria-label={aria_label(&all)} tabindex="0" xmlns="http://www.w3.org/2000/svg" style={format!("--curve-width: {}", props.line_width)}>
            <title>{ aria_label(&all) }</title>
            <defs>
                // ADR-0038: zoomed windows leave samples outside the plot
                // area (the y fit and the x window both clip) - curves,
                // markers, and the trace stay inside the frame.
                <clipPath id="plot-clip">
                    <rect x={LEFT.to_string()} y={TOP.to_string()} width={(RIGHT - LEFT).to_string()} height={(BOTTOM - TOP).to_string()} />
                </clipPath>
            </defs>
            { for grid_lines }
            { x_axis }
            { y_axis }
            { for x_labels }
            { for y_labels }
            <g clip-path="url(#plot-clip)">
                { for curve_layers }
                { for poi_nodes }
                { trace_node }
            </g>
        </svg>
    }
}

// ===== 3D surfaces (ADR-0015) =====

/// One parsed mesh element from the surface_parts markup: its tag and the
/// attribute list, in document order. The markup generator is ours, so the
/// format is fixed: self-closing tags, double-quoted attributes, values
/// without quotes.
struct MeshElem {
    tag: String,
    attrs: Vec<(String, String)>,
}

/// Parse the mesh markup into its element list. A light hand-rolled scan
/// (~32 KB per orbit frame) — no regex machinery needed for markup we
/// generate ourselves.
fn parse_mesh(content: &str) -> Vec<MeshElem> {
    let bytes = content.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i] != b'<' {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        i += 1; // past '<'
        let tag_start = i;
        while i < bytes.len() && !bytes[i].is_ascii_whitespace() && bytes[i] != b'>' {
            i += 1;
        }
        let tag = content[tag_start..i].to_string();
        let mut attrs = Vec::new();
        loop {
            while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b'/') {
                i += 1;
            }
            if i >= bytes.len() || bytes[i] == b'>' {
                break;
            }
            let name_start = i;
            while i < bytes.len() && bytes[i] != b'=' && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            let name = content[name_start..i].to_string();
            while i < bytes.len() && bytes[i] != b'"' {
                i += 1;
            }
            i += 1; // past the opening quote
            let value_start = i;
            while i < bytes.len() && bytes[i] != b'"' {
                i += 1;
            }
            let value = content[value_start..i].to_string();
            i += 1; // past the closing quote
            attrs.push((name, value));
        }
        out.push(MeshElem { tag, attrs });
    }
    out
}

/// Do two frames' mesh markup describe the same element sequence? The
/// generator emits only `<polyline>` (mesh) and `<line>` (frame) tags, so
/// the per-tag counts are a complete structural signature.
fn same_mesh_shape(old: &str, new: &str) -> bool {
    old.matches("<polyline").count() == new.matches("<polyline").count()
        && old.matches("<line").count() == new.matches("<line").count()
}

/// Apply a new frame's coordinates to the existing DOM elements. Only the
/// attributes that can change between frames are written (points, depth
/// opacity, stroke width for polylines; the four coordinates and width for
/// frame lines) — `fill`/`stroke` are constants. Returns false when the
/// live DOM does not match the markup shape, so the caller rebuilds.
fn patch_mesh(el: &web_sys::Element, content: &str) -> bool {
    let parsed = parse_mesh(content);
    let kids = el.children();
    if kids.length() as usize != parsed.len() {
        return false;
    }
    for (index, elem) in parsed.iter().enumerate() {
        let Some(kid) = kids.item(index as u32) else {
            return false;
        };
        for (name, value) in &elem.attrs {
            let mutable = match (elem.tag.as_str(), name.as_str()) {
                ("polyline", "points" | "stroke-opacity" | "stroke-width") => true,
                ("line", "x1" | "y1" | "x2" | "y2" | "stroke-width") => true,
                _ => false,
            };
            if mutable {
                let _ = kid.set_attribute(name, value);
            }
        }
    }
    true
}

/// Render the plotted surfaces as SVG content: mesh lines per grid row and
/// column with per-line depth shading (nearer lines more opaque), the
/// ground square and axes of the first surface on top, all painter-sorted
/// far to near. Built as a string (not diffed elements) so orbiting a
/// thousand-line mesh stays cheap. Returns (viewBox, inner content) — the
/// arrow keys rotate (ADR-0015, WCAG 2.1.1). The SVG content is raw HTML
/// (innerHTML-style) so thousand-line meshes re-render without diffing.
#[derive(Properties, PartialEq)]
pub struct Graph3DProps {
    pub view_box: String,
    pub content: String,
    pub aria_label: String,
    /// (dyaw, dpitch) from a drag or arrow key.
    pub on_orbit: Callback<(f64, f64)>,
    /// Wheel notch or pinch (ADR-0038): a camera-distance factor -
    /// `> 1` moves the camera out (zoom out), `< 1` in (zoom in).
    pub on_zoom: Callback<f64>,
}

#[function_component(Graph3D)]
pub fn graph3d_html(props: &Graph3DProps) -> Html {
    let svg_ref = use_node_ref();
    let g_ref = use_node_ref();
    let drag = use_state(|| std::rc::Rc::new(std::cell::RefCell::new(Option::<(f64, f64)>::None)));
    // The active pointers (ADR-0038): one drags the orbit, two pinch the
    // camera distance - orbiting suspends while the second finger is down.
    let pointers = use_state(|| std::rc::Rc::new(std::cell::RefCell::new(
        Vec::<(i32, f64, f64)>::new(),
    )));

    // The mesh is injected with Element::set_inner_html on an SVG <g>, not
    // via Yew vnodes: Yew's from_html_unchecked parses fragments in an HTML
    // <div>, so the polyline nodes would carry the HTML namespace and the
    // SVG renderer would never paint them (blank plot in every browser).
    //
    // ADR-0027: orbit frames keep the same element structure (one
    // <polyline> per mesh line, one <line> per frame segment — only the
    // coordinate/opacity values change), so a frame whose shape matches
    // the previous one is applied by writing attributes on the existing
    // elements instead of re-parsing and re-creating thousands of nodes.
    // Per-frame innerHTML churn garbage-collected ~3k elements per frame,
    // which stalled and flickered in WebView2 (Windows); patching is a
    // few thousand attribute writes with zero node churn — 60fps in every
    // engine. Structure changes (different surfaces) still rebuild.
    let last_markup = use_state(|| std::rc::Rc::new(std::cell::RefCell::new(None::<String>)));
    {
        let g_ref = g_ref.clone();
        let last_markup = last_markup.clone();
        let content = props.content.clone();
        use_effect_with(content, move |content| {
            let Some(el) = g_ref.cast::<web_sys::Element>() else {
                return;
            };
            let same_shape = last_markup
                .borrow()
                .as_ref()
                .map(|old| same_mesh_shape(old, content))
                .unwrap_or(false);
            if same_shape && patch_mesh(&el, content) {
                // patched in place; nothing else to do
            } else {
                el.set_inner_html(content);
            }
            *last_markup.borrow_mut() = Some(content.clone());
        });
    }

    {
        let svg_ref = svg_ref.clone();
        let on_orbit = props.on_orbit.clone();
        let on_zoom = props.on_zoom.clone();
        let drag = drag.clone();
        let pointers = pointers.clone();
        let listeners = use_state(Vec::<gloo_events::EventListener>::new);
        use_effect_with((), move |_| {
            let Some(el) = svg_ref.cast::<web_sys::Element>() else {
                return;
            };
            let mut bound = Vec::new();
            {
                let el_closure = el.clone();
                let drag = drag.clone();
                let pointers = pointers.clone();
                bound.push(gloo_events::EventListener::new(
                    &el,
                    "pointerdown",
                    move |e| {
                        if let Some(pe) = e.dyn_ref::<web_sys::PointerEvent>() {
                            el_closure.set_pointer_capture(pe.pointer_id()).ok();
                            let mut pts = pointers.borrow_mut();
                            pts.retain(|(id, _, _)| *id != pe.pointer_id());
                            pts.push((pe.pointer_id(), pe.client_x() as f64, pe.client_y() as f64));
                            // A second finger suspends the orbit (the pinch
                            // takes over, ADR-0038).
                            if pts.len() > 1 {
                                *drag.borrow_mut() = None;
                            } else {
                                *drag.borrow_mut() =
                                    Some((pe.client_x() as f64, pe.client_y() as f64));
                            }
                        }
                    },
                ));
            }
            // Drags accumulate into `pending` and commit at most once per
            // animation frame (ADR-0026): re-rendering per pointer event
            // re-injected the whole mesh SVG mid-drag — the plot flickered
            // and, combined with the stale-handle orbit reads, "shivered"
            // instead of rotating. One commit per frame = smooth orbit.
            let pending = std::rc::Rc::new(std::cell::RefCell::new(None::<(f64, f64)>));
            let frame =
                std::rc::Rc::new(std::cell::RefCell::new(None::<gloo_render::AnimationFrame>));
            {
                let el = el.clone();
                let drag = drag.clone();
                let on_orbit = on_orbit.clone();
                let on_zoom = on_zoom.clone();
                let pending = pending.clone();
                let frame = frame.clone();
                let pointers = pointers.clone();
                let el_move = el.clone();
                bound.push(gloo_events::EventListener::new(
                    &el,
                    "pointermove",
                    move |e| {
                        let el = el_move.clone();
                        if let Some(pe) = e.dyn_ref::<web_sys::PointerEvent>() {
                            // Pinch first (ADR-0038): with two pointers on
                            // the plot, the distance ratio zooms the camera
                            // and the orbit stays still.
                            {
                                let mut pts = pointers.borrow_mut();
                                if let Some(slot) =
                                    pts.iter_mut().find(|(id, _, _)| *id == pe.pointer_id())
                                {
                                    slot.1 = pe.client_x() as f64;
                                    slot.2 = pe.client_y() as f64;
                                }
                                if pts.len() >= 2 {
                                    let dx = pts[1].1 - pts[0].1;
                                    let dy = pts[1].2 - pts[0].2;
                                    let dist = (dx * dx + dy * dy).sqrt();
                                    drop(pts);
                                    let last = el
                                        .get_attribute("data-pinch-dist")
                                        .and_then(|v| v.parse::<f64>().ok());
                                    let _ = el
                                        .set_attribute("data-pinch-dist", &format!("{dist}"));
                                    if let Some(last) = last {
                                        if last > 1.0 && dist > 1.0 {
                                            on_zoom.emit(last / dist);
                                        }
                                    }
                                    return;
                                }
                            }
                            // Copy the start point out first: Option<(f64, f64)>
                            // is Copy, and holding the Ref across the body would
                            // make the borrow_mut below panic ("RefCell already
                            // borrowed") — the drag never orbits.
                            let start = *drag.borrow();
                            if let Some((lx, ly)) = start {
                                let dx = pe.client_x() as f64 - lx;
                                let dy = pe.client_y() as f64 - ly;
                                *drag.borrow_mut() =
                                    Some((pe.client_x() as f64, pe.client_y() as f64));
                                if dx.abs() > 0.5 || dy.abs() > 0.5 {
                                    {
                                        let mut p = pending.borrow_mut();
                                        let (a, b) = p.unwrap_or((0.0, 0.0));
                                        *p = Some((a + dx * 0.01, b + dy * 0.01));
                                    }
                                    if frame.borrow().is_none() {
                                        let pending = pending.clone();
                                        let frame_inner = frame.clone();
                                        let on_orbit = on_orbit.clone();
                                        let handle =
                                            gloo_render::request_animation_frame(move |_| {
                                                *frame_inner.borrow_mut() = None;
                                                if let Some((a, b)) = pending.borrow_mut().take() {
                                                    on_orbit.emit((a, b));
                                                }
                                            });
                                        *frame.borrow_mut() = Some(handle);
                                    }
                                }
                            }
                        }
                    },
                ));
            }
            // Pointerup / pointerleave commit whatever is still pending so
            // the final position is exact even if the last events arrived
            // between frames.
            {
                let drag = drag.clone();
                let on_orbit = on_orbit.clone();
                let pending = pending.clone();
                let pointers = pointers.clone();
                let el = el.clone();
                for event_name in ["pointerup", "pointerleave", "pointercancel"] {
                    let drag = drag.clone();
                    let on_orbit = on_orbit.clone();
                    let pending = pending.clone();
                    let pointers = pointers.clone();
                    let el_inner = el.clone();
                    bound.push(gloo_events::EventListener::new(
                        &el,
                        event_name,
                        move |e| {
                            if let Some(pe) = e.dyn_ref::<web_sys::PointerEvent>() {
                                pointers
                                    .borrow_mut()
                                    .retain(|(id, _, _)| *id != pe.pointer_id());
                            }
                            if pointers.borrow().len() < 2 {
                                let _ = el_inner.remove_attribute("data-pinch-dist");
                            }
                            *drag.borrow_mut() = None;
                            if let Some((a, b)) = pending.borrow_mut().take() {
                                on_orbit.emit((a, b));
                            }
                        },
                    ));
                }
            }
            {
                let el = el.clone();
                let on_zoom = on_zoom.clone();
                bound.push(gloo_events::EventListener::new(&el, "wheel", move |e| {
                    let Some(we) = e.dyn_ref::<web_sys::WheelEvent>() else {
                        return;
                    };
                    // The page must not scroll while the scene zooms.
                    we.prevent_default();
                    // One notch = the camera moves 1.15×; scrolling up
                    // moves it closer.
                    on_zoom.emit((we.delta_y() / 300.0).exp().clamp(0.5, 2.0));
                }));
            }
            {
                let el = el.clone();
                let on_orbit = on_orbit.clone();
                bound.push(gloo_events::EventListener::new(&el, "keydown", move |e| {
                    if let Some(ke) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                        let (dyaw, dpitch) = match ke.key().as_str() {
                            "ArrowLeft" => (-0.15, 0.0),
                            "ArrowRight" => (0.15, 0.0),
                            "ArrowUp" => (0.0, 0.15),
                            "ArrowDown" => (0.0, -0.15),
                            _ => return,
                        };
                        ke.prevent_default();
                        on_orbit.emit((dyaw, dpitch));
                    }
                }));
            }
            listeners.set(bound);
        });
    }

    html! {
        <svg
            ref={svg_ref}
            tabindex="0"
            role="img"
            aria-label={props.aria_label.clone()}
            viewBox={props.view_box.clone()}
            preserveAspectRatio="xMidYMid meet"
            class="graph3d-svg"
        >
            <g ref={g_ref}></g>
        </svg>
    }
}
