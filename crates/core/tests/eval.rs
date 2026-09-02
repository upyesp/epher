use bigdecimal::BigDecimal;
use epher_core::{
    eval, evaluate, parse, parse_latex, parse_script, run, run_all, sample, sample_parametric,
    sample_polar, Env, Sample, Session, Value,
};
use num_rational::BigRational;
use rust_decimal::Decimal;
use std::str::FromStr;

/// Evaluate source text with an empty environment — the common case in these
/// tests (the CLI's future `evaluate(text)` convenience does the same).
fn eval_str(src: &str) -> Value {
    let env = Env::default();
    eval(&parse(src).expect("parse"), &env).expect("eval")
}

/// Evaluate source text with an empty environment as a plain f64 - for
/// constants and other float-valued checks.
fn eval_f64(src: &str) -> f64 {
    match eval_str(src) {
        Value::Float(n) => n,
        other => panic!("expected a float from {src}, got {other:?}"),
    }
}

#[test]
fn literal_number_evaluates_to_float_value() {
    assert_eq!(eval_str("2"), Value::float(2.0));
}

#[test]
fn addition_of_two_numbers() {
    assert_eq!(eval_str("2 + 3"), Value::float(5.0));
}

#[test]
fn multiplication_of_two_numbers() {
    assert_eq!(eval_str("2 * 3"), Value::float(6.0));
}

#[test]
fn multiplication_binds_tighter_than_addition() {
    assert_eq!(eval_str("2 + 3 * 4"), Value::float(14.0));
}

#[test]
fn subtraction_is_left_associative() {
    assert_eq!(eval_str("5 - 2 - 1"), Value::float(2.0));
}

#[test]
fn division_is_left_associative() {
    assert_eq!(eval_str("8 / 4 / 2"), Value::float(1.0));
}

#[test]
fn parentheses_group_expressions() {
    assert_eq!(eval_str("2 * (3 + 4)"), Value::float(14.0));
}

#[test]
fn unary_minus_negates_a_number() {
    assert_eq!(eval_str("-2 + 5"), Value::float(3.0));
}

#[test]
fn division_by_zero_is_an_error() {
    assert!(eval(&parse("1 / 0").expect("parse"), &Env::default()).is_err());
}

#[test]
fn exponentiation_is_right_associative() {
    assert_eq!(eval_str("2 ^ 3 ^ 2"), Value::float(512.0));
}

#[test]
fn variable_resolves_from_environment() {
    let mut env = Env::default();
    env.set("x", Value::float(3.0));
    let result = eval(&parse("x + 2").expect("parse"), &env).expect("eval");
    assert_eq!(result, Value::float(5.0));
}

#[test]
fn unknown_variable_is_an_error() {
    assert!(eval(&parse("q").expect("parse"), &Env::default()).is_err());
}

#[test]
fn builtin_function_call() {
    assert_eq!(eval_str("sqrt(16)"), Value::float(4.0));
}

#[test]
fn function_call_args_are_expressions() {
    assert_eq!(eval_str("sqrt(9 + 7)"), Value::float(4.0));
}

#[test]
fn builtin_constants_pi_and_e() {
    assert_eq!(eval_str("pi"), Value::float(3.141592653589793));
    assert_eq!(eval_str("e"), Value::float(2.718281828459045));
}

#[test]
fn builtin_function_with_two_arguments() {
    assert_eq!(eval_str("min(2, 3)"), Value::float(2.0));
}

#[test]
fn unary_minus_binds_looser_than_power() {
    assert_eq!(eval_str("-2 ^ 2"), Value::float(-4.0));
    assert_eq!(eval_str("2 ^ -2"), Value::float(0.25));
}

#[test]
fn assignment_then_use_in_next_statement() {
    let mut env = Env::default();
    let script = parse_script("x = 5; x + 1").expect("parse_script");
    let result = run(&script, &mut env).expect("run").expect("value");
    assert_eq!(result, Value::float(6.0));
    assert_eq!(env.get("x"), Some(&Value::float(5.0)));
}

#[test]
fn user_defined_function() {
    let mut env = Env::default();
    let script = parse_script("def f(x) = x ^ 2; f(3)").expect("parse_script");
    let result = run(&script, &mut env).expect("run").expect("value");
    assert_eq!(result, Value::float(9.0));
}

#[test]
fn comparison_operators_produce_booleans() {
    assert_eq!(eval_str("2 > 1"), Value::Bool(true));
    assert_eq!(eval_str("2 < 1"), Value::Bool(false));
    assert_eq!(eval_str("2 >= 2"), Value::Bool(true));
    assert_eq!(eval_str("2 <= 1"), Value::Bool(false));
    assert_eq!(eval_str("2 == 2"), Value::Bool(true));
    assert_eq!(eval_str("2 != 2"), Value::Bool(false));
}

#[test]
fn if_expression_picks_branch_by_condition() {
    assert_eq!(eval_str("if 2 > 1 then 10 else 20"), Value::float(10.0));
    assert_eq!(eval_str("if 2 < 1 then 10 else 20"), Value::float(20.0));
}

#[test]
fn user_function_recurses() {
    let mut env = Env::default();
    let script = parse_script("def fact(n) = if n <= 1 then 1 else n * fact(n - 1); fact(5)")
        .expect("parse_script");
    let result = run(&script, &mut env).expect("run").expect("value");
    assert_eq!(result, Value::float(120.0));
}

#[test]
fn evaluate_convenience_adapter() {
    assert_eq!(evaluate("2 + 3 * 4").expect("evaluate"), Value::float(14.0));
}

#[test]
fn exact_rational_arithmetic() {
    assert_eq!(
        eval_str("frac(1, 3) + frac(1, 3)"),
        Value::Rational(BigRational::new(2.into(), 3.into()))
    );
}

#[test]
fn float_promotes_to_rational_when_mixed() {
    assert_eq!(
        eval_str("frac(1, 3) * 3"),
        Value::Rational(BigRational::new(1.into(), 1.into()))
    );
}

#[test]
fn while_loop_repeats_until_condition_fails() {
    let mut env = Env::default();
    let script = parse_script("x = 0; while x < 3 do x = x + 1; x").expect("parse_script");
    let result = run(&script, &mut env).expect("run").expect("value");
    assert_eq!(result, Value::float(3.0));
}

#[test]
fn decimal_arithmetic_is_exact() {
    assert_eq!(
        eval_str("dec(0.1) + dec(0.2)"),
        Value::Decimal(Decimal::from_str("0.3").unwrap())
    );
}

#[test]
fn float_promotes_to_decimal_when_mixed() {
    assert_eq!(
        eval_str("dec(0.5) * 2"),
        Value::Decimal(Decimal::from_str("1.0").unwrap())
    );
}

#[test]
fn boolean_operators() {
    assert_eq!(eval_str("2 > 1 and 3 > 2"), Value::Bool(true));
    assert_eq!(eval_str("2 > 1 or 3 < 2"), Value::Bool(true));
    assert_eq!(eval_str("not 2 > 1"), Value::Bool(false));
    assert_eq!(eval_str("not (2 > 3)"), Value::Bool(true));
}

#[test]
fn runaway_loop_hits_the_step_limit() {
    let mut env = Env::default();
    let script = parse_script("x = 0; while x < 100001 do x = x + 1").expect("parse_script");
    assert!(run(&script, &mut env).is_err());
}

#[test]
fn big_layer_arbitrary_precision() {
    assert_eq!(
        eval_str("big(0.1) + big(0.2)"),
        Value::Big(BigDecimal::from_str("0.3").unwrap())
    );
}

#[test]
fn latex_input_parses() {
    let env = Env::default();
    let frac = parse_latex(r"\frac{1}{2} + \frac{1}{2}").expect("parse_latex");
    assert_eq!(eval(&frac, &env).expect("eval"), Value::float(1.0));
    let sqrt = parse_latex(r"\sqrt{16}").expect("parse_latex");
    assert_eq!(eval(&sqrt, &env).expect("eval"), Value::float(4.0));
    let nested = parse_latex(r"\frac{\frac{1}{2}}{2}").expect("parse_latex");
    assert_eq!(eval(&nested, &env).expect("eval"), Value::float(0.25));
}

#[test]
fn sampler_binds_x_and_evaluates() {
    let expr = parse("x ^ 2").expect("parse");
    let env = Env::default();
    let samples = sample(&expr, 0.0, 2.0, 3, &env).expect("sample");
    assert_eq!(
        samples,
        vec![
            Sample { x: 0.0, y: 0.0 },
            Sample { x: 1.0, y: 1.0 },
            Sample { x: 2.0, y: 4.0 },
        ]
    );
}

#[test]
fn sampler_skips_points_where_eval_errors() {
    let expr = parse("1 / x").expect("parse");
    let env = Env::default();
    let samples = sample(&expr, -1.0, 1.0, 3, &env).expect("sample");
    // x = -1, 0, 1 — the x = 0 point errors (division by zero) and is skipped
    assert_eq!(
        samples,
        vec![Sample { x: -1.0, y: -1.0 }, Sample { x: 1.0, y: 1.0 }]
    );
}

#[test]
fn parametric_sampler_binds_t() {
    let x = parse("t").expect("parse");
    let y = parse("t ^ 2").expect("parse");
    let samples = sample_parametric(&x, &y, 0.0, 2.0, 3, &Env::default()).expect("sample");
    assert_eq!(
        samples,
        vec![
            Sample { x: 0.0, y: 0.0 },
            Sample { x: 1.0, y: 1.0 },
            Sample { x: 2.0, y: 4.0 },
        ]
    );
}

#[test]
fn polar_sampler_converts_to_xy() {
    // r = 1 (unit circle): θ = 0 → (1, 0); θ = π/2 → (0, 1)
    let r = parse("1").expect("parse");
    let samples =
        sample_polar(&r, 0.0, std::f64::consts::FRAC_PI_2, 2, &Env::default()).expect("sample");
    assert_eq!(samples.len(), 2);
    assert!((samples[0].x - 1.0).abs() < 1e-12 && samples[0].y.abs() < 1e-12);
    assert!(samples[1].x.abs() < 1e-12 && (samples[1].y - 1.0).abs() < 1e-12);
}

#[test]
fn values_display_cleanly() {
    assert_eq!(Value::float(5.0).to_string(), "5");
    assert_eq!(Value::float(0.5).to_string(), "0.5");
    assert_eq!(Value::Bool(true).to_string(), "true");
    assert_eq!(eval_str("frac(1, 3)").to_string(), "1/3");
    assert_eq!(eval_str("dec(0.1) + dec(0.2)").to_string(), "0.3");
}

#[test]
fn session_submits_and_keeps_history() {
    let mut session = Session::new();
    assert_eq!(session.submit("x = 5; x + 1"), "= 6");
    assert_eq!(session.submit("x * 2"), "= 10");
    assert_eq!(session.history().len(), 2);
    assert_eq!(session.submit(""), "");
    assert_eq!(session.history().len(), 2);
}

#[test]
fn session_def_only_line_produces_no_output_and_records_source() {
    let mut session = Session::new();
    assert_eq!(session.submit("def f(x) = x ^ 2"), "");
    assert!(!session.history().iter().any(|h| h.contains("error")));
    assert_eq!(
        session.def_sources().get("f").map(String::as_str),
        Some("def f(x) = x ^ 2")
    );
}

#[test]
fn session_with_history_seeds_and_submit_appends() {
    let mut session = Session::with_history(vec!["old  = 1".to_string()]);
    assert_eq!(session.history().len(), 1);
    assert_eq!(session.submit("1 + 1"), "= 2");
    assert_eq!(session.history().len(), 2);
}

#[test]
fn session_tracks_last_submitted_line() {
    let mut session = Session::new();
    assert_eq!(session.last_line(), None);
    session.submit("x = 1; y = x + 1");
    assert_eq!(session.last_line(), Some("x = 1; y = x + 1"));
}

// ---------------------------------------------------------------------
// Scientific function library

/// Two floats close enough to agree (independent expected values are
/// literals or std constants, so equality is approximate by nature).
fn assert_close(actual: f64, expected: f64) {
    // relative tolerance: works from tiny values up to 170!
    let scale = expected.abs().max(1.0);
    assert!(
        (actual - expected).abs() <= 1e-10 * scale,
        "expected {expected}, got {actual}"
    );
}

fn eval_number(src: &str) -> f64 {
    match eval_str(src) {
        Value::Float(n) => n,
        other => panic!("expected a number from {src:?}, got {other:?}"),
    }
}

fn eval_err(src: &str) -> String {
    let env = Env::default();
    match parse(src) {
        Err(e) => e.to_string(),
        Ok(expr) => match eval(&expr, &env) {
            Err(e) => e.to_string(),
            Ok(v) => panic!("expected an error from {src:?}, got {v}"),
        },
    }
}

/// The display string of an evaluation (ADR-0043 tests use it to
/// assert complex results, which have no float reading).
fn eval_display(src: &str) -> String {
    let env = Env::default();
    match eval(&parse(src).expect("parse"), &env) {
        Ok(v) => format!("{v}"),
        Err(e) => panic!("expected a value from {src:?}, got {e}"),
    }
}

#[test]
fn trigonometric_functions_work_in_radians() {
    assert_close(eval_number("sin(0)"), 0.0);
    assert_close(eval_number("sin(pi / 2)"), 1.0);
    assert_close(eval_number("cos(0)"), 1.0);
    assert_close(eval_number("cos(pi)"), -1.0);
    assert_close(eval_number("tan(0)"), 0.0);
    assert_close(eval_number("tan(pi / 4)"), 1.0);
}

#[test]
fn inverse_trigonometric_functions_work() {
    assert_close(eval_number("asin(1)"), std::f64::consts::FRAC_PI_2);
    assert_close(eval_number("asin(0)"), 0.0);
    assert_close(eval_number("acos(1)"), 0.0);
    assert_close(eval_number("atan(1)"), std::f64::consts::FRAC_PI_4);
    assert_close(eval_number("atan(0)"), 0.0);
}

#[test]
fn inverse_trigonometric_functions_reject_out_of_domain() {
    // ADR-0043: out-of-domain reals now fall back to the principal
    // complex result - asin(2) is 1.5708 - 1.3170i, acos(-2) is
    // 3.1416 - 1.3170i - instead of a domain error.
    assert!(eval_display("asin(2)").contains('i'));
    assert!(eval_display("asin(-1.5)").contains('i'));
    assert!(eval_display("acos(-2)").contains('i'));
    assert_close(eval_number("re(asin(2))"), std::f64::consts::FRAC_PI_2);
}

#[test]
fn hyperbolic_functions_work() {
    // sinh(1) = 1.1752011936438014 (independent value)
    assert_close(eval_number("sinh(1)"), 1.1752011936438014);
    assert_close(eval_number("cosh(0)"), 1.0);
    assert_close(eval_number("cosh(1)"), 1.5430806348152437);
    assert_close(eval_number("tanh(0)"), 0.0);
    assert_close(eval_number("tanh(1)"), 0.7615941559557649);
}

#[test]
fn inverse_hyperbolic_functions_work_and_reject_out_of_domain() {
    assert_close(eval_number("asinh(0)"), 0.0);
    assert_close(eval_number("asinh(1)"), 0.881373587019543);
    assert_close(eval_number("acosh(1)"), 0.0);
    assert_close(eval_number("atanh(0)"), 0.0);
    // ADR-0043: out-of-domain reals fall back to complex values.
    assert!(eval_display("acosh(0.5)").contains('i'));
    assert!(eval_display("atanh(1)").contains('i'));
    assert!(eval_display("atanh(-1.1)").contains('i'));
}

#[test]
fn angle_conversions_between_degrees_and_radians() {
    assert_close(eval_number("deg(pi)"), 180.0);
    assert_close(eval_number("deg(pi / 2)"), 90.0);
    assert_close(eval_number("deg(1)"), 57.29577951308232);
    assert_close(eval_number("rad(180)"), std::f64::consts::PI);
    assert_close(eval_number("rad(90)"), std::f64::consts::FRAC_PI_2);
}

#[test]
fn sampler_can_graph_a_trig_expression() {
    let env = Env::default();
    let expr = parse("sin(x)").expect("parse");
    let points = sample(&expr, 0.0, std::f64::consts::TAU, 5, &env).expect("sample");
    let ys: Vec<f64> = points.iter().map(|p| p.y).collect();
    assert_close(ys[0], 0.0);
    assert_close(ys[1], 1.0);
    assert_close(ys[2], 0.0);
    assert_close(ys[3], -1.0);
    assert_close(ys[4], 0.0);
}

#[test]
fn atan2_gives_the_angle_of_a_point() {
    assert_close(eval_number("atan2(1, 1)"), std::f64::consts::FRAC_PI_4);
    assert_close(eval_number("atan2(0, -1)"), std::f64::consts::PI);
    assert_close(eval_number("atan2(-1, 0)"), -std::f64::consts::FRAC_PI_2);
}

#[test]
fn min_and_max_take_any_number_of_arguments() {
    assert_close(eval_number("min(2, 3)"), 2.0);
    assert_close(eval_number("min(4, 1, 3, 2)"), 1.0);
    assert_close(eval_number("max(4, 1, 3, 2)"), 4.0);
    assert_close(eval_number("max(7)"), 7.0);
    assert!(eval_err("min()").contains("expects"));
}

#[test]
fn exponential_and_natural_logarithm() {
    assert_close(eval_number("exp(0)"), 1.0);
    assert_close(eval_number("exp(1)"), std::f64::consts::E);
    assert_close(eval_number("ln(1)"), 0.0);
    assert_close(eval_number("ln(e)"), 1.0);
    // ADR-0043: ln of a non-positive real is the principal complex
    // logarithm: ln(0) is -inf (the real limit), ln(-1) is i*pi.
    assert_eq!(eval_display("ln(0)"), "-inf");
    assert_close(eval_number("im(ln(-1))"), std::f64::consts::PI);
}

#[test]
fn logarithms_in_common_bases() {
    // calculator convention: log is base 10 (the LOG key), ln is natural
    assert_close(eval_number("log(100)"), 2.0);
    assert_close(eval_number("log(1)"), 0.0);
    assert_close(eval_number("log2(8)"), 3.0);
    assert_close(eval_number("log2(1)"), 0.0);
    assert_close(eval_number("logb(2, 8)"), 3.0);
    assert_close(eval_number("logb(10, 1000)"), 3.0);
    // ADR-0043: log of a non-positive real falls back to the principal
    // complex logarithm (log(0) keeps the real -inf limit).
    assert_eq!(eval_display("log(0)"), "-inf");
    assert!(eval_display("log(-5)").contains('i'));
    assert!(eval_display("log2(0)").contains('i'));
    // logb stays real-only: a bad base is still a domain error.
    assert!(eval_err("logb(1, 5)").contains("domain"));
    assert!(eval_err("logb(-2, 8)").contains("domain"));
}

