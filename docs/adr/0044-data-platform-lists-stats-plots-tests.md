# ADR-0044: the data platform — lists, statistics, linear regression, data plots, table upgrades, and tests

- Status: accepted
- Date: 2026-09-01
- Roadmap: feature-gap analysis round 3 (data platform: T1.6, T2.7, T2.11, T3.3, plus the T2.2 distributions T3.3 depends on)

## Context

Rounds 1 and 2 shipped the quick wins and the core block. The data
platform is the statistics-class expectation: a named column of
numbers, statistics over it, linear regression with a fitted-curve
overlay, data plots (histogram, box-and-whisker, scatter), a richer
table of values, and hypothesis tests with confidence intervals — the
z/t/chi-squared appliance of the stats course. Hypothesis tests need
the probability distributions first (T2.2), which this round pulls in
as a prerequisite. Eight of the nine researched apps have lists, data
plots, and regression; six have tests and intervals.

## Decision

### Lists as values (T1.6)

- A new `Value::List` variant holds a homogeneous column of floats
  (any count, including zero). The literal is `{1, 2, 3}` (NumWorks
  spelling, the user's chosen answer); elements are expressions, so
  `{1, 2 + 3, pi}` works. Lists nest only through variables: `d =
  {1, 2, 3}` binds one, and a list element must be a number — complex
  values are rejected with a type error.
- Element access is a postfix index: `d[2]` is the second element,
  1-based like the researched calculators. The index is any integer
  expression, evaluated when the access runs; out-of-range is an
  error. `list[i]` binds tighter than `^`: `d[2]^2` is `(d[2])^2`.
- Arithmetic is elementwise with scalar broadcast: `{1,2,3} * 2` is
  `{2, 4, 6}`, `2 / {1,2,3}` divides the scalar by each element,
  `{1,2} + {3,4}` is `{4, 6}`, and two lists of different lengths are
  a type error. Unary minus negates elementwise. Comparisons: `==` and
  `!=` compare whole lists; ordering comparisons reject lists.
- The existing variadic statistics accept a single list as their one
  argument: `sum product mean variance stdev median min max` all work
  on `{...}` and keep their variadic form. New builtins: `len`,
  `sort` (ascending copy), `mode` (the most frequent value, smallest
  on ties), `range` (max minus min), `quartile(list, k)` for k in
  1..3 (TI-style median-of-halves). All statistics reject complex
  elements; an empty list is a domain error.
- `linreg(xs, ys)` fits the least-squares line: two same-length
  numeric lists in, a display string out — `y = 2*x + 1 (r = 1)` —
  like solve's result spelling. The fitted line is a display, not a
  stored function; the overlay lives on the scatter plot (T2.7).

### Distributions (T2.2, pulled in as T3.3's prerequisite)

- Probability functions, pure f64 arithmetic (regularized incomplete
  beta and gamma via continued fractions, Acklam's inverse normal):
  `normpdf(x[, mu, sigma])`, `normcdf(x[, mu, sigma])`,
  `invnorm(p[, mu, sigma])` (1-argument forms are the standard
  normal), `tpdf(x, df)`, `tcdf(x, df)`, `invt(p, df)`,
  `chi2pdf(x, df)`, `chi2cdf(x, df)`, `invchi2(p, df)`,
  `binompdf(k, n, p)`, `binomcdf(k, n, p)`, `poissonpdf(k, lambda)`,
  `poissoncdf(k, lambda)`. Parameters are checked: probabilities in
  [0,1], degrees of freedom positive, p in (0,1).
- `invt` and `invchi2` are Newton on the CDF with the PDF as the
  derivative; `binompdf` uses the numerically stable recurrence and
  `binomcdf` sums it, so large n and extreme p stay accurate.

### Hypothesis tests and confidence intervals (T3.3)

- Four test functions, lists of data in, display strings out:
  `ztest(data, mu0, sigma)` reports `z = …, p = …` (two-sided),
  `ttest(data, mu0)` reports `t = …, p = …` (two-sided, n−1 degrees
  of freedom), `chisq_gof(observed, expected)` reports `chi2 = …,
  p = …` (k−1 degrees of freedom) for goodness of fit.
- Two intervals: `zinterval(data, sigma, level)` and
  `tinterval(data, level)` report `(lo, hi)`; the level is explicit
  (0.95, 0.99, …) and checked into (0,1).
- Results are display strings, exactly like `solve` and `linreg`:
  readable, copy-pasteable, and never mistaken for values that
  arithmetic could touch. One-way ANOVA is deferred (the report lists
  it alongside z/t/chi-squared; it is the least-used classroom piece
  and needs list-of-lists).

### Data plots (T2.7)

- Three graph-family members, `graph` commands that take list
  arguments instead of expressions:
  - `graph scatter(xs, ys)` — the points, plus the least-squares fit
    line and its `y = a*x + b (r = …)` legend entry when there are
    two or more points.
  - `graph histogram(data)` — a frequency histogram; an optional
    second argument sets the bin count (`graph histogram(d, 8)`),
    otherwise Sturges' rule (ceil(log2 n) + 1).
  - `graph boxplot(data)` — a box-and-whisker plot: min, Q1, median,
    Q3, max (whiskers to the extremes; no outlier marking this
    round).
- The core computes the plot primitives — points, the fit line, bin
  rectangles, the five-number box — and the frontends render, exactly
  the ADR-0006 seam: the web pane draws circles, bars, and boxes in
  SVG; the TUI draws glyphs and block characters in ASCII; `graph
  save` writes the same picture as SVG. A data plot is the pane's top
  priority (above curves, like the solar system): submitting one
  shows it; `graph clear` removes it.
- Data plots skip the points-of-interest analysis (it is defined for
  Cartesian curves) and the graph-domain keywords (`from a to b` does
  not apply; the plot window fits the data).

### Table upgrades (T2.11)

- `table <expr> [from a to b] [points n] [derivative <expr>]` gains an
  optional derivative column: the second expression is differentiated
  numerically at each x with the existing 5-point stencil and shown as
  a third column. The keyword can never collide (the language has no
  `derivative` identifier).
- Table cells follow the session's exact-fractions display preference:
  when the exact toggle is on (the default) and a value reconstructs
  as a small-denominator fraction, the cell shows `1/3` instead of
  `0.333`.
- The pasted-data-column editor and the table's own plot toggle are
  web-UI surfaces deferred with the report's blessing (plotting a
  table as a scatter is already `graph scatter` on a list, which the
  table experience now feeds); this round keeps the table text-based
  in every frontend.

## Consequences

- Lists are floats-only columns; elementwise arithmetic and the stats
  reject complex elements, and map/filter higher-order forms are
  deferred (arithmetic + the stats cover the report's ergonomic
  minimum).
- The display strings of `linreg`/tests are not values arithmetic can
  use; coefficients live on in the scatter overlay only.
- New builtins enter the catalog and the guide in all eight locales;
  the keypad stays frozen — lists and tests are typed, and
  autocomplete offers every new name.
- Distribution and test functions are deterministic f64 arithmetic —
  no new dependencies, wasm-safe, testable to the last digit against
  reference tables.
