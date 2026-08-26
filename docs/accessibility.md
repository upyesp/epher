# Accessibility (WCAG 2.2 AA) — audit and fixes

Living audit of epher against WCAG 2.2. Target: **Level AA** for the web/PWA
(and the Tauri shell, which wraps the same page). The TUI and CLI live in
terminals, so the applicable criteria are the keyboard-operability,
understandability, and theme-respect ones; the terminal emulator owns the rest
(contrast, font zoom, screen-reader output).

Contrast ratios below are computed (WCAG relative-luminance formula); text
checks use 4.5:1, non-text (UI component boundaries, focus indicators) 3:1.

## Web / PWA (and Tauri shell)

### Perceivable

| Criterion | Status | Evidence / notes |
|---|---|---|
| 1.1.1 Non-text content | PASS | Icon is a favicon (no alt needed). Button has text content plus `aria-label="Evaluate"` (2.4.6). The graph SVG is `role="img"` with a `title` and an `aria-label` naming the plotted expression, and a visible caption (`y = <source>`) sits above it — the TUI pattern, ported (ADR-0009). The exported/SVG document keeps the same `role`, `title`, and `aria-label`, so a saved file is accessible outside the app too (ADR-0020). |
| 1.3.1 Info & relationships | PASS | Native `form`/`input`/`button`, `ul` history, `h1` (visually hidden: the app name). Keypad tabs are an APG tablist (`role="tab"`/`tabpanel`, `aria-selected`, `aria-controls`); each button's accessible name is its token label. The menu bar (ADR-0017/0032) follows the APG menubar pattern: a vertical `role="menubar"` rail (`aria-orientation="vertical"`) of top-level `role="menuitem"` icon buttons with the menu name as `aria-label` and tooltip, `aria-haspopup="menu"`/`aria-expanded`, dropdowns as `role="menu"` with `menuitem`/`menuitemradio` items (`aria-checked` marks the active theme and language, with a non-color ✓ marker). The mobile hamburger panel is one `role="menu"` with File/Edit/Theme/Language/Help group labels; the graph pane's options row (ADR-0020) holds the display toggles as real labelled `input[type="checkbox"]` form controls and the line width as an `input[type="range"]` slider — focusable, keyboard-adjustable (arrows), each with a visible text label and `focus-visible` rings; the guide overlay (ADR-0018) is a `role="dialog"` with `aria-modal`, a labelled heading, an imperatively focused Close button, and a keyboard-scrollable body (`tabindex="0"`). The macOS-only "install the epher command" button (ADR-0011) is a native `button` after the status region; its outcome reports through the existing `role="status"` live region. |
| 1.3.2 Meaningful sequence | PASS | One fixed screen (ADR-0016): top bar (menus) → input → answer → history → keypad, top to bottom; the graph pane follows in DOM order. |
| 1.4.1 Use of color | PASS | No color-only information (result is text; errors are text). The graph's curve palette pairs each color with a visible caption at the curve's end (saved SVG) and the legend naming every expression (live pane), and the slider/checkbox row carries text labels (ADR-0020, ADR-0023). |
| 1.4.3 Contrast (AA) | PASS | All three themes (ADR-0017) record their ratios in the CSS comments at each token. Dark: `--text` on `--bg` 17.0:1; `--muted` history on `--bg` 6.6:1; `--on-accent` on `--accent` 10.0:1. Light: text 15.9:1, muted 6.3:1, accent-as-text 4.5:1 on the canvas / 4.8:1 on the panel, `--on-accent` 4.8:1. Night: text 12.1:1, muted 7.6:1, accent 7.4:1 / 7.1:1, `--on-accent` 7.2:1. TUI night text 12.1:1, hints 7.6:1, selection 7.2:1; TUI light result 5.9:1. |
| 1.4.4 Resize text 200% | PASS | Fixed viewport with internal scroll regions; `overflow-wrap: anywhere` on results. |
| 1.4.10 Reflow | PASS | Below 880px the panes stack as swipeable full-width panes (scroll-snap + pane-switch buttons); no horizontal scroll at 320px; desktop column + graph side by side from 880px. On touch, a 3D plot captures both swipe axes (touch-action: none) so a swipe rotates the surface instead of navigating away — the pane-switch buttons stay the non-gesture route back (ADR-0035); the 2D plot keeps pan-x, so the swipe-back gesture survives there. |
| 1.4.11 Non-text contrast | **FIXED** | Input boundary was 1.2:1 vs the page background (invisible field). Dark borders 3.5:1 vs `--bg`, 3.1:1 vs `--panel`; light borders 3.5:1; night borders 3.4:1 / 3.3:1. Focus indicators: see 2.4.7. Curves ≥3:1 in every theme — dark: accent 9.9:1, `#4da3ff` 7.0:1, `#ffb340` 10.3:1, `#c39dff` 8.4:1; light: accent 4.5:1, `#1e66c8` 5.2:1, `#9a5b00` 5.1:1, `#7a4bd6` 5.2:1; night: accent 7.4:1, `#ffb340` 11.6:1, `#ff9e8a` 10.3:1, `#e0483e` 5.1:1. |
| 1.4.12 Text spacing | PASS | No fixed line-heights that would clip. |
| 1.4.13 Content on hover | N/A | No hover-triggered content. |

