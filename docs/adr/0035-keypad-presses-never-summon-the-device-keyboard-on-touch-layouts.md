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
  one expression; the textarea's stored selection is read before the blur
  and written back after the value update, so the cursor still advances
  press to press. The value/selection writes work on an unfocused
  element — focus is presentation here, not state.
- **Scope.** Every keypad action takes the same path — `Text`, `Call`
  (functions), `Backspace`, `Clear`, and `Submit` (`=`) — so one change
  in the shared handler covers all of them. History picks keep their
  focus: loading a line to edit it is a request for the keyboard
  (ADR-0031), and the guide's code buttons keep theirs for the same
  reason. The `mobile_layout()` gate is the same 880px breakpoint
  ADR-0029/0030/0031 use, so the behavior tracks the layout flip.

## Decision

`on_keypad` no longer refocuses unconditionally. After a press it checks
`mobile_layout()`: on touch layouts it blurs the entry (closing the
device keyboard, explicitly and always), and on desktop it keeps
ADR-0016's refocus exactly as before. The entry's stored cursor still
composes successive presses on both layouts.

## Consequences

- On mobile, the keypad is a closed system: presses compose, submit,
  clear, and backspace without the device keyboard ever appearing; the
  result panel stays visible because nothing slides up to cover it.
  Tapping the entry still opens the keyboard for typing.
- Desktop behavior is byte-identical to ADR-0016.
- The Playwright battery pins the contract in a headless browser (the
  soft keyboard itself cannot be emulated, so the test asserts the
  honest proxy): on a mobile viewport, after a keypad press the
  textarea does not own focus; on a desktop viewport it does, and typed
  keys land in the entry. Composition (two presses, one expression) is
  asserted on both.
- `docs/accessibility.md` records the behavior; no guide change — the
  guide never described the web keypad's focus handling, only its
  contents.
