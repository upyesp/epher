# Scripts audit: duplication of built-in functionality

**Date:** 2026-10-26 · **Scope:** all 477 scripts in `epher scripts/` (10 fields,
51 areas) checked against the built-in inventory in `site/reference.md` §10
(170 built-ins) and the unit names of §11 (51 units).

Trigger: `finance/loans/extra-payment-savings.epher` failed in Pete's app with
`error: cannot define constant payment: the name is already a variable`, then
printed garbage (step limit exceeded, 2.9e191 interest figures). The report
covers the incident, the scripts that hand-roll what the language already
provides, and the fleet-level hazards that made the incident possible.

---

## 1. The incident, root-caused

The script itself is correct in a fresh session (verified with
`epher-cli`: its output matches its expected-output block exactly). The failure
needs a stale session variable:

1. Four loan scripts bind `payment` as a plain **variable**
   (`payment = tvm_pmt(...)`): `mortgage-total-interest`,
   `mortgage-balance-after-years`, `balloon-loan`, `car-loan-payment`.
   Four others bind it as a **constant** (`const payment = ...`):
   `mortgage-term`, `mortgage-rate`, `amortization-schedule`,
   `extra-payment-savings`.
2. The PWA persists session state (ADR-0009), so a `payment` variable from an
   earlier script survives into later runs — even days later.
3. Reference §7.2: defining a constant that names an existing variable is the
   `cannot define constant …: the name is already a variable` error.
   Reproduced byte-for-byte:
   `payment = 437.31` + `const payment = 5` → the exact error line.
4. The run **continues** after the error (per-line semantics). `payment`
   keeps the stale value (437.31 from the earlier loan), so `boosted` became
   537.31 — below the ~1000/month interest on a 200 000 loan at 6 % — and the
   payoff loop never converges: step limit at 99 999, balance astronomic.

So Pete's "the script is attempting to recreate a function epher already
supports" is half right: the script *calls* `tvm_pmt` (the payment is not
hand-rolled), but it *does* hand-roll the payoff countdown that `tvm_n` and
`amort` cover (§2.1), and the collision that broke it is a real
fleet-consistency defect (§2.2).

## 2. Findings

### 2.1 Genuine duplication — refactor candidates (2 scripts)

| Script | Hand-rolls | Built-in(s) to use |
|---|---|---|
| `finance/loans/extra-payment-savings.epher` | while-loop payoff countdown (months, last payment, interest) | `tvm_n(r, pv, pmt, fv)` + the one-line closed form; `amort(p, r, n, k)` for balances |
| `finance/loans/amortization-schedule.epher` | `def bal(k) = principal*(1+r)^k − payment*((1+r)^k−1)/r` | `amort(p, r, n, k)` — verified identical to 12 digits (`bal(60)` = `amort(...,60)` = 234027.443604) |

The sibling scripts already do it right: `mortgage-balance-after-years` calls
`amort()` for the same quantity, `mortgage-term` calls `tvm_n`, and
`extra-payment-savings` itself cites `tvm_n` in a closing comment without
using it.

**Verified refactor for `extra-payment-savings`** (reproduces every printed
value; the closed form is actually more accurate than the loop — the loop's
last payment differs in the 10th digit):

```
const months = ceil(tvm_n(rate / 12, -principal, boosted, 0))
def bal(k) = principal * (1 + rate/12)^k - boosted * ((1 + rate/12)^k - 1) / (rate/12)
const last = bal(months - 1) * (1 + rate / 12)
// interest with the extra: (months - 1) * boosted + last - principal
```

