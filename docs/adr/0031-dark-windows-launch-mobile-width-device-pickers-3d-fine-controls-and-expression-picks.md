# ADR-0031: Dark Windows launch, mobile width range, device file pickers, 3D fine controls, and expression-only history picks (v0.4.19)

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

Eight asks in one release:

1. On Windows the desktop window's first frame was light again, flipping
   to dark once the page painted — a timing-dependent flash.
2. On mobile the graph lines are too thick: the width slider needs a
   mobile range of 0–0.2 step 0.01 with 0.1 the default.
3. On Android, File → Open history (and Open script) surfaced a system
   permissions panel instead of the file explorer.
4. The website menu needs an "App" item that launches the PWA.
5. 3D plots need fine controls — horizontal rotation, vertical rotation,
   zoom, each −1..1 step 0.1 default 0, live — on top of the existing
   orbit gestures, in every frontend.
6. The hero's code block should show the command that draws the rotating
   saddle: `graph3d x ^ 2 - y ^ 2`.
7. The landing lede should read "command line, interactive REPL, full
   TUI interface, desktop GUI app, and an offline web app" and keep the
   history claim honest ("between sessions and user interfaces").
8. History picks should load the expression only — no `= answer` suffix —
   so the user can edit and re-run directly.

## Analysis

- **Windows flash:** the v0.4.3 blank-window fix removed `backgroundColor`
  from the shared window config (it broke WebView2 painting on Windows);
  linux/macos got it back through overlays (ADR-0025), so Windows's dark
  first frame depends on timing alone. The v0.4.1-era fix — WebView2's
  `--default-background-color` argument — was dropped in v0.4.2 only
  because the then-present `backgroundColor` made it redundant. Restoring
  that argument in the Windows overlay paints the webview's own first
  frame dark without touching the window background that broke painting.
- **Mobile width:** the slider is one control in both layouts; its range
  becomes a property of the 880px breakpoint. A stored desktop width
  (up to 4.0) is out of range on mobile, so each layout's load checks
  its own range and falls back to its own default (0.1 mobile, 1.0
  desktop); crossing the breakpoint re-clamps the live value through the
  same setter (which reads the layout itself). The ADR-0028 0.1 hairline
  floor stays a desktop-only rule — mobile's 0.00 floor is explicit in
  the request.
- **Android pickers:** the hidden `display:none` file input is the
  fallback everywhere today. The PWA now prefers the File System Access
  API's `showOpenFilePicker` (no file-type filter, ADR-0028) — the
  device's file explorer, straight from the menu tap. Cancellation
  (AbortError) stays silent like the save dialog; any other failure, and
  browsers without the API, fall back to the existing hidden input; the
  desktop shell keeps its native dialog path.
- **3D fine controls:** offsets ride on the orbit base view through one
  core mapping, `View3D::with_offsets` — horizontal adds `h × π` yaw,
  vertical adds `v × 0.8` pitch (the default pose's full range stays
  live: 0.6 + 0.8 = 1.4, exactly the pole clamp), zoom multiplies the
  camera distance by `2^-z`. 0 = the pose unchanged. The web renders
  three labelled sliders above the plot only while surfaces exist and
  resets them when a 3D graph is drawn into an empty pane; the TUI adds
  the same three rows to the Settings menu (only while surfaces exist),
  adjusted with Left/Right ±0.1 while highlighted, values shown in the
  labels; the TUI's SVG export and ASCII renderer use the effective
  pose, and the web's Copy SVG does too.
- **History picks:** ADR-0027's verbatim picks made re-running a
  two-step edit (strip the suffix by hand). The recorded format's
  `  ` separator makes the answer suffix structurally unambiguous — not
  a heuristic — so picks now load everything before the last
  `  = ` / `  error:` / `  warning:`. Graph lines and definitions pass
  through untouched. Applies to the web app, the desktop app (same
  frontend), and the TUI.
- **Site:** the header nav (landing, about, privacy, and the built guide
  pages) gains an App link to `/pwa/` in all eight locales; the hero's
  terminal card shows `$ graph3d x ^ 2 - y ^ 2` — the exact command the
  hero animates — and the lede is the requested wording, translated.

## Decision

- The Windows overlay passes `--default-background-color=141416` to
  WebView2; `backgroundColor` stays out of the Windows config forever
  (v0.4.3). *(Superseded by ADR-0032: WebView2 parses the argument as
  AARRGGBB, so `141416` was silently invalid and the flash remained —
  the current fix is hidden-until-loaded plus the valid `FF141416`.)*
- The width slider's range is mobile 0–0.2 step 0.01 (default 0.1) and
  desktop 0.1–4 step 0.1; stored values and breakpoint crossings clamp
  through the same setter.
- The PWA opens files through `showOpenFilePicker` with a hidden-input
  fallback.
- 3D fine controls ship in the web frontend (PWA + desktop GUI) and the
  TUI settings menu, all mapping through `View3D::with_offsets`.
- History picks load the expression part of an entry.

## Consequences

- The `adr31` browser suite pins the mobile slider range and re-clamp,
  the 3D sliders' real-time effect on the rendered mesh, the expression
  pick, the site's App link and the hero code block; the pty TUI smoke
  exercises the settings rows and the stripped pick; core tests pin
  `with_offsets` and `history_expression`.
- The Windows flash is covered by installer-marker verification (the
  flag is a build-time argument; the shipped exe embeds it).


## Amendment (2026-08-31): fine controls become the tuning strip (ADR-0041)

The three fine-control sliders (horizontal rotation, vertical
rotation, zoom) leave their text labels and the space below the plot
for the tuning strip directly above it, beside the line-thickness
slider. Each carries an icon - an arc arrow, turned a quarter-turn
for the vertical axis, a magnifier for zoom - with the words in its
tooltip ("horizontal rotation speed", "vertical rotation speed",
"zoom speed"). Spans, steps, and the reduced-motion behavior are
unchanged; the strip is shared by surfaces and the solar system and
wraps on phones.
