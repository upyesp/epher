# ADR-0086: the REPL edits its line like a shell

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-11-20

## Context

`run_repl` read stdin with `lines()`: a buffered, line-at-a-time loop with
no line editing. On a real terminal the arrow keys send escape sequences
the loop never interprets, so:

- Left and Right could not move the cursor inside the line; fixing a typo
  meant retyping the whole line.
- Up and Down could not recall earlier lines, although the shared store
  keeps every session's history (ADR-0040) and the TUI and desktop
  already recall it.
- Ctrl-C killed the whole process instead of abandoning the line.

The Python REPL's own behavior is the user expectation: emacs-style
movement inside the line, history recall with the arrow keys, Ctrl-C
abandons the current line for a fresh prompt, Ctrl-D ends the session.
A REPL that cannot edit its line reads as broken on every platform,
Windows included.

## Decision

On a terminal, the REPL reads its lines through
[rustyline](https://github.com/kkawakami/rustyline) (readline-style line
editing, cross-platform: termios on Unix, the Windows console API on
Windows).

- `run_repl` branches on `stdin.is_terminal()`. The terminal reader,
  `run_repl_interactive`, gives rustyline the localized prompt and the
  line; a piped reader, `run_repl_piped`, keeps the old
  `lines()` loop, prompt included, byte-for-byte. Piped sessions
  (`printf '...' | epher repl`, the CLI tests, scripted invocations)
  behave exactly as before; `epher -` still refuses a terminal stdin.
- The per-line semantics (`quit`/`exit`, `load`, `graph`, tables,
  statements, language switching, per-line persistence) move to one
  shared `repl_line` used by both readers, so the two paths cannot
  drift.
- History is rustyline's in-memory list, seeded at startup from the
  session history in the shared store and fed back through the existing
  `session.record` / `save_history` path. Up-arrow reaches previous
  sessions' lines on day one; no `.epher_history` file appears
  (ADR-0027's amendment retires history files; the store is the
  archive).
- Ctrl-C (`ReadlineError::Interrupted`) abandons the current line and
  prints a fresh prompt; Ctrl-D (`ReadlineError::Eof`) ends the session,
  like EOF did.

## Alternatives considered

- **Hand-rolled editor on crossterm** (already a TUI dependency):
  cursor movement, unicode widths, bracketed paste, Windows console
  quirks, and escape parsing are weeks of edge cases for behavior
  rustyline has shipped for a decade. Rejected.
- **No new dependencies** (the ADR-0045 habit for small features): a
  line editor is not small; the dependency is the feature. rustyline is
  MIT, self-contained, and purpose-built. Accepted.

## Consequences

- Arrow keys, Home/End, Ctrl-A/E/K/U/W and history recall work in the
  REPL on macOS, Linux, and Windows, matching the Python REPL's
  expectations.
- The CLI gains one dependency (`rustyline`); no new binary surface,
  no new files, no history file.
- Piped REPL behavior is unchanged and still test-verified; terminal
  behavior is verified by hand (a pipe cannot enter raw mode), noted in
  the 0.9.14 train checklist.

## Related (2026-11-24)

ADR-0093 gives the TUI, the desktop app, and the PWA the same recall
from an empty entry. There Down walks older and Up newer, because those
frontends show the list newest on top and the highlight moves the way
the key points; the REPL keeps rustyline's shell convention (Up is
older) because it shows no list.
