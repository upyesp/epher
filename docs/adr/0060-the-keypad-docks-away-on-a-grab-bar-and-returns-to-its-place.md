# ADR-0060: The keypad docks away on a grab bar, and returns to its place

Date: 2026-09-08

Status: Accepted

## Context

The keypad is always visible. On a phone the digits bank takes a third
of the screen (ADR-0016 docked it at the bottom so the entry, the
answer, and the history stack above it); in the desktop column and the
TUI's calc column it is a fixed block the history list can never grow
into. A user reading a long transcript or hunting an old history item
has no way to hand the keypad's space to the history — only the mobile
pane switch (calc ↔ graph, ADR-0016) hides it, and that replaces the
calc view entirely instead of expanding the history in place.

What the user asked for is the bottom-sheet gesture every phone
carries: a bar along the keypad's top with a grab area in the middle;
drag it down (or flick it) and the keypad slides out of view, the
history list growing to fill the space; drag the bar up — or tap it —
and the keypad slides back to exactly its previous place. The gesture
must work with a mouse (desktop PWA, desktop app) and a finger (mobile
PWA), the motion must be smooth, the bar must advertise itself, and
keyboard users need an equivalent the pointer does not gate.

The TUI already has all the plumbing this needs: mouse capture with
per-panel hit rects (ADR-0034), a keypad panel in the layout, and a
hints line for key discovery. What the terminal cannot do is animate:
a TUI repaints whole character cells on an event loop, with no
compositor to interpolate between frames.

## Decision

**Web (one implementation serving the mobile PWA, the desktop PWA, and
the desktop app).** The keypad section is wrapped in a drawer: a grab
bar, then a clipping wrapper around the keypad. The grab bar is a real
`<button>` — focusable, `aria-expanded`, `aria-controls` the keypad
panel, localized label — carrying a centered pill (44×4 px, accent on
hover/focus) as the visible affordance, on a 24 px strip (WCAG 2.5.8
target floor). `touch-action: none` on the bar so a finger drag drags
the drawer instead of scrolling or swiping the pane deck.

Pointer interactions (Pointer Events cover mouse, touch, and pen):

- **Drag**: pointerdown freezes the current height and measures the
  keypad's natural height; pointermove sets the wrapper's height in px
  (transition off), so the history list grows under the finger in real
  time; pointerup snaps.
- **Flick or half**: release collapses when the bar moved down past
  half the keypad's height or the last 80 ms velocity exceeds 0.5 px/ms
  downward; otherwise it springs back open. (`keypad_snap` is a pure
  function — unit-tested.)
- **Tap/click** (no drag): toggles, exactly like the keyboard path.
- **Keyboard**: Enter/Space on the button toggles; the same
  freeze-measure-animate path runs so keyboard users get the same
  motion.

Snapping animates the wrapper's height over 280 ms with an
ease-out-composite cubic-bezier, then the inline height is cleared and
the resting state is pure CSS (`[data-open="false"]` → height 0), so a
tab change or hints toggle at rest reflows naturally.
`prefers-reduced-motion: reduce` removes the transition (the pane
jumps, per that media query's contract).

The docked-away state is session state, not a stored setting: every
start shows the keypad, and "previous position" means the keypad's own
place — same tab, same scroll — which nothing else moves.

**TUI.** The keypad panel gains a docked state (`Ctrl+K` toggles; the
hints line carries the key). Shown, the panel draws as before with the
grab area drawn into its top border — three middle dots replacing the
border at the panel's center, bold so they read as a handle. Hidden,
one strip row remains where the keypad's top border was, the same
three dots centered. Mouse (ADR-0034): press on the strip (shown:
border row; hidden: the strip row), drag down two rows to dock away,
up two rows to restore, or release without moving to toggle — the
same grammar as the web bar. The transition is instant: the terminal
has no compositor, and redrawing partial rows to fake motion would
flicker on the event loop. Everything else about the gesture — the
bar, the grab dots, drag directions, keyboard parity — matches the
GUIs.

## Consequences

- The history list absorbs the keypad's space in all four frontends;
  `.history-box` (web) and the `Min(0)` history constraint (TUI)
  needed no change — they were already the flexible sibling.
- The web drawer animates height, not transform: the point is to give
  the space to the history live under the finger, which a translate
  (sliding over the history) cannot do. Height animation on a
  fixed-height keypad grid is cheap; the pane's flex layout reflows.
- The docked state is not persisted: a missing keypad on startup would
  read as a bug, and the gesture to bring it back is the same one that
  put it away. If frequent dockers ask, an amendment can store it.
- New localized strings: the grab button's show/hide labels in the
  eight guide locales, and the TUI hints line grew the Ctrl+K entry.
