# ADR-0097: a history entry records what was typed and what came back, not a display string

- **Status:** accepted (implemented the same day)
- **Deciders:** epher maintainers
- **Date:** 2026-11-26

## Context

A history entry today is the string the history list shows: the line as
typed, two spaces, then its last answer (`2 + 2  = 4`), its error
(`1/0  error: division by zero`), its warning, or nothing.
`Session::record_with_output` owns that spelling (the ADR-0096
implementation notes), `setting/history.json` holds a flat JSON list of
such strings under the `history` key (`crates/store/src/persist.rs`),
and the PWA keeps the same list in `localStorage["epher-history"]`.

The string serves two masters, and that is the problem.

- **What a user re-runs is not what the list shows.** Recall has to undo
  the format before the line goes back into an editor. The stripper,
  `epher_core::history_expression`, does `rfind` on three markers
  (`  = `, `  error: `, `  warning: `) and refuses to scan any entry
  that contains a newline. It runs at six places: the REPL's seed
  (`Session::history_expressions`), the TUI's `history_text`, two picks
  and the share link in the web crate, and `history_expressions`
  itself. The ADR-0086 amendment (2026-11-26) was one surface
  forgetting: the REPL seeded its editor with the display string, so
  pressing Up and Enter put the whole transcript back through the
  parser, which recorded `6+3  = 9  error: parse error: ...` in the
  store beside five clean copies of `6+3  = 9`. The fix put the strip
  behind one door per surface. The class of bug stays, because the
  stored form is the form that has to be undone.
- **The stripper is a heuristic with a tested edge.** A multi-line
  script is stored without its answer precisely because the stripper
  could not find the boundary. The TUI's own test uses a script whose
  second line reads `y  =  x + 5`, two spaces and all, and the newline
  guard is what keeps that line whole on recall. A table records as the
  bare line as well (its TUI test pins that). And the stripper trusts
  that no answer text ever contains a marker; nothing enforces that.
- **The markers are a protocol with no type.** The writer in core and
  the readers in three frontends agree on two spaces and three words by
  convention. ADR-0096 could add a cap and a collapse rule to the list
  without touching this. The next consumer, an export, an LSP recall,
  a share preview, has to learn the convention or compound the suffix.
- **Downgrade behavior is already defined.** `load_session` reads the
  history first and propagates a parse failure; the CLI and the TUI
  both turn that into `warning: could not load saved data (...);
  starting fresh` and an empty `Session`. Any format change inherits
  this path, and an ADR that changes the format has to say so.

## Decision

**A history entry is a record of two things: what was typed and what
came back.** The display string is derived, never stored.

- **The type.** `epher_core::HistoryEntry { expr: String, out:
  Option<String> }`. `expr` is the line exactly as typed, newlines
  intact, runnable verbatim. `out` is the answer as the surface showed
  it, keeping its leading `= `, `error: `, or `warning: ` (the
  result-pane tiers of ADR-0079 key on that prefix), or `None` for a
  line that produced no answer (a `graph`, `def`, or `const` line, a
  table). The kind is not a field: it is the prefix of `out`, and a
  `kind()` method derives it, so nothing can disagree with the tier.
  (The assessment that prompted this ADR sketched a `kind` field; it is
  derived instead.)
- **The doors.** `Session::history()` yields `&[HistoryEntry]`.
  `HistoryEntry::display()` is the one spelling of a list row (`expr`,
  two spaces, `out`; `expr` alone when `out` is `None`), and the list
  renderers in the TUI and the web crate call it, so rows look exactly
  as they do today. Every recall (`history_expressions`, the arrow walk
  of ADR-0093, the picks of ADR-0027 and ADR-0031, the share link of
  ADR-0038, the REPL seed of ADR-0086) reads `.expr`. `history_expression`
  leaves the recall path.
- **The 0096 rules compare the record.** Consecutive entries collapse
  when both `expr` and `out` are equal; a changed answer stays, as the
  existing rule and its test say. The cap is unchanged.
- **The stored form is an array of objects.** `history.json` and
  `epher-history` hold `[{"expr": "...", "out": "..."}, ...]`, with
  `out` omitted when absent. **The read path accepts both forms** (an
  untagged enum): a string is a legacy entry and is parsed once, at the
  load door, by the stripper's markers into a record. The heuristic
  moves from six recall sites, run on every recall, to one load site,
  run once per legacy entry; after the next save the file holds records
  and the legacy form never returns. `Session::with_history`, which
  both `load_session` and the PWA restore already go through, is that
  door.
- **No dual write.** The old key is not kept alongside a new one.
- **Recall presentation is unchanged.** A multi-line `expr` goes into a
  one-row editor the way the surfaces do it today (the TUI replaces
  newlines with `; `), a choice of the editor, not of the store.

## Consequences

- **Recall cannot compound, by type.** There is no suffix to forget to
  strip. The share link carries the expression exactly. The REPL bug of
  the ADR-0086 amendment becomes unrepresentable rather than fixed.
- **The list can grow independently of recall.** Showing or hiding the
  answer, or keeping a table's or a multi-line script's answer beside
  its line, becomes a rendering choice with no format change. Those are
  not decided here; the record merely stops forbidding them.
