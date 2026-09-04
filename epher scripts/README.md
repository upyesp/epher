# epher scripts

Ready-to-run scripts for the epher calculator, written by the maintainers.
Every file here is a plain `.epher` script: the same format the desktop app
and web app save and open, and the CLI and REPL run. The scripts are also
browsable on the website (Docs > Scripts), which builds its catalog from
this folder, so folder and file names are public: keep them simple.

The repository is organized by field, then by area inside the field:

```text
epher scripts/
  algebra/            equations, polynomials, sequences-series, matrices, complex
  astronomy/          moon, planets, time, sky
  calculus/           limits, derivatives, integrals, models
  finance/            interest, loans, savings, investing
  geometry/           triangles, circles, polygons, solids, coordinates
  number-theory/      primes, divisors, integer-sequences, bases
  probability/        counting, distributions, simulations, games
  statistics/         summary, regression, tests, distributions
  trigonometry/       angles, identities, triangles, waves
  README.md
```

`astronomy/moon/full-moons.epher` is the **reference example**: read it
first, and follow it when you contribute. It shows the language as it
stands today: plain arithmetic and `def` helpers, `const` knobs, the
`for`, `if`, `print` and strings surface (guide 1.7-1.11), recursion,
and the astronomy accessors and constants (guide 1.18). Each
use names the guide chapter that teaches it, so a reader can look any
line up. Its expected-output block shows the whole transcript of a run,
and that block is checked mechanically by `scripts/check-scripts.mjs`.

## Running a script

