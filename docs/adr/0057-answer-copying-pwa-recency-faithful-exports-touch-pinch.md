# ADR-0057: Answer copying, the PWA's recent activity, faithful exports, and the touch pinch

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-09-04

## Context

Four frictions, one round. Copying an answer meant selecting text by
hand; the PWA opened blank on every launch (its bridge has no native
store, ADR-0010, and nothing stored the session); a copied or saved
plot wore the dark palette even in the light theme and carried no
legend; and on touch devices the 3D pinch-zoom depended on a pointer
stream some engines cancel the moment a second finger lands.

## Decision

- **Answer copying.** A single copy icon sits just left of the answer,
  in both places an answer appears: the answer line (ADR-0016) and the
  result pane (ADR-0056). It copies the values behind what is on
  screen - the displayed answers without the `= ` voice, one per line,
  never joined with semicolons. The icon answers a press with a check
  for a moment and is present only while an answer is on screen.
- **The PWA remembers.** In the browser (no native store), each
  submission stores three localStorage keys: the history lines, the
  session bindings (`ans` and every assignment), and the answer text on
  screen. Startup restores them before the first submit, so reopening
  the PWA shows the same history, the same working names, and the last
  answer. Clear history clears them. The desktop shell keeps its
  native-store path (ADR-0010) unchanged.
- **Exports match the pane.** Copy SVG and Save PNG now take the app
  theme's palette (the same `--curve-*`, `--text`, and `--muted` colors
  the pane paints, WCAG 1.4.11 ratios recorded in the CSS) and the
  pane's legend entries, drawn as a color-swatch band under the plot.
  Hidden legend entries stay out, as before; hidden solar bodies now
  stay out through the same filter the live render uses. Width and
  zoom already traveled; the transparent background stays.
- **Pinch survives cancelled pointers.** The 3D scene gains a touch
  -event fallback: while two fingers move, the same distance ratio
  drives the camera and the zoom slider follows. The touch path and
  the pointer path mutually exclude through a flag, so engines
  delivering both never double-zoom, and the page never zooms in the
  gesture's place (ADR-0035's touch contract).

The 3D line-width slider also met `svg .curve`'s CSS default, which
overrode the mesh's stroke-width attribute and pinned every surface
line at 1px while the frame answered the slider. The 3D svg now sets
`--curve-width` to the mesh's px width, so slider, attribute, and CSS
agree.

## Consequences

- Export documents grow a legend band (canvas taller than 400 only
  when a legend exists); the PNG raster follows the document.
- localStorage holds at most the capped history, one bindings map,
  and one answer string; the site's `epher-example` handoff and the
  theme/language/display keys are untouched.
- The TUI's plot pane now carries the web app's localized pane name
  ("Result"), one word for the same pane everywhere.

## Amendment (2026-09-05): the pane copy carries the alignment

The result pane's copy wrote plain text only. For a `table` answer that
plain text is exactly the space-aligned form on screen — perfect in a
terminal, an editor, or a code block, and ragged the moment it landed
in a proportional-font app, because spaces carry no alignment there.

The copy now writes both clipboard flavors in one `ClipboardItem`:
`text/plain` unchanged (the aligned monospace text), and `text/html`
that carries the structure a rich-text paste consumes — a table answer
pastes as a real `<table>`, right-aligned cells, so values land in
columns in a document or a spreadsheet; any other multi-line answer
pastes as preformatted text; single answers as plain lines. Browsers
that refuse the two-flavor write fall back to plain text, so every
paste target keeps working. A table answer is recognized structurally
(rows of single-token cells split by runs of two or more spaces —
exactly `format_table`'s emission), not by a flag: a transcript's
single-spaced `= 5` voice is never mistaken for a table.
