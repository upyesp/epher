# Website and GitHub Pages

The project is published at **https://epher.org/** (custom domain on
GitHub Pages), built and deployed by the `pages` workflow
(`.github/workflows/pages.yml`).

## Site layout

| Path | Content | Source |
|---|---|---|
| `/` | Landing page — hero, features, downloads | `site/index.html` (static HTML/CSS/JS, committed) |
| `/privacy.html` | Privacy (what stays on your device) | `site/privacy.html` |
| `/guide/<lang>/` | User guide, eight languages | `site/guide/<lang>.md` → built |
| `/examples.html` | Copyable examples (CLI, REPL, app) | `scripts/build-examples.mjs` → built |
| `/pwa/` | The web app (PWA, offline-first) | `crates/web/dist` (built by trunk in CI) |
| GitHub Releases | unified platform installers (ADR-0011) | built by `.github/workflows/release.yml` |

The PWA dist is laid out by `crates/web/index.html`: `copy-file` puts the
manifest/sw/icon at the dist root (a `copy-dir` would bury them in
`dist/public/` and break installability), and `public_url = "./"` in
`Trunk.toml` keeps every asset reference relative so the app works from any
mount point. `public/sw.js` is network-first for navigations (so redeploys
reach users) and runtime-caches assets for offline use; bump its `CACHE`
constant when the strategy changes.

The PWA is dark-only and shares the marketing site's design tokens
(2026 redesign: `--bg #141416`, `--panel #1d1f22`, `--accent #2dd4bf`,
`--muted #9a9ba2`, the same border-strong, focus, and curve palette
contrast numbers) — see `docs/research/modern-ui-accessibility.md` and the
contrast comments in `crates/web/index.html`.

The user guide (ADR-0018) has one source: `site/guide/<lang>.md`. The
website pages, the web/desktop overlay, and the TUI pager all render
those files; the apps embed them at build time, so a guide edit reaches
every frontend on the next build.

The app layout is ADR-0016: a fixed-viewport scientific calculator —
input, answer panel, scrollable history, and a five-tab keypad covering
every function the language supports — with the graph in a fixed pane
beside the calculator on desktop (≥880px) and one horizontal swipe away
on mobile (scroll-snap panes plus pane-switch buttons). ADR-0017 adds a
menu bar (File/Edit/Settings) above the panes, three themes (light,
dark, night — token sets selected by a `data-theme` attribute, with the
same recorded-contrast discipline as the base palette), and file
open/save; the TUI mirrors all of it (F10 menu bar, side-by-side graph
from 72 columns, OSC 52 clipboard). ADR-0033 fits the whole TUI to a
standard 80×24 terminal: the keypad is always on screen (Tab focuses
it), the bottom hint strip wraps to two rows so the full key guide is
visible, and the Settings menu marks its Theme, Language, and 3D View
sections with labeled rules. ADR-0034 adds the pointer: menus, history
picks, keypad clicks, and graph drags (2D pan / 3D orbit), wheel zoom,
and double-click reset.

The landing page links to release assets via
`https://github.com/upyesp/epher/releases/latest/download/<asset>` so download
links never need a version number.

## Landing page design

- **i18n**: eight locales (`en`, `zh-CN`, `hi`, `es`, `fr`, `ar`, `de`, `pt`). Detection,
  stored preference, and English fallback mirror the `Localizer` in
  `crates/i18n`; the static page reimplements the ~15-line negotiation in
  `site/app.js` (there is no wasm on the landing page). `lang`/`dir` (RTL for
  Arabic) follow the active locale (WCAG 3.1.1).
- **Themes**: light/dark via `[data-theme]`; defaults to
  `prefers-color-scheme`, toggle persists to `localStorage` (`epher-theme`).
  An inline script in `<head>` applies both theme and stored language before
  first paint — no flash.
- **Catalogs**: the per-language string catalogs live in
  `site/i18n/<lang>.js` (plain scripts defining `window.EPHER_I18N`),
  loaded before `app.js` on the landing, examples, and Privacy pages.
  `app.js` holds no strings of its own; English is the fallback for any
  key a catalog has not translated yet, so a language file may lag behind
  `en.js` harmlessly.
- **Design (2026 redesign)**: teal accent (the amber read like every other
  developer site), fluid type via `clamp()`, sticky translucent header,
  feature grid, and a disclosure (hamburger) nav below 880px — WAI-ARIA
  APG pattern: `aria-expanded` on the button, `hidden` on the nav while
  collapsed (out of the tab order), Escape closes and restores focus, a
  click outside closes, and a `<noscript>` style shows the links stacked
  when JavaScript is off. The research behind the design decisions is in
  `docs/research/modern-ui-accessibility.md`.
