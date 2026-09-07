# ADR-0023: Native-feeling menus, pane toolbar, solid curves, and boot self-heal

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

Four interface reports arrived together:

1. **Menus did not behave like menus** — an open dropdown stayed open
   when the user clicked elsewhere on the page; Edit items did not close
   their menu on activation.
2. **No Quit item** — File menus in every frontend should end with Quit,
   which closes the app.
3. **Graph pane clutter** — the settings row lived at the pane's bottom
   while "Clear graph" sat at the top, and "Copy SVG" looked like a
   link next to a button-styled "Clear graph".
4. **Dashed extra curves** — every curve after the first was dashed;
   the first solid. Users expect all lines solid.
5. (Separately) **Windows blank screens** after upgrades, fixed by hand
   with Ctrl+F5.

## Decision

- **Menus (web/PWA):** a document-level `mousedown` listener closes the
  open menu when the click lands outside the menu bar — the native
  menubar contract. Activation of any item closes its menu (Edit items
  previously did not). Escape and the outside click both close; the bar
  itself keeps its APG menubar roles and keyboard pattern.
- **Quit:** File menus in the desktop/PWA app and the TUI end with a
  Quit item. Desktop: an IPC `quit` command exits the process. PWA: the
  app calls `window.close()`; browsers refuse for tabs they did not
  open, so after a moment the app says so honestly (localized hint).
  TUI: the item ends the event loop exactly like Ctrl+C.
- **Graph pane toolbar:** Clear graph and Copy SVG become equal,
  identically styled buttons, and the graph options row (POI toggles,
  line-width slider) moves up beside them — one wrapping toolbar above
  the plot. All controls stay real labelled form controls.
- **Solid curves:** all curves are solid in the live renderer and in
  saved SVGs. The dash patterns were the non-color channel (WCAG
  1.4.1); the replacement is a visible caption at each curve's end
  (saved SVG) plus the existing legend and `aria-label` (live pane) —
  a stronger channel, since it names the expression.
- **Boot self-heal:** the boot fallback retries once automatically —
  a failed first mount reloads the page instead of showing the error
  text (per-tab marker, cleared on any healthy mount). The desktop
  shell additionally disables the WebView2 disk cache
  (`additionalBrowserArgs: --disk-cache-size=0`), which was the source
  of the stale-bundle state Ctrl+F5 worked around.

## Consequences

- Menu behavior matches desktop habit; no menu markup moved to the
  native `popover` API because the APG roles and arrow-key pattern
  already deliver accessibility, and the outside-click contract was
  the missing piece.
- Saved SVGs change: solid lines plus expression captions; byte-stable
  and deterministic as before.
- The boot fallback's text now only appears when a retried load also
  fails — the diagnostics keep their naming-the-cause job.
