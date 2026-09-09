# ADR-0065: the imaginary unit is reserved

Date: 2026-09-17

Status: Accepted. Resolves the open question left by ADR-0064;
amends ADR-0043 (shadowable built-ins) and ADR-0063 (for loops scope
their variable).

## Context

ADR-0043 made every built-in name shadowable, `i` included, and
ADR-0063 stopped `for` loops from leaking an `i` into the session.
What remained was the deliberate bare assignment: `i = 5` bound in the
session, persisted through the shared store (ADR-0010 amendment), and
turned `3+4i` into `23` on every frontend until the binding was
removed — the defect the third gap analysis caught on a real install.
ADR-0064 left the question to the user rather than decide it.

The user's decision, taken 2026-09-17: **reserve `i`**; **drop** any
stored `i` silently when a store's bindings load; implement **now**,
in the same round as ADR-0064.

## Decision

`i` is the one reserved name in epher:

- `i = 5`, `const i = …`, and a destructuring pattern naming `i` are
  all refused with `cannot assign to i: that name is the imaginary
  unit`.
- `restore_bindings` (the one seam every frontend loads a store's
  session through) drops a stored `i` silently. Old installs carry no
  footgun forward and nothing is printed about it; `3+4i` simply
  answers `3+4i` again.
- Scoped uses keep working, because they never persist: `for i in …`
  keeps `i` as the classic loop counter (ADR-0063's scoping), and a
  parameter named `i` lives only for the call. `def i(…)` remains
  possible and harmless — a function name never shadows the constant
  in variable position.

No other built-in changes: `pi`, `e`, `tau`, `phi`, and the astronomy
constants stay shadowable exactly as before. The earlier reports'
claim that `pi` could not be shadowed was wrong — probing during this
round showed `pi = 5` succeeds — which makes reservation a genuine
first rather than a catch-up, and the guide says precisely that.

## Consequences

- The gap-analysis footgun is closed at the root: no assignment, no
  store, no frontend can shadow the imaginary unit. `4i` remains the
  literal spelling of `4 * i` (ADR-0043), now guaranteed to mean the
  same thing in every session.
- A user who genuinely wanted `i` as a counter variable renames to
  `k`, `n`, or `t` — the names the scripts collection already uses.
- The guide's §1.5 names note and §1.18 carry the reservation, in all
  eight languages.
