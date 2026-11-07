//! In-process timing harness for epher-core (calculator benchmark, stage 3).
//!
//! Boundary: parse + eval only. No printing or formatting happens inside a
//! timed span; the harness binary is the "small harness binary that links the
//! core crate" the benchmark plan defines (plan rule 3). Statement execution
//! mirrors the real script path's step model: `run` is called once per
//! statement, so each statement gets a fresh STEP_LIMIT exactly as the
//! shell's per-piece `submit_all` does.
//!
//! Modes:
//!   expr-batch <file> <warmups> <samples>   one pass = parse + eval of every line
//!   expr-verify <file>                      print each line's value (gate input)
//!   script-run  <file> <warmups> <samples>  one pass = parse_script + per-statement run
//!   script-verify <file>                    print each statement value (gate input)
//!
//! Output: one JSON object on stdout. Pass times are milliseconds (f64).
use std::io::{BufRead, Write};
use std::time::Instant;

use epher_core::{run, Env, Value};

fn to_f64(v: &Value) -> f64 {
    match v {
        Value::Float(f) => *f,
        Value::Rational(r) => {
            let n: f64 = r.numer().to_string().parse().unwrap_or(f64::NAN);
            let d: f64 = r.denom().to_string().parse().unwrap_or(f64::NAN);
            if d == 0.0 { f64::NAN } else { n / d }
        }
        Value::Decimal(d) => d.to_string().parse().unwrap_or(f64::NAN),
        Value::Big(b) => b.to_string().parse().unwrap_or(f64::NAN),
        Value::Complex(c) => {
            if c.im == 0.0 { c.re } else { f64::NAN }
        }
        Value::Bool(b) => {
            if *b {
                1.0
            } else {
                0.0
            }
        }
        _ => f64::NAN,
    }
}

fn read_lines(path: &str) -> Vec<String> {
    let f = std::fs::File::open(path).expect("open input file");
    std::io::BufReader::new(f)
        .lines()
        .map_while(Result::ok)
        .filter(|l| !l.trim().is_empty())
        .collect()
}

fn expr_pass(lines: &[String], env: &mut Env) -> Result<Option<Value>, String> {
    let mut last = None;
    for line in lines {
        let stmts = epher_core::parse_script(line).map_err(|e| e.to_string())?;
        last = run(&stmts, env).map_err(|e| e.to_string())?;
    }
    Ok(last)
}

fn script_pass(text: &str, env: &mut Env) -> Result<Option<Value>, String> {
    let stmts = epher_core::parse_script(text).map_err(|e| e.to_string())?;
    let mut last = None;
    // One `run` per statement: a fresh step budget each, mirroring the
    // shell's split-then-submit script semantics.
    for i in 0..stmts.len() {
        last = run(&stmts[i..i + 1], env).map_err(|e| e.to_string())?;
    }
    Ok(last)
}

fn timed(
    kind: &str,
    passes: impl Fn(&mut Env) -> Result<Option<Value>, String>,
    warmups: usize,
    samples: usize,
    verify_value: Option<f64>,
) {
    let mut env = Env::default();
    for _ in 0..warmups {
        passes(&mut env).expect("warm-up pass failed");
    }
    let mut times_ms = Vec::with_capacity(samples);
    let mut last_f64 = f64::NAN;
    for _ in 0..samples {
        let t0 = Instant::now();
        let value = passes(&mut env).expect("measured pass failed");
        times_ms.push(t0.elapsed().as_secs_f64() * 1000.0);
        if let Some(v) = value {
            last_f64 = to_f64(&v);
        }
    }
    let mut out = String::from("{");
    out.push_str(&format!("\"kind\":\"{kind}\",\"samples\":{samples},"));
    out.push_str(&format!(
        "\"passes_ms\":{},",
        serde_json::to_string(&times_ms).unwrap()
    ));
    match verify_value {
        Some(v) => out.push_str(&format!("\"final\":{v:.17e}")),
        None => out.push_str(&format!("\"final\":{last_f64:.17e}")),
    }
    out.push('}');
    println!("{out}");
}

fn verify(lines_mode: bool, path: &str) {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    if lines_mode {
        let lines = read_lines(path);
        let mut env = Env::default();
        for line in &lines {
            let value = expr_pass(std::slice::from_ref(line), &mut env)
                .expect("verify pass failed");
            let f = value.map(|v| to_f64(&v)).unwrap_or(f64::NAN);
            writeln!(out, "{f:.17e}").expect("write");
        }
    } else {
        let text = std::fs::read_to_string(path).expect("read script");
        let mut env = Env::default();
        let value = script_pass(&text, &mut env).expect("verify pass failed");
        let f = value.map(|v| to_f64(&v)).unwrap_or(f64::NAN);
        writeln!(out, "{f:.17e}").expect("write");
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let usage = "usage: bench_eval <expr-batch|expr-verify|script-run|script-verify> <file> [warmups samples]";
    match args.get(1).map(String::as_str) {
        Some("expr-batch") => {
            let (w, s) = counts(&args, usage);
            let lines = read_lines(&args[2].clone());
            timed("expr-batch", |env| expr_pass(&lines, env), w, s, None);
        }
        Some("expr-verify") => verify(true, &args[2]),
        Some("script-run") => {
            let (w, s) = counts(&args, usage);
            let text = std::fs::read_to_string(&args[2]).expect("read script");
            timed("script-run", |env| script_pass(&text, env), w, s, None);
        }
        Some("script-verify") => verify(false, &args[2]),
        _ => {
            eprintln!("{usage}");
            std::process::exit(2);
        }
    }
}

fn counts(args: &[String], usage: &str) -> (usize, usize) {
    if args.len() < 5 {
        eprintln!("{usage}");
        std::process::exit(2);
    }
    (
        args[3].parse().expect("warmups"),
        args[4].parse().expect("samples"),
    )
}
