# ADR-0071: the result pane re-renders what it receives

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-10-28

## Context

ADR-0056 routes long answers to the result pane, one answer per line.
The pane renders each answer as a plain monospace line at one visual
weight. Running the entire bundled script collection (477 scripts,
4,189 transcript lines, `docs/research/result-pane-presentation.md`)
showed what that costs:

- 44% of transcript lines are bare echoes of a script's `const` knobs
  and helper scratch, rendered at the same weight as results; 110
  scripts open with five or more bare lines before the first labeled
  result (worst: 40, `astronomy/ephemeris/schlyter-crosscheck`).
- Scripts end with checks and side notes, so "the last line" is often
  not the answer.
- Matrices print as one flat line (`{{1, 10, 78.85, 921.15}, …}`) —
  the longest line in the corpus is a 12×4 amortization schedule at
  305 characters — and long lists wrap without structure.
- The terminal pane had no wrap at all: a line wider than the pane
  clipped mid-number at the pane edge (the web's `pre-wrap` breaks at
  spaces, so the web mostly wrapped acceptably; the terminal did not).
- A 51-line transcript opened scrolled to its least informative 31
  lines.

The other tools people compute with converged on the same answers
long ago (survey in the research doc): MATLAB and every REPL dim or
suppress machinery; Jupyter and tibbles truncate with disclosure;
Mathematica and the handhelds render matrices as grids; Wolfram
discloses precision on demand. The pane does none of it today.

## Decision

The result pane re-renders what it receives. The transcript text is
unchanged — every statement's value stays exactly as the engine
produced it, and every copy path (the transcript button, the new
per-line button, the CLI) sends the original text verbatim. What
changes is presentation, decided by one pure contract in
`epher-core::pane` that both the web pane and the terminal pane call:

- **Two tiers.** A line carrying a `label: value` colon renders at
  full weight; a bare statement echo renders dimmed (the web's theme
  `--muted` ink, AA-contrast in every theme; the terminal's hints
  dim). Optionally a "show every value" path restores parity with the
  CLI.
- **The answer stands out.** The transcript's *last labeled line*
  renders emphasized — the web bolder and larger, the terminal in the
  result line's accent style.
- **Structures render as structures.** A line whose value is a matrix
  (`[[…]]` or `{{…}}`, at least 2×2) renders as a right-aligned grid,
  one row per line; a flat list longer than twenty elements folds
  behind a real "n more" button. Copy keeps the original line.
- **Wrap at values, never mid-value.** The terminal gains boundary
  wrapping with a hanging indent (tokens wider than any row chunk, so
  nothing clips); the web gains wrap points and a hanging indent, and
  keeps `overflow-wrap: anywhere` as the last resort for a single
  token wider than the pane. ISO timestamps render as one unbreakable
  token.
- **Affordances.** The pane names its size ("7 answers"; the
  terminal's title reads "Result · 7 answers"), scrolls to the first
  labeled line when a long transcript arrives, and copies per line on
  hover (always visible on touch).
- **Typing.** `: true` / `: false` verdicts wear a check (✓ accent /
  ✗ muted, the glyph `aria-hidden` — the text stays the truth); a
  trailing measure (`= 12 m`) dims; digit grouping on display follows
  the existing separators Results setting.
- **Short numbers, opt-in.** A pane-header toggle renders numeric
  tokens at six significant digits with an ellipsis and the full value
  on hover. Default off; copy is never shortened.

`print` is a builtin that returns a string value, so the transcript
carries no structural print marker; the tier test is the `": "` colon
(correct for 475 of 477 bundled scripts). A later engine change may
tag print-derived strings — the renderer would switch to it without
changing its presentation.

R8 of the research doc (a script-side `printtable`) stays out of
scope here: that is the future script-authoring exercise.

## Consequences

- The routing rule, the answer line, the CLI, and the scripts are
  untouched: a staged installers byte-diff shows no transcript
  changes, only presentation. CLI output is byte-identical.
- The pane-result section gains a meta row (count, short-numbers
  toggle) and per-line buttons — all real buttons, keyboard operable;
  the section remains `role="status"`. Dimmed setup lines use each
  theme's muted ink at documented AA contrast (6.3:1–7.6:1); they are
  content, not decoration.
- Elision and short-numbers are opt-in per pane (the unfold is per
  transcript, reset when a new result arrives); nothing hides by
  default except the visual dimming of setup echo.
- Fluent's directional isolates are invisible to the Paragraph but
  paint in the terminal's block titles; the terminal strips them
  there.
- New localized keys in all eight locales: `answers-count`,
  `more-values`, `short-numbers`, `copy-line`; the user guide's
  result-pane paragraph (§2.2, all eight locales) names the tiers.
- Tests: 17 native contract tests in `epher-core::pane` (tiering,
  wrap guards, grid parsing of both matrix flavors, padding,
  rounding, grouping); 5 terminal tests (dim/answer styles, grid
  rows, list wrap, wide-answer wrap, title count); the web pane
  compiles for wasm and renders through the same contract.

## Amendment (2026-10-28): the research doc's mid-digit claim, corrected

The research doc said the web breaks numbers mid-digit on long lists.
Measuring the rendering path: `white-space: pre-wrap` already breaks
at the spaces after commas, so the web's element-boundary wrapping was
mostly correct before; mid-digit breaking happened only for a single
token wider than the pane (rare in the corpus) and — the real
offender — in the terminal, which had no wrap and hard-clipped. The
doc's statistics and taxonomy stand; its R2 wording over-stated the
web case. The implementation carries both halves anyway: terminal
boundary wrap (the fix that matters) and web wrap points as cheap
insurance.
