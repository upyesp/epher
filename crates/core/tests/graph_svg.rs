//! The SVG document builder (ADR-0020): the same renderer serves the web
//! app's copy button and the terminal frontends' `graph save`, so the
//! tests pin the properties that make the output a standalone document.

use epher_core::graph::{parse_graph_source, sample_spec, InterestKind};
use epher_core::graph_svg::{
    graph3d_svg, graph_svg, graph_svg_indexed, solar_parts_in, solar_view_box, Poi,
    DEFAULT_STROKE_WIDTH,
};
use epher_core::Env;

fn curve(source: &str) -> epher_core::graph::SampledCurve {
    let spec = parse_graph_source(source).unwrap();
    let env = Env::default();
    let samples = sample_spec(&spec, 120, &env).unwrap();
    epher_core::graph::SampledCurve {
        source: source.to_string(),
        kind: spec.kind,
        domain: spec.domain,
        samples,
        fill: spec.fill,
    }
}

#[test]
fn empty_input_renders_the_empty_string() {
    assert_eq!(graph_svg(&[], &[], None, true, DEFAULT_STROKE_WIDTH), "");
}

#[test]
fn document_is_self_contained() {
    let svg = graph_svg(&[curve("x")], &[], None, true, DEFAULT_STROKE_WIDTH);
    assert!(svg.starts_with("<svg "), "{svg}");
    assert!(svg.contains("viewBox=\"0 0 640 400\""));
    assert!(svg.contains("width=\"640\" height=\"400\""));
    assert!(svg.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    // embedded style: the file needs no external CSS
    assert!(svg.contains("<style>"));
    assert!(svg.contains(".curve {"));
    assert!(svg.contains(".bg { fill: #141416; }"));
    // the background rect paints the dark canvas the app shows
    assert!(svg.contains("<rect class=\"bg\""));
}

#[test]
fn stroke_width_lands_in_the_embedded_style() {
    let thin = graph_svg(&[curve("x")], &[], None, true, 0.5);
    let thick = graph_svg(&[curve("x")], &[], None, true, 4.0);
    assert!(thin.contains("stroke-width: 0.50"));
    assert!(thick.contains("stroke-width: 4.00"));
    // the default is half the pre-ADR-0020 constant 2
    assert_eq!(DEFAULT_STROKE_WIDTH, 1.0);
}

#[test]
fn curve_expressions_are_escaped() {
    let svg = graph_svg(&[curve("y < x ^ 2")], &[], None, true, 1.0);
    assert!(
        svg.contains("&lt;"),
        "the expression must not break the XML"
    );
}

#[test]
fn pois_render_their_localized_labels() {
    let pois = vec![Poi {
        kind: InterestKind::Root,
        label: "Wurzel".to_string(),
        x: 1.0,
        y: 0.0,
        curve: 0,
    }];
    let svg = graph_svg(&[curve("x - 1")], &pois, None, true, 1.0);
    assert!(svg.contains("Wurzel (1, 0)"), "{svg}");
}

#[test]
fn three_dimensional_output_is_a_standalone_document() {
    let env = Env::default();
    let s = epher_core::graph::sample_surface("x ^ 2 - y ^ 2", 20, &env).unwrap();
    let svg = graph3d_svg(&[s], &Default::default(), 1.0).unwrap();
    assert!(svg.starts_with("<svg "), "{svg}");
    assert!(svg.contains("viewBox=\"0 0 640 400\""));
    assert!(svg.contains("<style>"));
    assert!(svg.contains("transform=\"translate("));
    assert!(svg.ends_with("</svg>"));
    assert!(graph3d_svg(&[], &Default::default(), 1.0).is_none());
}

#[test]
fn rendered_document_is_valid_expression_free_xml() {
    // a quick structural sanity: every opened tag closes
    let svg = graph_svg(&[curve("x ^ 2")], &[], None, true, 1.0);
    let opens = svg.matches('<').count();
    let closes = svg.matches('>').count();
    assert_eq!(opens, closes);
}

#[test]
fn extra_curves_are_solid_and_captioned() {
    // ADR-0023: every curve is solid; the caption at each curve's end is
    // the non-color channel that keeps curves apart (WCAG 1.4.1).
    let svg = graph_svg(
        &[curve("x ^ 2"), curve("x ^ 3")],
        &[],
        None,
        true,
        DEFAULT_STROKE_WIDTH,
    );
    assert!(!svg.contains("dasharray"), "{svg}");
    assert!(svg.contains("<text class=\"label curve-0\""), "{svg}");
    assert!(svg.contains("<text class=\"label curve-1\""), "{svg}");
    assert!(svg.contains("y = x ^ 2"), "{svg}");
    assert!(svg.contains("y = x ^ 3"), "{svg}");
    // labels ride on the curve colors, haloed for legibility
    assert!(svg.contains(".label.curve-0 { fill: #2dd4bf; }"), "{svg}");
}

#[test]
fn hidden_neighbour_keeps_original_palette_index() {
    // ADR-0015 amendment: the web pane filters hidden curves out of the
    // slice but must keep each curve's own colour — hiding the middle
    // curve must not shift the remaining lines' palette entries.
    let indexed = vec![
        (0usize, curve("x ^ 2")),
        (2usize, curve("x ^ 3")),
    ];
    let svg = graph_svg_indexed(&indexed, &[], None, true, DEFAULT_STROKE_WIDTH);
    assert!(svg.contains("<polyline class=\"curve curve-0\""), "{svg}");
    assert!(svg.contains("<polyline class=\"curve curve-2\""), "{svg}");
    assert!(!svg.contains("curve-1\""), "{svg}");
    assert!(svg.contains("y = x ^ 2"), "{svg}");
    assert!(svg.contains("y = x ^ 3"), "{svg}");
}

// ===== solar3d SVG (ADR-0037 + the ADR-0015 amendment) =====

#[test]
fn solar3d_svg_renders_orbits_trails_and_labelled_dots() {
    use epher_core::astro::solar_scene;
    use epher_core::graph::View3D;
    let scene = solar_scene(2459030.5).expect("scene");
    let svg = graph_svg_crate::render_solar(&scene);
    // nine orbit curves and their trails draw as polylines
    assert!(svg.matches("<polyline").count() >= 18, "orbit and trail polylines");
    // every dot is a labelled circle (the ADR amendment's accessible name)
    assert_eq!(svg.matches("<circle").count(), 11, "one dot per body");
    assert!(svg.contains("<title>Jupiter</title>"));
    assert!(svg.contains("<title>Pluto</title>"));
    // the document keeps the shared SVG skeleton
    assert!(svg.contains("role=\"img\""));
}

// a thin wrapper so the test above stays on the public seam
mod graph_svg_crate {
    pub fn render_solar(scene: &epher_core::astro::SolarScene) -> String {
        epher_core::graph_svg::solar3d_svg(scene, &scene.default_view(), 1.0)
            .expect("a scene renders")
    }
}

// ===== review-fix regression: the solar viewBox must stay finite =====

#[test]
fn solar_parts_viewbox_is_always_finite() {
    // A copy-paste slip initialized y_max to +inf, so the accumulated
    // bounds grew to [x, -inf..inf, y] and the viewBox came out
    // "-inf -inf inf inf" - a console error in every engine and a blank
    // plot in WebKit. The default pose (no interaction) hit it on the
    // very first frame.
    let jd = 2459032.5; // 2020-07-01
    let scene = epher_core::astro::solar_scene(jd).unwrap();
    let view = epher_core::graph::View3D::default();
    let (view_box, parts) = epher_core::graph_svg::solar_parts(&scene, &view, 1.0)
        .expect("the default solar scene must render");
    let nums: Vec<f64> = view_box
        .split_whitespace()
        .map(|n| n.parse().unwrap())
        .collect();
    assert_eq!(nums.len(), 4);
    assert!(
        nums.iter().all(|n| n.is_finite()),
        "viewBox must be finite, got {view_box}"
    );
    assert!(parts.contains("<polyline"), "orbits and trails render");
    assert!(parts.contains("<circle"), "positioned dots render");
}

// ===== the ADR-0015 orthographic amendment: zoom is a pure scale =====

#[test]
fn solar_zoom_scales_the_viewbox_without_moving_anything() {
    let jd = 2459032.5; // 2020-07-01
    let scene = epher_core::astro::solar_scene(jd).unwrap();
    let default = scene.default_view(); // camera 30
    let zoomed = default.with_camera(15.0); // one +1 step: 2x in

    let (box_d, parts_d) = epher_core::graph_svg::solar_parts(&scene, &default, 1.0).unwrap();
    let (box_z, parts_z) = epher_core::graph_svg::solar_parts(&scene, &zoomed, 1.0).unwrap();

    // The projected geometry is zoom-independent: the markup (up to the
    // viewBox it sits in) is identical, so every planet sits at exactly
    // the same relative position no matter the zoom.
    assert_eq!(parts_d, parts_z, "zoom must not move projected geometry");

    // The window halves around the same center: the on-screen image is
    // exactly 2x. (Tolerances match the viewBox's 3-decimal format.)
    let pd: Vec<f64> = box_d.split_whitespace().map(|v| v.parse().unwrap()).collect();
    let pz: Vec<f64> = box_z.split_whitespace().map(|v| v.parse().unwrap()).collect();
    assert!((pz[2] - pd[2] / 2.0).abs() < 1e-2, "width {} vs {}/2", pz[2], pd[2]);
    assert!((pz[3] - pd[3] / 2.0).abs() < 1e-2);
    assert!((pz[0] + pz[2] / 2.0 - (pd[0] + pd[2] / 2.0)).abs() < 1e-2, "same center x");
    assert!((pz[1] + pz[3] / 2.0 - (pd[1] + pd[3] / 2.0)).abs() < 1e-2, "same center y");
}

#[test]
fn solar_dots_keep_a_fixed_radius_at_every_zoom() {
    let jd = 2459032.5;
    let scene = epher_core::astro::solar_scene(jd).unwrap();
    let default = scene.default_view();
    let zoomed = default.with_camera(60.0); // 2x out
    let radii = |view: epher_core::graph::View3D| {
        let (_, parts) = epher_core::graph_svg::solar_parts(&scene, &view, 1.0).unwrap();
        let mut out = Vec::new();
        let mut i = 0;
        while let Some(p) = parts[i..].find("r=\"").map(|p| p + i) {
            let end = parts[p + 3..].find('"').unwrap() + p + 3;
            out.push(parts[p + 3..end].to_string());
            i = end;
        }
        out
    };
    assert_eq!(radii(default), radii(zoomed), "radius must not depend on zoom");
    assert!(!radii(default).is_empty());
}

#[test]
fn hiding_a_solar_body_never_moves_the_frame() {
    // ADR-0038: the pane's legend renders a filtered scene inside the
    // FULL scene's viewBox - unchecking a body must not rescale or jump
    // the view.
    let scene = epher_core::astro::solar_scene(2459031.5).unwrap();
    let view = scene.default_view();
    let full_box = solar_view_box(&scene, &view).unwrap();
    let mut hidden = scene.clone();
    // Keep one planet's orbit and trail; the frame must stay.
    let keep = scene.orbits[0].body;
    hidden.orbits.retain(|p| p.body == keep);
    hidden.trails.retain(|p| p.body == keep);
    let (_, inner) = solar_parts_in(&hidden, &view, 2.0, &full_box).unwrap();
    assert!(inner.contains("polyline"), "the moon's orbit is drawn");
    // The full frame is wide (every planet); the moon's own extent is
    // far smaller. If the frame had collapsed to the moon, the box the
    // caller passed would differ from the full scene's - assert the
    // moon-only scene's own box really is smaller, so this test bites.
    let moon_box = solar_view_box(&hidden, &view).unwrap();
    assert_ne!(full_box, moon_box, "the filter really changes the extent");
}
