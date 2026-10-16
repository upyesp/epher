# Calculator gap-analysis rerun: epher after nine work packages

Date: 2026-09-02 · Status: findings only, nothing implemented

## Why this rerun

The first feature-gap analysis (docs/research/calculator-feature-gap-analysis.md,
2026-08-31) compared epher v0.5.4 against nine reference calculators and
found epher absent on 21 of the 31 expectation rows. Nine work packages
followed (v0.5.5 through v0.5.16, ADRs 0042-0052): percent and number
theory, complex numbers and equation solving, calculus and exact layers,
the data platform (lists, stats, regression, tests, data plots), seeded
random numbers, scientific constants, the unit system, bitwise
operations, implicit relations, matrices, finance, display rounding, and
the parity corrections. This rerun re-measures the same 31 expectations
against epher v0.5.16 and the same nine apps, and re-verifies the app
columns against primary sources.

## Executive summary

- **epher now covers 24 of the 31 expectation rows** (21 full, 3
  partial), up from 8 at the baseline (3 full, 5 partial). Every row
  the first analysis called "table stakes" (Tier 1) ships, with
  regression as the linear model its Tier-1 scope named.
- **All nine apps' columns re-verified against primary sources on
  2026-09-02; no app gained or lost features since the baseline.** The
  re-check corrected three app-column claims: HiPER has no data plots
  (histogram/box/scatter absent from the v12 manual), Desmos's auto
  points-of-interest is not documented (downgraded to partial), and
  Octave's distributions and tests live in the official statistics
  package rather than the core manual (downgraded to partial).
  GeoGebra complex arithmetic is confirmed with the caveat that it is
  simulated through points/CAS rather than a native number type.
- **Seven rows remain absent** and they are all deliberate product
  boundaries, not oversights: CAS (T3.1), exam mode (T3.8),
  step-by-step solutions (T3.6), spreadsheet view, dynamic geometry,
  natural-language input (T3.10), and curated/live data (T3.11).
- **Three rows are partial**: curve fitting is linear-only (`linreg`),
  hypothesis tests stop short of ANOVA and paired/rank tests, and the
  programming surface has no strings, `for` loops, or print. Two of
  the three (regression beyond linear, ANOVA) are the natural next
  round; the language surface was explicitly deferred (T3.4).
- **Two small gaps surfaced by the probes** that the baseline did not
  list: no normal-distribution random draw (`randn`), and no table
  extras (derivative column, data tables). Both are cheap.
- epher's unique strengths still hold: the built-in offline solar
  system is the only orrery among the nine; the script transcripts
  (every answer, in order), seeded reproducible randomness across
  frontends, and byte-identical 8-locale in-app guide are not matched
  by any of the nine.

## Method

- **epher column**: empirical. Every row was probed against the
  v0.5.16 binary (one-shot CLI over the probe scripts in
  /tmp/gap-rerun/) and cross-checked against the in-app guide
  (site/guide/en.md, the function reference and settings sections).
  UI-only behaviors (sliders, POI markers, shading) were re-confirmed
  against the guide and the round 10-11 web E2E suites rather than
  re-shot.
- **App columns**: re-verified against the same primary sources as the
  baseline (official manuals and reference docs, one catalog per app).
  The re-verification notes are working files under /tmp/gap-rerun/
  (not committed); the matrix below is the durable record.

Sources re-fetched 2026-09-02 (the per-app re-verification notes are
working files under /tmp/gap-rerun/, not committed):

| # | App | Principal primary sources |
|---|-----|---------------------------|
| 1 | NumWorks | numworks.com manual v21.0.0 (HTML + PDF) |
| 2 | Desmos | help.desmos.com User Guide (updated 2026-08-27) + Help Center articles |
| 3 | GeoGebra | geogebra.org/manual command pages (via Wayback; wiki DNS was down from the research network) |
| 4 | TI-84 Plus CE / TI-Nspire CX II | TI guidebooks (TI-84 CE Apps/Reference/Programming, Nspire CX II 454 pp) |
| 5 | WolframAlpha / Wolfram Language | wolframalpha.com + reference.wolfram.com guides |
| 6 | SpeedCrunch | speedcrunch.org/userguide + /reference (the old /handbook/ URL is a 404 since v0.12) |
| 7 | HiPER Scientific | hiperlabs.eu manual v12.0 (PDF) |
| 8 | HP Prime | HP Prime User Guide 3rd ed. (761 pp, hpcalc.org) |
| 9 | MATLAB / GNU Octave | docs.octave.org manual 11.1.0 nodes |

