# ADR-0094: destructive keys are guarded

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-11-25

## Context

A keyboard audit of the 0.9.24 TUI found two destructive gestures that
fire instantly:

1. `q` quits the TUI whenever the entry is empty; the footer hint
   names it (`q quit`). `q` is an ordinary letter. A user who focuses
   the history list and then decides to type, or who clears the entry
   to start over, quits the whole screen with no confirmation and no
   transition. Ctrl+C stays the terminal's own emergency exit and the
   File menu keeps its Quit item (ADR-0023), but a bare letter that
   tears down the app is below the bar every other surface sets:
   the REPL, the desktop app, and the PWA have no equivalent.
2. Ctrl+L and the trash glyph in the history heading (ADR-0041) erase
   the entire history, whole sessions of work included, with no
   confirmation and no undo. The history lives in the shared store
   (ADR-0010), so the clear also propagates to every open frontend
   the moment it lands.

No other part of epher destroys work on a single unmodified keypress.

## Decision

**Destructive keys confirm themselves in the status line, not in
dialogs.**

- `q` on an empty entry shows `Press q again to quit` in the status
  tier (ADR-0079 amendment); a second `q` quits. Any other key, a
  submission, or Esc cancels the pending quit. Ctrl+C and File → Quit
  stay immediate.
- The first Ctrl+L, or a trash click, clears the history and the
  status line offers `History cleared · Ctrl+L restores`; the cleared
  entries are kept in memory and a second Ctrl+L restores them. The
  clear still propagates to the shared store immediately, so the store
  stays truthful, and the restore propagates the same way. The offer
  stands until the next evaluation replaces the status line.

## Alternatives considered

- A confirmation dialog or yes/no prompt for both actions (rejected:
  modal cost for everyday gestures; the status line is already the
  app's channel for "the app speaking", and ADR-0033's no-layout-budget
  rule favors reusing it).
- Rebinding quit to Ctrl+Q only (rejected: `q` in place with a
  confirmation keeps the discoverable, documented spelling and costs
  one extra press exactly when it matters).
- An undo stack for the history (rejected: one restore slot covers the
  real accident, "I cleared everything", without generalizing into a
  command history the store does not model).

## Consequences

- Quitting by `q` is two presses; the hint line changes to match
  (`q quit` keeps its place, the status line does the explaining).
- Ctrl+L becomes reversible at zero modal cost: no dialogs, no focus
  changes, no new surfaces, localized through the existing catalogs.
- The trash click and Ctrl+L share the guard, so the icon and the
  chord cannot drift apart.
