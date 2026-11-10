# Calculator benchmark harness (stage 3 pilot)

The measurement instrument for the benchmark plan
(docs/research/calculator-benchmark-plan.md). Everything below was declared
before the first published number and must not be adjusted after seeing
results; deviations are recorded in the final report.

## One-liner re-runs

```
node scripts/bench/run-pilot.mjs a        # session a: all three pilot tools
node scripts/bench/run-pilot.mjs b        # session b, same day (stability)
node scripts/bench/run-pilot.mjs a --tools epher      # one tool only
node scripts/bench/generate-data.mjs && python3 scripts/bench/referee.py
node scripts/bench/gen-octave.mjs          # regenerate Octave implementation files
cargo build --release --example bench_eval -p epher-core
```

Reproducibility bar (plan): an independent re-run on the same machine
reproduces published medians within 10 percent.

## Layout

- `generate-data.mjs` - fixed-seed datasets (mulberry32, seed 1, 10,000 lines
  each), emitted to `data/` and committed byte-identical (plan rule 10).
- `referee.py` - 50 to 60 digit decimal referee (parity-report method,
  stdlib only): computes `data/acceptance.json`.
- `crates/core/examples/bench_eval.rs` - epher's in-process timing binary
  (builds to `target/release/examples/bench_eval`).
- `implementations/<tool>/` - the exact sources timed, committed.
- `drivers/` - one driver per tool; each verifies the correctness gate
  before sampling.
- `preflight.mjs` - environment manifest + quiet-machine checklist (screen
  lock via scripts/a11y/session-lock.py, AC power, thermal, top CPU).
- `run-pilot.mjs` - orchestrates a session; results land incrementally in
  `results/raw/<date>/session-<a|b>/<tool>.json`.

## Declared protocol (stage 3 pilot scope)

Per the plan's stage table, stage 3 = drivers for three tools (epher native,
GNU Octave, SpeedCrunch), tasks W1 + W2 + W9a, both tracks, same-day re-run.
W10 interactive-latency drivers and all W-tier drivers belong to stage 4
("full run"); the stage 2 report's work-list mention of W10 in stage 3 was
superseded by the plan's stage table.

### Tasks

- W1: 10,000 scalar expressions (+ - * / ^, parens), batch-timed.
- W2: 10,000 transcendental microbench lines (sin cos tan exp log sqrt),
  batch-timed. Canonical `log(` means natural logarithm. Rendering table
  (function names only; never values): epher log->ln, SpeedCrunch log->ln,
  GNU Octave log unchanged.
- W9a: count primes below 10,000. Tracks: idiomatic per tool; the
  equivalent-algorithm track is the classic flag-array sieve with explicit
  loops.

### Tracks

- W1 and W2 fix the expressions, so the idiomatic and equivalent
  implementations coincide by construction; they are published as a single
  track and never presented as two numbers.
- W9a idiomatic: epher = trial division by candidates up to sqrt(n), chunked
  into five `for` statements to fit the per-statement step budget (STEP_LIMIT
  100,000 per statement, per crates/core/src/lib.rs; the shell's script path
  budgets per statement via submit_all); GNU Octave = vectorized flag-array
  sieve. These are different algorithms on one spec, reported per tool and
  never compared head-to-head without that caveat.
- W9a equivalent: GNU Octave plain-loop sieve. epher cannot express list
  index assignment (ADR-0044 gives reads only), so its equivalent cell is
  **not supported** and publishes no time (plan rule 11). SpeedCrunch has no
  scripting: W9a is not applicable to it (stage 2 matrix).

### Timing boundaries (one per published cell)

- epher: in-engine (parse + eval), measured by Instant inside
  `bench_eval` linking epher-core. Statement execution mirrors the shell's
  script path: one `run()` per statement (fresh step budget each).
- GNU Octave: in-engine via tic/toc inside the committed .m file around the
  block; process spawn and printf are outside the span.
- SpeedCrunch: GUI entry end-to-end - per-expression accessibility value
  injection + Return over the whole dataset, summed. AX value sets serialize
  on the app's main thread, so the loop self-throttles to the app's real
  rate; driver polling and post-checks are outside the span. Rationale
  (declared): the multiline-paste batch path proved non-deterministic to
  deliver (sometimes inserted literally or swallowed), so the reliable
  per-expression path is the instrument. SpeedCrunch numbers therefore
  include GUI entry overhead that epher/Octave in-engine numbers do not;
  tables state this per cell.

### Rules applied (see the plan for the full text)

- Correctness gate before timing: every tool's full output is compared
  against the referee at relative tolerance 1e-9 before any time is
  published. A failure publishes "incorrect", never a time.
- Warm-up: 3 untimed passes for every tool (rule 4).
- Samples: bucket by the measured warm-up pass duration, fixed in advance -
  under 1 s: 30; under 60 s: 7; otherwise 3 (rule 5).
- Caps: default 10 minutes wall per measured run; a run over the cap is
  published as censored (rule 6).
- Default configurations: tools as installed; the only configuration fact
  recorded for SpeedCrunch is its default display precision (about 20
  significant digits, well above the gate's needs) (rule 7).
- One machine, one set of conditions; preflight before each session;
  thermal recorded at start and end; 30 s cooldown between tools (rule 8).
- Statistics: min, median, MAD, p95 (nearest-rank) from committed raw
  samples; no trimming; samples beyond 10x the median are flagged, the run
  repeated once, both kept (rule: outliers).
- SpeedCrunch line-level instrument rule: if the post-pass check finds the
  input non-empty (a Return was swallowed), the pass is discarded and
  retried; discards are counted and reported.

### Known instrument facts (declared)

- SpeedCrunch's history accessibility value caps around 4.5 KB, so the gate
  reads results in 80-line windows with Edit > Clear History between windows
  (its confirmation dialog is answered programmatically).
- SpeedCrunch's input line swallows Return while its autocomplete popup is
  open; injecting the value via accessibility avoids the popup entirely.
- The referee gate would surface any rendering-table mistake (e.g. base-10
  vs natural log) as "incorrect".

## epher implementation files

- W1/W2: the dataset files themselves (one expression per line), evaluated
  per line into a shared Env - the REPL/session shape.
- W9a: `implementations/epher/w9a-primes.epher`.
