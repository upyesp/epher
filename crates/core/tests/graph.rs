//! Tests for the graph command grammar, analysis, tables, and tick steps
//! (ADR-0014) — pure math, no rendering.

use epher_core::graph::{
    analyze, free_names, nice_step, parse_graph_source, project_clipped, project_mesh,
    project_point, project_surface, sample_spec, sample_surface, surface_frame, table_rows,
    zoom_window, CurveKind, Fill, InterestKind, SampledCurve, View3D,
};
use epher_core::{parse, Env, Session, Value};

fn env() -> Env {
    Env::default()
}

fn float(v: &Value) -> f64 {
    match v {
        Value::Float(f) => *f,
        other => panic!("expected float, got {other:?}"),
    }
}

fn sampled(source: &str) -> SampledCurve {
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

#[test]
fn parses_plain_cartesian_with_default_domain() {
    let spec = parse_graph_source("x ^ 2").unwrap();
    assert!(matches!(spec.kind, CurveKind::Cartesian(_)));
    assert_eq!(spec.domain, (-10.0, 10.0));
    assert_eq!(spec.fill, None);
}

#[test]
fn parses_fill_prefixes() {
    assert_eq!(
        parse_graph_source("y < x ^ 2").unwrap().fill,
        Some(Fill::Below)
    );
    assert_eq!(
        parse_graph_source("y <= x ^ 2").unwrap().fill,
        Some(Fill::Below)
    );
    assert_eq!(
        parse_graph_source("y > sin(x)").unwrap().fill,
        Some(Fill::Above)
    );
    assert_eq!(
        parse_graph_source("y >= sin(x)").unwrap().fill,
        Some(Fill::Above)
    );
    assert_eq!(parse_graph_source("x + 1").unwrap().fill, None);
}

#[test]
fn parses_domain_bounds_including_expressions() {
    let spec = parse_graph_source("x ^ 2 from -5 to 5").unwrap();
    assert_eq!(spec.domain, (-5.0, 5.0));
    let spec = parse_graph_source("sin(x) from 0 to 2*pi").unwrap();
    assert!((spec.domain.0 - 0.0).abs() < 1e-12);
    assert!((spec.domain.1 - std::f64::consts::TAU).abs() < 1e-12);
}

#[test]
fn rejects_backwards_domain() {
    assert!(parse_graph_source("x from 5 to -5").is_err());
    assert!(parse_graph_source("x from 2 to 2").is_err());
}

#[test]
fn parses_parametric_with_commas_in_function_args() {
    let spec = parse_graph_source("param max(0, t), min(t, 1)").unwrap();
    assert!(matches!(spec.kind, CurveKind::Parametric { .. }));
    assert_eq!(spec.domain, (0.0, std::f64::consts::TAU));
    let spec = parse_graph_source("param t, t ^ 2 from 0 to 3").unwrap();
    assert_eq!(spec.domain, (0.0, 3.0));
}

#[test]
fn rejects_parametric_without_two_expressions() {
    assert!(parse_graph_source("param t").is_err());
    assert!(parse_graph_source("param t, t, t").is_err());
}

#[test]
fn parses_polar() {
    let spec = parse_graph_source("polar 2").unwrap();
    assert!(matches!(spec.kind, CurveKind::Polar(_)));
    assert_eq!(spec.domain, (0.0, std::f64::consts::TAU));
}

#[test]
fn samples_each_kind() {
    let spec = parse_graph_source("x ^ 2").unwrap();
    let s = sample_spec(&spec, 51, &env()).unwrap();
    assert_eq!(s.len(), 51);
    assert_eq!(s[0].x, -10.0);
    assert!((s[25].y - 0.0).abs() < 1e-9, "x=0 maps to y=0");

    let spec = parse_graph_source("param t, t").unwrap();
    let s = sample_spec(&spec, 50, &env()).unwrap();
    assert!((s[0].x - 0.0).abs() < 1e-9);
    assert!((s[0].y - 0.0).abs() < 1e-9);

    // A circle of radius 2: every sampled point lies on r = 2.
    let spec = parse_graph_source("polar 2").unwrap();
    let s = sample_spec(&spec, 64, &env()).unwrap();
    assert!(s.iter().all(|p| (p.x * p.x + p.y * p.y - 4.0).abs() < 1e-9));
}

#[test]
fn nice_steps_follow_one_two_five() {
    assert!((nice_step(10.0, 5) - 2.0).abs() < 1e-12);
    assert!((nice_step(100.0, 5) - 20.0).abs() < 1e-12);
    assert!((nice_step(1.0, 10) - 0.1).abs() < 1e-12);
    assert!((nice_step(7.0, 6) - 2.0).abs() < 1e-12);
    assert!((nice_step(0.03, 4) - 0.01).abs() < 1e-12);
}

#[test]
fn tables_keep_x_and_blank_undefined_rows() {
    let expr = parse("x ^ 2").unwrap();
    let rows = table_rows(&expr, -2.0, 2.0, 5, &env());
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[0], (-2.0, Some(4.0)));
    assert_eq!(rows[2], (0.0, Some(0.0)));

    let expr = parse("1 / x").unwrap();
    let rows = table_rows(&expr, -1.0, 1.0, 5, &env());
    assert_eq!(rows.len(), 5);
    assert_eq!(rows[2].1, None, "1/x is undefined at x=0");
    assert_eq!(rows[1].1, Some(-2.0));
    assert_eq!(rows[3].1, Some(2.0));
}