#[test]
fn cbrt_and_nth_root() {
    assert_close(eval_number("cbrt(27)"), 3.0);
    assert_close(eval_number("cbrt(-27)"), -3.0);
    assert_close(eval_number("root(3, 8)"), 2.0);
    assert_close(eval_number("root(2, 16)"), 4.0);
    assert_close(eval_number("root(3, -27)"), -3.0);
    assert!(eval_err("root(2, -4)").contains("domain"));
    assert!(eval_err("root(0, 8)").contains("domain"));
}

#[test]
fn hypot_computes_the_hypotenuse() {
    assert_close(eval_number("hypot(3, 4)"), 5.0);
    assert_close(eval_number("hypot(5, 12)"), 13.0);
}

#[test]
fn rounding_and_sign_functions() {
    assert_close(eval_number("abs(-3)"), 3.0);
    assert_close(eval_number("abs(3)"), 3.0);
    assert_close(eval_number("floor(2.7)"), 2.0);
    assert_close(eval_number("floor(-2.1)"), -3.0);
    assert_close(eval_number("ceil(2.1)"), 3.0);
    assert_close(eval_number("ceil(-2.7)"), -2.0);
    assert_close(eval_number("trunc(2.9)"), 2.0);
    assert_close(eval_number("trunc(-2.9)"), -2.0);
    // round is half away from zero, like a calculator
    assert_close(eval_number("round(2.5)"), 3.0);
    assert_close(eval_number("round(-2.5)"), -3.0);
    assert_close(eval_number("round(2.4)"), 2.0);
    assert_close(eval_number("sign(-5)"), -1.0);
    assert_close(eval_number("sign(5)"), 1.0);
    assert_close(eval_number("sign(0)"), 0.0);
}

#[test]
fn builtin_constants_tau_and_phi() {
    assert_close(eval_number("tau"), std::f64::consts::TAU);
    assert_close(eval_number("tau / 2"), std::f64::consts::PI);
    assert_close(eval_number("phi"), 1.618033988749895);
    // the golden ratio satisfies phi^2 = phi + 1
    assert_close(eval_number("phi ^ 2 - phi - 1"), 0.0);
}

#[test]
fn factorial_of_non_negative_integers() {
    assert_close(eval_number("fact(0)"), 1.0);
    assert_close(eval_number("fact(1)"), 1.0);
    assert_close(eval_number("fact(5)"), 120.0);
    assert_close(eval_number("fact(10)"), 3628800.0);
    // 170! is the largest factorial that fits in a double
    assert_close(eval_number("fact(170)"), 7.257415615307999e306);
    assert!(eval_err("fact(171)").contains("domain"));
    assert!(eval_err("fact(-1)").contains("domain"));
    assert!(eval_err("fact(2.5)").contains("expects integers"));
}

#[test]
fn combinations_and_permutations() {
    assert_close(eval_number("ncr(5, 2)"), 10.0);
    assert_close(eval_number("ncr(5, 0)"), 1.0);
    assert_close(eval_number("ncr(5, 5)"), 1.0);
    assert_close(eval_number("ncr(10, 3)"), 120.0);
    assert_close(eval_number("ncr(52, 5)"), 2598960.0);
    assert_close(eval_number("npr(5, 2)"), 20.0);
    assert_close(eval_number("npr(5, 5)"), 120.0);
    assert_close(eval_number("npr(10, 3)"), 720.0);
    assert!(eval_err("ncr(2, 5)").contains("domain"));
    assert!(eval_err("npr(3, 5)").contains("domain"));
    assert!(eval_err("ncr(5.5, 2)").contains("expects integers"));
}

#[test]
fn gcd_lcm_and_modulo() {
    assert_close(eval_number("gcd(12, 18)"), 6.0);
    assert_close(eval_number("gcd(0, 0)"), 0.0);
    assert_close(eval_number("gcd(-12, 18)"), 6.0);
    assert_close(eval_number("lcm(4, 6)"), 12.0);
    assert_close(eval_number("lcm(0, 5)"), 0.0);
    assert_close(eval_number("mod(7, 3)"), 1.0);
    assert_close(eval_number("mod(-7, 3)"), -1.0);
    assert_close(eval_number("mod(7, -3)"), 1.0);
    assert!(eval_err("mod(5, 0)").contains("zero"));
    assert!(eval_err("gcd(2.5, 3)").contains("expects integers"));
}

#[test]
fn statistics_sum_product_and_mean() {
    assert_close(eval_number("sum(1, 2, 3)"), 6.0);
    assert_close(eval_number("sum(5)"), 5.0);
    assert_close(eval_number("product(2, 3, 4)"), 24.0);
    assert_close(eval_number("product(7)"), 7.0);
    assert_close(eval_number("mean(1, 2, 3)"), 2.0);
    assert_close(eval_number("mean(2, 4)"), 3.0);
    assert!(eval_err("sum()").contains("expects"));
    assert!(eval_err("mean()").contains("expects"));
}

#[test]
fn statistics_median_sorts_before_selecting() {
    assert_close(eval_number("median(3, 1, 2)"), 2.0);
    assert_close(eval_number("median(1, 2, 3, 4)"), 2.5);
    assert_close(eval_number("median(9)"), 9.0);
    assert_close(eval_number("median(-5, -1, -3)"), -3.0);
}

#[test]
fn statistics_variance_and_stdev_are_population() {
    assert_close(eval_number("variance(2, 4)"), 1.0);
    assert_close(eval_number("variance(1, 2, 3)"), 0.6666666666666666);
    assert_close(eval_number("stdev(2, 4)"), 1.0);
    assert_close(eval_number("stdev(1, 2, 3)"), 0.816496580927726);
}

#[test]
fn postfix_factorial_operator() {
    assert_close(eval_number("5!"), 120.0);
    assert_close(eval_number("0!"), 1.0);
    assert_close(eval_number("(2 + 1)!"), 6.0);
    // factorial binds tighter than ^ and unary -
    assert_close(eval_number("3! ^ 2"), 36.0);
    assert_close(eval_number("2 ^ 3!"), 64.0);
    assert_close(eval_number("-5!"), -120.0);
    // (4!)! = 24!
    assert_close(eval_number("4!!"), 6.204484017332394e23);
    assert_close(eval_number("fact(5) + 5!"), 240.0);
    assert_eq!(eval_str("5! == 120"), Value::Bool(true));
    // ! and != stay distinct tokens
    assert_eq!(eval_str("5! != 100"), Value::Bool(true));
    assert!(eval_err("(-1)!").contains("domain"));
    assert!(eval_err("3.5!").contains("expects integers"));
}

#[test]
fn scientific_notation_literals() {
    assert_close(eval_number("1e3"), 1000.0);
    assert_close(eval_number("2E3"), 2000.0);
    assert_close(eval_number("1e-5"), 0.00001);
    assert_close(eval_number("2.5E-2"), 0.025);
    assert_close(eval_number("6.02e23"), 6.02e23);
    assert_close(eval_number("1e3 + 1"), 1001.0);
    assert_close(eval_number("1e2 * 2"), 200.0);
    // a bare e is still Euler's number, and 2e without an exponent is
    // still two separate tokens (an error)
    assert_close(eval_number("e"), std::f64::consts::E);
    assert!(eval_str_checked("2e").is_err());
    assert!(eval_str_checked("2eggs").is_err());
}

/// Parse+eval returning a Result, for error-path tests.
fn eval_str_checked(src: &str) -> Result<Value, epher_core::EpherError> {
    let env = Env::default();
    eval(&parse(src)?, &env)
}

// --- user-defined constants (ADR-0012) ----------------------------------

/// Run script lines against a fresh Env, returning the last value's display.
fn run_script(src: &str) -> Result<Value, epher_core::EpherError> {
    let mut env = Env::default();
    run(&parse_script(src)?, &mut env)?.ok_or(epher_core::EpherError::Parse("no value".into()))
}

#[test]
fn constant_defines_and_evaluates_to_its_value() {
    assert_eq!(run_script("const tax = 0.2").unwrap(), Value::float(0.2));
}

#[test]
fn constant_is_usable_in_later_statements() {
    assert_eq!(
        run_script("const tax = 0.2; 100 * (1 + tax)").unwrap(),
        Value::float(120.0)
    );
}

#[test]
fn constant_expression_is_evaluated_once_at_definition() {
    assert_eq!(
        run_script("const r = 2; const area = pi * r ^ 2; area").unwrap(),
        run_script("pi * 4").unwrap()
    );
}

#[test]
fn constant_cannot_be_reassigned() {
    let err = run_script("const tax = 0.2; tax = 0.25").unwrap_err();
    assert_eq!(err.to_string(), "cannot assign to constant tax");
}

#[test]
fn constant_cannot_be_redefined() {
    let err = run_script("const tax = 0.2; const tax = 0.25").unwrap_err();
    assert_eq!(err.to_string(), "constant already defined: tax");
}

#[test]
fn constant_cannot_take_a_variables_name() {
    let err = run_script("x = 5; const x = 6").unwrap_err();
    assert_eq!(
        err.to_string(),
        "cannot define constant x: the name is already a variable"
    );
}

#[test]
fn variable_cannot_be_defined_where_a_constant_exists() {
    // the reverse direction is the assign guard
    let err = run_script("const g = 9.81; g = 9.8").unwrap_err();
    assert_eq!(err.to_string(), "cannot assign to constant g");
}

#[test]
fn constant_is_visible_inside_functions_like_pi() {
    // variables are not visible in function bodies; constants are (ADR-0012)
    assert_eq!(
        run_script("const g = 9.81; def weight(m) = m * g; weight(80)").unwrap(),
        run_script("80 * 9.81").unwrap()
    );
}

#[test]
fn function_parameter_shadows_a_constant() {
    assert_eq!(
        run_script("const x = 100; def f(x) = x + 1; f(1)").unwrap(),
        Value::float(2.0)
    );
}

#[test]
fn constant_survives_across_statements_and_in_while_bodies() {
    assert_eq!(
        run_script("const step = 2; x = 0; while x < 10 do x = x + step; x").unwrap(),
        Value::float(10.0)
    );
}

#[test]
fn assignment_inside_a_loop_to_a_constant_is_an_error() {
    let err = run_script("const x = 0; while x < 5 do x = x + 1").unwrap_err();
    assert_eq!(err.to_string(), "cannot assign to constant x");
}

#[test]
fn user_constant_can_shadow_a_builtin() {
    // same as variables today: bindings and user constants win over pi/e
    assert_eq!(
        run_script("const pi = 3; pi * 2").unwrap(),
        Value::float(6.0)
    );
}

#[test]
fn const_prefixed_variable_names_are_still_assignments() {
    assert_eq!(
        run_script("const_tax = 5; const_tax").unwrap(),
        Value::float(5.0)
    );
}

#[test]
fn session_tracks_const_sources_for_save() {
    let mut session = Session::new();
    session.submit("const tax = 0.2");
    session.submit("tax");
    assert_eq!(
        session.const_sources().get("tax").map(String::as_str),
        Some("const tax = 0.2")
    );
    assert!(session.const_sources().get("const_tax").is_none());
}

#[test]
fn const_in_a_fresh_child_env_only_shadows_via_params() {
    // new_child copies constants: a direct env probe of the same guarantee
    let mut env = Env::default();
    run(
        &parse_script("const k = 7; def f() = k").expect("parse"),
        &mut env,
    )
    .unwrap();
    assert_eq!(
        run(&parse_script("f()").unwrap(), &mut env).unwrap(),
        Some(Value::float(7.0))
    );
}

#[test]
fn clear_history_empties_the_list_but_keeps_definitions() {
    let mut s = Session::new();
    s.submit("def f(x) = x + 1");
    s.submit("const a = 3");
    s.submit("f(a)");
    assert_eq!(s.history().len(), 3);
    s.clear_history();
    assert!(s.history().is_empty());
    // The environment survives: definitions and constants still work.
    assert_eq!(s.submit("f(a)"), "= 4");
}

// ADR-0001 seam unification: newlines and `;` are the same separator.
#[test]
fn newlines_separate_statements_like_semicolons() {
    let mut env = Env::default();
    let script = parse_script("x = 5\nx + 1").expect("parse_script");
    let result = run(&script, &mut env).expect("run").expect("value");
    assert_eq!(result, Value::float(6.0));
}

#[test]
fn newlines_and_semicolons_mix_freely() {
    let mut env = Env::default();
    let script = parse_script("x = 5;\ny = x + 1\nx * y").expect("parse_script");
    let result = run(&script, &mut env).expect("run").expect("value");
    assert_eq!(result, Value::float(30.0));
}

#[test]
fn windows_line_endings_are_one_separator() {
    let mut env = Env::default();
    let script = parse_script("x = 5\r\ny = x + 1\r\nx * y").expect("parse_script");
    let result = run(&script, &mut env).expect("run").expect("value");
    assert_eq!(result, Value::float(30.0));
}

#[test]
fn redundant_separators_are_skipped() {
    let mut env = Env::default();
    let script = parse_script("x = 5;;\n\ny = x + 1;").expect("parse_script");
    let result = run(&script, &mut env).expect("run").expect("value");
    assert_eq!(result, Value::float(6.0));
}

#[test]
fn run_all_collects_every_statement_value() {
    let mut env = Env::default();
    let script = parse_script("x = 10; y = x + 5; x + y").expect("parse_script");
    let values = run_all(&script, &mut env).expect("run_all");
    assert_eq!(
        values,
        vec![Value::float(10.0), Value::float(15.0), Value::float(25.0)]
    );
}

#[test]
fn newline_inside_an_expression_is_an_error() {
    // A newline is always a statement boundary: no expression spans lines.
    assert!(parse("1 +\n2").is_err());
}

// ===== ans: the previous answer (ADR-0021) =====

#[test]
fn ans_holds_the_previous_statement_value() {
    let mut env = Env::default();
    let script = parse_script("2 + 3; ans * 2").unwrap();
    let values = run_all(&script, &mut env).unwrap();
    assert_eq!(values, vec![Value::float(5.0), Value::float(10.0)]);
    // the binding is an ordinary variable in the environment
    assert_eq!(env.get("ans"), Some(&Value::float(10.0)));
}

#[test]
fn ans_before_any_result_is_unknown() {
    let env = Env::default();
    assert!(eval(&parse("ans").unwrap(), &env).is_err());
}

#[test]
fn definitions_and_errors_leave_ans_untouched() {
    let mut env = Env::default();
    let script = parse_script("def f(x) = x; 7; ans").unwrap();
    let values = run_all(&script, &mut env).unwrap();
    // the definition produces no value; ans is the 7 from statement two
    assert_eq!(values, vec![Value::float(7.0), Value::float(7.0)]);

    let mut env = Env::default();
    let script = parse_script("3; 1 / 0").unwrap();
    assert!(run_all(&script, &mut env).is_err());
    // the error statement did not clobber the earlier answer
    assert_eq!(env.get("ans"), Some(&Value::float(3.0)));
}

#[test]
fn ans_updates_inside_while_bodies() {
    let mut env = Env::default();
    // x runs 1,2,3; the body's last statement value is 3, so ans = 3
    let script = parse_script("x = 1; while x < 3 do x = x + 1; ans").unwrap();
    let values = run_all(&script, &mut env).unwrap();
    assert_eq!(values, vec![Value::float(1.0), Value::float(3.0)]);
}

#[test]
fn ans_can_be_assigned_like_any_variable() {
    let mut env = Env::default();
    let script = parse_script("ans = 9; ans").unwrap();
    let values = run_all(&script, &mut env).unwrap();
    assert_eq!(values, vec![Value::float(9.0), Value::float(9.0)]);
}

// ===== number bases: literals and conversion (ADR-0022) =====

#[test]
fn based_literals_parse_to_plain_numbers() {
    assert_eq!(eval_str("0b1010"), Value::float(10.0));
    assert_eq!(eval_str("0B10"), Value::float(2.0));
    assert_eq!(eval_str("0o17"), Value::float(15.0));
    assert_eq!(eval_str("0O17"), Value::float(15.0));
    assert_eq!(eval_str("0xFF"), Value::float(255.0));
    assert_eq!(eval_str("0Xff"), Value::float(255.0));
    assert_eq!(eval_str("0xFF + 0b1"), Value::float(256.0));
    assert_eq!(eval_str("0x10 * 2"), Value::float(32.0));
}

#[test]
fn bin_oct_hex_convert_integers_to_prefixed_strings() {
    assert_eq!(eval_str("bin(10)"), Value::Str("0b1010".into()));
    assert_eq!(eval_str("oct(10)"), Value::Str("0o12".into()));
    assert_eq!(eval_str("hex(255)"), Value::Str("0xff".into()));
    assert_eq!(eval_str("hex(-42)"), Value::Str("-0x2a".into()));
    // exact layers convert exactly, and the answer feeds back in
    assert_eq!(eval_str("bin(0b11111111)"), Value::Str("0b11111111".into()));
    assert_eq!(eval_str("hex(frac(255, 1))"), Value::Str("0xff".into()));
    assert_eq!(eval_str("hex(dec(42))"), Value::Str("0x2a".into()));
    assert_eq!(
        eval_str("hex(big(10 ^ 20))"),
        Value::Str("0x56bc75e2d63100000".into())
    );
    assert_eq!(eval_str("hex(0x2a)"), Value::Str("0x2a".into()));
}

#[test]
fn base_conversion_rejects_non_integers() {
    assert!(evaluate("bin(0.5)").is_err());
    assert!(evaluate("hex(frac(1, 2))").is_err());
    assert!(evaluate("oct(true)").is_err());
    assert!(evaluate("hex(1, 2)").is_err());
    assert!(evaluate("bin()").is_err());
}

#[test]
fn bad_based_literals_are_parse_errors() {
    assert!(parse("0b2").is_err());
    assert!(parse("0b").is_err());
    assert!(parse("0x").is_err());
    assert!(parse("0o8").is_err());
}

