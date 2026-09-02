# ADR-0042: percent, constants, number theory, suggestions, and PNG export

Date: 2026-08-31

## Status

Accepted (amends ADR-0012's constant resolution, ADR-0016's keypad
layout, ADR-0020's pane toolbar, ADR-0021's ans, and ADR-0039's hint
surfaces)

## Context

The competitive gap analysis (docs/research/calculator-feature-gap-analysis.md,
round 1 of the approved roadmap) found five quick wins epher lacked while
nearly every mainstream calculator has them: a percent operator, a
physical-constants library, number-theory helpers, input help
(autocomplete and function help), and a bitmap plot export. Users expect
each within minutes of first use.

## Decision

**Percent is a transparent /100 suffix.** `%` is a postfix operator
binding at factorial's level (tighter than `^`): `5%` is 0.05,
`10%%` is 0.001, and `200 + 10%` is 200.1. The Casio-style add-on
reading (`200 + 10%` = 220) is deliberately rejected: it is the one
context-dependent rule a grammar can have, and epher's grammar has none.
"Increase 200 by 10%" is spelled `200 * (1 + 10%)` - the transparent
spelling teaches what the calculation is. (Decision confirmed by the
user, 2026-08-31, against the add-on alternative.)

**A physical-constants catalog joins the built-ins.** Twenty-one CODATA
2022 values in SI units: `G`, `gamma`, `q_e`, `ev`, `eps_0`, `mu_0`,
`z_0`, `m_e`, `m_p`, `m_n`, `m_u`, `a_0`, `alpha`, `r_inf`, `mu_b`,
`n_a`, `faraday`, `r_gas`, `atm`, `wien`, `phi_0`. They resolve exactly
like the astronomy constants (ADR-0037): user variable, then user
constant, then built-in - shadowable, and `const` redefinitions keep
erroring. No keypad keys: the guide table and the suggestion list are
their surfaces.

**Number theory on the exact integers.** `isprime`, `nextprime`,
`prevprime`, `factors`, `totient`, `ndivisors`, and `modpow` work on the
integers f64 reaches exactly (|n| < 2^53): deterministic Miller-Rabin
(the 12-witness set is exact on all of u64), Pollard rho for splitting,
and BigInt arithmetic where the result must be exact (`modpow` returns
`big` precision). `factors(n)` returns a display Str following the
bin/oct/hex precedent - it reads like `2^3 * 3^2 * 5` and is not a value
to compute with; a list-valued form waits for list values. `isprime`
returns false for anything below 2: primes are positive by definition.

**Suggestions, F1 help, and auto-ans.** The web and desktop entry gains
a combobox: while typing a name, a listbox under the entry suggests
prefix matches from the session's own functions, constants, and
variables first, then the built-in catalog (`catalog()` in core - a
descriptive index, not load-bearing), capped at eight, each carrying its
`key-hint-*` description. Arrows move the highlight, Enter/Tab accept
(functions complete with an open paren), Esc closes, a click accepts
without stealing focus, and the textarea's aria-activedescendant follows
the highlight. F1 shows the hint for the word under the cursor in the
hint bar (web) or the answer line (TUI). An operator typed into an
empty entry - physically or from the keypad - inserts `ans` first
(SpeedCrunch/NumWorks behavior): operators continue from the previous
answer, digits and names start fresh. The CLI keeps its plain line
input; the TUI's Tab stays focus-cycling per ADR-0033/0034, so its
completion is the F1 help plus auto-ans, and its keypad gains the %
key.

**Save PNG beside Copy SVG.** The pane toolbar gains a PNG button that
serializes the same self-contained SVG document Copy SVG produces
(hidden curves excluded, ADR-0015 amendment), rasterizes it on a canvas
at twice its size, and saves it through the platform's save flow: the
desktop dialog over IPC (`save_png_dialog` carries bytes, not text), the
browser's File System Access picker, else a plain download. The CLI
keeps SVG as its only export.

## Consequences

- The suggestion list needs localized descriptions only where hints
  already exist; new names without hints suggest bare, and F1 says so -
  no new translation burden beyond the new keys themselves.
- The catalog is hand-maintained and drift-safe in one direction only:
  a test proves every catalog entry evaluates (no unknown names), but a
  builtin missing from the catalog merely does not suggest.
- % changes no existing behavior (it was a parse error), so no
  compatibility concerns; `200 + 10%` results change for nobody.
- PNG export is client-side and offline, like everything else.

## Amendment (2026-08-31): quick wins round 1 shipped

All five decisions above are implemented in round 1 (v0.5.5): the %
operator and keypad key, the 21 constants, the seven number-theory
builtins, the web/desktop suggestion combobox with F1 and auto-ans, and
the Save PNG button with its desktop IPC command. The TUI ships F1 +
auto-ans + the % keypad key; its Tab-completion stays open for a later
round.

## Amendment (2026-08-31): the % key moves to the n-Tab; the digits tab is frozen

Review of v0.5.5 caught the keypad overflow this ADR introduced: the
web digits tab's five-row grid holds 25 cells and the = key spans two
of them, so the bank was already exactly full at 24 keys - the % key
pushed = onto a sixth row and the 123 tab scrolled. The same count
applies to the TUI, whose 80x24 frame (ADR-0033) has no spare row
either.

The % key therefore moves to the number tab (nSigma on the web, num in
the TUI), beside the other exact-integer arithmetic: it stays one tap
from the operators it extends, and every tab still fits the fixed
keypad with no scrolling.

Policy, by direction of the project owner: the digits tab/bank is
frozen. No key may be added, removed, or moved there without explicit
approval, in any frontend. Guard tests hold the line (the web test
asserts the tab's exact 24 keys and that every tab fits the grid; the
TUI test asserts the bank's exact contents).

## Amendment (2026-09-02): transparent PNG (ADR-0055)

The PNG export rasterizes the transparent SVG document on a clear
canvas, so saved PNGs carry no painted background (the earlier text's
implicit dark backing came from the document's background rect, which
ADR-0055 removed).
