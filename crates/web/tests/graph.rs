//! Pure tests for the web graph renderer (ADR-0006/0014: the core samples,
//! each frontend renders). SVG as a string so the tests run natively — no
//! browser needed.

use epher_core::graph::{parse_graph_source, sample_spec, CurveKind, InterestKind, SampledCurve};
use epher_core::{Env, Sample};
use epher_web::graph::{
    geometry, graph_svg, segments, ticks, trace_nearest, Poi, TracePoint, DEFAULT_STROKE_WIDTH,
};
use epher_web::{anchored_window, slider_window, zoom_slider_value};

fn env() -> Env {
    Env::default()
}

fn curve(source: &str) -> SampledCurve {
    let spec = parse_graph_source(source).unwrap();
    let samples = sample_spec(&spec, 120, &env()).unwrap();
    SampledCurve {
        source: source.to_string(),
        kind: spec.kind,
        domain: spec.domain,
        samples,
        fill: spec.fill,
    }
}

fn samples_of(ys: &[f64]) -> Vec<Sample> {
    ys.iter()
        .enumerate()
        .map(|(i, y)| Sample {
            x: -10.0 + i as f64 * (20.0 / (ys.len() as f64 - 1.0)),
            y: *y,
        })
        .collect()
}

#[test]
fn empty_curves_render_nothing() {
    assert_eq!(graph_svg(&[], &[], None, true, DEFAULT_STROKE_WIDTH), "");
}

#[test]
fn a_line_maps_the_domain_onto_the_plot_area() {
    // y = x sampled from -10 to 10: with 6% y padding the curve still runs
    // corner to corner (the padding lifts it 6% off the plot edges).
    let svg = graph_svg(&[curve("x")], &[], None, true, DEFAULT_STROKE_WIDTH);
    assert!(svg.contains("viewBox=\"0 0 640 400\""));
    assert!(svg.contains("48.0,348.9"), "{svg}");
    assert!(svg.contains("632.0,31.1"), "{svg}");
}

#[test]
fn geometry_uses_the_union_of_domains_and_padded_y_range() {
    let curves = vec![curve("x from -5 to 5"), curve("x ^ 2")];
    let g = geometry(&curves).unwrap();
    assert_eq!(g.x_min, -10.0);
    assert_eq!(g.x_max, 10.0);
    // y range covers the parabola (0..=100) plus 6% padding of the whole
    // span on each side.
    assert!(g.y_min < -10.0 && g.y_min > -13.0, "y_min {}", g.y_min);
    assert!(g.y_max > 105.0 && g.y_max < 112.0, "y_max {}", g.y_max);
    assert!(g.zero_axis);
}

#[test]
fn jump_splitting_breaks_asymptote_branches() {
    // 1 / x: the two branches must never be joined by a vertical line.
    let s = segments(&samples_of(&[-20.0, -10.0, f64::NAN, 10.0, 20.0]), 40.0);
    assert_eq!(
        s.len(),
        2,
        "both branches survive as separate segments: {s:?}"
    );
    assert_eq!(s[0].len(), 2);
    assert_eq!(s[1].len(), 2);

    // A finite but huge jump (tan-style) also splits.
    let s = segments(&samples_of(&[1.0, 2.0, -1000.0, 900.0]), 2000.0);
    assert_eq!(s.len(), 2);

    // A steep but continuous line stays in one piece.
    let s = segments(&samples_of(&[0.0, 100.0, 200.0]), 2000.0);
    assert_eq!(s.len(), 1);
}

#[test]
fn ticks_land_on_nice_steps_and_snap_zero() {
    // Exact binary steps land on exact ticks, with the zero tick snapped.
    assert_eq!(ticks(-1.0, 1.0, 0.5), vec![-1.0, -0.5, 0.0, 0.5, 1.0]);
    assert_eq!(ticks(-0.5, 0.5, 0.25), vec![-0.5, -0.25, 0.0, 0.25, 0.5]);
    // Even when float drift drops the boundary value, the middle of the
    // range is intact and zero stays exactly 0.
    let t = ticks(-0.3, 0.3, 0.1);
    assert_eq!(t.len(), 5);
    assert_eq!(t[2], 0.0);
}

#[test]
fn fills_emit_polygons_closed_against_the_plot_edge() {
    let svg = graph_svg(
        &[curve("y < x ^ 2 from -2 to 2")],
        &[],
        None,
        true,
        DEFAULT_STROKE_WIDTH,
    );
    assert!(svg.contains("<polygon class=\"fill curve-0\""), "{svg}");
    assert!(svg.contains("368.0"), "closed against the bottom edge");
}

