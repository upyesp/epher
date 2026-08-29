# ADR-0015: Animation and 3D graphing

- Status: accepted
- Date: 2026-08-17
- Supersedes the "not reachable short-term" judgment on 3D in ADR-0014; the
  projection design makes it reachable for both renderers.

## Context

The competitive survey (docs/research/graphing-features.md) and its focused
follow-up (docs/research/animation-and-3d.md, 34 first-party citations) show
that every calculator that animates does it one way: **parameter-driven
playback** — a numeric parameter steps through a bounded interval and
everything referencing it redraws (Desmos slider Play, GeoGebra Animation,
Wolfram `{u, umin, umax, du}`, TI-Nspire slider Animate). None of them animate
a hidden clock. For 3D, the universal entry point is the **surface mesh**
(z = f(x, y) sampled on a grid), with orbit via drag plus arrow keys and an
axis box; nobody ships 3D points-of-interest or trace.

epher already has the parameters (user constants + sliders, ADR-0012/0014),
the samplers, and two renderers that draw lines (SVG, ASCII). The question is
how to add playback and a third dimension without a renderer rewrite.

## Decision

### Animation: a transport layer over constants

- No new language. A constant is animated; the guide's time-based example is
  `const t = 0` + `graph sin(x - t)` and playing t's slider — the Desmos model.
- Web/desktop: every slider row gets a play/pause button. Playback steps the
  constant by the slider's step (0.1) within the slider's shown bounds and
  **loops** (wraps), one step per 120 ms — a v±2 cycle takes ≈5 s, the vendor
  norm. Dragging the animated slider stops playback; the play button is also
  the pause button (WCAG 2.2.2: user-triggered, one control).
- **Reduced motion (WCAG 2.3.3):** `prefers-reduced-motion` degrades the play
  button to a **step button** — each press advances the parameter once, no
  looping playback. The research note found no vendor honoring the
  preference; epher closes the gap instead of ignoring it.
- TUI: the space bar (empty input) starts/stops playback; the loop ticks on a
  50 ms event poll and re-samples everything referencing the constant. The
  animated constant is the first one referenced by any plot.
- The animation loop in the web app communicates through a live
  `Rc<RefCell<PlaySpec>>` cell (plus a cell holding the freshest
  resample callback), not through Yew handles — handles captured at spawn
  read stale snapshots (the same lesson as the trace fix in ADR-0014).
- During playback the 3D plot keeps the **viewBox frozen** at play start, so
  the plot — and its pause button — do not jump around every tick.
- Explicit deferrals: speed control, direction modes (forward/backward/
  oscillate), loop modes (repeat/once), and orientation presets.

### 3D: project in core, draw with the existing renderers

- Grammar: `graph3d <expr(x, y)> [from a to b]` — z = f(x, y) over a square
  domain (default −5..5); several `graph3d` lines overlay; `graph3d clear`
  empties. `from a to b` is the existing 2D domain syntax (ADR-0014).
