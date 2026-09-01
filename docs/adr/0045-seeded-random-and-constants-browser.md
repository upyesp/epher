# ADR-0045: seeded random numbers and the constants browser

- Status: accepted
- Date: 2026-09-02
- Roadmap: feature-gap analysis round 4 (quick-win leftovers: the
  T2.2 seeded random that shipped with the distributions, and the T2.6
  constants browser; the CODATA set itself landed with ADR-0042)

## Context

Round 3 shipped the T2.2 probability distributions but not their
companion: seeded random numbers. Eight of the nine researched apps
offer `random()`/`randint`-style draws (every one except SpeedCrunch),
and the "random numbers" row of the expectation matrix is the last
unfilled item in the top block. The T2.6 constants library is two
halves: the standard physics/chemistry set (CODATA-backed values
already in the grammar since ADR-0042, ~35 builtins) and the discovery
surface the report asks for — SpeedCrunch's Ctrl+Space pattern: a
browser that lists the constants in groups and inserts a chosen name.
Typing a name is impossible when the name is unknown; the guide table
is not browseable from the calculator.

## Decision

### Seeded random numbers (T2.2 companion)

- `Env` gains the RNG state: a `Cell<u64>` SplitMix64 counter, seeded
  with a fixed constant by `Env::default()` (so `evaluate()` and the
  tests are deterministic) and with the system clock by
  `Session::new()` (so interactive sessions do not replay the same
  sequence). The seed is part of the session environment, not of a
  script, matching how `ans` and variables live.
- New builtins:
  - `random()` — uniform in `[0, 1)`.
  - `random(a, b)` — uniform real in `[a, b)`.
  - `randint(a, b)` — a uniformly chosen whole number in the closed
    range `[a, b]` (whole-number arguments, `a <= b`, no modulo bias:
    Lemire's rejection method).
  - `randseed(n)` — re-seeds the generator with the whole number `n`
    and returns it, so the seed is visible in history and a script can
    chain draws off one fixed point (`randseed(7)` then `random()`
    always gives the same first draw in every frontend).
- Draws are ordinary f64 values; everything else about the language is
  untouched. The keypad stays frozen; the three names enter the
  catalog and autocomplete.

### The constants browser (T2.6)

- Core exports the grouping the browser needs:
  `builtin_constant_groups() -> &'static [(&'static str, ConstGroup)]`
  — every builtin constant with its group (Math, Astronomy, Physics,
  Chemistry), the single source of truth for the frontends' browsers.
  The `ConstGroup` mirror of the guide's tables means the browser
  never drifts from the documented set.
- The CODATA/IAU set gains the remaining standard names so the groups
  are complete: Planck mass/length/time (`m_P`, `l_P`, `t_P`),
  classical electron radius (`r_e`), Compton wavelength (`lambda_c`),
  nuclear magneton (`mu_n`) in Physics; lunar mass/radius
  (`m_moon`, `r_moon`) in Astronomy. All SI units like their peers.
- Web app: the Help menu gains **Constants**, opening a dialog in the
  guide-dialog style: the groups as headed sections, each row a
  button with the name, its value (formatted with the session's
  display prefs), and the FTL hint line; a filter box narrows by name;
  activating a row inserts the name at the end of the entry field and
  refocuses it, and Escape closes. The dialog is a normal button list
  (focusable rows, aria roles) like the guide overlay.
- TUI: the Help menu gains **Constants** too, opening a browser view
  in the guide-pager style: grouped rows, arrow-key selection,
  Enter inserts the selected name into the input line, Escape closes.
- CLI: unchanged — `help` and the guide tables already cover it.

## Consequences

- Randomness is deterministic under `evaluate()` and testable to the
  draw; interactive sessions diverge by clock seed, like TI's
  randSeed default.
- The browser adds three new UI strings per locale (menu label,
  group names, filter placeholder) plus the nine new key-hint lines;
  the guide gains the random section and the new constant rows.
- No new dependencies: SplitMix64 is a few lines of integer
  arithmetic, wasm-safe, and free of the platform `rand` differences.
