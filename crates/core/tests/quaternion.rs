//! Quaternions (ADR-0085), through the evaluator seam: the literal, the
//! Hamilton algebra, the rotation family, the astronomy bridge, and the
//! names the language reserves.

use epher_core::{eval, parse, Env, Session, Value};

/// Evaluate source text with an empty environment.
fn eval_str(src: &str) -> Value {
    eval(&parse(src).expect("parse"), &Env::default()).expect("eval")
}

/// Evaluate and expect an error, returning its message.
fn eval_err(src: &str) -> String {
    match eval(&parse(src).expect("parse"), &Env::default()) {
        Ok(v) => panic!("expected an error from {src}, got {v:?}"),
        Err(e) => e.to_string(),
    }
}

fn q(w: f64, x: f64, y: f64, z: f64) -> Value {
    Value::Quaternion { w, x, y, z }
}

fn float_of(v: &Value) -> f64 {
    match v {
        Value::Float(x) => *x,
        other => panic!("expected a float, got {other:?}"),
    }
}

fn list_of(v: &Value) -> Vec<f64> {
    match v {
        Value::List(items) => items.iter().map(float_of).collect(),
        other => panic!("expected a list, got {other:?}"),
    }
}

fn components(v: &Value) -> [f64; 4] {
    match v {
        Value::Quaternion { w, x, y, z } => [*w, *x, *y, *z],
        other => panic!("expected a quaternion, got {other:?}"),
    }
}

fn assert_near(actual: f64, expected: f64, tol: f64) {
    assert!(
        (actual - expected).abs() <= tol,
        "expected {expected}, got {actual} (tolerance {tol})"
    );
}

fn assert_quaternion_near(actual: &Value, expected: [f64; 4], tol: f64) {
    let got = components(actual);
    for (i, (a, e)) in got.iter().zip(expected).enumerate() {
        assert!(
            (a - e).abs() <= tol,
            "component {i}: expected {e}, got {a} (in {got:?} against {expected:?})"
        );
    }
}

#[test]
fn a_literal_spells_all_four_parts() {
    let value = eval_str("1 + 2i + 3j + 4k");
    assert_eq!(value, q(1.0, 2.0, 3.0, 4.0));
    assert_eq!(format!("{value}"), "1 + 2i + 3j + 4k");
}

#[test]
fn the_units_alone_are_constants() {
    assert_eq!(eval_str("j"), q(0.0, 0.0, 1.0, 0.0));
    assert_eq!(eval_str("k"), q(0.0, 0.0, 0.0, 1.0));
    assert_eq!(format!("{}", eval_str("j")), "j");
    assert_eq!(format!("{}", eval_str("-2k")), "-2k");
    assert_eq!(format!("{}", eval_str("1 - 2k")), "1 - 2k");
    assert_eq!(format!("{}", eval_str("i")), "i");
    // Based literals carry the suffix too (the ADR-0043 rule).
    assert_eq!(format!("{}", eval_str("0xFFj")), "255j");
    // A longer identifier keeps the number and the name apart: the
    // suffix is not consumed, so `3jt` is not a literal.
    assert!(parse("3jt").is_err());
}

#[test]
fn the_display_drops_zero_parts() {
    assert_eq!(format!("{}", q(3.0, 0.0, 0.0, 0.0)), "3");
    assert_eq!(format!("{}", q(1.0, 1.0, 0.0, 0.0)), "1 + i");
    assert_eq!(format!("{}", q(0.0, -1.0, 0.0, 0.0)), "-i");
    assert_eq!(format!("{}", q(0.5, 0.0, 0.0, -2.0)), "0.5 - 2k");
    assert_eq!(format!("{}", q(0.0, 0.0, 0.0, 0.0)), "0");
}

