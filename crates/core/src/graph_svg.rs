//! The plot-to-SVG renderer (ADR-0014, ADR-0020): pure, frontend-free
//! geometry and document assembly shared by every frontend. The web app
//! uses the same helpers for its live (Yew) renderer; the terminal
//! frontends and the copy-to-clipboard button call [`graph_svg`] for a
//! self-contained document — embedded style, fixed 640×400 viewBox — that
//! looks the same pasted into a document as it does in the app's default
//! theme.
//!
//! Accessibility (ADR-0009): the document carries `role="img"`, a
//! `<title>`, and an `aria-label` naming every plotted expression; curve
//! colors are >= 3:1 on the background, and every curve carries a
//! visible caption at its end so solid lines stay distinguishable
//! without color (WCAG 1.4.1).

use crate::graph::{InterestKind, SampledCurve, Segment3D, Surface, View3D};
use crate::Sample;

pub const WIDTH: f64 = 640.0;
pub const HEIGHT: f64 = 400.0;
pub const LEFT: f64 = 48.0;
pub const RIGHT: f64 = 632.0;
pub const TOP: f64 = 12.0;
pub const BOTTOM: f64 = 368.0;

/// The plot geometry shared by every rendered layer: value ranges, tick
/// steps, and whether the horizontal zero axis belongs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Geometry {
    pub x_min: f64,
    pub x_max: f64,
    pub y_min: f64,
    pub y_max: f64,
    pub step_x: f64,
    pub step_y: f64,
    pub zero_axis: bool,
}

impl Geometry {
    /// Map data x → viewBox x.
    pub fn sx(&self, x: f64) -> f64 {
        LEFT + (x - self.x_min) / (self.x_max - self.x_min) * (RIGHT - LEFT)
    }

    /// Map data y → viewBox y.
    pub fn sy(&self, y: f64) -> f64 {
        TOP + (1.0 - (y - self.y_min) / (self.y_max - self.y_min)) * (BOTTOM - TOP)
    }

    /// Map a viewBox x back to data x.
    pub fn unx(&self, px: f64) -> f64 {
        self.x_min + (px - LEFT) / (RIGHT - LEFT) * (self.x_max - self.x_min)
    }

    /// Map a viewBox y back to data y.
    pub fn uny(&self, py: f64) -> f64 {
        self.y_min + (1.0 - (py - TOP) / (BOTTOM - TOP)) * (self.y_max - self.y_min)
    }
}

/// Compute the shared plot geometry for a set of curves: the union of their
/// domains, the y range padded 6%, and 1/2/5-style tick steps. `None` when
/// nothing can be drawn.
pub fn geometry(curves: &[SampledCurve]) -> Option<Geometry> {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    let mut any = false;
    for c in curves {
        if c.samples.is_empty() {
            continue;
        }
        x_min = x_min.min(c.domain.0);
        x_max = x_max.max(c.domain.1);
        for s in &c.samples {
            if s.y.is_finite() {
                y_min = y_min.min(s.y);
                y_max = y_max.max(s.y);
                any = true;
            }
        }
    }
    if !any || !x_min.is_finite() || x_max <= x_min {
        return None;
    }
    let y_span = (y_max - y_min).max(1e-9);
    let pad = y_span * 0.06;
    let (y_min, y_max) = (y_min - pad, y_max + pad);
    let y_span = y_max - y_min;
    Some(Geometry {
        x_min,
        x_max,
        y_min,
        y_max,
        step_x: crate::graph::nice_step(x_max - x_min, 10),
        step_y: crate::graph::nice_step(y_span, 8),
        zero_axis: y_min <= 0.0 && y_max >= 0.0,
    })
}