fn kinds(curves: &[SampledCurve]) -> Vec<(InterestKind, f64, f64)> {
    let pts = analyze(curves, &env());
    pts.iter().map(|p| (p.kind, p.x, p.y)).collect()
}

fn assert_point(pts: &[(InterestKind, f64, f64)], kind: InterestKind, x: f64, y: f64) {
    assert!(
        pts.iter()
            .any(|(k, px, py)| *k == kind && (px - x).abs() < 1e-4 && (py - y).abs() < 1e-4),
        "expected {kind:?} at ({x}, {y}) in {pts:?}"
    );
}

#[test]
fn finds_roots_by_sign_change() {
    let curves = [sampled("x ^ 2 - 1")];
    let pts = kinds(&curves);
    assert_point(&pts, InterestKind::Root, -1.0, 0.0);
    assert_point(&pts, InterestKind::Root, 1.0, 0.0);
}

#[test]
fn finds_extrema() {
    let curves = [sampled("-(x ^ 2) + 4")];
    let pts = kinds(&curves);
    assert_point(&pts, InterestKind::Maximum, 0.0, 4.0);

    let curves = [sampled("x ^ 2")];
    let pts = kinds(&curves);
    assert_point(&pts, InterestKind::Minimum, 0.0, 0.0);
}

#[test]
fn finds_intersections_between_curves() {
    let curves = [sampled("x ^ 2"), sampled("2 - x")];
    let pts = kinds(&curves);
    assert_point(&pts, InterestKind::Intersection, -2.0, 4.0);
    assert_point(&pts, InterestKind::Intersection, 1.0, 1.0);
}

#[test]
fn intersections_respect_domain_overlap() {
    // Second curve lives entirely left of the first: no overlap, so no
    // *intersections* (roots of the first curve may still appear).
    let curves = [
        sampled("x ^ 2 from 0 to 10"),
        sampled("x ^ 2 - 1 from -10 to -1"),
    ];
    assert!(analyze(&curves, &env())
        .iter()
        .all(|p| p.kind != InterestKind::Intersection));
}

#[test]
fn free_names_collects_variables_deeply() {
    let expr = parse("a * x ^ 2 + sin(b)").unwrap();
    let mut names = std::collections::BTreeSet::new();
    free_names(&expr, &mut names);
    assert_eq!(
        names,
        ["a", "b", "x"].iter().map(|s| s.to_string()).collect()
    );
}