#[test]
fn hamilton_products_anti_commute() {
    assert_eq!(eval_str("i * j"), eval_str("k"));
    assert_eq!(eval_str("j * i"), eval_str("-k"));
    assert_eq!(eval_str("k * k"), q(-1.0, 0.0, 0.0, 0.0));
    assert_eq!(eval_str("j * j"), q(-1.0, 0.0, 0.0, 0.0));
    // A product of two complex values stays complex: the promotion into
    // the four-part algebra only happens when a quaternion is present.
    assert!(matches!(eval_str("i * i"), Value::Complex(_)));
    assert_eq!(format!("{}", eval_str("i * i")), "-1");
}

#[test]
fn the_constructor_is_scalar_first() {
    assert_eq!(
        eval_str("quaternion(1, 2, 3, 4)"),
        eval_str("1 + 2i + 3j + 4k")
    );
    assert!(eval_err("quaternion(1, 2, 3)").contains("4"));
}

#[test]
fn floats_and_complex_promote_and_exact_layers_do_not() {
    assert_eq!(
        eval_str("quaternion(1, 2, 3, 4) + 1"),
        q(2.0, 2.0, 3.0, 4.0)
    );
    assert_eq!(eval_str("2 * j"), q(0.0, 0.0, 2.0, 0.0));
    assert_eq!(eval_str("(1 + 0i) * j"), q(0.0, 0.0, 1.0, 0.0));
    assert_eq!(eval_str("j - 2"), q(-2.0, 0.0, 1.0, 0.0));
    for src in [
        "frac(1, 2) * j",
        "dec(1) * j",
        "big(1) * j",
        "1 m * j",
        "{1, 2} * j",
        "[[1]] * j",
    ] {
        assert!(!eval_err(src).is_empty(), "{src} should refuse");
    }
}

#[test]
fn division_is_multiplication_by_the_inverse() {
    let quotient =
        eval_str("(quaternion(1, 2, 3, 4) * quaternion(5, 6, 7, 8)) / quaternion(5, 6, 7, 8)");
    assert_quaternion_near(&quotient, [1.0, 2.0, 3.0, 4.0], 1e-12);
    assert!(eval_err("j / quaternion(0, 0, 0, 0)").contains("zero"));
}

#[test]
fn equality_is_same_kind_and_componentwise() {
    assert_eq!(eval_str("j == j"), Value::Bool(true));
    assert_eq!(eval_str("j == k"), Value::Bool(false));
    assert_eq!(eval_str("j != k"), Value::Bool(true));
    assert!(eval_err("j == 1").contains("quaternion"));
    assert!(eval_err("j < k").contains("order"));
}

#[test]
fn transcendentals_extend_and_trig_refuses() {
    let round_trip = eval_str("ln(exp(j))");
    assert_quaternion_near(&round_trip, [0.0, 0.0, 1.0, 0.0], 1e-12);
    assert_quaternion_near(
        &eval_str("exp(quaternion(0, 0, 0, 0))"),
        [1.0, 0.0, 0.0, 0.0],
        0.0,
    );
    assert_eq!(eval_str("abs(quaternion(3, 0, 4, 0))"), Value::Float(5.0));
    assert_eq!(eval_str("abs(j)"), Value::Float(1.0));
    let root = eval_str("sqrt(quaternion(-4, 0, 0, 0))");
    assert_quaternion_near(&root, [0.0, 2.0, 0.0, 0.0], 1e-9);
    assert_quaternion_near(
        &eval_str("log(quaternion(100, 0, 0, 0))"),
        [2.0, 0.0, 0.0, 0.0],
        1e-12,
    );
    assert_eq!(
        eval_str("quaternion(1, 2, 3, 4) ^ 2"),
        eval_str("quaternion(1, 2, 3, 4) * quaternion(1, 2, 3, 4)")
    );
    for src in ["sin(j)", "cos(j)", "tan(j)", "asin(j)", "sinh(j)"] {
        assert!(!eval_err(src).is_empty(), "{src} should refuse");
    }
}