### Operable

| Criterion | Status | Evidence / notes |
|---|---|---|
| 2.1.1 Keyboard | PASS | Native input + keypad buttons; Enter activates from the field; every keypad and menu button is reachable and activatable; the menu bar closes on Escape and every item is an ordinary tab stop (ADR-0017); the guide overlay closes with Escape and its Close button takes focus on open (ADR-0018); scrollable regions (result box, history box, graph pane, mobile menu panel, guide body) carry `tabindex="0"` so their content is keyboard-scrollable; the TUI keypad is always on screen and takes focus with Tab, switches its banks with Tab/Shift+Tab, moves with arrows, inserts with Enter, and returns focus with Escape (ADR-0019, ADR-0033); the TUI also takes the pointer (ADR-0034); on touch layouts, on-screen keypad presses never summon the device keyboard — they insert, compose, and submit without refocusing the entry, which only a tap on the entry itself focuses (ADR-0035): menus and popup items, keypad bank tabs and cells, and history lines all click, and the graph panel drags (2D pan / 3D orbit), wheels (zoom), and double-clicks (reset view); the TUI menus open with F10, move with arrows, activate with Enter, close with Escape; the TUI guide pager scrolls with Up/Down/PgUp/PgDn/Home/End and closes with Esc/q. |
| 2.1.2 No keyboard trap | PASS | Keypad buttons are ordinary tab stops; the TUI keypad focus closes with Esc (Tab cycles its banks, ADR-0019); both menu systems close with Escape; dropdowns hold no focus; the guide overlay closes with Escape or the Close button, which holds focus on open. |
| 2.4.1 Bypass blocks | N/A | Single screen; nothing to skip. |
| 2.4.2 Page titled | PASS | `<title>epher</title>`. |
| 2.4.3 Focus order | PASS | Document order: menu rail → input → answer → history → keypad → graph pane; on mobile the pane switch buttons follow the menus in the same top bar. |
| 2.4.4 Link purpose | N/A | No links. |
| 2.4.6 Headings & labels | **FIXED** | Input has `aria-label`; button's bare `=` name replaced with `aria-label="Evaluate"`. |
| 2.4.7 Focus visible | **FIXED** | Was: no styles (browser-default ring on a dark theme, inconsistent). Now: `:focus-visible` accent outline (9.9:1 vs `--bg`, 8.9:1 vs `--panel`); the accent button gets an inset dark-teal ring (10.0:1 on the accent surface — an outer ring would not contrast). |
| 2.4.11 Focus not obscured | PASS | No sticky/overlay content (AA; 2.4.12 AAA not targeted). |
| 2.5.8 Target size (AA) | PASS | Keypad buttons ≥44×44px in a 5-column grid; tab buttons ≥44px wide; menu bar buttons ≥44px and menu items ≥40px tall; guide example buttons are ≥44px tall and the Clear graph button ≥40px; the install-cli button is ≥48px tall (padding `0.5rem 1rem` on `0.95rem` text — ~48px). |

### Understandable

| Criterion | Status | Evidence / notes |
|---|---|---|
| 3.1.1 Language of page | PASS* | `lang`/`dir` track the resolved locale (detection via `navigator.languages`, `dir="rtl"` for Arabic); the guide pages set both per locale at build time. *The landing page's static `html lang="en"` is updated at runtime from the stored preference; its initial paint is English until app.js runs. |
| 3.2.1/3.2.2 On focus/input | PASS | Focus lands in the field on load (intentional: it is the whole app); submit only updates the result region. |
| 3.3.1 Error identification | **FIXED** | Errors already appear as text (announced); now also `aria-invalid="true"` + `aria-describedby="epher-result"` on the input while an error is showing. |
| 3.3.2 Labels | PASS | `aria-label` on the input; placeholder is a hint only. |
| 3.3.3 Error suggestion | PASS | Core error strings are descriptive ("division by zero", "unknown name …"). |
| 3.3.7 Redundant entry | N/A | Single step; history shows prior entries. |

### Robust

| Criterion | Status | Evidence / notes |
|---|---|---|
| 4.1.2 Name/role/value | PASS | Native elements only, no ARIA roles on divs. |
| 4.1.3 Status messages | PASS | Result is `role="status"` + `aria-live="polite"` — submit results and errors are announced without stealing focus. |

## TUI (terminal)