#[test]
fn session_set_constant_updates_value_and_source() {
    let mut s = Session::new();
    s.submit("const a = 2");
    assert!((float(s.env().constant("a").unwrap()) - 2.0).abs() < 1e-12);
    s.set_constant("a", Value::float(3.5), "const a = 3.5".to_string());
    assert!((float(s.env().constant("a").unwrap()) - 3.5).abs() < 1e-12);
    assert_eq!(s.const_sources().get("a").unwrap(), "const a = 3.5");
}

// ===== 3D surfaces (ADR-0015) =====

#[test]
fn surface_sampling_fills_a_square_mesh() {
    let env = Env::default();
    let s = sample_surface("x ^ 2 + y ^ 2", 8, &env).unwrap();
    assert_eq!(s.xs.len(), 9);
    assert_eq!(s.ys.len(), 9);
    assert_eq!(s.zs.len(), 9);
    assert_eq!(s.zs[0].len(), 9);
    assert_eq!(s.domain, (-5.0, 5.0));
    // z = x² + y²: the corner (5, 5) is 50, the center is 0.
    assert!((s.zs[0][0] - 50.0).abs() < 1e-9);
    assert!((s.zs[4][4] - 0.0).abs() < 1e-9);
}

#[test]
fn surface_domain_and_errors() {
    let env = Env::default();
    let s = sample_surface("x * y from -2 to 2", 4, &env).unwrap();
    assert_eq!(s.domain, (-2.0, 2.0));
    assert!(sample_surface("x * y from 2 to -2", 4, &env).is_err());
    assert!(sample_surface("x * y from 3 to 3", 4, &env).is_err());
    assert!(sample_surface("", 4, &env).is_err());
}

#[test]
fn surface_drops_undefined_cells_but_keeps_the_mesh() {
    let env = Env::default();
    // Undefined at the origin: the hole must not produce segments touching
    // it, but the rest of the mesh stays.
    let s = sample_surface("1 / (x ^ 2 + y ^ 2)", 8, &env).unwrap();
    assert!(s.zs[4][4].is_nan());
    let segs = project_surface(&s, &View3D::default());
    // A full 8×8 mesh would have 8*7*2 = 112 segments; the hole removes the
    // few around the origin only.
    assert!(segs.len() > 100);
    for seg in &segs {
        assert!(seg.depth.is_finite());
    }
}

#[test]
fn projection_matches_hand_computed_views() {
    let view = View3D {
        yaw: 0.0,
        pitch: 0.0,
        camera: 12.0,
    };
    // Top-down at z = 0 (f = 1): x right, y up after the screen flip.
    let (sx, sy, _d) = project_point(2.0, 3.0, 0.0, &view);
    assert!((sx - 2.0).abs() < 1e-9);
    assert!((sy + 3.0).abs() < 1e-9);

    // Yaw 90°: the world x axis points along screen −y... verify with
    // sin/cos symmetry: rotating a point on the x axis by 90° puts it on
    // the y axis of the rotated frame.
    let yaw = View3D {
        yaw: std::f64::consts::FRAC_PI_2,
        pitch: 0.0,
        camera: 12.0,
    };
    let (sx, sy, _d) = project_point(2.0, 0.0, 0.0, &yaw);
    assert!(sx.abs() < 1e-9);
    assert!((sy + 2.0).abs() < 1e-9);
}

#[test]
fn orthographic_projection_is_depth_fair() {
    let view = View3D {
        yaw: 0.0,
        pitch: 0.0,
        camera: 12.0,
    };
    // The projection is affine (the ADR-0015 amendment): the same world
    // offset covers the same screen distance at every depth - zoom and
    // pose never distort relative positions.
    let (_, a1, _) = project_point(0.0, 1.0, 3.0, &view);
    let (_, a2, _) = project_point(0.0, 2.0, 3.0, &view);
    let (_, b1, _) = project_point(0.0, 1.0, -3.0, &view);
    let (_, b2, _) = project_point(0.0, 2.0, -3.0, &view);
    assert!(((a2 - a1) - (b2 - b1)).abs() < 1e-12);
}

