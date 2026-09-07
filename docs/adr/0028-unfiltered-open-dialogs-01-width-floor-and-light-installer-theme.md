# ADR-0028: Unfiltered open dialogs, 0.1 floor for line width, light installer theme with logo (v0.4.16)

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

Three reports after v0.4.15:

1. File → Open history / Open script showed the OS file explorer
   filtered to `.epher` and `.txt` — the user wants every file type
   visible.
2. The line-width slider's minimum of 0 was surprising: at 0 the curve
   lines vanish (a literal SVG stroke-width of 0), which reads as
   broken rather than intentional. Minimum should be 0.1.
3. The NSIS installer mixed dark and light pages. MUI2's theme
   mechanism (ADR-0025/0026/0027) can only darken the pages MUI2
   itself paints; the nsDialogs reinstall page and every native
   control stay in classic light colors. The result: a wizard that
   flipped between dark and light page to page. The user also asked
   for the epher logo on the installer and uninstaller pages.

## Analysis

- **Open filter.** The web frontend's hidden file pickers carried
  `accept=".epher,.txt,text/plain"`, which the webview's native file
  explorer turns into a visible type filter. The open flow reads any
  text file regardless — the filter only restricts what the user can
  *see*.
- **Slider floor.** ADR-0027 set `min="0"` per the then-requested
  range; the zero-stroke rendering it enables was confusing in
  practice. The floor moves to 0.1 (a hairline), maximum 4.0, step
  0.1.
- **Installer theme.** MUI2 exposes no mechanism to darken arbitrary
  custom pages or classic controls; per-control repainting is
  permanently off the table (ADR-0026: `SetCtlColors` subclasses
  controls, and double-subclassing MUI-managed windows made v0.4.13
  installers vanish controls and lock up). Light pages therefore
  "have to be light", as the user put it — so the whole wizard goes
  light: one consistent classic look instead of a dark/light mix.
  The logo: MUI2's default header layout (no `MUI_HEADERIMAGE_RIGHT`)
  draws the header bitmap in the **top-left corner of every page**,
  and the welcome/finish sidebar is a second, larger mark. Both are
  the official mechanism; the uninstaller shares the header bitmap.

## Decision

- **Open dialogs:** the hidden file inputs carry no `accept`
  attribute; the OS file explorer lists all file types, matching the
  desktop save dialogs (unfiltered since ADR-0027).
- **Slider:** `min="0.1" max="4" step="0.1"`.
- **Installer/uninstaller theme:** light, uniform, and logo-bearing —
  `MUI_BGCOLOR F0F0F0` (classic dialog gray, so MUI-painted pages
  match the nsDialogs page and native controls exactly),
  `MUI_TEXTCOLOR 000000`, `MUI_INSTFILESPAGE_COLORS "FFFFFF 000000"`.
  `SetSysColors(COLOR_BTNTEXT, 0x000000)` stays in both `.onInit`s:
  the finish-page checkboxes need classic-dark text on the light
  pages, and the call makes that deterministic regardless of the
  user's system colors. Header bitmap: light banner carrying the
  epher mark on the left (top-left corner of every page); sidebar
  bitmap: light column with a larger mark on the welcome/finish
  pages; both regenerated from the same monogram artwork as the app
  icons, in the page gray so they blend seamlessly.

## Consequences

- `nsis-theme-check.nsi` compiles the light defines, both bitmaps,
  and the pinned classic text color, so CI keeps the harness and the
  template in agreement.
- Installer verification switches from dark-artwork byte-identity to
  light-artwork byte-identity (same mechanism: the shipped
  `$PLUGINSDIR` bitmaps must match the repo files byte-for-byte).
- The 0.4.13–0.4.15 dark look is gone; every wizard page is now
  light by construction, not by omission.
