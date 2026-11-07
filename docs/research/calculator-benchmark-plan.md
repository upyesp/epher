# Calculator benchmark plan: speed and efficiency, stage 1 (methodology)

Date: 2026-11-07 · Status: scope decided 2026-11-07, plan only, no measurements taken

## Why this document

This is the third measurement series against the nine reference calculators
from the feature gap analyses (NumWorks, Desmos, GeoGebra, TI-84 Plus CE /
TI-Nspire CX II, WolframAlpha / Wolfram Language, SpeedCrunch, HiPER
Scientific, HP Prime, MATLAB / GNU Octave):

- the gap analyses (baseline through the 2026-11-07 fourth pass) measure
  which features exist,
- the parity report (docs/research/calculator-parity-report.md, 2026-09-01)
  measures whether numeric answers agree,
- this plan defines how to measure how fast the work is done: calculation
  speed and script processing, against the reference apps we can
  legitimately run plus epher. The scope decisions of 2026-11-07 (see
  Decisions, at the end) fix which apps those are and why the rest are
  absent.

Stage 1 (this document) fixes the methodology before anything is run, so
the results cannot be accused of being chosen after the fact. The fairness
rules, workload specs, sample counts, statistical methods, and caps below
are declared in advance and are not to be adjusted after seeing numbers.
Any deviation in later stages must be recorded as a deviation in the final
report.

## Tools under test and their delivery tiers

Absolute cross-platform timing is meaningless without a tier model: a
calculator SoC cannot fairly race a desktop CPU, and a browser page cannot
fairly race a native binary. Every tool is assigned one tier, and ranking
claims are made only within a tier.

| Tier | Meaning | Tools |
|---|---|---|
| N (native) | Local app on the benchmark machine, driven by CLI, script, or AppleScript | epher CLI / REPL / TUI, SpeedCrunch, GNU Octave, GeoGebra Classic 6 desktop |
| W (web) | Same pinned browser build, same profile discipline | Desmos, NumWorks web, GeoGebra web, epher web (PWA) |
| E (emulated) | Vendor emulator on the benchmark machine, observational only | NumWorks emulator (optional; go/no-go in stage 2) |

| Not benched | Reason (disclosed in every report table) |
|---|---|
| Wolfram Language / WolframAlpha | no license for Mathematica or the engine; WolframAlpha is a network service whose latency is network, not computation |
| TI-84 Plus CE / TI-Nspire CX II | no hardware and no emulator license |
| HP Prime | no hardware; the virtual calculator is likewise unavailable to license |
| HiPER | excluded by decision (2026-11-07); it would have required an Android emulator tier for a single app |

Rules that fall out of the tiers:

1. epher and GeoGebra are benchmarked in both N and W (native CLI and
   web app). A native-vs-web comparison would flatter the native side,
   so none is published.
2. Column labeling: the gap analyses' "MATLAB / GNU Octave" column is
   represented here by GNU Octave alone, and every table labels it as
   GNU Octave. No MATLAB version claim appears in any result.
3. E-tier entries are observational and never merged into N or ranked
   against it. The only candidate is NumWorks' emulator, and its
   clock-fidelity claim is verified against NumWorks' documentation in
   stage 2 before any E-tier number is accepted.
4. Version pinning as in the gap analysis: every number names the exact
   tool version and the date it was benched.

## Fairness rules

These apply to every workload and every tool, including epher.

1. **Task specs, not code.** A benchmark task is a written specification
   (inputs, operation, expected output, tolerance). Each tool implements
   it in its own idiomatic style. Where the language permits, a second
   "equivalent algorithm" implementation mirrors the same algorithm
   structure across tools. Both tracks are reported separately and are
   never mixed into one column. This prevents the classic failure modes:
   punishing a vectorizing language with loops, or flattering a scripting
   calculator with its optimized builtin that another tool lacks.
