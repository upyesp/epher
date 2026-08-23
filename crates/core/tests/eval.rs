use epher_core::{
    eval, evaluate, parse, parse_latex, parse_script, run, run_all, sample, sample_parametric,
    sample_polar, Sample, Env, Session, Value,
};
use bigdecimal::BigDecimal;
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
    let script =
        parse_script("def fact(n) = if n <= 1 then 1 else n * fact(n - 1); fact(5)")
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
    assert_eq!(samples, vec![Sample { x: -1.0, y: -1.0 }, Sample { x: 1.0, y: 1.0 }]);
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
    assert_eq!(
        eval_str("frac(1, 3)").to_string(),
        "1/3"
    );
    assert_eq!(
        eval_str("dec(0.1) + dec(0.2)").to_string(),
        "0.3"
    );
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
    assert_eq!(run_script("const pi = 3; pi * 2").unwrap(), Value::float(6.0));
}

#[test]
fn const_prefixed_variable_names_are_still_assignments() {
    assert_eq!(run_script("const_tax = 5; const_tax").unwrap(), Value::float(5.0));
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
    assert_eq!(values, vec![Value::float(10.0), Value::float(15.0), Value::float(25.0)]);
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
