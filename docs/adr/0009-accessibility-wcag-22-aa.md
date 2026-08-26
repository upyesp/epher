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
- **Verification.** Automated axe/Lighthouse checks are planned once a
  headless-browser test harness exists in this environment; until then the
  audit in `docs/accessibility.md` is updated by hand, and keyboard +
  screen-reader walkthroughs are part of the manual checklist.
