# ADR-0054: the stats class, table columns, space curves, and the language surface

Date: 2026-09-02 · Status: accepted

## Context

The gap-analysis rerun (docs/research/calculator-gap-analysis-rerun.md)
left a short, user-nameable backlog after v0.5.16: the two partial rows
real users can name (regression beyond linear, tests without ANOVA or a
paired t), the probe-surfaced small gaps (`randn`, table columns), the
niche-but-cheap 3D parametric curve, and the language surface (strings,
`for`, `print`) that T3.4 deferred. This round closes the first four and
pulls the language surface forward, per the standing review.

## Decision

### The stats class rides the existing machinery

- **`anova(list1, list2, …)`** fits one-way analysis of variance over two
  or more groups, unequal lengths welcome. Reports `F = …, p = …` in
  the display-string shape `ttest` established. The F CDF is the
  incomplete beta (`puruspe::betai` behind the ADR-0053 clamping
  wrapper): I_x(d1/2, d2/2) at x = d1·f/(d1·f + d2), textbook and
  crate working together.
- **`ttestpaired(a, b)`** is the paired t: differences of two
  same-length lists, tested against 0 through the one-sample path that
  already existed. Degenerate (zero-variance) pairs behave exactly
  like `ttest`'s.
- **`randn(mu, sigma)`** draws Box-Muller on the seeded splitmix stream.
  Two uniforms per draw; zero is mapped off to keep `ln` finite. The
  reproducibility property is the ADR-0045 one: same seed, same draws,
  every frontend.
- **`quadreg` / `expreg` / `powreg` / `logreg`** cover the quadratic
  (normal equations through a 20-line pivot solver: the reuse ladder's
  last resort: a plain-float 3×3 solve is smaller than any linear-
  algebra crate that could do it), and the three transformed fits
  through the linear fit that already existed, with each model's
  domain honesty enforced (y > 0, x > 0). Each reports `y = … (r = …)`;
  r is the correlation of the linearized pair, the same number TI and
  NumWorks report.
- **Scatter overlays**: `graph scatter(xs, ys, quadreg)` (and expreg,
  powreg, logreg) draws that model's curve over the points, captioned
  in the legend like the line. The fit struct generalizes `FitLine`
  into `{model, Fit}`, and both renderers sample the model instead of
  evaluating endpoints, so the quadratic can turn inside the window.

### Table: the data column and the display toggle

- **`values <list>`** takes the x column from a data list instead of
  an even grid (TI's paste-a-data-column). Mutually exclusive with
  `from a to b` and `points n`; capped at the same 1000 rows.
- **`exact` / `approx`** force the cell display for one table,
  overriding the session's exact-fraction setting for that command.

### 3D parametric curves

`graph3d param x(t), y(t), z(t)` plots a space curve over a t domain
(`from a to b`, default 0..2pi, matching the 2D parametric default). The
sampler binds `t` exactly as the 2D one does; the projection reuses the
solar scene's polyline projector; the frame sizes its ground square and
axes to the curve's bounding box. The web pane and the TUI wireframe
treat a curve set exactly like a surface set: one kind owns the pane,
orbit/zoom/animation carry over unchanged, and Copy SVG / Save PNG
export it.

### The language surface: strings, for, print

- **String literals** `"…"` join the grammar: no escape sequences, so
  a string cannot contain a double quote; one tokenizer pass, one
  rule to document. Strings concatenate with `+`, compare with
  `==`/`!=`, count with `len`, and index 1-based like lists. Mixing a
  string with a number is a type error, not a silent coercion.
- **`str(x)`** spells one value the way the answer panel does;
  **`print(a, b, …)`** joins its arguments with spaces into one line.
  Both return the line as the display string they produce; no output
  channel, no new history schema: a string is already a value the
  language carries.
- **`for i in 1 to 5 do body`** (inclusive, optional `step`, negative
  steps count down) and **`for x in d do body`** iterate and collect
  the body's values into a list: a comprehension by construction, the
  same shape as a `table`. The loop variable keeps its last value
  afterwards, like TI's For. Values are computed index-against-start,
  never accumulated, so `0 to 1 step 0.5` lands exactly. The runaway
  guard: at most 100,000 iterations, on top of the existing step
  budget. A reversed range against its step is simply empty.
- The §1.7 "epher has no text values" guide note is retired; `if`
  compares strings of equal kind.

## Consequences

- The two partial rows close: regression 8/9 → 9/9, tests 7/9 → 9/9;
  `randn` and the table columns close their probe rows. The remaining
  absent rows stay deliberate (CAS, exam mode, step-by-step,
  spreadsheet, geometry, natural language, live data).
- The language gains its first non-numeric values. Every list built as
  data stays floats-only (literal construction enforces it); only the
  `for` collector and string producers carry strings, and display of
  both was already defined.
- The web bundle grows only in code, not embedded data: the guide
  additions ride ADR-0053's on-demand path.
- Quick reference, stats, random, table, and 3D sections of the guide
  updated in all eight locales with byte-identical epher fences.
