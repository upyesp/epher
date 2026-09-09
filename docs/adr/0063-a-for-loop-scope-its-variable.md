# ADR-0063: a for loop scopes its variable

Date: 2026-09-09

Status: Accepted. Amends ADR-0054 (the loop variable kept its last
value afterwards, "like TI's For").

## Context

`for i in 1 to 3 do i` bound `i` in the session environment and left it
there — the last iteration's value — after the loop ended. That was a
deliberate nod to TI's `For`, and it is a footgun with two heads:

1. **In-session:** `i` is the imaginary unit (ADR-0043 makes it
   shadowable, exactly like other built-in names are). After any loop
   over `i`, `3+4i` silently stopped being the complex literal.
2. **Across sessions:** the session store persists the environment's
   bindings (ADR-0010 amendment). A loop from yesterday rode the
   snapshot into today: with a stored `i = 5` from a `for` loop,
   `3+4i` answered `23` and `im(3+4i)` answered `0` — the defect the
   calculator gap analysis (docs/research/
   calculator-gap-analysis-3rd.md, "Defects found by probing") caught
   on a real install.

The honest note in that analysis — the imaginary unit is shadowable,
`pi` is not, and a loop named `i` is a footgun that survives restarts —
framed the choice: reserve `i`, un-shadow it, or stop loops from
leaking. Reserving `i` would contradict ADR-0043's deliberate
shadowability; the other names stay shadowable, so `i` would be a
special case with no principle behind it.

## Decision

The `for` loop's control variable is scoped to the loop. When the loop
ends, the name reverts to the binding it had before the loop — usually
none, so nothing is left behind and nothing reaches the session store.
Assignments to other names inside the body persist exactly as before
(the accumulator pattern keeps working), and a binding the loop
variable shadows is restored, not destroyed:

```epher
i = 7
for i in 1 to 3 do i
i      // 7, not 3
```

An explicit user assignment to `i` still shadows the imaginary unit —
that stays ADR-0043's call, made on purpose. What changes is only that
a loop can no longer do the shadowing by accident, in this session or
the next.

Nested loops over the same name unwind in order: the inner loop owns
its `i` per iteration and restores the outer's; after the outer loop
ends, neither remains.

## Consequences

- `3+4i` means the complex literal after any loop over `i`; a fresh
  session can never inherit a loop's `i` from the store.
- Scripts that read the loop variable after the loop (there are none
  in the shipped collection — the checker's transcript replay caught
  no breakage) must bind the value to another name inside the body.
- The guide's loop section changes from "keeps its last value
  afterwards, like TI's For" to the scoping rule, in all eight
  languages.
