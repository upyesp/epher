# Calculator gap analysis, third pass: epher after the stats-class round

Date: 2026-09-06 · Status: findings only, nothing implemented

## Why this third pass

The baseline (docs/research/calculator-feature-gap-analysis.md, 2026-08-31)
found epher v0.5.4 absent on 21 of the 31 expectation rows. The rerun
(docs/research/calculator-gap-analysis-rerun.md, 2026-09-02) re-measured
after nine work packages and found 24 of 31 covered (21 full, 3 partial),
with seven rows absent by decision. ADR-0054 shipped the same day as the
rerun and closed the two partial rows real users could name; ADRs 0055
through 0061 followed through v0.5.37. This third pass re-measures the same 31 rows
against epher v0.5.37 and re-verifies the nine app columns against primary
sources, four days after the rerun's re-verification.

## Executive summary

- **epher v0.5.37 covers 24 of the 31 rows (23 full, 1 partial), up from
  24 (21 full, 3 partial) at the rerun.** The two named partial rows
  closed: regression is now the full quad/exp/power/log family, and the
  tests row gained one-way ANOVA and the paired t. `randn` and the
  table extras (derivative column, data columns, exact/approx) closed
  inside rows that were already full.
- **The programming surface improved but stays the one partial row:**
  strings, `for` loops, and `print` shipped (T3.4 pulled forward), and
  comments joined the grammar; what remains is input/file I/O beyond
  `print`, single-value returns, and no string escapes.
- **The seven absent rows are unchanged and still deliberate:** CAS,
  exam mode, step-by-step, spreadsheet view, geometry, natural-language
  input, curated/live data.
- **No app gained or lost a matrix-level feature since 2026-09-02.**
  One distribution-channel change surfaced: the hipercalc.com web-app
  domain is parked for sale as of 2026-09-06; HiPER itself (manual v12.0
  at hiperlabs.eu) is unchanged, and the rerun's HiPER correction (no
  data plots) re-verifies. Desmos and HP Prime could not be re-fetched
  this pass (help center bot-blocked; hpcalc.org serves malformed gzip
  to automated clients) — their columns are carried from the 2026-09-02
  re-verification, not re-fetched.
- **Two defects found by probing, not in the matrix:** the CLI cannot
  parse `graph3d param` space curves (web and TUI can), and a persisted
  loop variable named `i` shadows the imaginary unit in later sessions —
  on this machine the installed store answered `3+4i` with `23` until a
  clean store was used.

## Method

- **epher column**: empirical. Every row was probed against the v0.5.37
  binary (`epher 0.5.37`, `/usr/bin/epher`) in one-shot and piped modes
  over probe scripts in /tmp/gap3/, with `EPHER_STORE_DIR` pointed at a
  clean store after the first probes caught the installed machine
  store's leftover `i = 5` variable (see Notes). Cross-checked against
  the in-app guide (site/guide/en.md, sections 1.7-1.30 and the quick
  reference) and the ADR record (0054-0061). UI-only behaviors (POI
  markers, sliders, keypad banks/drawer, export) were re-confirmed
  against the guide and the ADRs as in the rerun; the round web E2E
  suites were not re-shot.
- **App columns**: re-verified against the primary sources where the
  network allowed; two apps carried forward explicitly.

Sources checked 2026-09-06:

