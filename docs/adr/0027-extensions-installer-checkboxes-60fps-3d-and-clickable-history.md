# ADR-0027: Extensions, readabler installer checkboxes, 60fps 3D, and clickable history (v0.4.15)

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

Eight reports after v0.4.14:

1. Save dialogs defaulted to `.epher` for both history and scripts; the
   user wants distinct extensions (`.ehs` / `.esr`) that can be changed,
   identically across TUI, desktop, and PWA.
2. The Windows installer's finish-page checkboxes ("Run epher", "Create
   desktop shortcut") drew black text on the dark page.
3. The 3D plot still flickered on Windows desktop and arrows felt
   limited; the goal is smooth 60fps orbit everywhere.
4. The line-width slider was missing above the 3D plot on Windows.
5. `graph`/`graph3d` commands echoed the command into the answer area.
6. The line-width slider should span 0–4.0 in 0.1 steps.
7. History lines should be clickable: picking one loads its text into
   the expression entry, replacing the current content.
8. On Linux Mint, File → Save history locked the desktop app.

## Analysis

- **Finish-page checkbox color.** MUI2 already applies `SetCtlColors`
  to its finish-page checkboxes, then strips their visual theme
  (`UXTHEME::SetWindowTheme`); a classic checkbox draws its label with
  `GetSysColor(COLOR_BTNTEXT)` — black — and `SetCtlColors` cannot
  recolor checkbox text (documented NSIS bug #443).
- **Windows 3D.** The orbit fix in ADR-0026 removed the per-event
  re-render, but every animation-frame commit still re-injected the
  whole mesh via `set_inner_html`: ~3k polyline nodes re-parsed,
  re-created, and garbage-collected per frame. WebView2 stalled on that
  churn; WebKitGTK and Chrome on the same hardware tolerated it.
- **Missing slider.** The width slider lives in the pane toolbar shared
  by 2D and 3D since v0.4.13 — present in the code on every platform;
  the Windows report matched a pre-0.4.13 build (the title bar carries
  the version for exactly this).
- **Answer echo.** The web and TUI paths set `graph: <source>` /
  `graph3d: <source>` into the answer area on success; history already
  recorded the command (correct).
- **Linux Mint lock.** `save_file_dialog` was a *synchronous* Tauri
  command: sync commands run on the main thread, and
  `blocking_save_file()` parks it for as long as the modal OS dialog is
  open (or forever, if the dialog appears behind the window — the Mint
  report: frozen app, no dialog). The v0.4.13 casing error had masked
  this by failing before the dialog opened.

## Decision

- **Extensions:** default names are `epher-history.ehs` and
  `epher-script.esr` in every frontend (TUI prompt prefill, desktop
  dialog prefill, PWA download name). The desktop save dialog sets no
  extension filter: rfd's Windows filter appends the filtered extension
  to typed names, which would fight "the user may change it" — with no
  filter, the typed name wins everywhere and the prefill is only a
  suggestion.
- **Finish checkboxes:** `SetSysColors(COLOR_BTNTEXT, 0xF5F6F7)` in the
  installer's and uninstaller's `.onInit` — one system call, no window
  procs touched, so the ADR-0026 failure mode cannot recur. Themed
  controls (navigation buttons, the uninstaller's own checkbox) ignore
  it and stay readable.
- **60fps 3D:** orbit frames keep the previous frame's element
  structure (one `<polyline>` per mesh line, one `<line>` per frame
  segment — a deterministic shape from our own generator), so a frame
  whose shape matches is applied by writing the mutable attributes
  (points, depth opacity, width) onto the existing elements instead of
  re-parsing the markup. Structure changes (different surfaces, slider
  changes of shape) still rebuild. Node churn drops to zero; orbit is
  a few thousand attribute writes per frame.
- **Answer area:** successful `graph`/`graph3d` lines leave the answer
  area empty in the web app and TUI (the plot is the result; the
  command is in the history). The CLI keeps its one-line confirmation
  on stdout — terminal output is its feedback channel (ADR-0013).
- **Slider:** `min="0" max="4" step="0.1"`. Zero is literal: an SVG
  stroke-width of 0 draws nothing, so at 0 the curves vanish and only
  the axes remain.
- **Clickable history:** web history lines are full-width buttons that
  load their text into the entry and focus it; the TUI gains a history
  focus mode in the Tab cycle (input → keypad → history): arrows move
  the selection (highlighted in the theme's selection colors, scrolled
  into view), Enter loads the line into the input without running it.
  The picked text is the displayed line verbatim — evaluation lines
  carry their `  =  answer` suffix and may be edited before re-running;
  graph lines are raw commands and re-run as-is. (Extended by the 2026
  amendment below: multi-line scripts are one item, picked whole.)
- **Linux save:** the command is async and runs the dialog inside
  `spawn_blocking` — the webview stays live while the dialog is open,
  wherever the dialog ends up on screen.

## Consequences

- `nsis-theme-check.nsi` compiles the finish page (Run +
  Create-desktop-shortcut checkboxes) and the `SetSysColors` call so
  CI keeps the template and the harness in agreement.
- The orbit suites pin the patched renderer: element counts stay
  constant across drag frames while projections keep changing.
- History entries remain `line  =  answer` for evaluations (unchanged
  file format, ADR-0025's open-history flow untouched); picking is a
  copy, never an execution.

## Amendment (2026-08-27): multi-line scripts are one history item

**Context.** The entry fields accepted multi-line scripts (Shift+Enter
composition, pasted scripts), but a submitted multi-line script was
recorded one line at a time — history could never give the script back
as a script. The user asked for the whole script to be selectable as a
single item, with a visible boundary between items in every frontend.

**Decision.**

- **One entry per submission, verbatim.** A submitted multi-line script
  becomes ONE history entry containing the script's lines joined with
  newlines — no answer suffix, nothing trimmed except blank lines and
  outer whitespace. Its statements still dispatch in order (each line
  and each `;` piece, graphs included); only the recording changes.
  Single-line submissions keep the original behavior: one entry per
  line, `;` separators intact, last answer appended.
- **Picking returns the whole script.** The web/desktop pick loads the
  entry verbatim into the entry field (a multi-line textarea, cursor at
  the end). The TUI input is one row, so its pick joins the script's
  lines with `; ` — the same separator (ADR-0001), so the script re-runs
  exactly as recorded. The CLI REPL's readline input is single-line and
  unchanged; multi-line scripts reach the CLI as piped stdin (`epher -`),
  which never touches interactive history.
- **Visible boundaries.** The web/desktop history list gives every entry
  a bottom border — one item sits between two rules, whether it is one
  row or many. The TUI draws a full-width `─` rule between entries;
  multi-line entries occupy one row per line, and the focus highlight
  covers all of an entry's rows. The TUI's selection and scroll now work
  in rows (a rule counts as a row that never picks), and the mouse maps
  a clicked row to its owning entry.
- **File format.** `.ehs` files keep one entry per line: a multi-line
  entry saves with its newlines escaped as `\n`, and opening a history
  file restores them. Old files load unchanged (no escapes present).
  The app store's JSON needs no change — entries are plain strings
  there. `history_expression` never strips a suffix inside a multi-line
  entry, so a line like `x = 2 + 2` survives a pick intact.

**Consequences.** The web's submit path distinguishes single-line from
multi-line submissions before dispatching, and suppresses per-line
recording for multi-line scripts. `save script` persists the whole
multi-line script. The browser suites pin: a 3-line submit yields one
history item, the item shows its lines between boundaries, and a pick
restores the full script; the TUI tests pin the `; `-joined pick and
the `.ehs` save/open round trip.
