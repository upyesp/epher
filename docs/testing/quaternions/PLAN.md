# Verifying epher's quaternions against third-party tools

A readable, follow-along test document for epher's quaternion support
(ADR-0085, shipped in 0.9.13). You compare epher's answers against three
reputable, free libraries plus your own eyes. Everything you need to obtain
and run the tools is in this document.

Two ways to test:

- **Part A**, the automated cross-check suite: 48 cases, one command. The
  suite already passes 48/48 against the 0.9.13 binary used to write this
  document; rerunning it after any change is the fastest regression check.
- **Parts B and C**, hands-on spot checks you run by hand in the CLI, REPL,
  web app, and TUI, with the exact expected output printed next to each.

Skim section 8 first: it lists the conventions that make epher *look*
different from the tools while being mathematically identical. Every
"disagreement" traced while writing the harness resolved to one of them.

---

## 1. The third-party tools, and how to get them

All three primary tools are free, open source, pip-installable, and widely
used in science and engineering. One optional web tool and one optional
astronomy library round out the set.

| Tool | What it verifies | License | Install |
|---|---|---|---|
| SciPy | axis-angle, rotation matrices, qrot, slerp, qangle | BSD-3 | `pip install scipy` |
| SymPy | exact Hamilton algebra, exp/ln/sqrt/powers, inverse | BSD-3 | `pip install sympy` |
| numpy-quaternion | a second, independent slerp | MIT | `pip install numpy-quaternion` |
| WolframAlpha (optional) | spot checks in a browser | free web service | https://www.wolframalpha.com |
| Skyfield (optional) | independent alt/az for the astronomy bridge | MIT | `pip install skyfield` |

### SciPy

- Homepage and docs: https://scipy.org , API reference:
  https://docs.scipy.org/doc/scipy/reference/spatial.transform.html
- Maintained by the SciPy community; the de-facto standard rotation library
  in scientific Python. The classes used here are
  `scipy.spatial.transform.Rotation` and `scipy.spatial.transform.Slerp`.
- Convention: quaternions are **scalar-last** `(x, y, z, w)`, epher is
  **scalar-first** `(w, x, y, z)`. The harness converts for you; know it if
  you compare by hand.

### SymPy

- Homepage: https://www.sympy.org , quaternion class docs:
  https://docs.sympy.org/latest/modules/algebras.html
- The symbolic mathematics system of the Python world. Its
  `sympy.algebras.Quaternion` implements exact Hamilton algebra with
  scalar-first parts, `exp()`, `log()`, `**`, `.inverse()`, `.conjugate()`,
  `.normalize()`: the same conventions as epher, so it is the cleanest
  reference for the algebra and transcendental groups.

### numpy-quaternion

- Homepage: https://github.com/moble/quaternion (README documents install
  as `pip install numpy-quaternion`), API docs:
  https://quaternion.readthedocs.io
- By Michael Boyle (McGill, formerly Caltech/LIGO); used in
  gravitational-wave research, where rotors are a daily tool. Provides
  `slerp_evaluate` as a second, independent implementation of the
  interpolation to check scipy's against. Its slerp demands **unit**
  rotors; non-unit inputs produce garbage (the harness normalizes first).

### WolframAlpha (optional, no install)

- Open https://www.wolframalpha.com and ask for a quaternion product, for
  example with the query:
  `quaternion multiplication (1+2i+3j+4k)*(0.4+1.1i-2j+3.3k)`
  (if Alpha treats `j` and `k` as plain variables, rephrase using its
  `Quaternion[w, x, y, z]` notation: `Quaternion[1,2,3,4]*Quaternion[0,1,0,0]`).
  The Hamilton product here is `-9 + 19.8i - 3j - 2.4k` (that value is
  SymPy-verified; WolframAlpha is the third opinion, not the reference).
  Wolfram is scalar-first `w + xi + yj + zk` like epher, so no conversion
  is needed for these spot checks.

### Skyfield (optional)

- Homepage: https://rhodesmill.org/skyfield , install `pip install skyfield`.
- By Brandon Rhodes; the modern Python astronomy library. On first use it
  downloads an ephemeris file (`de421.bsp`, about 17 MB) from the Naval
  Observatory mirror. Used in section 7 to recompute the Polaris altitude
  completely independently of epher's astronomy.

---