| # | App | Source this pass | Outcome |
|---|-----|------------------|---------|
| 1 | NumWorks | numworks.com/manual (HTML) + book.pdf, v21.0.0 | re-fetched; unchanged (regression, inference, distributions, exam mode, Python all present) |
| 2 | Desmos | desmos.com/scientific loads (200); help.desmos.com 403 (bot-blocked) | carried from 2026-09-02 re-verification, not re-fetched |
| 3 | GeoGebra | wiki.geogebra.org DNS down again; Wayback snapshot of the Manual fetched | partially re-fetched (CAS, Spreadsheet confirmed in snapshot); command-level claims carried from 2026-09-02 |
| 4 | TI-84 Plus CE / TI-Nspire CX II | education.ti.com guidebook search (200) | re-fetched; unchanged (TI-84 Plus CE Python, Nspire CX II present) |
| 5 | WolframAlpha / Wolfram Language | reference.wolfram.com ref/TTest, ref/RandomVariate (200) | re-fetched; unchanged |
| 6 | SpeedCrunch | speedcrunch.org/userguide + /reference/index.html (200) | re-fetched; unchanged |
| 7 | HiPER Scientific | hiperlabs.eu + manual v12.0 PDF (re-fetched); hipercalc.com parked | re-fetched; unchanged features (0 histogram/box/scatter hits in manual); web-app domain parked for sale |
| 8 | HP Prime | hpcalc.org unreachable for automated fetches (malformed gzip); no Wayback snapshot of the details page | carried from 2026-09-02 re-verification, not re-fetched |
| 9 | MATLAB / GNU Octave | docs.octave.org 11.1.0 + octave.sourceforge.io/statistics overview (200) | re-fetched; unchanged (ttest/anova still in the statistics package, not core) |

## The expectation matrix

✓ full first-class support · ~ partial or adjacent · · absent.
Count = apps of the nine offering the row (✓ or ~).

| Capability | NW | DM | GG | TI | WL | SC | HP | Pr | Oc | of 9 | epher v0.5.16 | epher v0.5.37 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Complex numbers | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | 9 | ✓ | ✓ |
| Numeric equation solving | ✓ | ~ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | 8 | ✓ | ✓ |
| Lists as values | ✓ | ✓ | ✓ | ✓ | ✓ | · | ~ | ✓ | ✓ | 8 | ✓ | ✓ |
| Regression / curve fitting | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | 8 | ~ linear only | ✓ |
| Probability distributions | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ~ | 8 | ✓ | ✓ |
| Random numbers | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | 8 | ✓ | ✓ |
| Matrices | ✓ | ~ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ✓ | 8 | ✓ | ✓ |
| Calculus (deriv/integral) | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | ~ | 8 | ✓ numeric | ✓ numeric |
| Data plots (hist/box/scatter) | ✓ | ✓ | ✓ | ✓ | ✓ | · | · | ✓ | ✓ | 7 | ✓ | ✓ |
| Units + conversion | ✓ | · | · | ✓ | ✓ | ✓ | ✓ | ✓ | · | 6 | ✓ | ✓ |
| Exact/fraction results | ✓ | ✓ | ✓ | ✓ | ✓ | ~ | ✓ | ✓ | · | 8 | ✓ default | ✓ default |
| Display formats (sci/eng/digits) | ✓ | ~ | · | ✓ | · | ✓ | ✓ | ✓ | ✓ | 7 | ✓ | ✓ |
| Table of values | ✓ | ✓ | ~ | ✓ | ~ | · | ✓ | ✓ | · | 6 | ✓ | ✓ |
| Auto points of interest | ✓ | ~ | ✓ | ✓ | ~ | · | ✓ | ✓ | · | 6 | ✓ | ✓ |
| Parameter sliders + animation | ~ | ✓ | ✓ | ✓ | ~ | · | ~ | ✓ | · | 6 | ✓ | ✓ |
| Inequality graphing | ✓ | ✓ | ✓ | ✓ | ✓ | · | ✓ | ✓ | · | 7 | ✓ | ✓ |
| Percentage operator | · | ✓ | · | · | ~ | ~ | ✓ | · | · | 4 | ✓ | ✓ |
| Hypothesis tests + CIs | ✓ | ✓ | ✓ | ✓ | ✓ | · | · | ✓ | ~ | 7 | ~ no ANOVA | ✓ |
| Constants library (physics…) | ✓ | · | · | ~ | ✓ | ✓ | ✓ | ✓ | · | 6 | ✓ | ✓ |
| Financial (TVM/NPV/IRR) | ~ | · | ✓ | ✓ | ~ | · | · | ✓ | · | 4 | ✓ | ✓ |
| Bitwise ops | ~ | · | · | ~ | ✓ | ✓ | ✓ | ✓ | ✓ | 7 | ✓ | ✓ |
| Prime factorization / nt extras | ✓ | · | ✓ | · | ✓ | · | ~ | ✓ | · | 5 | ✓ | ✓ |
| Implicit relations (x²+y²=1) | ✓ | ✓ | ✓ | ✓ | ✓ | · | · | ✓ | · | 6 | ✓ | ✓ |
| CAS (symbolic algebra) | · | · | ✓ | ✓ | ✓ | · | ✓ | ✓ | · | 4 | · | · |
| Exam mode | ✓ | · | ✓ | ✓ | · | · | · | ✓ | · | 4 | · | · |
| Step-by-step solutions | · | · | ~ | · | ✓ | · | ✓ | · | · | 3 | · | · |
| Spreadsheet view | · | ~ | ✓ | ✓ | · | · | · | ✓ | · | 3 | · | · |
| Geometry | · | · | ✓ | ✓ | · | · | · | ✓ | · | 3 | · | · |
| Programming surface | ✓ | · | ~ | ✓ | ✓ | · | · | ✓ | ✓ | 5 | ~ no strings/for/IO | ~ no input/IO |
| Natural-language input | · | · | · | · | ✓ | · | · | · | · | 1 | · | · |
| Curated/live data | · | · | · | · | ✓ | · | · | · | · | 1 | · | · |

