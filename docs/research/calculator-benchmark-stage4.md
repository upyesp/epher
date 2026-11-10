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

### Status

Session A (2026-11-08 into 2026-11-09) is complete for every cell that
could be honestly measured, with two exceptions recorded below. Session
B was **not run**: the stage-4 exit criterion (complete raw JSON for two
sessions on separate days) is not met, the A-vs-B 10 percent stability
bar is unassessed, and every number below is single-session. Pete
stopped the effort on 2026-11-09 after the desmos driver work ran past
its time budget.

The exceptions: Desmos W4 statistics has a gate but no time (the
mean/stdev/median helper readback never completed in three attempts;
plan rule 2). SpeedCrunch has **no stage-4 payload at all**: its
measured W1/W2/W4/W6/W7 data was computed twice (24 min per pass) but
lost both times - once when the runner re-emitted a failure cell over
the green payload after the python driver exited nonzero, once when the
driver crashed at the W6 restart on an empty AX enumeration (retry
added, untested at shutdown). The stage-3 pilot numbers (2026-11-07,
committed) remain the only published SpeedCrunch measurements.

### Coverage (measured cells per tool)

| Tool | W1 | W2 | W3 | W4 | W5 | W6 | W7 | W8 | W9 | W10 |
|---|---|---|---|---|---|---|---|---|---|---|
| epher native (N) | x | x | 100+500 | x | x | 42/50 partial, no time | x | lunar+sunrise+JD | a+b+c+d+e | spawn+REPL (native) |
| GNU Octave (N) | x | x | 100+500 both tracks | x | x | - | x | all three | a+b+c+d+e | - (native W10 file) |
| SpeedCrunch (G) | lost | lost | - | lost | - | lost | lost | - | - | - |
| epher web (W) | x | x | - | - | - | - | x | - | - | staged-entry boundary |
| GeoGebra (W) | x | x | - | - | - | - | x | - | a | - |
| Desmos (W) | x | x | - | gate failed, no time | integrals | - | not included | - | - | x |
| NumWorks (W) | - | - | - | - | - | - | - | - | - | x |

SpeedCrunch cells are W1, W2, W4, W6, W7 (W3/W5/W8/W9 not built this
stage); "lost" marks cells measured but not published (see Status).
Every green cell passed its referee gate before any timing was
recorded (plan rule 2). NumWorks publishes W10 only (declared
automation-cost deviation); Desmos W7 was excluded after re/im helper
probes timed out (declared).

### Session A medians (ms, single-session, gates green)

| Cell | epher N | Octave N | SpeedCrunch G | epher web W | GeoGebra W | Desmos W |
|---|---|---|---|---|---|---|
| W1 arithmetic (10k) | 5.6 | 31.9 | - | 877.0 | 8011.6 | 523902.8 |
| W2 transcendental (10k) | 2.6 | 27.0 | - | 807.2 | 5893.9 | 520463.0 |
| W3 LU n=100 | 2.4 | 14.0 | - | - | - | - |
| W3 LU n=500 | 108.8 | 248.7 | - | - | - | - |
| W4 statistics | 195.7 | 14.4 | lost | - | - | no time |
| W5 integrals | 0.185 | 7.3 | - | - | - | 116.3 (3 integrals) |
| W7 complex (200) | 0.507 | 5.2 | - | 36.25 | - | - |
| W8 lunar (100) | 26.6 | 40.1 | - | - | - | - |
| W9a prime sieve | 1.34 | 6.0 | - | - | 43.75 | - |
| W9e 10M loop | 18.6 | 56.4 | - | - | - | - |

Cross-tier rows are not comparable (the plan's tier caveat): the W-tier
numbers measure browser-input and helper-readback latency on an
unthrottled page, not raw evaluation.

### Interactive latency, W10 (native tier)

epher CLI: spawn-to-first-answer 2.95 ms, piped REPL keystroke-to-answer
0.16 ms. Octave CLI: 174.9 ms and 0.04 ms (its keystroke leg beats epher
because Octave's REPL is already warmed per session; the spawn leg
dominates for one-shot use). NumWorks emulator: launch 740.5 ms,
keystroke 2249 ms (trusted-click boundary). epher web: launch-to-staged
30.6 ms class, keystroke ~3 ms (staged-expression boundary, ADR-0038).

### Shipped-script sweep (epher only)

532 scripts, one pass each: 488 parse and evaluate, 44 error out
inside `bench_eval script-run`. The 44 are every script under
`graphs/`, and the cause is the harness path, not the scripts: `graph`,
`table`, and `save` are shell-level statements dispatched by the
frontends (ADR-0069), so the grammar-only `parse_script` entry rejects
them. The same scripts run clean through the real CLI file path
(`epher-cli <file>` exits 0, graph lines recorded into the plot state)
and paste clean into the TUI and web apps. Consequence for the sweep:
its timing table covers grammar-pure scripts only, so graphs scripts
(the sampling-loop-heavy ones, likely the slowest) are missing from the
medians. A follow-up should point the sweep at the CLI's whole-file
evaluator; the LSP's `evaluation_trace` shares the grammar-only path
and would flag graph lines the same way in editor diagnostics.
