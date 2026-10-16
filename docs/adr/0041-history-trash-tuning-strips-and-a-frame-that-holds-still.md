# ADR-0041: the history trash, tuning strips above the plot, and a frame that holds still

Date: 2026-08-31

## Status

Accepted (amends ADR-0027's history section, ADR-0020's line-width
slider, ADR-0031's fine controls, ADR-0038's live listeners, and
ADR-0039's captioned keypad height)

## Context

Another six-item review round:

1. The history list's Clear command was text; every other destructive
   act (Clear graph) is already an icon. The TUI had no visible
   spelling at all - Ctrl+L is documented nowhere on screen.
2. The digits keypad grew a scrollbar: the grid's bottom padding sat
   inside its own fixed height, and with the captions toggle on the
   keys grew again with nothing rescuing the window.
3. Wheel zoom on a plot died after toggling a legend checkbox - and
   stayed dead.
4. The adjustment sliders (line width; horizontal rotation, vertical
   rotation, zoom) were text-labelled, scattered - line width beside
   Copy SVG, the view controls below the plot - and readouts spent
   space.
5. 3D and solar plots visibly changed size while they moved: every
   frame refit the window to the current projection's bounds, so
   rotating or animating breathed in and out.

## Decision

**The history trash.** The Clear history command becomes the same
trash icon Clear graph uses, placed in the history head right of the
**History** heading; the tooltip and aria-label keep the words. The
TUI spells it in the panel's border title: `History 🗑`, and the glyph
is clickable - a click there clears, exactly like Ctrl+L, which stays.

**The keypad fits its window.** The grid loses the bottom padding that
pushed the digits bank past its own height, so the 123 tab shows five
rows with no scrollbar. The captioned state gets its own fixed window:
keys with hints are exactly 64px tall (captions clamp to two lines),
and the grid's height follows, still identical for every bank - one
window, two sizes, both exact.

**Listeners follow the node.** The plots bind wheel, trace, and pinch
listeners to the SVG element once, but Yew may replace that element on
a re-render - the legend's checkbox cycle does. The bind effect now
runs after every render and rebinds when the element it is holding no
longer is the element in the tree, so zoom, trace, and pinch survive
any legend change.

**One tuning strip above the plot.** Line thickness joins the view
controls in a compact strip directly above the plot in every kind:
2D carries thickness + zoom; 3D and solar carry thickness + horizontal
rotation + vertical rotation + zoom. Each slider is named by an icon
(strokes of growing weight; an arc arrow, turned a quarter-turn for
the other axis; a magnifier) with the words in its tooltip - line
thickness, horizontal rotation speed, vertical rotation speed, zoom
speed. The numeric readouts are gone; sliders align because they share
one width; the strip wraps on phones.

**The frame holds still.** The 3D projection is orthographic, so a
sphere around the world origin projects to a same-radius disc at every
pose. The 3D and solar windows are now fitted once to the scene's
bounding sphere around the origin (with the same 6% margin), not to
each frame's projected bounds: rotation, spin, and animation play
inside a window that never changes size. Degenerate scenes fall back
to the old per-frame fit.

## Consequences

- The history head reads: heading, then trash - the same shape in the
  app, the PWA, and the terminal; the terminal's is clickable.
- No keypad scrollbar in either state; the captioned window is 64px
  per row, so on small phones the pane scrolls as one unit instead.
- A legend cycle can no longer kill input on the plot - the failure
  the round's reporter hit is structurally impossible (the bind
  follows the node, whatever replaces it).
- Sliders above the plot cost one short row on 3D, already spent by
  the old below-plot block; 2D gains that row and loses the toolbar's
  width slider. Tooltips carry the localized words (four new Fluent
  messages, `tune-*`); the old Line width label is retired.
- The stable window is slightly larger than a per-frame max fit (a
  square around the scene's radius), the price of never refitting;
  frames no longer breathe, so plots read as steadier rather than
  smaller. The zoom slider and wheel keep their effect - zoom scales
  the fixed window as before.
- A core test pins the frame equality across two poses for both a
  surface and the solar scene; the browser suite pins the legend
  cycle, the strips, the trash, and the static frames.

## Amendment (2026-09-02): trash on the left, icons that reset (ADR-0055)

The history trash moves to the LEFT of the **History** heading (the
layout above said right). And each tuning-strip icon is now a real
button: pressing it resets that slider to its default (line width to
1.0 for 2D and 0.2 for 3D, zoom to the auto fit, rotation to 0).
