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

use crate::graph::{zoom_window, DataPlot, InterestKind, SampledCurve, Segment3D, Surface, View3D};
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

/// The plot geometry over an explicit x-window (ADR-0038): wheel, pinch,
/// and the zoom slider set the pane's window; the y range still comes
/// from the samples that fall inside it, padded like [`geometry`]. When
/// nothing is visible in the window the y range degrades to the samples'
/// full range so the axes stay drawable.
pub fn geometry_in(curves: &[SampledCurve], x_min: f64, x_max: f64) -> Option<Geometry> {
    if !(x_min.is_finite() && x_max.is_finite()) || x_max <= x_min {
        return geometry(curves);
    }
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    let mut any = false;
    for c in curves {
        for s in &c.samples {
            if s.y.is_finite() && s.x >= x_min && s.x <= x_max {
                y_min = y_min.min(s.y);
                y_max = y_max.max(s.y);
                any = true;
            }
        }
    }
    if !any {
        // Nothing inside the window (zoomed out past the data, or zoomed
        // into a gap): keep the axes honest with the samples' full range.
        return geometry(curves).map(|mut g| {
            g.x_min = x_min;
            g.x_max = x_max;
            g.step_x = crate::graph::nice_step(x_max - x_min, 10);
            g
        });
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

/// The scatter fit's legend text (the live web renderer's fit
/// caption): the model's own equation with the legend's short number
/// spelling. The linear form is what the line fit has always shown.
pub fn fit_legend(f: &crate::graph::Fit) -> String {
    use crate::graph::ScatterFit;
    let g = f.fit;
    match f.model {
        ScatterFit::Linreg => {
            format!("y = {}*x + {} (r = {})", label(g.a), label(g.b), label(g.r))
        }
        ScatterFit::Quadreg => format!(
            "y = {}*x^2 + {}*x + {} (r = {})",
            label(g.a),
            label(g.b),
            label(g.c),
            label(g.r)
        ),
        ScatterFit::Expreg => format!("y = {}*e^({}*x) (r = {})", label(g.a), label(g.b), label(g.r)),
        ScatterFit::Powreg => format!("y = {}*x^{} (r = {})", label(g.a), label(g.b), label(g.r)),
        ScatterFit::Logreg => format!("y = {} + {}*ln(x) (r = {})", label(g.a), label(g.b), label(g.r)),
    }
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

/// The default 3D line width (the width slider's value, ADR-0055): the
/// desktop range is 0.0-0.4 in 0.05 steps with 0.2 as the default, so
/// the mid-range value is what a fresh plot draws.
pub const THREE_D_DEFAULT_WIDTH: f64 = 0.2;

/// 3D widths are screen px: `vector-effect="non-scaling-stroke"` keeps
/// a mesh line at the slider's value times this factor no matter how the
/// scene letterboxes or zooms (the old world-unit strokes scaled with
/// the pane, so the default rendered several times thicker than the 2D
/// curves it sat beside). The factor turns the 0.2 default into 2 px.
pub const THREE_D_PX_PER_WIDTH: f64 = 10.0;

/// The embedded stylesheet: one of the app's three theme palettes
/// (ADR-0057), drawn on a transparent canvas so exported plots sit on
/// any document, slide, or page without a painted box. Colors mirror
/// the app's CSS variables for the chosen theme, so an export wears
/// the same colors the pane wears.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SvgPalette {
    Dark,
    Light,
    Night,
}

impl SvgPalette {
    /// (curve-0..3, foreground for grid/axes, muted for ticks, the
    /// label halo behind plot text) for one theme, mirroring
    /// crates/web/index.html's `--accent`, `--curve-*`, `--text`, and
    /// `--muted` variables with their recorded contrast ratios.
    fn colors(self) -> (&'static str, &'static str, &'static str, &'static str, [&'static str; 4]) {
        match self {
            SvgPalette::Dark => (
                "#f5f6f7",
                "#9a9ba2",
                "#141416",
                "#2dd4bf",
                ["#2dd4bf", "#4da3ff", "#ffb340", "#c39dff"],
            ),
            SvgPalette::Light => (
                "#1b1d1f",
                "#565d63",
                "#f7f8f8",
                "#0e8074",
                ["#0e8074", "#1e66c8", "#9a5b00", "#7a4bd6"],
            ),
            SvgPalette::Night => (
                "#ffb3a8",
                "#d98878",
                "#0d0000",
                "#ff6b5a",
                ["#ff6b5a", "#ffb340", "#ff9e8a", "#e0483e"],
            ),
        }
    }
}

/// The palette color for curve index `i` (wraps past four, exactly
/// like the CSS classes the live pane uses).
pub fn palette_curve(palette: SvgPalette, i: usize) -> &'static str {
    palette.colors().4[i % 4]
}

/// One legend row in an exported document (ADR-0057): the swatch color
/// and the caption text, the same entry the pane's legend list shows.
#[derive(Clone, Debug)]
pub struct LegendEntry {
    pub color: String,
    pub caption: String,
}

fn style_svg(stroke_width: f64, palette: SvgPalette) -> String {
    let (fg, muted, halo, _accent, curves) = palette.colors();
    let c0 = curves[0];
    let (c1, c2, c3) = (curves[1], curves[2], curves[3]);
    format!(
        "<style>\
.curve {{ stroke-width: {stroke_width:.2}; stroke-linejoin: round; stroke-linecap: round; fill: none; }}\
.curve-0 {{ stroke: {c0}; }}\
.curve-1 {{ stroke: {c1}; }}\
.curve-2 {{ stroke: {c2}; }}\
.curve-3 {{ stroke: {c3}; }}\
.label {{ font-size: 11px; font-family: ui-monospace, Menlo, Consolas, monospace; stroke: {halo}; stroke-width: 3px; paint-order: stroke; }}\
.label.curve-0 {{ fill: {c0}; }}\
.label.curve-1 {{ fill: {c1}; }}\
.label.curve-2 {{ fill: {c2}; }}\
.label.curve-3 {{ fill: {c3}; }}\
.fill.curve-0 {{ fill: {c0}; fill-opacity: 0.18; stroke: none; }}\
.fill.curve-1 {{ fill: {c1}; fill-opacity: 0.18; stroke: none; }}\
.fill.curve-2 {{ fill: {c2}; fill-opacity: 0.18; stroke: none; }}\
.fill.curve-3 {{ fill: {c3}; fill-opacity: 0.18; stroke: none; }}\
.grid {{ stroke: {fg}; stroke-opacity: 0.15; }}\
.axis {{ stroke: {fg}; stroke-opacity: 0.5; }}\
.tick {{ fill: {muted}; font-size: 11px; font-family: ui-monospace, Menlo, Consolas, monospace; }}\
.poi {{ fill: {c0}; }}\
.poi-label {{ fill: {fg}; font-size: 11px; font-family: ui-monospace, Menlo, Consolas, monospace; }}\
.trace {{ fill: {c0}; stroke: {halo}; stroke-width: 2; }}\
</style>"
    )
}

/// The legend band under an exported plot (ADR-0057): the pane's
/// legend entries, at most two per row, each a color swatch and its
/// caption in the theme's foreground. Returns (band height, markup);
/// an empty legend renders nothing. The caller grows the canvas by the
/// band height.
fn legend_band(legend: &[LegendEntry], palette: SvgPalette) -> (f64, String) {
    if legend.is_empty() {
        return (0.0, String::new());
    }
    let (fg, _, _, _, _) = palette.colors();
    let rows = legend.len().div_ceil(2);
    let mut out = String::new();
    for (k, e) in legend.iter().enumerate() {
        let row = (k / 2) as f64;
        let col = k % 2;
        let x = 14.0 + col as f64 * 312.0;
        let y = HEIGHT + 14.0 + row * 16.0;
        out.push_str(&format!(
            "<line x1=\"{x:.1}\" y1=\"{:.1}\" x2=\"{:.1}\" y2=\"{:.1}\" stroke=\"{}\" stroke-width=\"2.5\" stroke-linecap=\"round\"/>",
            y - 4.0,
            x + 18.0,
            y - 4.0,
            escape(&e.color),
        ));
        out.push_str(&format!(
            "<text x=\"{:.1}\" y=\"{y:.1}\" fill=\"{fg}\" font-size=\"12px\" font-family=\"ui-monospace, Menlo, Consolas, monospace\">{}</text>",
            x + 24.0,
            escape(&e.caption),
        ));
    }
    (rows as f64 * 16.0 + 6.0, out)
}

/// The plot geometry of a data plot (ADR-0044), optionally inside an
/// explicit x-window (ADR-0055: the wheel, pinch, and the zoom slider
/// work on data plots exactly as they do on 2D curves). Inside a
/// window the y range fits the elements that fall in it; a window that
/// holds nothing degrades to the data's full y range so the axes stay
/// drawable. `None` when the plot carries no data at all.
pub fn data_geometry(data: &DataPlot, window: Option<(f64, f64)>) -> Option<Geometry> {
    let (fx0, fx1, fy0, fy1) = crate::graph::data_ranges(data);
    if !(fx0.is_finite() && fx1.is_finite() && fx1 > fx0) {
        return None;
    }
    let (x_min, x_max) = match window {
        Some((lo, hi)) if lo.is_finite() && hi.is_finite() && hi > lo => (lo, hi),
        _ => (fx0, fx1),
    };
    use crate::graph::DataPlotKind;
    let mut y_min = fy0;
    let mut y_max = fy1;
    if window.is_some() {
        let (wy0, wy1) = match data.kind {
            DataPlotKind::Scatter => {
                let mut lo = f64::INFINITY;
                let mut hi = f64::NEG_INFINITY;
                for (x, y) in &data.points {
                    if *x >= x_min && *x <= x_max {
                        lo = lo.min(*y);
                        hi = hi.max(*y);
                    }
                }
                (lo, hi)
            }
            DataPlotKind::Histogram => {
                let mut hi = 0.0f64;
                for (a, b, c) in &data.bins {
                    if *b >= x_min && *a <= x_max {
                        hi = hi.max(*c);
                    }
                }
                (0.0, hi)
            }
            DataPlotKind::BoxPlot => (-0.5, 1.5),
        };
        // A window that holds nothing degrades to the data's full y
        // range so the axes stay drawable.
        if wy0.is_finite() && wy1.is_finite() {
            y_min = wy0;
            y_max = wy1;
        }
    }
    let y_span = (y_max - y_min).max(1e-9);
    let pad = y_span * 0.08;
    y_min -= pad;
    y_max += pad;
    Some(Geometry {
        x_min,
        x_max,
        y_min,
        y_max,
        step_x: crate::graph::nice_step(x_max - x_min, 10),
        step_y: crate::graph::nice_step(y_max - y_min, 8),
        zero_axis: y_min <= 0.0 && y_max >= 0.0,
    })
}

/// Render a data plot (ADR-0044) as a self-contained SVG document with
/// the same frame as [`graph_svg`]: scatter points with the fitted line,
/// histogram bars, or a box-and-whisker. Nothing to draw renders the
/// empty string.
pub fn data_svg(data: &DataPlot, stroke_width: f64) -> String {
    data_svg_in(data, None, stroke_width)
}

/// [`data_svg`] inside an explicit x-window (ADR-0055): the pane's
/// zoom window clips the picture the same way the live renderer does,
/// so the export shows what the pane shows.
pub fn data_svg_in(data: &DataPlot, window: Option<(f64, f64)>, stroke_width: f64) -> String {
    data_svg_styled(data, window, stroke_width, SvgPalette::Dark, &[])
}

/// [`data_svg_in`] wearing the app's theme palette and carrying the
/// pane's legend entries (ADR-0057): the export shows the same colors
/// and the same captions the pane does.
pub fn data_svg_styled(
    data: &DataPlot,
    window: Option<(f64, f64)>,
    stroke_width: f64,
    palette: SvgPalette,
    legend: &[LegendEntry],
) -> String {
    use crate::graph::DataPlotKind;
    let Some(geom) = data_geometry(data, window) else {
        return String::new();
    };
    let (band, legend_svg) = legend_band(legend, palette);
    let height = HEIGHT + band;
    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg viewBox=\"0 0 {WIDTH} {height}\" width=\"{WIDTH}\" height=\"{height}\" role=\"img\" aria-label=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">",
        escape(&data.source)
    ));
    svg.push_str(&format!("<title>{}</title>", escape(&data.source)));
    svg.push_str(&style_svg(stroke_width, palette));
    svg.push_str(&layers_svg(&geom, geom.zero_axis));
    for v in ticks(geom.x_min, geom.x_max, geom.step_x) {
        let x = geom.sx(v);
        svg.push_str(&format!(
            "<text class=\"tick\" x=\"{x}\" y=\"{}\" text-anchor=\"middle\">{}</text>",
            HEIGHT - 6.0,
            label(v)
        ));
    }
    for v in ticks(geom.y_min, geom.y_max, geom.step_y) {
        let y = geom.sy(v);
        svg.push_str(&format!(
            "<text class=\"tick\" x=\"{}\" y=\"{}\" text-anchor=\"end\">{}</text>",
            LEFT - 4.0,
            y + 4.0,
            label(v)
        ));
    }
    match data.kind {
        DataPlotKind::Scatter => {
            for (x, y) in &data.points {
                if window.is_some() && (*x < geom.x_min || *x > geom.x_max) {
                    continue;
                }
                svg.push_str(&format!(
                    "<circle class=\"poi\" cx=\"{}\" cy=\"{}\" r=\"4\" />",
                    geom.sx(*x),
                    geom.sy(*y)
                ));
            }
            if let Some(f) = data.fit {
                // Sample the model across the window (the quadratic can
                // turn inside it; endpoints alone would miss that).
                const FIT_SAMPLES: usize = 24;
                let mut pts = String::new();
                for k in 0..=FIT_SAMPLES {
                    let x = geom.x_min + (geom.x_max - geom.x_min) * k as f64 / FIT_SAMPLES as f64;
                    let y = f.fit.eval(x);
                    if y.is_finite() {
                        pts.push_str(&format!("{},{:.2} ", geom.sx(x), geom.sy(y)));
                    }
                }
                svg.push_str(&format!(
                    "<polyline class=\"curve curve-0\" points=\"{}\" />",
                    pts.trim_end()
                ));
                svg.push_str(&format!(
                    "<text class=\"label curve-0\" x=\"{}\" y=\"{}\">{}</text>",
                    LEFT + 6.0,
                    TOP + 16.0,
                    fit_label(data)
                ));
            }
        }
        DataPlotKind::Histogram => {
            for (lo, hi, count) in &data.bins {
                if window.is_some() && (*hi < geom.x_min || *lo > geom.x_max) {
                    continue;
                }
                let x0 = geom.sx((*lo).max(geom.x_min));
                let x1 = geom.sx((*hi).min(geom.x_max));
                let y0 = geom.sy(0.0);
                let y1 = geom.sy(*count);
                let w = (x1 - x0).max(0.5);
                svg.push_str(&format!(
                    "<rect class=\"fill curve-0\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" />",
                    x0,
                    y1,
                    w,
                    (y0 - y1).max(0.0)
                ));
            }
        }
        DataPlotKind::BoxPlot => {
            if let Some(b) = data.boxplot {
                let y = geom.sy(0.5);
                let x = |v: f64| geom.sx(v.clamp(geom.x_min, geom.x_max));
                if b[4] >= geom.x_min && b[0] <= geom.x_max {
                    svg.push_str(&format!(
                        "<line class=\"axis\" x1=\"{}\" y1=\"{y}\" x2=\"{}\" y2=\"{y}\" />",
                        x(b[0]),
                        x(b[4])
                    ));
                }
                for edge in [b[0], b[4]] {
                    if edge >= geom.x_min && edge <= geom.x_max {
                        svg.push_str(&format!(
                            "<line class=\"axis\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" />",
                            x(edge),
                            y - 10.0,
                            x(edge),
                            y + 10.0
                        ));
                    }
                }
                if b[3] >= geom.x_min && b[1] <= geom.x_max {
                    svg.push_str(&format!(
                        "<rect class=\"fill curve-0\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"20\" />",
                        x(b[1]),
                        y - 10.0,
                        (x(b[3]) - x(b[1])).max(0.5)
                    ));
                }
                if b[2] >= geom.x_min && b[2] <= geom.x_max {
                    svg.push_str(&format!(
                        "<line class=\"axis\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" />",
                        x(b[2]),
                        y - 10.0,
                        x(b[2]),
                        y + 10.0
                    ));
                }
            }
        }
    }
    svg.push_str(&legend_svg);
    svg.push_str("</svg>");
    svg
}

