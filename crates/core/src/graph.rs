//! Graph command parsing and analysis (ADR-0006, ADR-0014): everything a
//! frontend needs to turn a `graph …` line into plottable data lives here —
//! the command grammar, per-curve sampling, points of interest (roots,
//! intersections, extrema), tables of values, and tick-step selection.
//! Frontends only render.

use std::collections::BTreeSet;

use crate::{eval, evaluate, parse, CmpOp, Env, EpherError, Expression, Sample, Value};

/// The default x (or t/θ) domain for a curve kind, when the command names no
/// bounds.
pub fn default_domain(kind: &CurveKind) -> (f64, f64) {
    match kind {
        CurveKind::Cartesian(_) | CurveKind::Implicit(_) => (-10.0, 10.0),
        CurveKind::Parametric { .. } | CurveKind::Polar(_) => (0.0, std::f64::consts::TAU),
    }
}

/// How a curve fills toward the plot edge (`y < f(x)` / `y > f(x)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fill {
    Below,
    Above,
}

/// The kind of curve requested by a graph command.
#[derive(Debug, Clone, PartialEq)]
pub enum CurveKind {
    Cartesian(Expression),
    Parametric {
        x: Expression,
        y: Expression,
    },
    Polar(Expression),
    /// An implicit relation (ADR-0048): the whole equation
    /// (`x^2 + y^2 == 1`), sampled with marching squares.
    Implicit(Expression),
}

/// A parsed graph command: what to plot and over which domain.
#[derive(Debug, Clone, PartialEq)]
pub struct CurveSpec {
    pub kind: CurveKind,
    pub domain: (f64, f64),
    pub fill: Option<Fill>,
}

/// A sampled curve ready to render — the seam payload every frontend holds.
#[derive(Debug, Clone, PartialEq)]
pub struct SampledCurve {
    /// What the user typed after `graph` (the accessible caption/legend text).
    pub source: String,
    pub kind: CurveKind,
    pub domain: (f64, f64),
    pub samples: Vec<Sample>,
    pub fill: Option<Fill>,
}

/// Parse the text after `graph ` into a [`CurveSpec`].
///
/// Grammar (case-sensitive, matching the expression language):
/// - `y = f(x)`-style: `expr`, optionally prefixed `y <`, `y <=`, `y >`,
///   `y >=` for region filling
/// - parametric: `param <x(t)>, <y(t)>`
/// - polar: `polar <r(θ)>`
/// - any form may end with `from a to b` (numeric bounds, expressions with
///   built-in constants allowed — the language has no `from` identifier, so
///   the keyword can never collide with the expression itself)
pub fn parse_graph_source(source: &str) -> Result<CurveSpec, EpherError> {
    let source = source.trim();
    if source.is_empty() {
        return Err(EpherError::Parse("empty graph command".to_string()));
    }

    let (body, domain) = split_domain(source)?;
    let fill: Option<Fill>;
    let kind = if let Some(rest) = body.strip_prefix("param ") {
        fill = None;
        let parts = split_top_level(rest, ',');
        match parts.as_slice() {
            [x, y] => CurveKind::Parametric {
                x: parse(x.trim())?,
                y: parse(y.trim())?,
            },
            _ => {
                return Err(EpherError::Parse(
                    "parametric graphs need two expressions: `param <x(t)>, <y(t)>`".to_string(),
                ))
            }
        }
    } else if let Some(rest) = body.strip_prefix("polar ") {
        fill = None;
        CurveKind::Polar(parse(rest.trim())?)
    } else {
        let (expr, f) = match body.strip_prefix("y <=") {
            Some(r) => (r, Some(Fill::Below)),
            None => match body.strip_prefix("y >=") {
                Some(r) => (r, Some(Fill::Above)),
                None => match body.strip_prefix("y <") {
                    Some(r) => (r, Some(Fill::Below)),
                    None => match body.strip_prefix("y >") {
                        Some(r) => (r, Some(Fill::Above)),
                        None => (body, None),
                    },
                },
            },
        };
        fill = f;
        let parsed = parse(expr.trim())?;
        // An implicit relation (ADR-0048): the top level is `==`.
        // `x^2 + y^2 == 1`, `y == x^2`, and `x == 2` all plot; the
        // inequality fills above stay Cartesian.
        if matches!(parsed, Expression::Compare(CmpOp::Eq, ..)) {
            if f.is_some() {
                return Err(EpherError::Parse(
                    "implicit relations cannot be filled; drop the y < or y > prefix".to_string(),
                ));
            }
            CurveKind::Implicit(parsed)
        } else {
            CurveKind::Cartesian(parsed)
        }
    };
    let domain = match domain {
        Some(d) => d,
        None => default_domain(&kind),
    };
    if domain.0 >= domain.1 {
        return Err(EpherError::Parse(format!(
            "graph domain must run low to high, got {:.3} .. {:.3}",
            domain.0, domain.1
        )));
    }
    Ok(CurveSpec { kind, domain, fill })
}

/// A parsed `from a to b` domain suffix (or none), paired with the body
/// text that preceded it.
type DomainSplit<'a> = Result<(&'a str, Option<(f64, f64)>), EpherError>;

/// Split a trailing `from a to b` off the body; the bounds evaluate as
/// expressions (so `2*pi` works) over the built-in constants.
fn split_domain(source: &str) -> DomainSplit<'_> {
    let Some(idx) = source.rfind(" from ") else {
        return Ok((source, None));
    };
    let (body, bounds) = source.split_at(idx);
    let bounds = bounds.trim_start_matches(" from ");
    let Some((a, b)) = bounds.split_once(" to ") else {
        return Err(EpherError::Parse(
            "expected `from a to b` after the expression".to_string(),
        ));
    };
    let fa = evaluate(a.trim())?;
    let fb = evaluate(b.trim())?;
    let (Value::Float(a), Value::Float(b)) = (fa, fb) else {
        return Err(EpherError::Type(
            "graph domain bounds must be numbers".to_string(),
        ));
    };
    Ok((body.trim(), Some((a, b))))
}