#[test]
fn curves_get_distinct_classes_for_color_and_dash() {
    let svg = graph_svg(
        &[curve("x"), curve("x ^ 2")],
        &[],
        None,
        true,
        DEFAULT_STROKE_WIDTH,
    );
    assert!(svg.contains("class=\"curve curve-0\""), "{svg}");
    assert!(svg.contains("class=\"curve curve-1\""), "{svg}");
}

#[test]
fn points_of_interest_render_with_labels() {
    let pois = vec![Poi {
        kind: InterestKind::Root,
        label: "root".to_string(),
        x: 1.0,
        y: 0.0,
        curve: 0,
    }];
    let svg = graph_svg(
        &[curve("x ^ 2 - 1")],
        &pois,
        None,
        true,
        DEFAULT_STROKE_WIDTH,
    );
    assert!(svg.contains("class=\"poi\""), "{svg}");
    assert!(svg.contains("root (1, 0)"), "{svg}");
}

#[test]
fn points_of_interest_hide_when_markers_are_off() {
    let pois = vec![Poi {
        kind: InterestKind::Root,
        label: "root".to_string(),
        x: 1.0,
        y: 0.0,
        curve: 0,
    }];
    let svg = graph_svg(
        &[curve("x ^ 2 - 1")],
        &pois,
        None,
        false,
        DEFAULT_STROKE_WIDTH,
    );
    assert!(!svg.contains("class=\"poi\""), "{svg}");
    assert!(!svg.contains("class=\"poi-label\""), "{svg}");
    assert!(
        svg.contains("class=\"curve curve-0\""),
        "the plot itself stays"
    );
}

#[test]
fn trace_finds_the_nearest_sample_within_radius() {
    let curves = vec![curve("x")];
    let g = geometry(&curves).unwrap();
    let t = trace_nearest(&curves, &g, g.sx(2.0), g.sy(2.0));
    let Some(TracePoint { curve, index, x, y }) = t else {
        panic!("expected a trace point");
    };
    assert_eq!(curve, 0);
    assert!(index > 0);
    assert!((x - 2.0).abs() < 0.2, "nearest sample x ≈ 2, got {x}");
    assert!((y - 2.0).abs() < 0.2);

    // Far from any curve: nothing.
    assert!(trace_nearest(&curves, &g, 640.0, 400.0).is_none());
}

#[test]
fn trace_render_includes_the_cursor() {
    let t = TracePoint {
        curve: 0,
        index: 60,
        x: 0.0,
        y: 0.0,
    };
    let svg = graph_svg(&[curve("x")], &[], Some(t), true, DEFAULT_STROKE_WIDTH);
    assert!(svg.contains("class=\"trace\""), "{svg}");
}

#[test]
fn parametric_and_polar_curves_share_the_plot() {
    let svg = graph_svg(
        &[curve("param t, t ^ 2"), curve("polar 2")],
        &[],
        None,
        true,
        DEFAULT_STROKE_WIDTH,
    );
    assert!(svg.contains("class=\"curve curve-0\""), "{svg}");
    assert!(svg.contains("class=\"curve curve-1\""), "{svg}");
}

#[test]
fn legend_captions_keep_the_y_prefix_for_cartesian() {
    let c = curve("x ^ 2");
    assert_eq!(epher_web::graph::curve_caption(&c), "y = x ^ 2");
    let p = curve("polar 2");
    assert_eq!(epher_web::graph::curve_caption(&p), "polar 2");
    assert!(matches!(p.kind, CurveKind::Polar(_)));
}

#[test]
fn trace_at_center_of_sin_finds_the_zero_crossing() {
    let c = curve("sin(x)");
    let geom = geometry(&[c.clone()]).unwrap();
    // The viewBox is 640x400; sin(0)=0 passes exactly through the center.
    let found = trace_nearest(&[c], &geom, 320.0, 200.0);
    assert!(found.is_some(), "center of sin(x) must trace");
    let t = found.unwrap();
    assert!(t.x.abs() < 0.1, "x = {}", t.x);
    assert!(t.y.abs() < 0.1, "y = {}", t.y);
}

// ===== zoom (ADR-0038): the slider's two-decade range and the
// anchor-stable gesture windows =====

#[test]
fn the_zoom_slider_spans_two_decades_each_way() {
    let base = (-10.0, 10.0);
    // 0 keeps the fit window; -1 widens 100x (every object fits),
    // +1 narrows 100x (a single object fills the pane).
    let mid = slider_window(0.0, base, 0.0);
    assert!((mid.1 - mid.0 - 20.0).abs() < 1e-9);
    let out = slider_window(-1.0, base, 0.0);
    assert!((out.1 - out.0 - 2000.0).abs() < 1e-9);
    let in_ = slider_window(1.0, base, 0.0);
    assert!((in_.1 - in_.0 - 0.2).abs() < 1e-9);
    // The slider reads back the same value from its own window.
    assert!((zoom_slider_value(Some(out), base) - (-1.0)).abs() < 1e-9);
    assert!((zoom_slider_value(Some(in_), base) - 1.0).abs() < 1e-9);
    assert!((zoom_slider_value(Some(mid), base)).abs() < 1e-9);
    // No window: the slider sits at the fit position, 0.
    assert_eq!(zoom_slider_value(None, base), 0.0);
    // Values past the ends pin at the slider's ends.
    assert_eq!(zoom_slider_value(Some((-1.0e5, 1.0e5)), base), -1.0);
}