/// The fit's on-graph label: the model's own equation in the graph's
/// short number spelling. The linear form is the label the line fit
/// has always carried.
fn fit_label(plot: &DataPlot) -> String {
    let Some(f) = plot.fit else {
        return String::new();
    };
    let g = f.fit;
    match f.model {
        crate::graph::ScatterFit::Linreg => {
            format!("y = {}*x + {} (r = {})", short(g.a), short(g.b), short(g.r))
        }
        crate::graph::ScatterFit::Quadreg => format!(
            "y = {}*x^2 + {}*x + {} (r = {})",
            short(g.a),
            short(g.b),
            short(g.c),
            short(g.r)
        ),
        crate::graph::ScatterFit::Expreg => {
            format!("y = {}*e^({}*x) (r = {})", short(g.a), short(g.b), short(g.r))
        }
        crate::graph::ScatterFit::Powreg => {
            format!("y = {}*x^{} (r = {})", short(g.a), short(g.b), short(g.r))
        }
        crate::graph::ScatterFit::Logreg => {
            format!("y = {} + {}*ln(x) (r = {})", short(g.a), short(g.b), short(g.r))
        }
    }
}

/// One short number for a scatter legend entry.
fn short(x: f64) -> String {
    let s = format!("{x:.4}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".to_string()
    } else {
        s.to_string()
    }
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
    let indexed: Vec<(usize, SampledCurve)> = curves
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.clone()))
        .collect();
    graph_svg_styled(
        &indexed,
        pois,
        trace,
        markers,
        stroke_width,
        SvgPalette::Dark,
        &[],
    )
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
    graph_svg_styled(
        curves,
        pois,
        trace,
        markers,
        stroke_width,
        SvgPalette::Dark,
        &[],
    )
}