/// Split on a separator at paren depth zero (parametric commands use commas
/// while function calls may contain their own).
fn split_top_level(s: &str, sep: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            // Parentheses AND the list delimiters nest (ADR-0044):
            // `scatter({1, 2}, {3, 4})` splits on the outer commas
            // only, and parametric graphs keep working as before.
            '(' | '{' | '[' => depth += 1,
            ')' | '}' | ']' => depth = depth.saturating_sub(1),
            c if c == sep && depth == 0 => {
                parts.push(&s[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(&s[start..]);
    parts
}

/// Sample a parsed spec (ADR-0006: the core computes plot data).
pub fn sample_spec(spec: &CurveSpec, points: usize, env: &Env) -> Result<Vec<Sample>, EpherError> {
    let (a, b) = spec.domain;
    match &spec.kind {
        CurveKind::Cartesian(expr) => crate::sample(expr, a, b, points, env),
        CurveKind::Parametric { x, y } => crate::sample_parametric(x, y, a, b, points, env),
        CurveKind::Polar(expr) => crate::sample_polar(expr, a, b, points, env),
        CurveKind::Implicit(expr) => sample_implicit(expr, a, b, points, env),
    }
}

/// The difference `lhs - rhs` of an implicit equation (ADR-0048); the
/// sampler evaluates it on the grid and extracts the zero contour.
fn implicit_difference(expr: &Expression) -> Expression {
    match expr {
        Expression::Compare(CmpOp::Eq, l, r) => Expression::Sub(l.clone(), r.clone()),
        other => other.clone(),
    }
}

/// Sample an implicit relation over the square `[a, b] × [a, b]` with
/// marching squares (ADR-0048): the N×N grid evaluates the difference
/// at every vertex, each cell's corner signs pick the crossing case,
/// edge crossings interpolate linearly, and the ambiguous saddle cells
/// resolve with the cell-center average. The chained branches come out
/// as samples separated by non-finite pen-up markers, which the
/// renderers already split like vertical asymptotes.
pub fn sample_implicit(
    expr: &Expression,
    a: f64,
    b: f64,
    points: usize,
    env: &Env,
) -> Result<Vec<Sample>, EpherError> {
    let f = implicit_difference(expr);
    let n = points.max(2);
    let mut grid = vec![f64::NAN; (n + 1) * (n + 1)];
    let at = |i: usize, j: usize| -> f64 {
        let mut child = Env::new_child(env);
        let x = a + (b - a) * i as f64 / n as f64;
        let y = a + (b - a) * j as f64 / n as f64;
        child.set("x", Value::float(x));
        child.set("y", Value::float(y));
        match eval(&f, &child) {
            Ok(Value::Float(v)) if v.is_finite() => v,
            // Quantities evaluate to their SI value on the grid too.
            Ok(Value::Quantity { value, .. }) if value.is_finite() => value,
            _ => f64::NAN,
        }
    };
    for j in 0..=n {
        for i in 0..=n {
            grid[j * (n + 1) + i] = at(i, j);
        }
    }
    // Marching squares: walk the cells, collect interpolated segments.
    let mut segments: Vec<((f64, f64), (f64, f64))> = Vec::new();
    for j in 0..n {
        for i in 0..n {
            let v00 = grid[j * (n + 1) + i];
            let v10 = grid[j * (n + 1) + i + 1];
            let v01 = grid[(j + 1) * (n + 1) + i];
            let v11 = grid[(j + 1) * (n + 1) + i + 1];
            if [v00, v10, v01, v11].iter().any(|v| v.is_nan()) {
                continue;
            }
            let (x0, x1) = (
                a + (b - a) * i as f64 / n as f64,
                a + (b - a) * (i + 1) as f64 / n as f64,
            );
            let (y0, y1) = (
                a + (b - a) * j as f64 / n as f64,
                a + (b - a) * (j + 1) as f64 / n as f64,
            );
            let s00 = v00 > 0.0;
            let s10 = v10 > 0.0;
            let s01 = v01 > 0.0;
            let s11 = v11 > 0.0;
            let case = (s00 as u8) | ((s10 as u8) << 1) | ((s01 as u8) << 2) | ((s11 as u8) << 3);
            if case == 0 || case == 15 {
                continue;
            }
            // Edge crossing points (interpolated).
            let bottom = (x0 + (x1 - x0) * v00 / (v00 - v10), y0);
            let top = (x0 + (x1 - x0) * v01 / (v01 - v11), y1);
            let left = (x0, y0 + (y1 - y0) * v00 / (v00 - v01));
            let right = (x1, y0 + (y1 - y0) * v10 / (v10 - v11));
            // The 14 crossing cases, with the alternating diagonals
            // (6 and 9) resolved by the cell-center average: the center
            // decides which pair of positives connect through the
            // saddle, exactly the asymptotic decider.
            match case {
                1 | 14 => segments.push((bottom, left)),
                2 | 13 => segments.push((bottom, right)),
                3 | 12 => segments.push((left, right)),
                4 | 11 => segments.push((top, left)),
                7 | 8 => segments.push((top, right)),
                // Adjacent positives (the left or right column): the
                // contour crosses the cell from bottom to top once.
                5 | 10 => segments.push((bottom, top)),
                6 => {
                    let center = (v00 + v10 + v01 + v11) / 4.0;
                    if center > 0.0 {
                        segments.push((bottom, left));
                        segments.push((top, right));
                    } else {
                        segments.push((bottom, right));
                        segments.push((top, left));
                    }
                }
                9 => {
                    let center = (v00 + v10 + v01 + v11) / 4.0;
                    if center > 0.0 {
                        segments.push((bottom, right));
                        segments.push((top, left));
                    } else {
                        segments.push((bottom, left));
                        segments.push((top, right));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(chain_segments(&segments))
}

/// Chain interpolated segments into contour branches: greedy endpoint
/// matching within a cell's epsilon, each branch a run of samples, and
/// a non-finite pen-up sample between branches.
fn chain_segments(segments: &[((f64, f64), (f64, f64))]) -> Vec<Sample> {
    let mut out: Vec<Sample> = Vec::new();
    let mut used = vec![false; segments.len()];
    for start in 0..segments.len() {
        if used[start] {
            continue;
        }
        used[start] = true;
        let mut branch: Vec<Sample> = Vec::new();
        branch.push(Sample {
            x: segments[start].0 .0,
            y: segments[start].0 .1,
        });
        branch.push(Sample {
            x: segments[start].1 .0,
            y: segments[start].1 .1,
        });
        // extend forwards
        loop {
            let tail = branch.last().map(|s| (s.x, s.y)).expect("non-empty");
            let next = (0..segments.len()).find(|&i| {
                !used[i]
                    && (segments[i].0 == tail
                        || segments[i].1 == tail
                        || dist(segments[i].0, tail) < 1e-9
                        || dist(segments[i].1, tail) < 1e-9)
            });
            match next {
                Some(i) => {
                    used[i] = true;
                    let (p, q) = segments[i];
                    let next_point = if dist(p, tail) < dist(q, tail) { q } else { p };
                    branch.push(Sample {
                        x: next_point.0,
                        y: next_point.1,
                    });
                }
                None => break,
            }
        }
        if !out.is_empty() {
            out.push(Sample {
                x: f64::NAN,
                y: f64::NAN,
            });
        }
        out.extend(branch);
    }
    out
}

fn dist(a: (f64, f64), b: (f64, f64)) -> f64 {
    (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2)
}

/// The Cartesian expression, for curves that are `y = f(x)` (points-of-
/// interest analysis only applies to them).
pub fn cartesian_expr(kind: &CurveKind) -> Option<&Expression> {
    match kind {
        CurveKind::Cartesian(e) => Some(e),
        // Relations have no y = f(x) shape, so no points of interest.
        _ => None,
    }
}

/// Evaluate `expr` with `x` bound in a child environment (constant tables
/// and function tables stay visible; session bindings do not).
fn eval_at(expr: &Expression, x: f64, env: &Env) -> Option<f64> {
    let mut child = Env::new_child(env);
    child.set("x", Value::float(x));
    match eval(expr, &child) {
        Ok(Value::Float(v)) if v.is_finite() => Some(v),
        _ => None,
    }
}

/// A parsed `table` command: what to evaluate and over which x values,
/// plus an optional derivative column (ADR-0044). ADR-0054 adds the
/// `values <list>` column mode (rows at the list's x values; the
/// paste-a-data-column case) and the `exact`/`approx` display toggle.
/// Defaults match TI's table (start −5, end 5, 11 rows); `points` is
/// capped so a bad command can't demand unbounded work.
#[derive(Debug, Clone)]
pub struct TableSpec {
    pub expr: Expression,
    pub x_min: f64,
    pub x_max: f64,
    pub points: usize,
    /// The `derivative <expr>` column: evaluated numerically at each x
    /// with the 5-point stencil.
    pub derivative: Option<Expression>,
    /// The `values <list-expr>` column mode (ADR-0054): rows at these x
    /// values instead of the even grid. Mutually exclusive with
    /// `from a to b` and `points n`.
    pub values: Option<Expression>,
    /// The `exact`/`approx` suffix (ADR-0054): forces the cell display
    /// exact or decimal, overriding the session setting for this table.
    pub exact: Option<bool>,
}

/// Parse the text after `table `: `expr [from a to b] [points n]
/// [derivative expr]`. The language has no `from`/`to`/`points`/
/// `derivative` identifiers, so the keywords can never collide with the
/// expression.
pub fn parse_table_source(source: &str) -> Result<TableSpec, EpherError> {
    const DEFAULT_POINTS: usize = 11;
    const MAX_POINTS: usize = 1000;

    let source = source.trim();
    if source.is_empty() {
        return Err(EpherError::Parse("empty table command".to_string()));
    }
    // The `exact`/`approx` display suffix (ADR-0054) sits at the very
    // end, so strip it first; then `derivative <expr>`, then `points n`,
    // then `values <list>`, then the domain itself.
    let (rest, exact) = match source.strip_suffix(" exact") {
        Some(r) => (r.trim_end(), Some(true)),
        None => match source.strip_suffix(" approx") {
            Some(r) => (r.trim_end(), Some(false)),
            None => (source, None),
        },
    };
    let (rest, derivative) = match rest.rfind(" derivative ") {
        Some(idx) => {
            let (expr, d) = rest.split_at(idx);
            let d = d.trim_start_matches(" derivative ").trim();
            if d.is_empty() {
                return Err(EpherError::Parse(
                    "`derivative` needs an expression after it".to_string(),
                ));
            }
            (expr.trim(), Some(parse(d)?))
        }
        None => (rest, None),
    };
    let (rest, points) = match rest.rfind(" points ") {
        Some(idx) => {
            let (expr, n) = rest.split_at(idx);
            let n = n.trim_start_matches(" points ").trim();
            let n: usize = n
                .parse()
                .map_err(|_| EpherError::Parse(format!("`points {n}` needs a whole number")))?;
            if !(1..=MAX_POINTS).contains(&n) {
                return Err(EpherError::Parse(format!(
                    "`points` must be between 1 and {MAX_POINTS}"
                )));
            }
            (expr.trim(), Some(n))
        }
        None => (rest, None),
    };
    let (rest, values) = match rest.rfind(" values ") {
        Some(idx) => {
            let (expr, v) = rest.split_at(idx);
            let v = v.trim_start_matches(" values ").trim();
            if v.is_empty() {
                return Err(EpherError::Parse(
                    "`values` needs a list after it: `table x^2 values d`".to_string(),
                ));
            }
            (expr.trim(), Some(parse(v)?))
        }
        None => (rest, None),
    };
    let (body, domain) = split_domain(rest)?;
    let (x_min, x_max) = domain.unwrap_or((-5.0, 5.0));
    if let Some(v) = &values {
        if domain.is_some() {
            return Err(EpherError::Parse(
                "choose one x source: `from a to b` or `values <list>`, not both".to_string(),
            ));
        }
        if points.is_some() {
            return Err(EpherError::Parse(
                "choose one x source: `points n` or `values <list>`, not both".to_string(),
            ));
        }
        let _ = v;
    }
    let _ = &values;
    if x_min >= x_max {
        return Err(EpherError::Parse(format!(
            "table domain must run low to high, got {x_min:.3} .. {x_max:.3}"
        )));
    }
    Ok(TableSpec {
        expr: parse(body)?,
        x_min,
        x_max,
        points: points.unwrap_or(DEFAULT_POINTS),
        derivative,
        values,
        exact,
    })
}

/// A row of a table of values: x always present; y absent where the
/// expression has no value (TI-style blank rows). The derivative column
/// (ADR-0044) is present when the command named one.
/// One table row: the x, the expression's value (None where the
/// expression has no value), and the derivative column's value.
pub type TableRow = (f64, Option<f64>, Option<f64>);

pub fn table_rows(
    expr: &Expression,
    derivative: Option<&Expression>,
    x_min: f64,
    x_max: f64,
    points: usize,
    env: &Env,
) -> Vec<TableRow> {
    let mut out = Vec::new();
    for i in 0..points {
        let t = if points == 1 {
            0.0
        } else {
            i as f64 / (points - 1) as f64
        };
        let x = x_min + t * (x_max - x_min);
        let y = eval_at(expr, x, env);
        // The derivative column differentiates its expression at x with
        // the same 5-point stencil `derivative(expr, p)` uses (ADR-0044);
        // a constant expression differentiates to 0.
        let d = match derivative {
            Some(d) => crate::derivative_at(d, x, env).ok(),
            None => None,
        };
        out.push((x, y, d));
    }
    out
}

/// Table rows at the caller's x values (ADR-0054): the
/// `values <list>` column mode, where the x column comes from a data
/// list instead of an even grid. Capped at the same 1000 rows the
/// grid mode allows.
pub fn table_rows_at(
    expr: &Expression,
    derivative: Option<&Expression>,
    xs: &[f64],
    env: &Env,
) -> Result<Vec<TableRow>, EpherError> {
    if xs.is_empty() {
        return Err(EpherError::Type("the values list needs at least one x".to_string()));
    }
    if xs.len() > 1000 {
        return Err(EpherError::Type(format!(
            "the values list is capped at 1000 x values, got {}",
            xs.len()
        )));
    }
    Ok(xs
        .iter()
        .map(|&x| {
            let y = eval_at(expr, x, env);
            let d = match derivative {
                Some(d) => crate::derivative_at(d, x, env).ok(),
                None => None,
            };
            (x, y, d)
        })
        .collect())
}

/// What kind of notable point an [`InterestPoint`] marks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterestKind {
    Root,
    Intersection,
    Maximum,
    Minimum,
}

/// A point of interest on a plot: roots and extrema of a curve, or the
/// intersection of two curves (ADR-0014).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterestPoint {
    pub kind: InterestKind,
    pub x: f64,
    pub y: f64,
    /// The curve that carries this point (the higher index of the two
    /// for an intersection): the web's per-curve legend visibility
    /// filters a curve's points with it.
    pub curve: usize,
}

impl InterestPoint {
    pub fn coords(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}

/// Find roots, intersections, and extrema across the plotted curves.
/// Cartesian curves only (parametric/polar analysis is a documented
/// deferral); intersections need overlapping x domains.
pub fn analyze(curves: &[SampledCurve], env: &Env) -> Vec<InterestPoint> {
    let mut out = Vec::new();
    for (i, curve) in curves.iter().enumerate() {
        let Some(expr) = cartesian_expr(&curve.kind) else {
            continue;
        };
        roots_and_extrema(expr, &curve.samples, env, i, &mut out);
        for other in curves.iter().take(i) {
            if let Some(other_expr) = cartesian_expr(&other.kind) {
                intersections(expr, other_expr, curve, other, env, i, &mut out);
            }
        }
    }
    out.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal));
    out.dedup_by(|a, b| (a.x - b.x).abs() < 1e-6 && (a.y - b.y).abs() < 1e-6);
    out
}

/// Roots (sign changes over the sampled data, refined by bisection) and
/// local extrema (sampled turning points, refined by golden-section search).
fn roots_and_extrema(
    expr: &Expression,
    samples: &[Sample],
    env: &Env,
    curve: usize,
    out: &mut Vec<InterestPoint>,
) {
    let finite: Vec<&Sample> = samples.iter().filter(|s| s.y.is_finite()).collect();
    // Roots.
    for w in finite.windows(2) {
        let (a, b) = (w[0], w[1]);
        if a.y == 0.0 {
            out.push(InterestPoint {
                kind: InterestKind::Root,
                x: a.x,
                y: 0.0,
                curve,
            });
        } else if a.y * b.y < 0.0 {
            if let Some(x) = bisect(expr, a.x, b.x, 0.0, env) {
                out.push(InterestPoint {
                    kind: InterestKind::Root,
                    x,
                    y: 0.0,
                    curve,
                });
            }
        }
    }
    if let Some(last) = finite.last() {
        if last.y == 0.0 {
            out.push(InterestPoint {
                kind: InterestKind::Root,
                x: last.x,
                y: 0.0,
                curve,
            });
        }
    }
    // Extrema: a sample strictly above (or below) at least one neighbor and
    // no lower (higher) than the other — catches symmetric peaks where the
    // two neighbors tie, but never fires on a flat line (both sides strict).
    for w in finite.windows(3) {
        let (l, m, r) = (w[0], w[1], w[2]);
        if (m.y > l.y && m.y >= r.y) || (m.y >= l.y && m.y > r.y) {
            if let Some(x) = golden_extremum(expr, l.x, r.x, true, env) {
                out.push(InterestPoint {
                    kind: InterestKind::Maximum,
                    x,
                    y: eval_at(expr, x, env).unwrap_or(m.y),
                    curve,
                });
            }
        } else if (m.y < l.y && m.y <= r.y) || (m.y <= l.y && m.y < r.y) {
            if let Some(x) = golden_extremum(expr, l.x, r.x, false, env) {
                out.push(InterestPoint {
                    kind: InterestKind::Minimum,
                    x,
                    y: eval_at(expr, x, env).unwrap_or(m.y),
                    curve,
                });
            }
        }
    }
}

/// Intersections of two Cartesian curves over their overlapping domain:
/// sign changes of `f(x) - g(x)` on a shared grid, refined by bisection.
fn intersections(
    f: &Expression,
    g: &Expression,
    a: &SampledCurve,
    b: &SampledCurve,
    env: &Env,
    curve: usize,
    out: &mut Vec<InterestPoint>,
) {
    let lo = a.domain.0.max(b.domain.0);
    let hi = a.domain.1.min(b.domain.1);
    if lo >= hi {
        return;
    }
    const POINTS: usize = 160;
    let mut prev: Option<(f64, f64)> = None;
    for i in 0..POINTS {
        let x = lo + (hi - lo) * i as f64 / (POINTS - 1) as f64;
        let (Some(fx), Some(gx)) = (eval_at(f, x, env), eval_at(g, x, env)) else {
            prev = None;
            continue;
        };
        let d = fx - gx;
        if let Some((px, pd)) = prev {
            if d == 0.0 {
                out.push(InterestPoint {
                    kind: InterestKind::Intersection,
                    x,
                    y: fx,
                    curve,
                });
            } else if pd * d < 0.0 {
                if let Some(x) = bisect_diff(f, g, px, x, env) {
                    out.push(InterestPoint {
                        kind: InterestKind::Intersection,
                        x,
                        y: eval_at(f, x, env).unwrap_or(fx),
                        curve,
                    });
                }
            }
        }
        prev = Some((x, d));
    }
}

/// Bisect a sign change of `f(x) - target` on `(a, b)`.
fn bisect(expr: &Expression, mut a: f64, mut b: f64, target: f64, env: &Env) -> Option<f64> {
    let mut fa = eval_at(expr, a, env)? - target;
    for _ in 0..64 {
        let m = 0.5 * (a + b);
        let fm = eval_at(expr, m, env)? - target;
        if fa * fm <= 0.0 {
            b = m;
        } else {
            a = m;
            fa = fm;
        }
    }
    Some(0.5 * (a + b))
}

/// Bisect a sign change of `f(x) - g(x)` on `(a, b)`.
fn bisect_diff(f: &Expression, g: &Expression, mut a: f64, mut b: f64, env: &Env) -> Option<f64> {
    let mut da = eval_at(f, a, env)? - eval_at(g, a, env)?;
    for _ in 0..64 {
        let m = 0.5 * (a + b);
        let dm = eval_at(f, m, env)? - eval_at(g, m, env)?;
        if da * dm <= 0.0 {
            b = m;
        } else {
            a = m;
            da = dm;
        }
    }
    Some(0.5 * (a + b))
}

/// Golden-section search for a local extremum of `f` on `[a, b]`.
fn golden_extremum(expr: &Expression, a: f64, b: f64, maximum: bool, env: &Env) -> Option<f64> {
    let phi = 0.618_033_988_749_894_9;
    let (mut lo, mut hi) = (a, b);
    let mut c = hi - phi * (hi - lo);
    let mut d = lo + phi * (hi - lo);
    let better = |v: f64, w: f64| if maximum { v > w } else { v < w };
    for _ in 0..64 {
        let (fc, fd) = (eval_at(expr, c, env)?, eval_at(expr, d, env)?);
        if better(fc, fd) {
            hi = d;
            d = c;
            c = hi - phi * (hi - lo);
        } else {
            lo = c;
            c = d;
            d = lo + phi * (hi - lo);
        }
    }
    Some(0.5 * (lo + hi))
}

/// A "nice" tick step for a value span: 1, 2, or 5 × 10^k, aiming for at
/// most `target` intervals (both renderers grid to these steps).
pub fn nice_step(span: f64, target: usize) -> f64 {
    if !span.is_finite() || span <= 0.0 || target == 0 {
        return 1.0;
    }
    let raw = span / target as f64;
    let k = raw.log10().floor();
    let base = 10f64.powf(k);
    for m in [1.0, 2.0, 5.0, 10.0] {
        if m * base >= raw {
            return m * base;
        }
    }
    10f64.powf(k + 1.0)
}

/// Every variable name referenced anywhere in an expression (sliders bind
/// the constants among these — ADR-0014).
pub fn free_names(expr: &Expression, out: &mut BTreeSet<String>) {
    match expr {
        Expression::Literal(_) => {}
        Expression::StrLit(_) => {}
        Expression::Var(name) => {
            out.insert(name.clone());
        }
        Expression::Call(_, args) => {
            for a in args {
                free_names(a, out);
            }
        }
        Expression::Neg(e) | Expression::Factorial(e) | Expression::Not(e) => free_names(e, out),
        Expression::Matrix(rows) => {
            for row in rows {
                for item in row {
                    free_names(item, out);
                }
            }
        }
        Expression::BitNot(e) => free_names(e, out),
        Expression::Unit(inner, _, _, _) | Expression::In(inner, _, _, _) => free_names(inner, out),
        Expression::BitAnd(a, b)
        | Expression::BitOr(a, b)
        | Expression::BitXor(a, b)
        | Expression::ShiftLeft(a, b)
        | Expression::ShiftRight(a, b) => {
            free_names(a, out);
            free_names(b, out);
        }
        Expression::Add(a, b)
        | Expression::Sub(a, b)
        | Expression::Mul(a, b)
        | Expression::Div(a, b)
        | Expression::Pow(a, b)
        | Expression::And(a, b)
        | Expression::Or(a, b) => {
            free_names(a, out);
            free_names(b, out);
        }
        Expression::Compare(_, a, b) => {
            free_names(a, out);
            free_names(b, out);
        }
        Expression::If(a, b, c) => {
            free_names(a, out);
            free_names(b, out);
            free_names(c, out);
        }
        Expression::List(items) => {
            for item in items {
                free_names(item, out);
            }
        }
        Expression::Index(list, index) => {
            free_names(list, out);
            free_names(index, out);
        }
    }
}

// ===== data plots (ADR-0044) =====

/// The kind of data plot a `graph` command requested.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataPlotKind {
    Scatter,
    Histogram,
    BoxPlot,
}

/// The regression models a scatter can draw over its points
/// (ADR-0054): the same family the `*reg` builtins fit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScatterFit {
    Linreg,
    Quadreg,
    Expreg,
    Powreg,
    Logreg,
}

impl ScatterFit {
    /// The model named by the optional third `scatter` argument.
    pub fn named(name: &str) -> Option<Self> {
        match name {
            "linreg" => Some(Self::Linreg),
            "quadreg" => Some(Self::Quadreg),
            "expreg" => Some(Self::Expreg),
            "powreg" => Some(Self::Powreg),
            "logreg" => Some(Self::Logreg),
            _ => None,
        }
    }

    fn kind(self) -> crate::FitKind {
        match self {
            Self::Linreg => crate::FitKind::Linear,
            Self::Quadreg => crate::FitKind::Quadratic,
            Self::Expreg => crate::FitKind::Exponential,
            Self::Powreg => crate::FitKind::Power,
            Self::Logreg => crate::FitKind::Logarithmic,
        }
    }
}

/// The least-squares model of a scatter plot with its reported r
/// (ADR-0044 for the line, ADR-0054 for the family).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fit {
    pub model: ScatterFit,
    pub fit: crate::Fit,
}

/// The computed picture of a data plot: what the frontends render
/// (ADR-0006 seam — the core computes the primitives, the web draws
/// SVG and the TUI draws ASCII from this one struct).
#[derive(Debug, Clone, PartialEq)]
pub struct DataPlot {
    pub kind: DataPlotKind,
    /// What the user typed after `graph` (the accessible caption/legend text).
    pub source: String,
    /// Scatter: the points.
    pub points: Vec<(f64, f64)>,
    /// Scatter: the fitted model when there are enough points.
    pub fit: Option<Fit>,
    /// Histogram: one (lo, hi, count) per bin.
    pub bins: Vec<(f64, f64, f64)>,
    /// Boxplot: min, q1, median, q3, max.
    pub boxplot: Option<[f64; 5]>,
}

/// Evaluate one data-plot argument: an expression that must produce a
/// list of floats.
fn eval_list(expr: &Expression, env: &Env) -> Result<Vec<f64>, EpherError> {
    let v = crate::eval(expr, env)?;
    let Value::List(items) = &v else {
        return Err(EpherError::Type(format!("data plots need a list, got {v}")));
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        match item {
            Value::Float(x) => out.push(*x),
            other => {
                return Err(EpherError::Type(format!(
                    "data plots need numbers, got {other:?}"
                )))
            }
        }
    }
    Ok(out)
}

/// Does a `graph` source name a data plot (ADR-0044)? The command may
/// spell the keyword with or without a space before the paren:
/// `scatter(x, y)` and `scatter (x, y)` are the same command. Frontends
/// dispatch on this before the curve grammar.
pub fn is_data_plot_source(source: &str) -> bool {
    let body = source.trim();
    ["scatter", "histogram", "boxplot"].iter().any(|kw| {
        body.strip_prefix(kw)
            .map(|rest| rest.is_empty() || rest.starts_with(' ') || rest.starts_with('('))
            .unwrap_or(false)
    })
}

/// Parse and compute a data plot from the text after `graph `
/// (ADR-0044): `scatter <xs>, <ys>`, `histogram <data>[, <bins>]`,
/// `boxplot <data>`. The data arguments are expressions evaluated
/// against the session, so variables and literals both work:
/// `graph scatter({1, 2, 3}, {4, 5, 6})`, `graph histogram(d)`.
/// The `from a to b` domain keywords do not apply (the window fits
/// the data); a domain clause is an error.
pub fn sample_data_plot(source: &str, env: &Env) -> Result<DataPlot, EpherError> {
    let source = source.trim();
    let (body, domain) = split_domain(source)?;
    if domain.is_some() {
        return Err(EpherError::Parse(
            "data plots fit their window to the data: `from a to b` does not apply".to_string(),
        ));
    }
    let keyword_rest = |kw: &str| -> Option<&str> {
        let rest = body.strip_prefix(kw)?;
        if rest.is_empty() || rest.starts_with(' ') || rest.starts_with('(') {
            // The keyword may be spelled `scatter(...)` or `scatter (...)`:
            // peel the call parens (the closing one too).
            let inner = rest.trim_start_matches(' ').trim_start_matches('(');
            Some(inner.strip_suffix(')').unwrap_or(inner).trim())
        } else {
            None
        }
    };
    if let Some(rest) = keyword_rest("scatter") {
        let parts: Vec<&str> = split_top_level(rest, ',')
            .into_iter()
            .map(|s| s.trim())
            .collect();
        // The optional third argument names the fit model (ADR-0054):
        // `scatter xs, ys` fits the line; `scatter xs, ys, quadreg`
        // (or expreg/powreg/logreg) fits the family. The model word is
        // never an expression, so it is matched before parsing.
        let (data_parts, model) = match parts.as_slice() {
            [xs, ys] => ([*xs, *ys], ScatterFit::Linreg),
            [xs, ys, model] => {
                let Some(m) = ScatterFit::named(model) else {
                    return Err(EpherError::Parse(
                        "the scatter model is one of linreg, quadreg, expreg, powreg, logreg"
                            .to_string(),
                    ));
                };
                ([*xs, *ys], m)
            }
            _ => {
                return Err(EpherError::Parse(
                    "scatter needs two lists: `scatter <xs>, <ys>`".to_string(),
                ))
            }
        };
        let [xs, ys] = data_parts;
        let xs = eval_list(&parse(xs)?, env)?;
        let ys = eval_list(&parse(ys)?, env)?;
        if xs.len() != ys.len() {
            return Err(EpherError::Type(format!(
                "scatter lists have different lengths: {} and {}",
                xs.len(),
                ys.len()
            )));
        }
        if xs.is_empty() {
            return Err(EpherError::Type(
                "scatter needs at least one point".to_string(),
            ));
        }
        let points: Vec<(f64, f64)> = xs.into_iter().zip(ys).collect();
        // The minimum points per model mirror the `*reg` builtins: two
        // for the two-parameter models, three for the quadratic. A
        // requested model the data cannot support is an error; the
        // default line simply draws unfitted when a single point is
        // plotted (the ADR-0044 behavior).
        let min_points = match model {
            ScatterFit::Quadreg => 3,
            _ => 2,
        };
        let fit = if points.len() >= min_points {
            let (px, py): (Vec<f64>, Vec<f64>) = points.iter().copied().unzip();
            crate::fit_regression(model.kind(), &px, &py)
                .map(|f| Some(Fit { model, fit: f }))?
        } else if model != ScatterFit::Linreg {
            return Err(EpherError::Type(format!(
                "{} needs at least {} points, got {}",
                match model {
                    ScatterFit::Linreg => "linreg",
                    ScatterFit::Quadreg => "quadreg",
                    ScatterFit::Expreg => "expreg",
                    ScatterFit::Powreg => "powreg",
                    ScatterFit::Logreg => "logreg",
                },
                min_points,
                points.len()
            )));
        } else {
            None
        };
        return Ok(DataPlot {
            kind: DataPlotKind::Scatter,
            source: source.to_string(),
            points,
            fit,
            bins: Vec::new(),
            boxplot: None,
        });
    }
    if let Some(rest) = keyword_rest("histogram") {
        let parts: Vec<&str> = split_top_level(rest, ',')
            .into_iter()
            .map(|s| s.trim())
            .collect();
        let data = eval_list(&parse(parts[0])?, env)?;
        if data.is_empty() {
            return Err(EpherError::Type("histogram needs data".to_string()));
        }
        let bins = match parts.len() {
            1 => (data.len() as f64).log2().ceil() as usize + 1,
            2 => {
                let n = crate::eval(&parse(parts[1])?, env)?;
                let Value::Float(n) = n else {
                    return Err(EpherError::Type(
                        "the bin count must be a whole number".to_string(),
                    ));
                };
                if !(1.0..=50.0).contains(&n) || n.fract() != 0.0 {
                    return Err(EpherError::Type(
                        "the bin count must be a whole number between 1 and 50".to_string(),
                    ));
                }
                n as usize
            }
            _ => return Err(EpherError::Parse(
                "histogram takes the data and an optional bin count: `histogram <data>[, <bins>]`"
                    .to_string(),
            )),
        };
        let (mut lo, mut hi) = data
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| {
                (lo.min(*x), hi.max(*x))
            });
        if hi == lo {
            lo -= 0.5;
            hi += 0.5;
        }
        let span = hi - lo;
        let mut counts = vec![0.0; bins];
        for x in &data {
            let mut i = ((x - lo) / span * bins as f64).floor() as usize;
            if i >= bins {
                i = bins - 1;
            }
            counts[i] += 1.0;
        }
        let bin_edges: Vec<(f64, f64, f64)> = counts
            .into_iter()
            .enumerate()
            .map(|(i, c)| {
                (
                    lo + i as f64 * span / bins as f64,
                    lo + (i + 1) as f64 * span / bins as f64,
                    c,
                )
            })
            .collect();
        return Ok(DataPlot {
            kind: DataPlotKind::Histogram,
            source: source.to_string(),
            points: Vec::new(),
            fit: None,
            bins: bin_edges,
            boxplot: None,
        });
    }
    if let Some(rest) = keyword_rest("boxplot") {
        if split_top_level(rest, ',').len() != 1 {
            return Err(EpherError::Parse(
                "boxplot takes one list: `boxplot <data>`".to_string(),
            ));
        }
        let data = eval_list(&parse(rest)?, env)?;
        if data.is_empty() {
            return Err(EpherError::Type("boxplot needs data".to_string()));
        }
        let mut sorted = data.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).expect("floats are comparable"));
        let five = [
            sorted[0],
            crate::quartile_sorted(&sorted, 1),
            crate::quartile_sorted(&sorted, 2),
            crate::quartile_sorted(&sorted, 3),
            *sorted.last().expect("non-empty"),
        ];
        return Ok(DataPlot {
            kind: DataPlotKind::BoxPlot,
            source: source.to_string(),
            points: Vec::new(),
            fit: None,
            bins: Vec::new(),
            boxplot: Some(five),
        });
    }
    Err(EpherError::Parse(
        "data plots: `scatter <xs>, <ys>`, `histogram <data>[, <bins>]`, `boxplot <data>`"
            .to_string(),
    ))
}

