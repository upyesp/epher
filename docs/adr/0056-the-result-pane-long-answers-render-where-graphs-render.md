# ADR-0056: The result pane — long answers render where graphs render

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-09-03

## Context

The calculator has always shown its answer in a one-line answer panel
under the entry (ADR-0016). The scripts repository (v0.5.21) made
pasted scripts a first-class input, and a pasted script's transcript
does not fit a one-line panel: several answers joined with semicolons
wrap and scroll inside a 2.8rem box, which reads badly — the user
copies a script, pastes it, and gets a wall of clipped text. A short
single answer, by contrast, reads perfectly where it has always been.

## Decision

The graph pane becomes the result pane: one pane that renders whatever
a submission produced — curves, surfaces, the solar system as before,
and now long answers.

- A **short single answer** (no separator, no line breaks, and short
  enough for one calm line — about 44 characters on desktop, about 24
  on a phone) stays in the answer panel under the entry, exactly as
  before.
- **Anything longer** — a script's transcript with several answers, a
  table or matrix with its own line breaks, a long number — renders in
  the result pane, one answer per line, never joined with semicolons.
  The pane's heading and tab read "Result" in every locale.
- On mobile the result pane **slides into view** when a long result
  arrives, under the same rule as a drawn plot (ADR-0035's slide
  contract): the answer is on screen without a gesture, and the entry
  drops focus so the keyboard closes.
- Answers and plots share the pane without ceremony: a transcript
  renders above any curves, and a short answer or a plot message still
  uses the answer panel.

The routing rule (`answer_fits_at`) is pure — it takes the
mobile-layout answer as an argument — so it is tested natively and
consulted twice: where the render decides which region holds the text,
and where the submit path decides whether to slide.

## Consequences

- The one-line answer panel keeps only what it is good at; its
  semicolon-join layout (ADR-0055) now carries at most one inline
  answer and survives unchanged for messages and plots.
- The result pane is a live region (`role="status"`) like the answer
  panel, so a long transcript is announced without a focus change
  (WCAG 4.1.3).
- The user guide (§2.2) and the eight locale translations describe the
  routing; the pane tab's localized label moved from "Graph" to
  "Result" in the eight fluent locales.
- Long transcripts in the pane scroll inside the pane, like long plots;
  nothing about the fixed-viewport mobile layout (ADR-0035) changes.

## Amendment (2026-09-05): the terminal routes answers by the same rule

The routing was web-only: the terminal showed every answer in the
small answer area between the entry and the history, where a long
transcript clipped after six rows. The desktop app and the PWA sent
the same answer to the result pane, so the same paste behaved
differently depending on the frontend — confusing for anyone moving
between them.

The terminal now consults the same rule. `answer_fits` keeps the web's
definition (one answer, no line breaks, at most 44 characters wide
layout / 24 narrow, counting the whole line including the `= ` voice);
the TUI needs no separator test because its transcripts are
newline-joined. A long answer empties the answer line (which keeps
one row, so the layout does not jump) and renders in the result pane —
the pane the terminal already calls "Result" — one answer per line,
above any curves, with the plot sized to the rows the transcript
leaves. The every-answer-visible contract (ADR-0052) is carried by the
pane for transcripts, as it is on the web.