- Sampling: `epher-core::graph::sample_surface` — a grid (40 TUI, 30 web) of
  `z` values; undefined cells are NaN and split the mesh (discontinuity
  gaps, the survey's universal behavior). A surface with no finite values is
  an error.
- Projection: `View3D` (yaw, pitch, clamped to −1.4..1.4, perspective camera
  at 12) maps world points to screen coordinates in core
  (`project_point`); `project_surface` emits painter-sorted segments and
  `project_mesh` emits whole grid lines as polylines with mean depths. The
  renderers never see 3D — they draw 2D lines, exactly as they do for
  curves. The ground square + three axes come from `surface_frame`.
- Web/desktop: the mesh renders as SVG polylines, painter-sorted far-to-near,
  with per-line depth shading (**opacity**, not color — WCAG 1.4.1), the
  frame on top. Orbit: pointer drag (pointer capture) and arrow keys when
  the plot has focus (WCAG 2.1.1). The SVG content is an innerHTML string,
  not diffed nodes — a thousand-line mesh re-renders cheaply while orbiting.
- TUI: `render_ascii3d` plots the projected segments with Bresenham lines,
  depth-shaded glyphs (`*` near, `+` middle, `.` far), the frame in `o`;
  arrow keys (empty input) orbit; the legend names each surface (`z = …`)
  as the text alternative.
- Explicit deferrals: parametric surfaces and space curves, implicit
  surfaces, solids/color maps/lighting, a resolution knob (`points n`),
  3D points of interest, 3D trace, and named orientation presets.

## Consequences

- One numeric engine and one projection feed both renderers; no WebGL and no
  renderer rewrite. The 3D plot is an SVG image with a text alternative
  (aria-label), keyboard orbit, and no color-only cues, so the WCAG 2.2 AA
  posture of the 2D plot carries over.
- Animation reuses the exact resample path a slider drag uses, so animated
  constants persist via `save name` exactly like dragged ones (ADR-0012).
- Playback never starts on its own anywhere; reduced-motion users get a step
  button instead of motion.
- The TUI event loop polls with a timeout only while playing; idle behavior
  is unchanged.

## Amendment (2026-08-17): near-plane clipping and SVG-namespace injection

Two defects surfaced in the first end-user test of `graph3d` (Android Edge,
but reproducible in every browser):

1. **The plot was blank.** The mesh HTML was injected with Yew's
   `Html::from_html_unchecked`, which parses fragments inside a plain HTML
   `<div>` — so the polyline nodes carried the HTML namespace and the SVG
   renderer never painted them. Structural tests counted DOM nodes, not
   pixels, and missed this. The mesh is now injected imperatively with
   `Element::set_inner_html` on an SVG `<g>` (NodeRef + effect), which parses
   in the SVG context and produces painted, SVG-namespace geometry. Test
   verification now includes pixel-color sampling, not just node counts.
2. **The projection could explode.** With the camera at 12 units and typical
   surfaces reaching z = ±25, the camera plane cuts through the default plot;
   points near it made the perspective divide blow up (viewBox in the
   thousands) and squashed the surface to a sliver. `project_point` gains
   `project_clipped`, which clips segments against a near plane
   (`NEAR_DIST = 1.0` in front of the camera) before the divide; whole grid
   lines are split into visible runs at the crossings, and the frame and both
   renderers inherit the fix. The default camera moved to 30 units so it
   sits beyond typical surface z values — the default view shows the whole
   surface, and near-plane clipping engages only when the user orbits in
   close.

## Amendment (2026-08-18): playback rate, drag orbit, and touch

Three defects surfaced in end-user testing (Windows desktop, Android):

1. **Playback accelerated to a crash.** The web animation loop was spawned
   with Yew's `use_effect` (no dependencies), which re-runs after *every*
   render — each tick re-rendered, spawning another loop, so the step rate
   grew every 120 ms until the page died. The loop now spawns once
   (`use_effect_with((), …)`), holding the designed one step per 120 ms.
   The TUI stepped on its 50 ms poll (≈20 steps/s, 2.4× the web rate); it
   now paces to the same 120 ms per step while keeping the 50 ms poll so
   key presses stay responsive.
2. **Drag-orbit never worked — mouse or touch.** The pointermove handler
   matched `if let Some((lx, ly)) = *drag.borrow()` and then called
   `*drag.borrow_mut()` inside the block; the temporary `Ref` lives for the
   whole `if let`, so the `RefCell` panicked ("already borrowed") and the
   wasm handler died before the first orbit. The start point is now copied
   out (the tuple is `Copy`) before mutating. Keyboard orbit was unaffected,
   which is why browser tests (which used arrow keys) never caught it.
3. **Touch did not reach the plots on Android.** The 2D trace bound
   `mousemove`, which never fires for touch; it now binds `pointermove`
   like the 3D orbit (the SVG already carried `touch-action: none`). 3D
   drag and 2D trace are verified under mobile emulation with CDP touch
   events, and the playback rate is asserted in the browser suite (≈8.3
   steps/s, steady over 10 s, page alive).

## Amendment (2026-08-27): the slider and play button move above the points-of-interest list

The 2D pane ordered itself plot → trace → points-of-interest list → slider
rows. The animation slider and its play button are controls driving the
plot, so they belong with it: the slider rows now render directly beneath
the plot (and the trace readout), above the points-of-interest list. The
POI list is a passive readout and keeps the bottom of the pane. One DOM
move in the shared web frontend serves the desktop app and the PWA; the
TUI has no slider (its animation control is the space bar) and its pane
layout is unchanged.

## Amendment (2026-08-27): the pane shows one kind at a time

A 2D plot and a 3D plot share the pane's vertical space, which shrank
both below practical size. The pane now holds **one kind at a time**:
drawing a surface clears the 2D curves (and their points of interest);
drawing a curve clears the surfaces (and resets the 3D pose when the next
surface arrives, exactly as a first surface does). Same-kind overlays are
unaffected — several curves still share one 2D plot, several surfaces
still share one 3D plot. Explicit `graph clear` / `graph3d clear` remain
kind-specific, and a failed plot never clears anything. The web frontend
(desktop + PWA) and the TUI enforce the switch at their submit seams; the
CLI's `Plots` state is untouched because `graph save` / `graph3d save`
write separate SVG documents that never share a canvas.

## Amendment (2026-08-27): per-kind line widths with separate sliders, and the legend's visibility checkboxes

The line-width slider was one control serving both kinds (desktop) or
switching ranges with the kind (mobile, ADR-0031/0035). It is now **two
sliders, one per graph kind, only the kind in view shown**: 2D curves get
0–4 in steps of 0.1 (default 1.0), 3D surfaces get 0–0.2 in steps of 0.01
(default 0.1). Each kind remembers its own width under its own
localStorage key on every display (desktop included — the legacy shared
key still seeds both), and each kind's plot renders with its own value.
The slider sits in the pane toolbar right of **Copy SVG** on desktop and
wraps to its own row below the toolbar on mobile (`.graph-width`
`flex-basis: 100%` under the 880px media query); the parameter sliders
and play buttons stay below the plot per the earlier amendment.

The 2D legend gets a checkbox in front of every entry, checked by
default: unchecking hides that curve from the plot, its points of
interest, and the SVG export (the export shows what the pane shows). The
checkboxes are real labelled form controls (the curve's caption names
them), so the plot can be restored without touching the expression.
Points of interest now carry their owning curve's index
(`InterestPoint::curve` / `Poi::curve`) so a hidden curve's points
disappear with it. The TUI has no legend and is unaffected.

## Amendment (2026-08-28): the animation tick is deadline-paced and minimal, and hidden curves never shift the palette

Two playback defects reported against the web app and the desktop shell:

- **A jerky, slowing frame rate under load.** The loop slept 120 ms and
  then did its work, so the work time was added to the period on every
  tick: on a loaded machine a 60 ms tick stretched the frame to 180 ms,
  and the lag compounded. The loop now paces by deadlines: it does the
  tick's work first, then rests until the next 120 ms mark (the
  wasm-safe `js_sys::Date::now()` clock), so the period stays 120 ms
  whenever the work fits and an overrunning tick never compounds — it
  simply catches up at the next deadline.
