# ADR-0064: statement bodies, control flow, destructuring, and the string library

Date: 2026-09-17

Status: Accepted. Extends ADR-0054 (strings, `for`, `print`),
ADR-0043 (shadowable names), and ADR-0012 (constants). Records a
deliberate non-decision: script `input` and file I/O stay out of
scope.

## Context

The calculator gap analysis, third pass
(docs/research/calculator-gap-analysis-3rd.md), left exactly one
language row partial: the programming surface. What kept it partial
was input and file I/O beyond `print`, single-expression function
bodies, one value per function, no string library, no loop control,
and the shadowable `i` the same report documented as a footgun. The
baseline (calculator-feature-gap-analysis.md, T3.4) had named the same
residue: TI-Basic, NumWorks Python, PPL, and Octave all read input,
write files, run statement bodies, and return several values; epher
did neither.

Meanwhile the language's own idioms showed the strain. Scripts packed
loop state into lists (`while v[2] < n do v = {v[1] + 1, …}`) because
a body is one statement; coin-streak encoded two numbers as
`run * 1000 + best`; functions that needed two steps nested
conditionals; report scripts rounded numbers through arithmetic
because strings could not hold a formatted value.

## Decision

Four capabilities join the language, one is recorded as out of scope,
and one question stays open.

1. **Statement bodies for functions.** `def name(params) do … end` is
   the second body form beside `def name(params) = expr`. The
   statements run one after another in the call's own environment
   (parameters bound, caller's variables invisible, constants and
   functions visible); the last statement's value is the call's
   answer, and a body whose last statement produces no value is a
   named error ("produced no value"), not a silence. Every call
   carries its own 100,000-step budget, the same runaway guard a
   script gets.

2. **`return`, `break`, `continue`, and the statement `if`.**
   `return value` leaves the function now; `break` leaves the loop;
   `continue` starts the next pass. They are statements, so they sit
   behind an `if` — and `if` therefore exists as a statement beside
   the expression form: `if c then stmt [else stmt]` chooses between
   statements, and with no `else` taken it produces no value, which is
   how a `for` loop filters. `for` keeps collecting: values so far are
   the loop's answer after a `break`. At the top level the three jumps
   are structure errors ("break outside a loop") that name the
   mistake. Loop bodies stay one statement — a `do` body with a loop
   inside it, or a `return` behind the body's `if`, covers the rest.

3. **Destructuring.** `{a, b} = list` binds several names from one
   list, left to right; `_` skips a position; the statement's value is
   the whole list, like any assignment. Arity mismatches and
   non-lists are typed errors; constants keep their guard. This is the
   written form of "return several answers": a `do` body ends with
   `{mean, sd}` and the caller names them in one move.

4. **Strings grow up.** Escapes (`\n`, `\t`, `\r`, `\\`, `\"`) join
   the tokenizer — any other backslash is a named parse error — and
   ordering comparisons read dictionary order. List literals hold
   strings beside numbers (a `for` collect always could). Nine
   builtins cover report writing: `upper`, `lower`, `trim`,
   `substr(s, start[, len])` (1-based, clamping), `split(s, sep)`,
   `join(list, sep)` (spelling elements the way `print` does),
   `find(s, sub)` (1-based, 0 when absent), `replace(s, old, new)`,
   and `fixed(x, digits)` — a number as text with exactly that many
   decimals, the trailing zero a report wants kept. All nine sit on
   the data keypad bank in the PWA and the TUI, in the autocomplete
   catalog, and in the ×8 key-hint tables.

The statement splitter (shell `split_statements`, shared by CLI,
TUI, and web paste) tracks `def … do` block depth so `;` and newlines
inside a block body do not fragment a pasted program, and honors
string escapes; the tokenizer lets `_` start an identifier so the skip
position has a name. Functions persist as `def` source (ADR-0002), so
old stores load unchanged and block-bodied functions round-trip.

**Out of scope, by decision.** Script `input()` and file I/O stay out
of epher. Input breaks the synchronous evaluator that web and desktop
share, pauses a paste-as-one-paste run in the TUI, and collides with
the transcript checker's guarantee that every example runs
deterministically without a human; path-based I/O forks behavior
between a sandboxed PWA and four native frontends and cannot survive
the installers' portable scripts (ADR-0058). The calculator stays a
closed program: expressions in, answers out, state in the shared
store. Should data ever need to move, the epher-shaped seam is
store-scoped named datasets, not file paths — decided when a real need
arrives.

**Open:** the shadowable `i` (the one residue the third pass called a
footgun rather than a feature — a bare `i = 5` persists in the shared
store and turns `3+4i` into `23` everywhere). ADR-0063 scoped the loop
variable; whether bare assignment should follow is deferred to its own
decision with the user.

## Consequences

- The programming-surface row's honest residue narrows to input and
  I/O — exactly the two rows this ADR declares out of scope, so the
  row's framing changes from "not yet" to "by decision".
- Scripts stop packing: the collection's packed whiles convert where a
  known range, a `return`-scan, or a destructure reads better
  (monte-carlo-pi, coin-streak, galilean-moons, saturn-satellites in
  this round), each transcript re-verified against the engine.
- The guide teaches the new surface in §1.5, §1.7, §1.9–§1.12 and the
  quick reference, in all eight languages; code and output blocks stay
  locale-independent (ADR-0007).
- Deep recursion remains bounded by the host stack, not the step
  limit — pre-existing (the step budget counts statement
  executions), unchanged here, and worth its own note if scripts ever
  grow recursive scanners beyond a few hundred frames.