/// The plot window a data plot fits: (x_min, x_max, y_min, y_max).
/// Histogram y is the count; boxplot y is the fixed band 0..1.
pub fn data_ranges(data: &DataPlot) -> (f64, f64, f64, f64) {
    match data.kind {
        DataPlotKind::Scatter => {
            let (mut x0, mut x1, mut y0, mut y1) = (
                f64::INFINITY,
                f64::NEG_INFINITY,
                f64::INFINITY,
                f64::NEG_INFINITY,
            );
            for (x, y) in &data.points {
                x0 = x0.min(*x);
                x1 = x1.max(*x);
                y0 = y0.min(*y);
                y1 = y1.max(*y);
            }
            if let Some(f) = data.fit {
                const FIT_SAMPLES: usize = 24;
                for k in 0..=FIT_SAMPLES {
                    let x = x0 + (x1 - x0) * k as f64 / FIT_SAMPLES as f64;
                    let y = f.fit.eval(x);
                    if y.is_finite() {
                        y0 = y0.min(y);
                        y1 = y1.max(y);
                    }
                }
            }
            (x0, x1, y0, y1)
        }
        DataPlotKind::Histogram => {
            let (mut lo, mut hi, mut max) = (f64::INFINITY, f64::NEG_INFINITY, 0.0f64);
            for (a, b, c) in &data.bins {
                lo = lo.min(*a);
                hi = hi.max(*b);
                max = max.max(*c);
            }
            (lo, hi, 0.0, max)
        }
        DataPlotKind::BoxPlot => {
            let b = data
                .boxplot
                .expect("boxplot always carries its five numbers");
            (b[0], b[4], -0.5, 1.5)
        }
    }
}

