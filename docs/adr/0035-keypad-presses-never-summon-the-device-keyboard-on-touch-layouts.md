# ADR-0035: Mobile PWA usability — the onscreen keypad is the primary input, and a drawn plot slides into view

Date: 2026-08-25 (amended 2026-08-26)

Status: Accepted

## Context

This ADR is the mobile PWA's usability contract: on a touch viewport
(under 880px, the same breakpoint ADR-0029/0030/0031 use) the onscreen
keypad is the primary way to enter expressions, the result of every
command must land where the user can see it without extra gestures, and
a touch on the graph must act on the graph, not the navigation around
it. The keypad's focus discipline below came first; later amendments
folded in the graph pane's auto-slide — first codified in ADR-0029 —
the 3D swipe rotation, the kind-aware width slider, the slide back
after a clear, and the per-kind width memory, so the contract reads as
one piece.

On the mobile PWA every on-screen keypad press ended by refocusing the
expression entry — ADR-0016's rule, "focus returns to the input so typing
continues". On a desktop that is pure convenience: the pointer hands the
keyboard the focus it needs for the next keystrokes. On a touch device it
is a bug: programmatic focus of a textarea opens the soft keyboard, so
each tap of the keypad (which occupies the bottom of the fixed viewport,
ADR-0016) summoned the device keyboard, which slid up and covered the
keypad the user was pressing.

The project had already accepted the mirror image of this fix once:
ADR-0030 blurs the entry after the mobile auto-slide to the graph pane
because a freshly drawn plot should not sit under a keyboard. The keypad
press was the same problem at higher frequency — every press, not just
plot draws.

## Analysis

- **The keypad is the keyboard's stand-in on touch.** The whole point of
  the on-screen keypad on mobile is to *replace* typing; summoning the
  device keyboard after each press defeats it. When the user wants to
  type, they tap the entry — that is the gesture that should open the
  keyboard.
- **Desktop behavior is load-bearing.** ADR-0016's focus-return matters
  with a physical keyboard: click a function, keep typing. Nothing about
  the desktop should change; the fix is conditional on the layout.
- **The tap itself already leaves the entry** in most mobile browsers
  (focus moves to the button, the keyboard closes). Blurring explicitly
  covers the rest (browsers/webviews that keep the entry focused through
  a button tap) and makes the close deterministic rather than incidental.
- **Composition must survive the blur.** Successive keypad presses build
  one expression, and the next insertion point must always be immediately
  after what was just inserted — including mid-string: with the caret
  between two characters, a press inserts there and leaves the caret right
  after the inserted token; a press over a selected range replaces the
  range and leaves the caret after the replacement. But the blur that
  closes the mobile keyboard is also the moment the DOM selection dies
  (Chromium zeroes it), and the button's mousedown default action steals
  focus *before* the click handler runs — so at press time the DOM
  selection is already gone, unreadable. The selection therefore lives in
  app state, not the DOM: a cell holding a `(start, end)` range. It is
  mirrored by a document `selectionchange` listener while the entry owns
  focus (every user caret move and selection drag fires it), refreshed at
  each keypad mousedown — the handler runs before the default
  focus-stealing action, the last moment the entry's true selection is
  readable — and updated by every keypad action, by history picks, and by
  guide code loads (both put the cursor at the end of what they load).
  Keypad presses read the cell alone; keyboard activation of a keypad
  button (Tab + Enter) skips mousedown and relies on the mirror. On
  desktop the entry is refocused after each press, so the DOM and the
  cell stay in step either way.
- **Scope.** Every keypad action takes the same path — `Text`, `Call`
  (functions), `Backspace`, `Clear`, and `Submit` (`=`) — so one change
  in the shared handler covers all of them. The `mobile_layout()` gate
  is the same 880px breakpoint ADR-0029/0030/0031 use, so the behavior
  tracks the layout flip.
- **The keypad is the primary input on touch; the device keyboard opens
  only for a touch inside the entry.** That rule extends to everything
  that loads text into the entry from outside it: picking a history
  line and clicking a guide code button load the text with the cursor
  at its end, and on mobile they do *not* refocus the entry — the
  floating keyboard stays closed, and the keypad composes from the
  loaded text. (The earlier "picks keep focus because loading a line to
  edit requests the keyboard" reading is reversed: the pick itself is
  the edit gesture, and the keypad is what edits.) Desktop keeps
  ADR-0016's focus return for both.