From a terminal (the CLI runs every statement in order and prints each
statement's value). Every installer ships this whole folder, so on an
installed epher the path is the installed one - per operating system:

```sh
# Debian, Ubuntu, Fedora (deb, rpm)
epher /usr/lib/epher/scripts/astronomy/moon/full-moons.epher

# Windows (PowerShell)
epher "$env:LOCALAPPDATA\epher\scripts\astronomy\moon\full-moons.epher"

# macOS
epher /Applications/epher.app/Contents/Resources/scripts/astronomy/moon/full-moons.epher
```

From a source checkout (contributors running from the repository root
-- an installed epher has no `epher scripts` folder), the path carries
the folder name:

```sh
epher "epher scripts/astronomy/moon/full-moons.epher"
```

In the REPL, `load` takes a file path or a script saved with `save script`:

```text
epher> load /usr/lib/epher/scripts/astronomy/moon/full-moons.epher
```

In the web or desktop app, paste the whole file into the entry and press
Enter (Shift+Enter for new lines). Everything also works line by line in
the TUI. See the guide, chapter 4, for scripts, `save` and `load`.

## The format standard

The language shapes the format: a script file runs **one statement per
line** (`;` joins statements on a line), and every statement's value
shows as epher displays it, exact fractions included. A script file is a
whole program, so block comments may span lines in it. The reference
example turns those facts into a house style:

1. **Header comment first.** A `/* === ... === */` banner block, then
   `name.epher -- one line saying what it does`, the algorithm and its
   published source, an honest accuracy statement, how to run the
   script (the three installed commands, one per operating system),
   and what the file demonstrates.
2. **Knobs at the top.** Every value a user might change is one `const`
   in a clearly marked block right after the header. Nothing to edit
   below it.
3. **Cite your source.** Name the book, paper, or data source in the
   header and name each equation at its section divider.
4. **Section dividers.** A `// ---- label ----` line between sections:
   knob, model, helpers, results.
5. **One job per helper.** Short lowercase `def` names, each with a
   trailing note saying what it is. Helpers are defined before first
   use. A `def` body is one expression on one line; recursion is fine up
   to a few dozen calls, and longer iteration uses `for` or `while`
   (guide 1.8-1.12).
6. **Use the builtins.** epher ships a full function library (guide
   1.13-1.28): do not redefine `mean`, `gcd`, `isprime`, `solve`, or any
   other function the language already has. A script that demonstrates a
   formula states that it is the classroom formula and why the builtin
   is not used (the builtin is a display string, or the method itself is
   the point); when a builtin can do the job, the builtin does the job.
7. **Units in the code.** Write `30 deg`, `1 AU`, `24 hr`, convert with
   `in`, and say the unit in the note when a number is a count.
8. **Results last, labelled.** `print("label:", value)` (guide 1.29)
   puts a readable label on the same line as its value; when a result
   speaks for itself, leave it bare so it keeps epher's native display.
   Order outputs the way a reader wants them; one line, one answer.
   Values that are display strings (solve, linreg, the tests) print as
   they are. Keep printed lines short enough not to wrap at 80 columns.
9. **Show the expected output.** End with a `/* ---- expected output
   ---- ... */` block holding the shipped default's transcript, one line
   per output line, so anyone can check their build and their edit.
   `scripts/check-scripts.mjs` runs every script and compares its
   transcript to that block.
10. **Determinism.** No `now()`; any randomness is seeded with
    `randseed(n)` first (guide 1.23). Everything else must print the
    same transcript on every machine.
11. **Self-check when you can.** If a builtin can cross-check the
    script (like the ephemeris accessors), print the comparison.
12. **Honesty about accuracy.** State what is omitted and when the error
    grows. Never oversell.

## Language quick reference

The guide (chapter 1) is the full reference; this is the subset a script
author needs every day.

**Statements.** One per line; `;` joins them. `def f(x) = expr` defines
 a function (silent), `const name = expr` a knob (prints its value),
`name = expr` assigns (prints its value), and a bare expression prints
its value. Strings hold no escape sequences, so a `"` cannot appear
inside a string. A `def` body sees its parameters, the builtins,
`const` names, and other `def` names; it cannot read a variable set
with `=` (use `const` for a value a helper needs), and it sees only the
`const` names and `def` names that were already defined when the `def`
line ran (define helpers in dependency order, like the reference
example).

**Comments.** `//` or `#` comments to the end of the line; `/* ... */`
is a block comment. The house rules for which style to use:

- more than three consecutive comment lines become one block comment
  (`/*` on the first line, `*/` on the last), the header banner
  included;
- three or fewer consecutive comment lines keep the `//` style, one
  `//` per line;
- a note at the end of a statement line uses `//` after the statement;
- a comment never sits inside a statement (never between the tokens of
  one statement).

In whole-program input (a script file, a pasted program) a block
comment may span lines; in line-oriented entry (typing into the REPL or
TUI, or a piped script) a block comment closes on its own line.

**Values.** Numbers, exact fractions (`1/3` prints as `1/3`, while
`10/4` prints `2.5`), strings (`"..."`, `+` joins, `s[i]` is the i-th
character, 1-based), booleans (`true`/`false`), lists `{1, 2, 3}` (1-based
`d[i]`, elementwise arithmetic, numbers only), matrices `[[1, 2], [3, 4]]`
(`M[r][c]`), complex numbers (`3 + 4i`, `re im arg conj abs`), and
quantities with units (`5 m`, `30 deg`, `1 AU`, `72 km/hr`, converted
with `expr in unit`). Display shows about 12 significant digits and
exact fractions where the decimal would repeat.

**Operators.** `+ - * / ^ ! %`; `%` is postfix percent (`10%` is 0.1);
`mod(a, b)` for remainders (integers only); comparisons `== != < <= > >=`;
logic `and or not`; bitwise `& | xor ~ << >>` with `bits(n)`. Trig works
in radians; `deg(x)` and `rad(x)` convert, and `sin(30 deg)` is 0.5.

**Control flow.** `if cond then a else b` is an expression (the `else`
is required; chain with `else if`). `while cond do stmt` repeats one
statement; `for i in a to b step s do stmt` (or `for x in {..} do stmt`)
repeats and collects each body value into a list, which prints. Both
stop at 100,000 steps. A loop body is one statement, so multi-state
iteration packs its state into a list:

```epher
v = {0, 1}                          // {i, sum}
while v[1] < 100 do v = {v[1] + 1, v[2] + v[1]}
v[2]                                // sum of 1..100
```

Recursion is cleaner for small depths (keep under about 80 calls):

```epher
def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
fib(20)
```

**Built-in functions.** All of these exist; use them rather than
redefining them. The guide gives the examples.

- Math: `sin cos tan asin acos atan atan2 sinh cosh tanh asinh acosh atanh
  deg rad exp ln log log2 logb sqrt cbrt root hypot fact abs floor ceil
  round trunc sign ncr npr gcd lcm mod`
- Numbers: `frac dec big exact scientific engineering grouped`,
  base literals `0b… 0o… 0x…`, spellings `bin oct hex`
- Number theory: `isprime nextprime prevprime factors totient ndivisors
  modpow`
- Data: `len sum product min max mean median mode variance stdev range
  sort quartile(d, k)`, list arithmetic, `linreg quadreg expreg powreg
  logreg`
- Distributions and tests: `normpdf normcdf invnorm tpdf tcdf invt
  chi2pdf chi2cdf invchi2 binompdf binomcdf poissonpdf poissoncdf`,
  `ztest ttest zinterval tinterval chisq_gof anova ttestpaired`
  (the tests print display strings)
- Calculus: `solve lhs == rhs`, `derivative(expr, p)`, `integral(expr, a, b)`
- Finance: `tvm_n tvm_i tvm_pv tvm_pmt tvm_fv npv irr amort
  simple_interest compound_interest` (i is a fraction per period, 0.01
  is 1%; a trailing 0/1 picks end or beginning of period)
- Random: `random() random(a, b) randint(a, b) randn(mu, sigma)
  randseed(n)`
- Astronomy: `jd(y, m, d [, hr]) mjd delta_t lst hms2deg dms2deg
  deg2hms deg2dms`, the body accessors `ra decl dist alt az rise set
  transit mag phase illum diam` (Mercury 1 ... Neptune 8, Pluto 9,
  Sun 10, Moon 11), `kepler airmass dawes dist_mod mag2jy jy2mag`,
  `march_equinox june_solstice september_equinox december_solstice`
- Matrices: `det inv transpose trace dim ref rref`
- Constants: `pi e tau phi`; astronomy `au pc ly c g h h_bar k_b
  sigma_sb m_sun r_sun l_sun m_earth r_earth m_moon r_moon`; physics
  `G gamma q_e ev eps_0 mu_0 z_0 m_e m_p m_n m_u a_0 alpha r_inf mu_b
  n_a faraday r_gas atm wien phi_0 m_P l_P t_P r_e lambda_c mu_n`

## Checking the scripts

`scripts/check-scripts.mjs` runs every `.epher` file with a fresh store
(`EPHER_STORE_DIR`), compares each transcript to its expected-output
block, and reports failures:

```sh
cargo build -p epher-cli && node scripts/check-scripts.mjs
```

A script whose transcript does not match its block fails loudly, so
edits must update the block.

## Contributing

- One script per file, named `lowercase-with-hyphens.epher`; the name
  says what the script computes.
- New themes get a new field folder (or area folder inside one); keep
  the tree two levels deep below the README.
- Keep to the format standard above; the reference example is the bar.
- Run the script before opening a pull request and paste the transcript
  into its expected-output block. Run the checker above.
- Anything with a nontrivial algorithm cites its source; anything with
  an accuracy claim states its error.
- Contributions are licensed MIT, like the repo (see LICENSE).
