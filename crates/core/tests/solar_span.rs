//! The solar pane's time-window rules (ADR-0090 + the ADR-0091
//! amendment): the knob spans the outermost displayed orbit, one play
//! cycle completes that orbit, and the month window survives only as
//! the nothing-displayed fallback.

use epher_core::astro::{solar_time_span, solar_time_span_for, SolarPath};

fn unit_circle_orbit(body: i64, radius: f64) -> SolarPath {
    let points: Vec<[f64; 3]> = (0..64)
        .map(|k| {
            let a = std::f64::consts::TAU * k as f64 / 64.0;
            [radius * a.cos(), radius * a.sin(), 0.0]
        })
        .collect();
    SolarPath { body, points }
}

fn ellipse_orbit(body: i64, a: f64, e: f64) -> SolarPath {
    // True-anomaly samples of an ellipse with the Sun at the focus:
    // r ranges over a(1−e)..a(1+e), so the sampled extremes pin a.
    let points: Vec<[f64; 3]> = (0..360)
        .map(|k| {
            let nu = std::f64::consts::TAU * k as f64 / 360.0;
            let r = a * (1.0 - e * e) / (1.0 + e * nu.cos());
            [r * nu.cos(), r * nu.sin(), 0.0]
        })
        .collect();
    SolarPath { body, points }
}

#[test]
fn orbit_period_of_a_circular_orbit_is_one_kepler_year_per_unit_axis() {
    let period = epher_core::astro::orbit_period_days(&unit_circle_orbit(3, 1.0).points)
        .expect("a circle has a period");
    assert!(
        (period - epher_core::astro::SIDEREAL_YEAR_DAYS).abs() < 1e-6,
        "a = 1 AU orbits in one sidereal year: {period}"
    );
}

#[test]
fn orbit_period_follows_kepler_from_the_sampled_extremes() {
    // a = 4 AU, e = 0.25: the samples reach r = 3 and r = 5, whose
    // average recovers a, and Kepler gives 365.2569 × 4^1.5 days.
    let period = epher_core::astro::orbit_period_days(&ellipse_orbit(5, 4.0, 0.25).points)
        .expect("an ellipse has a period");
    let want = epher_core::astro::SIDEREAL_YEAR_DAYS * 8.0;
    assert!((period - want).abs() < 1.0, "P {period} vs Kepler {want}");
    assert!(
        epher_core::astro::orbit_period_days(&[]).is_none(),
        "no points, no period"
    );
    assert!(
        epher_core::astro::orbit_period_days(&[[0.0; 3]])
            .is_none(),
        "a point collapsed onto the Sun has no orbit"
    );
}

#[test]
fn the_knob_spans_the_outermost_displayed_orbit() {
    let jd = 2_461_366.5;
    let scene = epher_core::astro::solar_scene(jd).expect("the scene builds");

    // Everything displayed: Pluto rides the facade's elements and is
    // the outermost orbit, ~248 years — one 24 s cycle covers exactly
    // that one orbit (the width IS the period).
    let (lo, hi) = solar_time_span_for(jd, &scene.orbits);
    assert!((lo + hi - 2.0 * jd).abs() < 1e-9, "centered on jd");
    let width = hi - lo;
    assert!(
        (88_000.0..94_000.0).contains(&width),
        "one cycle is Pluto's orbit: {width} days"
    );
    let pluto_period = epher_core::astro::orbit_period_days(
        &scene
            .orbits
            .iter()
            .find(|p| p.body == 9)
            .expect("Pluto orbits")
            .points,
    )
    .expect("Pluto has a period");
    assert!(
        (width - pluto_period).abs() < 1e-6,
        "the sweep width equals the outermost period"
    );
    assert!(
        (lo - (jd - width / 2.0)).abs() < 1e-9,
        "the window centers on the drawn instant"
    );

    // Earth only: one cycle is exactly one year.
    let earth: Vec<SolarPath> = scene
        .orbits
        .iter()
        .filter(|p| p.body == 3)
        .cloned()
        .collect();
    let (lo, hi) = solar_time_span_for(jd, &earth);
    assert!(
        ((hi - lo) - epher_core::astro::SIDEREAL_YEAR_DAYS).abs() < 5.0,
        "Earth's cycle is a year: {}",
        hi - lo
    );

    // The outermost DISPLAYED body governs: Mercury beside Neptune
    // still spans Neptune's ~165-year orbit.
    let two: Vec<SolarPath> = scene
        .orbits
        .iter()
        .filter(|p| p.body == 1 || p.body == 8)
        .cloned()
        .collect();
    let (_, hi_two) = solar_time_span_for(jd, &two);
    let neptune_period = epher_core::astro::orbit_period_days(
        &scene
            .orbits
            .iter()
            .find(|p| p.body == 8)
            .expect("Neptune orbits")
            .points,
    )
    .expect("Neptune has a period");
    assert!(
        ((hi_two - jd) * 2.0 - neptune_period).abs() < 1e-6,
        "the span follows Neptune, not Mercury"
    );
}

#[test]
fn nothing_displayed_falls_back_to_the_month_window() {
    let jd = 2_461_366.5;
    assert_eq!(solar_time_span_for(jd, &[]), solar_time_span(jd));
    // A path with no usable points contributes nothing and never
    // poisons the span of the others.
    let degenerate = SolarPath {
        body: 11,
        points: Vec::new(),
    };
    let earth = unit_circle_orbit(3, 1.0);
    let (lo, hi) = solar_time_span_for(jd, &[degenerate, earth.clone()]);
    assert!(
        ((hi - lo) - epher_core::astro::SIDEREAL_YEAR_DAYS).abs() < 1e-6,
        "the moon's empty orbit is skipped"
    );
}