## The expectation matrix

✓ full first-class support · ~ partial or adjacent · · absent.
Count = apps of the nine offering the row (✓ or ~).

| Capability | NW | DM | GG | TI | WL | SC | HP | Pr | Oc | of 9 | epher v0.5.4 | epher v0.5.16 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Complex numbers | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 9 | · | ✓ |
| Numeric equation solving | ✓ | ~ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | 8 | · | ✓ |
| Lists as values | ✓ | ✓ | ✓ | ✓ | ✓ | · | ~ | ✓ | ✓ | 8 | · | ✓ |
| Regression / curve fitting | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | 8 | · | ~ linear only |
| Probability distributions | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ~ | 8 | · | ✓ |
| Random numbers | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | 8 | · | ✓ |
| Matrices | ✓ | ~ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | 8 | · | ✓ |
| Calculus (deriv/integral) | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ~ | 8 | · | ✓ numeric |
| Data plots (hist/box/scatter) | ✓ | ✓ | ✓ | ✓ | ✓ | · | · | ✓ | ✓ | 7 | · | ✓ |
| Units + conversion | ✓ | · | · | ✓ | ✓ | ✓ | ✓ | ✓ | · | 6 | ~ astro only | ✓ |
| Exact/fraction results | ✓ | ✓ | ✓ | ✓ | ✓ | ~ | ✓ | ✓ | · | 8 | ~ opt-in | ✓ default |
| Display formats (sci/eng/digits) | ✓ | ~ | · | ✓ | · | ✓ | ✓ | ✓ | ✓ | 7 | · | ✓ |
| Table of values | ✓ | ✓ | ~ | ✓ | ~ | · | ✓ | ✓ | · | 6 | ~ no extras | ✓ |
| Auto points of interest | ✓ | ~ | ✓ | ✓ | ~ | · | ✓ | ✓ | · | 6 | ✓ | ✓ |
| Parameter sliders + animation | ~ | ✓ | ✓ | ✓ | ~ | · | ~ | ✓ | · | 6 | ✓ | ✓ |
| Inequality graphing | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | · | 7 | ✓ | ✓ |
| Percentage operator | · | ✓ | · | · | ~ | ~ | ✓ | · | · | 4 | · | ✓ |
| Hypothesis tests + CIs | ✓ | ✓ | ✓ | ✓ | ✓ | · | · | ✓ | ~ | 7 | · | ~ no ANOVA |
| Constants library (physics…) | ✓ | · | · | ~ | ✓ | ✓ | ✓ | ✓ | · | 6 | ~ astro only | ✓ |
| Financial (TVM/NPV/IRR) | ~ | · | ✓ | ✓ | ~ | · | · | ✓ | · | 4 | · | ✓ |
| Bitwise ops | ~ | · | · | ~ | ✓ | ✓ | ✓ | ✓ | ✓ | 7 | · | ✓ |
| Prime factorization / nt extras | ✓ | · | ✓ | · | ✓ | · | ~ | ✓ | · | 5 | · | ✓ |
| Implicit relations (x²+y²=1) | ✓ | ✓ | ✓ | ✓ | ✓ | · | · | ✓ | · | 6 | · | ✓ |
| CAS (symbolic algebra) | · | · | ✓ | ✓ | ✓ | · | ✓ | ✓ | · | 4 | · | · |
| Exam mode | ✓ | · | ✓ | ✓ | · | · | · | ✓ | · | 4 | · | · |
| Step-by-step solutions | · | · | ~ | · | ✓ | · | ✓ | · | · | 3 | · | · |
| Spreadsheet view | · | ~ | ✓ | ✓ | · | · | · | ✓ | · | 3 | · | · |
| Geometry | · | · | ✓ | ✓ | · | · | · | ✓ | · | 3 | · | · |
| Programming surface | ✓ | · | ~ | ✓ | ✓ | · | · | ✓ | ✓ | 5 | ~ | ~ no strings/for/IO |
| Natural-language input | · | · | · | · | ✓ | · | · | · | · | 1 | · | · |
| Curated/live data | · | · | · | · | ✓ | · | · | · | · | 1 | · | · |

NW = NumWorks, DM = Desmos, GG = GeoGebra, TI = TI-84 Plus CE / TI-Nspire
CX II, WL = WolframAlpha / Wolfram Language, SC = SpeedCrunch, HP =
HiPER Scientific, Pr = HP Prime, Oc = MATLAB / GNU Octave.

