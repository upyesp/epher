# ADR-0055: keypad banks for the new language, transparent exports, and the graph-controls review

Date: 2026-09-02 · Status: accepted

## Context

The v0.5.18 milestone (ADR-0053/0054) added a large amount of language
surface, data types, and plot kinds. A desktop review round found the
following gaps and asked for fixes:

1. Many functions added since ADR-0043 were not reachable from the
   on-screen keypad, and the new data types (matrices, lists, strings)
   could not be typed from it at all.
2. The mobile (hamburger) menu was missing items the desktop menus
   carried.
3. SVG and PNG exports painted an opaque dark background.
4. Small UI/UX items: the guide menu label, the answer-area layout for
   multi-answer scripts, the history trash position, the animation
   loop's slider window, the 3D line-width default and range, zoom
   slider consistency across plot kinds, slider icons as reset
   buttons, grabbing an animated slider, and 3D plots missing the 2D
   legend/controls pattern.

## Decision

### The keypad covers the language it grew (ADRs 0016/0039 amended)

The `123` tab is untouched (its test holds). The other banks absorb the
new surface, and two new banks join the tab row (all of them fit the
fixed five-row grid, so nothing scrolls except astronomy):

- **`nΣ`** (number and statistics) gains the seeded-random keys
  `randint`, `random`, `randseed`, `randn`.
- **`ƒ`** (functions) gains complex parts and calculus: `re`, `im`,
  `arg`, `conj`, `derivative`, `integral`.
- **`data`** (new): the data-type keys `[` `]` `{` `}` `"` plus the
  matrix functions `det`, `inv`, `transpose`, `rref`, `dim` and the
  value functions `str`, `len`. This is how touch screens type
  `[[1, 2], [3, 4]]`, `{1, 2, 3}`, and `"text"`.
- **`dist`** (new): the regression family (`linreg`, `quadreg`,
  `expreg`, `powreg`, `logreg`), `anova`, `ttestpaired`, and the
  distribution family (`norm*`, `t*`, `chi2*`, `inv*`, `binom*`,
  `poisson*`, `ztest`, `zinterval`, `tinterval`, `ttest`).
- **`$`** (new): finance (`tvm_*`, `npv`, `irr`, `amort`,
  `compound_interest`, `simple_interest`).

Every new key carries a hint (key-hint-* in all eight locales, the
ADR-0039 accessibility surface). The tab-fit and hint-resolution tests
cover the new banks automatically.

### Mobile menu parity (ADR-0017 amended)

The hamburger panel gains the items desktop menus had and mobile
lacked: **Help → Constants** and the whole **Results** group (exact
fractions, notation, thousands separators). Mobile now carries every
menu action the desktop rail does.

### Transparent exports (ADRs 0020/0042 amended)

The exported SVG documents no longer paint a background: the
`<rect class="bg">` and its `.bg` style are gone from every document
builder (2D, data plots, 3D surfaces, space curves, solar). The colors
and embedded palette are unchanged. The PNG path rasterizes the same
document on a clear canvas, so it is transparent too. Every frontend
shares the change through `epher_core::graph_svg` (web copy/save,
desktop save dialogs, TUI and CLI/REPL `graph save`).

### The small items

- The in-app menu label for the built-in guide is now **In-app user
  guide** in all eight locales (it distinguishes the built-in copy from
  the website guide).
- The web/desktop answer area lays a script's answers on one line,
  separated by semicolons, when they fit; an answer is never split
  across lines, a long one moves whole to the next line, and
  multi-line outputs (tables, matrices) keep their own blocks.
- The history trash icon moves to the left of the **History** heading.
- The animation loop keeps the slider window fixed at the play span
  pressed at start, and `slider_span` changes window only outside the
  base ±10 (an earlier ±8 cutoff let the window chase the value, so
  the thumb stalled mid-track instead of wrapping at the right end).
  Grabbing the slider indicator stops playback on pointer-down, so the
  drag moves the value.
- The 3D line-width slider becomes 0.0-0.4 in steps of 0.05 with 0.2
  the default (ADR-0015 amended). 3D line widths are screen px:
  `vector-effect="non-scaling-stroke"` at 10x the slider value, so a
  line is 2 px at the default on any display and in the letterboxed
  exports; the old world-unit strokes scaled with the pane and rendered
  several times thicker than the 2D lines beside them.
- Wheel and pinch zoom move the zoom slider on every plot kind
  (ADR-0038 amended): the 3D zoom state IS the zoom slider's value (it
  may pass the slider ends, which pin like the 2D slider does), and
  data plots (scatter, histogram, boxplot) zoom by an x-window clip
  with the same wheel, pinch, slider, and reset behavior as curves;
  their exports save what the pane shows.
- Pressing the icon beside a tuning-strip slider resets that slider to
  its default (the icons are real buttons, keyboard-accessible).
- 3D surfaces and space curves get the 2D legend pattern (ADR-0015
  amended): one checkbox per scene element, each element wearing its
  own palette colour (scene parts carry the curve-N classes), hidden
  elements out of the plot and the export with their palette index
  kept. The space-curve pane also gains the graph toolbar it was
  missing (Clear/Copy SVG/Save PNG).

## Consequences

- The keypad tab row grows from seven to ten banks; the tab bar wraps
  on narrow panes, and every bank except astronomy fits five rows.
- Exported plots sit on any background; dark-theme contrast notes
  still describe the palette the colors were chosen for.
- 3D default lines are visibly thinner; the slider can go to 0.4.
- The zoom slider is a truthful readout on every plot kind.
- The guide documents the new keypad banks, the transparent exports,
  the same-line answers, the zoom behavior of data plots, the reset
  icons, and the slider-grab animation control (site/guide in all
  eight locales, fences byte-identical).