/// Split a curve's samples into polyline segments at non-finite points
/// (gaps, not jumps) *and* at vertical jumps larger than a third of the
/// sampled value range — a false asymptote line must never connect the two
/// branches of `1 / x` or `tan(x)`.
pub fn segments(samples: &[Sample], y_span: f64) -> Vec<Vec<(f64, f64)>> {
    let threshold = 0.35 * y_span;
    let mut out = Vec::new();
    let mut current: Vec<(f64, f64)> = Vec::new();
    for w in samples.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        if current.is_empty() && a.y.is_finite() {
            current.push((a.x, a.y));
        }
        if !b.y.is_finite() || (a.y.is_finite() && (b.y - a.y).abs() > threshold) {
            if !current.is_empty() {
                out.push(std::mem::take(&mut current));
            }
        } else {
            if current.is_empty() && a.y.is_finite() {
                current.push((a.x, a.y));
            }
            current.push((b.x, b.y));
        }
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// Tick positions across `[lo, hi]` at the given step, snapping values
/// within a hair of zero (float drift) to exact 0.
pub fn ticks(lo: f64, hi: f64, step: f64) -> Vec<f64> {
    let mut out = Vec::new();
    if !lo.is_finite() || !hi.is_finite() || step <= 0.0 {
        return out;
    }
    let start = (lo / step).ceil() as i64;
    let end = (hi / step).floor() as i64;
    for i in start..=end {
        let v = i as f64 * step;
        out.push(if v.abs() < step * 1e-9 { 0.0 } else { v });
    }
    out
}

/// A readable label for a tick value: up to 3 decimals, trailing zeros
/// trimmed, no exponent surprises for graph-scale numbers.
pub fn label(v: f64) -> String {
    let s = format!("{v:.3}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    let s = if s == "-0" { "0" } else { s };
    s.to_string()
}

/// XML-escape text that lands in SVG attributes and elements.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

/// A point of interest as the renderer sees it: a localized kind label and
/// its coordinates.
#[derive(Debug, Clone, PartialEq)]
pub struct Poi {
    pub kind: InterestKind,
    pub label: String,
    pub x: f64,
    pub y: f64,
    /// The curve that carries this point (see [`InterestPoint::curve`]):
    /// the renderer and the web legend filter with it.
    pub curve: usize,
}

/// The trace cursor: which curve and sample, with its data coordinates.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TracePoint {
    pub curve: usize,
    pub index: usize,
    pub x: f64,
    pub y: f64,
}

/// The nearest sample point to a viewBox position, within a 100px radius.
pub fn trace_nearest(
    curves: &[SampledCurve],
    geom: &Geometry,
    px: f64,
    py: f64,
) -> Option<TracePoint> {
    const MAX_D2: f64 = 100.0 * 100.0;
    let mut best: Option<(f64, TracePoint)> = None;
    for (ci, c) in curves.iter().enumerate() {
        for (si, s) in c.samples.iter().enumerate() {
            if !s.y.is_finite() {
                continue;
            }
            let d2 = (geom.sx(s.x) - px).powi(2) + (geom.sy(s.y) - py).powi(2);
            if d2 < MAX_D2 && best.as_ref().is_none_or(|(bd, _)| d2 < *bd) {
                best = Some((
                    d2,
                    TracePoint {
                        curve: ci,
                        index: si,
                        x: s.x,
                        y: s.y,
                    },
                ));
            }
        }
    }
    best.map(|(_, t)| t)
}

/// A visible legend entry: which curve index, and its display text.
pub fn curve_caption(c: &SampledCurve) -> String {
    match &c.kind {
        crate::graph::CurveKind::Cartesian(_) => format!("y = {}", c.source.trim()),
        _ => c.source.trim().to_string(),
    }
}

/// The aria-label listing every plotted expression.
pub fn aria_label(curves: &[SampledCurve]) -> String {
    let names: Vec<String> = curves.iter().map(curve_caption).collect();
    format!("Graph of {}", names.join(", "))
}

pub fn polyline_points(seg: &[(f64, f64)], geom: &Geometry) -> String {
    seg.iter()
        .map(|(x, y)| format!("{:.1},{:.1}", geom.sx(*x), geom.sy(*y)))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Fill polygon points for a curve with a region fill: the curve, then the
/// bottom (or top) edge of the plot, closed back to the start.
pub fn fill_points(seg: &[(f64, f64)], below: bool, geom: &Geometry) -> String {
    let edge = if below { BOTTOM } else { TOP };
    let mut pts: Vec<String> = seg
        .iter()
        .map(|(x, y)| format!("{:.1},{:.1}", geom.sx(*x), geom.sy(*y)))
        .collect();
    let first = seg.first().map(|(x, _)| *x).unwrap_or(0.0);
    let last = seg.last().map(|(x, _)| *x).unwrap_or(0.0);
    pts.push(format!("{:.1},{edge:.1}", geom.sx(last)));
    pts.push(format!("{:.1},{edge:.1}", geom.sx(first)));
    pts.join(" ")
}

/// The shared SVG layer stack (grid, axes, tick labels) as a string.
pub fn layers_svg(geom: &Geometry, x_axis: bool) -> String {
    let mut s = String::new();
    let x_min = geom.x_min;
    let x_max = geom.x_max;

    // Gridlines.
    for v in ticks(x_min, x_max, geom.step_x) {
        if v.abs() > (x_max - x_min) * 1e-9 {
            let x = geom.sx(v);
            s.push_str(&format!(
                "<line class=\"grid\" x1=\"{x:.1}\" y1=\"{TOP:.1}\" x2=\"{x:.1}\" y2=\"{BOTTOM:.1}\" />"
            ));
        }
    }
    for v in ticks(geom.y_min, geom.y_max, geom.step_y) {
        if v.abs() > (geom.y_max - geom.y_min) * 1e-9 {
            let y = geom.sy(v);
            s.push_str(&format!(
                "<line class=\"grid\" x1=\"{LEFT:.1}\" y1=\"{y:.1}\" x2=\"{RIGHT:.1}\" y2=\"{y:.1}\" />"
            ));
        }
    }

    // Axes: x = 0 only when it lies inside the plotted domain; y = 0 only
    // when it lies inside the value range.
    if x_min <= 0.0 && x_max >= 0.0 {
        let x = geom.sx(0.0);
        s.push_str(&format!(
            "<line class=\"axis\" x1=\"{x:.1}\" y1=\"{TOP:.1}\" x2=\"{x:.1}\" y2=\"{BOTTOM:.1}\" />"
        ));
    }
    if x_axis {
        let y = geom.sy(0.0);
        s.push_str(&format!(
            "<line class=\"axis\" x1=\"{LEFT:.1}\" y1=\"{y:.1}\" x2=\"{RIGHT:.1}\" y2=\"{y:.1}\" />"
        ));
    }

    // Tick labels: x along the bottom, y along the left.
    for v in ticks(x_min, x_max, geom.step_x) {
        let x = geom.sx(v);
        s.push_str(&format!(
            "<text class=\"tick\" x=\"{x:.1}\" y=\"{:.1}\" text-anchor=\"middle\">{}</text>",
            HEIGHT - 6.0,
            escape(&label(v))
        ));
    }
    for v in ticks(geom.y_min, geom.y_max, geom.step_y) {
        let y = geom.sy(v);
        s.push_str(&format!(
            "<text class=\"tick\" x=\"{:.1}\" y=\"{:.1}\" text-anchor=\"end\">{}</text>",
            LEFT - 4.0,
            y + 4.0,
            escape(&label(v))
        ));
    }
    s
}

/// The default curve stroke width — half the pre-slider constant 2 (the
/// plotted lines read thinner, and the slider ranges around it).
pub const DEFAULT_STROKE_WIDTH: f64 = 1.0;

/// The embedded stylesheet: the app's default (dark) palette, so the
/// document is self-contained and renders identically anywhere it lands.
/// Every color keeps the recorded contrast ratios of the app's dark theme
/// (WCAG 1.4.11: curves >= 3:1 on --bg).
fn style_svg(stroke_width: f64) -> String {
    format!(
        "<style>\
.bg {{ fill: #141416; }}\
.curve {{ stroke-width: {stroke_width:.2}; stroke-linejoin: round; stroke-linecap: round; fill: none; }}\
.curve-0 {{ stroke: #2dd4bf; }}\
.curve-1 {{ stroke: #4da3ff; }}\
.curve-2 {{ stroke: #ffb340; }}\
.curve-3 {{ stroke: #c39dff; }}\
.label {{ font-size: 11px; font-family: ui-monospace, Menlo, Consolas, monospace; stroke: #141416; stroke-width: 3px; paint-order: stroke; }}\
.label.curve-0 {{ fill: #2dd4bf; }}\
.label.curve-1 {{ fill: #4da3ff; }}\
.label.curve-2 {{ fill: #ffb340; }}\
.label.curve-3 {{ fill: #c39dff; }}\
.fill.curve-0 {{ fill: #2dd4bf; fill-opacity: 0.18; stroke: none; }}\
.fill.curve-1 {{ fill: #4da3ff; fill-opacity: 0.18; stroke: none; }}\
.fill.curve-2 {{ fill: #ffb340; fill-opacity: 0.18; stroke: none; }}\
.fill.curve-3 {{ fill: #c39dff; fill-opacity: 0.18; stroke: none; }}\
.grid {{ stroke: #f5f6f7; stroke-opacity: 0.15; }}\
.axis {{ stroke: #f5f6f7; stroke-opacity: 0.5; }}\
.tick {{ fill: #9a9ba2; font-size: 11px; font-family: ui-monospace, Menlo, Consolas, monospace; }}\
.poi {{ fill: #2dd4bf; }}\
.poi-label {{ fill: #f5f6f7; font-size: 11px; font-family: ui-monospace, Menlo, Consolas, monospace; }}\
.trace {{ fill: #2dd4bf; stroke: #141416; stroke-width: 2; }}\
</style>"
    )
}

/// Render curves, points of interest, and the trace cursor as a
/// self-contained SVG document (embedded style, 640×400 viewBox and size —
/// the copy button, the terminal `graph save`, and the tests all produce
/// the same bytes). `stroke_width` is the curve line width
/// ([`DEFAULT_STROKE_WIDTH`] unless the user moved the slider). Nothing to
/// draw renders the empty string.
pub fn graph_svg(
    curves: &[SampledCurve],
    pois: &[Poi],
    trace: Option<TracePoint>,
    markers: bool,
    stroke_width: f64,
) -> String {
    // The plain entry point draws every curve with its position in the
    // slice as the palette index — the callers without hidden curves
    // (TUI, shell, core tests) always pass full slices, so position and
    // original index coincide.
    let indexed: Vec<(usize, SampledCurve)> =
        curves.iter().enumerate().map(|(i, c)| (i, c.clone())).collect();
    graph_svg_indexed(&indexed, pois, trace, markers, stroke_width)
}

/// [`graph_svg`] with explicit palette indices: the web pane filters
/// hidden curves out of the slice but must keep each curve's own colour
/// (ADR-0015 amendment) — a hidden neighbour must not shift the palette.
pub fn graph_svg_indexed(
    curves: &[(usize, SampledCurve)],
    pois: &[Poi],
    trace: Option<TracePoint>,
    markers: bool,
    stroke_width: f64,
) -> String {
    let all: Vec<SampledCurve> = curves.iter().map(|(_, c)| c.clone()).collect();
    let Some(geom) = geometry(&all) else {
        return String::new();
    };
    let y_span = geom.y_max - geom.y_min;

    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg viewBox=\"0 0 {WIDTH} {HEIGHT}\" width=\"{WIDTH}\" height=\"{HEIGHT}\" role=\"img\" aria-label=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">",
        escape(&aria_label(&all))
    ));
    svg.push_str(&format!("<title>{}</title>", escape(&aria_label(&all))));
    svg.push_str(&style_svg(stroke_width));
    svg.push_str(&format!(
        "<rect class=\"bg\" x=\"0\" y=\"0\" width=\"{WIDTH}\" height=\"{HEIGHT}\" />"
    ));
    svg.push_str(&layers_svg(&geom, geom.zero_axis));

    for (i, c) in curves {
        let segs = segments(&c.samples, y_span);
        if let Some(fill) = c.fill {
            let below = matches!(fill, crate::graph::Fill::Below);
            for seg in &segs {
                svg.push_str(&format!(
                    "<polygon class=\"fill curve-{i}\" points=\"{}\" />",
                    fill_points(seg, below, &geom)
                ));
            }
        }
        for seg in &segs {
            svg.push_str(&format!(
                "<polyline class=\"curve curve-{i}\" points=\"{}\" />",
                polyline_points(seg, &geom)
            ));
        }
        // A visible caption at the curve's end: curves are all solid
        // (ADR-0023), so the caption — plus the aria-label and the
        // `<title>` — is the non-color channel that keeps them apart
        // (WCAG 1.4.1).
        if let Some(last) = c.samples.iter().rev().find(|s| s.y.is_finite()) {
            let (x, y) = (geom.sx(last.x), geom.sy(last.y));
            let lx = (x + 6.0).clamp(LEFT, RIGHT - 8.0);
            let ly = (y - 6.0).clamp(TOP + 8.0, BOTTOM);
            svg.push_str(&format!(
                "<text class=\"label curve-{i}\" x=\"{lx:.1}\" y=\"{ly:.1}\">{}</text>",
                escape(&curve_caption(c))
            ));
        }
    }

    if markers {
        for p in pois {
            let (x, y) = (geom.sx(p.x), geom.sy(p.y));
            svg.push_str(&format!(
                "<circle class=\"poi\" cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"4\" />"
            ));
            svg.push_str(&format!(
                "<text class=\"poi-label\" x=\"{:.1}\" y=\"{:.1}\">{}</text>",
                x + 7.0,
                y - 7.0,
                escape(&format!("{} ({}, {})", p.label, label(p.x), label(p.y)))
            ));
        }
    }

    if let Some(t) = trace {
        let (x, y) = (geom.sx(t.x), geom.sy(t.y));
        svg.push_str(&format!(
            "<circle class=\"trace\" cx=\"{x:.1}\" cy=\"{y:.1}\" r=\"5\" />"
        ));
    }

    svg.push_str("</svg>");
    svg
}