NW = NumWorks, DM = Desmos, GG = GeoGebra, TI = TI-84 Plus CE / TI-Nspire
CX II, WL = WolframAlpha / Wolfram Language, SC = SpeedCrunch, HP =
HiPER Scientific, Pr = HP Prime, Oc = MATLAB / GNU Octave.

The of-9 counts are unchanged from the rerun for every row: no app gained
or lost a feature between 2026-09-02 and 2026-09-06. (ADR-0054's
consequence note phrased the two closings as "8/9 → 9/9" and "7/9 → 9/9";
those numerators count epher's own coverage, not the nine apps — SpeedCrunch
still has no regression or tests, so the app counts stay 8 and 7.)

## What changed since the rerun

Closed since 2026-09-02 (partial became full):

- **Regression / curve fitting**: `quadreg`, `expreg`, `powreg`, `logreg`
  join `linreg` (ADR-0054), each reporting its model with the
  linearized-pair correlation r — the same number TI and NumWorks
  report — and each drawable over its scatter:
  `graph scatter(xs, ys, quadreg)`.
- **Hypothesis tests + CIs**: `anova(list1, list2, …)` (one-way F with
  p, unequal lengths welcome) and `ttestpaired(a, b)` join ztest, ttest,
  chisq_gof, and the intervals (ADR-0054). The stats-class row is at the
  TI/NumWorks floor now.

Closed inside already-full rows:

- **`randn(mu, sigma)`** — the normal draw the rerun's probe missed
  (Desmos randomNormal, TI randNorm), Box-Muller on the seeded stream,
  seed-reproducible like the uniform draws.
- **Table extras** — `table f(x) ... derivative g(x)` third column,
  `values <list>` data columns, `exact`/`approx` per-table display
  override (ADR-0054). The T2.11 scope from the baseline is fully in.

