# ADR-0017: Menu bar, themes, and file open/save

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

Three requests arrived together after the v0.4.3 layout work:

1. **A menu bar** on every frontend (TUI, desktop GUI, PWA) — `File`
   (Open, Save), `Edit` (Cut, Copy, Paste), `Settings` (Theme, Language).
2. **Three themes** — Light, Dark, Night. Night is new: long-wavelength
   reds on a near-black background, for people who keep their night
   vision; blue-free light does not contract the pupil the way a bright
   or cool screen does.
3. **File open/save** for scripts and history, plus clipboard access from
   the menus.

## Decision

### Themes are token sets, not paint jobs

The web/desktop UI already drew every color from CSS custom properties
(`--bg`, `--panel`, `--text`, `--muted`, `--accent`, `--border`, and the
curve palette `--curve-0..3`). Themes are therefore three token sets
selected by a `data-theme` attribute on the root element, with the
default dark set as the fallback so the first painted frame is unchanged.

- **Dark** — the existing palette, unchanged.
- **Light** — near-white canvas, darker teal accent (`#0e8074`, 4.5:1 on
  the background), darker curve colors; every value re-checked against
  WCAG 1.4.3 (text 4.5:1) and 1.4.11 (non-text 3:1) and recorded in the
  CSS comments.
- **Night** — `#0d0000` background, red-tinted text (`#ffb3a8`, 12.1:1),
  red accent (`#ff6b5a`, 7.4:1), and a warm-only curve palette
  (`#ffb340`, `#ff9e8a`, `#e0483e`) so no blue light leaks in; the
  existing dash patterns still distinguish curves beyond color.

The TUI applies the same three themes as explicit ratatui colors
(night: text `Rgb(255,179,168)` on `Rgb(13,0,0)`, selection
`Rgb(255,107,90)`; light: black on white; dark: the terminal's natural
colors).

### `theme` is a shell command

Like `language` (ADR-0008), the theme is a session command — `theme
light|dark|night` — classified by the shell kernel, persisted through
the same store (a `theme` setting beside `language`), and applied by
each frontend. The Settings menu is the discoverable spelling of the
same command; both paths persist. Scriptability and the menu never
diverge.

### Menus share one APG structure, with frontend-honest actions

The web/desktop menu bar follows the WAI-ARIA menubar pattern
(`role="menubar"`, top-level `role="menuitem"` buttons with
`aria-haspopup="menu"`, dropdowns as `role="menu"` with
`menuitem`/`menuitemradio` items, Escape closes, focus rings stay).
The TUI draws the same three menus (F10 opens, arrows move, Enter
activates, Escape closes) with the same items and radio semantics for
the current theme and language. Language names are always written in
the language itself (native names), independent of the UI language.

Each frontend implements the actions its platform allows, honestly:

- **Web/PWA/desktop** — Open loads a picked file's text into the entry
  field (review before running); Save downloads the history or the entry
  script; Cut/Copy/Paste use the async Clipboard API (paste inserts at
  the cursor; blocked reads explain Ctrl+V instead of failing silently).
- **TUI** — Open/Save prompt for a path in the input row (failed
  operations keep the path for correction); Copy/Cut emit the clipboard
  via **OSC 52** (works locally and over SSH, base64-payloaded,
  dependency-free encoder); Paste cannot read the terminal clipboard
  portably, so it explains the terminal's own paste key instead.

### The TUI gets the desktop layout on wide terminals

At ≥104 columns the TUI splits horizontally: the calculator column
(input, result, history, keypad, hints) on the left, the graph in its
own section on the right — mirroring the desktop GUI/PWA arrangement.
Narrow terminals keep the ADR-0016 vertical stack. The ASCII plot
renderers already scale to arbitrary widths, so the right-hand panel
uses the area it is given.

## Consequences

- Theme and language changes persist in the desktop store (desktop,
  TUI) and in localStorage (PWA); the desktop's `InitState` carries the
  stored theme at startup.
- The web app's fixed viewport budget gained a menu bar row; the entry
  field also sits slightly below the top bar instead of flush against
  it.
- **Amendment (v0.4.5):** on mobile (<880px) the inline menu bar folds
  into a hamburger button whose panel lists the same three menus as
  labeled groups (one `role="menu"` containing File, Edit, Theme, and
  Language groups, radio items included); the inline bar stays on
  desktop. The first release shipped the File→Open file input without
  its hiding rule, so the native picker button rendered in the top bar
  on phones — the input is now `display: none` (programmatic `click()`
  still opens the picker) and the result region carries `tabindex="0"`
  so its overflow scroll stays keyboard-reachable.
- A latent Fluent bug surfaced during translation: `{name}` is a
  message reference, not a variable — `theme-set` uses `{ $name }` in
  all eight catalogs.
- The guide's menus-and-themes section arrived with ADR-0018 (in-app
  guide), ADR-0024 (keypad banks), and ADR-0033 (TUI table).
- The inline APG menubar described here was replaced at the desktop
  breakpoint by a vertical icon rail in ADR-0032; the hamburger panel
  below 880px is unchanged.
