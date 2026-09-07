# ADR-0026: Three v0.4.13 regressions — NSIS repaint removed, save-dialog arg casing, 3D orbit accumulation

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

v0.4.13 shipped three regressions:

1. **The Windows installer locked up on the destination page.** The page
   showed its header text but no directory controls, and the first
   click anywhere froze the installer hard enough to need Task
   Manager.
2. **Every desktop save dialog failed.** Save history and Save script
   both reported `error: invalid args 'defaultName' for command
   'save_file_dialog': command save_file_dialog missing required key
   defaultName`. The TUI worked.
3. **The 3D plot wouldn't orbit.** Dragging painted a coloured line
   down the left edge of the pane, the surface "shivered" instead of
   rotating, and arrow keys only nudged it by one fixed step.

## Analysis

- **Installer.** ADR-0025's `epherPaint` walked every window and
  control of every MUI page and called `SetCtlColors` on each.
  `SetCtlColors` is not cosmetic: it *subclasses* the target control's
  window procedure. On the directory page that double-subclasses
  controls MUI itself subclasses (the destination edit field among
  them) — the corrupted message path makes the controls vanish, and
  the first click into a corrupted proc spins the installer. The
  welcome page displayed fine because painting the simple statics
  there was harmless; the failure only surfaced one page in.
- **Save dialog.** The web bridge serialized its args as
  `{content, default_name}`; the Tauri command macro deserializes
  invoke args as camelCase (`defaultName`) by convention. Every
  desktop save errored; the TUI never noticed because it calls the
  command natively in Rust, and the browser test suite never noticed
  because its invoke stub never validated the serialized shape.
- **3D orbit.** Two compounding defects. (a) `on_orbit` read
  `*view` — a Yew state handle — per event. Handles deref to the last
  *rendered* snapshot (the same rule that bit history persistence in
  ADR-0024), so a burst of drag/keyboard events each computed from the
  same base and overwrote each other: the graph "shivered" and arrows
  never accumulated. (b) Every pointer event re-rendered and
  re-injected the entire mesh SVG mid-drag, re-parsing thousands of
  polyline nodes per event — the flicker. The coloured line was text
  selection: dragging from the left side of the plot selected the
  y-axis tick labels and painted the selection band down the pane's
  left edge.

## Decision

- **NSIS theme = official MUI2 mechanism only.** `epherPaint` is
  deleted (template and CI harness). The theme is the color defines
  (`MUI_BGCOLOR`, `MUI_TEXTCOLOR`, `MUI_INSTFILESPAGE_COLORS`), the
  dark header/sidebar bitmaps, and the already-shipping uninstall
  behavior. The directory/license pages keep stock system-colored
  controls on the page background: nothing is subclassed, nothing can
  lock. `nsis-theme-check.nsi` now compiles exactly these constructs
  and may not contain `SetCtlColors` or any window walk.
- **Invoke args are camelCase.** `SaveFileArgs` gets
  `#[serde(rename_all = "camelCase")]`, and the tauri-save suite now
  asserts the exact serialized shape (`defaultName` present,
  `default_name` absent) through the real wasm path.
- **Orbit events accumulate in a live cell and commit once per
  frame.** `on_orbit` mutates a `Rc<RefCell<View3D>>` cell (the
  ADR-0023 live-cell pattern) and mirrors it into the state handle for
  rendering — no refresh-from-state effect (that pattern re-introduced
  the stale snapshot). The 3D drag handler accumulates pointer deltas
  into a pending slot and emits at most once per `requestAnimationFrame`,
  with a final commit on pointerup/leave/cancel, so the surface
  re-renders ~60×/s with a fresh projection instead of once per
  pointer event. `.graph3d-svg` gets `user-select: none`: dragging
  orbits, it never selects.

## Consequences

- The installer's regular pages are dark in the header and on the
  welcome/finish/log pages with system-colored controls elsewhere —
  less uniformly dark than ADR-0025 aimed for, but it renders, its
  controls work, and it cannot hang. Functionality wins over
  uniformity for install/uninstall.
- The orbit test suite pins the new contract: arrow presses must land
  in different places, a drag must produce a fresh projection every
  frame, and nothing may be selected during a drag.
- The web bridge now must keep camelCase for every Tauri-bound struct
  (documented on `SaveFileArgs`).
