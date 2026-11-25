# ADR-0079: a failure is its own tier in the result pane

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-10-28

## Context

ADR-0078 tiers transcript lines by a colon: `label: value` lines are
results, bare echoes are setup. Failures have colons of their own;
`error: cannot define constant r: the name is already a variable`; so a
failed line classified as **Labeled**. Two consequences followed:

1. A paste that died on its first statement rendered at full result
   weight, and if it was the only line it became the transcript's
   **answer** (the last labeled line), emphasized like a result.
2. It looked like every other line: the pane gave no signal that the
   paste produced nothing.

Found while auditing script output: pasting a script whose `const r`
collides with a variable already bound in the session (ADR-0012 makes
a name either a variable or a constant, never both) fails the whole
paste. `submit_all` is fail-fast, so the GUI shows exactly one line;
and that line was dressed as a result.

## Decision

Failures get their own presentation tier.

- `epher_core::pane::line_kind` gains `LineKind::Error`, matched on a
  `error: ` / `warning: ` prefix before the colon test, so an error
  containing its own colons can never pass for a labeled line.
- `final_labeled_index` skips errors: a failed paste never wears the
  answer's emphasis, and a transcript of one error has no answer line.
- The web pane renders `.pane-answer-error` in a dedicated `--danger`
  token per theme (light `#a3231a` 7.0:1, dark `#ff7b72` 6.2:1, night
  `#ffb3a8` 12.1:1; all AA on their canvas; night keeps its
  long-wavelength rule). The terminal renders errors in a warm red,
  bold: `LightRed` on dark, `#961e1e` on light, `#ff9682` on night.
- The constant-collision message names the remedy: "this session
  already bound it; Clear resets the session, or rename the binding".
- The pane title appends its answer count only when the transcript has
  more than one line, so a single failed line does not read
  "Result · 1 answers".

## Consequences

- A paste that produced nothing is unmistakable and unemphasized.
- Errors still occupy a display line in the transcript and still copy
  verbatim; only their styling and their eligibility for emphasis
  changed.
- The CLI's error text gains the parenthetical remedy, which appears in
  every frontend and in the injected error strings.
- No script changes: the corpus was verified clean (477/477); the
  collision is a session-state hazard for users who paste several
  scripts in one session, not a defect in any bundled script.
- Tests: two contract tests (`error_lines_are_their_own_tier`,
  `an_error_line_never_takes_the_answer_emphasis`) and one terminal
  test (`a_failed_paste_renders_in_the_error_style`).

## Amendment (2026-11-25): the tier covers the answer line, and action feedback gets a status tier

A keyboard audit of the 0.9.24 TUI found this ADR's tier living only
in the multi-line pane. An error that fits on the one-line answer
(`error: division by zero`) rendered in the answer's green bold, and
so did every action message: saved files, cleared graphs, the paste
hint, the F1 function help. By style alone, the app's failure and its
small talk were indistinguishable from its answers.

**Decision.** The failure tier is presentation-wide: wherever a result
renders, an `error: `/`warning: ` prefix selects the failure style,
the one-line answer included, and the file-prompt failure messages
join it. Action feedback that is not an evaluation gets its own
status tier: copied, saved, cleared, restored, and hint texts render
dimmer than any answer, in the hints' ink, so a status reads at a
glance as "the app speaking", never as "the calculator answering".
The status tier also carries the guards ADR-0094 adds (quit
confirmation, history restore), which is what makes those guards
visible without dialogs or modes.
