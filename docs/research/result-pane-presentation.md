# Result-pane presentation: how long answers and script transcripts render

**Goal:** understand why answers in the Result pane (the pane graphs render in,
ADR-0056) are hard to interpret, survey how other calculator apps present long
answers and script output, and recommend presentation-layer changes. Graphs are
out of scope (agreed fine). Changing the prebuilt scripts to print differently
is explicitly **out of scope** — a future exercise. Everything here is about
what the pane does with the transcript it already receives.

**Method:** the pane and the CLI render the same transcript — every statement's
value, one `= value` line per answer (`Session::submit`/`submit_quiet`, the web
joins with U+001F and splits back into `pane-answer` divs; the CLI prints the
same lines). So all **477** bundled scripts were run through `epher-cli` with
output captured per script (regenerate: `for f in "epher scripts"/**/*.epher;
do epher-cli "$f"; done`). A 40-script sample across all ten fields was read
line by line; the corpus statistics below are computed over all 477. The
cross-app survey in §4 is product knowledge of stable, widely documented UI
behavior (checked against the apps' conventions as of 2026-10, **not**
URL-verified live; confidence marked per row).

---

## 1. What the pane does today

- **Web/desktop (and PWA):** `pane-result` is a flex column; each answer is a
  `<div class="pane-answer">`, `ui-monospace`, `white-space: pre-wrap`,
  **`overflow-wrap: anywhere`**, `font-size: 1.15rem; font-weight: 300`. One
  copy button copies the whole transcript. No other affordance.
- **TUI:** the transcript renders as plain text at the top of the graph area,
  one answer per line; the plot shrinks to the rows left over (ADR-0056
  amendment). Pane width at 80 columns is 32 monospace cells.
- Routing (ADR-0056): a short single answer (≤44 chars wide layout / ≤24
  narrow, no newlines) stays on the answer line under the entry; everything
  else — a script's transcript, a table, a long number — goes to the pane.
- Desktop layout: calculator column fixed at 360px; the pane takes the rest,
  ≈55–85 monospace characters in a typical window.

Every line — setup echo, labeled result, raw list, matrix, timestamp, verdict —
renders in the same style at the same visual weight.

## 2. Corpus statistics (all 477 scripts, 4,189 answer lines)

| measure | value |
|---|---|
| answer lines per script | median **7**, p90 15, max 51 |
| scripts with ≥1 line wider than the 44-char answer-line cap | **290 / 477 (61%)** |
| longest single answer line | **305 chars** (`finance/loans/amortization-schedule`) |
| lines containing a label (`": "`) | 2,335 (56%); **unlabeled: 1,854 (44%)** |
| scripts opening with ≥5 bare unlabeled setup lines | **110 / 477 (23%)**; worst 40 (`astronomy/ephemeris/schlyter-crosscheck`) |
| scripts printing at least one `{…}` list | 201 |
| scripts printing a `{{…}}` matrix | 8 |
| scripts printing an ISO 8601 timestamp | 77 |
| scripts ending in a `: true`/`: false` verdict line | 178 |
| scripts using the `table` statement command | **0** |
| scripts using `print(...)` labels | 475 |

## 3. Output-shape taxonomy (with verbatim examples)

The 40-script sample reduces to eight recurring shapes. Counts are corpus-wide
where given.

**S1. Setup echo — bare input/knob values, unlabeled.** 44% of all lines; the
leading lines of nearly every script are its `const` definitions and helper
scratch, echoed as bare `= value` lines. Users must skip them to find results;
nothing in the rendering distinguishes them.

```
finance/loans/amortization-schedule (first 4 of 7 lines):
= 1000            ← principal knob
= 0.01            ← rate knob
= 12              ← term knob
= 88.8487886783   ← payment
astronomy/stars/messier-tonight: 31 bare lines before the first labeled line
```

**S2. Labeled results — `print("label:", value)`.** The good case, 56% of
lines. But label and value share one monospace style, so nothing stands out,
and the label is the dominant visual mass:

```
= sample mean: 81.3
= sample stdev (n - 1): 9.91127304302
```

**S3. Compound display strings from builtins.** Statistics/regression builtins
return pre-composed strings that pack several results into one line:

```
= one-sample t test of the scores against mu0 (t and p): t = 2.0101, p = 0.0753
= least-squares line, with its correlation r: y = 5.7143*x + 50.5714 (r = 0.9981)
```

**S4. Lists on one line.** 201 scripts. The z-scores example is 208 chars —
in the pane it wraps 3–5 times. The web's `pre-wrap` breaks at the spaces
after commas, so the browser's wrapping is mostly at element boundaries
already; the harm is that continuations start under the `= ` voice with no
hanging indent, and the terminal had no wrap at all, clipping such a line
mid-number at its 32-cell edge.