→ 295 months, last payment 602.261007745, interest saved 49138.4083124
(matches the script's expected output).

If the loop is kept for pedagogy (the header cites Kellison's amortization
recurrence), print a `tvm_n` cross-check line, as `distance-between-points-2d`
does with `hypot`.

### 2.2 Fleet inconsistency — the collision class (8 scripts)

The `payment` variable-vs-constant split (§1). With session persistence, any
two of these eight scripts run in one session can collide: variable → constant
is the error Pete hit; constant → constant with a different value is
`constant already defined` (§7.2, by design for re-pasting the *same* script).

**Options, in increasing scope:**
1. Make all eight bind the same way and give the name a script-specific
   spelling (`loan_payment`) — no collision, no language change.
2. App-level: run each script's knob section in a child scope, or offer
   "run in fresh session" — fixes the whole class for every future script.

### 2.3 Name-shadowing — bindings that collide with built-ins (13 scripts)

Legal today (only `i` is reserved, §7.1), but with session persistence a
shadowing variable poisons later runs that call the real built-in
(e.g. a variable `exp` makes a later `exp(2)` a type error). Variable
bindings first — these persist:

| Script | Binding | Shadows |
|---|---|---|
| `astronomy/coordinates/aberration-fix.epher` | `az = …` (apex vector z-component) | `az(body, jd, lat, lon)` |
| `astronomy/coordinates/apparent-place.epher` | `az = …` (apex vector z-component) | `az(…)` |
| `astronomy/ephemeris/schlyter-crosscheck.epher` | `im = …` (inclination) | `im(z)` |
| `astronomy/moon/moonrise-azimuth.epher` | `date = …` | `date(jd)` |
| `astronomy/navigation/noon-sight.epher` | `dec = decl(10, lan)` | `dec(x)` |
| `finance/investing/breakeven-quantity.epher` | `fixed = …` (fixed costs) | `fixed(x, d)` |
| `statistics/tests/chi-square-goodness-of-fit.epher` | `exp = …` (expected counts) | `exp(z)` — riskiest: it also calls built-ins near the shadow |
| `astronomy/eclipses/*.epher` (4 scripts) | `def find(k, n)` (event scan) | `find(s, sub)` |

Function definitions (shadow only while defined, but confusing to read):

| Script | Definition | Shadows |
|---|---|---|
| `astronomy/sky/circumpolar-check.epher` | `def cosh(dec, phi)` | `cosh(z)` (different meaning: cos of horizon hour angle) |
| `astronomy/sky/shadow-length.epher` | `def len(a)` | `len(v)` |
| `astronomy/time/easter-sunday.epher` | `def iso(y, m, d)` | `iso(jd)` (could be `iso(jd(y, m, d))` modulo the `T00:00:00` tail) |
| `calculus/models/radioactive-half-life.epher` | `def frac(t)` | `frac(n, d)` |
| `astronomy/sun/analemma-point.epher` | `def dec(t)` | `dec(x)`; *also* partially duplicates `decl(10, jd)` (Sun declination), though the script needs it from its own solar model for self-consistency |
| `astronomy/moon/phases-of-the-month.epher` | `def phase(k)` (phase *date*) | `phase(body, jd)` (phase *angle*) — different questions, same name |
| `astronomy/sky/eyepiece-planner.epher` | `def mag(ep)` (magnification) | `mag(body, jd)` (apparent magnitude) |
| `geometry/coordinates/*.epher` (4 scripts) | `def dist(p, q)` | `dist(body, jd)` (fine as a cross-check — the script prints `hypot agrees: true` — but the name collides) |

Renaming suggestion: `azimuth`, `incl`, `obs_date`, `sun_dec`, `fixed_cost`,
`expected`, `cosH`, `shadow_len`, `iso_date`, `frac_left`, `dist2d`. None of
these change results; all reduce future surprises.

### 2.4 Lesser observation — unit-name bindings (41 scripts)

Variables named like SI/derived units (`A`, `S`, `V`, `K`, `T`, `C`, `hr`,
`yr`, `lm`, `psi`, …) across algebra/matrices, geometry, astronomy and
calculus. Legal and currently unambiguous to the parser, but a readability
trap: `T = 2 * pi * sqrt(L / g)` reads as a tesla quantity. Low priority;
rename opportunistically.

### 2.5 Verified clean — by-design derivations that should stay

The fleet's stated convention is "formula and built-in cross-check", and it is
followed widely and well. Examined and confirmed clean:

- **Finance:** `mortgage-term` (`tvm_n`), `mortgage-rate` (`tvm_i`),
  `internal-rate-of-return` (`irr`), `net-present-value` (`npv`),
  `compound-interest` (`compound_interest`), `simple-interest`
  (`simple_interest`), `mortgage-balance-after-years` (`amort`),
  `chi-square-goodness-of-fit` (`chisq_gof`), `apy-from-nominal-rate` (no
  built-in twin exists).
- **Statistics:** `population-stdev` (teaches `stdev` + cross-check),
  `empirical-rule` (`normcdf`), `binomial-mean-variance` (theory, no twin),
  `two-sample-t-test` (Welch — **no built-in twin exists**; the engine has
  one-sample `ttest` and `ttestpaired` only. Candidate built-in request
  rather than a script defect), `dice-sum-simulation` (simulation loop, not a
  statistics re-implementation).
- **Astronomy:** `precession-j2000`, `equ-ecl`, `sundial-lines`
  (Meeus implementations with no engine twin; they *use* `jd`, `hms2deg`,
  `deg2hms`, `ra`, `decl`, `atan2` throughout), `oppositions-of-a-planet`
  (event search by sign change + bisection — no engine equivalent),
  `schlyter-crosscheck` (hand-rolls Kepler iteration *on purpose*: its stated
  job is "the classic low-precision model vs the engine"),
  `phases-of-the-month` (`def phase` returns phase *dates*, the built-in
  `phase(body, jd)` answers the phase *angle* — different questions),
  `planet-visibility-tonight` (`def hms` *wraps* `deg2hms`, fine).
- **Algebra/geometry:** `quadratic-formula` and siblings (formula +
  `solve` cross-check), `determinant-2x2`, `distance-between-points-2d`
  (`hypot` cross-check), `three-linear-equations` (`rref`), `cubic-roots`
  (`solve`), `newton-iteration-path` (graphs the method itself),
  `harmonic-number`, `infinite-geometric-sum`, `square-wave-harmonics`
  (series pedagogy, no built-in twins), `perfect-number-check` /
  `amicable-pair-search` (divisor *sums* — no built-in twin).

## 3. Method and coverage

- Built-in inventory parsed from `site/reference.md` §10 (170 callables) and
  §11 (51 units); grammar/rules from §7.
- Automated passes over all 477 scripts: (a) every top-level binding and
  `def` checked against both inventories; (b) concept-keyword scan
  (factorial, gcd, primes, taylor, riemann, julian date, kepler, amortization,
  …) with per-concept relevance filtering — scripts that call the relevant
  built-in anywhere in the body were not suspects. An earlier loose pass
  (335 hits) was tightened; substring noise ("mac**hin**ery",
  "algorit**hms**", "E**uclid**ean") was eliminated by word boundaries.
- All 477 headers read; ~35 suspect/candidate bodies read in full; every
  reported claim verified in source (and numerically where a formula was
  in question, via `epher-cli`).
- Known limits: judgment calls on "pedagogical vs incidental" are
  conservative — scripts that print a built-in cross-check were classed as
  by-design even when the built-in could replace the hand roll. Deep
  verification focused on finance, statistics and astronomy, where the
  built-in coverage is densest; algebra/geometry/trigonometry headers showed
  the cross-check convention uniformly.

## 4. Recommended actions, in order

> **Implementation status (2026-10-26, later the same day):** items 1–4 and 7
> are implemented in this tree — both refactors done and transcript-verified
> by `scripts/check-scripts.mjs` (477/477), the eight loan scripts renamed to
> per-script `const` bindings, all shadowing bindings renamed (the lint then
> caught four more the table above originally missed: the eclipse scripts'
> `def find`, `phases-of-the-month`'s `def phase`, `eyepiece-planner`'s
> `def mag` — all renamed; `day-length-year` and `prayer-times` were
> comment-prose false positives and need no change), and
> `scripts/lint-scripts.mjs` runs in `site-build.yml` before the site build.
> Item 5 (app-level session scoping) and item 6 (a Welch two-sample t
> builtin) are design decisions, deliberately left open.

1. Refactor `extra-payment-savings.epher` to `tvm_n` + closed form (recipe in
   §2.1), or keep the loop and add the printed `tvm_n` cross-check.
2. Replace `amortization-schedule.epher`'s `bal(k)` with `amort(p, r, n, k)`.
3. Unify the eight loan scripts on `const` bindings with distinctive names
   (`loan_payment`) — kills the incident class without app changes.
4. Rename the thirteen shadowing bindings of §2.3.
5. Consider the app-level fix (scripts run in a child scope / fresh session
   option) as the durable answer to session-state collisions.
6. Consider a two-sample (Welch) t built-in upstream — the fleet's hand
   version is the one statistics test the engine lacks.
7. Optional CI guard: a lint step in `site-build.yml` that fails when a script
   binds or defines a built-in or unit name, so §2.3-class regressions are
   caught at PR time.