/// The mesh and frame of a 3D surface set as raw polyline/line markup in
/// data coordinates (the live web renderer injects this into its own
/// letterboxed `<svg>`). Returns the content view box and the markup.
pub fn surface_parts(
    surfaces: &[Surface],
    view: &View3D,
    stroke_width: f64,
) -> Option<(String, String)> {
    use crate::graph::{project_mesh, surface_frame, Polyline3D};
    if surfaces.is_empty() {
        return None;
    }
    let mut mesh: Vec<Polyline3D> = Vec::new();
    for s in surfaces {
        mesh.extend(project_mesh(s, view));
    }
    let frame: Vec<Segment3D> = surface_frame(&surfaces[0], view);
    if mesh.is_empty() && frame.is_empty() {
        return None;
    }
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    let mut z_min = f64::INFINITY;
    let mut z_max = f64::NEG_INFINITY;
    for line in &mesh {
        for &(x, y) in &line.points {
            x_min = x_min.min(x);
            x_max = x_max.max(x);
            y_min = y_min.min(y);
            y_max = y_max.max(y);
        }
        z_min = z_min.min(line.depth);
        z_max = z_max.max(line.depth);
    }
    for seg in &frame {
        x_min = x_min.min(seg.x1).min(seg.x2);
        x_max = x_max.max(seg.x1).max(seg.x2);
        y_min = y_min.min(seg.y1).min(seg.y2);
        y_max = y_max.max(seg.y1).max(seg.y2);
    }
    if !x_min.is_finite() || x_max - x_min < 1e-9 || y_max - y_min < 1e-9 {
        return None;
    }
    let pad = (x_max - x_min).max(y_max - y_min) * 0.06;
    let x_min = x_min - pad;
    let x_max = x_max + pad;
    let y_min = y_min - pad;
    let y_max = y_max + pad;
    let span = z_max - z_min;
    let view_box = format!(
        "{x_min:.3} {y_min:.3} {:.3} {:.3}",
        x_max - x_min,
        y_max - y_min
    );
    let mut parts = String::new();
    // Painter's order: project_mesh already sorts far-to-near, so drawing
    // in order lets nearer lines overpaint farther ones.
    for line in &mesh {
        let t = if span < 1e-9 {
            1.0
        } else {
            ((line.depth - z_min) / span).clamp(0.0, 1.0)
        };
        // Depth cue without color: opacity 0.35 far → 0.95 near.
        let opacity = 0.35 + 0.6 * t;
        let points = line
            .points
            .iter()
            .map(|(x, y)| format!("{x:.3},{y:.3}"))
            .collect::<Vec<_>>()
            .join(" ");
        parts.push_str(&format!(
            "<polyline points=\"{points}\" fill=\"none\" stroke=\"currentColor\" stroke-opacity=\"{opacity:.3}\" stroke-width=\"{:.2}\"/>",
            1.2 * stroke_width
        ));
    }
    for seg in &frame {
        parts.push_str(&format!(
            "<line x1=\"{:.3}\" y1=\"{:.3}\" x2=\"{:.3}\" y2=\"{:.3}\" stroke=\"currentColor\" stroke-width=\"{:.2}\" stroke-opacity=\"0.9\"/>",
            seg.x1, seg.y1, seg.x2, seg.y2,
            1.4 * stroke_width
        ));
    }
    Some((view_box, parts))
}