// ===== 3D surfaces (ADR-0015) =====

/// A sampled `z = f(x, y)` surface over a square domain. `zs[row][col]`
/// holds z at `(xs[col], ys[row])`; undefined cells are NaN.
#[derive(Debug, Clone)]
pub struct Surface {
    pub source: String,
    pub domain: (f64, f64),
    pub xs: Vec<f64>,
    pub ys: Vec<f64>,
    pub zs: Vec<Vec<f64>>,
}

/// The camera pose for a 3D plot: yaw around the vertical (z) axis, then
/// pitch around the rotated x axis. The projection is orthographic (the
/// ADR-0015 amendment), so `camera` carries no distance: it is the zoom
/// state, and a render window shrinks in proportion to it (see
/// [`zoom_window`]). All angles in radians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View3D {
    pub yaw: f64,
    pub pitch: f64,
    pub camera: f64,
}

impl Default for View3D {
    fn default() -> Self {
        Self {
            yaw: 0.8,
            pitch: 0.6,
            // The zoom reference: renders scale their window by
            // `camera / 30.0`, so this value shows the whole scene.
            camera: 30.0,
        }
    }
}

impl View3D {
    /// Pitch is clamped so the view never flips over the poles.
    pub fn with_pitch(&self, pitch: f64) -> Self {
        Self {
            pitch: pitch.clamp(-1.4, 1.4),
            ..*self
        }
    }