/// [`graph_svg_indexed`] wearing the app's theme palette and carrying
/// the pane's legend entries (ADR-0057).
#[allow(clippy::too_many_arguments)]
pub fn graph_svg_styled(
    curves: &[(usize, SampledCurve)],
    pois: &[Poi],
    trace: Option<TracePoint>,
    markers: bool,
    stroke_width: f64,
    palette: SvgPalette,
    legend: &[LegendEntry],
) -> String {
    let all: Vec<SampledCurve> = curves.iter().map(|(_, c)| c.clone()).collect();
    let Some(geom) = geometry(&all) else {
        return String::new();
    };
    let y_span = geom.y_max - geom.y_min;

    let (band, legend_svg) = legend_band(legend, palette);
    let height = HEIGHT + band;
    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg viewBox=\"0 0 {WIDTH} {height}\" width=\"{WIDTH}\" height=\"{height}\" role=\"img\" aria-label=\"{}\" xmlns=\"http://www.w3.org/2000/svg\">",
        escape(&aria_label(&all))
    ));
    svg.push_str(&format!("<title>{}</title>", escape(&aria_label(&all))));
    svg.push_str(&style_svg(stroke_width, palette));
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

    svg.push_str(&legend_svg);
    svg.push_str("</svg>");
    svg
}