#[test]
fn parts_read_the_quaternion() {
    assert_eq!(eval_str("re(quaternion(1, 2, 3, 4))"), Value::Float(1.0));
    assert_eq!(
        eval_str("im(quaternion(1, 2, 3, 4))"),
        Value::List(vec![
            Value::Float(2.0),
            Value::Float(3.0),
            Value::Float(4.0)
        ])
    );
    assert_eq!(
        eval_str("conj(quaternion(1, 2, 3, 4))"),
        q(1.0, -2.0, -3.0, -4.0)
    );
    assert_eq!(eval_str("qw(quaternion(1, 2, 3, 4))"), Value::Float(1.0));
    assert_eq!(eval_str("qx(quaternion(1, 2, 3, 4))"), Value::Float(2.0));
    assert_eq!(eval_str("qy(quaternion(1, 2, 3, 4))"), Value::Float(3.0));
    assert_eq!(eval_str("qz(quaternion(1, 2, 3, 4))"), Value::Float(4.0));
    assert_near(
        float_of(&eval_str("qnorm(quaternion(1, 2, 3, 4))")),
        30f64.sqrt(),
        1e-12,
    );
    assert_near(
        float_of(&eval_str("qnorm(qunit(quaternion(1, 2, 3, 4)))")),
        1.0,
        1e-12,
    );
    assert_quaternion_near(
        &eval_str("qinv(quaternion(1, 2, 3, 4)) * quaternion(1, 2, 3, 4)"),
        [1.0, 0.0, 0.0, 0.0],
        1e-12,
    );
    assert_quaternion_near(
        &eval_str("qconj(quaternion(1, 2, 3, 4))"),
        [1.0, -2.0, -3.0, -4.0],
        0.0,
    );
    assert!(eval_err("qinv(quaternion(0, 0, 0, 0))").contains("zero"));
}

#[test]
fn the_axis_and_angle_pair_round_trips() {
    let built = eval_str("qaxis({0, 0, 1}, 90 deg)");
    let half = std::f64::consts::FRAC_1_SQRT_2;
    assert_quaternion_near(&built, [half, 0.0, 0.0, half], 1e-12);
    assert_near(
        float_of(&eval_str("qangle(qaxis({0, 0, 1}, 90 deg))")),
        std::f64::consts::FRAC_PI_2,
        1e-12,
    );
    let axis = eval_str("qaxis(qaxis({0, 0, 1}, 90 deg))");
    for (got, want) in list_of(&axis).iter().zip([0.0, 0.0, 1.0]) {
        assert_near(*got, want, 1e-12);
    }
    assert!(eval_err("qaxis({0, 0, 0}, 1)").contains("axis"));
    assert!(eval_err("qaxis({1, 0}, 1)").contains("3"));
}

#[test]
fn rotation_is_by_name_only() {
    let rotated = eval_str("qrot(qaxis({0, 0, 1}, 90 deg), {1, 0, 0})");
    for (got, want) in list_of(&rotated).iter().zip([0.0, 1.0, 0.0]) {
        assert_near(*got, want, 1e-12);
    }
    assert!(!eval_err("qaxis({0, 0, 1}, 90 deg) * {1, 0, 0}").is_empty());
    assert!(!eval_err("qrot(qaxis({0, 0, 1}, 90 deg), 1)").is_empty());
}

#[test]
fn the_matrix_bridge_round_trips() {
    let m = eval_str("q2m(qaxis({0, 0, 1}, 90 deg))");
    let Value::Matrix { rows, cols, data } = &m else {
        panic!("q2m should answer a matrix, got {m:?}");
    };
    assert_eq!((*rows, *cols), (3, 3));
    for (got, want) in data
        .iter()
        .zip([0.0, -1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0])
    {
        assert_near(*got, want, 1e-12);
    }
    let back = eval_str("m2q([[0, -1, 0], [1, 0, 0], [0, 0, 1]])");
    assert_quaternion_near(
        &back,
        [
            std::f64::consts::FRAC_1_SQRT_2,
            0.0,
            0.0,
            std::f64::consts::FRAC_1_SQRT_2,
        ],
        1e-12,
    );
    assert_eq!(
        eval_str("m2q([[1, 0, 0], [0, 1, 0], [0, 0, 1]])"),
        q(1.0, 0.0, 0.0, 0.0)
    );
    assert!(!eval_err("m2q([[1, 0], [0, 1]])").is_empty());
    // A plain number promotes to a real quaternion, whose rotation is
    // the identity.
    assert_eq!(
        eval_str("q2m(1)"),
        eval_str("[[1, 0, 0], [0, 1, 0], [0, 0, 1]]")
    );
}