- **Pages**: `/` (landing: hero, features, downloads), `/privacy.html`,
  `/examples.html` — the same header/footer chrome, content strings
  under `privacy-*` / `ex-*` keys. The guide pages share the header
  chrome via `scripts/build-guide.mjs` (labels in its `CHROME` map)
  with the same disclosure-nav script inlined.
- **Examples page** (ADR-0036): one static page, generated by
  `scripts/build-examples.mjs` from the index.html chrome. The epher code
  blocks are never localized (ADR-0007) and ship identical in every
  language; the captions and section prose are `ex-*` catalog keys. The
  copy buttons reuse the guide's `.example`/`.copy-btn` pattern from
  `guide.css`, with the copy script inlined and its labels read from the
  active catalog. Run `npm run build:examples` to regenerate.
- **Hero animation**: the landing hero's graph card is the 3D saddle
  `graph3d x ^ 2 - y ^ 2` rotating slowly — a runtime port in `site/app.js`
  of the app's projection (ADR-0030): constant-size per-frame-centered view
  box, the width slider at 0.1 (mesh 1.2x, frame 1.4x), one static frame
  under reduced motion. The terminal card above it shows that exact
  command with no shell prompt, copy-pasteable into any frontend
  (ADR-0032); the header nav (landing, privacy, guide pages) carries an
  App link to `/pwa/` in all eight locales.
- **Icon**: the epher mark is the monogram "e" (from the epher.svg artwork)
  on a rounded tile. Three variants live in `site/` and `crates/web/public/`:
  `icon.svg` (dark tile, white glyph — the default, the favicon, and the
  desktop/app-icon source), `icon-light.svg` (light tile, dark glyph — used
  in the dark theme), and `icon-plain.svg` (transparent, white glyph). The
  header brand icon swaps per theme (CSS `content:url` in `styles.css` plus
  a JS `src` sync for engines without img-content support); the favicon is
  always `icon.svg`. Desktop icons (`crates/tauri-app/src-tauri/icons/`) are
  regenerated with `cargo tauri icon site/icon.svg`.
- **Accessibility**: WCAG 2.2 AA — see `docs/accessibility.md`. Contrast
  values for both themes are recorded in `site/styles.css`; keep them in
  spec when editing colors.
- The English text in `index.html`, and `privacy.html` is the
  noscript fallback; `app.js` swaps in the other locales.

### Adding a string

1. Add the English text in the page with `data-i18n="key"` (or
   `data-i18n-aria` for aria-labels).
2. Add the key to `site/i18n/en.js` and to the seven other catalogs in
   `site/i18n/` (English fallback covers the gap until they land).
3. Keep the `docs/accessibility.md` checklist in mind (labels, language).

## User guide

`site/guide/<lang>.md` holds the user guide in each of the eight languages
(the master is `en.md`; translate it and keep the examples identical). The
`pages` workflow converts them to HTML with
`scripts/build-guide.mjs` (marked + a small template; heading ids, table of
contents, RTL, themes, and the WCAG patterns come from the shared
`styles.css`/`guide.css`). Output goes to `site/guide/<lang>/index.html`,
which is gitignored and generated in CI — run `npm run build:guide`
locally to preview. The landing page links to `guide/<lang>/` and the link
follows the visitor's active language. The in-app guide (web overlay and
TUI pager, ADR-0018) opens with a table of contents of the top-level
chapters: the web overlay's ToC buttons scroll the dialog body, the TUI
pins the chapter list above the content and jumps on click or number key.

Fenced code blocks have three kinds (keep the examples identical across
translations, and add new ones in the same order so the kinds stay aligned):

- ` ```epher ` / ` ```sh ` — what the reader types: rendered as a code block
  with lightweight epher syntax highlighting and a copy-to-clipboard button
  (labels localized in `CHROME` in `build-guide.mjs`)