- **A drawn plot must be seen, not discovered (ADR-0029).** The mobile
  layout holds the calculator and graph panes in a horizontal strip —
  the graph is one slide away. A user who submits `graph x ^ 2` and
  stays on the calculator would otherwise have to go look for their
  result. When a submitted `graph` or `graph3d` command draws a plot,
  the submit path slides the strip across to the graph pane — the same
  discrete instant jump the pane-switch buttons use (reduced-motion by
  construction, WCAG 2.3.3). Only successful draws slide: errors,
  `graph clear`, and plain evaluations leave the view alone, and the
  desktop layout (panes side by side, no strip) never scrolls. The
  slide pairs with ADR-0030's blur: dropping the entry's focus closes
  the device keyboard so the fresh plot is not sitting under it, and
  the plot is ready for touch rotation.
- **A swipe on a 3D graph rotates it — horizontally included.** With
  the graph pane in view, a swipe gesture on a 3D surface must orbit
  the surface, whichever axis the finger moves along: the vertical
  swipe already rotated (the strip only scrolls horizontally, so the
  browser never claimed it), but a horizontal swipe was captured by the
  strip's own pan and slid the pane back to the calculator — the graph
  the user was manipulating vanished mid-gesture. The 3D SVG therefore
  declares `touch-action: none`, and the rule's selector needs the
  extra specificity (`.plot-box svg.graph3d-svg`) because the 2D rule
  `.plot-box svg` (0,1,1) outranks a bare class (0,1,0) — the original
  selector silently lost and the swipe kept panning. The 2D plot keeps
  `pan-x`: a 2D graph does not orbit, so a horizontal swipe there
  remains the swipe-back-to-the-calculator gesture, and the pane-switch
  buttons are the always-available spelling of that same move.