#[test]
fn ans_holds_a_converted_string_like_any_value() {
    let mut env = Env::default();
    let script = parse_script("hex(255); ans").unwrap();
    let values = run_all(&script, &mut env).unwrap();
    assert_eq!(
        values,
        vec![Value::Str("0xff".into()), Value::Str("0xff".into())]
    );
}

/// ADR-0031: a history pick loads the expression — everything before the
/// last recorded answer suffix — so the user can edit and re-run it.
#[test]
fn history_expression_strips_the_answer_suffix() {
    use epher_core::history_expression;
    assert_eq!(history_expression("2 + 2  = 4"), "2 + 2");
    assert_eq!(history_expression("x = 10; x + 5  = 15"), "x = 10; x + 5");
    // The last suffix wins: single-space `=` in the expression itself
    // (assignments) is never mistaken for the double-space record marker.
    assert_eq!(history_expression("x  = 10  = 15"), "x  = 10");
    // Errors and warnings carry their own suffixes.
    assert_eq!(history_expression("1/0  error: division by zero"), "1/0");
    assert_eq!(history_expression("1/0  warning: unstable"), "1/0");
    // Entries without an answer suffix pass through untouched.
    assert_eq!(history_expression("graph x ^ 2"), "graph x ^ 2");
    assert_eq!(history_expression("def f(x) = x * x"), "def f(x) = x * x");
    assert_eq!(history_expression("2 + 2"), "2 + 2");
    // Multi-line script entries are verbatim: never suffix-stripped, even
    // when a line inside carries the double-space marker (ADR-0027).
    let script = "x = 10\ny  =  x + 5";
    assert_eq!(history_expression(script), script);
}

/// ADR-0031: the fine-control offsets — 0 leaves the pose unchanged; the
/// horizontal slider spans ±π of yaw, the vertical one keeps the full
/// −1..1 range live at the default pose, and zoom scales the camera.
#[test]
fn view3d_offsets_map_to_the_pose() {
    use epher_core::graph::View3D;
    let base = View3D::default();
    assert_eq!(base.with_offsets(0.0, 0.0, 0.0), base);
    let turned = base.with_offsets(0.5, -0.5, 1.0);
    assert!((turned.yaw - (0.8 + 0.5 * std::f64::consts::PI)).abs() < 1e-12);
    assert!((turned.pitch - (0.6 - 0.4)).abs() < 1e-12);
    // ADR-0038: the zoom slider spans two decades each way - +1 zooms in
    // 100× (a single object), -1 zooms out 100× (every object fits).
    assert!((turned.camera - 0.3).abs() < 1e-12);
    assert!((base.with_offsets(0.0, 0.0, -1.0).camera - 3000.0).abs() < 1e-9);
    // The vertical range's top lands exactly on the pitch clamp at the
    // default pose; beyond it the clamp holds.
    let up = base.with_offsets(0.0, 1.0, 0.0);
    assert!((up.pitch - 1.4).abs() < 1e-12);
    let over = base.with_offsets(0.0, 2.0, 0.0);
    assert!((over.pitch - 1.4).abs() < 1e-12);
    // ADR-0034: the mouse-wheel zoom sets the camera distance directly
    // (clamped away from zero — a zero camera degenerates the projection).
    let near = base.with_camera(12.0);
    assert!((near.camera - 12.0).abs() < 1e-12);
    // The floor guards the projection, not the user (ADR-0038): wheel and
    // pinch zoom in as far as they want.
    assert!((base.with_camera(0.0).camera - 0.01).abs() < 1e-12);
    assert_eq!(base.with_camera(-3.0).yaw, base.yaw);
}

/// The spin phase (ADR-0032) adds accumulated rotation with no pitch
/// clamp — a vertical spin is a full revolution — and applies the zoom.
#[test]
fn view3d_spin_phase_adds_unclamped_rotation() {
    use epher_core::graph::View3D;
    let base = View3D::default();
    assert_eq!(base.with_spin_phase(0.0, 0.0, 0.0), base);
    let spun = base.with_spin_phase(0.7, 2.9, -1.0);
    assert!((spun.yaw - (0.8 + 0.7)).abs() < 1e-12);
    // 0.6 + 2.9 = 3.5 rad: far past the static 1.4 clamp, unclamped.
    assert!((spun.pitch - 3.5).abs() < 1e-12);
    // zoom −1 under the ADR-0038 scale: the window grows 100×.
    assert!((spun.camera - 3000.0).abs() < 1e-9);
    // sin/cos continuity: wrapping the phase by a full turn changes
    // nothing in the projected pose.
    let a = View3D {
        yaw: 0.3,
        pitch: 4.1,
        camera: 30.0,
    };
    let b = View3D {
        yaw: 0.3 + std::f64::consts::TAU,
        pitch: 4.1 + std::f64::consts::TAU,
        camera: 30.0,
    };
    let (ax, ay, _) = epher_core::graph::project_point(1.0, 2.0, 3.0, &a);
    let (bx, by, _) = epher_core::graph::project_point(1.0, 2.0, 3.0, &b);
    assert!((ax - bx).abs() < 1e-9 && (ay - by).abs() < 1e-9);
}

// ===== The shared session snapshot (ADR-0010 amendment) =====

#[test]
fn session_bindings_round_trip_through_json_and_restore() {
    use epher_core::{Session, ValueBindings};
    let mut s = Session::new();
    s.submit("x = 5");
    s.submit("x * 2"); // sets ans = 10
    let json = serde_json::to_string(s.bindings()).expect("serialize bindings");
    let back: ValueBindings = serde_json::from_str(&json).expect("deserialize bindings");
    let mut s2 = Session::new();
    s2.restore_bindings(&back);
    // restoring bindings never touches history — that travels in its own
    // setting
    assert!(s2.history().is_empty());
    assert_eq!(s2.submit("x + ans").trim(), "= 15");
}

#[test]
fn every_value_variant_round_trips_through_json() {
    use bigdecimal::BigDecimal;
    use epher_core::Value;
    use num_bigint::BigInt;
    use num_complex::Complex;
    use num_rational::BigRational;
    use rust_decimal::Decimal;
    let values = vec![
        Value::float(1.5),
        Value::float(-0.0),
        Value::Bool(true),
        Value::Str("0xff".to_string()),
        Value::Complex(Complex::new(1.0, 2.0)),
        Value::Rational(BigRational::new(BigInt::from(1), BigInt::from(3))),
        Value::Decimal(Decimal::new(1234, 2)),
        Value::Big(BigDecimal::from(42)),
        Value::Quantity {
            value: 4.787_131_862_4e11,
            dims: [1, 0, 0, 0, 0, 0, 0],
            unit: Some(("AU".to_string(), 1.495_978_707e11)),
        },
    ];
    for v in values {
        let json = serde_json::to_string(&v).expect("serialize");
        let back: Value = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(v, back, "json was: {json}");
    }
}

// ===== Unit-suffix literals (ADR-0037) =====
//
// A number immediately followed by a unit token multiplies by the unit's
// SI factor and evaluates to a plain Float in SI units - metres, radians,
// seconds, watts per square metre hertz. The factors are grammar-level,
// so user shadowing cannot change what a literal means.

fn approx(name: &str, text: &str, expected: f64) {
    match epher_core::evaluate(text) {
        Ok(v) => match v {
            epher_core::Value::Float(x) => assert!(
                (x - expected).abs() <= 1e-9 * expected.abs().max(1.0),
                "{name}: {text} = {x}, expected {expected}"
            ),
            // A suffix literal is a quantity now (ADR-0046); the SI
            // value is what the old factor tests asserted.
            epher_core::Value::Quantity { value, .. } => assert!(
                (value - expected).abs() <= 1e-9 * expected.abs().max(1.0),
                "{name}: {text} = {value}, expected {expected}"
            ),
            other => panic!("{name}: {text} produced {other:?}, expected a float"),
        },
        Err(e) => panic!("{name}: {text} failed: {e}"),
    }
}

#[test]
fn unit_literals_multiply_by_the_si_factor() {
    let au = 1.495_978_707e11;
    approx("astronomical unit spaced", "3.2 AU", 3.2 * au);
    approx("astronomical unit tight", "3.2AU", 3.2 * au);
    approx("parsec", "1 pc", 3.085_677_581_491_367_3e16);
    approx("light year", "1 ly", 9.460_730_472_580_8e15);
    approx("day", "1 d", 86400.0);
    approx("hour", "5 hr", 18000.0);
    approx("minute", "5 min", 300.0);
    approx("julian year", "2 yr", 2.0 * 31_557_600.0);
    approx("jansky", "1 Jy", 1e-26);
}

#[test]
fn angle_literals_come_out_in_radians() {
    // Angles are dimensionless quantities: the value is in radians and
    // the display is a plain number (ADR-0046).
    approx("degree", "30 deg", std::f64::consts::PI / 6.0);
    approx("degree tight", "30deg", std::f64::consts::PI / 6.0);
    approx("arcminute", "1 arcmin", std::f64::consts::PI / 10800.0);
    approx("arcsecond", "1 arcsec", std::f64::consts::PI / 648000.0);
    assert_eq!(
        epher_core::format_value(
            &epher_core::evaluate("30 deg").unwrap(),
            &epher_core::DisplayPrefs::default()
        ),
        "0.523598775598"
    );
    // and they convert like any quantity
    approx("deg to rad", "30 deg in rad", std::f64::consts::PI / 6.0);
}

#[test]
fn suffixes_compose_with_functions() {
    // the spec's worked example: sin of 30 degrees is one half
    approx("sin of 30 deg", "sin(30 deg)", 0.5);
    approx("cos of an arcminute", "cos(1 arcmin)", 1.0 - 4.222e-8);
}

#[test]
fn user_shadowing_cannot_change_a_unit_literal() {
    // suffix factors are grammar-level constants: neither a user
    // constant nor a variable changes what `2 AU` means (ADR-0037)
    let script = "const au = 3; const deg = 9; 2 AU";
    match epher_core::run_all(
        &epher_core::parse_script(script).expect("parses"),
        &mut epher_core::Env::default(),
    ) {
        Ok(values) => {
            let last = values.last().expect("a value");
            match last {
                epher_core::Value::Quantity { value, dims, unit } => {
                    assert!((value - 2.0 * 1.495_978_707e11).abs() < 1e-3, "SI value");
                    assert_eq!(*dims, [1, 0, 0, 0, 0, 0, 0], "length dims");
                    assert_eq!(unit.as_ref().map(|(u, _)| u.as_str()), Some("AU"));
                }
                other => panic!("2 AU produced {other:?}"),
            }
        }
        Err(e) => panic!("script failed: {e}"),
    }
    let mut env = epher_core::Env::default();
    epher_core::run(
        &epher_core::parse_script("const deg = 9").expect("parses"),
        &mut env,
    )
    .expect("runs");
    let expr = epher_core::parse("sin(30 deg)").expect("parses");
    match epher_core::eval(&expr, &env).expect("evals") {
        epher_core::Value::Float(x) => assert!((x - 0.5).abs() < 1e-12),
        other => panic!("expected a float, got {other:?}"),
    }
}

#[test]
fn unit_tokens_are_reserved_in_suffix_position_but_calls_stay_calls() {
    // `min` the suffix and `min` the function coexist by position
    approx("suffix", "5 min", 300.0);
    // ... and `5 min + 5 min` keeps the unit, `min(3, 7)` stays a call
    match epher_core::evaluate("5 min + 5 min") {
        Ok(epher_core::Value::Quantity { value, unit, .. }) => {
            assert_eq!(value, 600.0);
            assert_eq!(unit.as_ref().map(|(u, _)| u.as_str()), Some("min"));
        }
        other => panic!("5 min + 5 min produced {other:?}"),
    }
    match epher_core::evaluate("min(3, 7)") {
        Ok(epher_core::Value::Float(x)) => assert_eq!(x, 3.0),
        other => panic!("min(3, 7) produced {other:?}"),
    }
    // an Ident followed by `(` is always a call, so the number before it
    // is trailing input - no implicit multiplication is born here
    assert!(epher_core::evaluate("30 deg(x)").is_err());
    // `h` is Planck's constant, not the hour: `5 h` is a parse error
    assert!(epher_core::evaluate("5 h").is_err());
    // unknown names after a number stay errors (no implicit multiplication)
    assert!(epher_core::evaluate("2 pi").is_err());
    // case-sensitive: `D` is not the day token
    assert!(epher_core::evaluate("1 D").is_err());
}

#[test]
fn unit_literals_work_in_scripts_and_domains() {
    let mut env = epher_core::Env::default();
    let script = epher_core::parse_script("x = 5 hr; x + 1 d").expect("parses");
    let values = epher_core::run_all(&script, &mut env).expect("runs");
    match values.last() {
        Some(epher_core::Value::Quantity { value, dims, .. }) => {
            assert!((value - (86400.0 + 18000.0)).abs() < 1e-9, "SI value");
            assert_eq!(*dims, [0, 0, 1, 0, 0, 0, 0], "time dims");
        }
        other => panic!("5 hr + 1 d produced {other:?}"),
    }
    // identical display units survive addition
    let mut env2 = epher_core::Env::default();
    let script2 = epher_core::parse_script("y = 5 hr + 3 hr").expect("parses");
    let values2 = epher_core::run_all(&script2, &mut env2).expect("runs");
    match values2.last() {
        Some(epher_core::Value::Quantity { value, unit, .. }) => {
            assert_eq!(*value, 28800.0);
            assert_eq!(unit.as_ref().map(|(u, _)| u.as_str()), Some("hr"));
        }
        other => panic!("5 hr + 3 hr produced {other:?}"),
    }
}

// ===== Astronomy constants (ADR-0037) =====

#[test]
fn astronomy_constants_resolve_like_pi_and_are_shadowable() {
    approx("c", "c", 2.997_924_58e8);
    approx("g", "g", 9.80665);
    approx("h", "h", 6.626_070_15e-34);
    approx(
        "h_bar",
        "h_bar",
        6.626_070_15e-34 / (2.0 * std::f64::consts::PI),
    );
    approx("k_b", "k_b", 1.380_649e-23);
    approx("sigma_sb", "sigma_sb", 5.670_374_419e-8);
    approx("au", "au", 1.495_978_707e11);
    approx("pc", "pc", 3.085_677_581_491_367_3e16);
    approx("ly", "ly", 9.460_730_472_580_8e15);
    approx("m_sun", "m_sun", 1.988_47e30);
    approx("r_sun", "r_sun", 6.957e8);
    approx("l_sun", "l_sun", 3.828e26);
    approx("m_earth", "m_earth", 5.9722e24);
    approx("r_earth", "r_earth", 6.371e6);
    // shadowable like pi (ADR-0037 keeps the existing resolution order)
    let mut env = epher_core::Env::default();
    epher_core::run(
        &epher_core::parse_script("const c = 42").expect("parses"),
        &mut env,
    )
    .expect("runs");
    let expr = epher_core::parse("c").expect("parses");
    assert_eq!(
        epher_core::eval(&expr, &env).expect("evals"),
        epher_core::Value::float(42.0)
    );
}

// ===== Time, angle, and optics functions (ADR-0037) =====

#[test]
fn jd_and_mjd_convert_calendar_dates() {
    // independent source: J2000.0 is 2000-01-01 12:00 TT = JD 2451545.0
    approx("J2000 epoch", "jd(2000, 1, 1, 12)", 2451545.0);
    approx(
        "jd with fractional hour",
        "jd(2000, 1, 1, 12.5)",
        2451545.020_833_333,
    );
    approx("jd defaults to midnight", "jd(2000, 1, 1)", 2451544.5);
    // MJD epoch: 1858-11-17 00:00 is MJD 0
    approx("MJD epoch", "mjd(1858, 11, 17)", 0.0);
    assert!(epher_core::evaluate("jd(2000, 13, 1)").is_err());
    assert!(epher_core::evaluate("jd(2000, 0, 1)").is_err());
    assert!(epher_core::evaluate("jd(2000, 1, 32)").is_err());
}

#[test]
fn now_reads_the_host_clock_as_a_julian_date() {
    match epher_core::evaluate("now()") {
        Ok(epher_core::Value::Float(x)) => {
            // mid-2026 is JD 2461xxx; the bound has decades of slack
            assert!(x > 2461000.0, "now() = {x} looks pre-2026");
            assert!(x < 2500000.0, "now() = {x} looks past year 2200");
        }
        other => panic!("now() produced {other:?}"),
    }
}

#[test]
fn hms_and_dms_pairs_convert_both_ways() {
    // six hours of right ascension is 90 degrees
    approx("hms2deg(6,0,0)", "hms2deg(6, 0, 0)", 90.0);
    approx(
        "hms2deg(6,42,14.32)",
        "hms2deg(6, 42, 14.32)",
        (6.0 + 42.0 / 60.0 + 14.32 / 3600.0) * 15.0,
    );
    approx("dms2deg", "dms2deg(23, 26, 30)", 23.441_666_667);
    // southern declinations carry the sign of the degrees argument
    approx("dms2deg negative", "dms2deg(-23, 26, 30)", -23.441_666_667);
    match epher_core::evaluate("deg2hms(90)") {
        Ok(epher_core::Value::Str(s)) => assert_eq!(s, "6h 0m 0s"),
        other => panic!("deg2hms(90) produced {other:?}"),
    }
    // Meeus's worked angle 100.55966 degrees spells 6h 42m 14s
    match epher_core::evaluate("deg2hms(100.55966)") {
        Ok(epher_core::Value::Str(s)) => assert_eq!(s, "6h 42m 14s"),
        other => panic!("deg2hms produced {other:?}"),
    }
    match epher_core::evaluate("deg2dms(23.441666)") {
        Ok(epher_core::Value::Str(s)) => assert_eq!(s, "23\u{b0} 26' 30\""),
        other => panic!("deg2dms produced {other:?}"),
    }
    // negative angles sign the degrees component
    match epher_core::evaluate("deg2dms(-23.441666)") {
        Ok(epher_core::Value::Str(s)) => assert_eq!(s, "-23\u{b0} 26' 30\""),
        other => panic!("deg2dms produced {other:?}"),
    }
    // seconds rounding carries into minutes: 90.0001 deg
    match epher_core::evaluate("deg2hms(90.006)") {
        Ok(epher_core::Value::Str(s)) => assert_eq!(s, "6h 0m 1s"),
        other => panic!("deg2hms produced {other:?}"),
    }
}

