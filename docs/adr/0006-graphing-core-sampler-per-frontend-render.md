# Graphing splits into a core Sampler and per-frontend renderers

- Status: accepted
- Date: 2026-08-13

## Amendment (2026-08-17)

The v1 plan this record sketched proved partly too conservative for the
pre-v1 releases. ADR-0014 shipped the parametric and polar forms while
still pre-v1, and ADR-0015 shipped 3D (the projection design made it
reachable for both renderers), superseding the "3D is deferred" judgment
here. The Sampler seam itself stands: everything after 0014/0015 still
computes in core and only renders per frontend.

Graphing is divided into compute and render. `epher-core` owns a `Sampler` that
turns an Expression and a domain into plottable data (sampling, domain and
discontinuity handling); each frontend renders that data its own way — vector
for the GUI and PWA, ASCII/blocks for the TUI, none for the CLI.

This seam protects "logic exists once": the sampling math is shared across every
frontend, and only the pixels differ. v1 graphs 2D (`y=f(x)`, parametric,
polar); 3D is deferred.