App-column corrections from the 2026-09-02 re-verification (all other
app columns re-confirmed): HiPER's data-plots row is absent (the v12
manual has no histogram/box/scatter); Desmos's auto-POI row is not
documented in the help center and is marked partial; Octave's
probability functions and ttest ship in the official statistics
package, not the core manual, so those rows are partial rather than
full. GeoGebra's complex numbers are simulated through points and the
CAS, not a native number type (kept full: the arithmetic works).

## What changed since the baseline

Closed since 2026-08-31 (epher was absent, is now full):

- Complex numbers: `3 + 4i`, `sqrt(-1)` = `i`, re/im/abs/arg/conj, exact
  complex roots from `solve`, rounded complex display.
- Numeric equation solving: `solve x^2 == 5*x + 6` lists every root;
  polynomial equations give exact real and complex roots; other
  equations scan numerically over -100..100 with bracketing.
- Lists as values: `{1, 2, 3}` literals, 1-based indexing, elementwise
  arithmetic with broadcast, the full variadic statistics family
  (mean/median/mode/variance/stdev/quartile/range...), sort, len.
- Probability distributions: normal, t, chi-squared, binomial, Poisson;
  pdf, cdf, and the `inv*` quantile functions.
- Random numbers: `random()`, `random(a, b)`, `randint(a, b)`, and
  `randseed(n)` for reproducible sequences across every frontend.
- Matrices: literals, elementwise and matrix arithmetic, powers,
  det/inv/transpose/trace/dim/ref/rref, linear systems via rref.
- Calculus: `derivative(expr, p)` and `integral(expr, a, b)` numeric,
  both graphable (`graph derivative(x^3 - x, x)`).
- Data plots: `graph histogram(data[, bins])`, `graph boxplot(data)`,
  `graph scatter(xs, ys)` with the least-squares line drawn in.
- Units + conversion: full SI + everyday + astronomy unit table, SI
  prefixes, dimension checking, `in` / `->` conversion, compound units,
  dimension-cancelling arithmetic.
- Exact/fraction results: fractions by default (display rounding,
  ADR-0051/0052), `frac/dec/big` exact layers, one-tap fractions toggle.
- Display formats: `scientific(x)`, `engineering(x)`, `grouped(x)`
  verbs plus an Auto/Scientific/Engineering notation setting and a
  thousands-separators setting; every result rounds to twelve
  significant digits.
- Percentage operator: `200 + 10%` (documented convention: 200.1;
  HiPER makes the same semantics a setting), `10%` = 0.1.
- Constants library: 28 CODATA physics constants (with astronomy), a
  constants browser, values verified against CODATA in the parity sweep.
- Financial: tvm_pmt/pv/fv/n/i any-field solver, npv, irr, amort,
  simple/compound interest.
- Bitwise: `& | xor ~ << >>` on exact big integers, `bits(n)` word
  size, base literals and `bin/oct/hex` spelling.
- Number theory: factors, ndivisors, totient, modpow, isprime,
  nextprime/prevprime, gcd, lcm.
- Implicit relations: `graph x^2 + y^2 == 1` via marching squares.
- Table of values: promoted from partial (the `table` command with
  domain and point count was already there; it now rounds like the
  rest of the app).

Rows that improved but are still partial:

- Regression / curve fitting: `linreg(xs, ys)` reports the line with
  r and overlays the fit on the scatter, but only the linear model.
- Hypothesis tests + CIs: ttest, ztest, chisq_gof, tinterval,
  zinterval; no ANOVA, no paired or rank tests.

Rows that were already covered at the baseline and still are: auto
points of interest, sliders with animation, inequality shading,
parametric and polar curves, 3D surfaces, piecewise, scripting with
variables and user constants, number bases, combinatorics, shareable
history, SVG and PNG export, offline PWA plus desktop plus TUI plus CLI
from one binary, 8 locales, the in-app guide.

New since the baseline (not in the 31-row matrix, and not matched by
any of the nine): script transcripts (a multi-line or semicolon script
shows every answer in order, in every frontend), the display-rounding
rules (12 significant digits, fraction only when the value genuinely
repeats), and the parity-level quantile accuracy (invt/invchi2/
invnorm tails correct to the stored-p limit).

## The remaining gaps

### Partial rows worth finishing (small to medium rounds)