#[test]
fn sidereal_time_tracks_the_stars() {
    // GMST at J2000 (2000-01-01 12h UT) is 18.6973746 h (Meeus 12.4).
    // The facade computes GAST, which differs by the equation of the
    // equinoxes (at most about 1.2 s of time), so the envelope is
    // generous; the longitude relation is asserted exactly.
    let at_greenwich = match epher_core::evaluate("lst(jd(2000, 1, 1, 12), 0)") {
        Ok(epher_core::Value::Float(x)) => x,
        other => panic!("lst produced {other:?}"),
    };
    assert!(
        (at_greenwich - 18.697_374_6).abs() < 0.002,
        "lst at Greenwich = {at_greenwich}, GMST is 18.6973746"
    );
    let east = match epher_core::evaluate("lst(jd(2000, 1, 1, 12), 90)") {
        Ok(epher_core::Value::Float(x)) => x,
        other => panic!("lst produced {other:?}"),
    };
    // 90 deg east is 6 hours ahead, wrapping past 24
    assert!((east - (at_greenwich + 6.0 - 24.0)).abs() < 1e-12);
    let west = match epher_core::evaluate("lst(jd(2000, 1, 1, 12), -90)") {
        Ok(epher_core::Value::Float(x)) => x,
        other => panic!("lst produced {other:?}"),
    };
    assert!((west - (at_greenwich - 6.0)).abs() < 1e-12);
}

#[test]
fn delta_t_is_the_earth_clock_correction() {
    // the Espenak-Meeus polynomial that solar-ephemeris carries puts
    // TT - UT1 near 69 s in 2015 (the IERS measured value was 67.64 s;
    // the polynomial band is honest to a few seconds, which is the
    // accuracy the guide documents)
    match epher_core::evaluate("delta_t(jd(2015, 1, 1))") {
        Ok(epher_core::Value::Float(x)) => assert!(
            (60.0..75.0).contains(&x),
            "delta_t(2015) = {x} is outside the Espenak-Meeus band"
        ),
        other => panic!("delta_t produced {other:?}"),
    }
    assert!(epher_core::evaluate("delta_t(jd(1900, 1, 1))").is_ok());
}

#[test]
fn optics_helpers() {
    // airmass at the zenith is 1; at 30 deg altitude it is sec(60) = 2
    approx("airmass zenith", "airmass(90)", 1.0);
    approx("airmass 30 deg", "airmass(30)", 2.0);
    assert!(epher_core::evaluate("airmass(0)").is_err());
    // Dawes' resolving power: 116/D arcseconds
    approx("dawes 100mm", "dawes(100)", 1.16);
    assert!(epher_core::evaluate("dawes(0)").is_err());
    // distance modulus: mu = 25 is a megaparsec
    approx("dist mod 25", "dist_mod(25)", 1e6);
    // Kepler's equation, Meeus's worked example 30.a: M = 5 deg, e = 0.1
    // gives E = 0.096953 rad = 5.55457 deg (the example's own rounding)
    match epher_core::evaluate("kepler(5, 0.1)") {
        Ok(epher_core::Value::Float(x)) => {
            assert!((x - 5.554_57).abs() < 1e-4, "kepler(5, 0.1) = {x}")
        }
        other => panic!("kepler produced {other:?}"),
    }
    // at half an orbit the eccentric anomaly equals the mean anomaly
    approx("kepler half orbit", "kepler(180, 0.8)", 180.0);
    approx("kepler zero", "kepler(0, 0.9)", 0.0);
}

#[test]
fn magnitude_and_jansky_convert_both_ways() {
    // the AB system's zero point: magnitude 0 is 3631 Jy
    approx("mag2jy(0)", "mag2jy(0)", 3631.0);
    approx("mag2jy(20)", "mag2jy(20)", 3631.0 * 1e-8);
    approx("jy2mag zero point", "jy2mag(3631)", 0.0);
    // the suffix converts the count to SI, per the ADR's worked example
    approx("mag2jy(20) Jy", "mag2jy(20) Jy", 3631.0 * 1e-8 * 1e-26);
}

// ===== Ephemeris accessors (ADR-0037) =====
//
// Body numbers: Mercury 1 through Neptune 8, Pluto 9, Sun 10, Moon 11.
// Positions are geocentric unless an observer is given. Bounds are wide
// enough for independent truth to fit comfortably; the ephemeris crate's
// own CI pins arcsecond accuracy against JPL Horizons.

fn float_at(text: &str) -> f64 {
    match epher_core::evaluate(text) {
        Ok(epher_core::Value::Float(x)) => x,
        other => panic!("{text} produced {other:?}"),
    }
}

#[test]
fn sun_places_match_the_equinoxes_and_solstices() {
    // apparent solar declination at the March equinox instant
    // (2000-03-20 07:35 UTC) is zero within a tenth of a degree
    let dec = float_at("decl(10, jd(2000, 3, 20, 7.583))");
    assert!(dec.abs() < 0.2, "solar dec at equinox = {dec}");
    // at the June solstice it is the obliquity
    let dec = float_at("decl(10, jd(2000, 6, 21, 1.8))");
    assert!((dec - 23.44).abs() < 0.1, "solar dec at solstice = {dec}");
    // right ascension stays a finite 0..360 across the year
    let ra = float_at("ra(10, jd(2000, 9, 22, 17.5))");
    assert!((0.0..360.0).contains(&ra), "ra = {ra}");
}

#[test]
fn distances_land_where_the_almanacs_say() {
    // Earth at perihelion (2000-01-03) was 0.9833 AU
    let d = float_at("dist(10, jd(2000, 1, 1))");
    assert!((0.9825..0.9842).contains(&d), "sun distance = {d} AU");
    // the Moon rides 0.0024..0.00275 AU out
    let d = float_at("dist(11, jd(2000, 1, 1))");
    assert!((0.00238..0.00275).contains(&d), "moon distance = {d} AU");
    // Pluto (the facade's own JPL elements) was about 33.3 AU in
    // mid-2020, still climbing from its 1989 perihelion
    let d = float_at("dist(9, jd(2020, 6, 1))");
    assert!((33.0..33.7).contains(&d), "pluto distance = {d} AU");
}

#[test]
fn moon_illumination_hits_zero_at_a_known_new_moon() {
    // new moon: 2000-01-06 18:14 UTC
    let illum = float_at("illum(11, jd(2000, 1, 6, 18.23))");
    assert!(illum < 0.02, "illum at new moon = {illum}");
    let illum = float_at("illum(11, jd(2000, 1, 21, 4.0))");
    assert!(illum > 0.9, "illum near full moon (2000-01-21) = {illum}");
}

#[test]
fn observer_accessors_see_the_noon_sun() {
    // the Sun stands about 66.5 deg high, near due south, at the
    // equator's local noon on New Year's Day 2000
    let alt = float_at("alt(10, jd(2000, 1, 1, 12), 0, 0)");
    assert!((65.0..68.0).contains(&alt), "solar altitude = {alt}");
    let az = float_at("az(10, jd(2000, 1, 1, 12), 0, 0)");
    assert!((170.0..190.0).contains(&az), "solar azimuth = {az}");
    // Pluto's topocentric altitude is a finite angle too
    let alt = float_at("alt(9, jd(2020, 6, 1, 0), 0, 0)");
    assert!((-90.0..=90.0).contains(&alt), "pluto altitude = {alt}");
}

#[test]
fn rise_set_and_transit_land_on_a_greenwich_day() {
    // Greenwich, 2000-03-20: sunrise 06:11, transit about 12:09,
    // sunset 18:14 UT. Fractions of the local mean-solar day.
    let day = float_at("jd(2000, 3, 20)");
    let rise = float_at("rise(10, jd(2000, 3, 20), 51.5, 0)") - day;
    assert!((0.20..0.32).contains(&rise), "sunrise fraction = {rise}");
    let transit = float_at("transit(10, jd(2000, 3, 20), 51.5, 0)") - day;
    assert!(
        (0.47..0.56).contains(&transit),
        "transit fraction = {transit}"
    );
    let set = float_at("set(10, jd(2000, 3, 20), 51.5, 0)") - day;
    assert!((0.72..0.82).contains(&set), "sunset fraction = {set}");
    // a body that never rises at a latitude is a domain error, not
    // NaN (the polar-night Sun, 78 north on the December solstice)
    assert!(epher_core::evaluate("rise(10, jd(2000, 12, 21), 78, 0)").is_err());
}

#[test]
fn brightness_and_size_accessors() {
    // Venus in January 2020 shone near -3.9
    let m = float_at("mag(2, jd(2020, 1, 1))");
    assert!((-4.9..-3.5).contains(&m), "venus mag = {m}");
    // Saturn near 0.5 around its 2020 opposition (rings included);
    // body 6 under the ADR numbering
    let m = float_at("mag(6, jd(2020, 7, 20))");
    assert!((-1.5..1.5).contains(&m), "saturn mag = {m}");
    // the Moon (Meeus's ch. 48 formula in the facade) is very bright
    let m = float_at("mag(11, jd(2000, 1, 21))");
    assert!((-13.0..-9.0).contains(&m), "moon mag = {m}");
    // angular diameters of the great lights, in degrees
    let d = float_at("diam(10, jd(2000, 1, 1))");
    assert!((0.5237..0.5424).contains(&d), "sun diameter = {d}");
    let d = float_at("diam(11, jd(2000, 1, 1))");
    assert!((0.49..0.57).contains(&d), "moon diameter = {d}");
    // phase geometry stays in its quadrants
    let phase = float_at("phase(4, jd(2020, 10, 6))");
    assert!(
        (0.0..90.0).contains(&phase),
        "mars phase at opposition = {phase}"
    );
    let illum = float_at("illum(4, jd(2020, 10, 6))");
    assert!(
        (0.7..1.01).contains(&illum),
        "mars illum at opposition = {illum}"
    );
}

#[test]
fn body_numbers_are_validated() {
    // Earth is the observer: it has no geocentric place to report
    assert!(epher_core::evaluate("ra(3, jd(2020, 1, 1))").is_err());
    assert!(epher_core::evaluate("dist(0, jd(2020, 1, 1))").is_err());
    assert!(epher_core::evaluate("dist(12, jd(2020, 1, 1))").is_err());
    // horizontal accessors need an observer
    assert!(epher_core::evaluate("alt(4, jd(2020, 1, 1))").is_err());
    assert!(epher_core::evaluate("rise(4, jd(2020, 1, 1), 51.5)").is_err());
}

#[test]
fn the_four_season_entries_land_on_their_instants() {
    // NASA/GSFC almanac values for the year 2000, UTC
    approx("march equinox 2000", "march_equinox(2000)", 2451623.816);
    approx("june solstice 2000", "june_solstice(2000)", 2451716.575);
    approx(
        "september equinox 2000",
        "september_equinox(2000)",
        2451810.228,
    );
    approx(
        "december solstice 2000",
        "december_solstice(2000)",
        2451900.068,
    );
}

// ===== review fixes: honest edges (ADR-0037) =====

#[test]
fn sun_apparent_magnitude_follows_the_inverse_square() {
    // at r about 1.0167 AU (2020-07) the Sun is about -26.70; the
    // published value at 1 AU is -26.74
    let m = float_at("mag(10, jd(2020, 7, 1))");
    assert!((-26.9..-26.4).contains(&m), "sun mag = {m}");
}

#[test]
fn sexagesimal_rounding_carries_all_the_way() {
    // 359.9999 deg is a hair under 24 h; the carry must wrap to 0h
    match epher_core::evaluate("deg2hms(359.9999)") {
        Ok(epher_core::Value::Str(s)) => assert_eq!(s, "0h 0m 0s", "{s}"),
        other => panic!("deg2hms produced {other:?}"),
    }
    // 59.99999 deg carries into a whole degree: 59d 59m 59.99964s
    // rounds the seconds up and the carry climbs to the degrees
    match epher_core::evaluate("deg2dms(59.99999)") {
        Ok(epher_core::Value::Str(s)) => assert_eq!(s, "60\u{b0} 0' 0\"", "{s}"),
        other => panic!("deg2dms produced {other:?}"),
    }
}

#[test]
fn calendar_validation_knows_month_lengths() {
    assert!(
        epher_core::evaluate("jd(2020, 2, 29)").is_ok(),
        "2020 is a leap year"
    );
    assert!(
        epher_core::evaluate("jd(2023, 2, 29)").is_err(),
        "2023 is not"
    );
    assert!(epher_core::evaluate("jd(2023, 2, 30)").is_err());
    assert!(
        epher_core::evaluate("jd(2000, 4, 31)").is_err(),
        "April has 30"
    );
    assert!(
        epher_core::evaluate("jd(2000, 2, 29)").is_ok(),
        "divisible by 400"
    );
}

// ===== Pluto positions/events and Sun phase/illum (ADR-0037 amendment);
// ===== integer powers in the exact layers (ADR-0005 amendment) =====

#[test]
fn pluto_positions_match_horizons() {
    // JPL Horizons (DE441), apparent geocentric place, 2020-07-01 00:00 UT:
    // RA 19h 44m 40s = 296.16642 deg, dec -22.25269 deg. Pluto rides the
    // facade's Keplerian elements (documented arcminute grade), so the
    // tolerance is 2 arcmin; the measured residuals are ~6x tighter.
    let ra = float_at("ra(9, jd(2020, 7, 1))");
    assert!((ra - 296.16642).abs() < 0.0333, "pluto ra = {ra}");
    let dec = float_at("decl(9, jd(2020, 7, 1))");
    assert!((dec + 22.25269).abs() < 0.0333, "pluto dec = {dec}");
}

#[test]
fn pluto_events_mirror_the_snapshot_convention() {
    let day = float_at("jd(2020, 7, 1)");
    // Pluto (dec -22.3) at the equator transits about 66 minutes after
    // local midnight (its RA is 19h45m, the Sun's is 6h40m)
    let transit = float_at("transit(9, jd(2020, 7, 1), 0, 0)") - day;
    assert!(
        (0.03..0.06).contains(&transit),
        "pluto transit fraction = {transit}"
    );
    // meridian passage is due south at the equator
    let az = float_at("az(9, transit(9, jd(2020, 7, 1), 0, 0), 0, 0)");
    assert!((179.5..=180.5).contains(&az), "pluto transit az = {az}");
    // Pluto is already up at that day's start, so set comes before the
    // next rise; both stay inside the local day window
    let set = float_at("set(9, jd(2020, 7, 1), 0, 0)") - day;
    assert!((0.0..1.0).contains(&set), "pluto set fraction = {set}");
    let rise = float_at("rise(9, jd(2020, 7, 1), 0, 0)") - day;
    assert!((0.0..1.0).contains(&rise), "pluto rise fraction = {rise}");
    // circumpolar south of -68: a domain error, never a NaN
    assert!(epher_core::evaluate("rise(9, jd(2020, 7, 1), -80, 0)").is_err());
}

#[test]
fn sun_phase_and_illum_are_their_definitions() {
    // the Sun's phase angle as seen from Earth is zero by definition
    // (Horizons reports phi 0.0000 and an illuminated fraction of 100%)
    let phase = float_at("phase(10, jd(2020, 7, 1))");
    assert_eq!(phase, 0.0);
    let illum = float_at("illum(10, jd(2020, 7, 1))");
    assert_eq!(illum, 1.0);
    // and the two satisfy the usual identity
    let id = float_at(
        "illum(10, jd(2024, 3, 15)) - (1 + cos(phase(10, jd(2024, 3, 15)) * pi / 180)) / 2",
    );
    assert!(id.abs() < 1e-12, "sun illum/phase identity = {id}");
}

#[test]
fn exact_layers_raise_to_integer_powers() {
    // big: exact, arbitrarily large
    assert_eq!(
        eval_str("big(2) ^ 100").to_string(),
        "1267650600228229401496703205376"
    );
    // a power of ten keeps a negative scale, which BigDecimal displays
    // in scientific notation - the value is exact either way
    assert_eq!(
        eval_str("big(10) ^ 40"),
        Value::Big("1e+40".parse().unwrap())
    );
    // negative integer exponents give exact reciprocals, normalized
    assert_eq!(eval_str("big(2) ^ -10").to_string(), "0.0009765625");
    // rationals: exact in both directions
    assert_eq!(eval_str("frac(2, 3) ^ 2").to_string(), "4/9");
    assert_eq!(eval_str("frac(2, 3) ^ -2").to_string(), "9/4");
    // decimals: exact
    assert_eq!(eval_str("dec(3) ^ 3").to_string(), "27");
    assert_eq!(eval_str("dec(2) ^ 10").to_string(), "1024");
    // fractional exponents refuse rather than guess (ADR-0005: the layer
    // keeps its exactness; work in floats for fractional powers)
    assert!(epher_core::evaluate("big(2) ^ 0.5").is_err());
    assert!(epher_core::evaluate("dec(2) ^ 0.5").is_err());
    assert!(epher_core::evaluate("frac(2, 3) ^ frac(1, 2)").is_err());
    // float pow of a negative base with a fractional exponent points at
    // root() instead of returning a bare NaN
    let err = epher_core::evaluate("(-8) ^ (1 / 3)")
        .unwrap_err()
        .to_string();
    assert!(err.contains("root"), "{err}");
    // ordinary float powers are untouched
    let x = float_at("2 ^ 0.5");
    assert!((x - std::f64::consts::SQRT_2).abs() < 1e-15);
}

#[test]
fn redeclaring_a_constant_with_the_same_value_is_a_noop() {
    // examples with `const` lines get pasted and re-pasted (ADR-0012
    // amendment): the same definition twice succeeds as a no-op
    assert_eq!(
        run_script("const a = 1\nconst a = 1\na + 1")
            .unwrap()
            .to_string(),
        "2"
    );
    // a changed value keeps the documented error
    let err = run_script("const b = 1\nconst b = 2")
        .unwrap_err()
        .to_string();
    assert!(err.contains("already defined"), "{err}");
    // a constant still never takes a variable's name
    let err = run_script("c = 1\nconst c = 2").unwrap_err().to_string();
    assert!(err.contains("already a variable"), "{err}");
}

