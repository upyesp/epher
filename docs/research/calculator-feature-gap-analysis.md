# Calculator feature-gap analysis: epher vs the apps people know

Date: 2026-08-31 · Status: recommendations only, nothing implemented

## Why this exists

epher's feature set grew from its own design decisions (ADR-0001 onward).
This report checks it against what users have come to expect from the
calculators they already use, so that picking up epher does not feel like
losing capabilities. Nine popular apps/websites were researched against
primary sources (official manuals, guidebooks, reference docs - no reviews),
one catalog each. The nine catalogs are the evidence base; this file is the
synthesis and the recommendation list. Each accepted recommendation would
get its own ADR and round.

## Sources

Per-app catalogs (primary-source URLs cited inside each):

| # | App | Catalog | Principal primary sources |
|---|-----|---------|---------------------------|
| 1 | NumWorks | `/tmp/epher-research/01-numworks.md` | numworks.com manual (HTML + book.pdf) |
| 2 | Desmos | `/tmp/epher-research/02-desmos.md` | help.desmos.com Help Center API (85 articles) |
| 3 | GeoGebra | `/tmp/epher-research/03-geogebra.md` | geogebra.org/manual (command/algorithm pages) |
| 4 | TI-84 Plus CE Python / TI-Nspire CX II | `/tmp/epher-research/04-ti.md` | TI guidebooks + 84CE reference PDF + Nspire eGuide |
| 5 | WolframAlpha / Wolfram Language | `/tmp/epher-research/05-wolfram.md` | wolframalpha.com about/examples + reference.wolfram.com guides |
| 6 | SpeedCrunch | `/tmp/epher-research/06-speedcrunch.md` | speedcrunch.org handbook sources (complete) |
| 7 | HiPER Scientific | `/tmp/epher-research/07-hiper.md` | hiperlabs.eu manual v12.0 PDF + Play listings |
| 8 | HP Prime | `/tmp/epher-research/08-hpprime.md` | HP Prime User Guide 3rd ed. (HP copyright, via hpcalc.org) |
| 9 | MATLAB / GNU Octave | `/tmp/epher-research/09-octave-matlab.md` | docs.octave.org latest manual (mathworks.com unreachable, 403) |

The catalogs are not committed to the repo (they are working notes with
scratch fetches); this file is the durable record.

## What epher already covers (do not re-propose)

Verified against the v0.5.4 binary and guide:

