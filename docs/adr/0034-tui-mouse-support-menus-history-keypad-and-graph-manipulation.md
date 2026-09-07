# ADR-0034: TUI Mouse Support — Menus, History, Keypad, and Graph Manipulation

Date: 2026-08-25

Status: Accepted

## Context

The TUI was keyboard-only while the web and desktop apps are fully
mouse-driven. Users asked for the pointer everywhere it has a natural
spelling in a terminal:

1. **Menus** — click the menu bar to open a menu, click a popup item to
   activate it.
2. **History** — click a line to load it, like the web's clickable
   history (ADR-0027).
3. **Keypad** — click the bank tabs and the cells; the web's keypad is a
   grid of buttons, the TUI's should answer to the same clicks.
4. **Graphs** — manipulate the plot: orbit the 3D surface, pan and zoom
   the 2D plot, and a way back to the defaults.

The terminal already speaks the mouse: every mainstream emulator reports
clicks, drags, and wheel scrolls, and crossterm parses both encodings
(X10 and SGR) from the same event stream the keys arrive on.

## Analysis

- **Hit-testing needs real coordinates.** The draw pass computed every
  panel's rect locally and threw them away. Mouse handling must resolve
  clicks against the *rendered* layout — including localized menu labels
  and the keypad's per-bank cell geometry, which vary with locale and
  pane width.
- **The 2D plot had no viewport.** The ASCII renderer always auto-fit the
  samples; there was nothing to pan or zoom. Worse, indexing the grid
  with sample positions assumed every sample lands inside — the first
  pan exposed an out-of-bounds panic in the renderer.
- **3D manipulation already had a model.** The web's drag orbit
  (`yaw += dx·0.01`, `pitch += dy·0.01`, pitch clamped ±1.4) and the
  camera distance are the same `View3D` the TUI renders with; the TUI
  needed the same inputs.
- **Menu activation code was inline in the key handler.** Mouse clicks
  on popup items need exactly the Enter arm's behavior, including the
  language re-resolve and store persistence — it had to be shared, not
  copied.
- **Diff rendering constrains testing.** The pty smoke reconstructs
  frames from ratatui's changed-cells stream; the harness must keep a
  persistent grid (a real terminal's model) instead of rebuilding from
  an empty screen each capture.

## Decision

**The draw pass records its layout.** `App` gains an `Areas` record the
draw function fills as it renders: the five menu-label rects (measured
from the localized labels actually painted), every panel rect, the five
keypad bank-label rects, the current bank's cell width and column count,
the history scroll offset, and the open popup's menu index + rect. Mouse
events resolve through this snapshot of the last frame.

**Menus.** Clicking a menu label opens that menu (or closes it, when it
is already open — and clicking outside a popup closes it without acting,
the browser convention). Clicking a popup item sets the highlight and
activates it through the same shared `perform_menu_action` the Enter key
uses; section rules are not clickable. The popup's row list is built by
one `menu_rows` function shared by draw and hit-testing, so a click
always lands on the same row the user sees.

**History.** A click on a displayed line picks it — the expression only,
the same `  `-separated answer suffix stripped as ADR-0031 — with the
panel's scroll offset accounted for. The arrows/Enter path is unchanged.

**Keypad.** Clicking a bank tab selects that bank; clicking a cell moves
the highlight there and inserts the token. A click never changes focus:
the pointer is a second spelling of the same input, and typing after a
click must keep working without an Escape.

**Graphs.** The graph panel takes drags and the wheel:
- 2D: a drag pans the viewport (the plot follows the pointer — the
  window moves through the data), the wheel zooms around the center
  (×0.8 in, ×1.25 out per step). A new `view2d` override in `App` holds
  the ranges; `None` means auto-fit, and plotting or clearing a graph
  drops the override. The renderer clips samples outside the window
  (this fixed the out-of-bounds panic the first pan exposed).
- 3D: a drag orbits with the web's exact sensitivity (0.01 rad/cell,
  pitch clamped ±1.4), the wheel scales the camera distance (×0.9/×1.1,
  floored at 0.5), and the fine-control sliders still compose on top.
- A double-click (two left presses within 500 ms, within one cell) on
  the graph panel resets it: 2D re-fits the samples, 3D returns to the
  default pose (offsets untouched — they belong to the sliders).
- The wheel also scrolls the guide pager.

**Capture is scoped to the TUI's lifetime.** Mouse capture is enabled
after `ratatui::init` and disabled before restore, so no shell the user
returns to is left in a mouse-reporting mode.

## Consequences

- The TUI's mouse surface mirrors the apps: menus, popup items, history
  picks, keypad bank tabs and cells, and 2D/3D graph manipulation all
  answer to the pointer; every keyboard path still works.
- Four new App methods carry the mouse state changes (`graph2d_pan`,
  `graph2d_zoom`, `graph2d_reset`, `view_reset_pose`,
  `view_set_camera`, `history_pick_row`, `keypad_select_bank`,
  `keypad_set`, `menu_select`) and are unit-tested; `View3D::with_camera`
  joined the core.
- The renderer's out-of-bounds clipping (2D pan/zoom) is regression-
  tested with extreme viewports.
- The pty smoke drives real mouse events through SGR sequences; its
  capture keeps a persistent grid because ratatui diff-renders.
- The guide (all eight locales) documents the mouse row in the TUI
  keybindings table; the accessibility notes record the pointer as the
  keyboard alternative's counterpart.
