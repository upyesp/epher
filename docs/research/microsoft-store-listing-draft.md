# Microsoft Store listing: draft text for editing

Draft of 2026-09-18, for the user to edit before submission. Field names follow
Partner Center's Windows app submission form. All limits and dimensions below
are verified from primary sources in
docs/research/microsoft-store-publishing.md; wording is epher's site voice
(lowercase brand, no exclamation marks, plain declaratives).

## Name (reserved app name)

    epher

The title must be the bare app name: Store policy 10.1.1 — "Your product
title or name must be unique and must not contain marketing or descriptive
text, including extraneous use of keywords" — and every top listing does
exactly this ("WhatsApp", "Windows Calculator"). The descriptor lives in
the keywords and the description's first line instead.

## Description (limit 10,000 characters)

Two versions. The tight one matches what the most-popular listings
actually do (140-784 characters, pitch in line one); the extended one
spends more of the budget so Store search indexes the feature breadth.
Pick one — or tight + a trimmed extended.

### Tight (recommended; ~830 characters)

---
epher is a scriptable, graphing calculator for Windows. Compute, graph,
and script in one fast, native app - fully offline, with no accounts and
no telemetry.

Calculate with exact arithmetic: fractions stay fractions, radicals stay
radicals. Complex numbers, matrices, units, calculus, statistics,
finance, and symbolic algebra (CAS).

Graph 2D curves with automatic points of interest, implicit relations
like x^2 + y^2 = 1, parameter sliders and animation, data plots, and 3D
surfaces.

Then script it: save any calculation as a reusable script, start from
hundreds of ready-to-run scripts, or drive everything from the built-in
`epher` command line.

Free and open source. Eight interface languages. Works fully offline.
---

### Extended (~2,300 characters; the same text, grouped per WhatsApp's
### bolded mini-header style, for when full feature indexing is wanted)

---
epher is a scriptable, graphing calculator for Windows. Compute, graph,
and script in one fast, native app - fully offline, with no accounts and
no telemetry.

CALCULATE
- Exact arithmetic: fractions stay fractions, radicals stay radicals
- Complex numbers, matrices, lists as first-class values
- Units and conversion, physical constants, percentage operator
- Calculus (numerical derivatives and integrals), numeric equation
  solving
- Statistics: regression and curve fitting, probability distributions,
  hypothesis tests, confidence intervals
- Finance: TVM, NPV, IRR; number theory and bitwise operations
- CAS: symbolic algebra

GRAPH
- 2D graphs with automatic points of interest: roots, extrema,
  intersections
- Implicit relations: x^2 + y^2 = 1 just works
- Parameter sliders and animation - see how a graph responds as you
  drag
- Data plots: histogram, box plot, scatter, with a table of values
- 3D surfaces and 3D curves
- Inequality graphing

SCRIPT
- A complete scripting language: variables, loops, functions - save
  calculations as reusable scripts
- Hundreds of ready-to-run scripts across astronomy, finance,
  statistics, and every field of mathematics
- The `epher` command line comes built in: one-shot calculations, piped
  scripts, REPL, and SVG export, right from any terminal

AND
- Works fully offline. No accounts, no analytics, no telemetry, no
  cookies
- Eight interface languages: English, Spanish, French, German,
  Portuguese, Chinese, Hindi, Arabic
- Exam mode for classrooms
- Free and open source (GPL) - source at github.com/upyesp/epher

epher keeps everything in a single folder on your machine and nothing on
anyone else's. Privacy policy: https://epher.org/privacy.html
---

## Short description (limit 1,000; the listing shows ~270)

    Scriptable graphing calculator: exact math, 2D/3D graphs, scripting,
    fully offline. No accounts, no telemetry.

## What's new (limit 1,500)

    First release of epher on the Microsoft Store: the graphing
    calculator, the scripting engine, and the epher command line in one
    package, with the same version number as the GitHub release.

## Keywords (max 7 terms)

    graphing calculator
    scientific calculator
    math
    plotting
    cas
    units converter
    statistics

## Copyright line

    Copyright (c) upyesp.org

## Support / contact

    Website: https://epher.org
    Support: https://github.com/upyesp/epher/issues

## Category

    Utilities & tools

(the category whose documented example is literally "calculators"; this
is also where Windows Calculator sits per displaycatalog).

## Age rating questionnaire

Nothing controversial: no user-generated content, no online interaction,
no ads, no purchases inside the app, no health/biometric/gambling
content. The questionnaire resolves to the lowest tier (Everyone / 3+ /
PEGI 3). The `epher scripts` folder is static content shipped inside the
app, not user-generated content from other users.

## Privacy policy URL (required field)

    https://epher.org/privacy.html

Already live and Store-adequate: the page states no accounts, no
analytics, no telemetry, no cookies, and names the single `.epher` data
folder.

## Website (optional field)

    https://epher.org

## Screenshots: the shot list

Hard rules, verified from the Store documentation:

- PNG, 50 MB max each
- 1366 x 768 minimum; up to 3840 x 2160 supported - capture at your
  display's native resolution, the bigger the better (Windows
  Calculator's seven screenshots are all 3840 x 2160)
- 1 screenshot is the minimum, 10 the maximum; Microsoft recommends
  5-8. This list produces 8; use at least the first 4.
- One caption per screenshot, 200 characters max
- Keep the interesting content in the top two-thirds of the frame:
  Microsoft overlays text on the bottom third in merchandising
- No baked-in marketing text, logos, or device frames on the captures
  themselves
- Crop to the app window: no desktop background, no taskbar
- Consistent theme across the set (pick light or dark; keep every
  capture in it)
- Focus assist on (no notification toasts), and no personal data in any
  terminal prompt or path (`C:\Users\demo>`)

The captures, in the order they should appear in the listing:

1. **The hero graph.** One 2D graph that is beautiful at a glance - a
   damped oscillation or a rose family with a legend and the automatic
   points of interest marked (roots/extrema dots visible). The
   single most-seen image; it must read as "graphing calculator" at
   thumbnail size, so thick curves, high contrast, minimal chrome.
2. **Sliders and animation.** The same graph family with the parameter
   slider panel visible, one slider mid-drag, showing the curve
   updating. This is the feature that separates epher from static
   plotters.
3. **3D surface.** A rotated surface (saddle or sinc) in the 3D view.
4. **Exact arithmetic.** The console/REPL in Windows Terminal: a short
   session showing fractions staying exact (1/3 + 1/6 = 1/2), a complex
   number, and a matrix determinant - three lines each, large font
   (Windows Terminal at 16-18pt so text survives thumbnail size).
5. **Scripting.** A saved script with its output - a small regression
   or TVM finance problem with its result table.
6. **Data plots.** A histogram + scatter from an inline list, with the
   table of values visible.
7. **TUI.** `epher tui` inside Windows Terminal with a plot drawn
   inline - proves the same engine drives the terminal.
8. **Units and CAS.** One view: a units conversion and a symbolic
   integral (CAS) with the result.

Every capture should be a real session someone could reproduce from the
scripts repository - no mocked data.

## Store tile and icons (not screenshots, but prepared alongside)

- **Store tile icon, 1:1, 300 x 300**: strongly recommended, this is
  what merchandising and search results use. One 1024x1024 master,
  downscaling cleanly.
- App icon ladder (44x44 through 256x256) comes from the same master
  for the package manifest.
- Optional art: poster 720x1080 (or 1440x2160), box art 1080x1080 (or
  2160x2160), "super hero" 1920x1080 (needed only if a video trailer is
  attached). None required; skip for v1.
- Draft tile composition: the epher icon on the brand background, no
  text (logos with text shrink badly at 300 px).