#[test]
fn m2q_returns_the_positive_scalar_representative() {
    // The matrix for a 180 degree turn about z has w = 0; a matrix just
    // past it (a 270 degree turn) has a negative scalar component in
    // its raw extraction, and m2q flips the sign (the SPICE rule).
    let value = eval_str("m2q(q2m(qaxis({0, 0, 1}, 270 deg)))");
    let parts = components(&value);
    assert!(parts[0] >= 0.0, "scalar part should be >= 0, got {parts:?}");
}

#[test]
fn slerp_follows_the_shortest_path() {
    let mid = eval_str("qslerp(quaternion(1, 0, 0, 0), qaxis({0, 0, 1}, 90 deg), 0.5)");
    // Half of a 90 degree turn is a 45 degree turn, whose quaternion
    // holds the half angle in its cosine and sine.
    let half_angle = std::f64::consts::PI / 8.0;
    assert_quaternion_near(&mid, [half_angle.cos(), 0.0, 0.0, half_angle.sin()], 1e-12);
    // 270 degrees is the long way round: the shortest path is -90.
    let short =
        eval_str("qrot(qslerp(quaternion(1, 0, 0, 0), qaxis({0, 0, 1}, 270 deg), 0.5), {1, 0, 0})");
    for (got, want) in list_of(&short).iter().zip([
        std::f64::consts::FRAC_1_SQRT_2,
        -std::f64::consts::FRAC_1_SQRT_2,
        0.0,
    ]) {
        assert_near(*got, want, 1e-12);
    }
    assert_quaternion_near(&eval_str("qslerp(j, j, 0.3)"), [0.0, 0.0, 1.0, 0.0], 1e-12);
}

#[test]
fn the_horizon_bridge_matches_alt_and_az() {
    let env = Env::default();
    let alt = eval(&parse("alt(10, 2460000.5, 51.5, -0.12)").unwrap(), &env).unwrap();
    let az = eval(&parse("az(10, 2460000.5, 51.5, -0.12)").unwrap(), &env).unwrap();
    // Build the star-style equatorial unit vector from the Sun's own
    // apparent place, rotate it, and read alt/az off the horizon frame
    // (east, north, up). The Sun's parallax is 8.8 arcseconds, so the
    // agreement is to a hundredth of a degree.
    let ra = float_of(&eval(&parse("ra(10, 2460000.5)").unwrap(), &env).unwrap());
    let dec = float_of(&eval(&parse("decl(10, 2460000.5)").unwrap(), &env).unwrap());
    let v = format!(
        "{{{:.17} * cos({ra} deg), {:.17} * sin({ra} deg), {:.17}}}",
        dec.to_radians().cos(),
        dec.to_radians().cos(),
        dec.to_radians().sin()
    );
    let rotated = eval_str(&format!("qrot(qeq2hor(2460000.5, 51.5, -0.12), {v})"));
    let parts = list_of(&rotated);
    assert_near(parts[2].asin().to_degrees(), float_of(&alt), 0.01);
    assert_near(
        parts[0].atan2(parts[1]).to_degrees().rem_euclid(360.0),
        float_of(&az),
        0.01,
    );
}