    pub fn with_yaw(&self, yaw: f64) -> Self {
        Self { yaw, ..*self }
    }

    /// Set the zoom state (mouse-wheel zoom, ADR-0034): smaller values
    /// shrink the render window, so the scene grows. Relative positions
    /// never change - the projection is affine. The floor only guards the
    /// projection against degenerating (a zero camera), not the user:
    /// wheel and pinch zoom in as far as they want (ADR-0038).
    pub fn with_camera(&self, camera: f64) -> Self {
        Self {
            camera: camera.max(0.01),
            ..*self
        }
    }

    /// Apply the fine-control sliders' offsets (ADR-0031), each −1..1
    /// with 0 = this pose unchanged: horizontal adds `h × π` to the yaw;
    /// vertical adds `v × 0.8` to the pitch — the full range stays live
    /// at the default pose (pitch 0.6 + 0.8 = 1.4, exactly the clamp) —
    /// and zoom scales the render window by `10^(-2z)` (ADR-0038): 0 is
    /// the default window, +1 shrinks it 100× (a single object fills the
    /// pane), −1 grows it 100× (every object fits).
    pub fn with_offsets(&self, h: f64, v: f64, z: f64) -> Self {
        Self {
            yaw: self.yaw + h * std::f64::consts::PI,
            pitch: (self.pitch + v * 0.8).clamp(-1.4, 1.4),
            camera: self.camera * 10f64.powf(-2.0 * z),
        }
    }

