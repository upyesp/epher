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
    match eval(&parse(src).expect("parse"), &env) {
        Err(e) => e.to_string(),
        Ok(v) => panic!("expected an error from {src:?}, got {v}"),
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
    assert!(eval_err("asin(2)").contains("domain"));
    assert!(eval_err("asin(-1.5)").contains("domain"));
    assert!(eval_err("acos(-2)").contains("domain"));
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
    assert!(eval_err("acosh(0.5)").contains("domain"));
    assert!(eval_err("atanh(1)").contains("domain"));
    assert!(eval_err("atanh(-1.1)").contains("domain"));
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
    assert!(eval_err("ln(0)").contains("domain"));
    assert!(eval_err("ln(-1)").contains("domain"));
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
    assert!(eval_err("log(0)").contains("domain"));
    assert!(eval_err("log(-5)").contains("domain"));
    assert!(eval_err("log2(0)").contains("domain"));
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
    approx("degree", "30 deg", std::f64::consts::PI / 6.0);
    approx("degree tight", "30deg", std::f64::consts::PI / 6.0);
    approx("arcminute", "1 arcmin", std::f64::consts::PI / 10800.0);
    approx("arcsecond", "1 arcsec", std::f64::consts::PI / 648000.0);
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
    match epher_core::run_all(&epher_core::parse_script(script).expect("parses"), &mut epher_core::Env::default()) {
        Ok(values) => {
            let last = values.last().expect("a value");
            assert_eq!(*last, epher_core::Value::float(2.0 * 1.495_978_707e11));
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
    assert_eq!(values.last(), Some(&epher_core::Value::float(86400.0 + 18000.0)));
}

// ===== Astronomy constants (ADR-0037) =====

#[test]
fn astronomy_constants_resolve_like_pi_and_are_shadowable() {
    approx("c", "c", 2.997_924_58e8);
    approx("g", "g", 9.80665);
    approx("h", "h", 6.626_070_15e-34);
    approx("h_bar", "h_bar", 6.626_070_15e-34 / (2.0 * std::f64::consts::PI));
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
    approx("jd with fractional hour", "jd(2000, 1, 1, 12.5)", 2451545.020_833_333);
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
    assert!((0.47..0.56).contains(&transit), "transit fraction = {transit}");
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
    assert!((0.0..90.0).contains(&phase), "mars phase at opposition = {phase}");
    let illum = float_at("illum(4, jd(2020, 10, 6))");
    assert!((0.7..1.01).contains(&illum), "mars illum at opposition = {illum}");
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
    approx("september equinox 2000", "september_equinox(2000)", 2451810.228);
    approx("december solstice 2000", "december_solstice(2000)", 2451900.068);
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
    assert!(epher_core::evaluate("jd(2020, 2, 29)").is_ok(), "2020 is a leap year");
    assert!(epher_core::evaluate("jd(2023, 2, 29)").is_err(), "2023 is not");
    assert!(epher_core::evaluate("jd(2023, 2, 30)").is_err());
    assert!(epher_core::evaluate("jd(2000, 4, 31)").is_err(), "April has 30");
    assert!(epher_core::evaluate("jd(2000, 2, 29)").is_ok(), "divisible by 400");
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
    assert!((0.03..0.06).contains(&transit), "pluto transit fraction = {transit}");
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
    assert_eq!(eval_str("big(10) ^ 40"), Value::Big("1e+40".parse().unwrap()));
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
    let err = epher_core::evaluate("(-8) ^ (1 / 3)").unwrap_err().to_string();
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
        run_script("const a = 1\nconst a = 1\na + 1").unwrap().to_string(),
        "2"
    );
    // a changed value keeps the documented error
    let err = run_script("const b = 1\nconst b = 2")
        .unwrap_err()
        .to_string();
    assert!(err.contains("already defined"), "{err}");
    // a constant still never takes a variable's name
    let err = run_script("c = 1\nconst c = 2")
        .unwrap_err()
        .to_string();
    assert!(err.contains("already a variable"), "{err}");
}
