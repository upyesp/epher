# ADR-0025: Apple Silicon only, dark launch, 3D pane controls, themed NSIS, uninstall cleanup, consistent TUI layout, split Open

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

Seven reports arrived together:

1. **The Intel Mac build is dead weight** — macOS is Apple Silicon only
   now; the x86_64 dmg and its download link should go.
2. **Non-Windows launches flash white** — Linux and macOS windows paint
   light for a beat before the dark page arrives.
3. **The graph pane toolbar only serves 2D** — a 3D surface gets no Copy
   SVG, no line-width slider.
4. **The Windows installer looks stock** — light MUI pages against the
   app's dark identity.
5. **Uninstalling keeps the history** — reinstalling on Windows showed
   the old history (the NSIS "delete app data" checkbox is off by
   default, and it never covered `~/.epher` anyway, where the store
   lives).
6. **The TUI's history vanished on the user's Linux terminal** — the
   desktop layout engaged only at ≥104 columns, and the stacked
   layout's fixed 20-row graph squeezed the history section to zero
   rows in a standard 80×24 terminal.
7. **File → Open dumped everything into the expression input** —
   opening a saved history file put its lines in the entry box instead
   of the history section.

## Decision

- **macOS is Apple Silicon only.** The `macos-x86_64` release job is
  removed, the landing page drops the Intel button (and the
  `mac-download-intel` string in all eight locales), and the release
  body text follows. The Apple Silicon job gains a CLI smoke step: the
  freshly built unified binary must report the tag version, evaluate
  `hex(255)` → `0xff` and `0b1010 + 0o17` → `25`, and write both a 2D
  and a 3D SVG (stroke-width and mesh markers asserted) — the same
  step runs on the Linux job.

- **Dark first frame on Linux and macOS.** The window gets
  `backgroundColor: "#141416"` in `tauri.linux.conf.json` and
  `tauri.macos.conf.json` platform overlays only. The base config
  keeps the window without it: on Windows the property changes window
  compositing and the WebView2 layer never paints on some setups (the
  v0.4.3 blank window), which is why it was dropped before. The
  overlays repeat the full window array (platform config arrays
  replace, not merge).

- **The graph pane toolbar serves 3D too.** `surface_parts` and
  `graph3d_svg` take a stroke width (mesh ×1.2, frame ×1.4, the same
  ratio as the fixed 1.2/1.4 defaults, so width 1.0 is byte-identical
  to the old output); the live 3D renderer threads the same value.
  The toolbar now always shows Clear graph, Copy SVG, and the
  line-width slider while the pane is non-empty; the POI toggles stay
  2D-only (surfaces have no points of interest) — controls shown are
  controls that do something. Copy SVG exports the 2D document when
  curves exist, otherwise the 3D document at the current orbit pose.

- **A dark NSIS installer and uninstaller.** The tauri NSIS template is
  vendored at `nsis/installer.nsi` (config `bundle.windows.nsis.template`)
  and themed: `MUI_BGCOLOR 141416`, `MUI_TEXTCOLOR F5F6F7`,
  `MUI_INSTFILESPAGE_COLORS`, plus an `epherPaint` SHOW function that
  walks the outer and inner dialogs and repaints every control dark
  (the standard MUI2 dark-theme technique — same stock widgets, same
  keyboard/RTL/accessibility behavior, just dark), instantiated for
  both installer and uninstaller contexts. The reinstall page and the
  uninstaller's confirm page call it inline. Dark header and sidebar
  bitmaps (generated from the brand palette and icon) replace the
  stock banner. `nsis-theme-check.nsi` compiles the additions against
  the system makensis in CI; the full rendered template compiles in
  the Windows bundling job with tauri's makensis 3.11 toolset.

- **Uninstall clears the app data.** The NSIS "delete app data"
  checkbox starts checked and the checked path also removes
  `$PROFILE\.epher`, where the store actually lives. On Linux, deb and
  rpm gain a `postRemoveScript` that removes every human user's
  `~/.epher` on remove/purge (skipping upgrades). Windows PATH cleanup
  behavior is unchanged. The AppImage and macOS have no uninstall
  step (delete the file / drag to Trash), so no hook exists there.

- **One TUI layout.** The wide (desktop-style) layout engages at 72
  columns instead of 104, so a standard 80×24 terminal shows history
  below the answer and the graph pane on the right, exactly like a
  bigger terminal — the layout was always terminal-size-driven, and
  the platforms run the same binary. The stacked fallback (below 72
  columns) shrinks its graph to 14 rows (12 with the keypad open) so
  history keeps room at 24 rows instead of collapsing to nothing.

- **File → Open splits into Open history and Open script.** Both
  frontends: the TUI's File menu gains a fifth item and two prompt
  kinds; the web app gets two hidden pickers. Opening a history file
  clears the current history and records each non-empty line without
  executing anything, then reports the loaded count (and, on desktop,
  persists it through the store like every submit); opening a script
  replaces the entry text. Save history / Save script stay as they
  are.

## Alternatives considered

- **backgroundColor in the base config.** Rejected — that is exactly
  what broke Windows in v0.4.3. The overlays keep the Windows path
  byte-identical to the fixed behavior.
- **Full custom-drawn NSIS pages.** Rejected; repainting the stock MUI2
  controls keeps the accessible, RTL-aware, battle-tested widgets and
  costs a fraction of the maintenance.
- **One Open item that guesses the file kind.** Rejected; content-based
  guessing is fragile, and the split matches the save pair
  symmetrically.

## Amendment note

The dark NSIS theme this ADR decided on did not survive contact with
the Windows installer (ADR-0026: the `SetCtlColors` walk subclasses
controls and locked the directory page), and the partial remedies in
ADR-0026/0027 could not darken MUI2's own pages or native controls. The
whole wizard went light in ADR-0028 — the current installer theme is a
uniform classic-light wizard with the epher logo. Everything else in
this ADR (Apple Silicon only, dark launch, 3D toolbar, uninstall
cleanup, TUI layout, split Open) stands as written.

## Consequences

- The vendored NSIS template must be kept in sync with the tauri
  bundler version (noted in its header); its theme additions are
  compile-checked independently in CI so drift fails loudly.
- Uninstalling deliberately deletes user data (opt-out via the
  checkbox on Windows; no prompt on Linux) — this is the requested
  behavior: reinstall means clean slate.
- The TUI's narrow fallback now shows a shorter graph below 72
  columns; the 3D SVG exports changed only when the width differs
  from 1.0 (determinism preserved).
