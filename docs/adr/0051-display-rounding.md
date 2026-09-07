# ADR-0051: display rounding — twelve significant digits, decimals stay decimal

- Status: accepted
- Date: 2026-09-02
- Roadmap: feature-gap analysis T1.4 (exact results by default) follow-up

## Context

Exact fractions on by default shipped (ADR-0043) with a reconstruction
rule that showed a fraction for every value with a good
small-denominator convergent. That made `0.1 + 0.2` display as `3/10`
and `0.1` as `1/10` — and with exact fractions off, the same results
showed the raw float noise (`0.30000000000000004`). None of the nine
reference calculators does either: they compute in binary floats too,
then round the display to ~12 significant digits, so `0.1 + 0.2`
shows `0.3` and `1/3` shows `1/3` only in exact mode.

## Decision

- **Auto notation rounds displayed floats to twelve significant
  digits**, the NumWorks/TI/HP/SpeedCrunch/HiPER convention, and only
  when the rounded spelling is shorter than the shortest round-trip
  decimal. The guard protects exact integers: 1234567890123456 keeps
  every digit (its rounded spelling is no shorter).
- **A terminating decimal displays as a decimal** — a reconstructed
  fraction whose denominator reduces to only 2s and 5s (`3/10`,
  `1/8`, `2001/10`) spells as `0.3`, `0.125`, `200.1`. Only a
  repeating value keeps the fraction (`1/3`, `2/3`, `1/7`). This
  applies to the result line, table cells, matrices, and complex
  parts alike.
- The rounding works on the decimal spelling, not the float: scaling
  a double and back can land on a value whose shortest spelling still
  needs more digits (5.551115123125783e-17 comes back as
  5.551115123130001e-17).
- `exact(x)` keeps reconstructing on request, `frac`/`dec`/`big`
  keep their exact layers, and the notation modes (scientific,
  engineering) and display verbs are untouched. Values underneath
  stay floats (ADR-0005).

## Consequences

- `0.1 + 0.2` shows `0.3` with exact fractions on or off; `sqrt(2)`
  shows `1.41421356237`; `pi` shows `3.14159265359`; table cells
  follow the same rule. The guide's twelve-digit reference values
  (normcdf 0.975002104852 and friends) now match the app exactly.
- The fraction rule is display-only and cheap; no engine change, no
  provenance tracking.