New language surface (changes the programming row's partial note):

- **Strings and print** (§1.7): `"…"` literals, `+` concatenation,
  `==`/`!=`, `len`, 1-based indexing; `str(x)` spells a value the way
  the answer panel does; `print(a, b, …)` joins with spaces. No escape
  sequences (a string cannot contain a double quote) — verified by probe.
- **`for` loops** (§1.10): `for i in a to b [step s] do body` and
  `for x in list do body`, collecting the body's values into a list
  (a comprehension by construction); the loop variable keeps its last
  value afterwards, like TI's For; 100,000-step runaway guard. Negative
  steps count down (probed: `{5, 4, 3, 2, 1}`).
- **Comments** (§1.13): `//` and `#` to end of line, `/* … */` blocks —
  shipped with the scripts collection's needs.
- Still missing for the row: input/file I/O beyond `print`, multiple
  return values, richer string operations. Hence the row stays partial:
  "~ no input/IO".

New since the rerun outside the matrix (none of it matched by all of the
nine; most of it matched by none):

- **3D parametric space curves**: `graph3d param cos(t), sin(t), t` over
  a t domain (ADR-0054) — the T3.9 niche gap closed on web/desktop/TUI.
  CLI caveat in the Notes below.
- **The keypad banks and drawer** (ADR-0055, ADR-0060): the PWA's on-screen
  keypad covers the whole language in banks (123, trig, ƒ, nΣ, data, 0x,
  dist, $, π∇, ☉) and docks away on a grab bar; the TUI gained the same
  banks (Tab focuses them). Every function in the guide is on a key.
- **Answer copying** (ADR-0057): a copy icon puts the answer's values on
  the clipboard, one per line; long answers render in the result pane
  (ADR-0056).
- **Graph controls** (ADR-0055): line-thickness slider per plot kind,
  zoom slider on every plot, rotation-speed sliders on 3D/solar plots,
  legend checkboxes that hide the curve and its points of interest.
- **The scripts collection rides the installers** (ADR-0058): all 333
  scripts ship in every artifact (verified on this machine at
  /usr/lib/epher/scripts/), the guide's copy-paste commands name their
  operating system, and the website's Scripts page browses the same
  collection.
- **The astronomy scripts collection** (31 scripts: moon, planets, sky,
  time — including full-moons.epher, a Meeus ch. 49 implementation that
  runs end-to-end, probed): single-location under `epher scripts/astronomy/`,
  observing from Greenwich by default.
- **Linux installs from apt/dnf/flathub/snap/aur** (ADR-0061) and
  **TUI paste-as-one-paste** (ADR-0059).
- REPL/entry UX from the baseline's T3.5 is effectively closed:
  autocomplete with per-function descriptions, F1 help under the cursor,
  and auto-`ans` on operator-first lines are in every frontend.

## The remaining gaps

### The one partial row

1. **Programming surface** (5/9 apps offer one). The three named gaps
   (strings, `for`, print) shipped; what keeps the row partial is input
   and file I/O beyond `print`, single-value returns, and no escape
   sequences or string library to speak of. TI-Basic and NumWorks Python
   both have input; Octave and PPL have I/O and rich string functions.
   Small rounds would close the first two; the string library is a
   product decision.

### Deliberate product boundaries (unchanged, absent by decision)

- **CAS / symbolic algebra** (4/9): expand/factor/simplify are unknown
  names (probed); `derivative` stays numeric; polynomial `solve` gives
  exact roots only where the roots are rational or simple complex
  (`x^2 == 2` answers decimals). Still the right boundary per ADR-0004.
- **Exam mode** (4/9): classroom hardware only.
- **Step-by-step solutions** (3/9): Wolfram Pro and HiPER's learner
  feature; needs the symbolic groundwork.
- **Spreadsheet view** (3/9): GeoGebra/Nspire/HP surface.
- **Dynamic geometry** (3/9): GeoGebra/HP; a product of its own.
- **Natural-language input** (1/9): Wolfram's moat.
- **Curated/live data** (1/9): Wolfram only; needs network.

### Defects found by probing (not matrix rows)

1. **The CLI cannot parse `graph3d param`.** `epher 'graph3d param
   cos(t), sin(t), t'` fails with "parse error: unexpected trailing
   input" in one-shot and piped modes: the shell's `submit_surface`
   (crates/shell/src/plots.rs) never consults
   `parse_space_curve_source`, while the web frontend (crates/web/src/
   lib.rs:3990) and the TUI (crates/tui/src/lib.rs:1998) do. Surfaces
   and 2D param curves parse fine in the CLI. One routing branch closes
   it.
2. **A persisted variable named `i` shadows the imaginary unit.** The
   rerun noted for loops leave their variable bound; the session
   snapshot then carries it across sessions (§3.3's shared store). On
   this machine the installed store held `i = 5` from an earlier loop,
   so `epher '3+4i'` answered `23` and `im(3+4i)` answered `0` until a
   clean `EPHER_STORE_DIR` was used. `3+4i` in a clean store is the
   complex literal. The guide says `i` works "exactly like `pi`" — but
   `pi` cannot be shadowed by a bare assignment, so the honest note is:
   the imaginary unit is shadowable, `pi` is not, and a loop named `i`
   is a footgun that survives restarts.

### Documented conventions that differ from some apps (unchanged)

- `200 + 10%` = 200.1 (probed); `200 * (1 + 10%)` = 220.
- `mod(-10, 3)` = -1 (probed).
- stdev/variance are population (`stdev({2, 4})` = 1, probed).
- tvm rates as fractions; binompdf(k, n, p) k-first; temperature scales
  not units; atanh(2) takes the C99 branch (probed:
  0.549306144334-1.57079632679i).
- `solve k*x == 12` with unbound k stays a typed error (probed); with
  `const k = 3` it answers `x = 4`.

## epher's unique strengths (re-checked, all hold)

- **The offline solar system** — still the only orrery among the nine,
  now surrounded by a 31-script astronomy collection that ships inside
  every installer and by reference scripts (full-moons.epher) that teach
  the Meeus algorithms in epher's own grammar.
- **The scripts collection as a first-class artifact** — 333 scripts in
  nine categories, browsed on the website, carried by every installer,
  with copy-paste commands that name their operating system. None of the
  nine ships its examples as runnable, categorized programs beside the
  binary.
- **Script transcripts, seeded reproducible randomness across every
  frontend, and the byte-identical 8-locale guide** — still unmatched.
- **A keypad that covers the whole language** in banks, in the PWA and
  the TUI alike — the hardware-calculator keypad experience on a web
  app, without abandoning the one-grammar entry line.
- **One binary, five frontends** (desktop, PWA, TUI, CLI, REPL) with one
  shared live store — still beyond any of the nine, whose web, desktop,
  and hardware variants do not share sessions.

## Notes and corrections

- **App-side change**: hipercalc.com (the HiPER Calc web app URL) is
  parked for sale as of 2026-09-06. HiPER's manual (v12.0,
  hiperlabs.eu/assets/raw/hipercalc-manual-12.0.pdf) is unchanged and
  re-verifies the rerun's corrections: no histogram/box/scatter, and
  regression present. No matrix cell changes.
- **Carried forward, not re-fetched**: Desmos (help.desmos.com 403 to
  automated clients; the scientific page itself loads but is a JS
  shell) and HP Prime (hpcalc.org serves malformed gzip to automated
  clients; the Wayback availability API holds no snapshot of the
  details page). Both columns are the 2026-09-02 re-verification,
  unchanged.
- GeoGebra's wiki was DNS-dead from this network again (as in the
  rerun); the Wayback snapshot of the Manual page confirms the CAS and
  spreadsheet rows, and the command-level claims carry from 2026-09-02.
- The machine-store incident (`i = 5`) is worth a regression test: the
  first probes of this pass were run against the installed
  `~/.epher` store and returned `23` for `3+4i`. All matrix probes use
  a clean store; both behaviors are reported above.
- The probe files live in /tmp/gap3/ (probe1-4.epher, p5.epher,
  c.epher, plus the fetched source pages) and are working files, not
  committed; this report is the durable record.
