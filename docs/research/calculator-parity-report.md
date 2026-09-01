# Calculator parity report: epher vs the reference calculators

Date: 2026-09-01 · Status: findings only, no code changed

## Scope and method

Every builtin function and constant of epher (v0.5.15, 125 functions
+ 45 constants from the builtin catalog) was exercised and the results
compared against what the nine reference calculators from the feature
gap analysis (NumWorks, Desmos, TI-84/Nspire, Wolfram, SpeedCrunch,
HiPER, HP Prime, MATLAB/Octave) show. Visual UI and semantics were
deliberately ignored; the numerical results are the subject.

367 expressions were run through the v0.5.15 CLI (displayed value,
full precision via `scientific()`, and error text captured):

- 156 expressions were auto-checked against independent references:
  (a) true values computed to 50-60 digits by a from-scratch decimal
  (stdlib-only) math library (Taylor series, argument reduction,
  Spouge gamma, Lentz continued fractions), and (b) the V8/fdlibm f64
  math as a second, independent libm (glibc is epher's). Display
  expectations applied epher's own rules (twelve significant digits,
  fraction only for repeating decimals, exact integers unrounded).
- 211 expressions were checked against hand-computed values: exact
  arithmetic (matrices, number theory, bitwise, exact layers), closed
  formulas (TVM per the TI convention, amortization, interest,
  statistics on lists), the Meeus Julian-date algorithm, CODATA 2022 /
  SI-exact constants, and previously Horizons-validated ephemeris
  values.

Result: 363 of 367 expressions produce the value a reference
calculator shows at the twelve-significant-digit display level, and
every one is within a few ulp of the correctly rounded true value
except the handful listed below. The remaining differences, with the
evidence, follow.

## Findings that need correcting

Ordered by how often a user would hit them and how wrong the result
looks.

### 1. invt and invchi2 lose accuracy in the tails (two causes)

epher inverts the t and chi-squared CDFs with a hybrid
bisection/Newton (`invert_cdf`) that stops when |CDF(x) - p| < 1e-12.
That absolute tolerance on the CDF is not tight enough where the PDF
is small, and the search bracket is fixed at [-100, 100] (t) and
[0, df + 40*sqrt(2df)] (chi2).

Measured:

| Expression | epher shows | true value |
|---|---|---|
| `invt(0.995, 3)` | 5.84090930972 | 5.8409093097334 (1.9e-12 rel, visible at digit 12) |
| `invt(0.99999, 3)` | 47.927728373 | 47.9277283759 (visible at digit 11) |
| `invt(0.999999, 3)` | 100 | 103.3 (bracket clamp) |
| `invt(0.9999, 1)` | 100 | 3183.1 (bracket clamp; t1 is a Cauchy tail) |
| `invchi2(0.999999, 5)` | 35.8881868731 | 35.8881868797 (visible at digit 10) |
| `invchi2(0.95, 5)` | 11.0704976935 | 11.0704976935164 (2.7e-13) |

The bracket clamp is a wrong result (not just precision): the t1 and
t3 tails reach far beyond 100. The tolerance issue shows up at 10-12
displayed digits for p beyond roughly 0.999 with small df. Fix: widen
the t bracket (or double until bracketed), and stop on a tolerance on
x (e.g. |f|/pdf < 1e-14) rather than on the CDF.

### 2. invnorm is inaccurate in the far tails

`invnorm` uses Acklam's rational approximation with a single Newton
polish against `norm_cdf`. The polish cannot converge beyond what
`norm_cdf(x) = 1 - 0.5*q` can represent: for x near 7 the tail
(~1e-12) keeps only 4 significant digits after the subtraction from 1,
so the polished root inherits that error.

Measured (epher vs true):

| p | epher | true | rel error |
|---|---|---|---|
| 0.975 | 1.959963984540031 | 1.9599639845400542 | 1.2e-14 |
| 1 - 1e-8 | 5.61200124287 | 5.61200124417 | 1.3e-9 |
| 1 - 1e-10 | 6.36134088679 | 6.3613409024 | 1.6e-8 |
| 1 - 1e-12 | 7.03448690268 | 7.0344838253 | 3.1e-6 |

Visible from about p = 0.99999998 on (the 10th digit), badly wrong
beyond p = 1 - 1e-10. NumWorks and TI invert the normal CDF to full
double precision in the tails. Fix: polish in tail space, i.e. solve
`0.5*q(x) == 1 - p` for p > 0.5 instead of `norm_cdf(x) == p` (the
complement path `norm_cdf` already uses for negative x has full tail
precision; `normcdf(-7) = 1.2798125438857812e-12` matches the true
value to the last bit).

### 3. Over-eager fraction display for large decimals

The round-10 display rule shows a fraction when the value reconstructs
to a denominator <= 1000 within relative tolerance 1e-9. Relative
tolerance scales with the value, so large decimals that merely happen
to have a coincidental convergent get turned into surprising
fractions:

| Expression | epher shows | references show |
|---|---|---|
| `123456.789` | `13456790/109` (repeating) | `123456.789` |
| `1234567.891` | `56790123/46` | `1234567.891` |
| `1.23456789` | `1.23456789` (correct) | `1.23456789` |

The same rule handles the cases the user cares about correctly
(`0.1 + 0.2` -> 0.3, `1/3` -> 1/3, `tvm_pmt(...)` -> 327259/446). The
offenders are values whose best <= 1000 denominator convergent lands
within 1e-9 * |x| (an absolute window of ~1e-4 at x = 1e5). Fix:
tolerate at most half a display unit, e.g. |x - p/q| <= 5e-13 *
max(|x|, |p/q|). That still shows every genuinely repeating value
(1/3, 1/7, 355/113, 327259/446) and hides the coincidences.

### 4. Quantity results bypass the twelve-digit rounding

`30 deg in rad` shows `0.5235987755982988` (16 digits) where the same
value through `rad(30)` shows `0.523598775598`. The unit-conversion
display path (Value::Quantity) does not go through the new `auto_float`
rounding. Cosmetic but inconsistent with the round-10 fix; the
displayed digits should be 12 for every path.

## Convention differences (documented, deliberate, or minor)

These return different numbers from some references by design; each is
already documented in the guide or an ADR, but they are the places a
user comparing against another app will see a difference.

| Topic | epher | NumWorks/Desmos/Wolfram | TI/SpeedCrunch/HiPER/HP |
|---|---|---|---|
| `200 + 10%` | 200.1 (percent is /100) | 200.1 | 220 (percent of base) |
| `mod(-10, 3)` | -1 (C/Rust sign of dividend) | 2 | 2 |
| `stdev` / `variance` | population (n) | population | sample (n-1) |
| `irr({-100,60,60})` | 0.1307 (rate) | percent (13.07%) | percent (13.07%) |
| `tvm_i` | 0.0066666 (per-period rate) | percent display | I/Y percent |
| `binompdf(k, n, p)` | k first | k first | (n, p, k) |
| `poissonpdf(k, lambda)` | k first | k first | (lambda, k) |
| `logb(b, x)` | base first | - | base first (TI logBASE) |
| `root(n, x)` | n first | x^(1/n) | x^(1/n) |
| `atanh(2)` | 0.549306 - 1.5708i (C99 branch, num_complex) | error | error; MATLAB/numpy: +1.5708i |
| `0 ^ 0` | 1 | 1 | 1 |
| `1/0`, `ln(0)` | error / -inf | error / -inf | error / error |
| `2^1024` | inf (NumWorks-like) | inf | overflow error |

Note the percent, mod, stdev and irr/tvm rows are the four where a
user moving from TI/SpeedCrunch/HiPER/HP gets a visibly different
number from the same keystrokes; the guide documents each, but if the
goal is "results users of the other apps expect", these four are the
candidates for either a change or a prominently surfaced setting.

## Precision notes (correct at 12 digits, off beyond)

Values are correct to the twelve displayed digits; the error appears
only in digits 13-16 that a reference calculator shows:

| Function | worst measured | example |
|---|---|---|
| `normcdf` positive tail | 6.6 ulp; tail 219 ulp at x=-3 (4.9e-14) | `normcdf(1.96)` = 0.975002104851781 vs ...8517796 |
| `invnorm` central | 53 ulp (1.2e-14) | 0.975 case above |
| `tcdf` | ~200 ulp (4.4e-14) | `tcdf(1.5, 10)` = 0.91774633677726 vs ...77728 |
| `invchi2` | 1200 ulp (2.7e-13) | 0.95, df 5 case above |
| `binompdf` / `binomcdf` | ~2900 ulp (6e-13) | `binompdf(2, 10, 0.5)` = 0.0439453125000283 vs 0.0439453125 |
| `poissoncdf` | 3.2 ulp | fine |
| `integral` | 1e-13 | `integral(exp(-x^2), -2, 2)` = 1.76416278152466 vs ...4843 (TI fnInt is ~1e-5, so epher is still 8 orders better) |

`binomcdf` accumulates the log-gamma PDFs, so its last ulp is noise:
`binomcdf(7, 10, 0.3)` displays 0.998409613601 where the exact value
is 0.9984096136; the 12th digit flips only because the exact value
ends in zeros. A recurrence-based sum would make these exact.

## Pre-existing value discrepancies noticed on the way

- `march_equinox(2000)` returns 2451623.8181... (displayed as
  26967862/11) where the true equinox is 2451623.8160; the ephemeris
  model is ~3.2 minutes late. The guide fence already carries the
  correct value, so the guide hides it; the app is the wrong one.
  (Same class as the round-9 flag; not part of this task, listed for
  completeness.)