- **The format change is one-way.** After the first save under this
  ADR, a pre-0097 binary cannot read the list: it prints the standing
  warning, starts a fresh session (so saved functions and bindings are
  not loaded for that run either, since `load_session` reads history
  first), and its next history save replaces the file. Functions and
  bindings on disk are not touched by that save, each being its own
  key. This is the path every unreadable store takes today, the warning
  tells the user, and the release train (ADR-0073) ships forward. The
  ADR accepts the cost rather than the split brain of a dual write.
- **The legacy parse keeps the heuristic's edges, once.** A legacy
  answer that itself contains a marker is cut the way it would be cut
  today, and a legacy multi-line entry stays answerless. The damage is
  bounded to entries written before this ADR and is not repeated.
- **Test churn is the bulk of the work.** History assertions across
  core, store, cli, and tui compare strings (`assert_eq!(app.history(),
  &[...])`); they move to `display()` or a small comparison helper. The
  behavior they pin does not change.
- **`clear history`, Ctrl+L, and the one-slot restore (ADR-0094,
  ADR-0096) are unaffected.** They act on the list, not the format.

## Alternatives considered

- **Keep the string, keep the one-door strip** (the state after
  ADR-0096). Correct today, and the next consumer starts the bug class
  over. Declined.
- **Write both forms**, the legacy list under `history` and records
  under a second key. Doubles every write, and on a downgrade the old
  binary updates only the legacy list, so the lines a user runs there
  vanish at the next upgrade. Declined.
- **Store the kind.** It is the prefix of `out`; a field invites
  disagreement with the result-pane tier. Declined.
- **Rewrite the file on load.** A read that writes fails on a read-only
  store and migrates a file the user may never touch again. The next
  save migrates. Declined.

## Related

- ADR-0010 (the shared store and its snapshot), ADR-0027 and its
  amendment (multi-line scripts as one entry), ADR-0031 (expression-only
  picks), ADR-0038 (share links), ADR-0057 (the PWA's own copy),
  ADR-0079 (the tiers keyed on the answer's prefix), ADR-0086 and its
  2026-11-26 amendment (the REPL seed), ADR-0093 (the arrow walk),
  ADR-0094 (the guarded clear), ADR-0096 (the cap and the collapse).
- Code: `Session::record_with_output`, `Session::history_expressions`,
  `HistoryEntry` (`crates/core/src/lib.rs`); `history`,
  `save_history`, `load_session` (`crates/store/src/persist.rs`);
  `App::history_text` (`crates/tui/src/lib.rs`); the PWA restore and
  save in `crates/web/src/lib.rs`.

## Implementation notes (2026-11-26)

- Implemented the same day. `HistoryEntry` lives in `epher_core` beside
  `Session`. The stored form and the legacy door are the type's own
  `Deserialize` (an untagged repr: an object, or a string handed to
  `from_legacy`) and its `From<String>`/`From<&str>`, so every load path
  (`persist::history`, the desktop `init`, the PWA's `epher-history`
  restore, and any `with_history`/`restore_history` seed) parses a
  legacy entry once, right before `with_history` sees it.
  `with_history` and `restore_history` take `impl Into<HistoryEntry>`.
- `history_expression` is gone. Its body survives as the private
  `split_legacy`, called only by `from_legacy`, so "leaves the recall
  path" is enforced by the compiler rather than by review.
- The doors found during implementation were seven, not six: the
  desktop IPC carried the list twice more, `InitState.history` (the
  `init` answer) and the `save_history` command's argument, both
  `Vec<String>`. They are records now, in the same build as the webview
  that reads them (ADR-0087).
- An entry compares equal to the string that spells it (`PartialEq<str>`,
  `&str`, `String`, against `display()`), so a history assertion still
  reads as a transcript and the existing suites pin what they pinned;
  the ADR-0096 collapse rule uses the derived struct equality (`expr`
  and `out`). `kind()` is `HistoryKind::{Silent, Answer, Error, Warning}`.
- Tests: the stripper test became the load-door test
  (`a_legacy_display_string_parses_once_at_the_load_door`, with the
  multi-line edge and the `display()` round trip); the store gained two
  format tests (records on disk with `out` omitted; a legacy list loads
  and the next save writes records); the CLI's clear-history test pins
  the record on disk through the real binary. Outside the suites: the
  real REPL, driven through a pty against a hand-written legacy
  `history.json`, loaded without the warning, recalled `1/0` (not the
  transcript line) and the multi-line entry joined for its one row, and
  wrote records after one line; the 0.9.25 release binary on that
  migrated file printed `warning: could not load saved data
  (serialization error: invalid type: map, expected a string); starting
  fresh`, the cost this ADR accepts, observed.
- The one-shot CLI joined later the same day, after the release
  surfaced it: it had been the one surface still writing half-entries
  (`out: None`) because its stdout prints the bare piped form.
  `run_one_shot` now records through `record_with_output` like every
  other surface: the last value the run produced, spelled by the
  session's own `display()` prefs (`= 5`), so a multi-statement
  invocation carries the answer `ans` ends with, and a failure records
  `error: ...` and still persists, the REPL's contract. stdout is
  unchanged (ADR-0001's bare piped form); script-file runs stay silent
  (the `load` precedent, ADR-0040).
