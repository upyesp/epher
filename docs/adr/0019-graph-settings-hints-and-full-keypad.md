# ADR-0019: Graph pane settings, a full-function keypad, and the hints row

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

Three reports from using v0.4.6:

1. The Settings menu had nothing to say about the graph pane: the
   points-of-interest list under the plot and the highlighted points on
   the plot itself were always on.
2. On wide terminals the TUI's graph panel ran the full height of the
   body while the key hints lived in the bottom row of the 46-column
   calculator column — so the panel sat level with (and visually cut
   into) the hints, and the hint text itself was clipped at the column
   edge.
3. The TUI keypad showed 20 keys; the language has ~44 functions,
   constants, and commands. Most of them (`asin`, `hypot`, `variance`,
   `phi`, …) were unreachable from the keypad.

## Decision

### Settings → Graph

The Settings menu (and the mobile panel) gains a **Graph** group with
two `menuitemcheckbox` toggles:

- **Points of interest** — the list under the plot (web/desktop) or in
  the Graph panel (TUI).
- **Highlighted plot points** — the markers and labels drawn on the
  plot itself (web/desktop only; the ASCII plot marks nothing).

Both are display-only: the analysis always runs, so switching back is
instant and needs no re-evaluation. The web app persists them in
`localStorage` (`epher-poi-list`, `epher-poi-markers`) like the theme;
the TUI persists its single toggle in the native store (`pois`)
alongside `theme` and `language`. They are deliberately not shell
commands — they tune a pane, they do not compute.

### The hints row spans the terminal

The wide TUI layout now splits body → (content, hints) vertically
first, then content → (calculator, graph) horizontally. The graph
panel ends one row above the hints, and the hints run the full
terminal width instead of being clipped at the calculator column.

### The keypad grows banks

The TUI keypad mirrors the web keypad's coverage minus the digits
bank (a terminal already has number keys): **trig**, **fn**, **num**,
**var**. **Tab** opens the keypad and cycles its banks (Shift+Tab
cycles back), arrows move the highlight with column clamping across
ragged rows, **Enter** inserts, **Esc** closes. Tab no longer closes
the keypad — with four banks it has to switch them, and Esc was
already the closer. Cell width derives from the bank's column count
(5 columns → 8-wide cells, 4 → 11, enough for `variance`).

## Consequences

- `epher_tui::banks()` exposes the grid for tests; a regression test
  pins every function, constant, and command as reachable.
- The Settings dropdown now mixes one checkbox with the theme and
  language radios — each item carries its own role and checked state
  (`menuitemcheckbox` vs `menuitemradio`), and the ✓ marks stay as the
  non-color state marker.
- Browser suites assert both toggles end-to-end: apply, persistence
  across reload, independence from each other, and the mobile panel's
  Graph group; the TUI dropdown smoke-test shows `✓ Points of
  interest` first.
- The guide documents the toggles and the new Tab behaviour in all
  eight languages; the `tui-keypad` block title in each catalog now
  reads "Tab banks … Esc closes".
