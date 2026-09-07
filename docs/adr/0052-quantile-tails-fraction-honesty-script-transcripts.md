# ADR-0052: quantile tails, fraction honesty, and script transcripts

- Status: accepted
- Date: 2026-09-02
- Roadmap: calculator parity sweep (all 125 functions vs nine reference
  calculators) follow-up; supersedes parts of ADR-0051's reconstruction
  tolerance

## Context

The calculator parity sweep (docs/research/calculator-parity-report.md)
ran all 125 builtins and 45 constants against independent references.
Four findings needed fixing:

- **`invt`/`invchi2` tails** — `invert_cdf` stopped on a 1e-12 CDF
  residual (far too loose where the PDF is flat) and clamped to the
  caller's bracket: `invt(0.999999, 3)` returned 100 (true 103.3),
  `invt(0.9999, 1)` returned 100 (true 3183.1), and ordinary tail
  quantiles were wrong at the 10th-12th digit.
- **`invnorm` far tails** — the single Newton polish ran against
  `norm_cdf(x) = 1 - 0.5*q`, which loses the tail's digits once the
  CDF saturates toward 1; extreme quantiles stalled at 1e-8 relative
  error.
- **Fraction display** — the 1e-9 relative reconstruction tolerance
  scaled with the value, so large decimals with a *coincidental*
  convergent displayed as surprising fractions: `123456.789` became
  `13456790/109` (whose decimal differs at the 9th digit), and the TVM
  payment displayed `327259/446` although the true root is
  733.764573879376...
- **Quantity display** — the value inside a `Value::Quantity` was
  spelled raw: `30 deg in rad` showed 16 digits where `rad(30)` shows
  twelve.
- **Script transcripts** — a submitted multi-line or semicolon script
  displayed only its last answer; the guide's multi-line examples
  silently dropped the intermediate results (some fences showed them,
  others did not).

## Decision

- **Quantile inversions solve the survivor, not the CDF, for p > 0.5.**
  `invert_cdf` gains the cancellation-free survivor (the upper tail:
  the beta/gamma tail computed directly) and inverts
  `survivor(x) == 1 - p`; the CDF path handles p <= 0.5. The bracket
  doubles outward until it straddles the root (64 doublings, bounded),
  degenerate p returns the bracket edge as before, and convergence is
  measured on the Newton step relative to |x| (5e-14) instead of the
  CDF residual. The t survivor needs the sign branch (1 - 0.5*I for
  negative t); `t_survivor` provides it.
- **`invnorm` polishes in tail space.** The Newton solves
  `0.5*q(0.5, x^2/2) == min(p, 1-p)` (the survivor is even, so one
  formula covers both signs; the derivative is -sign(x)*pdf(x)).
  Measured: 1e-15 relative at every tail — the exact quantile of the
  stored p, which is the most any f64 calculator can do (the residual
  "error" vs 60-digit references at p ~ 1-1e-12 is the input's own
  f64 rounding: `1 - 1e-12` is not representable).
- **The fraction reconstruction tolerance is half a display unit**
  (5e-13 relative), for the display path, `exact()`, and the shell's
  exact table cells alike: a fraction shows only when it agrees with
  the value through all twelve displayed digits. Genuine repeating
  values (1/3, 1/7, 355/113, 500/121) still reconstruct; coincidental
  convergents do not. This supersedes ADR-0051's 1e-9 figure, and
  `exact()` now returns the terminating decimal's own fraction
  (`exact(123456.789)` is `123456789/1000`, not `13456790/109`).
- **Quantity values round like every other result** — the number
  inside `Value::Quantity` goes through `auto_float` before the unit
  text; the length guard keeps exact integers intact.
- **A script's every answer displays, in order, one per line.**
  `Session::submit_all` runs the script with `run_all` and returns the
  whole transcript; the CLI's line modes, the REPL, the TUI's entry,
  and the web's answer area all show it. History keeps the established
  compact record (the line with its last answer appended; multi-line
  scripts verbatim). Statements that produce no value (`def`, `while`,
  `graph`) contribute nothing, errors stop the run as before, and the
  TUI's answer area grows to fit the transcript (up to six lines).
  The guide's multi-line example blocks now show every answer, and the
  translated guides that had dropped the statistics output block got it
  back, byte-identical across all eight locales.

## Consequences

- `invt(0.999999, 3)` is 103.299467779429, `invt(0.9999, 1)` is
  3183.098757118, `invchi2(0.999999, 5)` is 35.888186879610,
  `invnorm(1 - 1e-12)` is 7.034486910048 (the stored-p root), and all
  of them match the references at the twelfth digit.
- `123456.789`, `1234567.891`, `tvm_pmt(360, 0.08/12, -100000, 0)` =
  733.764573879, and `tvm_pv(...)` = -99999.3766557 display their true
  values; the misleading coincidental fractions are gone. `march_equinox(2000)`
  now displays 1012520636/413 (agrees to 13 digits; the old
  26967862/11 agreed to 9).
- `x = 10; y = x + 5; x + y` shows `= 10`, `= 15`, `= 25` in every
  frontend; the guide's multi-line fences show the same transcript the
  app shows, byte-identical in all 8 locales.
- The one-shot CLI, piped mode, `load`, and one-shot `epher "…"`
  behave alike; the web keeps per-statement error recovery (later
  statements still run after an error) while the CLI/TUI stop at the
  first error, exactly as before this round.

## Amendment (2026-09-02): answers flow on one line (ADR-0055)

In the web and desktop answer area the transcript now flows on one
line with semicolons between answers when they fit; an answer is never
split across lines (a long one moves whole to the next line), and
multi-line outputs (tables, matrices) keep their own blocks. The
terminal frontends keep the one-per-line transcript described above.
