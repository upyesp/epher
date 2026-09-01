# Architecture Decision Records

epher's significant technical decisions, in the [Nygard ADR
style](https://cognitect.com/blog/2011/11/15/documenting-architecture-decisions):
Context, Decision, Consequences, Status. This directory is the record of
*why* the code is the way it is.

## Process

- **Numbers are immutable.** ADR-0007 is ADR-0007 forever; nothing is
  renumbered, deleted, or renamed (the file name of an ADR whose title
  later broadens keeps the original name — see ADR-0035).
- **Amendments stay in place.** A decision that evolves is amended in
  its own file (see ADR-0004, ADR-0015, ADR-0035) or superseded by a
  later ADR; earlier records are *never rewritten*, only annotated with
  a pointer.
- **Supersession is explicit.** When a later ADR reverses or absorbs an
  earlier decision, the earlier record says so ("Superseded by …").
- **One decision per ADR.** Each ADR records one decision (or one
  tightly coupled cluster) with its own context and consequences. The
  release-batch ADRs (0016–0034) predate this rule and remain as they
  were written — the rule governs new ADRs.

## Index

| # | Title | Date | Status |
| --- | --- | --- | --- |
| 0001 | Compile CLI and TUI natively; compile core, web, and desktop to WASM | 2026-08-13 | accepted |
| 0002 | Two physical Stores, one schema | 2026-08-13 | accepted |
| 0003 | Desktop PWA bridges to the Native Store via the File System Access API | 2026-08-13 | accepted |
| 0004 | Build a custom DSL instead of embedding a language engine (amended: `\n` ≡ `;`) | 2026-08-13 | accepted |
| 0005 | Float-by-default numerics behind one `Value` enum; GMP/rug excluded | 2026-08-13 | accepted |
| 0006 | Graphing splits into a core Sampler and per-frontend renderers (amended: parametric/polar shipped in 0014; 3D shipped in 0015) | 2026-08-13 | accepted |
| 0007 | Localize the UI; never localize the scripting language | 2026-08-13 | accepted |
| 0008 | UI localization: Fluent catalogs embedded at build time (amended: de/pt joined) | 2026-08-13 | accepted |
| 0009 | Accessibility: WCAG 2.2 AA for the web/PWA, keyboard-first terminals elsewhere (amended: automated axe scans in the `a11y` Playwright suite) | 2026-08-13 | accepted |
| 0010 | The Desktop App Owns the Native Store; the Webview Bridges to It (amended: the store carries the shared session snapshot — variables and `ans` — and the CLI one-shot joins the store; publish/subscribe — live sync between open frontends, and the Windows store path fixed) | 2026-08-14 (amended 2026-08-27) | accepted |
| 0011 | One `epher` Binary Hosts Every Frontend | 2026-08-15 | accepted |
| 0012 | User-defined constants: `const name = value`, visible like `pi` | 2026-08-16 | accepted |
| 0013 | The command line follows clig.dev | 2026-08-17 | accepted |
| 0014 | Graphing expansion: multi-curve, trace, analysis, tables, sliders (3D deferral superseded by 0015) | 2026-08-17 | accepted |
| 0015 | Animation and 3D graphing (amended: near-plane clipping; playback rate, drag orbit, touch; the slider rows sit above the points-of-interest list; the pane shows one kind at a time; per-kind line-width sliders and legend visibility checkboxes; deadline-paced minimal tick; 3D space curves and positioned points per 0037) | 2026-08-17 (amended 2026-08-27, 2026-08-29) | accepted |
| 0016 | Calculator-style fixed layout with keypad input (keypad banks amended by 0022/0024; mobile focus rule amended by 0035; TUI keypad amended by 0019/0024/0033) | 2026-08-21 | accepted |
| 0017 | Menu bar, themes, and file open/save (menubar → icon rail: 0032) | 2026-08-22 | accepted |
| 0018 | The user guide inside the app, and one button to clear the graph pane (amended 2026-08-27: in-app table of contents with click-to-jump) | 2026-08-22 | accepted |
| 0019 | Graph pane settings, a full-function keypad, and the hints row | 2026-08-22 | accepted |
| 0020 | SVG export from every frontend and the graph pane's options row | 2026-08-23 | accepted |
| 0021 | `ans` — the previous answer (amended: `ans` persists as part of the shared session snapshot — desktop only — and travels live between open frontends) | 2026-08-23 (amended 2026-08-27) | accepted |
| 0022 | Number bases — `0b`/`0o`/`0x` literals and `bin`/`oct`/`hex` | 2026-08-23 | accepted |
| 0023 | Native-feeling menus, pane toolbar, solid curves, and boot self-heal | 2026-08-23 | accepted |
| 0024 | Save dialogs, the 0x keypad bank, and TUI menu paint order | 2026-08-23 | accepted |
| 0025 | Apple Silicon only, dark launch, 3D pane controls, themed NSIS, uninstall cleanup, consistent TUI layout, split Open (NSIS theme superseded by 0026–0028) | 2026-08-24 | accepted |
| 0026 | Three v0.4.13 regressions — NSIS repaint removed, save-dialog arg casing, 3D orbit accumulation | 2026-08-24 | accepted |
| 0027 | Extensions, readable installer checkboxes, 60fps 3D, and clickable history (amended 2026-08-27: multi-line scripts are one history item, picked whole, with visible item boundaries) | 2026-08-24 | accepted |
| 0028 | Unfiltered open dialogs, 0.1 floor for line width, light installer theme with logo | 2026-08-24 | accepted |
| 0029 | Auto-slide to the graph pane after drawing on mobile | 2026-08-24 | superseded by 0035 (absorbed; the focus consequence was reversed by 0030) |
| 0030 | Five frontends everywhere, mobile blur after the auto-slide, scripts as one history entry, and a rotating 3D hero | 2026-08-24 | accepted (the blur decision is carried by 0035) |
| 0031 | Dark Windows launch, mobile width range, device file pickers, 3D fine controls, and expression-only history picks (Windows launch superseded by 0032) | 2026-08-25 | accepted |
| 0032 | Dark Windows launch, slider ends, 3D spin controls, vertical icon rail, and the bare hero command | 2026-08-25 | accepted |
| 0033 | TUI layout fits 80×24 — always-visible keypad, wrapped hints, and sectioned settings | 2026-08-25 | accepted |
| 0034 | TUI mouse support — menus, history, keypad, and graph manipulation | 2026-08-25 | accepted |
| 0035 | Mobile PWA usability — the onscreen keypad is the primary input, and a drawn plot slides into view (file name keeps the original keypad-focus title) (amended 2026-08-26, 2026-08-27, 2026-08-28: touch no-arrow hints, per-kind sliders, examples tap-to-stage, visible-space fitting) | 2026-08-25 (amended 2026-08-26, 2026-08-27) | accepted |
| 0036 | The website Examples page — copyable code for every frontend (amended 2026-08-27: the app section leads, four CLI examples moved into it, touch taps stage examples in the app; amended 2026-08-28: base-conversion example in the app section, the REPL section last) | 2026-08-27 (amended 2026-08-27, 2026-08-28) | accepted |
| 0037 | Astronomy units, constants, time functions, and ephemeris: unit-suffix literals, the solar-ephemeris facade, accessor functions, and solar3d | 2026-08-29 | accepted |
| 0038 | Zoom on every tile, the solar legend, guide search, shareable history, and the keypad's dead keys | 2026-08-30 | accepted |
| 0039 | A fixed-height keypad with scrolling, and a meaning for every key | 2026-08-30 | accepted |
| 0040 | PHP-style comments, a roomier graph, script files for the CLI and REPL, a fuller share, and a legend that never leaves | 2026-08-31 | accepted |
| 0041 | The history trash, tuning strips above the plot, and a frame that holds still | 2026-08-31 | accepted |
| 0042 | Percent, the constants catalog, number theory, suggestions with F1, and PNG export | 2026-08-31 | accepted |
| 0043 | Complex numbers, equation solving, numeric calculus, exact fraction display, and result formats | 2026-08-31 | accepted |
| 0044 | Lists, statistics, linear regression, distributions, tests, data plots, and table upgrades | 2026-09-01 | accepted |

## Decision chains

Some topics evolved across several ADRs; the current state of each lives
at the end of the chain.

- **Graph-pane options and line width** — 0019 (Settings menu) → 0020
  (pane row, slider 0.5–4) → 0023 (top toolbar) → 0025 (3D toolbar) →
  0027 (0–4 step 0.1) → 0028 (0.1–4 floor) → 0031 (mobile range) →
  0032 (slider ends) → **0035 (per-kind widths on mobile; desktop
  shared width 0.1–4)**.
- **Installer theme** — 0025 (dark via `SetCtlColors` walk) → 0026
  (walk deleted; official MUI2 mechanism only) → 0027 (`SetSysColors`
  finish checkboxes) → **0028 (light, uniform, logo-bearing wizard)**.
- **Windows first frame** — 0025 (overlays; Windows excluded) → 0031
  (`--default-background-color=141416`, ineffective) → **0032
  (hidden-until-loaded + valid AARRGGBB `FF141416`)**.
- **Mobile graph-pane behavior** — 0029 (auto-slide) → 0030 (blur after
  the slide) → 0031 (mobile width range) → **0035 (the mobile PWA
  usability contract: keypad focus discipline, slide-in and slide-back,
  3D swipe rotation, per-kind widths)**.
- **TUI keypad** — 0016 (4×5, Tab-gated) → 0019 (four banks) → 0024
  (0x bank) → **0033 (always visible; banks and geometry sized to the
  real panes)**.
- **Keypad hints and height** — 0016 (the five-row digits tab) →
  **0039 (every tab is the digits tab's height and longer banks scroll;
  every key speaks a localized hint through aria-labels, the docked
  hint bar, the touch captions toggle, and the TUI's `?` key-help
  overlay)**. → **0042 (the hints suggest and answer F1)**
- **History** — 0021 (`ans`) → 0025 (open-history replaces) → 0027
  (clickable history) → 0030 (`;` scripts as one entry) → **0027
  amendment (multi-line scripts as one item, picked whole, visible
  boundaries)**. → **0041 (the trash beside the
  heading, clickable in the terminal too)**.
- **Website guide and examples** — 0018 (one guide, three renderers) →
  **0018 amendment (in-app table of contents)**; → **0036 (Examples
  page with the guide's copy buttons)**.
- **Astronomy** — **0037 (unit-suffix literals as SI sugar, astro constants
  and time functions, solar-ephemeris behind a core facade, accessor
  functions, solar3d)** → **0015 amendment (3D space curves and positioned
  points render the solar system)**.
- **Graph interaction** - 0015 (orbit) → 0031 (fine controls, ±2× zoom)
  → 0034 (TUI wheel/drag) → **0038 (wheel + pinch on every tile, the
  slider spans two decades each way, zoomable windows re-sample the
  plot; the amendment: stable solar frames, the spin-loop cell fix,
  Help above Settings, TUI guide search and POI copy)**. → **0041 (listeners follow the replaced
  SVG node; the 3D and solar frames fit the bounding sphere and never
  resize while moving)**.
- **Solar system** - **0037** → **0038 (per-body legend checkboxes on
  the solar pane)**. → **0040 (hiding every body leaves an
  empty framed plot with the legend intact)**.
- **Graph commands** - 0014 (cartesian/parametric/polar grammar) → 0019
  (hints) → 0020 (SVG export) → 0038 (per-curve legend) → **0044 (data
  plots: `graph scatter`, `graph histogram`, `graph boxplot`)**.
- **History** - 0021 (`ans`) → 0025 → 0027 → 0030 → **0038 (share icon
  on every item, `?expr=` links stage the entry; `clear`/`history`
  keypad keys finally run)**. → **0042 (auto-ans on an empty entry)**
- **Calculations** - 0005 (float default) → 0022 (number bases) → 0042 (percent,
  primes) → **0043 (complex values, numeric solve, calculus, exact fractions,
  engineering/scientific/grouped formats)** → **0044 (lists, elementwise
  arithmetic, statistics, distributions, tests, regression)**
  → **0045 (seeded random, the constants browser)**
- **In-app guide** - 0018 → 0018 amendment (ToC) → **0038 (search box
  with chapter + snippet hits)**.
- **Script language** - 0001 (one grammar) → 0037 (unit suffixes) →
  **0040 (PHP-style comments: `//`, `#`, and `/* ... */`)** → **0042 (percent, physics constants, number theory)**
  → **0043 (`solve` statement, `4i` literals, lazy `derivative`/`integral` arguments)**
  → **0044 (`{1, 2, 3}` list literals, `list[i]` indexing, the `graph scatter/histogram/boxplot` family, `table … derivative …`)**
  → **0045 (seeded random, the constants browser)**
  → **0046 (quantities, unit prefixes, the `in` conversion operator)**
  → **0047 (bitwise operators, the `bits(n)` word size)**
- **Script files** - 0013 (`epher -` pipes) → **0040 (`epher file.es`
  runs a script file; the REPL's `load` runs a file or a saved script;
  `save script name` still stores one)**.
- **Share** - **0038 (share icon, `?expr=` links)** → **0040 (the share
  reads as message, expression, link)**.
- **Graph pane layout** - 0016 (fixed-size plot) → 0031 (fine controls)
  → **0040 (a 38vh plot floor, a wrapping legend, icon toolbar commands,
  sliders below the plot, no "3D" heading, an unboxed Clear history,
  and 34px keypad tabs)**. → **0041 (tuning strips above
  the plot, the captioned keypad's own exact window)**.
