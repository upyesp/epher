# ADR-0035: Keypad presses never summon the device keyboard on touch layouts

Date: 2026-08-25

Status: Accepted

## Context

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
- `docs/accessibility.md` records the behavior; no guide change — the
  guide never described the web keypad's focus handling, only its
  contents.