2. **Correctness gate before timing.** Every task has acceptance values
   computed by a 50 to 60 digit decimal referee (the parity report's
   stdlib-only method, reused). A tool whose output falls outside the
   task tolerance is recorded as "incorrect" and publishes no time. A
   fast wrong answer is not a benchmark result.
3. **Declared timing boundary.** Every published number states what it
   includes: end-to-end (process spawn + parse + eval + print) or
   in-engine (parse + eval only). Today no tool gets an in-engine number
   unless it exposes a timer itself (Octave tic/toc, Python
   time.perf_counter inside NumWorks MicroPython). epher's core has no timing builtin (verified against the
   dispatch catalog on 2026-11-07), so epher's in-engine numbers come
   from a small harness binary that links the core crate and calls the
   evaluator in-process, exactly the role scripts/perf-sweep plays
   end-to-end today. Harness-measured engine timing is labeled as such
   for every tool; spawn costs are never mixed into in-engine columns.
4. **Warm-up policy, fixed in advance.** JIT-bearing runtimes (Octave's
   JIT where enabled, browser JS engines, WASM) get a fixed warm-up phase
   (3 untimed runs) before measurement. The same policy applies to every
   tool in the tier regardless of whether it needs it.
5. **Sample counts declared up front.** At least 30 measured runs for
   tasks under 1 second, at least 7 for tasks under 1 minute, at least 3
   otherwise. All raw samples are committed; nothing is averaged away.
6. **Declared caps.** Each task has a wall-clock cap fixed in advance
   (default 10 minutes). A tool that exceeds the cap is published as
   "censored at cap", not silently dropped and not given a fake time.
7. **Default configurations.** Every tool is benched as installed, in its
   default configuration, with the configuration recorded. No
   benchmark-only tuning (flags, recompiles, alternate interpreters) is
   applied to any single tool. If a configuration choice is made for one
   tool for a legitimate reason, it is documented and mirrored where
   applicable.
8. **One machine, one set of conditions.** N, W, and E tiers all run on
   the benchmark machine (the Mac Mini, Apple Silicon, macOS 26; exact
   model and build recorded in the environment manifest). Quiet-machine
   checklist before each session: no other heavy processes, scheduled
   jobs paused, AC power, displays on (GUI tools need the desktop),
   screen-lock guard active (scripts/a11y/session-lock.py already
   implements the check). Cooldown pauses between tools to avoid thermal
   skew; thermal state recorded at session start and end.
9. **Browser discipline (W tier).** One pinned browser build, a dedicated
   fresh profile, fixed viewport, extensions off, hardware acceleration
   state documented, timing taken from performance.now() inside the page
   via the DevTools protocol. The browser choice and build are part of
   the manifest.
10. **Fixed randomness and fixed data.** Any task using randomness fixes
    seeds per tool (epher randseed(n), MATLAB rng(1), Octave rand with a
    fixed seed, seeded PRNG in JS). All bulk inputs (matrices, datasets,
    expression lists, string corpora) are generated once by a committed
    generator script into scripts/bench/data/ and fed byte-identical to
    every tool.
11. **"Not supported" is not a time.** A tool that cannot express a task
    is recorded as not supported for that cell. Nothing else is inferred
    from an absence.
12. **Licensing honesty.** The bench set as decided contains no paid
    licenses: Octave, SpeedCrunch, Desmos, NumWorks, and the epher
    surfaces are all free to run, and GeoGebra's noncommercial terms are
    verified in stage 2 to cover this use before any GeoGebra number is
    accepted. If a licensed product (MATLAB, Wolfram, TI/HP software)
    later becomes available, it joins the set under these same rules and
    the report gains a dated row for it.

## The workload suite

Task sizes are fixed so the same spec runs everywhere. The suite keeps a
deliberate epher flavor (units, astronomy) because the benchmark should
measure epher's real workload, not only generic arithmetic where every
f64 engine is indistinguishable.

### Common-denominator calculations (W1 to W8)

