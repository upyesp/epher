# ADR-0036: The website Examples page — copyable code for every frontend

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

The website teaches epher through the eight-language user guide, but a
reader who just wants something to paste has to dig through prose. The
user asked for a new top-level site section, **Examples**: one page of
copyable epher code — commands, scripts, and graphs — grouped by
frontend, with a one-sentence explanation per example and the guide's
copy-button treatment on every code block.

## Decision

- **One page, one place.** A new top-level navigation item **Examples**
  opens `site/examples.html`; the link joins the header and footer navs
  of the landing, about, privacy, and guide pages in all eight locales.
- **Content.** Three sections, each with a one-to-two-sentence
  introduction:
  - **The command line** — ten examples: plain calculations, piping a
    script into the `epher` command (stdin as script, `epher -`), piping
    epher's output into another command, `;`-joined statements,
    `def`-and-call, a recursive function inside a piped script, and 2D
    and 3D graphs saved as SVG image files (`graph save …` /
    `graph3d save …`).
  - **The REPL** — a session built on the `ans` keyword, one block per
    step; the introduction notes the blocks run in sequence in one
    session.
  - **TUI, desktop app, and web app** — one section for the three
    frontends that share the entry field: a multi-line script, a 2D
    curve, a shaded 2D curve (`y <` fill), an animated 2D curve (`const`
    + the play button), a two-curve multi-line plot, and two 3D
    surfaces.
  Every example has exactly one explanatory sentence; the page defers to
  the user guide for detail. Every code block carries a copy button in
  the guide's style (`.example` / `.copy-btn` from `guide.css`).
- **Localization split, like everywhere else.** The epher code blocks
  are never localized (ADR-0007) and ship byte-identical in every
  language. The captions, section prose, and the nav label are
  `ex-*` / `nav-examples` keys in the eight site catalogs, so the page
  follows the visitor's language like about and privacy do. The copy
  button's labels come from the active catalog.
- **Generated, not hand-assembled.** `scripts/build-examples.mjs` builds
  the page from an example list (caption key + code + fence kind),
  reusing the guide builder's highlighter and escape helpers, and
  borrows about.html's chrome (header, disclosure nav, theme, catalogs,
  app.js) by template replacement. CI runs it after `build:guide`
  (`npm run build:examples`).

## Consequences

- `build-guide.mjs` exports its highlighter/escape helpers and guards
  its build behind direct execution so the examples builder can import
  it; its `CHROME` map gains the `examples` label for the guide pages'
  navs.
- The browser suites pin the page: three sections, ten/ four/ seven
  examples, copy buttons that copy the plain code text, and the nav
  item present on every page in every locale.

## Amendment (2026-08-27): the app section comes first, four CLI examples moved into it, and a tap on a phone stages an example in the app

The command-line section led the page, but the entry-field frontends are
the ones most readers use. The **TUI, desktop app, and web app** section
now comes first, the REPL second, and the command line last. Four plain
calculations that were CLI examples (`epher "2 + 3 * 4"` and friends)
moved into the app section as their first examples, reformatted for the
entry field (`2 + 3 * 4`): a straightforward calculation, powers and
roots, exact fractions, and defining a function and calling it on the
same line (keys `ex-a8`–`ex-a11`, captions carried over; the old
`ex-1c`/`ex-2c`/`ex-3c`/`ex-7c` keys are gone from the catalogs).

On touch devices a tap anywhere on an example (outside its copy button)
copies the code and opens the app with it **staged in the entry field,
ready to run** (ADR-0035 amendment): the page stores the code under the
`epher-example` localStorage key and navigates to `/pwa/`, and the app
consumes the key at startup into its entry with the cursor at the end —
on mobile without summoning the device keyboard, the same rule as guide
code loads. The copy button still only copies (it stops propagation).
The page's note (catalog key `ex-tap`) explains the gesture; desktop
users keep the plain copy buttons.

## Amendment (2026-08-28): the REPL section closes the page, and the app section shows arithmetic across number bases

The previous amendment put the command line last, but the REPL's
session-style blocks (`epher>` prompt, answers feeding `ans`) read best
at the very end of the page. The section order is now **TUI, desktop
app, and web app**, then **the command line**, then **the REPL** last.
The catalog keys are unchanged — order lives in
`scripts/build-examples.mjs` only.

The app section gained one more example near the top, after the four
moved calculations (key `ex-a12`): **arithmetic across number bases** —
`0xff + 0b1` mixes hex and binary in one expression, and `hex(ans)`
spells the answer as hex (`0x100`). It shows the `0x`/`0b` prefixes and
the `hex` conversion function (ADR-0022) in one two-line script, and
fits the section's plain entry-field format.