- **The width slider follows the graph kind on mobile — and remembers
  each kind's value independently.** On the small screen the thickness
  slider's range was the layout's alone (ADR-0031): 0–0.2 step 0.01
  for every graph, which is right for a 3D wireframe but starves a 2D
  curve of the desktop's 0.1–4 step 0.1 range. A 3D surface therefore
  keeps the thin range with its 0.1 default, while a 2D-only graph
  gets exactly the desktop slider — range, step, and the desktop
  default (1.0). A single shared width was wrong: one remembered value
  applied to every new graph, so a width picked for a 2D curve re-
  shaped (or got re-clamped onto) every later 3D surface and vice
  versa. The widths are therefore stored per kind (`epher-line-width-2d`
  / `epher-line-width-3d`, falling back to the legacy shared key and
  then the kind's default), each kind renders with its own value — a
  2D curve at 2.5 stays 2.5 while a 3D surface sits at 0.15 — and the
  slider shows and edits the kind in view (3D while any surface is
  plotted). Desktop keeps its one shared width, unchanged.
- **A cleared graph pane is nothing to look at — slide back.** The
  mirror of the draw slide (ADR-0029): on mobile, once the graph pane
  has been emptied the view slides back to the calculator. The Clear
  Graph button always empties the pane, so it always slides back; the
  `graph clear` and `graph3d clear` commands slide back exactly when
  they leave the pane empty — a `graph3d clear` with a 2D curve still
  plotted keeps the pane in view, because there is still a graph to
  look at. The submit path knows both facts at the right moment: it
  remembers whether the pane had content before the loop and whether
  anything remains after it.

## Decision

`on_keypad` no longer refocuses unconditionally. After a press it checks
`mobile_layout()`: on touch layouts it blurs the entry (closing the
device keyboard, explicitly and always), and on desktop it keeps
ADR-0016's refocus exactly as before.

Every keypad press inserts at the stored selection — a mid-string caret
lands the insertion there, a selected range is replaced — and moves the
insertion point to immediately after what was just inserted. Because the
mobile blur zeroes the DOM selection, the selection lives in a cell that
the `selectionchange` mirror and the keypad mousedown refresh keep
current, and each action rewrites it to its new position; a press on an
unfocused entry therefore still lands on the right spot, without ever
summoning the soft keyboard. `C` and `=` reset the cell to 0, history
picks and guide code loads set it to the end of the loaded text.

History picks and guide code loads follow the same touch rule: on
mobile they set the text and the cursor cell but do not refocus the
entry, so the device keyboard stays closed; on desktop they focus the
entry as before.

On mobile, a submitted command that draws a plot slides the graph pane
into view immediately: the `graph` and `graph3d` success arms emit the
graph-pane scroll from the submit path, only under `mobile_layout()`,
and drop the entry's focus at the same moment (ADR-0030) so the
keyboard closes over nothing the user needs. The slide is the pane
switch's own discrete jump; nothing else — errors, clears, plain
evaluations, or any desktop submit — moves the view.

With the graph pane in view, a swipe on a 3D surface orbits it on both
axes (`touch-action: none` on the 3D SVG, with the selector specificity
needed to beat the 2D `pan-x` rule); the 2D plot keeps `pan-x`, so
swiping back to the calculator works exactly there and the pane-switch
buttons cover the same move everywhere.

On mobile the width slider's range is the graph kind's: a 3D surface
keeps ADR-0031's 0–0.2 step 0.01 with the 0.1 default, a 2D-only graph
gets the desktop range (0.1–4 step 0.1) and the desktop default. Each
kind remembers its own width — stored under its own key and restored
when its kind is in view — and each kind's plot renders with its own
value; the slider shows and edits the kind in view. Desktop keeps one
shared width, unchanged.

On mobile, a clear that empties the graph pane slides the view back to
the calculator: the Clear Graph button always (it empties the pane by
definition), and the `graph clear` / `graph3d clear` commands exactly
when nothing remains plotted.

## Consequences

- On mobile, the keypad is a closed system: presses compose, submit,
  clear, and backspace without the device keyboard ever appearing, and
  history picks and guide code loads enter text the same way; the
  result panel stays visible because nothing slides up to cover it.
  Touching inside the entry is the only gesture that opens the
  keyboard for typing.
- Desktop behavior is byte-identical to ADR-0016.
- The Playwright battery pins the contract in a headless browser (the
  soft keyboard itself cannot be emulated, so the test asserts the
  honest proxy): on a mobile viewport, after a keypad press the
  textarea does not own focus; on a desktop viewport it does, and typed
  keys land in the entry. Insertion-point behavior is asserted
  behaviorally on both layouts: consecutive presses compose; a
  mid-string caret (positioned by keyboard, then blurred by the press
  itself) inserts at the caret and the next press lands immediately
  after the just-inserted token; a selected range is replaced and the
  next press continues right after the replacement; `C` and `=` reset
  the insertion point to 0. On a mobile viewport a history pick loads
  the expression without the textarea gaining focus and a following
  keypad press composes at the end of it; on a desktop viewport the
  pick focuses the entry.
- The `mobile-scroll` suite (v0.4.17, ADR-0029) pins the auto-slide:
  mobile `graph` and `graph3d` submits slide (the strip's scroll
  reaches the graph pane and the pane-switch state follows), `graph
  clear` and failed graphs do not slide, and desktop submits never
  scroll. Focus stays with the entry's rules above — on mobile the
  slide drops it, so the keyboard closes over the fresh plot.
- The `mobile-graph` suite pins the new rules with CDP-dispatched
  touch swipes and slider reads: a horizontal swipe across the 3D plot
  rotates the mesh without moving the strip, a vertical one rotates
  too, a horizontal swipe on a 2D plot still slides the strip back to
  the calculator; the Clear Graph button and the clear commands slide
  the view back exactly when the pane ends up empty (a `graph3d clear`
  with a 2D curve left does not); the width slider carries the 3D
  range (0–0.2/0.01, default 0.1) or the desktop range (0.1–4/0.1,
  default 1.0) with the graph kind, each kind renders its own
  remembered width (2.5 and 0.15 side by side), flips restore each
  kind's value, both survive a reload, and desktop attributes stay
  byte-identical. The suite types commands with focus({preventScroll})
  because the browser's own caret reveal would otherwise scroll the
  strip during the test.
- `docs/accessibility.md` records the behavior; no guide change — the
  guide never described the web keypad's focus handling, only its
  contents.

## Amendment (2026-08-27): the arrow-key 3D hint is not displayed on touch layouts

The 3D pane hints "Drag to rotate · arrow keys rotate · non-zero rotation
sliders spin" under every plot. On a touch layout the arrow keys do not
exist — rotation is the swipe this ADR already defines — so the hint
advertises an unavailable affordance. The hint is not displayed under the
mobile media query (<880 px): `.graph3d-hint { display: none }` hides it
for touch layouts (and screen readers — `display: none` removes it from
the accessibility tree, so mobile users never hear about arrow keys
either). Desktop, the TUI, and the web at ≥880 px keep the hint.