```
= the standardized scores, average 0 with sd 1: {-0.989081897348, 0.393505701095,
  1.03162305422, -1.83990503485, ...}    ← actually one 208-char line; breaks as
  {-0.989081897348, 0.393505701095, 1.03162305422, -1.83990503485, 0.712564377659...
```

**S5. Matrices as flat text.** 8 scripts; the worst line in the corpus (305
chars) is a 12×4 amortization schedule flattened:

```
= {{1, 10, 78.85, 921.15}, {2, 9.21, 79.64, 841.51}, … {12, 0.88, 87.97, 0}}
```
This is tabular data (payment #, interest, principal, balance) with a natural
grid presentation — rendered as a wall of wrapped digits.

**S6. Script-built text tables.** Astronomy scripts align columns with spaces
inside print strings; `white-space: pre-wrap` preserves the alignment only
until a row exceeds the pane width, and TUI panes (32 cells) hard-wrap them:

```
= Mercury  a 0.38709831  e 0.205636837102  i 7.00339863858
=          L 186.414966303  peri 77.5006274631  node 48.297312545
```

**S7. ISO 8601 timestamps inline** (77 scripts), up to 80+ chars with a label
prefix; mid-token breaking splits `2026-02-25T14:22:19` across lines.

**S8. Verdict lines** (178 scripts): scripts end with a check
(`back in the first row: true`, `halving h halves the error: true`) rendered
identically to every number.

Also observed: values with unit suffixes (`= 12 m`), money without grouping
separators (`= 359247.028874`), and 12-significant-digit defaults everywhere
(`359.247028874 months`) — precise, honest (ADR-0051), but visually loud.

## 4. How other calculator apps present long answers and script output

| app | convention (confidence) | lesson for epher |
|---|---|---|
| **MATLAB / Octave** (high) | Assignments echo *labeled* (`x = 1000`); `;` suppresses echo entirely — machinery is silent by default. Matrices render as aligned grids, right-aligned columns. `format short` shows ~5 sig digits; full precision kept internally. | Echo suppression and label-on-echo are the core machinery/answer distinction; grids for matrices. |
| **Jupyter / IPython** (high) | Output cells: dim `Out[n]:` prompt, bright value. pandas renders DataFrames as HTML grids with header bars and row indices; truncates with `…` plus a shape summary (`10 rows × 4 columns`). NumPy summarizes long arrays (`[0.123, …, 0.456]`). Long text output collapses behind a "show more" control. | Dim the prompt/label, emphasize value; truncate-with-disclosure for long structures; state what was elided. |
| **Julia REPL / R console** (high) | Value shown with dim type annotation; matrices aligned; R's `data.frame` prints header + aligned columns; tibble truncates to 10 rows with "… with 15 more rows". | Aligned structure + explicit truncation notice. |
| **Wolfram \| Alpha / Mathematica** (high) | Results in labeled pods ("Result", "Decimal approximation"); alternate forms behind disclosure; "More digits" expands precision; matrices get `MatrixForm` grids; notebooks style `Out` labels gray and italic. | Labeled result blocks, precision disclosure, grids for structures. |
| **Numi / Soulver** (high) | Every line shows its result under/right of the expression, dim gray, with the *final* result visually dominant; hover reveals alternates (hex, currency). | Hierarchy by color not size; attach answers to lines; emphasize the final answer. |
| **Desmos** (high) | Expression list: each row's value renders inline; tables are a first-class object beside expressions. | Tables deserve structure, not flat text. |
| **TI-Nspire / HP Prime / Casio ClassPad** (high) | Pretty-printed answers: stacked fractions, matrices as bracketed grids; exact/approx toggle per answer; answers kept in a history list with the question. | Structures are shown as structures, not source syntax. |
| **Qalculate!** (high) | One main result plus interpretive extras (exact form, simplified, alternate bases) as sub-lines; matrices as grids. | The answer can carry annotation without changing its value. |
| **SpeedCrunch** (high) | Scrollable history, syntax highlighting, right-aligned answers, digit grouping on display. | Display grouping and alignment are pane concerns, not value concerns. |
| **Windows Calculator / PCalc** (high) | History pairs expression and answer; answer emphasized, working lines smaller. | Transcript lines can have two visual tiers without behavior change. |

**Convergent conventions** across all of them:
1. Machinery is visually distinct from (or absent from) the answer.
2. Labels/prompts are dim; values are emphasized.
3. Tabular structures render as aligned grids with headers, never flat text.
4. Long output truncates with an explicit disclosure control.
5. Display precision is decoupled from internal precision.
6. The final answer gets the most visual weight.

epher's pane implements none of these six today: one flat monospace style,
character-level wrapping, no truncation, no hierarchy.

## 5. Recommendations

Presentation-layer only — the transcript text, the scripts, and copy behavior
are unchanged; the pane re-renders what it receives. Ordered by impact on the
corpus evidence.

**R1 — Two-tier transcript: dim the setup echo. (fixes 110 scripts' worst
fault)** Lines without a `": "` label are bare statement echoes (S1); render
them ~50% opacity / one size down, and label- or `print`-derived lines at full
weight. This is MATLAB's machinery/answer distinction and every REPL's dim
prompt. Caveat found in source: `print` is a builtin that *returns a string
value* (`crates/core/src/lib.rs`), so the transcript today carries no
structural print marker — a renderer must either sniff the text (trivial:
labeled lines contain `": "`, bare echoes are numeric/list values) or the
engine tags print-derived strings when building the transcript (a small
core change that alters no script and no output text; the flag rides the
existing ANSWER_SEP join). Optionally a "show every value" toggle restores
parity with the CLI.

**R2 — Wrap at value boundaries; never break a number mid-digit. (fixes
the terminal's clipping; insures the web)** The terminal pane now wraps at
value boundaries with a hanging indent (tokens wider than any row chunk, so
nothing clips — the old behavior cut a long list at the 32-cell edge
mid-number). The web gains the same boundary wrap points as `<wbr>`
insurance plus the hanging indent, keeping `overflow-wrap: anywhere` only
as the last resort for a single token wider than the pane; timestamps
render as unbreakable tokens. Note (measured after the survey): the web's
`pre-wrap` already broke at the spaces after commas, so the mid-digit
harm was mostly a terminal problem — the original wording here
over-stated the web case; see the amendment in ADR-0071.

**R3 — Render matrices and long lists as aligned blocks. (fixes S4/S5, the
two longest lines in the corpus)** When a line's value is a `{{…}}` matrix,
render it as a right-aligned column grid (Mathematica `MatrixForm` style) —
the 305-char amortization line becomes a 12-row × 4-column table at a glance.
For `{…}` lists longer than the pane width, wrap element-per-line (or N per
line) with hanging indent, tibble-style, optionally eliding the middle beyond
~20 elements with an "n more" disclosure. Keep copy = the original single-line
value.

**R4 — Emphasize the last labeled line as the answer.** Scripts end with
checks (`: true`) or subsidiary values (S8), not necessarily the answer; the
*last labeled* line is the reliable proxy (475/477 scripts print labels). Give
it full weight/larger size; TUI renders it after the transcript, above the
plot. Numi/MATLAB's `ans` convention.

**R5 — Transcript affordances in the pane header:** answer count ("7 answers"
/ "51 answers"), scroll to first labeled line on arrival (or a collapsed
"setup ▸" expander for the S1 block), and per-line copy on hover. Today a
51-line transcript opens scrolled to its least informative 31 lines.

**R6 — Subtle typing for recognizable value shapes:** ISO timestamps rendered
as one unbreakable token (optionally in the UI font), `true`/`false` verdicts
with a check/cross glyph (178 scripts), unit suffixes (`12 m`) dimmed, digit
grouping on display via the existing `DisplayPrefs.separators` (copy stays
ungrouped). All presentational; none change the value.

**R7 — Precision disclosure (option, default off):** a display preference
rendering pane values at ~6 sig digits with full precision on hover/copy
(Wolfram's "more digits", MATLAB's `format short`), layered on ADR-0051 rather
than replacing it. The corpus is full of `359.247028874 months, or
29.9372524062 years` where 6 digits carry all the meaning.

**R8 — Longer term: a first-class table object.** The `table` command already
renders aligned, headered grids (ADR-0044) and zero scripts use it; `print`
already carries labels. A `printtable(...)`/labeled-structure output would let
scripts opt into S6-quality tables — but that is the future script-side
exercise, not this one. The pane work above is independent and useful
immediately.

**Priority:** R1 + R2 are the cheap wins — R1 is a styling change on both
frontends, R2's terminal wrap is the real clipping fix — and together they
address the two most damaging patterns (setup walls, clipped/mid-value
breaks). R3 is the single biggest readability win for the worst outputs.
R4–R6 polish. R7 is opt-in.
R8 is future.

**Accessibility notes:** any dimming must keep ≥4.5:1 contrast for labeled
lines (WCAG 1.4.3 — dimmed setup lines are content, not decoration; treat
them as `small`/`muted` text at AA); the pane stays `role="status"`; any
disclosure control ("n more", "setup ▸") must be a real button, keyboard
operable, and announced; grid rendering must remain selectable/copyable text,
not drawn pixels.

**Out of scope:** changing what the scripts print (labels, rounding, the
`table` command), graph rendering, the answer-line routing rule (ADR-0056),
and copy semantics.