#[test]
fn gesture_windows_keep_the_anchor_still() {
    let base = (-10.0, 10.0);
    let cur = (-10.0, 10.0);
    // Zoom in 4x around x = 5: the anchor's share of the window holds.
    let win = anchored_window(cur, 5.0, 0.25, base.1 - base.0);
    assert!((win.1 - win.0 - 5.0).abs() < 1e-9);
    assert!((5.0 - win.0) / 5.0 - (5.0 - cur.0) / 20.0 < 1e-9);
    // Zooming out around the center recenters on the anchor.
    let wide = anchored_window(cur, 0.0, 4.0, base.1 - base.0);
    assert!((wide.1 - wide.0 - 80.0).abs() < 1e-9);
    // One event zooms at most 5x in or out (a wheel notch or a pinch
    // step); repeated steps keep the journey going until the clamp
    // takes over at nine decades from the base window.
    let mut deep = cur;
    for _ in 0..200 {
        deep = anchored_window(deep, 0.0, 0.25, base.1 - base.0);
    }
    assert!(((deep.1 - deep.0) / (20.0 * 1e-9) - 1.0).abs() < 1e-6);
    let mut far = cur;
    for _ in 0..200 {
        far = anchored_window(far, 0.0, 4.0, base.1 - base.0);
    }
    assert!(((far.1 - far.0) / (20.0 * 1e9) - 1.0).abs() < 1e-6);
}

#[test]
fn answer_fits_routes_short_single_answers_to_the_answer_line() {
    // The headless desktop width (>880px): one short answer, no line
    // breaks, fits the answer line (ADR-0056).
    assert!(epher_web::answer_fits_at("= 2", false));
    assert!(epher_web::answer_fits_at(
        "= [[1, 2, 3], [4, 5, 6], [7, 8, 10]]",
        false
    ));
    assert!(epher_web::answer_fits_at(
        "= they agree within 1e-12: true",
        false
    ));
    // A script transcript (several answers) renders in the result pane.
    assert!(!epher_web::answer_fits_at("= 2\u{1f}= 6", false));
    // The same rule on a phone: a shorter cap, same decisions otherwise.
    assert!(epher_web::answer_fits_at("= 0.5", true));
    assert!(!epher_web::answer_fits_at(
        "= [[1, 2, 3], [4, 5, 6], [7, 8, 10]]",
        true
    ));
    // So does a table or matrix with its own line breaks.
    assert!(!epher_web::answer_fits_at("x  y\n1  2", false));
    // So does an answer too long for one calm line.
    let long = format!("= {}", "1234567890".repeat(5));
    assert!(!epher_web::answer_fits_at(&long, false));
    // Empty keeps whatever routing the absence implies (the answer line
    // renders nothing either way).
    assert!(!epher_web::answer_fits_at("", false));
}

#[test]
fn three_d_width_contract_follows_the_layout() {
    // The touch layout keeps ADR-0035's original slider and default
    // (0-0.2 step 0.01, 0.1); the desktop keeps ADR-0055's revision
    // (0-0.4 step 0.05, 0.2). Both render at 10 px per unit of width
    // (non-scaling stroke), so the defaults draw 1 px and 2 px.
    assert_eq!(
        epher_core::graph_svg::three_d_width_range(true),
        (0.2, 0.01)
    );
    assert_eq!(
        epher_core::graph_svg::three_d_width_range(false),
        (0.4, 0.05)
    );
    assert_eq!(
        epher_core::graph_svg::three_d_default_width(true),
        epher_core::graph_svg::THREE_D_DEFAULT_WIDTH_MOBILE
    );
    assert_eq!(
        epher_core::graph_svg::three_d_default_width(false),
        epher_core::graph_svg::THREE_D_DEFAULT_WIDTH
    );
    assert_eq!(
        epher_core::graph_svg::three_d_default_width(true)
            * epher_core::graph_svg::THREE_D_PX_PER_WIDTH,
        1.0
    );
    assert_eq!(
        epher_core::graph_svg::three_d_default_width(false)
            * epher_core::graph_svg::THREE_D_PX_PER_WIDTH,
        2.0
    );
}