The web app's graphing (ADR-0014/0015) is documented in guide section 2.4:
curves, points of interest, sliders with play/pause animation, 3D surfaces
(`graph3d`), and export.
- ` ```text ` — what epher answers, REPL/TUI transcripts, URLs, paths: the
  plain box

Adding a guide language: add `<lang>.md`, add chrome strings in
`build-guide.mjs`, add the landing page strings in `site/app.js`, and add
the option to the `lang-select` in `site/index.html`.

## Releases

Push a version tag and the `release` workflow builds and attaches everything:

```
git tag v0.3.1
git push origin v0.3.1
```

One download per platform (ADR-0011): every installer carries the unified
`epher` executable — one-shot CLI, REPL (`epher repl`), piped
scripts (`epher -`), TUI (`epher tui`), and the desktop GUI (bare `epher` /
`epher gui`). The command surface follows [clig.dev](https://clig.dev/)
(ADR-0013): examples-first `-h`, full `--help`, `epher help` pages the
manual (`man epher` where installed), errors on stderr, exit codes 0/1/2.
Graphing (ADR-0014) is the shared engine's other half: the TUI and the
GUI/PWA plot multiple curves (Cartesian, parametric, polar, and shaded
regions) with trace, points-of-interest analysis, sliders for constants,
SVG export, and `table` commands; the one-shot CLI stays a pure
expression evaluator.
Windows installs two subsystem builds of the same program
(console `epher.exe` on PATH; GUI-subsystem `epher-gui.exe` as the
double-click target — no console flash); macOS and Linux install the one
binary, with the `.app` bundle and the `Terminal=false` desktop entry
deciding GUI vs terminal. The old per-frontend archives (v0.1.x–v0.2.x)
are gone.

Stable asset names (the landing page depends on them):

```
epher-windows-x86_64.exe
epher-macos-aarch64.dmg
epher-linux-x86_64.{deb,rpm,AppImage}
```

macOS is Apple Silicon only (ADR-0025); the Intel build and its
download link were removed in v0.4.13. The release workflow smoke-tests
the unified CLI on the macOS and Linux jobs before packaging.

- Windows: NSIS installer; `installerHooks` (`nsis-hooks.nsh`) adds the
  install dir to the user PATH so `epher` works from any terminal. The
  installer template is vendored and themed (`nsis/installer.nsi`,
  light MUI2 color defines + epher header/sidebar bitmaps, ADR-0028;
  earlier releases tried a dark theme (ADR-0025) but MUI2 cannot darken
  custom pages or classic controls, so the wizard mixed dark and light
  pages — the whole theme now uses the official MUI2 mechanism in
  classic light colors; the header bitmap puts the epher logo in the
  top-left corner of every page — MUI2's default header layout — and
  the welcome/finish sidebar carries a larger mark; the uninstaller
  shares the header bitmap; the per-control repaint that locked the
  destination page was removed in ADR-0026, and ADR-0027/0028 pin the
  finish-page checkbox labels via `SetSysColors(COLOR_BTNTEXT)`, a
  single system call with no window procs touched), and the
  uninstaller's "delete app data" checkbox starts checked and removes
  `%USERPROFILE%\.epher`. `makensis nsis-check.nsi` and `makensis
  nsis-theme-check.nsi` compile-verify the hook and theme additions
  (locally and as a dedicated job in the release workflow); the full
  rendered template compiles in the Windows bundling job.
- macOS: unsigned dmg; the app's "Install the epher command" button
  symlinks `/usr/local/bin/epher` (osascript fallback for admin rights).
- Linux: deb/rpm install `epher` into `/usr/bin` and the man page into
  `/usr/share/man/man1/epher.1` (generated by
  `cargo run -p epher-cli --example gen-man`, from packaging/man/epher.1);
  their postrm removes every user's `~/.epher` on uninstall (ADR-0025,
  upgrades untouched); the AppImage covers Arch
  and every other distro. The Linux leg builds on ubuntu-22.04 (glibc
  2.35) so the binary runs on Debian 12 and Ubuntu 22.04 or newer.
  `tauri.linux.conf.json` (and its macOS twin) set the window
  `backgroundColor` so launches never flash white — kept out of the
  base config because it broke WebView2 painting on Windows.

File → Save history / Save script pre-fill `epher-history.ehs` /
`epher-script.esr` in every frontend (TUI prompt, desktop dialog, PWA
download) — the names are editable suggestions, no extension filter is
applied (ADR-0027) — and the desktop dialog runs off the main thread so
the window stays live while it is open.

macOS and Windows builds are unsigned. If the landing page's download
links need to change (e.g. a new platform), change the names here and in
`site/index.html` together.

## First-time setup (already done for this repo)

1. Enable Pages with the workflow source:
   `gh api repos/upyesp/epher/pages -f build_type=workflow`
2. Set the custom domain (DNS: apex A records to GitHub's Pages IPs,
   `www` CNAME to `upyesp.github.io`):
   `gh api repos/upyesp/epher/pages -X PUT -f cname=epher.org`
3. Re-enable "Enforce HTTPS" once the certificate state is `approved`
   (the setting resets when a domain is added).
4. Push `main` — the `pages` workflow builds and deploys.
If the repository ever gets recreated, redo steps 1–3; the custom domain
is repo Pages settings, not stored in the repo (no `CNAME` file is needed
with the workflow deploy).
