//! The direct single-body ephemeris path against the full sky snapshot
//! (ADR-0037's facade): for the Sun and Moon, the accessors compute the
//! place directly from the crate's public modules, mirroring the
//! crate's private `topocentric_sky_at_time` — so every value the
//! accessors return must equal, bit for bit, what the snapshot's JSON
//! carries for the same body (after the same string quantization).
//! These tests pin that identity: if a `solar-ephemeris` upgrade
//! changes the private path and the mirror drifts, they fail loudly
//! here instead of letting the scripts' transcripts drift silently.
//! (The planets still ride the full snapshot: the crate's
//! `planet_apparent_ecliptic` needs a VSOP2013 table handle it keeps
//! private, so there is nothing to mirror yet.)

use epher_core::evaluate;

/// What the accessor language returns for one call, as f64.
fn float_at(text: &str) -> f64 {
    match evaluate(text) {
        Ok(epher_core::Value::Float(x)) => x,
        other => panic!("{text} produced {other:?}"),
    }
}

/// One field of one body's entry in a freshly built sky snapshot.
fn snapshot_field(jd: f64, lat: f64, lon: f64, name: &str, key: &str) -> f64 {
    let json = solar_ephemeris::sky_snapshot_json(jd, lat, lon, 0.0);
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("snapshot parses");
    parsed["bodies"]
        .as_array()
        .expect("bodies array")
        .iter()
        .find(|b| b["name"] == name)
        .unwrap_or_else(|| panic!("no {name} in snapshot"))
        .get(key)
        .and_then(|v| v.as_f64())
        .unwrap_or_else(|| panic!("no {key} for {name}"))
}

/// (body number, crate body name, jd, lat, lon) — the great lights
/// across seasons, distances, and hemispheres.
const CASES: &[(i64, &str, f64, f64, f64)] = &[
    (10, "Sun", 2451544.5, 0.0, 0.0),
    (10, "Sun", 2461000.75, 64.1, -21.9),
    (10, "Sun", 2458849.5, -33.9, 151.2),
    (11, "Moon", 2458988.5, 28.6, 77.2),
    (11, "Moon", 2460500.5, -54.8, -68.3),
    (11, "Moon", 2460925.5, 52.5, 13.4),
];

#[test]
fn direct_ra_dec_dist_alt_az_and_diam_match_the_snapshot() {
    for (number, name, jd, lat, lon) in CASES {
        // geocentric place (the accessors fix the observer at 0,0)
        let ra = float_at(&format!("ra({number}, {jd})"));
        let want = snapshot_field(*jd, 0.0, 0.0, name, "geocentric_apparent_ra_deg");
        assert_eq!(ra, want, "ra of {name} at {jd}");
        let dec = float_at(&format!("decl({number}, {jd})"));
        let want = snapshot_field(*jd, 0.0, 0.0, name, "geocentric_apparent_dec_deg");
        assert_eq!(dec, want, "decl of {name} at {jd}");
        let dist = float_at(&format!("dist({number}, {jd})"));
        let want = snapshot_field(*jd, 0.0, 0.0, name, "distance_km") / 149_597_870.7;
        assert_eq!(dist, want, "dist of {name} at {jd}");
        // topocentric place at the case's observer
        let alt = float_at(&format!("alt({number}, {jd}, {lat}, {lon})"));
        let want = snapshot_field(*jd, *lat, *lon, name, "alt_deg");
        assert_eq!(alt, want, "alt of {name} at {jd} ({lat}, {lon})");
        let az = float_at(&format!("az({number}, {jd}, {lat}, {lon})"));
        let want = snapshot_field(*jd, *lat, *lon, name, "az_deg");
        assert_eq!(az, want, "az of {name} at {jd} ({lat}, {lon})");
        // angular size from the geocentric distance
        let diam = float_at(&format!("diam({number}, {jd})"));
        let want = snapshot_field(*jd, 0.0, 0.0, name, "angular_size_arcsec") / 3600.0;
        assert_eq!(diam, want, "diam of {name} at {jd}");
    }
}

#[test]
fn direct_path_stays_exact_over_a_bisect_like_sweep() {
    // A bisect-like sweep: 25 consecutive instants, one body. Every
    // accessor value must equal the snapshot's — the scripts that scan
    // transit hundreds of such steps, and their transcripts print the
    // values at full precision.
    let jd0 = 2460845.0;
    for k in 0..25 {
        let jd = jd0 + k as f64 * 0.037;
        let alt = float_at(&format!("alt(11, {jd}, 52.5, 13.4)"));
        let want = snapshot_field(jd, 52.5, 13.4, "Moon", "alt_deg");
        assert_eq!(alt, want, "moon alt at step {k} (jd {jd})");
    }
}
