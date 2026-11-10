# Calculator benchmark stage 3: harness and pilot

Date: 2026-11-07 · Status: findings only; stage 3 of 5
(docs/research/calculator-benchmark-plan.md)

## What stage 3 built

The measurement harness lives in scripts/bench/ (README there declares the
protocol in full): a fixed-seed data generator and a 60-digit decimal
referee (the parity report's stdlib-only method) producing
data/acceptance.json; an in-process timing binary for epher
(crates/core/examples/bench_eval.rs, links epher-core, boundary parse +
eval); committed implementation files for GNU Octave (generated batch and
verify files plus handwritten sieves) and epher (w9a-primes.epher); one
driver per tool, each of which runs the correctness gate before any
sampling; a preflight that records the environment manifest and enforces
the quiet-machine checklist; and a session runner whose results land
incrementally under results/raw/<date>/session-<a|b>/.

## Pilot results

Session a and session b ran the same day on the same machine (Apple
Silicon Mac Mini, macOS 26, AC power, lock guard active; manifests committed
alongside). Gate status first, because a failed gate publishes no time
(plan rule 2): every tool x task cell below passed its full gate in both
sessions - epher and GNU Octave against all 10,000 acceptance values per
task (plus the exact W9a count), SpeedCrunch against all 10,000 per task
read back from its history in 120-line windows. SpeedCrunch delivered
every chunk on the first attempt in both sessions (zero chunk retries).

Medians in milliseconds per pass over the whole 10,000-item dataset (W1,
W2) or one script run (W9a); boundary per the README:

| Cell | boundary | a | b | delta |
|---|---|---|---|---|
| epher W1 | in-engine parse+eval | 5.497 | 5.519 | +0.4% |
| epher W2 | in-engine parse+eval | 2.556 | 2.556 | 0.0% |
| epher W9a (trial division) | in-engine | 12.639 | 12.644 | 0.0% |
| GNU Octave W1 | read+parse+eval via tic/toc around run() | 31.663 | 31.910 | +0.8% |
| GNU Octave W2 | same | 26.791 | 26.790 | 0.0% |
| GNU Octave W9a idiomatic (vectorized sieve) | same | 5.814 | 5.742 | -1.2% |
| GNU Octave W9a equivalent (loop sieve) | same | 19.820 | 19.642 | -0.9% |
| SpeedCrunch W1 | GUI bulk paste, chunked | 15,539.7 | 15,607.9 | +0.4% |
| SpeedCrunch W2 | GUI bulk paste, chunked | 16,487.9 | 16,498.4 | +0.1% |

Sample counts: 30 per epher and Octave cell (all passes well under 1 s);
7 per SpeedCrunch cell (15 to 17 s passes). Raw samples, gates, retry
counters, boundary strings, and versions are in the committed JSON files,
which are the record; this table is their summary.

## Instrument findings worth recording

- The gate caught two real implementation bugs before any timing was
  recorded: both committed Octave sieves initially flagged 2 as composite
  (flags(2) = false), counting 1228, and the equivalent-track loop sieve
  degraded further from there; and the first epher W2 run failed 1,666 of
  10,000 lines - exactly the log() sixth of the dataset - because epher's
  log() is base-10 while the canonical spec means natural log. Both were
  fixed and re-gated before any samples were taken.
- epher's in-engine pass over 10,000 expressions takes about 5 ms, i.e.
  roughly half a millisecond per expression including parse. Its W9a
  trial-division script needs chunked for statements because the step
  budget is per statement (STEP_LIMIT 100,000, applied by the shell's
  script path per piece); the harness binary mirrors that per-statement
  budget model.
- GNU Octave's tic/toc had to move outside run(): run() parses the whole
  file before executing, so a timer inside the file measures execution
  only. The driver now times run() from outside, covering read + parse +
  evaluation; that boundary correction was made before any published
  number.
- SpeedCrunch automation facts (stage 2 spikes, extended here): the input
  line swallows Return while the autocomplete popup is open, so the driver
  never types - the reliable paths are multiline paste into an empty
  focused input (evaluates every line) and AX value injection. Multiline
  paste delivery itself proved non-deterministic when input was non-empty
  or focus had just moved, so the driver verifies the empty-input
  precondition before every paste, detects a swallowed paste exactly
  (the chunk's result count never arrives), and retries with a count. A
  fresh launch restores the previous session's history, so the driver
  clears once before counting. SpeedCrunch renders minus as U+2212 and its
  history accessibility value caps at about 4.5 KB, hence 120-line chunks
  with a clear between chunks.
- The SpeedCrunch timing boundary is GUI bulk ingest (per-chunk paste to
  all-results-present, summed) and includes the app's batch processing plus
  the driver's 30 ms poll granularity; it is not comparable to the
  in-engine cells and every table labels it as such.

## Stability (exit criterion)

The stage 3 exit criterion is that session a and session b medians
reproduce within 10 percent. Worst absolute delta across the nine cells:
1.2 percent (Octave W9a idiomatic); every other cell is at or under 0.8
percent. The criterion is met with a wide margin. Re-run with:

```
node scripts/bench/compare-sessions.mjs 2026-11-07 a b
```

## Deviations and disclosures

- W10 interactive-latency drivers and all W-tier drivers are stage 4 work
  per the plan's stage table (the stage 2 report had sketched W10 for
  stage 3; the stage table governs).
- SpeedCrunch's W1/W2 boundary is GUI bulk ingest, not in-engine; there is
  no scripting surface to time in-engine. Declared per cell.
- epher's W9a equivalent track is recorded as not supported (no list index
  assignment in the language); SpeedCrunch's W9a row is not applicable (no
  scripting). Neither publishes a time (plan rule 11).
- Smoke sessions (results under session-smoke*) were used to debug the
  instruments and were deleted before commit; only session a and session b
  are the record.
