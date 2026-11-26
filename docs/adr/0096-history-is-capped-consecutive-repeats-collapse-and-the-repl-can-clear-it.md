# ADR-0096: history is capped, consecutive repeats collapse, and the REPL can clear it

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-11-26

## Context

Every native surface (CLI, REPL, TUI, desktop app) writes its lines into
one shared history, `setting/history.json` in the store (ADR-0010), as a
flat JSON list of display strings, oldest first, each entry the line as
typed followed by its last answer (`2 + 2  = 4`), its error
(`1/0  error: division by zero`), or nothing (a `graph` or `def` line).
The PWA keeps the same shape in its own sandboxed storage. An assessment
on 2026-11-26 (ADR-0086 amendment, same day) found three policy gaps
that no surface handles and that no ADR decides:

- **Nothing bounds the list.** No crate caps, prunes, or truncates it.
  `Session::history` grows for the life of the installation, and every
  submitted line rewrites the whole file. A daily user's history reaches
  thousands of entries; the TUI and desktop list renders all of them and
  rustyline seeds all of them into the REPL's editor at startup.
- **Repeats pile up.** Neither `Session::submit` nor `Session::record`
  compares against the previous entry, so re-running a line, the most
  common thing a calculator user does, appends an identical copy every
  time. One inspected store held five consecutive `6+3  = 9` entries.
  Every shell since `ksh` collapses consecutive duplicates.
- **The REPL cannot clear history.** The TUI has Ctrl+L with a one-slot
  undo (ADR-0094), the desktop app and the PWA have their clear control,
  and all three persist the clear to the store. The REPL has no command
  and no key, so a user whose history the old REPL seed corrupted (the
  ADR-0086 amendment) could not remove the damage from the surface that
  caused it.

## Decision

- **Cap.** `Session` keeps the most recent `HISTORY_MAX` entries
  (1,000, bash's default `HISTSIZE`); when a push would exceed the cap,
  the oldest entry is dropped. The cap applies on write, in `Session`,
  so every surface inherits it and the store file never grows past it.
  Loading a larger history from an older store trims it on load; nothing
  is lost that the cap would not have dropped anyway.
- **Consecutive repeats collapse.** A push equal to the last entry (the
  full display string, so `2 + 2  = 4` twice is a repeat but
  `2 + 2  = 4` then `2 + 2  = 5` after a definition change is not) moves
  the existing entry to the end instead of appending a copy. The order
  of recency is kept, which is what recall walks follow; the count is
  not. Non-adjacent repeats stay: they are a real record of what the
  user did.
- **`clear history` is a shell command**, classified in `epher-shell`
  beside `reset`, that empties `Session::history` and persists the
  empty list through the existing per-line save. The REPL accepts it
  like any other command; the TUI, the desktop app, and the PWA route
  it through the same classifier, so one spelling clears history on
  every surface. `reset` is unchanged: it forgets definitions and keeps
  history (ADR-0025), and the two names now read as the pair they are.
- The REPL keeps its own clear gesture too: Ctrl+L, matching the TUI
  (ADR-0094), through rustyline's `ControlL` binding, which runs the
  same `clear history` path. No undo slot in the REPL: the TUI's
  one-slot restore exists because Ctrl+L sits beside other destructive
  keys in a menu-driven interface; a typed command is its own
  confirmation.

## Consequences

- The store file has a known ceiling, the TUI and desktop lists stay
  scrollable, and the REPL's startup seed stays bounded.
- A user who runs the same line fifty times has one entry, not fifty;
  the Up-arrow walk reaches further back into real work.
- Every surface can clear history with the same words. The REPL gains
  its first history-management gesture, and the damage the old seed
  caused is now removable from the REPL itself.
- `HISTORY_MAX` is a constant, not a setting. If a user ever needs to
  raise it, that is a new ADR; the default is the one bash ships.
- Entries are still display strings. Structured entries (`expr`/`out`
  pairs, which would let the lists show answers while recalling only the
  expression) remain a possible future ADR; the cap and the collapse
  are compatible with either shape.

## Related

- ADR-0010 (the shared store), ADR-0025 (`reset`), ADR-0027 (multi-line
  entries), ADR-0086 (the REPL's editor; its 2026-11-26 amendment fixed
  the seed this ADR's clear command cleans up after), ADR-0093 (the
  arrow walk), ADR-0094 (guarded destructive keys).

## Implementation notes (2026-11-26)

Implemented the same day, with three details the decision above left
open:

- **The collapse keeps the entry where it is** rather than moving it to
  the end: when the repeat *is* the last entry, those are the same
  position, and the rule reduces to "an entry equal to the last one is
  not pushed". `push_history` is the one door; `submit`, `submit_all`,
  `record`, and `restore_history` all go through it, so the cap holds
  through ADR-0094's restore too.
- **Ctrl-L clears on the key press, not on the next Enter.** rustyline
  runs a custom key handler inside its editor, out of reach of the
  session; the first cut had the handler repaint and the loop clear
  after `readline` returned, which a pty test showed only happens on
  the *following* Enter, by which time the user had typed `quit` onto a
  stale recalled line. The handler now raises a flag and returns
  `Cmd::AcceptLine`, so `readline` returns at once; the loop sees the
  flag, clears both lists (the editor's in place, so the arrows stop
  recalling immediately; the session's, persisted), and discards the
  buffer: Ctrl-L never submits a half-typed line.
- **The web persists inside its command arm.** The desktop/PWA submit
  closure works on a local session and publishes once after its loop;
  the command arm `continue`s past that epilogue, so `clear history`
  publishes and persists itself there, handle for handle the way the
  clear button does (`session.set`, `bridge.save_history(&[])`, the
  PWA's `epher-history` and `epher-result` keys). The TUI saves after
  `run_command` returns; the CLI's per-line save covers the REPL.

The user guide's History section (2.3) and REPL section describe the
cap, the collapse, `clear history`, and Ctrl+L in all eight languages.
- **One spelling of an entry.** `Session::record_with_output(line, out)`
  owns `line  = answer`; `submit`, `submit_all`, the TUI, and the
  desktop/PWA submit path all record through it, so the three places
  that used to format the string themselves cannot drift from
  `history_expression`, the stripper that undoes it. A test pins the
  pair: what the door writes, the stripper reads back exactly.