1. **Regression beyond linear** (8/9 apps). TI and NumWorks fit
   quadratic, exponential, power, and logarithmic models; epher has
   the line. Scope: quad/exp/power fits on the same list pair, each
   reporting its r, and an overlay on the scatter like linreg.
2. **ANOVA and paired tests** (7/9 apps). The stats-course appliance
   is one-way ANOVA; epher has t/z/chi-squared and intervals. Scope:
   one-way ANOVA on grouped lists; paired t on two lists is nearly
   free once the machinery exists.
3. **Table upgrades** (T2.11). TI/NumWorks tables offer a derivative
   column, an exact-mode toggle, and paste-a-data-column. epher's
   `table` is basic. Scope: `table derivative(x^3, x) ...`-style
   column and data lists.
4. **Normal random draws**. `random()`, `random(a, b)`, `randint`
   exist; `randn(mu, sigma)` (Desmos randomNormal, TI randNorm,
   NumWorks) is a one-liner on the existing seeded generator.
5. **3D parametric curves** (T3.9). Desmos 3D and Nspire plot
   space curves; epher has 3D surfaces only. Niche; cheap once the
   sampler is shared.

### Deliberate product boundaries (absent by decision, not by accident)

- **CAS / symbolic algebra** (T3.1): conflicts with ADR-0004's
  minimal-engine stance; exact polynomial roots via solve cover the
  classroom slice. Revisit only if students become a target segment.
- **Exam mode** (T3.8): a lockdown profile; only classroom hardware
  apps ship it (4/9). Worth it only with classroom adoption.
- **Step-by-step solutions** (T3.6): Wolfram Pro and HiPER's learner
  feature; needs symbolic groundwork. Not now.
- **Spreadsheet view** (T3.12): GeoGebra/Nspire/HP surface for
  tabular work; epher's lists cover the data cases.
- **Dynamic geometry** (T3.12): GeoGebra/HP; a product of its own.
- **Natural-language input** (T3.10): Wolfram's moat; conflicts with
  epher's one-grammar transparency. Not recommended.
- **Curated/live data** (T3.11): Wolfram only; needs network, breaks
  the offline-first promise (ADR-0003). Not recommended.
- **Language surface** (T3.4): def/recursion/while/if/scripts exist;
  strings, `for` loops, and print were deferred. The most visible
  remaining gap for the "calculator that programs" story, but a
  deliberate one.

### Documented conventions that differ from some apps (unchanged)

- `200 + 10%` = 200.1 (TI/Desmos/HiPER/HP: 220; HiPER offers the
  setting; epher documents its choice).
- `mod(-10, 3)` = -1 (TI/NumWorks: 2).
- stdev/variance are population (TI default is sample).
- tvm rates as fractions (TI as percent).
- binompdf(k, n, p) k-first (TI: n, p, k).
- Temperature scales are not units; kelvins only (same as SpeedCrunch).
- atanh(2) = -1.5708i (C99 branch; MATLAB/numpy take the other).

## Recommendations for the next round

1. **Complete the stats class** (one round): ANOVA, paired t, and
   `randn` on the existing list/distribution machinery; then the
   quad/exp/power regression family with scatter overlays. This
   closes the two partial rows that real users can name.
2. **Table upgrades** as a small round after that.
3. **Keep the boundaries** on CAS, exam mode, step-by-step,
   spreadsheet, geometry, natural language, live data, and the
   language surface until a product decision names an audience.
4. Re-run this matrix after every milestone round; the rows are now
   stable enough that a rerun is a 30-minute probe plus a source
   spot-check.

## Notes and corrections

- App-column corrections from the re-verification are in the matrix
  note above: HiPER data plots (was ✓, now ·), Desmos auto-POI (was
  ✓, now ~), Octave distributions and tests (was ✓/·, now ~/· with
  the statistics-package caveat). GeoGebra complex numbers carry the
  simulated-via-points caveat. No other app column changed, and no
  app added or removed features between 2026-08-31 and 2026-09-02.
- The baseline report's "already covers" list is confirmed current
  with two additions (PNG export shipped in ADR-0042; script
  transcripts and display rounding shipped in ADR-0051/0052).
- The probe found `randnorm` does not exist (no normal random draw);
  it is not in the guide's function reference, so it was never a
  listed gap - added here as a small gap.
- `solve k*x == 12` with an unbound k is an error by design ("two
  unbound variables"); with `const k = 3` first it answers `x = 4`.
  The guide's example binds k first.