| Task | Spec | Notes |
|---|---|---|
| W1 scalar arithmetic | 10,000 generated expressions, mixed + - * / ^ with parens, from a fixed seed; batch-timed | measures parser + evaluator overhead per expression |
| W2 transcendental microbench | 10,000 iterations over a fixed mix of sin, cos, tan, exp, log, sqrt on fixed arguments | the classic libm probe |
| W3 linear algebra | fixed 100x100 and 500x500 matrices (shipped files): matmul, determinant, solve | loop fallback in the equivalent track where no matrix type exists; cap applies; Octave is the reference vectorized implementation |
| W4 statistics | fixed 10,000-sample dataset (shipped file): mean, sample stdev, median, percentiles, simple linear regression | ties to the language's documented z/t interval semantics |
| W5 numerical methods | three fixed definite integrals to relative tolerance 1e-9 (one with a closed form for checking); three fixed root finds to 1e-12 | adaptive-method implementations, idiomatic track |
| W6 unit conversions | fixed table of 50 conversions across length, mass, energy, temperature | epher's `expr in unit` strength; the feasibility matrix shows which of the nine do units at all |
| W7 complex arithmetic | fixed set of 200 mixed complex expressions | several calculators do this natively |
| W8 date and astronomy | lunar phase for 100 fixed dates; sunrise/sunset for 20 fixed date/latitude pairs; Julian day round trips | mirrors epher's astronomy script categories (its largest at 131 scripts); expressible fully in Octave, partially elsewhere |

### Script processing (W9a to W9e)

Implemented in each scriptable tool's own language: epher script files,
NumWorks MicroPython, GNU Octave .m files, and GeoGebra scripting
(feasibility in stage 2; if GeoGebra's scripting cannot express these
tasks cleanly, it is not applicable and the report says so). With HiPER,
TI-Basic, HP PPL, and Wolfram Language out of scope (Decisions), the
scriptable set is epher, NumWorks, Octave, and GeoGebra. Tools with no
user scripting (SpeedCrunch, Desmos) are not applicable here, and the
report says so plainly.

| Task | Spec | Probes |
|---|---|---|
| W9a sieve | primes to 10,000, count them | loop + array speed |
| W9b fixed-point iteration | iterate x = g(x) to 1e-12 on a fixed g, cap 1,000 iterations | numeric convergence loop |
| W9c string processing | reverse words and balance parens over a fixed 1,000-line input | string library and allocation behavior |
| W9d lunar phase from date | compute phase angle for the 100 fixed dates of W8 in a loop | the shipped-script workload |
| W9e branchy accumulator | 50,000 iterations of a loop with conditionals and a running accumulator | pure interpreter overhead |

Each W9 implementation is a committed source file per tool
(scripts/bench/implementations/<tool>/), so anyone can read exactly what
was timed.

### epher-only sweep (not cross-app)

All 532 shipped scripts timed end-to-end, one pass, medians per category.
This extends scripts/perf-sweep (which already times 10 astronomy scripts
against CI budgets). It is epher's own regression and inventory
measurement; it is never ranked against the reference apps because they
do not share the catalog.

### Interactive latency (W10, included)

Included in the first run (decision 2026-11-07):

- launch-to-ready: process spawn to first prompt or first answer (N
  tier); page load to first rendered result (W tier via the DevTools
  protocol).
- keystroke-to-answer in interactive surfaces (epher REPL/TUI, Octave
  CLI, SpeedCrunch): automated only if the stage 2 automation spike
  shows it can be driven reliably; a task that cannot be automated
  reliably is dropped rather than published flaky.

## Measurement protocol

