# ADR-0048: implicit relations

- Status: accepted
- Date: 2026-09-02
- Roadmap: feature-gap analysis round 7 (T2.10 implicit relations —
  `x^2 + y^2 == 1` plotted as a curve, five of the nine apps)

## Context

Desmos, GeoGebra, the Nspire relations, HP's Advanced Graphing, and
NumWorks conics all plot equations in two unknowns directly. epher's
graph family only plots `y = f(x)` (plus parametric and polar forms),
so `graph x^2 + y^2 == 1` currently draws nothing — the expression
evaluates to a boolean per x, which the sampler drops. The report
scopes the feature to marching squares on the existing sampler; the
legend, zoom, export, and slider machinery should come for free.

## Decision

- The graph grammar gains the implicit relation: a body whose top
  level is `==` becomes `CurveKind::Implicit(equation)`. `y == x^2`,
  `x == 2`, and `x^2 + y^2 == 1` all parse as relations; the
  inequality fills (`y < …`) stay as they are, and an equation with a
  fill prefix is a parse error. `from a to b` still sets the domain,
  and the relation is sampled over the square `[a, b] × [a, b]` (the
  default domain, like the other kinds).
- The core samples the relation with marching squares on an N×N grid
  (N = the usual 120 points): each vertex evaluates the difference
  `lhs − rhs` with `x` and `y` bound, each cell's corner signs pick
  the crossing case, edge crossings are linearly interpolated, and
  the ambiguous saddle cells (cases 5 and 10) are resolved with the
  cell-center average. The resulting segments chain into contour
  branches.
- The branches leave the sampler as ordinary `Sample`s separated by
  non-finite "pen-up" markers — the same gap mechanism vertical
  asymptotes already use — so the renderers (web SVG, TUI ASCII, SVG
  export, PNG export) and the view fitting split and draw them for
  free, exactly like `segments()` splits `1 / x`.
- Points of interest do not apply (the existing `cartesian_expr`
  gate returns None for relations), and the caption shows the
  equation as typed, like every other curve's source text.

## Consequences

- `graph x^2 + y^2 == 1` plots a circle, `graph y == x^2` a parabola,
  `graph x == 2` a vertical line — every relation renders with the
  existing legend, zoom, pan, export, and slider behavior.
- Relations are sampled, never solved: sparse or near-singular
  regions may look blocky at the default grid, exactly like the
  researched apps' implicit plots.
- One new parse branch and one new sampler; no frontend changes
  beyond the guide, because the seam payloads are unchanged.
