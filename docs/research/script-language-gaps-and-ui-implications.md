# Script-language gaps after the third gap analysis, and what each costs per frontend

Date: 2026-09-17 · Status: gaps 3-6 closed by ADR-0064 (statement
bodies, return/break/continue, destructuring, the string library); gaps
1-2 declared out of scope by decision; gap 7 closed by ADR-0065 (the
imaginary unit is reserved, stored `i` bindings drop at load) ·
Basis:
docs/research/calculator-gap-analysis-3rd.md (2026-09-06, measured on
v0.5.37), re-verified by probe against the current staging binary
(v0.5.40 + satellites, `03cc614`).

## Why this note

The third gap analysis left exactly one partial matrix row that is a
language row — **programming surface** (5 of 9 apps offer one) — plus a
handful of language-adjacent findings. This note spells out what the
remaining gaps are, what the comparison apps put on the other side of
each gap, and what closing each gap would cost in each of epher's five
frontends. It makes no changes and recommends no round.

Everything below marked "verified" was re-probed against the current
binary; the two probe defects the third pass found (CLI `graph3d param`,
loop-`i` shadowing across sessions) were fixed on staging since the
report (a898b8a, d6f7be0 / ADR-0063) and are closed.

## What epher's language has (the floor everything else stands on)

One grammar, five frontends, one shared live store. Verified surface:
exact fractions by default, lists and matrices as values, strings with
`+`/`==`/`!=`/`len`/1-based indexing/`str`, `print`, `for` (range and
list forms, body = one statement, values collected into a list),
`while`, `if`/`then`/`else` as an expression, `def f(x) = expr`
(recursion works; `fib(10)` = 55), comments (`//`, `#`, `/* */`),
seeded randomness, a 100,000-step runaway guard, and store persistence
(`save name` carries definitions across sessions and frontends).

## The gaps, each against the apps that have it

### 1. No input — the user cannot feed a running script

`input()` is an unknown name (verified). TI-Basic has `Input`/`Prompt`,
NumWorks Python `input()`, HP Prime PPL `INPUT`, Octave `input`. The
third pass names this first: it is half of what keeps the programming
row partial.

### 2. No file I/O beyond `print` to the answer pane

No `open`, no read/write (verified). PPL, Octave, and NumWorks Python
all read and write files. Note epher's twist: `save`/`load` already
persist named definitions in the shared store — what is missing is
data I/O (tables, datasets, results) and path-based files.

### 3. Functions are single expressions

`def f(x) = expr` — the body is one expression (verified: a body with
two statements is a parse error; there is no `return`, no `do…end`
bodies, no early exit). TI-Basic, PPL, Python, and Octave all have
statement bodies with explicit return. epher compensates with
`if/then/else`-as-expression and recursion, but a two-step helper
("compute a, then decide with a") must be written as a nested
conditional or split into two defs.

### 4. One value per function; no destructuring

Returning a list is the house idiom (`def stats(xs) = {mean, sd}`
works, verified), but `{a, b} = f(x)` does not parse — callers index
`v[1]`, `v[2]` (verified). The baseline's T3.4 named multiple return
values; the loop-body "pack state in a list" style is the same
limitation felt inside control flow.

### 5. Strings are second-class