#[test]
fn php_style_comments_are_ignored() {
    // ADR-0040: the language gains PHP-style comments — `//` and `#`
    // run to the end of the line, `/* ... */` may span lines and sit
    // inline between tokens.

    // line comments: both spellings, trailing or standalone
    assert_eq!(eval_str("2 + 2 // four").to_string(), "4");
    assert_eq!(eval_str("2 + 2 # four").to_string(), "4");
    // a comment line before the expression: the statement-separator view
    // (a leading newline was always a script-level thing, not an
    // expression-level one)
    assert_eq!(
        run_script("// just a note\n2 + 2").unwrap().to_string(),
        "4"
    );
    assert_eq!(run_script("# just a note\n2 + 2").unwrap().to_string(), "4");
    // a comment on its own line is a silent no-op: the script has no
    // statements, so a session submit prints nothing at all
    assert!(parse_script("// only").unwrap().is_empty());
    let mut session = Session::new();
    assert_eq!(session.submit("// only"), "");

    // block comments: inline between tokens, across lines, and alone
    assert_eq!(eval_str("2 /* twice */ * 2").to_string(), "4");
    assert_eq!(
        eval_str("1 + /* a note\nthat spans lines */ 2").to_string(),
        "3"
    );
    assert_eq!(eval_str("/* only */ 5").to_string(), "5");
    // a block comment's newlines never become statement separators
    assert_eq!(eval_str("1 /* \n */ + /* \n */ 2").to_string(), "3");

    // comments between script statements: the separators still count
    assert_eq!(
        run_script("a = 1 // first\nb = 2 # second\n/* third */\na + b")
            .unwrap()
            .to_string(),
        "3"
    );

    // a slash that is not a comment opener divides as always
    assert_eq!(eval_str("8 / 2 / 2").to_string(), "2");

    // an unterminated block comment is a parse error, like PHP's
    let err = eval_str_checked("2 /* never closed")
        .unwrap_err()
        .to_string();
    assert!(err.contains("unterminated block comment"), "{err}");
}

// ===== percent suffix (ADR-0042): a transparent /100, never add-on =====

#[test]
fn percent_is_a_div100_suffix() {
    assert_eq!(eval_str("5%").to_string(), "0.05");
    assert_eq!(eval_str("50%").to_string(), "0.5");
    assert_eq!(eval_str("10%%").to_string(), "0.001");
    assert_eq!(eval_str("-5%").to_string(), "-0.05");
    // binds tightest, like factorial
    assert!((eval_f64("10% ^ 2") - 0.01).abs() < 1e-15);
}

#[test]
fn percent_composes_with_arithmetic_without_addon_magic() {
    // the deliberate non-Casio reading: % never looks at the surrounding
    // operator, so 200 + 10% is 200 + 0.1, and "increase 200 by 10%" is
    // spelled 200 * (1 + 10%)
    assert_eq!(eval_str("200 + 10%").to_string(), "200.1");
    assert!((eval_f64("200 * (1 + 10%)") - 220.0).abs() < 1e-9);
    assert_eq!(eval_str("50 * 10%").to_string(), "5");
    assert!((eval_f64("(1 + 10%) ^ 2") - 1.21).abs() < 1e-12);
}

#[test]
fn percent_works_in_functions_and_graphs_parse() {
    assert_eq!(
        run_script("def half(x) = x / 2\nhalf(200%)")
            .unwrap()
            .to_string(),
        "1"
    );
    // a graph expression with percent parses (sampling is eval's job)
    assert!(parse("200 + 10% * x").is_ok());
}

// ===== number theory (ADR-0042) =====

#[test]
fn primality_is_exact() {
    assert_eq!(eval_str("isprime(97)"), Value::Bool(true));
    assert_eq!(eval_str("isprime(2)"), Value::Bool(true));
    assert_eq!(eval_str("isprime(1)"), Value::Bool(false));
    assert_eq!(eval_str("isprime(0)"), Value::Bool(false));
    assert_eq!(eval_str("isprime(-7)"), Value::Bool(false));
    // the largest prime below 2^53: exact as an f64 literal, and the
    // Miller-Rabin must not flinch
    assert_eq!(eval_str("isprime(9007199254740881)"), Value::Bool(true));
    assert_eq!(eval_str("isprime(9007199254740882)"), Value::Bool(false));
    assert_eq!(eval_str("isprime(1000000007)"), Value::Bool(true));
    assert_eq!(
        eval_str("isprime(1000000007 * 998244353)"),
        Value::Bool(false)
    );
}

#[test]
fn prime_neighbours() {
    assert_eq!(eval_str("nextprime(1)").to_string(), "2");
    assert_eq!(eval_str("nextprime(10)").to_string(), "11");
    assert_eq!(
        eval_str("nextprime(1000000000000)").to_string(),
        "1000000000039"
    );
    assert_eq!(eval_str("prevprime(10)").to_string(), "7");
    assert_eq!(eval_str("prevprime(3)").to_string(), "2");
    let err = eval_str_checked("prevprime(2)").unwrap_err().to_string();
    assert!(err.contains("no prime below"), "{err}");
}

#[test]
fn modular_power_is_exact() {
    assert_eq!(eval_str("modpow(2, 10, 1000)").to_string(), "24");
    assert_eq!(
        eval_str("modpow(2, 100, 1000000007)").to_string(),
        "976371285"
    );
    let err = eval_str_checked("modpow(2, -1, 7)")
        .unwrap_err()
        .to_string();
    assert!(err.contains("must be >= 0"), "{err}");
    let err = eval_str_checked("modpow(2, 3, 0)").unwrap_err().to_string();
    assert!(err.contains("division by zero"), "{err}");
}

#[test]
fn factorization_tools() {
    assert_eq!(eval_str("factors(360)").to_string(), "2^3 * 3^2 * 5");
    assert_eq!(eval_str("factors(97)").to_string(), "97");
    assert_eq!(eval_str("factors(1)").to_string(), "1");
    // a semiprime within f64's exact range forces the rho splitter, not
    // just trial division
    assert_eq!(
        eval_str("factors(1000003 * 1000033)").to_string(),
        "1000003 * 1000033"
    );
    assert_eq!(eval_str("totient(1)").to_string(), "1");
    assert_eq!(eval_str("totient(12)").to_string(), "4");
    assert_eq!(eval_str("ndivisors(360)").to_string(), "24");
    let err = eval_str_checked("factors(0)").unwrap_err().to_string();
    assert!(err.contains("positive integer"), "{err}");
}

// ===== physical constants (ADR-0042) =====

#[test]
fn physical_constants_are_available_and_shadowable() {
    assert_eq!(eval_str("n_a").to_string(), "602214076000000000000000");
    assert_eq!(eval_f64("q_e"), 1.602_176_634e-19);
    assert_eq!(eval_str("atm").to_string(), "101325");
    assert_eq!(eval_f64("G"), 6.67430e-11);
    assert_eq!(eval_f64("gamma"), 0.577_215_664_901_532_9);
    assert_eq!(eval_f64("ev"), 1.602_176_634e-19);
    // the exact ones compose: an electronvolt of Na particles is a faraday
    assert!((eval_f64("ev * n_a") - eval_f64("faraday")).abs() < 1e-2);
}

// ===== builtin catalog (ADR-0042) =====

#[test]
fn catalog_is_sorted_and_unique() {
    let cat = epher_core::catalog();
    assert!(cat.len() > 100, "catalog looks too short: {}", cat.len());
    for pair in cat.windows(2) {
        assert!(
            pair[0].name < pair[1].name,
            "catalog not sorted at {} < {}",
            pair[0].name,
            pair[1].name
        );
    }
}

#[test]
fn every_catalog_name_is_live() {
    use epher_core::{eval, parse, CatalogKind, Env};
    for entry in epher_core::catalog() {
        let source = match entry.kind {
            CatalogKind::Constant => entry.name.to_string(),
            CatalogKind::Function => format!("{}(1)", entry.name),
        };
        let outcome = eval(
            &parse(&source).expect("catalog name parses"),
            &Env::default(),
        );
        match outcome {
            Ok(_) => {}
            // argument-shaped errors are fine — only "unknown" means the
            // catalog drifted from the real builtins
            Err(e) => assert!(
                !matches!(e, epher_core::EpherError::UnknownName(_)),
                "catalog entry {} does not evaluate: {e}",
                entry.name
            ),
        }
    }
}

// --- Round 2: complex numbers, solve, calculus, exact, formats (ADR-0043)

fn run_script_text(src: &str) -> Vec<Value> {
    let mut env = Env::default();
    let script = parse_script(src).expect("parse");
    run_all(&script, &mut env).expect("run")
}

#[test]
fn imaginary_literals_and_the_i_constant() {
    assert_eq!(eval_display("i"), "i");
    assert_eq!(eval_display("3 + 4i"), "3+4i");
    assert_eq!(eval_display("2.5i"), "2.5i");
    assert_eq!(eval_display("0xFFi"), "255i");
    assert_eq!(eval_display("i^2"), "-1");
    assert_eq!(eval_display("i^3"), "-i");
    assert_eq!(eval_display("(1 + i)^2"), "2i");
    assert_eq!(eval_display("(2i)^2"), "-4");
    assert_eq!(eval_display("i^(-1)"), "-i");
    assert_eq!(eval_display("-4i"), "-4i");
    assert_eq!(eval_display("2e3i"), "2000i");
    // an i glued to a longer name is not an imaginary suffix: `4it` is
    // a number followed by a name, which the grammar rejects
    assert!(eval_err("4it").contains("trailing"));
    // i is shadowable like pi
    assert_eq!(
        run_script_text("i = 5\ni + 1").last().unwrap().to_string(),
        "6"
    );
}

#[test]
fn complex_arithmetic_and_parts() {
    assert_eq!(eval_display("(3 + 4i) * (1 - i)"), "7+i");
    assert_eq!(eval_display("(3 + 4i) / i"), "4-3i");
    assert_eq!(eval_display("re(3 + 4i)"), "3");
    assert_eq!(eval_display("im(3 + 4i)"), "4");
    assert_eq!(eval_display("arg(-1)"), "3.141592653589793");
    assert_eq!(eval_display("conj(3 - 4i)"), "3+4i");
    assert_eq!(eval_display("abs(3 + 4i)"), "5");
    assert_eq!(eval_display("re(5)"), "5");
    assert_eq!(eval_display("im(5)"), "0");
    assert_eq!(eval_display("conj(5)"), "5");
    assert_eq!(eval_display("1 / i"), "-i");
    // complex / 0 errors like real / 0
    assert!(eval_err("(1 + i) / 0").contains("division by zero"));
}

#[test]
fn real_domain_errors_now_fall_back_to_complex() {
    assert_eq!(eval_display("sqrt(-1)"), "i");
    assert_eq!(eval_display("sqrt(-4)"), "2i");
    assert_eq!(eval_display("ln(-1)"), "3.14159265359i");
    // exp(i*pi) is -1 up to sin(pi)'s f64 noise
    assert_close(eval_number("re(exp(i * pi))"), -1.0);
    assert_close(eval_number("im(exp(i * pi))"), 0.0);
    assert!(eval_display("sin(2 + i)").contains('i'));
    assert!(eval_display("sqrt(2i)").contains('i'));
    // cbrt keeps the real branch for real inputs
    assert_eq!(eval_display("cbrt(-8)"), "-2");
}

#[test]
fn integer_functions_reject_complex() {
    assert!(eval_err("fact(2i)").contains("expects"));
    assert!(eval_err("gcd(2i, 4)").contains("expects"));
    assert!(eval_err("floor(1 + i)").contains("expects"));
    assert!(eval_err("isprime(3 + 0i)").contains("expects"));
    assert!(eval_err("2i < 3").contains("compare"));
    assert!(eval_err("min(1, 2i)").contains("expects"));
}

#[test]
fn solve_polynomials_get_every_root() {
    assert_eq!(eval_display_script("solve x^2 == 5*x + 6"), "x = -1, x = 6");
    assert_eq!(eval_display_script("solve x^2 == -1"), "x = -i, x = i");
    assert_eq!(
        eval_display_script("solve x^2 + 2*x + 5 == 0"),
        "x = -1-2i, x = -1+2i"
    );
    assert_eq!(eval_display_script("solve x^2 - 2*x + 1 == 0"), "x = 1");
    assert_eq!(eval_display_script("solve 2*x + 3 == 7"), "x = 2");
    assert_eq!(
        eval_display_script("solve x^4 - 1 == 0"),
        "x = -1, x = -i, x = i, x = 1"
    );
    assert_eq!(
        eval_display_script("solve (x - 1)^2 * (x + 2) == 0"),
        "x = -2, x = 1"
    );
    assert_eq!(eval_display_script("solve x == x"), "x is any number");
    assert_eq!(eval_display_script("solve x == x + 1"), "no solution");
}

#[test]
fn solve_uses_constants_and_bound_parameters() {
    // a user constant resolves as a parameter
    assert_eq!(
        run_script_text("const k = 3\nsolve k*x == 12")
            .last()
            .unwrap()
            .to_string(),
        "x = 4"
    );
    // a bound variable resolves as a parameter; x stays symbolic even
    // when the session holds a value for it
    assert_eq!(
        run_script_text("x = 10\nsolve sin(x) == 0\nx")
            .last()
            .unwrap()
            .to_string(),
        "10"
    );
}

#[test]
fn solve_errors_are_instructive() {
    // no equation
    assert!(script_err("solve x + 1").contains("needs an equation"));
    // no variable
    assert!(script_err("solve 5 == 5").contains("no variable"));
    // an unbound second name
    assert!(script_err("solve x^2 + y^2 == 1").contains("not the only unknown"));
}

#[test]
fn solve_transcendental_numeric_scan() {
    // roots of sin(x) == 0 include x = 0 (a sample point) and pi
    let out = eval_display_script("solve sin(x) == 0");
    assert!(out.contains("x = 0,"));
    assert!(out.contains("x = 3.141592653589793,"));
    // tan has no roots at its poles; the residual check keeps them out
    let out = eval_display_script("solve tan(x) == 0");
    assert!(!out.contains("x = 1.5707963267948966"));
    // no real roots
    assert_eq!(
        eval_display_script("solve sin(x) == 2"),
        "no real roots found in -100..100"
    );
}

#[test]
fn derivative_is_numeric_and_graphable() {
    assert_close(eval_number("derivative(x^2, 3)"), 6.0);
    assert_close(eval_number("derivative(x^3 - x, 2)"), 11.0);
    assert_close(eval_number("derivative(sin(t), 0)"), 1.0);
    assert_eq!(eval_display("derivative(5, 2)"), "0");
    // x stays symbolic even when the session holds a value for it
    // (the same rule as solve): a stored x must not zero the derivative
    assert_close(
        run_script_text("x = 10\nderivative(x^2, 3)")
            .last()
            .map(|v| match v {
                Value::Float(f) => *f,
                _ => 0.0,
            })
            .unwrap(),
        6.0,
    );
    // bound parameters stay parameters
    assert_close(
        run_script_text("a = 2\nderivative(a * x^2, 3)")
            .last()
            .map(|v| match v {
                Value::Float(f) => *f,
                _ => 0.0,
            })
            .unwrap(),
        12.0,
    );
    assert!(eval_err("derivative(x*y, 1)")
        .to_string()
        .contains("variables"));
    assert!(eval_err("derivative(x^2)").contains("expects 2"));
}

#[test]
fn integral_is_adaptive_and_signed() {
    assert_close(eval_number("integral(x^2, 0, 3)"), 9.0);
    assert_close(eval_number("integral(sin(x), 0, pi)"), 2.0);
    assert_close(eval_number("integral(x^2, 3, 0)"), -9.0);
    assert_eq!(eval_display("integral(x^2, 5, 5)"), "0");
    // a constant integrand integrates to (b - a) * c
    assert_close(eval_number("integral(4, 0, 2)"), 8.0);
    // a parameter inside the integrand
    assert_close(
        run_script_text("b = 3\nintegral(b * x, 0, 1)")
            .last()
            .map(|v| match v {
                Value::Float(f) => *f,
                _ => 0.0,
            })
            .unwrap(),
        1.5,
    );
    assert!(eval_err("integral(x*y, 0, 1)")
        .to_string()
        .contains("variables"));
    assert!(eval_err("integral(x^2, 0)").contains("expects 3"));
}

#[test]
fn exact_reconstructs_small_fractions() {
    assert_eq!(eval_display("exact(0.3333333333333333)"), "1/3");
    assert_eq!(eval_display("exact(0.30000000000000004)"), "3/10");
    assert_eq!(eval_display("exact(0.5)"), "1/2");
    assert_eq!(eval_display("exact(2.5)"), "5/2");
    // irrationals pass through
    assert_eq!(eval_display("exact(pi)"), "3.141592653589793");
    assert_eq!(eval_display("exact(sqrt(2))"), "1.4142135623730951");
    // negatives reconstruct with their sign
    assert_eq!(eval_display("exact(-0.25)"), "-1/4");
    // non-floats pass through
    assert_eq!(eval_display("exact(frac(1, 7))"), "1/7");
}

#[test]
fn format_verbs_spell_numbers() {
    assert_eq!(eval_display("scientific(12345)"), "1.2345e4");
    assert_eq!(eval_display("engineering(12345)"), "12.345e3");
    assert_eq!(eval_display("engineering(0.5)"), "500e-3");
    assert_eq!(eval_display("engineering(999)"), "999");
    assert_eq!(
        eval_display("grouped(1234567.89)"),
        "1\u{2009}234\u{2009}567.89"
    );
    assert_eq!(
        eval_display("grouped(-987654321)"),
        "-987\u{2009}654\u{2009}321"
    );
    assert_eq!(eval_display("scientific(0)"), "0e0");
}

#[test]
fn display_prefs_shape_the_result_line() {
    use epher_core::{format_value, DisplayPrefs, Notation};
    let v = |s: &str| evaluate(s).expect("eval");
    let auto = DisplayPrefs::default();
    assert_eq!(format_value(&v("1/3"), &auto), "1/3");
    assert_eq!(format_value(&v("0.1 + 0.2"), &auto), "0.3");
    assert_eq!(format_value(&v("pi"), &auto), "3.14159265359");
    let plain = DisplayPrefs {
        exact_fractions: false,
        ..DisplayPrefs::default()
    };
    assert_eq!(format_value(&v("1/3"), &plain), "0.333333333333");
    let sci = DisplayPrefs {
        exact_fractions: false,
        notation: Notation::Scientific,
        separators: false,
    };
    assert_eq!(format_value(&v("12345"), &sci), "1.2345e4");
    let eng = DisplayPrefs {
        exact_fractions: false,
        notation: Notation::Engineering,
        separators: false,
    };
    assert_eq!(format_value(&v("12345"), &eng), "12.345e3");
    let sep = DisplayPrefs {
        exact_fractions: false,
        notation: Notation::Auto,
        separators: true,
    };
    assert_eq!(
        format_value(&v("1234567.89"), &sep),
        "1\u{2009}234\u{2009}567.89"
    );
}