## 2. Getting epher 0.9.13

**Route A, the released build (simplest).** Install from
https://epher.org (download links) or
https://github.com/upyesp/epher/releases/tag/v0.9.13 (installers for
Windows, macOS, Linux; editor extensions; the `epher` CLI ships inside).
Then check:

```sh
epher --version        # must print: epher 0.9.13
```

**Route B, build from a checkout.**

```sh
cd /Users/pete/Developer/code/epher
export PATH="$HOME/.cargo/bin:$PATH"      # cargo is not on the default PATH
cargo build --release -p epher-cli
target/release/epher-cli --version        # must print: epher 0.9.13
```

**Trap.** Always verify `--version` before trusting a binary. The cargo
binary is `epher-cli`; a plain `target/release/epher` can be a stale copy
left by older builds and has silently "passed" checks before.

**How to run expressions.** Three equivalent ways:

```sh
epher "1 + 2i + 3j + 4k"                 # one-shot
printf 'q = qaxis({0, 0, 1}, 90 deg)\nq\n' | epher -     # a script on stdin
epher repl                               # interactive, prompt: epher>
```

Every statement in **script mode** (the `epher -` form) prints an answer
line prefixed `= `, assignments included. The one-shot form prints the bare
value without the prefix. In the REPL, assignments do not echo; `ans` holds
the previous result. Expected values below are written as script-mode lines.

---

## 3. Setup for Part A (once, about a minute)

```sh
python3 -m venv /tmp/qvenv
/tmp/qvenv/bin/pip install scipy sympy numpy-quaternion
```

That installs the three primary tools into an isolated environment
(verified working with the macOS system Python 3.9; if `pip` warns about an
upgrade it is safe to ignore). `python3 -m venv` needs nothing beyond the
system Python.

---

## 4. Part A: the automated cross-check suite (48 cases)

From the repo checkout:

```sh
/tmp/qvenv/bin/python docs/testing/quaternions/qcheck.py \
    --epher target/release/epher-cli     # or --epher epher for an installed CLI
```

Expected ending: `48 passed, 0 failed of 48`, with a PASS line per case.
A failure prints `FAIL`, the case id, and both sides' values, for example:

```
FAIL  C2  rotation       q2m matches scipy as_matrix
      epher=[...] scipy=[...]
```

Tolerance is 1e-9 relative (epher prints 12 significant digits, so exact
string equality is never the test). Rotation results compare up to the sign
`q <-> -q` (both name the same rotation; signs are folded before comparing).

What each group covers, and against which tool:

| Group | Cases | Reference tool | What it proves |
|---|---|---|---|
| algebra | A1-A16 | SymPy | literals and `quaternion(w,x,y,z)` agree; Hamilton products, reversed order, associativity, conjugate, `abs`, `qunit`, `qinv`, division, `re`/`im`/`qw..qz`, `i*i=-1`, `i*j=k`, `j*k=i`, `k*i=j`, complex promotion, precedence |
| transcendental | B1-B6 | SymPy | `exp(pi*i)=-1`, exp of a general quaternion, `ln` of a unit quaternion, `log` is base-10 (`ln(q)/ln(10)`), `sqrt(q)^2=q` with the branch matching `exp(ln(q)/2)`, integer and negative powers |
| rotation | C1-C9 | SciPy | `qaxis` = `from_rotvec`, `q2m` = `as_matrix`, `m2q` reads a scipy matrix with scalar part >= 0, `m2q(q2m(q))` round-trips, `qrot` = `R @ v`, `qangle` = scipy's magnitude (angle < pi), `qaxis(q)` recovers the axis, `q` and `-q` rotate identically, `q*q` doubles the angle |
| slerp | D1-D6 | SciPy + numpy-quaternion | endpoints, quarter, midpoint, a general pair, the shortest-path rule (270 deg reads as -90), symmetry `slerp(q1,q2,.5) = slerp(q2,q1,.5)` |
| horizon | E1-E2 | hand-rolled formulas + the guide transcript | `qeq2hor` reproduces the guide's printed Sun vector exactly and matches an independent GMST/hour-angle implementation to 5e-4; Polaris over Greenwich lands within a moon-width of the latitude |
| refusals | F1-F8 | the error contract | `k=3`, `j=1`, `i=5` refused with the reserved-name message; `def f(k)` stays legal; `q * {1,0,0}` refused; `q == 3` type error; `sin(quaternion)` refused; `1 m` does not promote |

