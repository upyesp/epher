# ADR-0091: the solar time knob spans the outermost displayed orbit, and a legend toggle re-spans it and refits the frame

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-11-22

## Context

ADR-0090 gave the solar pane's time knob a month-wide window and a
~24-second play cycle, which made the guide's `const t = now(); solar3d
t` come alive for the first time. But a month is one Earth lap either
way: with every body checked — the pane's default — the sweep covers
sixty days while the frame holds Pluto's 49-AU aphelion. Neptune
crosses 0.2° per play cycle; the user watches outer planets that are,
for animation purposes, furniture.

The pane already had the machinery to do better, pointed the wrong way.
The legend's per-body checkboxes (ADR-0038) hid bodies without ever
moving the frame, by the ADR-0038 amendment's hide-never-rescales rule,
so the view kept a frame sized for bodies the user had sent away, and
the sweep never depended on what was actually being watched.

## Decision

The displayed body set governs both the sweep and the frame.

- **The sweep is one outer orbit.** The knob's span is the drawn Julian
  Date ± half the orbital period of the OUTERMOST body still displayed,
  so one play cycle — 200 ticks at the 120 ms cadence, ~24 s, the
  ADR-0015 vendor-norm speed and ADR-0090's cycle — completes exactly
  that body's full orbit. The period comes from the sampled orbit
  itself: the ellipse's extreme distances from the Sun give the
  semi-major axis, and Kepler's third law gives the days
  (`epher_core::astro::solar_time_span_for`). Pluto's ~248-year orbit
  is the everything-checked default; unchecking the outermost planet
  hands the sweep to the next one out. The same helper serves both
  frontends, as in ADR-0090.
- **A toggle re-spans the animation.** Enabling or disabling a body
  recomputes the span at once; a running play picks up the new sweep
  without stopping (the span re-centers on the knob's current value,
  so the watched instant never jumps), and the knob's slider track
  follows. The month window of ADR-0090 survives as the fallback for
  the nothing-displayed pane — the legend can hide every orbit, and
  the pane still renders and animates (the ADR-0039 empty pane).
- **A toggle refits the frame.** The pane's viewBox now follows the
  DISPLAYED orbits: hiding the outermost body shrinks the frame to
  just big enough for the next one out, re-checking widens it back.
  The toggle zeroes the zoom offset as it lands, so the fit is exact
  whatever the user had zoomed to. This supersedes the ADR-0038
  amendment's hide-never-rescales rule for the solar pane (the
  curve/surface legends keep it). The ADR-0041 stillness rule is not
  weakened: the fit reads the fixed orbit curves, not the moving dots,
  so the frame still never resizes WHILE the system rotates or
  animates — only when the displayed set changes. Exports, which
  already fit what they show, now match the pane.
- **The terminal follows the same rule.** The TUI has no per-body
  checkboxes; its displayed set is the whole scene, so its space-bar
  cycle spans Pluto's orbit. Both frontends animate the same sky at
  the same rate, as ADR-0090 required.

## Consequences

- The knob is honest about what it shows: watch Jupiter alone and one
  press of play is one Jovian year; check everything and one cycle is
  a full Plutonian revolution, with the inner planets' blur as the
  read of the system's clock speed.
- The default sweep is ~248 years, not ADR-0090's month. The guide's
  promise ("watch the planets move") now means: drag across two and a
  half centuries, or watch Pluto lap once in ~24 s. Users wanting the
  month-long inner-system view uncheck the giants — the legend is the
  control for exactly this.
- The reduced-motion press (ADR-0090: one twentieth of the span) rides
  along automatically: three days per press on the fallback month,
  ~12 years on Pluto's sweep.
- `SOLAR_TIME_WINDOW_DAYS` (30) is demoted from the span itself to the
  nothing-displayed fallback; `solar_time_span` keeps its signature.
- The ADR-0038 amendment's hide-never-rescales guarantee is narrowed
  to the curve and surface legends; solar legend toggles are the
  documented exception, and the pane's comment says so where the frame
  is computed.
- Browser regression `scripts/browser/solar-animate.mjs` pins the
  whole contract: the outermost-orbit sweep, a mid-play toggle keeping
  playback running on the new sweep, the frame narrowing and restoring
  with the outermost body, the month fallback, and the empty pane.
  Core unit tests pin Kepler's law against the sampled orbits.
