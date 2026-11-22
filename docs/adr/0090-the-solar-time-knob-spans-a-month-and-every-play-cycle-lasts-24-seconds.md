# ADR-0090: the solar pane's time knob spans a month and every play cycle lasts ~24 seconds

- **Status:** accepted; the span rule is superseded by ADR-0091
  (2026-11-22): the knob now spans the outermost displayed orbit, with
  this ADR's month window as the nothing-displayed fallback. The
  ~24-second cycle, the proportionate play step, and the
  reduced-motion press stand unchanged.
- **Deciders:** epher maintainers
- **Date:** 2026-11-22

## Context

The guide (§1.x astronomy) and the examples page hand out the same
script: `const t = now()` then `solar3d t`, and promise that pressing
the knob's play button lets you "watch the planets move". A 2026-11-22
report found the promise hollow: the slider appeared, the graphic
never visibly changed, and play looked dead.

Three facts compounded:

1. **The knob window was day-scale.** `slider_span` gives any constant
   outside −10..10 a tight v±2 window (the ADR-0055 fix against
   million-wide Julian-Date sliders). For a Julian Date, ±2 is two
   *days*: Earth crosses ~2° across the whole slider track, and the
   inner planets read as nailed down. The window was right for
   dragging a number; it was wrong for *time*, where a month is the
   smallest span in which the system visibly comes alive.
2. **The play step was fixed at 0.1.** One tick advanced t by 0.1 —
   2.4 hours of orbital time. Mercury moves 0.2° per tick; Earth
   0.06°. Nothing a human eye can see, ticking at the loop's 120 ms
   cadence. The step that suits a −10..10 parameter is invisible on a
   day-scaled one.
3. **Reduced motion made it worse.** Under
   `prefers-reduced-motion: reduce` (Windows "Show animations" off,
   iOS Reduce Motion, RDP sessions), play performs a *single* 0.1 step
   per press — 2.4 hours, no loop — so the press looked like a no-op.
   The WCAG 2.3.3 rule (no autoplay) is right; the step size was not.

Separately, the terminal UI could not animate the guide's script at
all: `const t = now(); solar3d t` is a multi-statement submission whose
script scope evaporates its constants when it closes (the ADR-0012
amendment), and the TUI's play and re-sample paths gated on the
session environment only — the same gap the web pane closed for its
sliders in the 2026-11-12 fix, unfixed on the TUI side.

## Decision

The solar pane's time knob gets time-scale semantics, and the TUI gets
the web pane's draw-environment rule.

- **A month-wide window.** The solar knob's slider and play span is
  the drawn Julian Date ± `SOLAR_TIME_WINDOW_DAYS` (30), shared by
  both frontends through `epher_core::astro::solar_time_span`. Every
  other pane keeps the generic `slider_span` rule untouched.
- **A proportionate play step.** The play loop steps `(hi − lo) / 200`
  per tick: 200 ticks at the 120 ms cadence, so one full cycle always
  lasts ~24 s — the ADR-0015 vendor-norm speed — whatever the span's
  width. Over the base −10..10 span this is the historical 0.1
  exactly; over the solar month it is 0.3 days per tick, where the
  inner planets visibly travel frame to frame.
- **A visible reduced-motion step.** Under reduced motion a play
  press advances one twentieth of the span instead of a bare 0.1:
  three days per press on the solar knob, one unit on a −10..10
  constant. Still no autoplay (WCAG 2.3.3 unchanged); the press is
  merely no longer sub-perceptual.
- **The TUI mirrors the web pane.** `App` captures a draw-time
  environment at every successful draw (the newest scope's defs win;
  stored extras linger harmlessly, the web `absorb_defs` rule). Play,
  tick, and re-sample resolve constants against the live session
  first and the draw-time snapshot second, so script-scoped constants
  keep animating after their scope closes, exactly as the web knobs
  have since the 2026-11-12 fix.

## Consequences

- The guide's example does what it says on every surface: dragging
  the knob sweeps a month of ephemeris, and play cycles it in ~24 s.
- Non-solar behavior is bit-identical: −10..10 spans step 0.1 as
  before; the TUI's 2D/3D surface knobs keep their historical ±2 span
  at 0.1 per tick.
- The reduced-motion press size changes for every pane (0.1 →
  span/20). This is a deliberate UX trade: a press must be perceptible
  to be a control at all.
- The step formula lives in one helper per frontend (`play_step` in
  web, the span arithmetic in `toggle_play` in the TUI) so the 24 s
  cycle cannot silently drift between them.
- Browser regression `scripts/browser/solar-animate.mjs` (wired into
  the a11y workflow) drives the real PWA through the guide's exact
  script and asserts the knob spans a month, a drag moves the planets,
  and play animates and is announced; TUI regressions pin the
  script-shaped, interactive, and historical-span behaviors.
