# ADR-0033: TUI Layout Fits 80×24 — Always-Visible Keypad, Wrapped Hints, and Sectioned Settings

Date: 2026-08-25

Status: Accepted

## Context

Four findings from using the v0.4.20 TUI on a standard terminal:

1. **The keypad is lost.** The TUI's keypad (ADR-0016) only rendered while
   it had focus — a Tab-gated panel that appeared and disappeared. Users
   who never pressed Tab saw no keypad at all, and even users who knew the
   binding perceived the panel as missing from the screen.
2. **The hint row is clipped.** The bottom key-guide row was a single
   fixed line. Its English text is 136 characters, the localized texts up
   to 159 — on an 80-column terminal roughly half of it was cut off, which
   also hid the shortcuts it advertises (contributing to "controls seem
   to be missing": the hinted keys are invisible).
3. **Other controls squeezed out.** The Settings popup listed its
   subsections flat — points of interest, three themes, eight languages,
   and the 3D fine controls all in one undifferentiated list, with no
   markers for where "Theme" ends and "Language" begins. And the keypad's
   border title ("Keypad — Tab banks, arrows move, Enter inserts, Esc
   closes", 60 characters) clipped inside its 46-column pane.
4. **No layout budget.** The layout was tuned for no particular terminal;
   the only sizing discipline was "shrink the graph so history keeps
   rows". The standard 80×24 terminal is the floor the layout must fit.

## Analysis

The TUI is a screen, not a document: it cannot scroll the whole layout, so
"fits 80×24" means every panel has a row budget at that size and every
string fits its row. The current budgets:

- Menu bar 1 row, hints 1 row → 22 rows for content.
- The calculator column (wide layout, 46 columns) needed input 3 + result
  1 + keypad 7 = 11 rows, leaving 11 for history — the keypad fits
  *permanently* at 80×24; nothing needed to be cut to make room.
- The hint text needs `ceil(len / width)` rows: 2 at 80 columns for every
  locale (the longest, French, is 159 characters), 3 below ~53 columns.
  Two wrapped rows show the complete key guide instead of half of it.
- The Settings popup has 12 rows without 3D surfaces, 15 with; three
  labeled section rules bring it to 18 rows + 2 borders = 20 — inside the
  23 available. No scrolling popup needed.
- The narrow stack (<72 columns) kept a fixed 14-row graph and dropped the
  keypad entirely when unfocused; at 60×24 it must fit input 3 + result 1
  + keypad 7 + hints 3 = 14 fixed rows, leaving 9 to split between
  history and graph (both `Min(0)` share it).

## Decision

**The keypad is always part of the screen.** Every layout width renders
it: in the wide layout the calculator column reads input → answer →
keypad → history (top to bottom, like the app), and in the narrow stack
it sits between the graph and the hints. Tab still moves focus onto it
(input → keypad → history → input), and the panel's border title changes
to "Keypad · Enter inserts · Esc closes" while focused so the mode is
still discoverable — it just no longer moves panels around.

**The hint strip wraps.** Its height is computed from the text
(`ceil(chars / width)`, clamped 1–3), so the entire key guide is visible
on 80×24 (2 rows) and on narrower terminals (3 rows). The old "one fixed
row" was the clip.

**The Settings popup marks its subsections.** Labeled rule rows — a bold
"─ Theme " caption plus a dim "────" fill to the popup's width — separate
points of interest, Theme, Language, and the 3D View controls. The rules
are display-only: the highlight and activation indices stay in item
space, so arrow movement and Enter behavior are unchanged (tests and
muscle memory both survive), and the popup's height/width math counts
them so nothing clips at the right border or the bottom.

**The keypad and graph size to their real panes.** The keypad's cell
width now derives from its actual pane width (44 usable columns → 8-wide
cells for 5-column banks, 11 for 4-column banks, clamped 6–11), so
narrower terminals shrink the grid instead of clipping it; the keypad
title is short (the instruction lives in the focused variant). The ASCII
plot's size comes from the graph panel's real dimensions on every layout
variant — the old hardcoded narrow sizes are gone.

**The budget at 80×24**: menu 1, content 21, hints 2. Calculator column:
input 3, answer 1, keypad 7, history 10. Graph panel: 34 columns × 21
rows. Everything present, nothing clipped. At 60×24 (narrow): input 3,
answer 1, history ~4, graph ~4, keypad 7, hints 3.

## Consequences

- The TUI at 80×24 shows all five panels (input, answer, keypad,
  history, graph) plus the complete two-row key guide; no Tab press is
  needed to see the keypad.
- The guide's TUI table (all eight locales) and the accessibility notes
  now describe focusing the always-visible keypad instead of opening it.
- Menu-navigation indices are untouched (the rules are display-only), so
  the existing TUI tests and pty smokes stay valid; new pty checks cover
  the always-visible keypad, the wrapped hints, and the section rules.
- Four new locale strings (keypad title, focused title, Theme, Language,
  View section labels) across eight locales; the old long keypad title is
  gone.
- The layout still degrades gracefully below 24 rows (history shrinks
  first, then nothing is dropped until the terminal is shorter than the
  fixed 13-row column).