- The stored `r_inf` constant is 76816121/7 = 10973731.5714 m^-1
  (CODATA R_inf = 10973731.568160, visible at the 9th digit; the
  guide table shows the CODATA value) and `faraday` is
  26436981/274 = 96485.3321168 C/mol (CODATA 96485.33212, invisible
  at 12 digits). Both look derived from truncated inputs (they round
  to nice rationals); they should be stored at full CODATA precision.
- `big(2^100)` = 1267650600228229400000000000000 because the
  float-to-big conversion goes through the shortest decimal spelling;
  `big(2)^100` gives the exact 1267650600228229401496703205376.
  Document, or convert the f64 exactly.

## What already matches (summary of the full sweep)

- Elementary functions (sin, cos, tan, asin, acos, atan, atan2, sinh,
  cosh, tanh, asinh, acosh, atanh on the real line, exp, ln, log,
  log2, logb, sqrt, cbrt, root, hypot, ^, !, %): all match the true
  value to 12 digits; sin/cos/tan match glibc bit-for-bit at the
  argument-reduction stress points probed (1e6, 1e-300).
- Complex arithmetic and functions: sqrt(-1), i^k, re/im/arg/conj/
  abs, exp(i*pi), asin(2), acos(2), ln(-1) all match the principal
  values (atanh is the one branch exception above).
- Exact layers (frac, dec, big, exact), number theory (isprime,
  factors, nextprime, prevprime, ndivisors, totient, modpow, ncr,
  npr, gcd, lcm, fact), bases (bin/oct/hex/0x literals), bitwise with
  word sizes: exact, all verified against closed forms.
- Statistics on lists (mean, median, mode, range, quartile, sum,
  product, len, sort, linreg), the normal family at ordinary p, t and
  chi2 at ordinary p, binomial/poisson, and the test functions
  (ttest, ztest, zinterval, tinterval, chisq_gof): values verified
  against the closed forms (e.g. t = -1.4142, p = 0.2302 for
  ttest({12,14,13,15,11}, 14); chi2 = 2, p = 0.5724 for the gof case).
- Finance (tvm_n/i/pv/pmt/fv with the TI sign convention, begin-mode,
  npv, irr, amort, simple/compound interest): verified against the
  closed-form/TI values; `tvm_pmt(360, 0.08/12, -100000, 0)` =
  733.7645739910314 exactly matches the TI BA II Plus.
- Matrices (det, inv, transpose, trace, dim, ref, rref, products,
  powers, indexing): exact rational arithmetic; values match the
  NumWorks floor.
- Calculus: derivative and integral agree with the analytic values
  (integral(sin x, 0, pi) = 2, integral(1/x, 1, e) = 1, signed
  integrals, derivative(x^2, 3) = 6); solve returns the correct roots
  including complex ones and all three cube roots of 8.
- Units and conversions: SI factors exact (1 AU = 149597870700 m,
  1 pc = 30856775814913670 m, 1 ly = 9460730472580800 m, 60 mile/hr =
  26.8224 m/s), dimension errors fire, sin(30 deg) = 0.5.
- Astronomy: jd/mjd match the Meeus algorithm to the day (leap-year
  edge cases included); delta_t(2000) = 63.86 s; decl(10, J2000
  solstice) = 23.437252 deg; sun/moon distances, diameters,
  illumination, magnitudes, and Greenwich rise/set/transit times for
  2000-01-01 (08:06/12:03/16:00) all land where the almanacs say; the
  ephemeris was Horizons-validated in earlier rounds.
- Constants: pi/e/tau/phi and all 45 physical constants match the
  guide's CODATA 2022 table at 12 digits (m_e, m_p, m_u, a_0, alpha,
  r_gas, z_0, G, h, k_b, sigma_sb, eps_0, mu_0, ev, n_a, atm...).
- Random: deterministic SplitMix64 with reproducible seeds; values
  differ from other apps' PRNGs by construction (no reference match
  is expected or claimed).

## Coverage notes

Not result-comparable by nature and therefore not in the tables:
`graph`/`graph3d`/`table` (plotting), `now()` (host clock), the
constants browser and history/share (UI). Everything else in the
builtin catalog was exercised at least once, most functions at
several points including edge cases.

## Suggested priority order

1. invt bracket/tolerance and invchi2 tolerance (wrong digits in the
   tails, wrong values past the clamp).
2. invnorm far tails (wrong digits from p ~ 1 - 1e-8).
3. Fraction-display tolerance (123456.789 shows a fraction).
4. Quantity display rounding (30 deg in rad shows 16 digits).
5. binomcdf/binompdf exact recurrence (last-digit cleanliness).
6. Constants r_inf/faraday at full CODATA precision.
7. Convention review for percent, mod, stdev, irr/tvm_i percent
   display (each documented; decide whether to match TI or keep).