#[test]
fn the_units_are_reserved_names() {
    let mut session = Session::new();
    let out = session.submit("k = 5");
    assert!(out.contains("quaternion unit"), "got {out:?}");
    let out = session.submit("const j = 1");
    assert!(out.contains("quaternion unit"), "got {out:?}");
    let out = session.submit("{j, x} = {1, 2}");
    assert!(out.contains("quaternion unit"), "got {out:?}");
    // Scoped uses keep working: the classic loop counter, and a
    // parameter named k lives only for the call.
    assert_eq!(session.submit("for k in 1 to 3 do k"), "= {1, 2, 3}");
    assert_eq!(session.submit("def k(x) = x + 1"), "");
    assert_eq!(session.submit("k(2)"), "= 3");
}

#[test]
fn stored_unit_names_are_dropped_at_load() {
    let mut bindings = std::collections::HashMap::new();
    bindings.insert("j".to_string(), Value::Float(5.0));
    bindings.insert("k".to_string(), Value::Float(6.0));
    bindings.insert("x".to_string(), Value::Float(7.0));
    let mut session = Session::new();
    session.restore_bindings(&bindings);
    assert_eq!(session.submit("x"), "= 7");
    assert_eq!(session.submit("j"), "= j");
    assert_eq!(session.submit("k"), "= k");
}

#[test]
fn quaternions_are_refused_where_they_do_not_belong() {
    for src in ["{j}", "[[j]]", "mean(j)", "sqrt({j})"] {
        assert!(!eval_err(src).is_empty(), "{src} should refuse");
    }
}

#[test]
fn a_quaternion_survives_the_store_round_trip() {
    let mut session = Session::new();
    assert_eq!(
        session.submit("q = 1 + 2i + 3j + 4k"),
        "= 1 + 2i + 3j + 4k"
    );
    let stored = serde_json::to_string(session.bindings()).expect("serialize");
    let back: epher_core::ValueBindings = serde_json::from_str(&stored).expect("deserialize");
    let mut fresh = Session::new();
    fresh.restore_bindings(&back);
    assert_eq!(fresh.submit("q"), "= 1 + 2i + 3j + 4k");
    // The last answer carries it too.
    assert_eq!(fresh.submit("ans"), "= 1 + 2i + 3j + 4k");
}

#[test]
fn the_refusals_name_both_sides() {
    // A complex component names the part and what a quaternion cannot
    // hold (ADR-0085 refuses biquaternions).
    let err = eval_err("quaternion(i, 1, 2, 3)");
    assert!(err.contains("w part must be a real number"), "got {err:?}");
    assert!(err.contains("got i"), "got {err:?}");
    let err = eval_err("quaternion(1, 1 m, 2, 3)");
    assert!(err.contains("x part"), "got {err:?}");
    // A refused mix spells both sides, never a Debug dump.
    let err = eval_err("2 m * j");
    assert!(err.contains("cannot combine"), "got {err:?}");
    assert!(!err.contains("Quaternion {"), "Debug leaked: {err:?}");
    // arg has no quaternion reading: the same type error as log2.
    let err = eval_err("arg(j)");
    assert_eq!(err, "type error: arg expects a number, got j");
    assert_eq!(eval_err("log2(j)"), "type error: log2 expects a number, got j");
}

#[test]
fn the_angle_reports_what_the_quaternion_encodes() {
    // The double cover: q and -q are the same rotation, and qangle
    // reports each quaternion's own angle in 0..2pi while m2q answers
    // the representative with a scalar part of at least zero.
    let long = eval_str("qangle(qaxis({0, 0, 1}, 270 deg))");
    assert_near(float_of(&long), 3.0 * std::f64::consts::FRAC_PI_2, 1e-12);
    let canonical = eval_str("qangle(m2q(q2m(qaxis({0, 0, 1}, 270 deg))))");
    assert_near(
        float_of(&canonical),
        std::f64::consts::FRAC_PI_2,
        1e-12,
    );
    // The zero quaternion has no angle; its square root is still zero.
    assert!(eval_err("qangle(quaternion(0, 0, 0, 0))").contains("zero"));
    assert_quaternion_near(
        &eval_str("sqrt(quaternion(0, 0, 0, 0))"),
        [0.0, 0.0, 0.0, 0.0],
        0.0,
    );
}