- **Clocks.** Wall time from a monotonic clock around the measured span;
  CPU time from the OS (wait4/getrusage on the harness's child process)
  for N-tier end-to-end runs; performance.now() inside the page for
  W-tier.
- **Memory.** N-tier: child-process peak RSS, measured uniformly by the
  harness (not by per-tool introspection, which would compare different
  definitions). W-tier: JS heap via the DevTools protocol. E/H: not
  measured, disclosed.
- **Statistics.** Every cell publishes min, median, median absolute
  deviation, and p95 from the committed raw samples. Comparative claims
  ("epher is faster than X at W3") additionally require a bootstrap
  confidence interval or a Mann-Whitney U test at alpha 0.01, with the
  method chosen per claim before it is checked.
- **Outliers.** No trimming. Samples beyond 10x the median are flagged
  and the run is repeated once; both runs stay in the record and the
  repeat is the reported one only if the flag reproduces.
- **Result format.** One JSON file per tool per session
  (results/raw/<date>/<tool>.json): tool, version, tier, task, track,
  boundary, samples, units, seed, machine manifest hash, timestamp,
  notes. Reports are generated from these files; no hand-copied numbers.

## Reproducibility artifacts

- `scripts/bench/` layout: one directory per tool (driver + README
  naming the exact invocation), shared runner
  (`scripts/bench/run <tool|tier|all>`), data generator, implementations
  per tool, and results/raw/ committed to the repo.
- Environment manifest committed with each session: hardware model,
  macOS build, power and thermal state, browser and build, every tool
  version, harness commit hash.
- Acceptance criterion for the whole effort: an independent re-run of
  the harness on the same machine reproduces published medians within
  10 percent, or the report explains the divergence. Re-run commands are
  one-liners documented in scripts/bench/README.md.
- The existing perf-budget workflow stays what it is (a CI regression
  gate for epher); the benchmark harness is a measurement instrument and
  gates nothing.

## Limitations, declared in advance

- Emulator fidelity: only NumWorks' emulator remains in E tier, and only
  after its hardware-faithfulness claim is verified against their
  documentation in stage 2. E-tier numbers are observational, never
  ranked against N.
- Coverage: the benched reference set is SpeedCrunch, GNU Octave,
  GeoGebra, Desmos, and NumWorks. Wolfram, the TI-84 Plus CE /
  TI-Nspire CX II, HP Prime, and HiPER are outside the bench (see
  Decisions); every report table discloses this so no absence reads as
  a claim about those products.
- Energy use, GPU/plot frame rates, and UI smoothness are out of scope.
- A benchmark measures what the harness measures; every table states its
  timing boundary, and no table mixes boundaries.

## Stage plan

| Stage | Work | Exit criterion |
|---|---|---|
| 2. Feasibility and access | per-tool x per-task feasibility matrix; GeoGebra license terms verified; NumWorks emulator clock-fidelity check and go/no-go; automation spike per tool, including the W10 drivers | matrix and go/no-go per tool committed |
| 3. Harness and pilot | drivers for three tools (epher, Octave, SpeedCrunch), W1 + W2 + W9a on both tracks; same-day re-run for stability | pilot JSON committed; medians reproduce within 10 percent |
| 4. Full run | all feasible tools x all feasible tasks, both tracks, two sessions on separate days | complete raw JSON for both sessions |
| 5. Report | docs/research/calculator-benchmark-results.md in gap-analysis house style: per-task tables within tiers, every claim carrying tool version, boundary, and sample count | report reviewed by Pete; site follow-up only on request |

## Decisions (2026-11-07, Pete)

The five open questions this plan first posed are decided:

1. **MATLAB and Wolfram: no licenses.** The MATLAB/Octave column is GNU
   Octave only, labeled as such everywhere; Wolfram Language is not
   timed, and WolframAlpha (a network service) was never in timing
   scope.
2. **No hardware calculators.** The TI-84 Plus CE / TI-Nspire CX II and
   HP Prime are excluded from the benchmark process entirely: no
   hardware, and no emulator licenses to stand in for it.
3. **HiPER excluded.** It would have required an Android-emulator tier
   for a single app; not worth it.
4. **W10 interactive latency is included** in the first run, subject to
   the reliability caveat above (a driver that cannot automate a
   surface reliably drops that measurement rather than publishing
   flaky numbers).

The resulting bench set: epher (native and web), SpeedCrunch, GNU
Octave, GeoGebra (desktop and web), Desmos, NumWorks (web; emulator
go/no-go in stage 2).