    /// Compose an animated spin (ADR-0032): `yaw`/`pitch` are the
    /// accumulated rotation phase from the fine-control sliders' continuous
    /// spin, `zoom` the static zoom offset. Unlike [`Self::with_offsets`]
    /// the pitch is NOT clamped — a vertical spin is a full revolution
    /// around the horizontal axis; the sine/cosine projection keeps the
    /// pose continuous through the poles.
    pub fn with_spin_phase(&self, yaw: f64, pitch: f64, zoom: f64) -> Self {
        Self {
            yaw: self.yaw + yaw,
            pitch: self.pitch + pitch,
            camera: self.camera * 10f64.powf(-2.0 * zoom),
        }
    }
}

/// A mesh segment in screen space with its mean view depth — larger depth
/// is nearer to the camera, and renderers draw far-to-near (painter's
/// algorithm) so nearer lines overpaint farther ones.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment3D {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub depth: f64,
}

/// Parse a `graph3d` body: an expression in x and y plus an optional
/// `from a to b` square domain (default −5..5).
pub fn parse_surface_source(source: &str) -> Result<(Expression, (f64, f64)), EpherError> {
    let source = source.trim();
    if source.is_empty() {
        return Err(EpherError::Parse("empty graph3d command".to_string()));
    }
    let (body, domain) = split_domain(source)?;
    let expr = parse(body.trim())?;
    Ok((expr, domain.unwrap_or((-5.0, 5.0))))
}

/// Evaluate `expr` with `x` and `y` bound in a child environment (constant
/// tables and function tables stay visible; session bindings do not).
fn eval_at_xy(expr: &Expression, x: f64, y: f64, env: &Env) -> Result<Option<f64>, EpherError> {
    let mut child = Env::new_child(env);
    child.set("x", Value::float(x));
    child.set("y", Value::float(y));
    match eval(expr, &child) {
        Ok(Value::Float(v)) if v.is_finite() => Ok(Some(v)),
        // A non-finite value (e.g. sqrt(-1)) is a hole in the mesh, not
        // an evaluation error: it must not be reported as the cause.
        Ok(_) => Ok(None),
        Err(e) => Err(e),
    }
}

/// Sample `z = f(x, y)` over a square `grid × grid` mesh.
pub fn sample_surface(source: &str, grid: usize, env: &Env) -> Result<Surface, EpherError> {
    let (expr, (a, b)) = parse_surface_source(source)?;
    if a >= b {
        return Err(EpherError::Parse(
            "surface domain needs two bounds with the first smaller: `from a to b`".to_string(),
        ));
    }
    let grid = grid.clamp(4, 96);
    let mut xs = Vec::with_capacity(grid + 1);
    let mut ys = Vec::with_capacity(grid + 1);
    for i in 0..=grid {
        let v = a + (b - a) * i as f64 / grid as f64;
        xs.push(v);
        ys.push(v);
    }
    let mut zs = vec![vec![f64::NAN; grid + 1]; grid + 1];
    let mut first_err: Option<EpherError> = None;
    for (r, &y) in ys.iter().enumerate() {
        for (c, &x) in xs.iter().enumerate() {
            match eval_at_xy(&expr, x, y, env) {
                Ok(Some(v)) => zs[r][c] = v,
                Ok(None) => {}
                Err(e) => {
                    if first_err.is_none() {
                        first_err = Some(e);
                    }
                }
            }
        }
    }
    if zs.iter().flatten().all(|z| z.is_nan()) {
        // When every cell failed, say why: an undefined name, a division
        // by zero, or — if the cells are holes rather than errors — the
        // generic no-finite-values message.
        return match first_err {
            Some(e) => Err(e),
            None => Err(EpherError::Domain(format!(
                "no finite values for the surface: {source}"
            ))),
        };
    }
    Ok(Surface {
        source: source.trim().to_string(),
        domain: (a, b),
        xs,
        ys,
        zs,
    })
}

/// Project one world point: yaw around z, pitch around the rotated x axis,
/// then an orthographic drop of the view depth. Returns (screen x,
/// screen y, view depth); screen y grows upward and depth grows toward
/// the viewer. The projection is affine: zoom (via [`zoom_window`]) and
/// orbit change scale and pose, never relative positions.
/// Rotate a world point into camera space (x right, y up, z toward the
/// viewer): yaw around the z axis, then pitch around the rotated x axis.
fn to_camera(x: f64, y: f64, z: f64, view: &View3D) -> (f64, f64, f64) {
    let (sy, cy) = view.yaw.sin_cos();
    let (sp, cp) = view.pitch.sin_cos();
    let xr = x * cy - y * sy;
    let yr = x * sy + y * cy;
    let yp = yr * cp - z * sp;
    let zp = yr * sp + z * cp;
    (xr, yp, zp)
}

/// Orthographic screen mapping: drop the view depth, keep the rotated
/// coordinates. Affine, so no point ever explodes and relative positions
/// are exact at every zoom.
fn to_screen(xr: f64, yp: f64, _zp: f64, _view: &View3D) -> (f64, f64) {
    (xr, -yp)
}

/// Project one world point; raw. Callers that draw whole segments or
/// meshes use [`project_clipped`] so undefined (NaN) cells split runs.
pub fn project_point(x: f64, y: f64, z: f64, view: &View3D) -> (f64, f64, f64) {
    let (xr, yp, zp) = to_camera(x, y, z, view);
    let (sx, sy) = to_screen(xr, yp, zp, view);
    (sx, sy, zp)
}