/// The mesh and frame of a 3D surface set as raw polyline/line markup in
/// data coordinates (the live web renderer injects this into its own
/// letterboxed `<svg>`). Returns the content view box and the markup.
/// Surfaces keep their position in the slice as the palette index, so
/// every scene element carries its own legend colour.
pub fn surface_parts(
    surfaces: &[Surface],
    view: &View3D,
    stroke_width: f64,
) -> Option<(String, String)> {
    let indexed: Vec<(usize, Surface)> = surfaces.iter().cloned().enumerate().collect();
    scene_parts_indexed(&indexed, &[], view, stroke_width)
}

/// The mesh and frame of a set of 3D parametric curves as raw
/// polyline/line markup in data coordinates (ADR-0054); the curve
/// sibling of [`surface_parts`]: one polyline per curve run (split at
/// non-finite samples), the ground square and axes on top, all
/// painter-sorted far to near.
pub fn curve_parts(
    curves: &[crate::graph::SpaceCurve],
    view: &View3D,
    stroke_width: f64,
) -> Option<(String, String)> {
    let indexed: Vec<(usize, crate::graph::SpaceCurve)> =
        curves.iter().cloned().enumerate().collect();
    scene_parts_indexed(&[], &indexed, view, stroke_width)
}

/// The 3D scene markup shared by surface sets and space curves: mesh
/// lines (surface grid lines or curve runs) with per-line depth
/// shading, the frame of the first scene element on top.
pub fn scene_parts(
    surfaces: &[Surface],
    curves: &[crate::graph::SpaceCurve],
    view: &View3D,
    stroke_width: f64,
) -> Option<(String, String)> {
    let si: Vec<(usize, Surface)> = surfaces.iter().cloned().enumerate().collect();
    let ci: Vec<(usize, crate::graph::SpaceCurve)> = curves.iter().cloned().enumerate().collect();
    scene_parts_indexed(&si, &ci, view, stroke_width)
}

