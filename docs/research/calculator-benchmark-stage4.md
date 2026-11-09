# Calculator benchmark, stage 4: full matrix

Status: **in progress** (session A measured 2026-11-08; session B due 2026-11-09).
Stage 3 established the harness and the two-session ±10% stability bar on a
three-tool pilot; stage 4 extends to the full W1-W10 matrix across all seven
candidate tools and adds two epher-only studies (W10 N-tier native surfaces,
the 532-script shipped-script sweep).

## What stage 4 measures

Per task W1-W9 (spec in `calculator-benchmark-plan.md`), every tool publishes
a median wall-clock time per operation at its declared boundary, gated by the
referee before any timing is recorded (plan rule 2: incorrect means no time).
W10 measures interactive latency (launch-to-ready, keystroke-to-answer).

Tools and coverage declared up front (deviations recorded here, not silently):

| tool | cells | notes |
|---|---|---|
| epher (native) | W1-W9 full + sweep | W6 partial: 42/50 rows (no cal/BTU/kWh/MJ/hp, no temperature offset scales) |
| epher (web) | W1, W2, W7, W10 | wasm build; chunked-script boundary, see below |
| Octave | W3-W9 native + equivalent, W1/W2 single, W10 | complex W7 has one algorithmic track (no equivalent-choice axis) |
| SpeedCrunch | W1, W2, W4 subset, W6 rows, W7, W10 | no scripting; per-value automation boundary |
| GeoGebra | W1, W2, W7, W9a, W10 | W3/W5/W8 cells not built this stage (automation cost) |
| Desmos | W1, W2, W4 subset, W5 integrals, W10 | W7 excluded: re/im helper expressions time out |
| NumWorks | W10 only | W9 script cells deferred: keystroke-level automation cost (declared) |

## Boundaries and tolerances (declared before measurement)

- W3 referee is float64 LU (Decimal at n=500 impractical; declared). det/trace
  relative tolerance 1e-6 (n=100) / 1e-4 (n=500); x-hat = A*ones exactness via
  referee-published b embedded in the `.epher` files; residual L1 gate
  1e-7*n. n=500 matrices pre-scaled by n^(-1/2) so float64 det stays finite
  (raw det overflowed to Infinity; declared).
- W3 det gates compare magnitudes: independent LU factorizations may flip the
  sign of det under pivot near-ties (observed between epher and Octave LUs).
- W4: Type-7 percentiles; epher's native quartile is the Excel method, so the
  epher implementation computes Type-7 manually via sort+index (declared).
- W5: native integral/solve where a tool has them; explicit Simpson/bisection
  as the equivalent track (Octave).
- W6 coverage rule: a tool publishes a time only at 50/50 unit support;
  otherwise "partial: n/50", no time.
- W7: complex arithmetic has no algorithm choice (same rule as W1/W2);
  `^` excluded from the corpus.
- W8 gates are algorithm-agreement gates (1.0 deg lunar phase, 3 min sunrise,
  jd*1e5 integer), not precision gates; referee uses Meeus ch.47/48 and NOAA
  sunrise, verified against epher to worst 0.118 deg / 1.5 min before any
  timing.
- W9c gate is exact-string equality (epher has no char-code builtin);
  balanced count 801.
- xhat_sum tolerance per size: 1e-9 at n=100, 1e-6 at n=500 (measured
  inv-solve drift ~7e-7 at cond ~1e6; widened before any W3-500 timing was
  recorded, and the widening is recorded here).

## Harness additions

- `gen-data2.mjs` + `referee2.py`: stage-4 datasets and the float64/Decimal
  referee -> `data/acceptance2.json`. Stage-3 `acceptance.json` still holds
  W1/W2 and w9a_count; the runner merges the two before gating.
- `bench_eval` example gained `expr-verify-cont` and `script-verify-all`
  (per-statement quantity/string outputs for gating without timing).
- Implementation generators: `gen-epher.mjs` (17 cells), `gen-octave2.mjs`
  (native + equivalent tracks, `__astro.m`, `__luhelpers.m`).
- W-tier drivers share `drivers/cdp.mjs` (headless Chrome for Testing + raw
  CDP; per-process profile dirs after a SingletonLock collision). Desmos
  readback = HelperExpression + observe('numericValue'); GeoGebra = Apps API
  evalCommand; NumWorks = trusted CDP clicks on `[data-key]` spans and LCD
  canvas readback; epher web = chunked textarea submit with 24-term grouped
  checksum lines (a single 500-term line opens the autocomplete popup, which
  swallows Enter).
- `run-stage4.mjs`: sequential runner, per-tool 75-min timeout (desmos 300
  min: its helper-arrival latency makes a W1 pass ~13 min), preflight
  manifest, cooldowns between tools.
- `w10-native.mjs`: epher CLI one-shot spawn->first byte and piped REPL
  line->answer; Octave CLI the same two legs (TUI dropped per the plan's
  automation-reliability caveat).
- epher-only sweep: all 532 shipped scripts (`epher scripts/`), one pass
  each, in-engine parse+eval per script, medians per top-level category;
  process spawn excluded (same boundary as every epher in-engine cell).

## Bugs the gates caught before timing (selection)

- referee ly factor carried a spurious *1000 (W6 rows 6/7): epher was right.
- epher w4 r formula: sum((y-ybar)^2) is n * population variance, not
  (n-1); the gate caught a 5e-5 relative error in lin_r.
- octave w9c trailing newline inflated the balanced count to 802; the
  generator strips it.
- epher web: acceptance W1/W2 values are strings, sums render as rationals
  a/b - both must be parsed, not concatenated.
- Desmos API finding: a HelperExpression whose latex is a bare
  constant-only expression (e.g. \sin(1.413)) never produces a
  numericValue; wrapping the expression in \frac{...}{1} forces a full
  evaluation tree and returns the exact value (probed 2026-11-08). The
  desmos driver frac-wraps every W1/W2 expression; arithmetic helpers
  work either way.
- epher web W10 launch-to-ready boundary: the share link (?expr=) only
  stages the expression in the entry (ADR-0038); there is no
  auto-submit, so launch-to-ready is polled as "expression staged", not
  "answer present". The original answer-present definition produced no
  samples in two runs before the redefinition, so no numbers are being
  invalidated.
- Desmos spans require an unthrottled renderer: helper-expression
  updates ride the page's frame loop, so a backgrounded headless page
  throttles to ~1 Hz and per-helper spans inflate 3-12x (measured
  2026-11-09: 82 ms mean unthrottled vs ~1000 ms throttled). All
  benchmark chromes now launch with background-timer-throttling
  disabled; the desmos cells were re-measured after this fix, and the
  pre-fix desmos numbers are not used.

## Sessions

Session A: 2026-11-08, `results/raw/2026-11-08/session-a/`. Session A
spilled past midnight: the desmos W2/W4/W5 cells re-measured in the
early hours of 2026-11-09 (the only tool whose cells were still
failing at midnight) are recorded with their true timestamps and live
in session A's directory. Session B: 2026-11-10, same command, a
separate day from every session-A cell per the plan's stability rule.
Stability bar: A-vs-B medians within 10% per cell; tools that only
publish coverage (W6 partials) compare gate notes.

## Results

(filled after session B and the compare pass)
