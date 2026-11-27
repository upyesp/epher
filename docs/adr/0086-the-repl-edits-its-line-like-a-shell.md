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

## Amendment 2026-11-26: the editor is seeded with expressions, never the transcript

The decision seeded rustyline "from the session history in the shared
store" without saying which text: the seed loop passed the raw entries,
so Up recalled `2 + 2  = 4`, the display transcript, and Enter made it
`error: parse error: expected ';' or a newline between statements`,
which `repl_line` then saved back into the shared store for every other
surface to see. One inspected store held
`6+3  = 9  error: parse error: …` as its fourth entry. The TUI, the
desktop app, and the PWA never had the problem because each of their
recall sites strips the suffix through `epher_core::history_expression`;
the REPL was the one surface that forgot, and the piped REPL tests
could not see it because a pipe never enters raw mode.

- `Session::history_expressions()` is now the one recall door: the
  history with `history_expression` applied to every entry, multi-line
  scripts joined with `; `. The REPL seeds its editor from it
  (`repl_history_lines`), and the other surfaces' recall sites are the
  same text by construction.
- A line typed in the session joins the editor as typed; a line recalled
  from the editor is already an expression, so resubmitting it never
  compounds a second suffix. rustyline's default skip of a line equal to
  the previous one keeps a re-run from doubling the walk.
- Two unit tests cover the seed as a pure function, the case the pipe
  cannot drive; the fix was also verified on a pty against the real
  editor (Up recalls `6+3`, Enter yields `= 9`, the store gains
  `6+3  = 9`).

The REPL still has no way to clear a damaged history; ADR-0096 adds
`clear history` as a shell command on every surface. Everything else in
this ADR stands.

## Amendment 2026-11-27: foreign writes reload at the line boundary, and Up ends a stale read

The editor was seeded once, so recall froze at startup: entries other
frontends wrote while the REPL was open were unreachable by the
arrows, and the stale view's next per-line save erased them from the
store (ADR-0010 amendment, 2026-11-27, has the pub/sub contract and
the audit that found it). The reader now reloads the store's session
at every line boundary and rebuilds the recall list through the one
door (`repl_history_lines`), so the seed policy above governs every
reload, not just startup. Because rustyline cannot reload while a read
is in flight, Up ends a stale read: with a foreign write pending, an
Up key handler (the Ctrl-L `ConditionalEventHandler` pattern) ends the
readline, the loop reloads and restores the half-typed buffer
(`readline_with_initial`), and the very next Up walks the fresh list.
The Ctrl-L clear and Ctrl-C fresh-prompt semantics are unchanged; a
piped session reloads silently between lines, keeping transcripts
byte-identical. Interactive recall is pty-verified
(`scripts/repl-pubsub-harness.py`); a pipe still cannot enter raw
mode.