/// [`scene_parts`] with explicit palette indices (ADR-0015 amendment):
/// the web pane filters hidden scene elements out of the slice but each
/// keeps its own colour, exactly like the 2D curves - a hidden neighbour
/// must not shift the remaining surfaces' or curves' colours.
pub fn scene_parts_indexed(
    surfaces: &[(usize, Surface)],
    curves: &[(usize, crate::graph::SpaceCurve)],
    view: &View3D,
    stroke_width: f64,
) -> Option<(String, String)> {
    use crate::graph::{curve_frame, project_curve, project_mesh, Polyline3D};
    if surfaces.is_empty() && curves.is_empty() {
        return None;
    }
    // Each mesh line remembers the palette index of the scene element it
    // came from, so one surface never borrows its neighbour's colour.
    let mut mesh: Vec<(usize, Polyline3D)> = Vec::new();
    for (i, s) in surfaces {
        for line in project_mesh(s, view) {
            mesh.push((*i, line));
        }
    }
    for (i, c) in curves {
        for line in project_curve(c, view) {
            mesh.push((*i, line));
        }
    }
    // The frame comes from the first scene element (the surface's
    // square domain, or the curve's bounding box).
    let frame: Vec<Segment3D> = if let Some((_, s)) = surfaces.first() {
        crate::graph::surface_frame(s, view)
    } else if let Some((_, c)) = curves.first() {
        curve_frame(c, view)
    } else {
        Vec::new()
    };
    if mesh.is_empty() && frame.is_empty() {
        return None;
    }
    // The scene's bounding sphere around the origin (ADR-0041): the
    // same rotation-stable window the surface renderer uses.
    let mut world: Vec<[f64; 3]> = Vec::new();
    for (_, s) in surfaces {
        for (i, &x) in s.xs.iter().enumerate() {
            for (j, &y) in s.ys.iter().enumerate() {
                if let Some(&z) = s.zs.get(i).and_then(|row| row.get(j)) {
                    if z.is_finite() {
                        world.push([x, y, z]);
                    }
                }
            }
        }
        let (a, b) = s.domain;
        for &(x, y) in &[(a, a), (b, a), (b, b), (a, b)] {
            world.push([x, y, 0.0]);
        }
    }
    for (_, c) in curves {
        for p in &c.points {
            if p.iter().all(|v| v.is_finite()) {
                world.push(*p);
            }
        }
    }
    let mut z_min = f64::INFINITY;
    let mut z_max = f64::NEG_INFINITY;
    for (_, line) in &mesh {
        z_min = z_min.min(line.depth);
        z_max = z_max.max(line.depth);
    }
    let span = z_max - z_min;
    let view_box = match crate::graph::stable_window(world.iter().copied(), view) {
        Some((wx, wy, ww, wh)) => format!("{wx:.3} {wy:.3} {ww:.3} {wh:.3}"),
        None => {
            let mut x_min = f64::INFINITY;
            let mut x_max = f64::NEG_INFINITY;
            let mut y_min = f64::INFINITY;
            let mut y_max = f64::NEG_INFINITY;
            for (_, line) in &mesh {
                for &(x, y) in &line.points {
                    x_min = x_min.min(x);
                    x_max = x_max.max(x);
                    y_min = y_min.min(y);
                    y_max = y_max.max(y);
                }
            }
            for seg in &frame {
                x_min = x_min.min(seg.x1).min(seg.x2);
                x_max = x_max.max(seg.x1).max(seg.x2);
                y_min = y_min.min(seg.y1).min(seg.y2);
                y_max = y_max.max(seg.y1).max(seg.y2);
            }
            if !(x_min.is_finite() && x_max.is_finite() && y_min.is_finite() && y_max.is_finite())
                || x_max - x_min < 1e-9
                || y_max - y_min < 1e-9
            {
                return None;
            }
            let pad = (x_max - x_min).max(y_max - y_min) * 0.06;
            let (wx, wy, ww, wh) =
                zoom_window(x_min - pad, x_max + pad, y_min - pad, y_max + pad, view);
            format!("{wx:.3} {wy:.3} {ww:.3} {wh:.3}")
        }
    };
    let mut parts = String::new();
    // Painter's order: the mesh runs are sorted far-to-near, so drawing
    // in order lets nearer lines overpaint farther ones.
    for (i, line) in &mesh {
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
        // Every scene element wears its own palette colour (the curve-*
        // classes both the live pane and the export stylesheet colour),
        // and the width is a fixed px count (vector-effect): the slider
        // holds screen width, so zoom and letterboxing never fatten a
        // line. The multiplier turns the default 0.2 into 2 px.
        parts.push_str(&format!(
            "<polyline class=\"curve curve-{i}\" points=\"{points}\" fill=\"none\" stroke-opacity=\"{opacity:.3}\" stroke-width=\"{:.2}\" vector-effect=\"non-scaling-stroke\"/>",
            THREE_D_PX_PER_WIDTH * stroke_width
        ));
    }
    for seg in &frame {
        parts.push_str(&format!(
            "<line x1=\"{:.3}\" y1=\"{:.3}\" x2=\"{:.3}\" y2=\"{:.3}\" stroke=\"currentColor\" stroke-width=\"{:.2}\" stroke-opacity=\"0.9\" vector-effect=\"non-scaling-stroke\"/>",
            seg.x1, seg.y1, seg.x2, seg.y2,
            THREE_D_PX_PER_WIDTH * stroke_width * 1.2
        ));
    }
    Some((view_box, parts))
}