/// Render a 3D surface set as a self-contained SVG document: the mesh and
/// frame of [`surface_parts`], letterboxed into the same 640×400 canvas
/// the 2D plot uses, on the default dark background. `stroke_width` scales
/// the mesh and frame lines (1.0 = the default weight). `None` when
/// nothing can be drawn.
pub fn graph3d_svg(surfaces: &[Surface], view: &View3D, stroke_width: f64) -> Option<String> {
    let (view_box, parts) = surface_parts(surfaces, view, stroke_width)?;
    let mut it = view_box.split_whitespace();
    let (mut x, mut y, mut w, mut h) = (0.0f64, 0.0f64, 1.0f64, 1.0f64);
    if let (Some(a), Some(b), Some(c), Some(d)) = (it.next(), it.next(), it.next(), it.next()) {
        if let (Ok(a), Ok(b), Ok(c), Ok(d)) = (a.parse(), b.parse(), c.parse(), d.parse()) {
            (x, y, w, h) = (a, b, c, d);
        }
    }
    // Letterbox the content box into WIDTH×HEIGHT, exactly what
    // preserveAspectRatio="xMidYMid meet" does in the live renderer.
    let scale = (WIDTH / w).min(HEIGHT / h);
    let tx = (WIDTH - w * scale) / 2.0 - x * scale;
    let ty = (HEIGHT - h * scale) / 2.0 - y * scale;
    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg viewBox=\"0 0 {WIDTH} {HEIGHT}\" width=\"{WIDTH}\" height=\"{HEIGHT}\" role=\"img\" aria-label=\"3D graph\" xmlns=\"http://www.w3.org/2000/svg\">"
    ));
    svg.push_str("<title>3D graph</title>");
    svg.push_str(&style_svg(stroke_width));
    svg.push_str(&format!(
        "<rect class=\"bg\" x=\"0\" y=\"0\" width=\"{WIDTH}\" height=\"{HEIGHT}\" />"
    ));
    svg.push_str(&format!(
        "<g stroke=\"#f5f6f7\" transform=\"translate({tx:.3} {ty:.3}) scale({scale:.5})\">{parts}</g>"
    ));
    svg.push_str("</svg>");
    Some(svg)
}