#[test]
fn painter_orders_far_before_near() {
    let env = Env::default();
    let s = sample_surface("x ^ 2 - y ^ 2", 12, &env).unwrap();
    let segs = project_surface(&s, &View3D::default());
    let depths: Vec<f64> = segs.iter().map(|s| s.depth).collect();
    let mut sorted = depths.clone();
    sorted.sort_by(|a, b| b.total_cmp(a));
    assert_eq!(depths, sorted);
}

#[test]
fn surface_frame_has_ground_square_and_axes() {
    let env = Env::default();
    let s = sample_surface("x + y", 8, &env).unwrap();
    let frame = surface_frame(&s, &View3D::default());
    // 4 ground edges + 3 axes = 7 segments.
    assert_eq!(frame.len(), 7);
    for seg in &frame {
        assert!(seg.depth.is_finite());
    }
}

#[test]
fn surface_uses_constants_from_the_environment() {
    let mut env = Env::default();
    env.set_constant("c", Value::float(2.0));
    let s = sample_surface("c * x * y", 6, &env).unwrap();
    // z = 2xy at every grid point (x = xs[c], y = ys[r]).
    for r in 0..s.zs.len() {
        for c in 0..s.zs[r].len() {
            let expect = 2.0 * s.xs[c] * s.ys[r];
            assert!((s.zs[r][c] - expect).abs() < 1e-9);
        }
    }
}

#[test]
fn mesh_projection_splits_runs_at_undefined_cells() {
    let env = Env::default();
    let s = sample_surface("1 / (x ^ 2 + y ^ 2)", 10, &env).unwrap();
    let lines = epher_core::graph::project_mesh(&s, &View3D::default());
    // 11 rows + 11 columns, some split at the hole; painter-sorted.
    assert!(lines.len() >= 22);
    let depths: Vec<f64> = lines.iter().map(|l| l.depth).collect();
    let mut sorted = depths.clone();
    sorted.sort_by(|a, b| b.total_cmp(a));
    assert_eq!(depths, sorted);
    for line in &lines {
        assert!(line.points.len() >= 2);
    }
}

#[test]
fn orthographic_projection_stays_bounded_at_every_depth() {
    let env = Env::default();
    // z = x^2 - y^2 over [-5, 5] reaches z = -25; the old perspective
    // divide blew up near the camera plane (viewBox in the thousands,
    // plot a sliver). The orthographic projection is affine, so every
    // projected coordinate stays at the rotated world magnitude.
    let s = sample_surface("x ^ 2 - y ^ 2", 20, &env).unwrap();
    let view = View3D::default();
    let lines = project_mesh(&s, &view);
    assert!(lines.len() > 10);
    for line in &lines {
        for &(x, y) in &line.points {
            assert!(x.is_finite() && x.abs() < 200.0, "x = {x}");
            assert!(y.is_finite() && y.abs() < 200.0, "y = {y}");
        }
    }
    let frame = surface_frame(&s, &view);
    for seg in &frame {
        for (x, y) in [(seg.x1, seg.y1), (seg.x2, seg.y2)] {
            assert!(x.is_finite() && x.abs() < 200.0);
            assert!(y.is_finite() && y.abs() < 200.0);
        }
    }
}

#[test]
fn segments_project_at_any_depth() {
    let view = View3D {
        yaw: 0.0,
        pitch: 0.0,
        camera: 12.0,
    };
    // Orthographic: depth changes nothing about the mapping, so a
    // segment "behind" the old camera plane projects like any other.
    let (x1, y1, zp1, x2, y2, zp2) = project_clipped(0.0, 0.0, 13.0, 1.0, 0.0, 14.0, &view)
        .expect("finite segments always project");
    assert!(zp2 > zp1);
    assert!((x2 - x1 - 1.0).abs() < 1e-12);
    // Undefined cells are dropped.
    assert!(project_clipped(0.0, 0.0, f64::NAN, 1.0, 0.0, 0.0, &view).is_none());
}