## 5. Part B: hands-on spot checks by hand

Run these in any of the three ways from section 2. Every expected value
below was captured from the 0.9.13 binary and independently confirmed by
the Part A suite.

| # | Type this | Expected answer |
|---|---|---|
| B1 | `1 + 2i + 3j + 4k` | `1 + 2i + 3j + 4k` |
| B2 | `quaternion(1, 2, 3, 4)` | `1 + 2i + 3j + 4k` (constructor is scalar-first) |
| B3 | `(1 + 2i + 3j + 4k) * quaternion(0, 1, 0, 0)` | `-2 + i + 4j - 3k` (matches SymPy and WolframAlpha) |
| B4 | `i * i` | `-1` |
| B5 | `i * j` | `k` |
| B6 | `exp(pi * i)` | `-1+0.000000000000000122464679915i` (Euler) |
| B7 | `abs(1 + 2i + 3j + 4k)` | `5.47722557505` (= sqrt 30) |
| B8 | `q = qaxis({0, 0, 1}, 90 deg)` then `q` | `0.707106781187 + 0.707106781187k` |
| B9 | `qrot(q, {1, 0, 0})` | `{0.000000000000000222044604925, 1, 0}` (x maps to y) |
| B10 | `deg(qangle(q))` | `90` |
| B11 | `qaxis(q)` | `{0, 0, 1}` |
| B12 | `q2m(q)` | `[[0.000000000000000222044604925, -1, 0], [1, 0.000000000000000222044604925, 0], [0, 0, 1]]` |
| B13 | `ln(q)` | `0.785398163397k` (the half-angle: 45 deg about z) |
| B14 | `qslerp(quaternion(1, 0, 0, 0), q, 0.5)` | `0.923879532511 + 0.382683432365k` (the 45-deg turn) |
| B15 | `m2q(q2m(q))` | `0.707106781187 + 0.707106781187k` (round trip) |
| B16 | `k = 3` | `error: cannot assign to k: that name is the quaternion unit` |

Cross-check any line against the tools: paste the same expression into a
`/tmp/qvenv/bin/python` session (see the conventions in section 8 first),
or into WolframAlpha (scalar-first there too). Example SymPy session:

```python
from sympy.algebras import Quaternion as Q
Q(1,2,3,4) * Q(0,1,0,0)        # -2 + 1*i + 4*j + (-3)*k  : matches B3
```

## 6. Part C: the surfaces the suite cannot see (manual, about 15 minutes)

All production URLs; the same pages exist on the preview at
https://upyesp.github.io/epher-preview/ (mounted under `/epher-preview/`).

- [ ] **Front page chip.** https://epher.org/ has a fourth "try this" chip
  showing the three-line quaternion snippet. Clicking "try this" opens the
  app with the code loaded; it answers `0.707106781187 + 0.707106781187k`,
  then `0.707106781187 + 0.707106781187k`, then
  `0.923879532511 + 0.382683432365k`.
- [ ] **Web result pane.** A quaternion result renders as `w + xi + yj + zk`;
  `q2m(q)` renders the 3x3 matrix legibly; shortened forms work (`exp(pi*i)`
  shows the near-zero imaginary part, not garbage).
- [ ] **In-app guide.** Help -> User guide: the astronomy section's last
  sentence reads "Constants and functions in this section are in the
  reference." No raw `[reference](...)` markdown anywhere.
- [ ] **Website guide link.** On https://epher.org/guide/ (same sentence,
  astronomy section), the link goes to
  https://epher.org/reference/#9-built-in-constants and the anchor scrolls
  to the constants table. Spot-check one non-English locale.
