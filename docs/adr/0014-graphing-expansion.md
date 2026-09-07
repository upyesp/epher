# Graphing expansion: multi-curve, trace, analysis, tables, sliders

## Status

Accepted (2026-08-17).

## Context

The pre-0014 implementation of graphing (ADR-0006) was minimal: one
curve (`y = f(x)`), a fixed domain, static rendering — SVG in the
web/desktop app, ASCII in the TUI, nothing in the CLI. (ADR-0006's own
"v1 graphs parametric and polar" was the v1 plan; this ADR ships both
forms pre-v1.) The competitive landscape (surveyed from primary sources in
`docs/research/graphing-features.md`: Desmos, GeoGebra, NumWorks, TI, and
Wolfram) converges on a common feature set — expression lists, trace with
coordinates, roots/intersections/extrema analysis, tables of values,
adjustable domains, region shading, sliders for free variables, and export.
This ADR records how epher adopts that set within its own seams (core
computes, frontends render, per ADR-0006) and what it deliberately defers.

## Decisions

### Grammar lives in core

`parse_graph_source` (in `epher-core::graph`) owns the whole command
grammar, so every frontend behaves identically and the surface is
unit-testable without a browser:

- Cartesian: `graph <expr> [from a to b]`
- Parametric: `graph param <x(t)>, <y(t)> [from a to b]` (comma split at
  paren depth zero, so function calls with arguments keep working)
- Polar: `graph polar <r(θ)> [from a to b]`
- Regions: `graph y < <expr>` / `graph y <= <expr>` shade below the curve;
  `y >` / `y >=` shade above. (`<expr>` itself may not begin with a `y`
  comparison; documented, not guarded.)
- Default domains: −10..10 for Cartesian; 0..2π for parametric and polar.
- `graph clear` empties the plot.

`table <expr> [from a to b] [points n]` has its own grammar
(`parse_table_source`), with the same `from a to b` bounds syntax, TI-style
defaults (start −5, end 5, 11 rows), and a 1000-point cap.

### Multiple curves

Each `graph` line overlays one more curve (`Vec<SampledCurve>` per
frontend). The web app distinguishes curves by colour **and** dash pattern
(colours verified ≥ 3:1 on the app background, WCAG 1.4.11; dashes keep
curves distinguishable without colour, 1.4.1); the TUI uses distinct glyphs
(`o x + *`). Both render a legend naming every plotted expression — the
legend is the accessible text alternative for the plot.

### Points of interest, computed in core

`analyze` finds roots and local extrema of each Cartesian curve and
intersections between curve pairs (sign changes over the sampled data,
refined by bisection; extrema refined by golden-section search).
Intersections require overlapping domains. The web app marks them on the
plot and lists them beneath it; the TUI lists them under the ASCII plot.
Parametric/polar analysis is deferred (see below).

### No false asymptotes

Renderers split curve segments at non-finite points *and* at jumps larger
than 35% of the sampled value span — the two branches of `1 / x` or
`tan(x)` must never be joined by a vertical line.

### Trace (web/desktop only)

The plot is keyboard-focusable; pointer move/tap and arrow keys trace the
nearest sampled point. Coordinates are announced in an `aria-live` region
(WCAG). The TUI has no trace in this iteration — its input line owns the
keyboard; see deferrals.

### Sliders (web/desktop only)

Any session constant referenced by a plotted expression gets a slider
beneath the plot. Sliders re-sample every curve and re-run the analysis.
`Session::set_constant` updates both the value and the recorded source
text, so a later `save <name>` persists the slider's value (ADR-0012's
contract holds). The TUI has no sliders; changing a constant there means
re-typing the `const` line (see deferrals).

### Tables through the shell

`table` is a shell command (`Command::Table`), not a graph-panel feature:
it works in the REPL, piped scripts, the TUI, and the web app through the
one shell policy, and its output is plain monospace text (blank rows use
an em dash, TI-style). The one-shot CLI stays expression-only (ADR-0013's
calculator-first rule) — `epher table …` as an argument is not a thing, by
design.

### Export

The web app's **Copy SVG** button copies the current plot via the same
string renderer the unit tests exercise. No raster export, no share URLs.

## Deferred (deliberately)

- **3D** — deferred in ADR-0006, and shipped by ADR-0015 (its
  projection design made it reachable for both renderers).
- **Implicit relations** (`x^2 + y^2 = 25`) — needs marching-squares-style
  region sampling; the `y < f(x)` fill covers the common teaching cases.
- **Parametric/polar analysis** — roots, extrema, and intersections for
  non-Cartesian curves (intersection of a parametric curve with a Cartesian
  one is the common case; it needs curve-distance minimization, not the
  current sign-change machinery).
- **TUI trace and sliders** — the TUI's keyboard belongs to the input line;
  a modal trace mode is a real feature, not a bolt-on. The POI list is the
  TUI's answer to "read the coordinates" for now.
- **Zoom/pan gestures** — domain bounds cover windowing; gesture zoom needs
  a window-state model that no current command surface expresses. Numeric
  viewport bounds are the reachable subset.
- **The rest of the CALC menu** (TI) / **Find menu** (NumWorks) — value-at-x,
  derivative, definite integral, tangent line, area between curves: the
  same core analysis engine extends to them, but each needs a command
  surface decision first.
- **Regressions** (Desmos/NumWorks/TI) — least-squares fitting wants a
  points/lists model in the language that doesn't exist yet.
- **Share URLs / cloud saving** — no account layer exists.

## Consequences

- `epher-core::graph` is the single source of grammar, sampling, analysis,
  tick steps, and tables; every frontend is thin rendering over it.
- Seven new localized strings (`poi-*`, `graph-points`, `graph-copy`,
  `graph-copied`, `graph-copy-failed`) join the eight catalogs; the guide
  documents the whole feature set in all eight languages.
- The web graph component is now interactive (trace, sliders, copy) — its
  accessibility surface (keyboard trace, `aria-live` announcements, legend
  as text alternative, contrast-verified palette) is part of the contract,
  not an afterthought.

## Amendment (2026-09-05): the table command joins the history

A `table` run landed in the result pane but never in the history list —
in the web app (and so the desktop app and PWA), the TUI, and the CLI
REPL alike. Every other submitted line joins the history: expressions
record their answer (ADR-0021), and the plot commands record the bare
command with the plot as the output (ADR-0027: "the command joins the
history list"). The table was the one computation command whose
dispatch branch — the shell-command branch, shared with `save`,
`language`, and `theme` — recorded nothing, so a table could not be
reached, re-run, or shared from the history it belonged in.

The table now records like the plots do: the bare command line, no
answer suffix — the rendered table is the output, and picking the
entry loads the command, so re-running it regenerates the table. The
recording stays in each frontend's dispatch (not in the shell kernel
or `Session::submit`): a table never reaches the evaluator, and the
kernel's `run_command` must stay recording-free for the `load` path,
whose lines record nothing beyond the `load` line itself (ADR-0040).
Multi-statement lines already record once for the whole line at their
tails, so only the single-statement table changes behavior. The
administrative commands (`save`, `language`, `theme`) keep leaving no
entry, and the piped CLI keeps recording nothing (scripts are not
interactive pasts).
