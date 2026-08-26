# Accessibility: WCAG 2.2 AA for the web/PWA, keyboard-first terminals elsewhere

- Status: accepted
- Date: 2026-08-13

Accessibility is an architecture constraint, not a styling afterthought. The
web/PWA — and therefore the Tauri shell, which wraps the same page — targets
**WCAG 2.2 Level AA**, verified per criterion in `docs/accessibility.md`. The
TUI and CLI target the criteria that a text application controls: full
keyboard operability, a visible focus/cursor, palette-based colors that respect
terminal themes (including high-contrast ones), and text alternatives inside
the app (e.g. a graph caption) since the terminal emulator owns font size,
zoom, and screen-reader output.

Consequences for the architecture:

- **Native controls first.** The web frontend uses a real `form`/`input`/
  `button`, `ul` history, and a single `h1` — no ARIA roles on divs, no custom
  widgets. Keyboard activation, focus order, and names come for free.
- **Contrast is a review gate.** Text 4.5:1, non-text (field boundaries, focus
  indicators) 3:1, computed from the WCAG relative-luminance formula; the
  values for the current theme are recorded in `docs/accessibility.md` and
  must be recomputed whenever colors change.
- **Status messages via live regions.** The result area is
  `role="status"`/`aria-live="polite"`; errors additionally set
  `aria-invalid` + `aria-describedby` on the input. Errors never rely on color
  alone.
- **i18n and a11y move together.** The `lang` attribute must track the
  Localizer-resolved locale (ADR-0008) and `dir="rtl"` when Arabic is active;
  labels and placeholder text move to Fluent catalogs at the same time as
  locale detection lands in the web app.
- **Verification.** The axe-core scan in the Playwright suite is the
  automated leg: the `a11y` suite scans the wcag2a/2aa/21a/21aa/22aa
  rule tags across the app's states — desktop dark/light/night, the
  graph pane with 2D and 3D plots drawn, the guide overlay open, and
  the mobile layout (baseline, hamburger open, and after the
  auto-slide). The policy is zero violations; axe's only allowed
  "incomplete" results are color-contrast on targets it cannot resolve
  (symbol-glyph buttons, whose contrast the suite computes itself from
  the WCAG relative-luminance formula and asserts >= 4.5:1; plot-SVG
  text, which the palette audit below covers; and elements behind an
  open overlay, which the uncovered baseline scans cover). The hand
  audit in `docs/accessibility.md` — contrast math per theme,
  keyboard and screen-reader walkthroughs — remains the human leg
  that catches what static rules cannot.
