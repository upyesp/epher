# ADR-0020: SVG export from every frontend and the graph pane's options row

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

Two asymmetries after v0.4.8:

1. Only the desktop and PWA could carry a plot out of the app (the
   **Copy SVG** button). CLI, REPL, and TUI users could draw — the TUI
   even plots 3D wireframes — but not save the picture.
2. The copy button's output was class-based SVG that depends on the
   page's CSS: pasted into a document it rendered as unstyled black
   lines. Meanwhile the plotted-line thickness was fixed at 2 viewBox
   units with no way to thin it, and the graph display settings added
   in ADR-0019 lived in the Settings menu — far from the plot they
   change.

The pure renderer (`geometry`, `segments`, `ticks`, the SVG string
builder) lived in the web crate even though none of it touches the DOM.

## Decision

1. **One renderer, in core.** The pure plot-to-SVG code moves to
   `epher_core::graph_svg`; the web crate re-exports it and keeps only
   the Yew (live) renderers. `graph_svg` gains a `stroke_width`
   parameter and emits a **self-contained document**: embedded
   `<style>` with the default dark palette, 640×400 viewBox and size,
   and no background (transparent since ADR-0055). The copy button and
   every terminal save produce the same bytes. 3D export (`graph3d_svg`) letterboxes the mesh into the
   same canvas with a transform — the identical math
   `preserveAspectRatio` performs in the live renderer.
2. **`graph save <file>` and `graph3d save <file>`.** The `graph`
   grammar grows a `save` form beside `clear`. The REPL, piped scripts,
   and one-shot strings share a `Plots` state (epher-shell) across
   their lines; the TUI reuses the same save helpers from its pane
   state, including the current 3D orbit pose. Files write as UTF-8
   SVG; nothing else about the graph commands changes.
3. **The pane owns its options.** The ADR-0019 Settings-menu group
   moves onto the graph pane as an options row at its bottom: two
   labelled checkboxes (the POI list, the on-plot markers) and a
   **line-width slider** (0.5–4, default 1 — half the old constant).
   Real form controls, not menu items: they are adjustments with
   immediate visual effect, so they belong beside the plot. The slider
   sets `--curve-width` on the SVG element; the CSS default equals the
   new `DEFAULT_STROKE_WIDTH`. Persistence stays localStorage-based
   (`epher-line-width`), now restored on desktop too (the pre-0020
   desktop build silently dropped the POI settings on restart).
4. The exported document always uses the default dark palette — the
   file is deterministic regardless of the app theme at save time. Its
   background is transparent (ADR-0055).

## Consequences

- A plot saved from the TUI, the REPL, a piped script, the desktop, and
  the PWA is the same picture: 640×400, dark, self-contained, ready for
  documents.
- The web crate's public `graph` module surface is unchanged except for
  the new `stroke_width` argument on `graph_svg`.
- The Settings menu shrinks to Theme + Language (11 items); the mobile
  panel to five groups. Guide, accessibility notes, and suites updated
  in step.
- Terminal saves always include POI markers in the TUI when its POI
  setting is on; CLI/REPL saves include them (there is no setting
  there). The list itself is recomputed at save time — analysis is
  cheap and always-on (ADR-0019).


## Amendment (2026-08-31): the width slider joins the tuning strip (ADR-0041)

The line-width slider leaves the toolbar (it sat right of Copy SVG,
wrapping below on phones) for the tuning strip directly above the
plot. It is named by an icon and a tooltip ("line thickness") instead
of a text label, the numeric readout is gone, and the ranges are
unchanged - 0–4 step 0.1 for 2D, 0–0.2 step 0.01 for 3D, only the
kind in view shown, each kind remembering its own value.

## Amendment (2026-08-31): Save PNG beside Copy SVG (ADR-0042)

The pane toolbar gains a Save PNG icon button next to Copy SVG. It
serializes the same self-contained SVG document (hidden curves still
excluded), rasterizes it on a canvas at twice its size, and saves
through the platform's flow: the native save dialog over a new
`save_png_dialog` IPC command on the desktop, the browser's File System
Access picker in the PWA, a plain download as fallback. The CLI keeps
SVG as its only export - `graph save plot.svg` is unchanged.

## Amendment (2026-09-02): transparent exports (ADR-0055)

Exported SVG documents no longer paint a background: the background rect
and its `.bg` style are removed from every document builder (2D curves,
data plots, 3D surfaces, space curves, solar scenes). The embedded
palette is unchanged; the picture simply sits on the reader's
background. Save PNG rasterizes the same document on a clear canvas, so
it exports transparent too. The earlier claims above of an opaque dark
document describe the pre-0055 behavior.
