# ADR-0083: the landing hero is a real capture of all five surfaces

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-11-10

## Context

Since ADR-0030 the landing page's visual has been the 3D saddle
`graph3d x ^ 2 - y ^ 2` rotating slowly, drawn live by `site/app.js` as
a port of the app's 3D renderer. The port was written before any of the
five surfaces had a good photograph, and it shows the engine, not the
product: a visitor sees a wireframe, not a calculator. Keeping the
render faithful to `crates/core/src/graph.rs` is a standing maintenance
tax on two files that must drift in lockstep, and the animated hero is
also the landing page's main per-frame script work.

The project now has a real composite capture: all five surfaces (CLI,
REPL, TUI, desktop app, mobile web app) photographed on one machine,
each showing the same equation `solve x^2 == 5*x + 6` with its answer,
and the desktop app graphing that same quadratic so its roots (x = -1
and x = 6) visibly match every history in the frame.

## Decision

The landing hero's animated SVG is replaced by that capture:
`site/img/hero.webp` (1600x900, 41 KB) with `site/img/hero.png`
(262 KB) as the fallback of a `<picture>` element, alt text
"epher screen shot", intrinsic `width`/`height` so the layout does not
shift while it loads. The image is dark in both themes, the same
contract the terminal card already has, so it keeps the terminal
border and shadow rather than theme-flipping panel tokens.

The `initHero3d` port and its CSS are deleted. The terminal card above
the hero loses the `graph3d x ^ 2 - y ^ 2` example and keeps three:
`2 + 3 * 4`, `solve x^2 == 5*x + 6`, and the Planck script. The
`docs/website.md` hero section describes the capture instead of the
animation. ADR-0030's hero decision is superseded on this point; its
record stands.

## Consequences

- **`site/app.js` loses its largest function** (245 lines): no hero
  render loop, no `requestAnimationFrame` work on the landing page, and
  no second renderer to keep in sync with the engine's projection.
- **The reduced-motion exception disappears with the animation**: a
  static image needs no `prefers-reduced-motion` branch, and the page's
  motion contract is trivially met.
- **The hero no longer demonstrates 3D**. The features section's
  "Graphs in 2D and 3D" card and `examples.html` still carry 3D; the
  saddle remains one keystroke away via the IDE and the examples page.
- **The capture is a build artifact of the machine, not the tree**:
  refreshing it means re-shooting the five surfaces and regenerating
  `site/img/hero.webp` and `site/img/hero.png` (bump the `?v=` query
  when the pixels change). The history it shows is staged in isolated
  store directories, never a user's real data.
- **Landing weight drops on the critical path**: the webp is 41 KB
  against hundreds of KB of per-frame canvas churn; the PNG fallback
  only serves engines without WebP.