The terminal emulator provides font size, zoom, contrast themes, and
screen-reader output; the app must stay usable through them.

| Item | Status | Evidence / notes |
|---|---|---|
| Keyboard operability | PASS | All actions are keys (Enter evaluate, Esc clear, Ctrl+C / q quit); hints footer now shows them (`tui-hints`, localized). |
| Focus visible | **FIXED** | The terminal cursor now sits at the end of the input text every frame (was: wherever the shell left it). Width is unicode-aware. |
| Theme respect | PASS | No forced background colors; text colors are palette-based (`Color::Green` result, `DarkGray` hints), so user themes (incl. high-contrast) apply. |
| Screen-reader output | **FIXED** | The graph panel now carries a text caption (`y = <source>`) above the ASCII plot, so terminal screen readers announce what the plot shows. |
| Zoom/reflow | PASS | Layout is proportional (`Min(0)` history, fixed 20-row graph); 200% terminal zoom reflows. |

## CLI

Plain text in, plain text out — no ANSI colors, no interaction beyond stdin.
Theme-safe by construction; nothing to fix.

## Marketing site (epher.org: landing, About, Privacy, guide pages)

The 2026 redesign (teal accent replacing amber) keeps the same bar.
Contrast values are recorded in `site/styles.css` (single source of
truth); the design rationale and research live in
`docs/research/modern-ui-accessibility.md`.

| Criterion | Status | Evidence / notes |
|---|---|---|
| 1.4.3 Contrast (AA) | PASS | light: text 17.0:1, muted 6.4:1, links 5.5:1, primary button 5.5:1. dark: text 16.9:1, muted 6.6:1, links 12.5:1, primary button 11.4:1. |
| 1.4.11 Non-text contrast | PASS | Icons/rings ≥ 3.7:1 (light) / 11.0:1 (dark); interactive control borders and card edges 6.8:1 (light) / 3.5:1 (dark, 3.1:1 vs panel). |
| 1.4.1 Use of color | PASS | No color-only indicators; links are underlined. |
| 2.1.1 Keyboard | PASS | The disclosure (hamburger) nav is a native `button`; links are real anchors; no pointer-only interaction. |
| 2.4.1 Bypass blocks | PASS | Skip link on every page; single `main` landmark. |
| 2.4.7 Focus visible | PASS | 3px `--ring` outline, offset 2px; accent-filled controls use an inset ring in their text color. |
| 2.4.11 Focus not obscured | PASS | Sticky header is translucent; focus rings on header controls remain visible. |
| 2.5.8 Target size (AA) | PASS | All interactive targets ≥ 44×44 CSS px (nav links, buttons, icon buttons, select — 2.5.5 best practice, not just the 24px AA floor). |
| 2.3.3 Animation from interactions | PASS | Only 150 ms color transitions; all motion disabled under `prefers-reduced-motion`. Smooth scrolling is also disabled under the preference. |
| 4.1.2 Name, role, value | PASS | Menu button: native `button` + visually-hidden label + `aria-expanded` + `aria-controls`; nav is a labelled `nav` landmark. |
| 3.1.1/3.1.2 Language | PASS | `lang`/`dir` track the active locale (RTL for Arabic) on all pages; guide pages bake both in at build time. |
| 2.4.5/ARIA APG nav | PASS | Disclosure pattern per the WAI-ARIA Authoring Practices: `hidden` removes collapsed links from the tab order, Escape closes and restores focus, click-outside closes, link activation closes. A `<noscript>` style shows the links stacked when JavaScript is off. Desktop never hides the nav behind the button. |

## Known gaps (tracked elsewhere)

None at the moment: the UI is fully localized (ADR-0008) and the
automated axe scan runs in the `a11y` Playwright suite (ADR-0009).

## How to re-verify

1. **Contrast**: recompute ratios (formula above) after any color change; the
   3:1 boundary rule applies to borders and focus indicators.
2. **Keyboard**: Tab input → button; Enter evaluates; button activates via
   Enter and Space; no outline is removed without a `:focus-visible`
   replacement.
3. **Screen reader**: page reads h1, labeled field, result announcements;
   `aria-invalid` flips on errors.
4. **TUI**: cursor is visible in the input field; hints line shown; `graph`
   shows the caption; all keys work with a screen reader's pass-through.
5. **Automated**: the `a11y` Playwright suite (axe-core, the wcag2a/2aa/21a/
   21aa/22aa rule tags) scans every state — desktop dark/light/night, the
   graph pane with 2D and 3D plots, the guide overlay, and the mobile
   layout (baseline, hamburger open, post-auto-slide). Zero violations are
   required; axe's unresolved color-contrast targets are either computed in
   the suite (glyph buttons, >= 4.5:1) or covered by this audit (plot-SVG
   text) and the uncovered baseline scans.