fn script_err(src: &str) -> String {
    let mut env = Env::default();
    match run_all(&parse_script(src).expect("parse"), &mut env) {
        Err(e) => e.to_string(),
        Ok(_) => panic!("expected an error from {src:?}"),
    }
}

fn eval_display_script(src: &str) -> String {
    run_script_text(src)
        .last()
        .map(|v| v.to_string())
        .unwrap_or_else(|| panic!("no value from {src:?}"))
}

// ===== data platform (ADR-0044) =====

#[test]
fn list_literals_evaluate_to_list_values() {
    match eval_str("{1, 2, 3}") {
        Value::List(items) => {
            assert_eq!(
                items,
                vec![Value::float(1.0), Value::float(2.0), Value::float(3.0)]
            )
        }
        other => panic!("expected a list, got {other:?}"),
    }
    assert_eq!(eval_str("{}"), Value::List(vec![]));
    // elements are expressions
    assert_eq!(eval_str("{1, 2 + 3, pi}"), eval_str("{1, 5, pi}"));
    // trailing comma is allowed
    assert_eq!(eval_str("{1, 2,}"), eval_str("{1, 2}"));
}

#[test]
fn list_indexing_is_one_based_and_binds_tight() {
    assert_eq!(eval_str("{10, 20, 30}[2]"), Value::float(20.0));
    assert_eq!(eval_str("{10, 20, 30}[1 + 1]"), Value::float(20.0));
    let mut env = Env::default();
    env.set("d", eval_str("{5, 6}"));
    assert_eq!(
        eval(&parse("d[2]").unwrap(), &env).unwrap(),
        Value::float(6.0)
    );
    // index binds tighter than ^
    assert_eq!(eval_str("{2, 3}[1]^2"), Value::float(4.0));
    // out of range and non-list index are errors
    let err = eval(&parse("{1}[5]").unwrap(), &Env::default()).unwrap_err();
    assert!(err.to_string().contains("out of range"), "{err}");
    let err = eval(&parse("2[1]").unwrap(), &Env::default()).unwrap_err();
    assert!(err.to_string().contains("needs a list"), "{err}");
    let err = eval(&parse("{1}[1.5]").unwrap(), &Env::default()).unwrap_err();
    assert!(err.to_string().contains("whole number"), "{err}");
}

#[test]
fn list_arithmetic_is_elementwise_with_scalar_broadcast() {
    assert_eq!(eval_str("{1, 2, 3} * 2"), eval_str("{2, 4, 6}"));
    assert_eq!(eval_str("2 / {1, 2, 4}"), eval_str("{2, 1, 0.5}"));
    assert_eq!(eval_str("{1, 2} + {3, 4}"), eval_str("{4, 6}"));
    assert_eq!(eval_str("-{1, 2}"), eval_str("{-1, -2}"));
    assert_eq!(eval_str("{2, 3} ^ 2"), eval_str("{4, 9}"));
    let err = eval(&parse("{1, 2} + {3}").unwrap(), &Env::default()).unwrap_err();
    assert!(err.to_string().contains("different lengths"), "{err}");
    let err = eval(&parse("{1, 2} + 1i").unwrap(), &Env::default()).unwrap_err();
    assert!(err.to_string().contains("cannot combine"), "{err}");
}

#[test]
fn list_equality_compares_whole_lists() {
    assert_eq!(eval_str("{1, 2} == {1, 2}"), Value::Bool(true));
    assert_eq!(eval_str("{1, 2} != {1, 3}"), Value::Bool(true));
    let err = eval(&parse("{1, 2} < {3, 4}").unwrap(), &Env::default()).unwrap_err();
    assert!(err.to_string().contains("cannot compare"), "{err}");
}

#[test]
fn stats_accept_a_list_or_variadic_numbers() {
    assert_eq!(eval_str("sum({1, 2, 3})"), eval_str("6"));
    assert_eq!(eval_str("sum(1, 2, 3)"), eval_str("6"));
    assert_eq!(eval_str("mean({1, 2, 3})"), eval_str("2"));
    assert_eq!(eval_str("median({3, 1, 2})"), eval_str("2"));
    assert_eq!(
        eval_str("stdev({1, 2, 3, 4})"),
        eval_str("stdev(1, 2, 3, 4)")
    );
    assert_eq!(eval_str("min({3, 1, 2})"), eval_str("1"));
    assert_eq!(eval_str("max({3, 1, 2})"), eval_str("3"));
}

#[test]
fn list_shape_builtins() {
    assert_eq!(eval_str("len({1, 2, 3})"), Value::float(3.0));
    assert_eq!(eval_str("sort({3, 1, 2})"), eval_str("{1, 2, 3}"));
    assert_eq!(eval_str("mode({1, 2, 2, 3})"), Value::float(2.0));
    assert_eq!(eval_str("mode({2, 1, 2, 1})"), Value::float(1.0)); // smallest on ties
    assert_eq!(eval_str("range({3, 1, 7})"), Value::float(6.0));
    assert_eq!(
        eval_str("quartile({1, 2, 3, 4, 5, 6, 7, 8}, 1)"),
        Value::float(2.5)
    );
    assert_eq!(
        eval_str("quartile({1, 2, 3, 4, 5, 6, 7, 8}, 3)"),
        Value::float(6.5)
    );
    assert_eq!(eval_str("quartile({1, 2, 3, 4, 5}, 2)"), Value::float(3.0));
    // empty lists are domain errors for the statistics
    let err = eval(&parse("mean({})").unwrap(), &Env::default()).unwrap_err();
    assert!(err.to_string().contains("mean"), "{err}");
    let err = eval(&parse("len(5)").unwrap(), &Env::default()).unwrap_err();
    assert!(err.to_string().contains("expects a list"), "{err}");
}

#[test]
fn linreg_fits_and_reports_r() {
    assert_eq!(
        eval_str("linreg({1, 2, 3}, {2, 4, 6})"),
        Value::Str("y = 2*x + 0 (r = 1)".into())
    );
    match eval_str("linreg({1, 2, 3, 4}, {2.1, 4.2, 5.8, 8.1})") {
        Value::Str(s) => {
            assert!(s.starts_with("y = 1.96*x + 0.15"), "{s}");
            assert!(s.contains("r = 0.9979"), "{s}");
        }
        other => panic!("expected a display string, got {other:?}"),
    }
    let err = eval(&parse("linreg({1}, {2})").unwrap(), &Env::default()).unwrap_err();
    assert!(err.to_string().contains("at least 2"), "{err}");
    let err = eval(&parse("linreg({1, 2}, {3})").unwrap(), &Env::default()).unwrap_err();
    assert!(err.to_string().contains("same-length"), "{err}");
}

#[test]
fn normal_family_matches_reference_values() {
    assert_close(eval_f64("normcdf(0)"), 0.5);
    assert_close(eval_f64("normcdf(1.96)"), 0.975002104852);
    assert_close(eval_f64("normcdf(2, 2, 0.5)"), 0.5);
    assert_close(eval_f64("invnorm(0.975)"), 1.959963984540);
    assert_close(eval_f64("normpdf(0)"), 0.398942280401);
    assert_close(eval_f64("normpdf(0, 0, 2)"), 0.199471140201);
    // p outside (0, 1) is a domain error
    let err = eval(&parse("invnorm(1.5)").unwrap(), &Env::default()).unwrap_err();
    assert!(err.to_string().contains("probability"), "{err}");
}

#[test]
fn t_family_matches_reference_values() {
    assert_close(eval_f64("tcdf(0, 5)"), 0.5);
    assert_close(eval_f64("tcdf(2, 10)"), 0.963305982615);
    assert_close(eval_f64("tpdf(0, 10)"), 0.389108383966);
    assert_close(eval_f64("invt(0.975, 10)"), 2.228138851986);
    assert_close(eval_f64("invt(0.95, 20)"), 1.724718242921);
    let err = eval(&parse("tcdf(1, -1)").unwrap(), &Env::default()).unwrap_err();
    assert!(err.to_string().contains("df"), "{err}");
}

#[test]
fn chi2_family_matches_reference_values() {
    assert_close(eval_f64("chi2cdf(3.84, 1)"), 0.949956478752);
    assert_close(eval_f64("chi2cdf(5.99, 2)"), 0.949963372913);
    assert_close(eval_f64("invchi2(0.95, 2)"), 5.991464547108);
    assert_close(eval_f64("chi2pdf(1, 2)"), 0.303265329856);
}

#[test]
fn quantile_tails_invert_to_full_precision() {
    // The parity sweep found the old inversions stalling in the tails:
    // a 1e-12 residual tolerance and a fixed [-100, 100] bracket made
    // invt clamp at 100 for p beyond 0.99999 and lose the last digits
    // of ordinary tail quantiles (ADR-0052). The survivor-space
    // inversion converges to the exact quantile of the stored p.
    assert_close(eval_f64("invt(0.995, 3)"), 5.840909309733);
    assert_close(eval_f64("invt(0.99999, 3)"), 47.927728375934);
    assert_close(eval_f64("invt(0.999999, 3)"), 103.299467779429);
    assert_close(eval_f64("invt(0.9999, 1)"), 3183.098757118151);
    assert_close(eval_f64("invt(0.000001, 3)"), -103.299467779429);
    assert_close(eval_f64("invchi2(0.999999, 5)"), 35.888186879610);
    // invnorm polishes in tail space: 1e-15 relative everywhere.
    assert_close(eval_f64("invnorm(0.99999999)"), 5.612001243305);
    assert_close(eval_f64("invnorm(1 - 1e-10)"), 6.361340889697);
    assert_close(eval_f64("invnorm(1e-12)"), -7.034483825301);
    assert_close(eval_f64("invnorm(0.5)"), 0.0);
}

#[test]
fn discrete_families_match_reference_values() {
    assert_close(eval_f64("binompdf(2, 10, 0.5)"), 0.0439453125);
    assert_close(eval_f64("binomcdf(2, 10, 0.5)"), 0.0546875);
    assert_close(eval_f64("binomcdf(5, 10, 0.9)"), 0.0016349374);
    assert_close(eval_f64("poissonpdf(2, 2)"), 0.270670566473);
    assert_close(eval_f64("poissoncdf(3, 2)"), 0.857123460499);
    assert_close(eval_f64("poissoncdf(0, 2)"), 0.135335283237);
}

#[test]
fn tests_and_intervals_report_statistics() {
    let script = "d = {12, 15, 14, 16, 13, 15, 14, 17}";
    assert_eq!(
        eval_display_script(&format!("{script}\nztest(d, 14, 1.5)")),
        "z = 0.9428, p = 0.3458"
    );
    assert_eq!(
        eval_display_script(&format!("{script}\nttest(d, 14)")),
        "t = 0.8819, p = 0.4071"
    );
    assert_eq!(
        eval_display_script(&format!("{script}\nzinterval(d, 1.5, 0.95)")),
        "(13.4606, 15.5394)"
    );
    assert_eq!(
        eval_display_script(&format!("{script}\ntinterval(d, 0.95)")),
        "(13.1594, 15.8406)"
    );
    assert_eq!(
        eval_display_script("chisq_gof({20, 30, 25, 25}, {25, 25, 25, 25})"),
        "chi2 = 2, p = 0.5724"
    );
    let err = script_err("ttest({1}, 0)");
    assert!(err.contains("at least 2"), "{err}");
}

#[test]
fn data_plots_compute_primitives() {
    use epher_core::graph::{sample_data_plot, DataPlotKind};
    let env = Env::default();
    let scatter = sample_data_plot("scatter({1, 2, 3}, {2, 4, 6})", &env).unwrap();
    assert_eq!(scatter.kind, DataPlotKind::Scatter);
    assert_eq!(scatter.points, vec![(1.0, 2.0), (2.0, 4.0), (3.0, 6.0)]);
    let fit = scatter.fit.expect("fit for 3 points");
    assert_eq!(fit.model, epher_core::graph::ScatterFit::Linreg);
    assert!((fit.fit.a - 2.0).abs() < 1e-9);
    assert!((fit.fit.b).abs() < 1e-9);
    assert!((fit.fit.r - 1.0).abs() < 1e-9);

    let hist = sample_data_plot("histogram({1, 2, 2, 3, 3, 3, 4}, 4)", &env).unwrap();
    assert_eq!(hist.kind, DataPlotKind::Histogram);
    assert_eq!(hist.bins.len(), 4);
    let counts: Vec<f64> = hist.bins.iter().map(|(_, _, c)| *c).collect();
    assert_eq!(counts, vec![1.0, 2.0, 3.0, 1.0]);
    let total: f64 = counts.iter().sum();
    assert_eq!(total, 7.0);

    let boxed = sample_data_plot("boxplot({1, 2, 2, 3, 3, 3, 9})", &env).unwrap();
    assert_eq!(boxed.kind, DataPlotKind::BoxPlot);
    let five = boxed.boxplot.expect("five numbers");
    assert_eq!(five, [1.0, 2.0, 3.0, 3.0, 9.0]);

    // a single point has no fit; domain keywords are rejected
    let single = sample_data_plot("scatter({1}, {2})", &env).unwrap();
    assert!(single.fit.is_none());
    let err = sample_data_plot("scatter({1, 2}, {3, 4}) from -5 to 5", &env).unwrap_err();
    assert!(err.to_string().contains("from a to b"), "{err}");
    let err = sample_data_plot("histogram({1, 2, 3}, 4, 5)", &env).unwrap_err();
    assert!(err.to_string().contains("bin count"), "{err}");
}

#[test]
fn table_commands_parse_and_evaluate_with_derivative() {
    use epher_core::graph::{parse_table_source, table_rows};
    let env = Env::default();
    let spec = parse_table_source("x ^ 2 from -1 to 1 points 3 derivative x ^ 2").unwrap();
    assert!(spec.derivative.is_some());
    let rows = table_rows(
        &spec.expr,
        spec.derivative.as_ref(),
        spec.x_min,
        spec.x_max,
        spec.points,
        &env,
    );
    assert_eq!(rows.len(), 3);
    let (x0, y0, d0) = rows[0];
    assert!((x0 - -1.0).abs() < 1e-12);
    assert!((y0.unwrap() - 1.0).abs() < 1e-9);
    assert!(
        (d0.unwrap() - -2.0).abs() < 1e-6,
        "y' at -1 is -2, got {}",
        d0.unwrap()
    );
    let (_, y1, d1) = rows[1];
    assert!((y1.unwrap() - 0.0).abs() < 1e-9);
    assert!((d1.unwrap() - 0.0).abs() < 1e-6);

    // without a derivative clause the third column is absent
    let plain = parse_table_source("x from 0 to 1 points 2").unwrap();
    assert!(plain.derivative.is_none());
}

// ===== seeded random numbers (ADR-0045) =====

/// Run script lines against a fresh Env, returning the last value — the
/// env-persistent counterpart of `eval_str` (the RNG state lives in the
/// Env, ADR-0045).
fn eval_in_env(src: &str, env: &mut Env) -> Result<Value, epher_core::EpherError> {
    run(&parse_script(src).expect("parse"), env)?
        .ok_or(epher_core::EpherError::Parse("no value".into()))
}

#[test]
fn seeded_random_is_reproducible_and_re_seedable() {
    // randseed pins the sequence: same seed, same draws, in any session.
    assert_eq!(eval_str("randseed(7)"), Value::float(7.0));
    let mut env = Env::default();
    let u0 = eval_in_env("random()", &mut env).unwrap();
    let u1 = eval_in_env("random()", &mut env).unwrap();
    let mut twin = Env::default();
    let v0 = eval_in_env("random()", &mut twin).unwrap();
    let v1 = eval_in_env("random()", &mut twin).unwrap();
    assert_eq!(u0, v0, "first draw reproducible across sessions");
    assert_eq!(u1, v1, "second draw reproducible across sessions");
    assert_ne!(u0, u1, "draws advance");
    // a different seed diverges (astronomically unlikely to collide)
    let mut other = epher_core::Env::default();
    let o0 = eval_in_env("randseed(8)", &mut other).unwrap();
    assert_eq!(o0, Value::float(8.0));
    let o1 = eval_in_env("random()", &mut other).unwrap();
    assert!(o1 != u0, "different seed gives a different first draw");
}

#[test]
fn random_ranges_and_errors() {
    // random(a, b) is uniform in [a, b): every draw lands in range.
    let mut env = Env::default();
    let _ = eval_in_env("randseed(3)", &mut env).unwrap();
    for _ in 0..64 {
        let v = eval_in_env("random(-2, 2)", &mut env).unwrap();
        let x = match v {
            epher_core::Value::Float(x) => x,
            other => panic!("random returned {other:?}"),
        };
        assert!((-2.0..2.0).contains(&x), "draw {x} outside [-2, 2)");
    }
    assert!(
        eval_str_checked("random(2, 2)").is_err(),
        "a == b is an error"
    );
    assert!(
        eval_str_checked("random(3, 1)").is_err(),
        "a > b is an error"
    );
    assert!(
        eval_str_checked("random(5)").is_err(),
        "one arg is an error"
    );
}

#[test]
fn randint_is_inclusive_and_integral() {
    let mut env = epher_core::Env::default();
    let _ = eval_in_env("randseed(11)", &mut env).unwrap();
    // Reference draws for seed 11, m = 6 (SplitMix64 + Lemire).
    let mut expected = [2.0, 2.0, 4.0];
    for want in &mut expected {
        let v = eval_in_env("randint(1, 6)", &mut env).unwrap();
        assert_eq!(v, Value::float(*want), "reference draw");
    }
    // closed range: both endpoints are reachable
    let _ = eval_in_env("randseed(2)", &mut env).unwrap();
    let mut saw = std::collections::HashSet::new();
    for _ in 0..300 {
        let v = eval_in_env("randint(0, 1)", &mut env).unwrap();
        match v {
            Value::Float(x) => {
                assert!(x == 0.0 || x == 1.0);
                saw.insert(x as u32);
            }
            other => panic!("randint returned {other:?}"),
        }
    }
    assert!(saw.len() == 2, "both endpoints occur over 300 draws");
    assert!(
        eval_str_checked("randint(1.5, 3)").is_err(),
        "whole numbers only"
    );
    assert!(
        eval_str_checked("randint(3, 1)").is_err(),
        "a <= b required"
    );
}