- **Parameter sliders with animation** - `const a = 1` + `graph a*x^2`
  makes a slider; every slider has a play button (Desmos's signature loop).
- **Points of interest** - roots, turning points, and curve intersections
  are auto-marked and listed after every graph command (the thing every
  graphing-calculator manual calls "trace analysis" anchors).
- **Trace**, wheel/pinch zoom, pan, per-curve legend checkboxes, inequality
  shading (`graph y < x^2`), parametric + polar + domain-restricted plots,
  3D surfaces with orbit/spin.
- **Function tables** - `table x^2 from -2 to 2 points 5`.
- **Piecewise** - `graph if x < 0 then -x else x` (if is an expression).
- Scripting language (def/recursion/if/while/scripts/comments), variables,
  user constants, shareable history links, save/load, SVG export, offline
  PWA + desktop + TUI + CLI from one binary, 8 locales, in-app guide.
- Exact arithmetic opt-in (`frac`, `dec`, `big`), number bases 2/8/16 both
  directions, combinatorics (fact/ncr/npr), variadic stats
  (sum/product/mean/variance/stdev/median).
- Astronomy unit suffixes, constants, Julian-date helpers, and a live
  offline solar system - **unique among the nine**: no researched app has
  a built-in orrery.

## The expectation matrix

✓ full · ~ partial/adjacent · · absent. Count = apps offering it of 9.

| Capability | NW | DM | GG | TI | WL | SC | HP* | Pr | Oc | epher |
|---|---|---|---|---|---|---|---|---|---|---|
| Complex numbers | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | · (0/9 absent - epher only) |
| Numeric equation solving | ✓ | ~ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | · |
| Lists as values | ✓ | ✓ | ✓ | ✓ | ✓ | · | ~ | ✓ | ✓ | · |
| Regression / curve fitting | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | · |
| Probability distributions | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | · |
| Random numbers | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | · |
| Matrices | ✓ | ~ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | · |
| Calculus (deriv/integral) | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ~ | · |
| Data plots (hist/box/scatter) | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | · |
| Units + conversion | ✓ | · | · | ✓ | ✓ | ✓ | ✓ | ✓ | · | ~ astro only |
| Exact/fraction results | ✓ | ✓ | ✓ | ✓ | ✓ | ~ | ✓ | ✓ | · | ~ opt-in only |
| Display formats (sci/eng/digits) | ✓ | ~ | · | ✓ | · | ✓ | ✓ | ✓ | ✓ | · |
| Table of values | ✓ | ✓ | ~ | ✓ | ~ | · | ✓ | ✓ | · | ~ no extras |
| Auto points of interest | ✓ | ✓ | ✓ | ✓ | ~ | · | ✓ | ✓ | · | ✓ |
| Parameter sliders + animation | ~ | ✓ | ✓ | ✓ | ~ | · | ~ | ✓ | · | ✓ |
| Inequality graphing | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | · | ✓ |
| Percentage operator | · | ✓ | · | · | ~ | ~ | ✓ | · | · | · |
| Hypothesis tests + CIs | ✓ | ✓ | ✓ | ✓ | ✓ | · | · | ✓ | · | · |
| Constants library (physics…) | ✓ | · | · | ~ | ✓ | ✓ | ✓ | ✓ | · | ~ astro only |
| Financial (TVM/NPV/IRR) | ~ | · | ✓ | ✓ | ~ | · | · | ✓ | · | · |
| Bitwise ops | ~ | · | · | ~ | ✓ | ✓ | ✓ | ✓ | ✓ | · |
| Prime factorization / nt extras | ✓ | · | ✓ | · | ✓ | · | ~ | ✓ | · | · |
| Implicit relations (x²+y²=1) | ✓ | ✓ | ✓ | ✓ | ✓ | · | · | ✓ | · | · |
| CAS (symbolic algebra) | · | · | ✓ | ✓ | ✓ | · | ✓ | ✓ | · | · |
| Exam mode | ✓ | · | ✓ | ✓ | · | · | · | ✓ | · | · |
| Step-by-step solutions | · | · | ~ | · | ✓ | · | ✓ | · | · | · |
| Spreadsheet view | · | ~ | ✓ | ✓ | · | · | · | ✓ | · | · |
| Geometry | · | · | ✓ | ✓ | · | · | · | ✓ | · | · |
| Programming surface | ✓ | · | ~ | ✓ | ✓ | · | · | ✓ | ✓ | ~ no strings/for/IO |
| Natural-language input | · | · | · | · | ✓ | · | · | · | · | · |
| Curated/live data | · | · | · | · | ✓ | · | · | · | · | · |

\* SC = SpeedCrunch, HP* = HiPER, Pr = HP Prime, Oc = Octave/MATLAB,
NW = NumWorks, DM = Desmos, GG = GeoGebra, TI = TI-84/Nspire, WL = Wolfram.

The pattern is stark: **everything in the top block is missing from epher
and present in at least seven of the nine.** That block is what "users
have come to expect" means in practice.

## Recommendations

### Tier 1 - table stakes (expected by essentially every audience)

**T1.1 Numeric equation solving.** `solve` is the most-expected feature
after arithmetic (8/9 apps; even SpeedCrunch users who lack it file it as
the top request in forum lore). Expectation shape (NumWorks/TI/HiPER):
type `x^2 - 5x + 6 == 0`, get **all** roots, exact-where-possible;
plus solve-for-any-variable and small linear systems. Scope for epher:
numeric root-finding over a domain (bracket + Newton, e.g. bisection
safeguard), polynomial root-finder for all-roots display; a `solve`
statement form fits the DSL. No CAS required.

**T1.2 Complex numbers.** 9/9 - the only capability every researched app
has and epher alone lacks. Expectation shape: `3 + 4i` parses; `sqrt(-1)`
returns `i` instead of an error (or is a mode); `re im abs arg conj`;
display as a+bi or polar; complex roots appear in equation solving (T1.1).
Scope: a complex Value variant; most built-ins extend mechanically; a
complex-format setting mirrors NumWorks/TI.

**T1.3 Units with conversion.** 6/9 - and both mobile scientifics users
actually carry (HiPER, SpeedCrunch, plus NumWorks, Nspire, HP Prime,
Wolfram). Expectation shape: quantities (`5 m`, `60 mile/hr`), a
conversion operator (`in`/`->`/`→`), SI prefixes, and dimension checking
(`5 m + 3 s` is an error). Scope: epher already owns the grammar
mechanism (astro suffixes prove it); generalize to a unit table with
dimensions, reusing SpeedCrunch's model (quantity reduction to SI) and
HiPER's CONV groups for the UI. Temperature (non-linear) can be
excluded like SpeedCrunch does.

**T1.4 Exact results by default / fraction display.** 7/9 keep or make
exact forms a first-class result (NumWorks shows exact + decimal
simultaneously). Expectation shape: `1/3` is presentable as a fraction;
a one-tap (or one-word) toggle decimal↔fraction on any result; mixed
numbers. Scope: display-level rational reconstruction (continued
fractions) plus promoting the existing frac/dec/big machinery; no engine
change needed for the toggle.

**T1.5 Percentage.** Small, but it is the canonical "my calculator app
does this" gesture (HiPER, Desmos keypad; SpeedCrunch deprecated theirs).
Expectation shape: `5%` → 0.05; `200 + 10%` add-on semantics decided and
documented (HiPER makes the semantics a setting); a Δ% helper is a bonus.

**T1.6 Lists as values, then data stats + linear regression.** 8/9 have
list literals; regression is 8/9. This cluster is the stats-class and
data-entry expectation: a named column of numbers, `mean(a)`/`stdev(a)`
over it, `linreg(x, y)` with r and a fitted-curve overlay on the scatter.
Scope: list Value + literals `{1, 2, 3}` (NumWorks spelling) or `[1, 2, 3]`
(Octave/GeoGebra spelling - decide one), element access, map/filter
ergonomics; regression models first linear (then quad/exp/power), storing
fitted functions as ordinary epher functions. This is the largest Tier 1
item; ship it as lists first, regression second.

### Tier 2 - strong differentiators (graphing/scientific audiences)

**T2.1 Calculus helpers (numeric).** Derivative at a point, `d/dx` as a
graphable expression, definite integral (shaded area under the curve like
Desmos/HiPER). 8/9 have some form. Numeric only (adaptive Simpson/Gauss-
Kronrod) keeps ADR-0004's spirit; the graph integration reuses the
existing POI/legend machinery.

**T2.2 Probability distributions + seeded random.** 8/9. Normal, t, χ²,
binomial, Poisson (PDF/CDF/inverse) as functions; `random()`,
`randint(a, b)`, seed control. SpeedCrunch proves even a "small" calculator
is expected to ship the discrete three.

**T2.3 Matrices.** 8/9. Literal entry, `det inv transpose rref`, solving
linear systems; eigenvalues later. NumWorks's minimal set
(inverse/det/transpose/trace/dim/ref/rref) is the right floor. Scope:
matrix Value + arithmetic + the six functions; linear solve via rref.

**T2.4 Graph-analysis parity.** epher marks roots/turning points/intersections
but not: derivative value at the cursor, tangent line at a point, signed
area / area between curves, integral shading. All six graphing apps
with analysis tools have these; they complete the trace experience.

**T2.5 Display formats.** Decimal/scientific/**engineering** notation,
significant-digit control, thousands separators (6/9). A settings panel +
`format` verbs; engineers expect `1.234 5×10⁶` spelled with SI-ish
exponents (HiPER's "engineering SI").

**T2.6 Scientific constants library.** 5/9 ship 120-150+ constants
(CODATA-backed). epher has ~15, astronomy-weighted. Add the standard
physics/chemistry set with a constants browser (SpeedCrunch's Ctrl+Space
pattern). Cheap, high homework value.

**T2.7 Data plots.** Histogram (adjustable bins), box-and-whisker,
scatter from data (7/9). Depends on T1.6 lists. The plot commands can be
graph-family members: `graph histogram(data)`.

**T2.8 Bitwise operations.** and/or/xor/not/shifts on `big` integers +
word-size/signedness settings (5-6/9; SpeedCrunch, HiPER, HP Prime, Octave).
Programmer-mode staple; epher already has the base literals.

**T2.9 Number-theory extras.** Prime factorization, divisors, is-prime,
modular power (4/9, and NumWorks exposes factorization as a one-tap
"additional result" on any integer). Cheap given `big`; high
homework value; pairs with T1.4's "additional results" idea.

**T2.10 Implicit relations.** `x^2 + y^2 == 1` plotted as a curve (5/9:
Desmos, GeoGebra, Nspire relations, HP Advanced Graphing, NumWorks
conics). Scope: marching squares on the same sampler; inherits legend/
export for free.

**T2.11 Table upgrades.** Derivative column, exact-mode toggle, data
tables (paste a column), plot-table-points-as-scatter (the TI/NumWorks
table experience vs epher's basic table).

### Tier 3 - specialists (only if epher wants those audiences)

**T3.1 CAS / symbolic algebra.** Solve symbolically, expand/factor/
simplify, symbolic derivatives/integrals/limits (5/9; the defining HP
Prime/Nspire-CAS/GeoGebra feature). Largest possible scope; conflicts
with ADR-0004 minimalism unless staged as an exact-polynomial module on
the existing big/frac machinery. Recommend: not now; revisit if students
become a target segment.

**T3.2 Financial functions.** TVM solver (any-field), NPV/IRR,
amortization (TI, HP Prime, NumWorks simple/compound, GeoGebra commands).
Self-contained function family; no engine work; moderate i18n burden.
Good "business users" magnet.

**T3.3 Hypothesis tests + confidence intervals.** 6/9; the stats-course
appliance (z/t/χ²/ANOVA + intervals with wizards). Depends on T2.2
distributions. Big UI surface (editors for each test).

**T3.4 Language surface: strings, for loops, print.** The scripting DSL
lacks strings, for loops, and output - the three things TI-Basic/PPL/
Octave programmers reach for first. Also: multiple return values,
select-to-evaluate. Positions epher better as the "calculator that
programs" (NumWorks Python is a major draw).

**T3.5 REPL/entry UX.** Autocomplete (SpeedCrunch, NumWorks editor,
Octave), F1 context help on the function under the cursor, live
as-you-type evaluation, auto-`ans` when a line starts with an operator
(NumWorks/SpeedCrunch). Low-risk quality features; the auto-ans one is
tiny and delightful.

**T3.6 Step-by-step solutions.** Wolfram (Pro) and HiPER's defining
learner feature. Requires symbolic groundwork (T3.1) for anything beyond
arithmetic-style steps. Not now.

**T3.7 Export/share extras.** PNG export, printable session export
(HTML), embeddable graph links (Desmos). epher has SVG + share links;
PNG is the cheap addition (canvas render on export click).

**T3.8 Exam mode.** 4/9 (all classroom hardware). A lockdown profile for
the PWA/desktop (disable history/constants/scripts, timed, watermark).
Only worth it if classroom adoption becomes a goal.

**T3.9 3D parametric curves and vector fields.** Nspire/Desmos-3D/WL
territory; epher's 3D is surfaces-only. Nice later; niche now.

**T3.10 Natural-language input.** Wolfram's moat; a huge, open-ended NLP
surface that also conflicts with epher's "one grammar" transparency.
Not recommended.

**T3.11 Curated/live data (currency, weather, stocks).** Wolfram only;
requires network and violates epher's offline-first promise (ADR-0003).
Not recommended as a core feature.

**T3.12 Everything-else category.** Dynamic geometry (GeoGebra/HP),
spreadsheet view (HP/GeoGebra/Nspire), periodic table (NumWorks), RPN
mode (HiPER/HP), OCR photo input (HiPER premium; server round-trip),
sonification (Desmos), package ecosystem (Octave). Each is a product of
its own; none is a general "calculator expectation".

## Suggested sequencing (if approved in batches)

1. **Quick wins** (days each): T1.5 percentage, T2.6 constants,
   T2.9 number-theory extras, T3.5 auto-ans + autocomplete + help,
   T3.7 PNG export.
2. **Core block** (one round each): T1.1 solve, T1.2 complex,
   T1.4 exact-by-default toggle, T2.5 formats, T2.1 calculus numeric.
3. **Data platform** (2-3 rounds): T1.6 lists → stats/regression →
   T2.7 data plots → T2.11 table upgrades → T3.3 tests.
4. **Units** (1-2 rounds): T1.3 unit system (grammar exists), then
   T2.8 bitwise as the parallel programmer track.
5. **Deferred pending direction**: T2.10 implicit, T2.3 matrices (large
   but well-understood), T3.2 finance, then the T3.1/T3.6/T3.10 class.

## Corrections to the working inventory

The brief given to the researchers (`/tmp/epher-research/EPHER-INVENTORY.md`)
listed "no tables of values" - wrong: epher's `table` command exists
(function tables with domain and point count). Two researchers' notes
propagate that error (04-ti, 02-desmos gap lists). Sliders were correctly
attributed in the guide but are easy to miss - epher's const-slider +
animation loop is at parity with the Desmos interaction. This file's
"already covers" section is the corrected record.