/// Render a 3D surface set as a self-contained SVG document: the mesh and
/// frame of [`surface_parts`], letterboxed into the same 640×400 canvas
/// the 2D plot uses, on a transparent background. `stroke_width` is the
/// width slider's value: every line renders at
/// `THREE_D_PX_PER_WIDTH × stroke_width` screen px
/// (`vector-effect="non-scaling-stroke"`), so an export matches the
/// pane no matter the scene's letterbox scale. `None` when nothing can
/// be drawn.
pub fn graph3d_svg(surfaces: &[Surface], view: &View3D, stroke_width: f64) -> Option<String> {
    let (view_box, parts) = surface_parts(surfaces, view, stroke_width)?;
    letterboxed_3d_svg(&view_box, &parts, stroke_width, SvgPalette::Dark, &[])
}

/// Render a 3D curve set as a self-contained SVG document: the same
/// letterboxed canvas [`graph3d_svg`] uses (ADR-0054), with the same
/// transparent background and px line widths.
pub fn graph3d_curve_svg(
    curves: &[crate::graph::SpaceCurve],
    view: &View3D,
    stroke_width: f64,
) -> Option<String> {
    let (view_box, parts) = curve_parts(curves, view, stroke_width)?;
    letterboxed_3d_svg(&view_box, &parts, stroke_width, SvgPalette::Dark, &[])
}

/// [`graph3d_svg`] with explicit palette indices (ADR-0015 amendment):
/// the web pane filters hidden surfaces out of the slice but each keeps
/// its own colour, so a hidden neighbour never shifts the rest.
pub fn graph3d_svg_indexed(
    surfaces: &[(usize, Surface)],
    view: &View3D,
    stroke_width: f64,
) -> Option<String> {
    graph3d_svg_styled(surfaces, view, stroke_width, SvgPalette::Dark, &[])
}

/// [`graph3d_svg_indexed`] wearing the app's theme palette and
/// carrying the pane's legend entries (ADR-0057).
pub fn graph3d_svg_styled(
    surfaces: &[(usize, Surface)],
    view: &View3D,
    stroke_width: f64,
    palette: SvgPalette,
    legend: &[LegendEntry],
) -> Option<String> {
    let (view_box, parts) = scene_parts_indexed(surfaces, &[], view, stroke_width)?;
    letterboxed_3d_svg(&view_box, &parts, stroke_width, palette, legend)
}

/// [`graph3d_curve_svg`] with explicit palette indices: the space-curve
/// sibling of [`graph3d_svg_indexed`].
pub fn graph3d_curve_svg_indexed(
    curves: &[(usize, crate::graph::SpaceCurve)],
    view: &View3D,
    stroke_width: f64,
) -> Option<String> {
    graph3d_curve_svg_styled(curves, view, stroke_width, SvgPalette::Dark, &[])
}

