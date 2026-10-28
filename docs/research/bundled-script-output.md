# Bundled-script output readability: recommendations

**Goal:** follow-up to `result-pane-presentation.md` (ADR-0078 shipped the
pane-side work: tiers, grids, boundary wrap, emphasis, disclosures). That
batch deliberately left the scripts themselves untouched ("R8 … a future
script-authoring exercise"). This doc recommends what to change in the
bundled scripts — with one correction to the earlier plan: the sweep
must start with **correctness**, because the corpus-wide recapture found
shipped errors and wrong numbers the pane work had hidden in plain sight.

**Method:** all 477 scripts re-run through `epher-cli` (byte-identical
transcript to what the result pane receives), per-script captures
measured programmatically; sources inspected for the failure pattern.
Regenerate captures: run each script under `epher scripts/` with the CLI.

## 0. CORRECTED: the scripts are sound; a name collision is the real defect

An early draft of this doc claimed 26 scripts were broken. **That was a
measurement artifact and the claim is withdrawn.** The capture loop ran
one-shot `epher-cli <file>` per script without `EPHER_STORE_DIR`, and
one-shot runs persist session variables between invocations (that is
how `epher "x = 5"` then `epher "x * 2"` works). A leftover `r` from an
earlier script therefore poisoned later `const r` scripts. With a fresh
store per run — exactly what `scripts/check-scripts.mjs` does — **all
477 scripts pass, zero error lines**, and `circle-area` prints its
correct `area: 78.5398163397`.

The underlying behaviour it exposed is real and user-facing, though:

```text
# a session where the user ran r = 3, then pastes circle-area:
error: cannot define constant r: the name is already a variable
= area: 28.2743338823        ← the knob's 5 was ignored; r = 3 was used
= A = C^2 / (4 pi) agrees: true   ← the self-check cannot catch this
```

- **Semantics (ADR-0012, deliberate):** a name is a variable or a
  constant, never both. A pasted script's `const r` is rejected when the
  session already bound `r`.
- **In the pane:** `submit_all` is fail-fast, so the GUI shows only the
  error line and discards the paste's results — safe, but cryptic.
- **In the CLI file path:** errors print and evaluation continues,
  computing the rest of the script from the stale variable — the
  plausible-but-wrong numbers above, with a passing self-check.

**Fixes shipped in this batch (ADR-0079):**

- `error:` / `warning:` lines are their own presentation tier: never
  dimmed as setup, never eligible for the answer emphasis, and rendered
  in a dedicated error style (web `--danger` token at AA on all three
  themes; terminal warm red, bold; night keeps its long-wavelength
  rule). An error whose own text contains `": "` no longer passes for a
  labeled result.
- The collision message now names the fix: "…this session already bound
  it; Clear resets the session, or rename the binding".
- The pane title's count is singular-safe (no "Result · 1 answers").

**Still recommended (script side, cheap):** the 10 scripts that bind a
bare `r =` (e.g. `astronomy/sky/spring-neap-tides.epher`) could use a
more specific name (`radius`, `earthRadius`), shrinking the collision
surface for users who paste several scripts in one session.

## 1. Enabler: make knob bindings silent (language change, ADR required)

1,813 of 4,189 transcript lines (43%) are unlabeled echoes of knob and
intermediate bindings. Even the humblest script opens with them:

```text
geometry/triangles/pythagorean-hypotenuse
= 3                                 ← the legs, echoed
= 4
= hypotenuse of the 3-4-5 triangle: 5
```

Echo is documented behavior (`epher scripts/README.md`: "the CLI runs
every statement in order and prints each statement's value"), so this is
a language-semantics decision, not a script edit. ADR-0078's R1 already
dims these lines in the pane; the recommendation is to stop *emitting*
most of them.

- **Recommended: `const` definitions become silent.** Constants are the
  knob convention (345 scripts use them); a knob is machinery, not an
  answer — exactly what `def` already is (defs contribute nothing today).
  Corpus-wide this removes roughly 1,200–1,400 lines with no information
  loss in the pane (values remain in the script text, and R1 already
  de-emphasized them), and the CLI transcript shrinks to results.
  Trade-off, stated honestly: a CLI user loses the "saw the knobs"
  confirmation in the terminal; the pane keeps a record via the script's
  own labeled lines. The docs/ADR and every script's expected block are
  regenerated mechanically (`check-scripts.mjs --update` + review).
- **Conservative alternative: an explicit silent-binding form** (for
  example a `quiet` statement prefix, `quiet x = …`), adopted by the
  sweep only where the echo is pure noise. No existing transcript
  changes; more per-script churn; the corpus keeps its knob echo
  everywhere else.
- **Not recommended:** silencing plain assignments too. Loop-emitted
  computed values and bare-expression answers flow through the same
  statement shape; a blanket silence would hide real results and break
  the every-answer-visible contract (ADR-0052).

## 2. Label the bare outputs (script sweep, ~50 scripts)

Three measured groups need labels, independent of §1:

- **12 scripts are 100% unlabeled** — their transcripts are walls of
  bare values: all seven trig `identities` scripts (`double-angle-sine`
  prints the test angles, a bare `{true, …}` verdict list, then bare
  identity lines), `quadrant-of-angle`,
  `radians-degrees-conversion`, `is-prime-check`,
  `savings-goal-duration` (two knob echoes and an unlabeled
  goal/months matrix), `ecliptic-position`. The reader cannot tell
  which number is which.
- **32 scripts end on an unlabeled line** — the last thing the user
  reads is a bare value; ADR-0078's R4 emphasizes the *last labeled*
  line, so these scripts forfeit the emphasis.
- **216 scripts contain runs of ≥3 consecutive bare lines** — usually
  loop-emitted data (month tables, per-step values) printed without
  labels; `day-length-year` (twelve bare day-length values, one per
  month), `synodic-table` and friends convert these to
  `print("month 3 day length:", 12.17)` or a matrix row.

Sweep rule: every value a user is meant to read gets a `print("what it
is:", value)` label; anything else is a binding, and under §1 becomes
silent.

## 3. Give matrices a header row (8 scripts)

ADR-0078 renders `{{…}}` matrices as aligned grids, but the columns are
unlabeled. Scripts that print matrices add a header row as the first
row:

```text
= {{month, interest, principal, balance}, {1, 10, 78.85, 921.15}, …}
```

The grid renderer treats it as one row — good enough at a glance, and
`amortization-schedule` (the 305-character wall that motivated R3)
becomes self-describing. Zero engine work.

## 4. Convention: the answer is the last labeled line

ADR-0078's R4 emphasizes the transcript's last labeled line and R5
scrolls to the first. Two follows-ons:

- **Script convention:** close with the answer, not the check. Scripts
  move their `: true` verification lines *above* the final result line
  (or accept the check below the emphasized answer — cosmetic either
  way, but "last labeled = the takeaway" should be written into
  `epher scripts/README.md` so new scripts keep it).
- **Pane amendment (small):** scroll the pane to the *emphasized final
  line* instead of the first labeled line, so a script that ends with
  its answer opens on the answer. One-line change in the shipped
  `use_effect` selector.

## 5. Units as trailing tokens

R6 dims terminal unit tokens (`= 12 m`). Scripts adopt the trailing-unit
style where natural (`print("pendulum period:", t, "s")` → `= 2.01 s`)
instead of embedding units inside label prose (`period (s):`), so the
dimming and the value-boundary wrapping have clean tokens to work with.
Apply opportunistically during the §2 sweep, not as its own batch.

## 6. Deferred / rejected

- **`printtable(...)` builtin:** the R3 grids plus §3 header rows cover
  every matrix-printing script in the corpus; a headed-table builtin
  stays deferred until a script genuinely needs one.
- **Rounding inside scripts:** never — values must stay engine-precision
  (ADR-0051); shortening is the pane's opt-in R7 view, and copy must
  remain verbatim.
- **Splitting compound stat strings** (`t = 2.0101, p = 0.0753`): they
  read fine on one line; no change.

## Rollout

1. **Batch A (now):** the 26 correctness fixes. Ship alone — they are
   bugs, verified per-script against each script's stated formula, with
   reviewed expected-block diffs.
2. **Batch B:** the §1 ADR + silent `const`, expected blocks regenerated
   corpus-wide.
3. **Batch C:** the §2–§5 label/units/header sweep (~50–60 files) and the
   one-line pane scroll amendment.
Every batch validates with `node scripts/check-scripts.mjs` (fresh runs
vs expected blocks) plus the existing core/TUI suites.
