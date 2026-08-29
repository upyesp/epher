//! Graph command parsing and analysis (ADR-0006, ADR-0014): everything a
//! frontend needs to turn a `graph …` line into plottable data lives here —
//! the command grammar, per-curve sampling, points of interest (roots,
//! intersections, extrema), tables of values, and tick-step selection.
//! Frontends only render.

use std::collections::BTreeSet;

use crate::{eval, evaluate, parse, Env, EpherError, Expression, Sample, Value};

/// The default x (or t/θ) domain for a curve kind, when the command names no
/// bounds.
pub fn default_domain(kind: &CurveKind) -> (f64, f64) {
    match kind {
        CurveKind::Cartesian(_) => (-10.0, 10.0),
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
    Parametric { x: Expression, y: Expression },
    Polar(Expression),
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
        CurveKind::Cartesian(parse(expr.trim())?)
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
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
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
    }
}

/// The Cartesian expression, for curves that are `y = f(x)` (points-of-
/// interest analysis only applies to them).
pub fn cartesian_expr(kind: &CurveKind) -> Option<&Expression> {
    match kind {
        CurveKind::Cartesian(e) => Some(e),
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

/// A parsed `table` command: what to evaluate and over which x values.
/// Defaults match TI's table (start −5, end 5, 11 rows); `points` is
/// capped so a bad command can't demand unbounded work.
#[derive(Debug, Clone)]
pub struct TableSpec {
    pub expr: Expression,
    pub x_min: f64,
    pub x_max: f64,
    pub points: usize,
}

/// Parse the text after `table `: `expr [from a to b] [points n]`.
/// The language has no `from`/`to`/`points` identifiers, so the keywords
/// can never collide with the expression.
pub fn parse_table_source(source: &str) -> Result<TableSpec, EpherError> {
    const DEFAULT_POINTS: usize = 11;
    const MAX_POINTS: usize = 1000;

    let source = source.trim();
    if source.is_empty() {
        return Err(EpherError::Parse("empty table command".to_string()));
    }
    // The `points n` suffix sits after the domain, so strip it first.
    let (rest, points) = match source.rfind(" points ") {
        Some(idx) => {
            let (expr, n) = source.split_at(idx);
            let n = n.trim_start_matches(" points ").trim();
            let n: usize = n
                .parse()
                .map_err(|_| EpherError::Parse(format!("`points {n}` needs a whole number")))?;
            if !(1..=MAX_POINTS).contains(&n) {
                return Err(EpherError::Parse(format!(
                    "`points` must be between 1 and {MAX_POINTS}"
                )));
            }
            (expr.trim(), n)
        }
        None => (source, DEFAULT_POINTS),
    };
    let (body, domain) = split_domain(rest)?;
    let (x_min, x_max) = domain.unwrap_or((-5.0, 5.0));
    if x_min >= x_max {
        return Err(EpherError::Parse(format!(
            "table domain must run low to high, got {x_min:.3} .. {x_max:.3}"
        )));
    }
    Ok(TableSpec {
        expr: parse(body)?,
        x_min,
        x_max,
        points,
    })
}

/// A row of a table of values: x always present; y absent where the
/// expression has no value (TI-style blank rows).
pub fn table_rows(
    expr: &Expression,
    x_min: f64,
    x_max: f64,
    points: usize,
    env: &Env,
) -> Vec<(f64, Option<f64>)> {
    let mut out = Vec::new();
    for i in 0..points {
        let t = if points == 1 {
            0.0
        } else {
            i as f64 / (points - 1) as f64
        };
        let x = x_min + t * (x_max - x_min);
        out.push((x, eval_at(expr, x, env)));
    }
    out
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
        Expression::Var(name) => {
            out.insert(name.clone());
        }
        Expression::Call(_, args) => {
            for a in args {
                free_names(a, out);
            }
        }
        Expression::Neg(e) | Expression::Factorial(e) | Expression::Not(e) => free_names(e, out),
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
/// pitch around the rotated x axis, with a perspective camera `camera`
/// units out along the view axis. All angles in radians.
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
            // Far enough out that the camera plane sits beyond typical
            // surface z values (|z| <= 25 over a ±5 domain), so the
            // default view shows the whole surface instead of a near-
            // plane cut; near-plane clipping still engages when the user
            // orbits in close.
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

    /// Set the camera distance (mouse-wheel zoom, ADR-0034).
    pub fn with_camera(&self, camera: f64) -> Self {
        Self {
            camera: camera.max(0.5),
            ..*self
        }
    }

    /// Apply the fine-control sliders' offsets (ADR-0031), each −1..1
    /// with 0 = this pose unchanged: horizontal adds `h × π` to the yaw;
    /// vertical adds `v × 0.8` to the pitch — the full range stays live
    /// at the default pose (pitch 0.6 + 0.8 = 1.4, exactly the clamp) —
    /// and zoom multiplies the camera distance by `2^-z` (0 = the default
    /// distance, +1 halves it, −1 doubles it).
    pub fn with_offsets(&self, h: f64, v: f64, z: f64) -> Self {
        Self {
            yaw: self.yaw + h * std::f64::consts::PI,
            pitch: (self.pitch + v * 0.8).clamp(-1.4, 1.4),
            camera: self.camera * 2f64.powf(-z),
        }
    }

    /// Compose an animated spin (ADR-0032): `yaw`/`pitch` are the
    /// accumulated rotation phase from the fine-control sliders' continuous
    /// spin, `zoom` the static zoom offset. Unlike [`Self::with_offsets`]
    /// the pitch is NOT clamped — a vertical spin is a full revolution
    /// around the horizontal axis and may carry the camera under the plot;
    /// the sine/cosine projection keeps the pose continuous through the
    /// poles.
    pub fn with_spin_phase(&self, yaw: f64, pitch: f64, zoom: f64) -> Self {
        Self {
            yaw: self.yaw + yaw,
            pitch: self.pitch + pitch,
            camera: self.camera * 2f64.powf(-zoom),
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
/// then a perspective divide. Returns (screen x, screen y, view depth);
/// screen y grows upward and depth grows toward the camera.
/// Distance of the near clipping plane from the camera, in view units.
/// Segments closer to the camera than this (or behind it) are not
/// projected: without clipping, a surface crossing the camera plane makes
/// the perspective divide blow up to huge screen coordinates and the plot
/// becomes a sliver in a giant (mostly empty) view box.
pub const NEAR_DIST: f64 = 1.0;

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

/// Perspective divide: map a camera-space point to screen coordinates.
fn to_screen(xr: f64, yp: f64, zp: f64, view: &View3D) -> (f64, f64) {
    let f = view.camera / (view.camera - zp);
    (xr * f, -yp * f)
}

/// Project one world point; raw, with no near-plane clipping. Callers that
/// draw whole segments or meshes should use [`project_clipped`] instead so
/// geometry crossing the camera plane does not explode.
pub fn project_point(x: f64, y: f64, z: f64, view: &View3D) -> (f64, f64, f64) {
    let (xr, yp, zp) = to_camera(x, y, z, view);
    let (sx, sy) = to_screen(xr, yp, zp, view);
    (sx, sy, zp)
}

/// Project a world-space segment to screen coordinates, clipping it against
/// the near plane. Returns the clipped endpoints and their camera-space z
/// values (`(sx1, sy1, zp1, sx2, sy2, zp2)`), or None when the segment is
/// fully behind the near plane or touches an undefined (NaN) cell.
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
    let (mut xr1, mut yp1, mut zp1) = to_camera(x1, y1, z1, view);
    let (mut xr2, mut yp2, mut zp2) = to_camera(x2, y2, z2, view);
    // zp measures distance along the view axis toward the camera (which
    // sits at zp = view.camera): points with zp > near are too close to
    // (or behind) the camera to project stably.
    let near = view.camera - NEAR_DIST;
    if zp1 > near && zp2 > near {
        return None;
    }
    if zp1 > near {
        let t = (near - zp1) / (zp2 - zp1);
        xr1 += t * (xr2 - xr1);
        yp1 += t * (yp2 - yp1);
        zp1 = near;
    } else if zp2 > near {
        let t = (near - zp1) / (zp2 - zp1);
        xr2 = xr1 + t * (xr2 - xr1);
        yp2 = yp1 + t * (yp2 - yp1);
        zp2 = near;
    }
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
            if zp <= view.camera - NEAR_DIST {
                let (sx, sy) = to_screen(xr, yp, zp, view);
                if sx.is_finite() && sy.is_finite() {
                    run.push((sx, sy, zp));
                    started = true;
                }
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
/// trail) into visible screen runs: segments crossing the near plane are
/// clipped, and each run carries its mean view depth for painter's-order
/// drawing. Same treatment as a mesh grid line, from explicit points.
pub fn project_space_curve(points: &[[f64; 3]], view: &View3D) -> Vec<Polyline3D> {
    let mut out = Vec::new();
    let mut run: Vec<(f64, f64, f64)> = Vec::new(); // (sx, sy, zp)
    let mut flush = |out: &mut Vec<Polyline3D>, run: &mut Vec<(f64, f64, f64)>| {
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
        if zp <= view.camera - NEAR_DIST {
            let (sx, sy) = to_screen(xr, yp, zp, view);
            if sx.is_finite() && sy.is_finite() {
                run.push((sx, sy, zp));
            }
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

/// Project one world point for a positioned dot: screen coordinates plus
/// view depth, or None when the point is behind the camera plane.
pub fn project_world_dot(x: f64, y: f64, z: f64, view: &View3D) -> Option<(f64, f64, f64)> {
    let (xr, yp, zp) = to_camera(x, y, z, view);
    if zp > view.camera - NEAR_DIST {
        return None;
    }
    let (sx, sy) = to_screen(xr, yp, zp, view);
    if !sx.is_finite() || !sy.is_finite() {
        return None;
    }
    Some((sx, sy, zp))
}