Verified: no escape sequences at all (`\` is "unexpected character";
a string cannot contain `"` or a newline); arithmetic beyond `+` is
refused with "strings only support + (concatenation)"; no ordering
comparison (`"a" < "b"` is a type error); no `upper`/`lower`/
`substr`/`split`/`join`/`find`/`replace`/`format` — none exist. What
exists: literals, `+`, `==`/`!=`, `len`, indexing, `str()`. The third
pass calls the string library "a product decision" — the smallest
useful core is escapes + split/join + a formatted-number function,
because scripts that print reports (finance schedules, astronomy
tables) currently round through arithmetic instead of formatting.

### 6. Loop control is minimal

No `break`, no `continue` (verified: a `break` attempt is a parse
error); bodies are one statement; the runaway guard is the only exit.
TI-Basic has `Exit`, Python `break`/`continue`, PPL and Octave both.
The one-statement body plus list-collecting is coherent, but
"scan until found" (the Galilean-moons phenomenon scan) is written as
a `while` with a sentinel because `break` does not exist.

### 7. The imaginary unit is still shadowable by bare assignment

Post-ADR-0063 the *loop* variable is scoped, but `i = 5` persists in
the shared store, and afterwards `3+4i` answers `23` (verified today;
the third pass documented the cross-session version of this on the
installed machine store). `pi` cannot be shadowed; `i` can — the
guide's "works exactly like `pi`" is not true for assignment. This is
the one gap that is a footgun rather than a missing feature, and the
store makes it cross-frontend and cross-restart.

### Deliberate boundaries (listed for completeness; not gaps to remove)

CAS/symbolic algebra (4/9 apps): `expand`/`factor`/`simplify` are
unknown names, `derivative` is numeric, `solve x^2 == 2` answers
decimals — ADR-0004's boundary, re-affirmed by the third pass.
Step-by-step solutions, natural-language input, curated/live data:
absent by decision; the first needs the symbolic groundwork, the other
two are product/network boundaries. Exam mode, spreadsheet view,
geometry: surfaces, not language.

## What each gap costs, per frontend

epher's five frontends — desktop (Tauri), PWA/web, TUI, CLI, REPL —
share one engine, one store, one guide (×8 locales), one keypad
language (banks, in PWA and TUI), and one scripts collection with
transcript-checked examples (433). Any language addition lands in all
five at once; the per-frontend cost differs mainly by *how the user
reaches the feature*.

| Gap | Desktop (Tauri) | PWA/web | TUI | CLI | REPL |
|---|---|---|---|---|---|
| 1. input | modal dialog + async suspend of the evaluator; biggest lift | same as desktop, through WASM boundary; blocks the synchronous eval loop | prompt on the entry line; must pause a paste-as-one-paste block mid-run (ADR-0059) | defined EOF behavior needed: checker runs scripts with stdin closed; piped mode reads stdin naturally | natural fit: prompt on the same entry line |
| 2. file I/O | Tauri FS is capability-scoped; OS permission prompts | no arbitrary FS — OPFS at best; path-based I/O forks behavior from the other four | native FS, mechanical | native FS, mechanical; scripts in installers can't hardcode paths (ADR-0058) | native FS, mechanical |
| 3. statement bodies / return | engine only; result pane unchanged | engine only | engine only; paste blocks get longer, unchanged mechanics | engine only; transcripts re-verify | engine only |
| 4. multiple returns / destructuring | engine + parser only | same | same | same; scripts stop indexing `v[1]`/`v[2]` | same |
| 5. string library + escapes | engine; result pane already renders `print` lines | engine; keypad needs a home for new functions (bank extension, ADR-0055) | engine; same keypad banks (ADR-0060) | engine; checker verifies new string scripts' transcripts | engine; autocomplete/F1 lists grow in all five entry lines |
| 6. break/continue | engine only; runaway guard stays the backstop | engine only | engine only | engine only | engine only |
| 7. reserve/warn on `i` | language + store decision; migration for stores holding `i = 5` | same store | same store | same store | same store |

Cross-cutting, for every gap regardless of frontend: the guide grows in
all eight locales; every new function needs a key in the keypad banks
(every guide function is on a key — the PWA and TUI keypads are the two
places keys live); autocomplete descriptions and F1 help in all five
entry lines; any new example scripts join the transcript checker and
need round printed floats for cross-platform determinism.

## Reading the table

- Gaps 3, 4, 6 are pure engine work — they touch all frontends only in
  the trivial sense that the grammar grows. No frontend has an
  architectural stake. These are the cheap removes.
- Gap 5 is engine work plus one product decision (where string
  functions live on the keypad) and the usual ×8 guide/checker tax.
- Gap 7 is a store-policy decision more than a parser one; the fix is
  small but it is the only gap where existing user stores need a
  migration thought.
- Gaps 1 and 2 are the expensive ones, and they are expensive for
  exactly the frontends that make epher unusual. `input` breaks the
  synchronous evaluator model in web/desktop and the paste-as-one-block
  model in the TUI, and it collides with the transcript-checker
  guarantee (deterministic, non-interactive runs). Path-based file I/O
  forks immediately across a sandboxed PWA and four native frontends.
  Both gaps have an epher-shaped alternative worth considering before
  any work: store-scoped named datasets (read/write tables in the
  shared store, the way `save`/`load` carry definitions) instead of
  `input()` and file paths — that keeps one grammar, one store, five
  identical frontends, and the checker guarantee intact.
- If the goal is only to move the programming row from partial to full
  in the matrix's terms, input and I/O are precisely what TI-Basic,
  NumWorks Python, PPL, and Octave have that epher lacks. The matrix
  can also honestly move the other way: re-scoring "programming
  surface" to credit epher's store-shared, transcript-verified,
  keypad-complete scripting is defensible, but the third pass's
  framing (input/IO are the named residue) is the conservative read.

## Provenance

Probes run 2026-09-17 against the staging build (`03cc614`), clean
`EPHER_STORE_DIR`, one-shot mode: `def` bodies, recursion, `if/then/else`
expressions, `{a,b} =` parse error, string escape/compare/library
errors (verbatim: "strings only support + (concatenation)"), `input`/
`open`/`upper`/`substr`/`split`/`format`/`find` unknown, `break` parse
error, `i = 5` → `3+4i` = `23`. Comparison rows from the third pass
(2026-09-06), whose app columns were primary-source-checked there.
