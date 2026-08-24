# ADR-0030: Five frontends everywhere, mobile blur after the auto-slide, scripts as one history entry, and a rotating 3D hero (v0.4.18)

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-08

## Context

Four related usability asks:

1. On the mobile PWA, the v0.4.17 auto-slide to the graph pane (ADR-0029)
   left the expression entry focused — the virtual keyboard stays open and
   covers the freshly drawn plot. The pane should be ready for touch
   rotation instead.
2. A script entered as one line with `;` separators was recorded in the
   history as one entry *per statement* — the script the user typed
   disappeared as a script.
3. The website called epher "four ways" / "four frontends" everywhere and
   folded the REPL into the CLI row of the guide, while the product has
   five frontends (CLI, REPL, TUI, desktop GUI, web app) — the same five
   named by ADR-0011.
4. The website hero showed a static 2D curve; the project's signature
   visual is the 3D saddle `graph3d x ^ 2 - y ^ 2`.

## Analysis

- **Blur:** the auto-slide already had the entry's node ref in scope; a
  `blur()` on the same condition as the slide (mobile layout, successful
  draw) closes the mobile keyboard. Touch rotation needs no focus — the
  surface's pointer handlers listen on the SVG itself.
- **History:** the web submit loop and the TUI `submit_line` split on `;`
  and recorded per piece. `Session` gained `set_last_line`; a
  multi-statement line now records exactly one entry — the line as typed,
  with the last answer appended exactly as single statements record theirs
  (when the final statement is an evaluation) — and `save script` persists
  the whole line.
- **Five ways:** the site i18n (all eight locales), the guide (all eight
  locales), the man page DESCRIPTION, and the CLI `--help` long text now
  name five ways; the guide table lists the REPL right after the CLI row.
- **Hero 3D:** a static SVG cannot be rotated honestly; frame precomputation
  (the build-time renderer emits ~10KB per frame) would bloat the site. The
  site's `app.js` instead ports the app's 3D projection (yaw/pitch/camera
  30, perspective divide, painter's-order mesh with depth-cued opacity,
  frame from `surface_frame`) — a small runtime renderer that updates one
  constant set of SVG elements per frame (no node churn). Line thickness is
  the width slider at 0.1 (mesh 1.2×, frame 1.4×). The view box is
  constant-size and centered per frame on the content, so the mesh rotates
  without pumping; reduced motion renders one static frame (the app's
  default pose).

## Decision

- Mobile auto-slide blurs the entry (mobile layout only), leaving the graph
  pane touch-ready.
- Multi-statement lines are one history entry in the web app and the TUI;
  `save script` saves the whole script line.
- "Five ways/frontends" is the wording everywhere the modes are counted:
  website, all eight guide locales, man page, `--help`.
- The website hero is a looping 3D saddle rotation, faithful to the app's
  renderer, at width 0.1.

## Consequences

- The `adr30` browser suite pins: mobile graph/graph3d submit slides *and*
  blurs; script history is one entry with semicolons (and re-runs from a
  history pick); the site hero renders 50 mesh + 7 frame elements, stays
  inside its view box, rotates, and the lede/f1-title say five.
- The desktop GUI inherits the web app's behavior through the same Yew
  frontend.
