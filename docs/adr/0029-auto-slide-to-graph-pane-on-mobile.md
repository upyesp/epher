# ADR-0029: Auto-slide to the graph pane after drawing on mobile (v0.4.17)

- **Status:** superseded by ADR-0035 — the mobile PWA usability
  contract absorbed the auto-slide into one record with the rest of the
  mobile graph-pane behavior (keypad focus discipline, the slide back
  after a clear, 3D swipe rotation, per-kind widths).
- **Deciders:** epher maintainers
- **Date:** 2026-08-24

## Context

On the mobile layout (<880px) the calculator and graph panes form a
swipeable horizontal strip (ADR-0016): the graph is one slide away,
with the pane-switch buttons as the non-swipe spelling. A user who
enters `graph x ^ 2` on the calculator pane submits the command and
then sees… the calculator still — the curve exists but is off-screen.
The ask: after a graph command that actually draws something, slide
the view across to the graph pane automatically.

## Analysis

The strip scrolls via the shared `scroll_pane` callback, which jumps
discretely (the reduced-motion behavior, WCAG 2.3.3) and feeds the
`active_pane` state through the scroll event, so the pane-switch
buttons stay in sync for free. The submit path already walks each
statement of the submitted entry; the successful arms of the `graph`
and `graph3d` branches are the exact places that know a plot was
*drawn* — errors, `graph clear`, `graph save`, and plain evaluations
must not move the view. The callback was defined after the submit
closure, so it moves above it (it depends on nothing stateful).

## Decision

- On the successful draw of a 2D or 3D plot, the submit path emits
  `scroll_pane("graph-pane")` — but only when `mobile_layout()`: a
  window narrower than 880px, mirroring the CSS breakpoint exactly.
- The desktop layout (panes side by side, no horizontal strip) never
  scrolls.
- The slide is the existing discrete jump: instant, reduced-motion by
  construction, and the pane-switch's `aria-pressed` state follows
  automatically via the scroll event.

## Consequences

- `mobile-scroll` suite pins the contract: mobile graph/graph3d
  submits slide (scrollLeft reaches the graph pane and the Graph
  switch reflects it), `graph clear` and failed graphs do not slide,
  and desktop submits never scroll.
- This ADR originally ended "focus stays in the entry field; the
  slide is view-only convenience." ADR-0030 reversed that: the slide
  blurs the entry on mobile so the keyboard closes over the fresh
  plot — ADR-0035 carries the current rule.
