# ADR-0021: `ans` — the previous answer

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

Both keypads (web and TUI) have carried an `ans` key since ADR-0016 —
the pocket-calculator convention for "the previous answer" — but the
language never defined the name: pressing the key inserted the token and
evaluation answered `error: unknown name: ans`. A key that errors is
worse than no key.

## Decision

Every value-producing statement records its result as the ordinary
variable `ans`, at the single statement-execution seam the top level and
loop bodies share:

- `2 + 3` then `ans * 2` evaluates `ans` as 5; in a multi-statement
  script `ans` is the previous statement's value.
- Statements that produce no value (`def`) leave `ans` untouched; so do
  errors — a failed line never clobbers the last good answer.
- `ans` is a normal variable: it lives in the session environment, is
  never persisted to the store, and can be assigned like any other
  variable. Before the first successful result it is simply undefined
  (`unknown name: ans`).
- Evaluation inside graph sampling, `table` rows, and condition
  checks does not update `ans` — only executed statements do, matching
  pocket calculators, where drawing or tabulating never moves `Ans`.

## Consequences

- The keypad `ans` keys now do what their label promises, in every
  frontend.
- One shared execution seam (`stmt_value`) — the previous duplication
  between `run_inner` and `execute_stmt` collapsed into it.
- Guide §1.5 documents the variable in all eight locales (two short
  code blocks per locale, audit-fenced identically).

## Amendment (2026-08-27): `ans` persists as part of the shared session snapshot

The "never persisted to the store" bullet is amended for the desktop
installation. Under the ADR-0010 amendment, the native store carries a
`session` setting — the environment's bindings, `ans` among them — saved
with every submitted line and restored at startup. `ans` therefore
travels between the CLI one-shot, the REPL, the TUI, and the desktop app
(last-write-wins, like every other store document). The browser PWA stays
session-only: its `ans` still lives and dies with the page. Everything
else in this ADR is unchanged: `ans` remains an ordinary variable (now a
shared one), statements that produce no value still leave it untouched,
and failed lines still never clobber the last good answer.

## Amendment (2026-08-27): `ans` travels live between open frontends

The session snapshot already persisted per line (2026-08-27 amendment
above). With the publish/subscribe store sync (ADR-0010 amendment), an
open frontend also **receives** the snapshot: `x = 5` typed in the TUI
updates `ans` and the variables in an already-open desktop app on the
next store write, and vice versa. Desktop-only, like the persistence
itself; the PWA remains session-only.