- **The tick did more than the frame needs.** It re-sampled every curve
  and surface even when only one constant moved, re-ran the full
  points-of-interest analysis (bisection and golden-section searches)
  on every tick, and rewrote the visibility state each time. The tick
  now re-samples only the curves/surfaces whose expression references
  the animated constant, runs the analysis at 2 Hz (every fourth tick —
  the markers track the moving curve without gating every frame), and
  leaves the legend checkboxes untouched mid-playback.

Storage is deliberately untouched by playback in every frontend: the
web and the TUI persist only on user actions (submit, clear, menu
actions; play/pause and the sliders commit through the session state,
never through the store), verified on the desktop app by watching the
store files' mtimes across five seconds of playback — they do not move.

The legend checkboxes also exposed a palette bug: hiding a curve
re-indexed the remaining visible curves (position 0, 1, …), so a line
could change colour when a neighbour was hidden, disagreeing with its
legend swatch. The pane and the SVG export now carry each curve's
ORIGINAL palette index through the filter, so a hidden neighbour never
shifts the remaining colours (the exported SVG gained the indexed
variant `graph_svg_indexed`; the plain `graph_svg` keeps the
position-index behaviour for the terminal frontends, which never
filter).

## Amendment (2026-08-29): 3D parametric space curves and positioned points (the solar system view)

The original deferral list included "parametric surfaces and space curves".
This amendment lifts **space curves** (parametric surfaces stay deferred) and
adds positioned points, because the astronomy feature set (ADR-0037) needs to
show the solar system: orbits as curves and planet positions as dots.

- **Grammar:** `graph3d param <x(t)>, <y(t)>, <z(t)> [from a to b]`,
  mirroring the 2D `param` keyword from ADR-0014. Sampling walks t across the
  domain with the same run-splitting rule as 2D curves (NaN breaks the run,
  so discontinuities and out-of-domain stretches leave gaps).
- **Projection:** each sampled point goes through the same `View3D` transform
  as surface points; the projected run becomes painter-sorted polylines with
  mean-depth opacity, identical to mesh lines. The renderers still never see
  3D.
- **Positioned points:** a new 3D element kind, a labelled dot at (x, y, z),
  projected like any point and drawn after the lines (filled circle, radius
  shrinking with depth, label text in the legend/accessible description).
  This is the 3D analogue of the 2D points-of-interest markers, but placed by
  data instead of computed from a curve.
- **The solar system preset (ADR-0037):** `solar3d [t]` builds the scene from
  the ephemeris facade: each body's orbit as a space curve (sampled over one
  heliocentric period), each body's position as a positioned dot, trails
  behind the dots for motion context. The optional time argument may be a
  user constant, so the existing animation transport (constant playback, the
  ADR-0015 play button) animates the solar system with no new playback
  machinery. Orbit (drag, arrow keys, touch), the frozen viewBox behaviour,
  and the SVG/ASCII renderers all apply unchanged.
