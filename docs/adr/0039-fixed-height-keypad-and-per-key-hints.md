# ADR-0039: a fixed-height keypad with scrolling, and a meaning for every key

Date: 2026-08-30

## Status

Accepted (amends ADR-0016's keypad layout and ADR-0033's TUI keypad
sections)

## Context

Two user reports about the keypads (ADR-0016):

1. The astronomy keypad is very long and fills much of the screen. The
   `astro` tab carries 56 keys - eight grid rows at five columns - and
   the grid had only a `min-height`, so selecting the tab grew the
   keypad over the history and entry above it. The other six tabs kept
   the five-row height the digits tab reserves (ADR-0016), which made
   the whole pane jump.
2. The `123` tab's buttons are intuitive, but everything else is not.
   `hms2deg`, `dawes`, `h_bar`, or even `logb`'s argument order ask a
   question the labels cannot answer. The hint had to reach every kind
   of user on every frontend:
   - mouse users expect a hover tooltip;
   - touch users (the PWA on a phone) have no hover at all;
   - keyboard users need the answer at focus, not only at the pointer;
   - screen reader users need the meaning announced with the button;
   - TUI users need the same knowledge inside a 80x24 terminal.

## Decision

**The keypad is always exactly the digits tab's height.** The grid gets
a `max-height` alongside the existing `min-height` (five 44 px rows
plus gaps) and scrolls vertically. Every tab is now the same height:
the astronomy bank scrolls instead of growing, and no tab resize jumps
the pane.

**Every non-obvious key speaks its meaning through one localized
message per key** (`key-hint-*` in the FTL catalogs - 130 messages in
all eight locales, covering every function, command, constant, unit
suffix, and operator glyph; only the digit keys and `.` are bare,
because their labels say what they do). One message serves every
affordance:

- **Screen readers**: each button's `aria-label` becomes
  `"{token}: {hint}"`, so focus announces the token and its meaning
  together ("jd: Julian Day of a moment").
- **Mouse and keyboard (web)**: a docked hint bar between the tabs and
  the grid shows the hovered or focused key's hint. A docked bar cannot
  clip the way a floating tooltip would inside the newly scrolling
  grid, and it answers for focus as well as hover. The bar is
  `aria-hidden` - the `aria-label` already carries the text.
- **Touch (web)**: a `?` toggle beside the tab list (a sibling outside
  the `role="tablist"`, which must not contain non-tab buttons) captions
  every key with its hint. Discovered and reliable where hover does not
  exist; `aria-pressed` tracks the state, and the toggle's state is
  session-local like the other keypad view state.
- **TUI**: `?` opens a key-help overlay listing the current bank's keys
  with their hints (the same FTL messages), scrollable with the arrows
  and closed with `q` or Esc - modal like the guide pager. The Help
  menu gains "Key help" as its second item, and the overlay opens only
  when nothing else owns the key: the entry is empty (the same rule as
  `q`) or the prompt and menu are inactive.

The desktop app shares the web UI, so every desktop OS gets the bar,
the toggle, the scroll fix, and the ARIA changes from the same code;
the TUI reads the same FTL catalogs, so a hint is translated once for
every frontend.

**Bank changes start at the top** (web): the grid panel is re-keyed per
tab, so switching banks no longer inherits the previous bank's scroll
offset.

## Consequences

- The astronomy keys cost a scroll instead of screen area; the five-row
  window is the digits tab's, so nothing else moves.
- A new language function needs a `key-hint-*` message alongside its
  keypad entry - the i18n parity test fails a locale that drops one,
  and the web and TUI tests fail a keypad key without one.
- The hint texts are short by construction (one line in the bar, three
  caption lines in a button); long explanations belong in the guide,
  which the hints complement, not replace.
- Unit suffixes that repeat a constant's token (`pc`, `ly`, `AU`) get
  distinct hint messages (`key-hint-pc` vs `key-hint-u-pc`), so the two
  buttons no longer sound identical to a screen reader.

## Amendment (2026-08-31): the hints learn to suggest and to answer F1 (ADR-0042)

The same `key-hint-*` messages now surface in two new places. The web
entry's suggestion list carries each matching function's or constant's
hint beside its name, and F1 (word under the cursor) prints the hint in
the bar above the keypad on the web and in the TUI's answer line. Names
without a hint suggest bare and F1 says so in plain words; the
every-locale parity test still guards every hint that exists.