- [ ] **Examples page.** https://epher.org/examples.html shows ex-a22 "A
  slew to the pole star"; loading it into the app reproduces its transcript,
  ending with the altitude `51.1432389219` (about a moon-width below the
  latitude 51.4779, exactly as the example's caption claims).
- [ ] **TUI.** A quaternion literal evaluates; Help -> User guide opens the
  pager and the same degraded link sentence shows there; quaternion and
  matrix results scroll without mangling.
- [ ] **Desktop app.** One smoke calculation (it wraps the web artifact).
- [ ] **Editor grammar (spot-check two clients).** In VS Code and one other
  editor: `1 + 2i + 3j + 4k` highlights the unit suffixes, `qaxis` /
  `qslerp` / `qeq2hor` highlight as built-ins and appear in autocomplete.
- [ ] **Error UX everywhere.** Typing `k = 3` in the web app, TUI, and REPL
  shows the friendly reserved-name message, not a stack trace.

## 7. Part D: optional deeper cross-checks

**WolframAlpha.** Beyond B3: try `exp(pi*i)` (answer `-1`) and
`slerp`-shaped products via two `Quaternion` objects if you want a third
opinion on interpolation-free algebra.

**Skyfield, fully independent altitude check.** Recompute Polaris over
Greenwich without touching epher's astronomy:

```sh
/tmp/qvenv/bin/pip install skyfield
```

```python
from skyfield.api import Star, load
from skyfield.units import angle
ts = load.timescale()
t = ts.utc(2023, 2, 25, 0, 0, 0)
planets = load('de421.bsp')            # downloads ~17 MB on first run
earth = planets['earth']
greenwich = earth + 51.4779, -0.0015   # lat, lon
polaris = Star(ra_hours=(37.9545 / 15), dec_degrees=89.2642)
alt, az, _ = greenwich.at(t).observe(polaris).apparent().altaz()
print(alt.degrees, az.degrees)
```

Skyfield answers an altitude near 51.1 degrees (azimuth near 359.5). It
agrees with epher's `qeq2hor` pipeline to within a few hundredths of a
degree; the residue is Polaris proper motion between the J2000 catalog
position and the date, which the example takes as a documented shortcut.
`altaz()` applies no refraction unless you set pressure, so this compares
epher's geometric promise like for like.

## 8. Read this before judging a "mismatch"

Conventions that make epher look different while being mathematically
identical. All were confirmed while building the harness; none is a bug.

1. **Part order.** epher is scalar-first `(w, x, y, z)` everywhere (SPICE,
   MATLAB, Octave, Wolfram). SciPy is scalar-last `(x, y, z, w)`.
2. **The double cover.** `q` and `-q` are the same rotation. scipy may
   return either representative; compare rotations, not raw components.
3. **Canonical representative.** `m2q` always answers scalar part >= 0.
   `qangle` reports the encoded angle in `0..2pi` (SPICE), so the same
   rotation can read as `t` or `2pi - t`; scipy's `magnitude()` wraps to
   `[0, pi]` instead. For angles below pi they agree exactly.
4. **Literals are additive chains.** `4k * Q` binds like `4x` before the
   `+`: write `(1 + 2i + 3j + 4k) * Q` in products, exactly as you would
   parenthesize `1 + 2x` in algebra.
5. **ln and log differ, on purpose.** `ln` is the natural quaternion
   logarithm (the half-angle map, identical to SymPy's `log`); `log` stays
   base-10, so `log(q) = ln(q) / ln(10)` componentwise (verified to machine
   precision).
6. **Short display forms.** Pure-scalar quaternions print as plain numbers
   (`qslerp(..., 0)` prints `1`) and pure-vector quaternions print without
   a scalar term (`0.35i + 0.35j + 0.35k`).
7. **`t` is taken.** `t` is a pre-bound variable holding the current Julian
   date, so `const t = ...` is refused ("name is already a variable"). Bind
   `jd` instead, as the guide's astronomy section does.
8. **Float noise.** Values like `0.000000000000000222044604925` (2.2e-16)
   in matrix and vector outputs are IEEE rounding, not errors; `cos(45 deg)`
   is not an exact binary fraction.

## 9. Recording results

| Group | Cases | Reference | Date | Result |
|---|---|---|---|---|
| algebra | A1-A16 | SymPy | | |
| transcendental | B1-B6 | SymPy | | |
| rotation | C1-C9 | SciPy | | |
| slerp | D1-D6 | SciPy + numpy-quaternion | | |
| horizon | E1-E2 | independent formulas + guide transcript | | |
| refusals | F1-F8 | error contract | | |
| hands-on | B1-B16 (section 5) | captured values | | |
| manual | section 6 checklist | eyes | | |
| optional | section 7 | WolframAlpha, Skyfield | | |

Library versions: `/tmp/qvenv/bin/pip freeze` after section 3; epher's
version prints at the top of the Part A run.