#[test]
fn random_inside_functions_advances_the_shared_sequence() {
    let mut session = epher_core::Session::default();
    assert_eq!(session.submit("def d6() = randint(1, 6)"), "");
    assert_eq!(session.submit("randseed(11)"), "= 11");
    assert_eq!(session.submit("d6()"), "= 2");
    assert_eq!(session.submit("d6()"), "= 2");
    assert_eq!(session.submit("d6()"), "= 4");
    // and the session sequence continues where the function left off
    assert_eq!(session.submit("randint(1, 6)"), "= 4");
}

// ===== the constants library (ADR-0045) =====

#[test]
fn new_codata_constants_have_expected_values() {
    let cases = [
        ("m_P", 2.176_434e-8),
        ("l_P", 1.616_255e-35),
        ("t_P", 5.391_247e-44),
        ("r_e", 2.817_940_320_5e-15),
        ("lambda_c", 2.426_310_238_67e-12),
        ("mu_n", 5.050_783_699e-27),
        ("m_moon", 7.342e22),
        ("r_moon", 1.737_4e6),
    ];
    for (name, want) in cases {
        match eval_str(name) {
            epher_core::Value::Float(x) => {
                let rel = (x - want).abs() / want;
                assert!(rel < 1e-9, "{name}: {x} vs {want}");
            }
            other => panic!("{name} is not a float: {other:?}"),
        }
    }
}

#[test]
fn constant_groups_cover_every_builtin_and_resolve() {
    let groups = epher_core::builtin_constant_groups();
    let mut seen = std::collections::HashSet::new();
    let mut sorted = groups.iter().map(|(n, _)| *n).collect::<Vec<_>>();
    for &(name, _) in groups {
        assert!(seen.insert(name), "duplicate {name} in groups");
        match eval_str(name) {
            epher_core::Value::Float(_) | epher_core::Value::Complex(_) => {}
            other => panic!("group constant {name} does not evaluate: {other:?}"),
        }
    }
    sorted.sort_unstable();
    assert!(
        groups.iter().zip(sorted.iter()).all(|((n, _), s)| *n == *s),
        "groups are sorted by name"
    );
    // and every catalog constant appears in the groups
    for entry in epher_core::catalog() {
        if entry.kind == epher_core::CatalogKind::Constant {
            assert!(
                seen.contains(entry.name),
                "{} missing from groups",
                entry.name
            );
        }
    }
    assert!(
        groups.len() >= 40,
        "the library is substantial: {}",
        groups.len()
    );
}

// ===== units with conversion (ADR-0046) =====

/// The SI value of a quantity (the value field), else the float.
fn eval_si(src: &str) -> f64 {
    match eval_str(src) {
        Value::Float(x) => x,
        Value::Quantity { value, .. } => value,
        other => panic!("{src} produced {other:?}"),
    }
}

#[test]
fn quantities_arithmetic_tracks_dimensions() {
    // same dims add and keep the unit; mismatched dims are an error
    assert_eq!(format_value_of("5 m + 3 m"), "8 m");
    assert_eq!(format_value_of("5 hr + 3 hr"), "8 hr");
    assert!(eval_str_checked("5 m + 3 s").is_err(), "m + s errors");
    assert!(
        eval_str_checked("5 m + 3").is_err(),
        "number + quantity errors"
    );
    // multiplication and division compose dims
    assert_eq!(format_value_of("5 m * 3 m"), "15 m^2");
    assert_eq!(format_value_of("2 m / 3 s"), "0.666666666667 m/s");
    // a number scales a quantity
    assert_eq!(format_value_of("2 * 3 m"), "6 m");
    assert_eq!(format_value_of("(3 m) / 2"), "1.5 m");
    // powers scale the dims; unit powers bind to the unit
    assert_eq!(format_value_of("2 m^2"), "2 m^2");
    assert_eq!(format_value_of("(2 m)^2"), "4 m^2");
    assert_eq!(format_value_of("(3 m)^3"), "27 m^3");
    assert!(
        eval_str_checked("(3 m)^0.5").is_err(),
        "fractional power errors"
    );
    // dims that cancel collapse back to plain numbers
    assert_eq!(eval_str("5 m / 5 m"), Value::float(1.0));
    assert_eq!(eval_str("2 m * 3 s / 6 m / 1 s"), Value::float(1.0));
    // dimensionless quantities behave like numbers
    assert_eq!(eval_si("30 deg + 0.5"), std::f64::consts::PI / 6.0 + 0.5);
    assert_eq!(eval_si("2 * 30 deg"), std::f64::consts::PI / 3.0);
    assert_eq!(eval_str("(30 deg) / (60 deg)"), Value::float(0.5));
}

#[test]
fn dimension_errors_name_the_operands() {
    match eval_str_checked("5 m + 3 s") {
        Err(epher_core::EpherError::Dimension(msg)) => {
            assert_eq!(msg, "cannot add 5 m and 3 s");
        }
        other => panic!("expected a dimension error, got {other:?}"),
    }
    match eval_str_checked("5 m < 3 s") {
        Err(epher_core::EpherError::Dimension(msg)) => {
            assert_eq!(msg, "cannot compare 5 m and 3 s");
        }
        other => panic!("expected a dimension error, got {other:?}"),
    }
}

#[test]
fn the_conversion_operator_rescales_and_remembers() {
    // `in` and `->` are the same operator
    assert_eq!(
        format_value_of("60 mile/hr in km/hr"),
        "96.56064 km/hr"
    );
    assert_eq!(
        format_value_of("60 mile/hr -> km/hr"),
        "96.56064 km/hr"
    );
    assert_eq!(format_value_of("1 km in m"), "1000 m");
    assert_eq!(format_value_of("3.2 AU in km"), "478713186.24 km");
    // a plain number converts into a quantity
    assert_eq!(format_value_of("5 in km"), "5 km");
    assert_eq!(format_value_of("5 -> kg"), "5 kg");
    // the dims must match
    match eval_str_checked("5 m in s") {
        Err(epher_core::EpherError::Dimension(msg)) => {
            assert!(msg.contains("cannot convert 5 m to s"), "{msg}");
        }
        other => panic!("expected a dimension error, got {other:?}"),
    }
    match eval_str_checked("2 m^2 in m") {
        Err(epher_core::EpherError::Dimension(_)) => {}
        other => panic!("expected a dimension error, got {other:?}"),
    }
    // conversions compose with arithmetic: `in` binds loosest
    assert_eq!(format_value_of("5 m + 3 m in km"), "0.008 km");
    // area conversion
    assert_eq!(format_value_of("2 m^2 in cm^2"), "20000 cm^2");
}

#[test]
fn si_prefixes_resolve_on_any_unit() {
    assert_eq!(format_value_of("1 km in m"), "1000 m");
    assert_eq!(format_value_of("5 ms in s"), "0.005 s");
    assert_eq!(format_value_of("3 MPa in Pa"), "3000000 Pa");
    assert_eq!(format_value_of("2 dam in m"), "20 m");
    assert_eq!(format_value_of("1 um in m"), "0.000001 m");
    assert_eq!(format_value_of("1 µs in s"), "0.000001 s");
    assert_eq!(format_value_of("2 kg in g"), "2000 g");
    // prefixed names are their own units in the display too
    assert_eq!(format_value_of("5 km"), "5 km");
    assert_eq!(format_value_of("1 GHz in Hz"), "1000000000 Hz");
}

#[test]
fn unit_chains_powers_and_roots() {
    assert_eq!(format_value_of("60 mile/hr"), "60 mile/hr");
    assert!((eval_si("60 mile/hr") - 26.8224).abs() < 1e-9, "SI value");
    assert_eq!(format_value_of("5 m/s^2"), "5 m/s^2");
    assert_eq!(format_value_of("1 km/hr in m/s"), "0.277777777778 m/s");
    // sqrt halves even dims; odd dims are a dimension error
    assert_eq!(format_value_of("sqrt(4 m^2)"), "2 m");
    assert_eq!(format_value_of("sqrt(9 m^2) in km"), "0.003 km");
    match eval_str_checked("sqrt(4 m)") {
        Err(epher_core::EpherError::Dimension(msg)) => {
            assert!(msg.contains("square root"), "{msg}");
        }
        other => panic!("expected a dimension error, got {other:?}"),
    }
    // x / hr divides by the variable hr; the chain needs the suffix
    let mut env = Env::default();
    let script = parse_script("hr = 2; 60 mile/hr").expect("parses");
    let values = run_all(&script, &mut env).expect("runs");
    match values.last() {
        Some(Value::Quantity { value, .. }) => {
            assert!(
                (value - 26.8224).abs() < 1e-9,
                "chain still wins over the variable"
            )
        }
        other => panic!("60 mile/hr with hr=2 gave {other:?}"),
    }
    let mut env = Env::default();
    let script = parse_script("hr = 2; 12 / hr").expect("parses");
    let values = run_all(&script, &mut env).expect("runs");
    assert_eq!(
        values.last(),
        Some(&Value::float(6.0)),
        "x / hr stays a division"
    );
}

#[test]
fn derived_display_names_and_grouping() {
    assert_eq!(format_value_of("5 kg * 3 m / 1 s^2"), "15 N");
    assert_eq!(format_value_of("2 J"), "2 J");
    assert_eq!(format_value_of("1 W in mW"), "1000 mW");
    assert_eq!(format_value_of("2 m * 3 m / 1 s"), "6 m^2/s");
    // separators apply to the numeric part only
    let prefs = epher_core::DisplayPrefs {
        separators: true,
        ..epher_core::DisplayPrefs::default()
    };
    assert_eq!(
        epher_core::format_value(&eval_str("3.2 AU"), &prefs),
        "3.2 AU"
    );
    assert_eq!(
        epher_core::format_value(&eval_str("478713186240 m"), &prefs),
        "478\u{2009}713\u{2009}186\u{2009}240 m"
    );
}

#[test]
fn quantity_comparisons_and_shadowing() {
    assert_eq!(eval_str("5 m < 6 m"), Value::Bool(true));
    assert_eq!(eval_str("5 m == 5 m"), Value::Bool(true));
    assert_eq!(eval_str("5 m != 5 m"), Value::Bool(false));
    assert_eq!(
        eval_str("5 m == 5 km"),
        Value::Bool(false),
        "SI values compare"
    );
    // the unit grammar still beats user shadowing (ADR-0037)
    let mut env = Env::default();
    let script = parse_script("const m = 2; 5 m").expect("parses");
    let values = run_all(&script, &mut env).expect("runs");
    match values.last() {
        Some(Value::Quantity { value, .. }) => assert_eq!(*value, 5.0),
        other => panic!("5 m with const m = 2 gave {other:?}"),
    }
    // `in` stays usable as a variable name outside a conversion
    let mut env = Env::default();
    let script = parse_script("in = 5; in + 1").expect("parses");
    let values = run_all(&script, &mut env).expect("runs");
    assert_eq!(values.last(), Some(&Value::float(6.0)));
}

/// Format with the default display preferences.
fn format_value_of(src: &str) -> String {
    epher_core::format_value(&eval_str(src), &epher_core::DisplayPrefs::default())
}

// ===== bitwise operations (ADR-0047) =====

#[test]
fn bitwise_operators_compute_and_mask() {
    // the four operators on whole numbers
    assert_eq!(format!("{}", eval_str("0xFF & 0x0F")), "15");
    assert_eq!(format!("{}", eval_str("5 | 3")), "7");
    assert_eq!(format!("{}", eval_str("5 xor 3")), "6");
    assert_eq!(format!("{}", eval_str("~0")), "-1");
    assert_eq!(format!("{}", eval_str("1 << 8")), "256");
    assert_eq!(format!("{}", eval_str("-8 >> 1")), "-4");
    // results are exact Big values: 1 << 60 keeps every digit
    match eval_str("1 << 60") {
        Value::Big(b) => assert_eq!(b.to_string(), "1152921504606846976"),
        other => panic!("1 << 60 produced {other:?}"),
    }
    // the default word is 64 bits: 1 << 100 wraps to 0, ~0 stays -1
    assert_eq!(format!("{}", eval_str("1 << 100")), "0");
    assert_eq!(format!("{}", eval_str("~0 & 0xFF")), "255");
    // precedence: shift binds tighter than |, comparisons loosest
    assert_eq!(format!("{}", eval_str("1 | 2 << 3")), "17");
    assert_eq!(eval_str("5 & 3 == 1"), Value::Bool(true));
    // negative shifts reverse the direction
    assert_eq!(format!("{}", eval_str("8 << -1")), "4");
    assert_eq!(format!("{}", eval_str("8 >> -1")), "16");
    // non-whole operands are type errors
    assert!(eval_str_checked("5.5 & 3").is_err(), "fractional operand");
    assert!(eval_str_checked("1 << 0.5").is_err(), "fractional shift");
    assert!(eval_str_checked("5 & x").is_err(), "unknown operand");
}

#[test]
fn bits_word_size_is_session_state() {
    let mut session = Session::default();
    assert_eq!(session.submit("bits()"), "= 64");
    assert_eq!(session.submit("bits(8)"), "= 8");
    // ~0 in 8-bit two's complement is -1; 255 fits; 1 << 8 wraps to 0
    assert_eq!(session.submit("~0"), "= -1");
    assert_eq!(session.submit("255 & 1"), "= 1");
    assert_eq!(session.submit("1 << 8"), "= 0");
    assert_eq!(session.submit("255"), "= 255");
    // 16-bit words: 0xFFFF is -1 as a signed word
    assert_eq!(session.submit("bits(16)"), "= 16");
    assert_eq!(session.submit("0xFFFF & 0xFFFF"), "= -1");
    assert_eq!(session.submit("0x7FFF & 0x7FFF"), "= 32767");
    // 1 << 15 is the sign bit, so the max positive word is 32767
    assert_eq!(session.submit("(1 << 15) - 1"), "= -32769");
    assert_eq!(session.submit("0x7FFF"), "= 32767");
    // back to the default
    assert_eq!(session.submit("bits(64)"), "= 64");
    assert_eq!(session.submit("1 << 40"), "= 1099511627776");
    // invalid sizes are domain errors; non-whole sizes are type errors
    assert!(eval_str_checked("bits(7)").is_err(), "7 bits");
    assert!(eval_str_checked("bits(2.5)").is_err(), "fractional size");
}

#[test]
fn big_comparisons_work_across_types() {
    // the bitwise round exposed a pre-existing gap: exact types could
    // not compare; they can now (ADR-0047).
    assert_eq!(eval_str("big(5) == 5"), Value::Bool(true));
    assert_eq!(eval_str("big(5) < 6"), Value::Bool(true));
    assert_eq!(eval_str("big(5) > 5"), Value::Bool(false));
    assert_eq!(eval_str("(1 << 60) == big(2)^60"), Value::Bool(true));
    assert_eq!(eval_str("frac(1, 3) < 0.5"), Value::Bool(true));
    assert_eq!(
        eval_str("dec(0.1) + dec(0.2) == dec(0.3)"),
        Value::Bool(true)
    );
}

#[test]
fn matrix_literal_debug() {
    let e = epher_core::parse("[[1, 2], [3, 4]]");
    eprintln!("parse: {e:?}");
    assert!(e.is_ok());
}

// ===== matrices (ADR-0049) =====

#[test]
fn matrix_literals_arithmetic_and_indexing() {
    // the literal, echoed back
    assert_eq!(
        format!("{}", eval_str("[[1, 2], [3, 4]]")),
        "[[1, 2], [3, 4]]"
    );
    // elementwise + and -, the matrix product, and scaling
    assert_eq!(
        format!("{}", eval_str("[[1, 2], [3, 4]] + [[1, 1], [1, 1]]")),
        "[[2, 3], [4, 5]]"
    );
    assert_eq!(
        format!("{}", eval_str("[[1, 2], [3, 4]] * [[5, 6], [7, 8]]")),
        "[[19, 22], [43, 50]]"
    );
    assert_eq!(
        format!("{}", eval_str("[[1, 2], [3, 4]] * 2")),
        "[[2, 4], [6, 8]]"
    );
    assert_eq!(
        format!("{}", eval_str("2 * [[1, 2], [3, 4]]")),
        "[[2, 4], [6, 8]]"
    );
    assert_eq!(
        format!("{}", eval_str("-[[1, 2], [3, 4]]")),
        "[[-1, -2], [-3, -4]]"
    );
    // the matrix power: n = 0 is the identity
    assert_eq!(
        format!("{}", eval_str("[[1, 2], [3, 4]] ^ 2")),
        "[[7, 10], [15, 22]]"
    );
    assert_eq!(
        format!("{}", eval_str("[[1, 2], [3, 4]] ^ 0")),
        "[[1, 0], [0, 1]]"
    );
    // indexing: rows are lists, so M[2][1] is the element
    assert_eq!(format!("{}", eval_str("[[1, 2], [3, 4]][2]")), "{3, 4}");
    assert_eq!(format!("{}", eval_str("[[1, 2], [3, 4]][2][1]")), "3");
    // whole-matrix equality
    assert_eq!(
        eval_str("[[1, 2], [3, 4]] == [[1, 2], [3, 4]]"),
        Value::Bool(true)
    );
    assert_eq!(
        eval_str("[[1, 2], [3, 4]] != [[1, 1], [3, 4]]"),
        Value::Bool(true)
    );
    // shape and type errors
    assert!(eval_str_checked("[[1, 2], [3]]").is_err(), "ragged rows");
    assert!(
        eval_str_checked("[[1, 2]] + [[1, 2], [3, 4]]").is_err(),
        "shape mismatch"
    );
    assert!(
        eval_str_checked("[[1, 2]] * [[1, 2]]").is_err(),
        "product dims"
    );
    assert!(
        eval_str_checked("2 / [[1, 2]]").is_err(),
        "divide by a matrix"
    );
    assert!(eval_str_checked("[[1, 2]] < [[1, 2]]").is_err(), "ordering");
    assert!(
        eval_str_checked("[[1, 2], [3, 4]] ^ 0.5").is_err(),
        "fractional power"
    );
}

