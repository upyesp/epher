//! The satellite accessor functions against JPL Horizons (ADR-0037's
//! facade): the Galileans ride Meeus's E5 theory and Saturn's eight
//! major moons Meeus ch. 46, both transcribed into the facade and fed
//! by the crate's real Earth/planet positions. The anchors below were
//! taken from the Horizons observer ephemeris (geocenter, astrometric
//! J2000) for 2026-09-17 00:00 UT: the model's projected sky offsets,
//! mapped through Jupiter's/Saturn's projected-pole position angle,
//! reproduce Horizons to ~0.2-3 arcseconds. The tolerances pin that
//! grade - a regression beyond it means the port or a dependency
//! drifted.

use epher_core::evaluate;

fn floats(text: &str) -> Vec<f64> {
    match evaluate(text) {
        Ok(epher_core::Value::List(items)) => items
            .into_iter()
            .map(|v| match v {
                epher_core::Value::Float(f) => f,
                other => panic!("{text} produced {other:?}"),
            })
            .collect(),
        other => panic!("{text} produced {other:?}"),
    }
}

fn float_at(text: &str) -> f64 {
    match evaluate(text) {
        Ok(epher_core::Value::Float(x)) => x,
        other => panic!("{text} produced {other:?}"),
    }
}

/// Project a moon's (x, y) into the sky offsets (dRA·cos dec, dDec) in
/// arcseconds: x is West-positive, and the planet's projected pole
/// points `pa_deg` east of north (negative at these epochs).
fn sky_offsets(x: f64, y: f64, sc: f64, pa_deg: f64) -> (f64, f64) {
    let (spa, cpa) = (pa_deg.to_radians().sin(), pa_deg.to_radians().cos());
    (-(x * sc) * cpa + (y * sc) * spa, -(x * sc) * spa + (y * sc) * cpa)
}

/// (satellite, model offset vs the Horizons anchor, in arcseconds)
const JUP_ANCHORS: &[(usize, f64, f64)] = &[
    (1, -42.7, 15.7),   // Io
    (2, 129.9, -46.0),  // Europa
    (3, -227.6, 81.6),  // Ganymede
    (4, 272.4, -96.4),  // Callisto
];

const SAT_ANCHORS: &[(usize, f64, f64)] = &[
    (1, 22.2, -3.6),    // Mimas
    (5, -25.3, -3.6),   // Rhea
    (6, 166.4, -22.6),  // Titan
];

const JDD: f64 = 2461300.5; // 2026-09-17 00:00 UT
const JUP_PA: f64 = -19.63; // projected-pole position angle (deg)
const SAT_PA: f64 = -3.07;

#[test]
fn galileans_match_horizons_to_about_two_arcseconds() {
    for (sat, want_dra, want_ddec) in JUP_ANCHORS {
        let x = float_at(&format!("satx(5, {sat}, {JDD})"));
        let y = float_at(&format!("saty(5, {sat}, {JDD})"));
        let d = float_at(&format!("dist(5, {JDD})"));
        let sc = 71492.0 / (d * 149_597_870.7) * 206_264.806_247;
        let (dra, ddec) = sky_offsets(x, y, sc, JUP_PA);
        assert!(
            (dra - want_dra).abs() < 3.0,
            "sat {sat} dRA*cos: {dra} vs Horizons {want_dra}"
        );
        assert!(
            (ddec - want_ddec).abs() < 3.0,
            "sat {sat} dDec: {ddec} vs Horizons {want_ddec}"
        );
    }
}

#[test]
fn saturn_moons_match_horizons_to_the_chapter_grade() {
    // ch. 46's own accuracy for the inner moons is a few times coarser
    // than E5's: against the same Horizons anchors the upstream
    // reference implementation shows the same residuals (Rhea's dec
    // ~6-10"), so the tolerance here pins the published grade, not an
    // ephemeris-grade match.
    for (sat, want_dra, want_ddec) in SAT_ANCHORS {
        let x = float_at(&format!("satx(6, {sat}, {JDD})"));
        let y = float_at(&format!("saty(6, {sat}, {JDD})"));
        let d = float_at(&format!("dist(6, {JDD})"));
        let sc = 60268.0 / (d * 149_597_870.7) * 206_264.806_247;
        let (dra, ddec) = sky_offsets(x, y, sc, SAT_PA);
        assert!(
            (dra - want_dra).abs() < 10.0,
            "sat {sat} dRA*cos: {dra} vs Horizons {want_dra}"
        );
        assert!(
            (ddec - want_ddec).abs() < 12.0,
            "sat {sat} dDec: {ddec} vs Horizons {want_ddec}"
        );
    }
}

#[test]
fn separation_and_phenomenon_are_well_formed() {
    // Io's projected separation from Jupiter's center at the anchor
    // (Horizons: 45.5" from the same offsets)
    let sep = float_at(&format!("satsep(5, 1, {JDD})"));
    assert!((40.0..50.0).contains(&sep), "io sep = {sep}");
    // states are codes 0..=4
    for body in [5, 6] {
        for sat in 1..=4 {
            let ph = float_at(&format!("satphen({body}, {sat}, {JDD})"));
            assert!((0.0..=4.0).contains(&ph), "phen {body}/{sat} = {ph}");
        }
    }
    // Titan far out: hundreds of arcseconds
    let sep = float_at(&format!("satsep(6, 6, {JDD})"));
    assert!((100.0..250.0).contains(&sep), "titan sep = {sep}");
}

#[test]
fn satellite_args_are_validated() {
    // only Jupiter and Saturn have tabulated satellites
    assert!(evaluate(&format!("satx(4, 1, {JDD})")).is_err());
    assert!(evaluate(&format!("satx(5, 5, {JDD})")).is_err());
    assert!(evaluate(&format!("satx(6, 9, {JDD})")).is_err());
    assert!(evaluate(&format!("satphen(5, 0, {JDD})")).is_err());
}