/// [`graph3d_curve_svg_indexed`] wearing the app's theme palette and
/// carrying the pane's legend entries (ADR-0057).
pub fn graph3d_curve_svg_styled(
    curves: &[(usize, crate::graph::SpaceCurve)],
    view: &View3D,
    stroke_width: f64,
    palette: SvgPalette,
    legend: &[LegendEntry],
) -> Option<String> {
    let (view_box, parts) = scene_parts_indexed(&[], curves, view, stroke_width)?;
    letterboxed_3d_svg(&view_box, &parts, stroke_width, palette, legend)
}

/// Letterbox a scene's content box into the WIDTH×HEIGHT canvas, the
/// way preserveAspectRatio="xMidYMid meet" would.
fn letterboxed_3d_svg(
    view_box: &str,
    parts: &str,
    stroke_width: f64,
    palette: SvgPalette,
    legend: &[LegendEntry],
) -> Option<String> {
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
    let (band, legend_svg) = legend_band(legend, palette);
    let height = HEIGHT + band;
    let (fg, _, _, _, _) = palette.colors();
    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg viewBox=\"0 0 {WIDTH} {height}\" width=\"{WIDTH}\" height=\"{height}\" role=\"img\" aria-label=\"3D graph\" xmlns=\"http://www.w3.org/2000/svg\">"
    ));
    svg.push_str("<title>3D graph</title>");
    svg.push_str(&style_svg(stroke_width, palette));
    svg.push_str(&format!(
        "<g stroke=\"{fg}\" transform=\"translate({tx:.3} {ty:.3}) scale({scale:.5})\">{parts}</g>"
    ));
    svg.push_str(&legend_svg);
    svg.push_str("</svg>");
    Some(svg)
}

// ===== solar3d (ADR-0037 + the ADR-0015 amendment) =====

/// Body colours for the solar system view, shared by this renderer and
/// the live legends: visible against the dark default theme at better
/// than 3:1 (ADR-0009), one hue per body, the Sun and Moon distinct.
pub fn solar_color(body: i64) -> &'static str {
    crate::astro::body_color(body)
}

/// The scene as SVG parts: every orbit and trail as a depth-sorted
/// polyline (painter's order, far-to-near), then the positioned dots on
/// top with depth-scaled radii and `<title>` accessible names (the
/// ADR-0015 amendment's labelled points). Returns the view box and the
/// part markup, the same contract as [`surface_parts`].
pub fn solar_parts(
    scene: &crate::astro::SolarScene,
    view: &View3D,
    stroke_width: f64,
) -> Option<(String, String)> {
    let view_box = solar_view_box(scene, view)?;
    solar_parts_in(scene, view, stroke_width, &view_box)
}

/// The scene's viewBox - the projection's extent with the 6% margin -
/// computed from **every** body. The pane's legend (ADR-0038) renders a
/// filtered scene inside this frame: hiding a body must never let the
/// remaining geometry jump, rescale, or collapse the view.
pub fn solar_view_box(scene: &crate::astro::SolarScene, view: &View3D) -> Option<String> {
    // The bounding sphere around the origin (ADR-0041): orbits and trails
    // are fixed world curves and the dots ride on them, so the frame holds
    // still while the system rotates or animates.
    let world = scene
        .orbits
        .iter()
        .chain(scene.trails.iter())
        .flat_map(|p| p.points.iter().copied())
        .chain(scene.dots.iter().map(|d| d.xyz));
    if let Some((wx, wy, ww, wh)) = crate::graph::stable_window(world, view) {
        return Some(format!("{wx:.3} {wy:.3} {ww:.3} {wh:.3}"));
    }
    use crate::graph::{project_space_curve, project_world_dot};
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    let mut consider = |x: f64, y: f64| {
        x_min = x_min.min(x);
        x_max = x_max.max(x);
        y_min = y_min.min(y);
        y_max = y_max.max(y);
    };
    for path in scene.orbits.iter().chain(scene.trails.iter()) {
        for run in project_space_curve(&path.points, view) {
            for &(x, y) in &run.points {
                consider(x, y);
            }
        }
    }
    for dot in &scene.dots {
        if let Some((x, y, _zp)) = project_world_dot(dot.xyz[0], dot.xyz[1], dot.xyz[2], view) {
            consider(x, y);
        }
    }
    if !(x_min.is_finite() && x_max.is_finite() && y_min.is_finite() && y_max.is_finite())
        || x_max - x_min < 1e-9
        || y_max - y_min < 1e-9
    {
        return None;
    }
    let pad = (x_max - x_min).max(y_max - y_min) * 0.06;
    let (wx, wy, ww, wh) = zoom_window(x_min - pad, x_max + pad, y_min - pad, y_max + pad, view);
    Some(format!("{wx:.3} {wy:.3} {ww:.3} {wh:.3}"))
}