#[test]
fn matrix_functions_cover_the_numworks_floor() {
    // det and trace
    assert_eq!(format!("{}", eval_str("det([[1, 2], [3, 4]])")), "-2");
    assert_eq!(
        format!("{}", eval_str("det([[2, 0, 0], [0, 3, 0], [0, 0, 5]])")),
        "30"
    );
    assert_eq!(format!("{}", eval_str("trace([[1, 2], [3, 4]])")), "5");
    // inv with exact-fraction display
    assert_eq!(
        format_value_of("inv([[1, 2], [3, 4]])"),
        // 3/2 is a terminating decimal, so it displays as 1.5
        "[[-2, 1], [1.5, -0.5]]"
    );
    assert_eq!(
        format_value_of("inv([[1, 2], [3, 4]]) * [[1, 2], [3, 4]]"),
        "[[1, 0], [0, 1]]"
    );
    // singular matrices are a domain error
    match eval_str_checked("inv([[1, 2], [2, 4]])") {
        Err(epher_core::EpherError::Domain(msg)) => assert!(msg.contains("singular"), "{msg}"),
        other => panic!("expected a domain error, got {other:?}"),
    }
    // transpose and dim
    assert_eq!(
        format!("{}", eval_str("transpose([[1, 2], [3, 4]])")),
        "[[1, 3], [2, 4]]"
    );
    assert_eq!(
        format!("{}", eval_str("dim([[1, 2], [3, 4], [5, 6]])")),
        "{3, 2}"
    );
    assert_eq!(format!("{}", eval_str("dim([[1, 2], [3, 4]])[1]")), "2");
    // rref solves the classic system; ref stops at the echelon form
    assert_eq!(
        format_value_of("rref([[2, 1, 5], [1, -1, 1]])"),
        "[[1, 0, 2], [0, 1, 1]]"
    );
    let refd = format!("{}", eval_str("ref([[2, 1, 5], [1, -1, 1]])"));
    assert!(
        refd.starts_with("[[1, 0.5"),
        "ref is the echelon form: {refd}"
    );
    // non-square det/trace are domain errors; a matrix arg is required
    assert!(eval_str_checked("det([[1, 2, 3], [4, 5, 6]])").is_err());
    assert!(eval_str_checked("trace(5)").is_err());
}

// ===== finance (ADR-0050) =====

#[test]
fn tvm_solves_any_field_of_the_mortgage() {
    // The classic 8% mortgage: 360 monthly payments of 733.76 against
    // a 100,000 loan (TI sign convention: money out is negative).
    // The rate for a 733.76 payment is just under the nominal 8%/12:
    // bisection lands on the true root for the rounded payment.
    let i = 0.006_666_611_990_680_783;
    // The payment's true value (the exact rational TVM root) is
    // 733.764573879376...; 327259/446 differs at the 9th digit, so
    // the half-display-unit tolerance keeps the decimal (ADR-0052).
    assert_eq!(
        format_value_of("tvm_pmt(360, 0.08/12, -100000, 0)"),
        "733.764573879"
    );
    assert!((eval_f64("tvm_pmt(360, 0.08/12, -100000, 0)") - 733.764_573_99).abs() < 1e-6);
    assert!((eval_f64("tvm_n(0.08/12, -100000, 733.76, 0)") - 360.009_321_3).abs() < 1e-4);
    assert!((eval_f64("tvm_i(360, -100000, 733.76, 0)") - i).abs() < 1e-12);
    // 360 payments of 733.76 at 8%/12 are worth 99999.3766557 today
    // (the exact rational value; 7699952/77 differs at the 6th digit)
    assert_eq!(
        format_value_of("tvm_pv(360, 0.08/12, 733.76, 0)"),
        "-99999.3766557"
    );
    // a 12-month 1% loan of 1000 with payments of 88.85 ends at ~0
    assert!(eval_f64("tvm_fv(12, 0.01, -1000, 88.85)").abs() < 0.05);
    // annuity-due timing lowers the payment
    let end = eval_f64("tvm_pmt(12, 0.01, -1000, 0)");
    let begin = eval_f64("tvm_pmt(12, 0.01, -1000, 0, 1)");
    assert!(begin < end, "begin payments are smaller: {begin} vs {end}");
    assert!((end - 88.848_788_678).abs() < 1e-6);
    // zero-rate closed forms
    assert_eq!(eval_f64("tvm_fv(5, 0, -100, 20)"), 0.0);
    assert_eq!(eval_f64("tvm_pmt(5, 0, -100, 0)"), 20.0);
    // out-of-range problems report the searched range
    match eval_str_checked("tvm_i(12, -1000, 0, 0)") {
        Err(epher_core::EpherError::Domain(msg)) => assert!(msg.contains("no solution"), "{msg}"),
        other => panic!("expected a domain error, got {other:?}"),
    }
    assert!(
        eval_str_checked("tvm_pmt(12, 0.01, -1000, 0, 2)").is_err(),
        "timing must be 0/1"
    );
    assert!(
        eval_str_checked("tvm_pmt(12, 0.01, -1000)").is_err(),
        "four fields needed"
    );
}

#[test]
fn npv_irr_and_amortization_match_the_references() {
    assert!((eval_f64("npv(0.1, {-100, 60, 60})") - 4.132_231_4).abs() < 1e-6);
    assert!((eval_f64("irr({-100, 60, 60})") - 0.130_662_386_3).abs() < 1e-9);
    assert!((eval_f64("irr({-1000, 500, 500, 500})") - 0.233_751_928_5).abs() < 1e-9);
    // the amortization schedule: balance after k payments
    assert_eq!(eval_f64("amort(1000, 0.01, 12, 0)"), 1000.0);
    assert!((eval_f64("amort(1000, 0.01, 12, 6)") - 514.921_064_58).abs() < 1e-6);
    assert!(eval_f64("amort(1000, 0.01, 12, 12)").abs() < 1e-9);
    assert!(
        eval_f64("amort(1000, 0, 10, 5)") == 500.0,
        "zero-rate prorates"
    );
    // simple and compound interest
    assert_eq!(eval_f64("simple_interest(1000, 0.05, 2)"), 100.0);
    assert!((eval_f64("compound_interest(1000, 0.05, 2)") - 102.5).abs() < 1e-12);
    // errors: bad lists and ranges
    assert!(
        eval_str_checked("npv(0.1, 5)").is_err(),
        "a list is required"
    );
    assert!(
        eval_str_checked("amort(1000, 0.01, 12, 13)").is_err(),
        "k beyond n"
    );
    assert!(
        eval_str_checked("amort(1000, 0.01, 12.5, 3)").is_err(),
        "whole periods"
    );
}

// ===== display rounding and terminating decimals (ADR-0051) =====

#[test]
fn display_rounds_to_twelve_significant_digits() {
    use epher_core::{format_value, DisplayPrefs};
    let auto = DisplayPrefs::default();
    let f = |s: &str| format_value(&evaluate(s).expect("eval"), &auto);
    // terminating decimals stay decimals, clean or reconstructed
    assert_eq!(f("0.1"), "0.1");
    assert_eq!(f("0.5"), "0.5");
    assert_eq!(f("0.125"), "0.125");
    assert_eq!(f("0.1 + 0.2"), "0.3");
    assert_eq!(f("0.30000000000000004"), "0.3");
    assert_eq!(f("100.1 - 100"), "0.1");
    assert_eq!(f("200 + 10%"), "200.1");
    assert_eq!(f("4.2"), "4.2");
    assert_eq!(f("784.8000000000001"), "784.8");
    // repeating values keep the fraction
    assert_eq!(f("1/3"), "1/3");
    assert_eq!(f("2/3"), "2/3");
    assert_eq!(f("1/7"), "1/7");
    assert_eq!(f("355/113"), "355/113");
    // irrationals round to twelve digits
    assert_eq!(f("sqrt(2)"), "1.41421356237");
    assert_eq!(f("pi"), "3.14159265359");
    assert_eq!(f("sin(pi)"), "0.000000000000000122464679915");
    // exact integers never round
    assert_eq!(f("1234567890123"), "1234567890123");
    assert_eq!(f("1234567890123456"), "1234567890123456");
    // fractions off still rounds the float the same way
    let plain = DisplayPrefs {
        exact_fractions: false,
        ..DisplayPrefs::default()
    };
    let p = |s: &str| format_value(&evaluate(s).expect("eval"), &plain);
    assert_eq!(p("0.1 + 0.2"), "0.3");
    assert_eq!(p("1/3"), "0.333333333333");
    // exact() keeps reconstructing on request
    assert_eq!(eval_display("exact(0.30000000000000004)"), "3/10");

    // The half-display-unit reconstruction tolerance (ADR-0052): a
    // fraction shows only when it agrees with the value through all
    // twelve displayed digits. Large decimals with a coincidental
    // convergent (123456.789 used to display as 13456790/109, whose
    // decimal differs at the 9th digit) stay decimals now; genuine
    // repeating values and exact rationals keep their fractions.
    assert_eq!(f("123456.789"), "123456.789");
    assert_eq!(f("1234567.891"), "1234567.891");
    assert_eq!(f("1/3"), "1/3");
    assert_eq!(f("355/113"), "355/113");
    assert_eq!(f("500/121"), "500/121");
    assert_eq!(f("1/7"), "1/7");
    // the TVM payment and present value no longer display misleading
    // coincidental fractions: the true roots are 733.764573879376...
    // and -99999.376655736...
    assert_eq!(f("tvm_pmt(360, 0.08/12, -100000, 0)"), "733.764573879");
    assert_eq!(f("tvm_pv(360, 0.08/12, 733.76, 0)"), "-99999.3766557");
    // exact() still reconstructs what the display hides
    assert_eq!(f("exact(0.3333333333333333)"), "1/3");
    assert_eq!(f("exact(0.30000000000000004)"), "3/10");
    assert_eq!(f("exact(123456.789)"), "123456789/1000");
}

#[test]
fn quantity_values_round_to_twelve_digits_too() {
    use epher_core::{format_value, DisplayPrefs};
    let auto = DisplayPrefs::default();
    let f = |s: &str| format_value(&evaluate(s).expect("eval"), &auto);
    // The value inside a quantity goes through the same rounding as
    // every other result line (ADR-0052): the raw 16-digit spelling of
    // `30 deg in rad` is gone.
    assert_eq!(f("30 deg in rad"), "0.523598775598");
    assert_eq!(f("rad(30)"), "0.523598775598");
    assert_eq!(f("2 m / 3 s"), "0.666666666667 m/s");
    assert_eq!(f("60 mile/hr in km/hr"), "96.56064 km/hr");
    // integers keep their exact spelling via the length guard
    assert_eq!(f("1 AU in m"), "149597870700 m");
    assert_eq!(f("1 pc in m"), "30856775814913670 m");
}

#[test]
fn submit_all_returns_every_answer_in_order() {
    let mut session = Session::new();
    // A script's whole transcript, one answer per line, in order
    // (ADR-0052); the history entry keeps the last answer suffix.
    assert_eq!(session.submit_all("x = 10; y = x + 5; x + y"), "= 10\n= 15\n= 25");
    assert_eq!(session.history().last().unwrap(), "x = 10; y = x + 5; x + y  = 25");
    assert_eq!(session.submit_all("x * 2"), "= 20");
    // def produces no answer; a later expression does
    assert_eq!(session.submit_all("def f(t) = t ^ 2\nf(3)"), "= 9");
    // while loops produce no value; the surrounding statements do
    assert_eq!(session.submit_all("x = 0; while x < 5 do x = x + 1; x"), "= 0\n= 5");
    // single-value lines behave exactly like submit
    assert_eq!(session.submit_all("2 + 3"), "= 5");
    assert_eq!(session.submit_all(""), "");
}

// ===== stats class and the language surface (ADR-0054) =====

#[test]
fn randn_is_seeded_and_distributed() {
    // Reproducible: the same seed, the same draws; the property the
    // seeded-random ADR (0045) established for `random`.
    let a = run_script_text("randseed(7)\nrandn(0, 1)\nrandn(10, 2)");
    let b = run_script_text("randseed(7)\nrandn(0, 1)\nrandn(10, 2)");
    assert_eq!(a, b);
    assert_eq!(a.len(), 3); // randseed also returns a value
    let v = match a[1].clone() {
        epher_core::Value::Float(x) => x,
        other => panic!("expected a float, got {other:?}"),
    };
    assert!(v.is_finite(), "the draw is a real number, got {v}");
    // Same seed, same stream position: mu and sigma shift the same
    // standard draw (the z underneath c[1] is the z underneath a[1]).
    let c = run_script_text("randseed(7)\nrandn(10, 2)");
    let c1 = match c[1].clone() {
        epher_core::Value::Float(x) => x,
        other => panic!("expected a float, got {other:?}"),
    };
    assert!((c1 - (10.0 + 2.0 * v)).abs() < 1e-9, "{c1} vs mu+2*{v}");
    // zero sigma is rejected (sigma must be positive)
    assert!(script_err("randn(0, 0)").contains("sigma > 0"));
}

#[test]
fn anova_reports_f_and_p() {
    // Perfectly separated groups: F(2,6) = 27, p ~= 0.001.
    assert_eq!(
        eval_display_script("anova({1, 2, 3}, {4, 5, 6}, {7, 8, 9})"),
        "F = 27, p = 0.001"
    );
    // Identical groups: no effect at all.
    assert_eq!(
        eval_display_script("anova({1, 2, 3}, {1, 2, 3})"),
        "F = 0, p = 1"
    );
    // Degenerate inputs are domain errors, not panics.
    assert!(script_err("anova({1})").contains("at least two lists"));
    assert!(script_err("anova({1}, {2})").contains("more data points than groups"));
}

#[test]
fn ttestpaired_tests_the_differences() {
    // Pairs (80,82) (85,84) (90,91): differences -2, 1, -1.
    assert_eq!(
        eval_display_script("ttestpaired({80, 85, 90}, {82, 84, 91})"),
        "t = -0.7559, p = 0.5286"
    );
    // Length mismatch is a type error.
    assert!(script_err("ttestpaired({1, 2}, {1})").contains("different lengths"));
}

#[test]
fn regression_family_fits_and_reports_r() {
    // Exact y = x^2 recovers exactly.
    assert_eq!(
        eval_display_script("quadreg({1, 2, 3, 4}, {1, 4, 9, 16})"),
        "y = 1*x^2 + 0*x + 0 (r = 1)"
    );
    // expreg on y = 2*e^(x) recovers a = 2, b = 1.
    let out = eval_display_script("expreg({1, 2, 3}, {5.43656, 14.7781, 40.17107})");
    assert!(out.starts_with("y = 2*e^(1*x)"), "{out}");
    // powreg on y = 3*x^2 recovers a = 3, b = 2.
    let out = eval_display_script("powreg({1, 2, 3}, {3, 12, 27})");
    assert!(out.starts_with("y = 3*x^2"), "{out}");
    // logreg on y = 5 + 2*ln(x) recovers a = 5, b = 2.
    let out = eval_display_script("logreg({1, 2, 3}, {5, 6.386294, 7.197225})");
    assert!(out.starts_with("y = 5 + 2*ln(x)"), "{out}");
    // Domain honesty: transformed models reject out-of-domain data.
    assert!(script_err("expreg({1, 2}, {-1, 2})").contains("y > 0"));
    assert!(script_err("powreg({-1, 2}, {1, 2})").contains("x > 0"));
    assert!(script_err("logreg({0, 2}, {1, 2})").contains("x > 0"));
    assert!(script_err("quadreg({1, 2}, {1, 4})").contains("at least 3"));
}

#[test]
fn strings_concatenate_compare_and_index() {
    assert_eq!(eval_display_script("\"hello\" + \" \" + \"world\""), "hello world");
    assert_eq!(eval_display_script("len(\"hello\")"), "5");
    assert_eq!(eval_display_script("len(\"\")"), "0");
    assert_eq!(eval_display_script("\"hello\"[1]"), "h");
    assert_eq!(eval_display_script("\"hello\"[5]"), "o");
    assert_eq!(eval_display_script("\"a\" == \"a\""), "true");
    assert_eq!(eval_display_script("\"a\" != \"b\""), "true");
    // Mixed arithmetic with a string is a type error, not a surprise.
    assert!(script_err("\"a\" + 1").contains("only support +"));
    // Ordering is deliberately unsupported.
    assert!(script_err("\"a\" < \"b\"").contains("cannot compare"));
    // len reaches strings as well as lists.
    assert_eq!(eval_display_script("len({1, 2, 3})"), "3");
    // A string survives a variable round trip.
    assert_eq!(eval_display_script("s = \"hi there\"\nlen(s)"), "8");
    // Unterminated strings are a parse error, not a panic.
    assert!(parse_script("s = \"oops")
        .unwrap_err()
        .to_string()
        .contains("unterminated"));
}

#[test]
fn for_loops_iterate_ranges_and_lists() {
    // The classic range: inclusive, collecting the body values.
    assert_eq!(
        eval_display_script("for i in 1 to 5 do i^2"),
        "{1, 4, 9, 16, 25}"
    );
    // List iteration: the data-column loop.
    assert_eq!(
        eval_display_script("d = {2, 3, 4}\nfor x in d do 10*x"),
        "{20, 30, 40}"
    );
    // Step: half steps land exactly (no drift).
    assert_eq!(eval_display_script("for i in 0 to 1 step 0.5 do i"), "{0, 0.5, 1}");
    // Negative steps count down.
    assert_eq!(eval_display_script("for i in 3 to 1 step -1 do i"), "{3, 2, 1}");
    // A reversed range with positive step is simply empty.
    assert_eq!(eval_display_script("for i in 5 to 1 do i"), "{}");
    // The loop variable keeps its last value (TI's For behavior).
    assert_eq!(eval_display_script("for i in 1 to 3 do i\ni"), "3");
    // print turns the loop into readable lines.
    assert_eq!(
        eval_display_script("for i in 1 to 3 do print(\"line\", i)"),
        "{line 1, line 2, line 3}"
    );
    // Assignments inside the body work like any script statement.
    assert_eq!(
        eval_display_script("total = 0\nfor i in 1 to 4 do total = total + i"),
        "{1, 3, 6, 10}"
    );
    // Runaway guards: zero step and absurd ranges are domain errors.
    assert!(script_err("for i in 1 to 3 step 0 do i").contains("nonzero"));
    assert!(script_err("for i in 1 to 200000 do i").contains("at most 100000"));
    // Iterating a non-list is a type error with guidance.
    assert!(script_err("for i in 5 do i").contains("list or a range"));
}

#[test]
fn str_and_print_format_like_the_display() {
    // str spells one value the way the answer panel does.
    assert_eq!(eval_display_script("str(42)"), "42");
    assert_eq!(eval_display_script("str(0.1 + 0.2)"), "0.3");
    // print joins with spaces.
    assert_eq!(eval_display_script("print(\"x =\", 42)"), "x = 42");
    assert_eq!(eval_display_script("print()"), "");
    // strings concatenate onto printed results.
    assert_eq!(eval_display_script("print(\"a\") + \"!\""), "a!");
}
