# ADR-0016: Calculator-style fixed layout with keypad input

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

The PWA/desktop UI was a single scrolling column (input, result, graph,
history), and the TUI a read-eval-print screen. Users asked for the app to
feel like a scientific calculator: clickable buttons exposing every function
the epher language supports, alongside the existing typed input — so every
task has two ways in (buttons for discovery, typing for speed).

Space is the problem: a scientific calculator's key surface is large (~50
functions + digits + operators), and the graph must stay on screen without
scrolling.

## Decision

**One fixed viewport, no page scrolling.** The app fills the viewport
(`100dvh`, `overflow: hidden` on `body`). The calculator column is, top to
bottom:

1. **Input** (fixed height) — the existing textarea. First, so the user's
   entry point leads; the outcome follows (amended 2026-08-22: user
   feedback — the original answer-first order hid the input below the
   result panel).
2. **Answer panel** (fixed height, `aria-live`) — the result of the last
   evaluation, always visible.
3. **History** (flex-1, `overflow-y: auto`, `tabindex="0"`) — fixed size
   because its neighbors are fixed; scrolls internally.
4. **Keypad** (fixed height, ~5 rows) — tabbed buttons.

**The graph area is a fixed pane.** Desktop (≥880px, the site's breakpoint):
the graph pane sits to the right of the calculator column, both fixed at
viewport height. Mobile: the two panes sit side by side in a horizontal
scroll-snap container — swipe left for the graph, right for the calculator —
plus two accessible pane-switch buttons (visible on touch layouts). The
panes are always 100% of the viewport width, so swiping is a full-pane flip.

**All graphs scale to fit the pane.** The 2D and 3D SVGs already carry
viewBoxes; the pane gives each an `xMidYMid meet` fit (`width: 100%;
height: 100%`), so every plot letterboxes inside its fixed box instead of
growing it. Pointer tracing compensates for the letterbox: the fitted
content box is computed from the viewBox ratio before mapping to data
coordinates. 3D orbit is delta-based and needs no change.

**Keypad design.** Buttons insert text at the textarea cursor — the
evaluator, separators, and error reporting stay exactly as they are for
typed input; `=` submits the form, `C` clears the entry, `⌫` backspaces.
Five tabs, five columns:

- **123** — digits, `+ − × ÷ ^`, parens, `, ; .`, `ans`, `C`, `⌫`, `=`
- **trig** — sin cos tan asin acos atan sinh cosh tanh asinh acosh atanh
  deg rad atan2
- **ƒ** — ln log log2 logb exp sqrt cbrt root hypot abs floor ceil round
  trunc sign min max
- **nΣ** — gcd lcm mod fact ncr npr sum product mean median variance stdev
  frac dec big
- **π∇** — pi e tau phi x t ans graph graph3d table clear history

Functions insert `name(` with the cursor after the paren; constants and
variables insert their name; commands insert their keyword followed by a
space. Button labels are the language tokens themselves (the scripting
language is never localized, ADR-0007). After a press, focus returns to the
input so typing continues; buttons stay keyboard-reachable.

The tabs are a real APG tablist (`role="tab"`/`tabpanel`,
`aria-selected`); the keypad group is labelled. Every button is ≥44px
(WCAG 2.5.8); function rows sit 4–5 per row at ≥44px.

**TUI.** The terminal can't be clicked, but it gets the same two-way
surface: Tab toggles keypad focus, arrow keys move the highlight, Enter
appends the token to the input, Esc (or Tab again) returns to typing. The
keypad panel renders when focused (the graph panel shrinks by its height);
when closed the layout is unchanged. Keys are the same set, condensed to
4×5; graph commands insert their keyword.

**Window sizing.** The Tauri window grows to 1024×720 (min 900×640) so the
side-by-side graph is the default on desktop.

## Amendment notes

Recorded after the fact, per the ADR process (numbers immutable, earlier
records annotated, never rewritten):

- **The keypad evolved.** The bank list above is the original five
  (123, trig, ƒ, nΣ, π∇); ADR-0022 added `bin`/`oct`/`hex` keys and
  ADR-0024 moved them, with `frac`/`dec`/`big`/`!`, into a dedicated
  `0x` bank. The TUI keypad grew the same banks (ADR-0019, ADR-0024)
  and became an always-visible panel (ADR-0033).
- **The focus rule is desktop-only.** "After a press, focus returns to
  the input" still holds on desktop; on mobile the press must never
  summon the device keyboard — ADR-0035 (the mobile PWA usability
  contract) carries the touch rule.

## Consequences

- The app no longer scrolls vertically on any surface; content that can
  grow (history, graph chrome) scrolls inside fixed regions, which are
  keyboard-focusable (WCAG 2.1.1).
- The web app keeps its existing dark palette and contrast numbers
  (ADR-0016 changes layout, not tokens).
- Nothing changes for the CLI or the REPL; the scripting language is
  untouched — the keypad is a second spelling of the same input.
- The TUI gains ~40 lines of keypad state/rendering; its input remains a
  plain string (insertion appends at the end, where the terminal cursor
  already lives).