/// Project a world-space segment to screen coordinates. Returns the
/// endpoints and their camera-space z values
/// (`(sx1, sy1, zp1, sx2, sy2, zp2)`), or None when either end touches an
/// undefined (NaN) cell. The projection is orthographic, so every finite
/// segment projects - there is no camera plane to clip against.
pub fn project_clipped(
    x1: f64,
    y1: f64,
    z1: f64,
    x2: f64,
    y2: f64,
    z2: f64,
    view: &View3D,
) -> Option<(f64, f64, f64, f64, f64, f64)> {
    if !z1.is_finite() || !z2.is_finite() {
        return None;
    }
    let (xr1, yp1, zp1) = to_camera(x1, y1, z1, view);
    let (xr2, yp2, zp2) = to_camera(x2, y2, z2, view);
    let (sx1, sy1) = to_screen(xr1, yp1, zp1, view);
    let (sx2, sy2) = to_screen(xr2, yp2, zp2, view);
    if !sx1.is_finite() || !sy1.is_finite() || !sx2.is_finite() || !sy2.is_finite() {
        return None;
    }
    Some((sx1, sy1, zp1, sx2, sy2, zp2))
}

/// Project a surface's mesh to screen segments, far-to-near (painter's
/// algorithm). Rows and columns both contribute, so the wireframe reads as
/// a mesh; segments touching undefined cells are dropped.
pub fn project_surface(surface: &Surface, view: &View3D) -> Vec<Segment3D> {
    let n = surface.xs.len();
    let mut segments = Vec::with_capacity(n * (n - 1) * 2);
    let mut push = |r1: usize, c1: usize, r2: usize, c2: usize| {
        if let Some((x1, y1, zp1, x2, y2, zp2)) = project_clipped(
            surface.xs[c1],
            surface.ys[r1],
            surface.zs[r1][c1],
            surface.xs[c2],
            surface.ys[r2],
            surface.zs[r2][c2],
            view,
        ) {
            segments.push(Segment3D {
                x1,
                y1,
                x2,
                y2,
                depth: (zp1 + zp2) / 2.0,
            });
        }
    };
    for r in 0..n {
        for c in 0..n - 1 {
            push(r, c, r, c + 1);
        }
    }
    for c in 0..n {
        for r in 0..n - 1 {
            push(r, c, r + 1, c);
        }
    }
    segments.sort_by(|a, b| b.depth.total_cmp(&a.depth));
    segments
}

/// The orientation aids around a surface: the ground square (the domain at
/// z = 0), the three axes through the origin within the plotted bounds, and
/// the vertical extent of the surface — as projected segments.
pub fn surface_frame(surface: &Surface, view: &View3D) -> Vec<Segment3D> {
    let (a, b) = surface.domain;
    let mut frame = Vec::with_capacity(8);
    let mut edge = |x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64| {
        if let Some((sx1, sy1, zp1, sx2, sy2, zp2)) = project_clipped(x1, y1, z1, x2, y2, z2, view)
        {
            frame.push(Segment3D {
                x1: sx1,
                y1: sy1,
                x2: sx2,
                y2: sy2,
                depth: (zp1 + zp2) / 2.0,
            });
        }
    };
    // Ground square at z = 0.
    edge(a, a, 0.0, b, a, 0.0);
    edge(b, a, 0.0, b, b, 0.0);
    edge(b, b, 0.0, a, b, 0.0);
    edge(a, b, 0.0, a, a, 0.0);
    // Axes through the origin, within the plotted bounds.
    edge(a, 0.0, 0.0, b, 0.0, 0.0);
    edge(0.0, a, 0.0, 0.0, b, 0.0);
    edge(0.0, 0.0, a, 0.0, 0.0, b);
    frame
}

/// A projected mesh polyline (one row or one column of the surface grid)
/// with its mean view depth. Undefined cells split a polyline into runs.
#[derive(Debug, Clone, PartialEq)]
pub struct Polyline3D {
    pub points: Vec<(f64, f64)>,
    pub depth: f64,
}

/// Project a surface's mesh as whole grid lines (rows and columns), which
/// SVG renderers draw as few elements with per-line depth shading. Rows and
/// columns are interleaved far-to-near by their mean depth (painter's
/// algorithm at line granularity).
pub fn project_mesh(surface: &Surface, view: &View3D) -> Vec<Polyline3D> {
    let n = surface.xs.len();
    let mut lines = Vec::new();
    // Rows: fixed y, varying x.
    for r in 0..n {
        let y = surface.ys[r];
        lines.extend(line_runs(&surface.xs, &vec![y; n], &surface.zs[r], view));
    }
    // Columns: fixed x, varying y.
    for c in 0..n {
        let x = surface.xs[c];
        let zs: Vec<f64> = surface.zs.iter().map(|row| row[c]).collect();
        lines.extend(line_runs(&vec![x; n], &surface.ys, &zs, view));
    }
    lines.sort_by(|a, b| b.depth.total_cmp(&a.depth));
    lines
}

/// Project one grid line (a row or a column) into visible runs: undefined
/// cells split runs, and segments crossing the near plane are clipped so
/// coordinates stay bounded.
fn line_runs(cx: &[f64], cy: &[f64], cz: &[f64], view: &View3D) -> Vec<Polyline3D> {
    let mut out = Vec::new();
    let mut run: Vec<(f64, f64, f64)> = Vec::new(); // (sx, sy, zp)
    let flush = |out: &mut Vec<Polyline3D>, run: &mut Vec<(f64, f64, f64)>| {
        if !run.is_empty() {
            let depth = run.iter().map(|p| p.2).sum::<f64>() / run.len() as f64;
            out.push(Polyline3D {
                depth,
                points: std::mem::take(run)
                    .into_iter()
                    .map(|(x, y, _)| (x, y))
                    .collect(),
            });
        }
    };
    let mut started = false;
    for i in 0..cx.len() {
        if !cz[i].is_finite() {
            flush(&mut out, &mut run);
            started = false;
            continue;
        }
        if !started {
            let (xr, yp, zp) = to_camera(cx[i], cy[i], cz[i], view);
            let (sx, sy) = to_screen(xr, yp, zp, view);
            if sx.is_finite() && sy.is_finite() {
                run.push((sx, sy, zp));
                started = true;
            }
            continue;
        }
        match project_clipped(cx[i - 1], cy[i - 1], cz[i - 1], cx[i], cy[i], cz[i], view) {
            Some((sx1, sy1, zp1, sx2, sy2, zp2)) => {
                // The clip may have moved the start point; replace the
                // stored end of the run, then extend it.
                *run.last_mut().unwrap() = (sx1, sy1, zp1);
                run.push((sx2, sy2, zp2));
            }
            None => {
                flush(&mut out, &mut run);
                started = false;
            }
        }
    }
    flush(&mut out, &mut run);
    out
}

// ===== Space curves and positioned points (ADR-0015 amendment) =====

/// Project an arbitrary world-space polyline (a `solar3d` orbit or
/// trail) into visible screen runs: undefined points split runs, and
/// each run carries its mean view depth for painter's-order drawing.
/// Same treatment as a mesh grid line, from explicit points.
pub fn project_space_curve(points: &[[f64; 3]], view: &View3D) -> Vec<Polyline3D> {
    let mut out = Vec::new();
    let mut run: Vec<(f64, f64, f64)> = Vec::new(); // (sx, sy, zp)
    let flush = |out: &mut Vec<Polyline3D>, run: &mut Vec<(f64, f64, f64)>| {
        if !run.is_empty() {
            let depth = run.iter().map(|p| p.2).sum::<f64>() / run.len() as f64;
            out.push(Polyline3D {
                depth,
                points: std::mem::take(run)
                    .into_iter()
                    .map(|(x, y, _)| (x, y))
                    .collect(),
            });
        }
    };
    if let Some(first) = points.first() {
        let (xr, yp, zp) = to_camera(first[0], first[1], first[2], view);
        let (sx, sy) = to_screen(xr, yp, zp, view);
        if sx.is_finite() && sy.is_finite() {
            run.push((sx, sy, zp));
        }
    }
    for pair in points.windows(2) {
        let [x1, y1, z1] = pair[0];
        let [x2, y2, z2] = pair[1];
        match project_clipped(x1, y1, z1, x2, y2, z2, view) {
            Some((sx1, sy1, zp1, sx2, sy2, zp2)) => {
                if run.is_empty() {
                    run.push((sx1, sy1, zp1));
                } else {
                    *run.last_mut().unwrap() = (sx1, sy1, zp1);
                }
                run.push((sx2, sy2, zp2));
            }
            None => flush(&mut out, &mut run),
        }
    }
    flush(&mut out, &mut run);
    out
}

