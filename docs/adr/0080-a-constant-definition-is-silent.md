# ADR-0080: a constant definition is silent

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-10-28

## Context

Every statement printed its value, `const` included. A script written
the way `epher scripts/README.md` recommends — `const` knobs at the
top, `print`-labeled results at the bottom — therefore opened with a
column of bare numbers whose meaning lived only in the source:

```text
= 3
= 4
= hypotenuse of the 3-4-5 triangle: 5     ← the only line a reader needs
```

Corpus measurement (477 scripts, fresh stores): 1,854 of 4,189
transcript lines (44%) are bare echoes of knob and intermediate
bindings; 125 scripts open with five or more, 110 of them leading
(measured on fresh data — see `docs/research/bundled-script-output.md`).
ADR-0078 dimmed these lines in the result pane, which helps the pane
but leaves the transcript, the CLI, the LSP inlay hints, and every
copy of the output carrying noise that the reader must learn to skip.

The language already draws this line elsewhere: `def` produces no
value because a definition is machinery, not an answer. `const` is
the same kind of statement — the knob convention exists precisely
because a constant is a setting.

## Decision

`const` defines silently. The binding is made exactly as before
(immutable, readable in later statements, refused when the name is
taken, re-declarable with the same value), but the statement
contributes no transcript line and leaves `ans` untouched — the same
contract `def` has.

Concretely, `Statement::Const` in `stmt_flow` runs `define_constant`
and returns `Flow::Normal(None)`.

## Consequences

- **Transcripts speak.** The corpus's expected-output blocks lose
  their knob column: 345 scripts, 1,058 lines removed (the only added
  lines are `*/` terminators the updater had dropped). `messier-tonight`
  drops from 51 lines to 17; `circle-area` from 4 to 3. Every removal
  is a bare knob echo — no labeled line, no computed loop value, and no
  `=` answer was touched (verified by reviewing the 345 regenerated
  blocks).
- **The CLI's one-shot semantics tighten.** `epher "const x = 1"` now
  prints nothing; `epher "const x = 1; x + 1"` prints `= 2`. The REPL
  shows no answer for a const line, and the LSP inlay hints drop the
  const line's hint while keeping the binding visible to the lines that
  use it.
- **`ans` is unaffected in kind.** A knob is not an answer: `ans`,
  `save script`, history, and the `const` re-declaration path all keep
  their behaviour; only the displayed value disappears.
- **Docs follow the engine, and are now checked.** The guide's §1.6
  and §7.2 and the `epher scripts` README describe the silence;
  `scripts/check-guide.mjs` replays every `epher` transcript block in
  the guide against the CLI (128 blocks) and runs in CI next to
  `check-scripts.mjs`. A language change that edits no documentation
  can no longer leave the guide describing an output the app does not
  print.
- **Assignments still print.** `x = 5` echoes `5` as before: the
  sweep covers the knob convention only, and loop-emitted values,
  destructuring, and bare expression answers are untouched — silencing
  statements wholesale would hide real results and break the
  every-answer-visible contract (ADR-0052).
- ADR-0078's R1 dimming stays: it still tiers the bare echoes that
  remain (assignments, intermediates) against labeled results.

## Alternatives considered

- **A `quiet` statement prefix adopted by the sweep.** No semantics
  change and no existing transcript edits, but it adds a keyword, must
  be applied to ~345 scripts by hand, and leaves the language's most
  common definition statement printing a value nobody reads.
- **Silence every definition-shaped statement (including `=`).**
  Rejected: assignments carry computed data in this corpus (loop
  results, destructuring), and the every-answer-visible contract
  depends on them printing.