#[test]
fn zoom_scales_the_window_without_distortion() {
    // The zoom contract (the ADR-0015 amendment): projected geometry is
    // zoom-independent, and the render window scales around its center,
    // so every point lands exactly k x its default-zoom screen position.
    let default = View3D::default();
    let zoomed = default.with_camera(15.0); // one +1 zoom step: 2x in
    let pts = [
        [2.0, -1.0, 0.5],
        [-3.0, 4.0, -2.0],
        [0.0, 0.0, 7.0],
    ];
    let mut base = Vec::new();
    let mut close = Vec::new();
    for &[x, y, z] in &pts {
        base.push(project_point(x, y, z, &default));
        close.push(project_point(x, y, z, &zoomed));
    }
    for (b, c) in base.iter().zip(&close) {
        assert_eq!(b.0, c.0);
        assert_eq!(b.1, c.1);
        assert!((b.2 - c.2).abs() < 1e-12);
    }
    // The window around those projections halves, centered identically.
    let (x_min, x_max, y_min, y_max) = (-3.0, 5.0, -2.0, 6.0);
    let (bx, by, bw, bh) = zoom_window(x_min, x_max, y_min, y_max, &default);
    let (zx, zy, zw, zh) = zoom_window(x_min, x_max, y_min, y_max, &zoomed);
    assert!((zw - bw / 2.0).abs() < 1e-12);
    assert!((zh - bh / 2.0).abs() < 1e-12);
    assert!((zx + zw / 2.0 - (bx + bw / 2.0)).abs() < 1e-12, "same center x");
    assert!((zy + zh / 2.0 - (by + bh / 2.0)).abs() < 1e-12, "same center y");
    // End to end: map each point into its window's canvas (0..1), then
    // the zoomed canvas position must be exactly the default position
    // pulled k = bw/zw times toward the center - a pure scale, nothing
    // else.
    let k = bw / zw;
    for (b, c) in base.iter().zip(&close) {
        let base_x = (b.0 - bx) / bw;
        let zoom_x = (c.0 - zx) / zw;
        assert!((zoom_x - (0.5 + k * (base_x - 0.5))).abs() < 1e-12);
        let base_y = (b.1 - by) / bh;
        let zoom_y = (c.1 - zy) / zh;
        assert!((zoom_y - (0.5 + k * (base_y - 0.5))).abs() < 1e-12);
    }
}

#[test]
fn surface_with_undefined_name_reports_the_name() {
    let env = Env::default();
    // The guide's animated example without the defining constant: every
    // cell fails, and the error must say why instead of the generic
    // no-finite-values message.
    let err = sample_surface("sin(a * (x ^ 2 + y ^ 2)) from -3 to 3", 8, &env).unwrap_err();
    assert_eq!(err.to_string(), "unknown name: a");
}

#[test]
fn surface_with_all_holes_keeps_the_generic_message() {
    let env = Env::default();
    // (-1) ^ 0.5 is NaN silently (powf, no error raised): holes, not
    // errors — the generic message. sqrt(-1) and ln(-1), by contrast,
    // raise domain errors that are reported as the cause.
    let err = sample_surface("(-1) ^ 0.5 + x", 8, &env).unwrap_err();
    assert!(err.to_string().contains("no finite values for the surface"));
}

#[test]
fn surface_reporting_division_by_zero_when_everywhere() {
    let env = Env::default();
    let err = sample_surface("1 / 0", 8, &env).unwrap_err();
    assert_eq!(err.to_string(), "division by zero");
}

// ===== The solar system scene (ADR-0037 + the ADR-0015 amendment) =====

use epher_core::astro::{solar_scene, SolarScene};
use epher_core::graph::{project_space_curve, project_world_dot};

fn scene() -> SolarScene {
    solar_scene(2459030.5).expect("the scene builds at 2020-07-01")
}