/// The render window for a zoomed 3D view. `x_min..x_max` and `y_min..
/// y_max` are the projected bounds at the current pose; the window is
/// those bounds scaled around their center by `view.camera / 30.0` (1.0
/// at the default zoom, halved per +1 zoom step). Because the projection
/// is affine and the window scales around a fixed center, zooming maps
/// every projected point to exactly `k ×` its default-zoom screen
/// position - relative positions are invariant under zoom by
/// construction (the ADR-0015 amendment).
///
/// Returns `(x_min, y_min, w, h)` in projected units.
/// The rotation-stable 3D window (ADR-0041): a square around the world
/// origin covering every point's distance from it. The 3D projection is
/// orthographic, so a sphere around the origin projects to a disc of the
/// same radius at every pose - a scene framed this way keeps its size and
/// position while it rotates, spins, or animates, instead of refitting
/// the window each frame. Returns `None` when no point is finite or the
/// scene collapses to a point at the origin; callers fall back to the
/// per-frame fit.
pub fn stable_window(
    points: impl Iterator<Item = [f64; 3]>,
    view: &View3D,
) -> Option<(f64, f64, f64, f64)> {
    let mut r2 = 0.0f64;
    let mut any = false;
    for [x, y, z] in points {
        if !(x.is_finite() && y.is_finite() && z.is_finite()) {
            continue;
        }
        any = true;
        r2 = r2.max(x * x + y * y + z * z);
    }
    if !any {
        return None;
    }
    let r = r2.sqrt() * 1.06; // the same 6% breathing room the per-frame fit pads
    if r < 1e-9 {
        return None;
    }
    Some(zoom_window(-r, r, -r, r, view))
}

pub fn zoom_window(
    x_min: f64,
    x_max: f64,
    y_min: f64,
    y_max: f64,
    view: &View3D,
) -> (f64, f64, f64, f64) {
    let zoom = view.camera / 30.0;
    let cx = (x_min + x_max) / 2.0;
    let cy = (y_min + y_max) / 2.0;
    let w = (x_max - x_min) * zoom;
    let h = (y_max - y_min) * zoom;
    (cx - w / 2.0, cy - h / 2.0, w, h)
}
pub fn project_world_dot(x: f64, y: f64, z: f64, view: &View3D) -> Option<(f64, f64, f64)> {
    let (xr, yp, zp) = to_camera(x, y, z, view);
    let (sx, sy) = to_screen(xr, yp, zp, view);
    if !sx.is_finite() || !sy.is_finite() {
        return None;
    }
    Some((sx, sy, zp))
}

// ===== 3D parametric curves (ADR-0054) =====

/// A sampled space curve: `graph3d param x(t), y(t), z(t)` over a t
/// domain (Desmos 3D and Nspire plot these; ADR-0054 brings epher to
/// them). Points are consecutive samples; non-finite coordinates split
/// the drawn line, exactly like undefined surface cells.
#[derive(Debug, Clone, PartialEq)]
pub struct SpaceCurve {
    pub source: String,
    pub t_domain: (f64, f64),
    pub points: Vec<[f64; 3]>,
}

/// Parse the text after `graph3d ` as a space curve:
/// `param <x(t)>, <y(t)>, <z(t)> [from a to b]`. The default t domain
/// matches the 2D parametric default (0..2pi).
pub fn parse_space_curve_source(
    source: &str,
) -> Result<(Expression, Expression, Expression, (f64, f64)), EpherError> {
    let source = source.trim();
    let Some(rest) = source.strip_prefix("param ") else {
        return Err(EpherError::Parse(
            "3D parametric curves are spelled `graph3d param <x(t)>, <y(t)>, <z(t)>`".to_string(),
        ));
    };
    let (body, domain) = split_domain(rest)?;
    let parts: Vec<&str> = split_top_level(body, ',')
        .into_iter()
        .map(|s| s.trim())
        .collect();
    let [x, y, z] = parts.as_slice() else {
        return Err(EpherError::Parse(
            "space curves need three expressions: `param <x(t)>, <y(t)>, <z(t)>`".to_string(),
        ));
    };
    let domain = domain.unwrap_or((0.0, std::f64::consts::TAU));
    if domain.0 >= domain.1 {
        return Err(EpherError::Parse(format!(
            "graph domain must run low to high, got {:.3} .. {:.3}",
            domain.0,
            domain.1
        )));
    }
    Ok((parse(x)?, parse(y)?, parse(z)?, domain))
}

/// Sample a space curve over its t domain; `t` is bound for each point
/// like the 2D parametric sampler binds it. Points that do not
/// evaluate to numbers are skipped; a curve needs at least two of them.
pub fn sample_space_curve(source: &str, points: usize, env: &Env) -> Result<SpaceCurve, EpherError> {
    let (x, y, z, domain) = parse_space_curve_source(source)?;
    let mut child = Env::new_child(env);
    let mut out: Vec<[f64; 3]> = Vec::with_capacity(points);
    for i in 0..points {
        let t = if points == 1 {
            0.0
        } else {
            i as f64 / (points - 1) as f64
        };
        let t = domain.0 + t * (domain.1 - domain.0);
        child.set("t", Value::float(t));
        let (Ok(Value::Float(px)), Ok(Value::Float(py)), Ok(Value::Float(pz))) = (
            eval(&x, &child),
            eval(&y, &child),
            eval(&z, &child),
        ) else {
            continue;
        };
        out.push([px, py, pz]);
    }
    if out.len() < 2 {
        return Err(EpherError::Type(
            "the space curve needs at least two defined points".to_string(),
        ));
    }
    Ok(SpaceCurve {
        source: source.to_string(),
        t_domain: domain,
        points: out,
    })
}

/// Project a sampled space curve for the SVG renderer: one polyline
/// per visible run, far-to-near.
pub fn project_curve(curve: &SpaceCurve, view: &View3D) -> Vec<Polyline3D> {
    let mut runs = project_space_curve(&curve.points, view);
    runs.sort_by(|a, b| b.depth.total_cmp(&a.depth));
    runs
}

/// The scene frame for a space curve: the ground square and the axes,
/// sized to the curve's bounding box (the surface frame sizes itself
/// to its square domain the same way).
pub fn curve_frame(curve: &SpaceCurve, view: &View3D) -> Vec<Segment3D> {
    let mut x_min = f64::INFINITY;
    let mut x_max = f64::NEG_INFINITY;
    let mut y_min = f64::INFINITY;
    let mut y_max = f64::NEG_INFINITY;
    let mut z_min = f64::INFINITY;
    let mut z_max = f64::NEG_INFINITY;
    for [x, y, z] in &curve.points {
        if !([x, y, z].iter().all(|v| v.is_finite())) {
            continue;
        }
        x_min = x_min.min(*x);
        x_max = x_max.max(*x);
        y_min = y_min.min(*y);
        y_max = y_max.max(*y);
        z_min = z_min.min(*z);
        z_max = z_max.max(*z);
    }
    // A degenerate extent (a circle in a plane, a straight line) still
    // deserves a frame; grow the flat directions to a unit span.
    let grow = |lo: &mut f64, hi: &mut f64| {
        if *hi - *lo < 1e-9 {
            *lo -= 0.5;
            *hi += 0.5;
        }
    };
    grow(&mut x_min, &mut x_max);
    grow(&mut y_min, &mut y_max);
    grow(&mut z_min, &mut z_max);
    let mut frame = Vec::with_capacity(8);
    let mut edge = |x1: f64, y1: f64, z1: f64, x2: f64, y2: f64, z2: f64| {
        if let Some((sx1, sy1, zp1, sx2, sy2, zp2)) = project_clipped(x1, y1, z1, x2, y2, z2, view)
        {
            frame.push(Segment3D {
                x1: sx1,
                y1: sy1,
                x2: sx2,
                y2: sy2,
                depth: (zp1 + zp2) / 2.0,
            });
        }
    };
    edge(x_min, y_min, 0.0, x_max, y_min, 0.0);
    edge(x_max, y_min, 0.0, x_max, y_max, 0.0);
    edge(x_max, y_max, 0.0, x_min, y_max, 0.0);
    edge(x_min, y_max, 0.0, x_min, y_min, 0.0);
    edge(x_min, 0.0, 0.0, x_max, 0.0, 0.0);
    edge(0.0, y_min, 0.0, 0.0, y_max, 0.0);
    edge(0.0, 0.0, z_min, 0.0, 0.0, z_max);
    frame
}