/// Draw the scene inside an explicit viewBox (ADR-0038): the pane's
/// legend filters the bodies but keeps the full scene's frame, so the
/// view never jumps when a body is hidden.
pub fn solar_parts_in(
    scene: &crate::astro::SolarScene,
    view: &View3D,
    stroke_width: f64,
    view_box: &str,
) -> Option<(String, String)> {
    use crate::graph::{project_space_curve, project_world_dot};
    struct Line {
        points: Vec<(f64, f64)>,
        depth: f64,
        color: &'static str,
    }
    let mut lines: Vec<Line> = Vec::new();
    for path in scene.orbits.iter().chain(scene.trails.iter()) {
        for run in project_space_curve(&path.points, view) {
            lines.push(Line {
                points: run.points,
                depth: run.depth,
                color: solar_color(path.body),
            });
        }
    }
    if lines.is_empty() {
        return None;
    }
    // Painter's order across every body: far lines first.
    lines.sort_by(|a, b| b.depth.total_cmp(&a.depth));
    let depths: Vec<f64> = lines.iter().map(|l| l.depth).collect();
    let z_min = depths.iter().cloned().fold(f64::INFINITY, f64::min);
    let z_max = depths.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let span = z_max - z_min;
    let mut parts = String::new();
    for line in &lines {
        let t = if span < 1e-9 {
            1.0
        } else {
            ((line.depth - z_min) / span).clamp(0.0, 1.0)
        };
        let opacity = 0.4 + 0.55 * t;
        let points = line
            .points
            .iter()
            .map(|(x, y)| format!("{x:.3},{y:.3}"))
            .collect::<Vec<_>>()
            .join(" ");
        parts.push_str(&format!(
            "<polyline points=\"{points}\" fill=\"none\" stroke=\"{}\" stroke-opacity=\"{opacity:.3}\" stroke-width=\"{:.2}\" vector-effect=\"non-scaling-stroke\"/>",
            line.color,
            THREE_D_PX_PER_WIDTH * stroke_width
        ));
    }
    // Positioned dots on top, each with an accessible name. The radius
    // is fixed in projected units, so it scales exactly with the view
    // window: zoom never changes a dot relative to the geometry around
    // it (the ADR-0015 orthographic amendment).
    let mut dots_out = String::new();
    for dot in &scene.dots {
        if let Some((x, y, _zp)) = project_world_dot(dot.xyz[0], dot.xyz[1], dot.xyz[2], view) {
            let radius = 3.0_f64;
            dots_out.push_str(&format!(
                "<circle cx=\"{x:.3}\" cy=\"{y:.3}\" r=\"{radius:.2}\" fill=\"{}\"><title>{}</title></circle>",
                solar_color(dot.body),
                crate::astro::body_name(dot.body),
            ));
        }
    }
    parts.push_str(&dots_out);
    Some((view_box.to_string(), parts))
}

/// The `solar3d` scene as a self-contained SVG document - the same
/// letterboxed 640×400 skeleton as [`graph3d_svg`], transparent
/// background and px orbit widths included.
pub fn solar3d_svg(
    scene: &crate::astro::SolarScene,
    view: &View3D,
    stroke_width: f64,
) -> Option<String> {
    solar3d_styled(scene, view, stroke_width, SvgPalette::Dark, &[])
}

/// [`solar3d_svg`] wearing the app's theme palette and carrying the
/// pane's legend entries (ADR-0057).
pub fn solar3d_styled(
    scene: &crate::astro::SolarScene,
    view: &View3D,
    stroke_width: f64,
    palette: SvgPalette,
    legend: &[LegendEntry],
) -> Option<String> {
    let (view_box, parts) = solar_parts(scene, view, stroke_width)?;
    let mut it = view_box.split_whitespace();
    let (mut x, mut y, mut w, mut h) = (0.0f64, 0.0f64, 1.0f64, 1.0f64);
    if let (Some(a), Some(b), Some(c), Some(d)) = (it.next(), it.next(), it.next(), it.next()) {
        if let (Ok(a), Ok(b), Ok(c), Ok(d)) = (a.parse(), b.parse(), c.parse(), d.parse()) {
            (x, y, w, h) = (a, b, c, d);
        }
    }
    let scale = (WIDTH / w).min(HEIGHT / h);
    let tx = (WIDTH - w * scale) / 2.0 - x * scale;
    let ty = (HEIGHT - h * scale) / 2.0 - y * scale;
    let names: Vec<&str> = scene
        .dots
        .iter()
        .map(|d| crate::astro::body_name(d.body))
        .collect();
    let (band, legend_svg) = legend_band(legend, palette);
    let height = HEIGHT + band;
    let mut svg = String::new();
    svg.push_str(&format!(
        "<svg viewBox=\"0 0 {WIDTH} {height}\" width=\"{WIDTH}\" height=\"{height}\" role=\"img\" aria-label=\"Solar system: {}\" xmlns=\"http://www.w3.org/2000/svg\">",
        names.join(", ")
    ));
    svg.push_str("<title>Solar system</title>");
    svg.push_str(&style_svg(stroke_width, palette));
    svg.push_str(&format!(
        "<g transform=\"translate({tx:.3} {ty:.3}) scale({scale:.5})\">{parts}</g>"
    ));
    svg.push_str(&legend_svg);
    svg.push_str("</svg>");
    Some(svg)
}
