# ADR-0093: from an empty entry, the arrow keys walk the history

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-11-24

## Context

The REPL learned readline-style line editing (ADR-0086): on a terminal
the left and right arrows move inside the line, and up and down recall
earlier lines, seeded from the shared store so yesterday's work is one
key press away. The TUI, the desktop app, and the PWA already show the
history as a list under the answer, clickable in every one of them
(ADR-0027), and the TUI's Tab reaches that list and moves a selection
with the arrows (ADR-0033 amendment). But none of the three lets the
arrows recall a line from the entry itself: a user who wants the
previous statement reaches for the mouse, or for Tab and Enter, or
retypes it. The REPL's gesture is the one every shell teaches, and its
absence in the apps reads as a gap, not a choice.

The constraint is that the arrows already mean something while the
entry holds text: the ADR-0035 amendment gives Left/Right to the caret
and Up/Down to the lines of a multi-line entry in the TUI, and the
web's textarea moves its caret natively. And on an empty entry the TUI
spent the arrows on the 3D orbit (ADR-0015), which the web never did
(its orbit is a drag).

## Decision

Every frontend with an entry and a history list (the TUI, the desktop
app, and the PWA, the last two one Yew crate) walks the history with
the arrow keys from an empty entry, under one policy,
`epher_shell::history_step`, shared like the shell commands (ADR-0010)
so the paths cannot drift:

- **While the entry holds text, nothing changes.** Left/Right move the
  caret, Up/Down move between the lines of a multi-line entry.
- **On an empty entry, Down recalls the newest line** into the entry
  (the expression only, the recorded answer stays out, as a click does;
  the TUI `;`-joins a multi-line script exactly as its picks do). Down
  again recalls the next older line; Up walks back toward newer. The
  arrows follow the list as it renders, newest on top, the rule the
  TUI's history focus already uses, so the highlight moves the way the
  key points. Up from the newest line returns the entry to empty (a
  shell's "past the newest" step, mirrored); Down from the oldest stays
  put, no wrap.
- **The walk outlives the first recall.** After the first Down the
  entry holds text, but Up/Down keep walking until the text changes by
  any other hand: a typed or deleted character, a paste, a click on a
  row, a submit, Esc. Left/Right and Home/End move the caret inside the
  recalled line and leave the walk in place.
- **The list shows where the walk is.** The recalled row wears the
  history list's selection highlight (the TUI's focus colors; the web's
  accent rule and panel tint, with `aria-current`), and the list
  scrolls to keep it in view (on the web, on desktop layouts only).
- **In the TUI, the plain Up/Down no longer orbit on an empty entry.**
  Left/Right still rotate the yaw, Shift+Up/Down take the pitch, and
  the mouse drag (ADR-0034) and the Settings fine controls (ADR-0031)
  are unchanged. The hint line narrows accordingly (`← → / Shift+↑ ↓
  rotate`) and names the walk (`↑ ↓ history`) exactly while the arrows
  would recall: the entry empty (or already walking) and a line to
  recall.

Clicks stay picks, not walks: a click loads its row and clears the
highlight, the same gesture it always was.

## Alternatives considered

- **A shell's direction (Up = older).** The REPL and every shell use it,
  and no list is on screen there. The apps render the list newest on
  top, so Up-for-older would send the highlight up the screen while the
  list reads down; the TUI already settled this for its history focus
  (ADR-0033 amendment). Rejected for the apps; the REPL keeps
  rustyline's convention.
- **Keeping the orbit on plain Up/Down when the history is empty.** The
  keys' meaning would flip the moment the first line is recorded, an
  inconsistency nobody can see. Rejected; the orbit keeps ← → and gains
  Shift+↑ ↓.
- **A click on a row starts a walk from that row.** Consistent with
  "the highlight is where the arrows continue", but it turns Up/Down
  into recalls right after a user clicks a multi-line script to edit
  its middle lines. Rejected; the walk is the keyboard's, from an empty
  entry.

## Consequences

- The REPL, the TUI, the desktop app, and the PWA all recall earlier
  lines from the entry with the arrows; in the apps the list shows
  which line the entry shows.
- The walk policy is one function with one test suite; the TUI and the
  web each carry only their own text spelling, highlight, and reset
  points. A browser regression (`scripts/browser/history-walk.mjs`)
  drives the real PWA build through the whole walk.
- The TUI's keyboard-only 3D pitch moves from plain Up/Down to
  Shift+Up/Down (terminals that do not distinguish Shift+arrow fall
  back to the drag and the fine controls); ADR-0015 is annotated.
- ADR-0086's behaviour is unchanged; its record points here for the
  apps' spelling.

## Amendment (2026-11-25): either arrow starts the walk

A keyboard audit of the 0.9.24 TUI found the walk's starting rule too
pure. On an empty entry, Down recalls the newest line and Up is a
silent no-op: there is no row above the newest, so the spatial model
has no step there. But every shell, and the TUI's own footer hint
(`↑ ↓ history`, shown exactly when the walk would start), promises
more than silence, and Up is the gesture every read-eval-print loop
teaches. A user reaching for it gets a dead key.

**Either arrow starts the walk.** From an empty entry, Down recalls
the newest line as before, and Up now recalls it too: the entry sits
above the list, and the first step in either direction lands on the
top row. From there the spatial rule governs unchanged, Down walks
older, Up walks newer, and Up past the newest returns the entry to
empty. `epher_shell::history_step` gains the same rule so the apps
cannot drift; the REPL keeps rustyline's shell convention (ADR-0086,
unchanged).

Alternatives considered: adopting the shell's direction wholesale
(rejected for the reasons the original decision records), and leaving
Up dead while the hint explains why (rejected: a hint explaining a
dead key is still a dead key).