#[test]
fn the_scene_carries_ten_dots_and_nine_orbits() {
    let s = scene();
    // Sun, Mercury, Venus, Earth, Mars, Jupiter, Saturn, Uranus,
    // Neptune, Pluto, Moon — eleven dots total
    assert_eq!(s.dots.len(), 11, "dots: {:?}", s.dots);
    // orbits for Mercury..Neptune plus Pluto (the Moon's orbit is a
    // point at solar-system scale and is not drawn)
    assert_eq!(s.orbits.len(), 9, "orbits: {}", s.orbits.len());
    assert_eq!(s.trails.len(), 9);
    // the Sun sits at the origin
    let sun = s.dots.iter().find(|d| d.body == 10).expect("the Sun");
    assert!(sun.xyz.iter().all(|c| c.abs() < 1e-9));
    // every orbit is a closed-ish run of finite points
    for orbit in &s.orbits {
        assert!(orbit.points.len() >= 128, "orbit {} sampled thinly", orbit.body);
        assert!(orbit.points.iter().all(|p| p.iter().all(|c| c.is_finite())));
    }
}

#[test]
fn planet_dots_sit_at_their_almanac_distances() {
    let s = scene();
    let norm = |d: &epher_core::astro::SolarDot| {
        (d.xyz[0] * d.xyz[0] + d.xyz[1] * d.xyz[1] + d.xyz[2] * d.xyz[2]).sqrt()
    };
    // 2020-07-01: Jupiter about 4.85 AU, Neptune about 29.7 AU out
    let jupiter = s.dots.iter().find(|d| d.body == 5).expect("Jupiter");
    assert!((4.5..5.2).contains(&norm(jupiter)), "jupiter r = {}", norm(jupiter));
    let neptune = s.dots.iter().find(|d| d.body == 8).expect("Neptune");
    assert!((29.0..30.5).contains(&norm(neptune)), "neptune r = {}", norm(neptune));
    // the Moon is 0.0026ish from Earth
    let earth = s.dots.iter().find(|d| d.body == 3).expect("Earth");
    let moon = s.dots.iter().find(|d| d.body == 11).expect("Moon");
    let sep = ((moon.xyz[0] - earth.xyz[0]).powi(2)
        + (moon.xyz[1] - earth.xyz[1]).powi(2)
        + (moon.xyz[2] - earth.xyz[2]).powi(2))
    .sqrt();
    assert!((0.0024..0.00275).contains(&sep), "earth-moon = {sep} AU");
}

#[test]
fn trails_end_where_the_dots_are() {
    let s = scene();
    for trail in &s.trails {
        let dot = s.dots.iter().find(|d| d.body == trail.body).expect("its dot");
        let last = trail.points.last().expect("a non-empty trail");
        let gap: f64 = last
            .iter()
            .zip(dot.xyz.iter())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f64>()
            .sqrt();
        // the trail's final sample is on the orbit near the dot: within
        // a couple of percent of the orbital radius
        let radius: f64 = (dot.xyz[0] * dot.xyz[0] + dot.xyz[1] * dot.xyz[1]).sqrt();
        assert!(gap < radius * 0.05 + 1e-6, "body {}: trail ends {gap} from the dot", trail.body);
    }
}

#[test]
fn projection_maps_the_scene_to_screen_space() {
    let s = scene();
    let view = s.default_view();
    // orbit runs project to bounded polylines with depths
    let orbit = &s.orbits.iter().find(|o| o.body == 5).expect("Jupiter orbit");
    let runs = project_space_curve(&orbit.points, &view);
    assert!(!runs.is_empty());
    for run in &runs {
        assert!(run.points.iter().all(|(x, y)| x.is_finite() && y.is_finite()));
    }
    // dots project inside a sane screen window
    for dot in &s.dots {
        if let Some((x, y, _depth)) = project_world_dot(dot.xyz[0], dot.xyz[1], dot.xyz[2], &view) {
            assert!(x.abs() < 1000.0 && y.abs() < 1000.0);
        }
    }
}
