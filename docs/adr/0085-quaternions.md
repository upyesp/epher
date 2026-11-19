# ADR-0085: quaternions

- Status: accepted
- Date: 2026-11-18
- Amends: ADR-0043 (the promotion precedent line), ADR-0065 (the
  reserved names list)
- Research: docs/research/quaternion-usage-in-astronomy.md and
  docs/research/quaternion-support-in-calculator-tools.md (both
  2026-11-18, primary sources)

## Context

The goal is first-class quaternion support across every frontend,
framed for astronomers: full Hamilton algebra as the substrate,
rotations as the pitch. Two surveys grounded it.

The nine comparison tools: only three have any quaternion support,
none in the calculator class. Wolfram needs an add-on package with
minimal operations; MATLAB's `quaternion` class needs the Robotics
System Toolbox (R2018a); GNU Octave needs a Forge package. The
established practice among those three: a four-argument functional
constructor, Hamilton scalar-first, display `a + bi + cj + dk`
mirroring complex `a + bi`.

Astronomy software: quaternions are native only in spacecraft
attitude work (SPICE C-kernels, CCSDS AEM/ACM messages, GMAT);
astrometry is matrix-native everywhere (ERFA/SOFA, astropy,
Skyfield, FITS WCS have zero quaternion content). SPICE itself is
Hamilton scalar-first and its tiny core API is q2m/m2q/qxq; the
ops/file world (CCSDS, GMAT) lists the scalar last. Nothing in
epher today produces or consumes a rotation; the astronomy family
is scalar.

Two variants were considered and rejected as harmful: `3j`/`3k`
literal suffixes while bare `j`/`k` stay assignable (this recreates
the exact ADR-0065 defect class: a stored `k = 5` silently turns
`3k` into a quaternion), and operator rotation `q * v` (it hides
the composition convention inside an overload; a wrong convention
yields the inverse rotation, not an error).

## Decision

### The value, the constructor, and the literals

- New `Value::Quaternion { w, x, y, z }`, f64 components, Hamilton
  scalar-first: `quaternion(1, 2, 3, 4)` is 1 + 2i + 3j + 4k
  (SPICE, MATLAB, and Octave order).
- `quaternion(w, x, y, z)` is the constructor, the MATLAB/Octave
  spelling rather than a short `quat`.
- `3j` and `3k` join `4i` as numeric-literal suffix tokens under the
  ADR-0043 rules: they apply to decimal and based literals and are
  not consumed when the letter starts a longer identifier.
- `j` and `k` join `i` as the language's reserved names, with the
  full ADR-0065 treatment: session-level assignment, `const`, and a
  destructuring pattern naming them are refused with an error naming
  the quaternion unit; `restore_bindings` drops stored `j` and `k`
  silently; scoped uses keep working (`for k in ...` stays the
  classic counter, parameters named `j`/`k` live for the call;
  `def j(...)` remains possible).
- Display is the shortest algebraic form `a + bi + cj + dk`: zero
  parts dropped, `3j` and `1 - 2k` spellings, and a pure real
  renders as a plain number (the ADR-0043 complex rule).

### Arithmetic and the promotion tower

- Full quaternion algebra: `+`, `-`, `*`, `/` (division by
  multiplication with the inverse), unary minus.
- `Float` and `Complex` promote into quaternion arithmetic, the
  canonical embedding (so `i * j` answers `k`); `Rational`,
  `Decimal`, `Big`, `Quantity`, `Matrix`, and `List` combinations
  are type errors, ADR-0043's precedent line.
- `exp`, `ln`, `log`, `sqrt` extend to quaternions (exp and log are
  the angular-velocity to rotation map); `abs` is the norm; `^`
  takes quaternion bases and exponents through exp/log. The trig
  family rejects quaternions with a type error.
- `==` and `!=` compare components, same kind only (`q == 3` is a
  type error); ordering refuses, like complex.
- `re(q)` is `w`, `im(q)` is the vector part as a 3-list, `conj(q)`
  is the quaternion conjugate, `abs(q)` the norm; the dedicated part
  functions below remain the readable spelling.

### The rotation family

- Vectors are 3-element lists; angles are radians, and angle
  quantities convert (`qaxis([0, 0, 1], 30 deg)` works); the
  quaternion itself is dimensionless.
- `qw(q)`, `qx(q)`, `qy(q)`, `qz(q)` read the parts.
- `qaxis(axis, angle)` builds a unit quaternion (axis normalized);
  one-argument `qaxis(q)` extracts the axis as a 3-list;
  `qangle(q)` answers the rotation angle in radians.
- `qrot(q, v)` rotates a 3-list by `q v q^-1` and returns a 3-list.
  `q * list` stays a type error: rotation happens by name only,
  never through an operator overload.
- `q2m(q)` and `m2q(M)` convert to and from the 3x3 rotation
  Matrix (SPICE's own names; `M` must be 3x3). `m2q` returns the
  representative with scalar >= 0, the SPICE rule.
- `qslerp(q1, q2, t)` is spherical interpolation on the shortest
  path (`q2` negated when the dot product is negative).
- `qconj`, `qnorm`, `qunit`, `qinv` complete the algebra helpers.
- `qeq2hor(jd, lat, lon)` answers the quaternion that rotates a
  geocentric-equatorial unit vector into the horizon frame `alt`/`az`
  report in (east, north, up): geometric, no refraction, conventions
  inherited rather than reinvented. It agrees with `alt`/`az` for a
  distant object, and by the implementation's own measurement a star's
  or planet's agreement is arcsecond-scale; a nearby body's parallax
  needs its distance, which the rotation cannot know, so the Moon
  differs by up to about a degree and `alt`/`az` stay its spelling.
  It is the one bridge between the astronomy family and rotations, and
  it is defined by consistency with shipped behavior.

### Refusals

Type errors, each deliberate: quaternions as list or matrix
elements; complex components inside quaternions (no biquaternions);
every mixed combination outside the promotion tower; ordering; the
trig family; the `solve` statement; statistics and tests; graphing.
No WCS or frame machinery (matrix-native per the research, and
nothing in epher to anchor it).

### Docs and delivery

- Guide: an astronomy subsection with a self-verifying worked
  example (build a star's equatorial unit vector from `ra`/`decl`,
  rotate with `qeq2hor`, check against `alt`/`az`); examples-page
  entries; language-reference rows for `quaternion` and the q
  family; the features table row reads epher full, Wolfram and
  Octave partial, the rest absent; the eight-language translation
  pipeline as usual.
- Every frontend inherits through core (CLI REPL, TUI, desktop,
  PWA, LSP, and every IDE extension). Grammar work is the suffix
  tokens `3j`/`3k` in the parser, tree-sitter, and the extension
  grammars; no keypad keys; LaTeX input needs nothing new (`i`,
  `j`, `k` are constants).

## Consequences

- Stored sessions lose `j` and `k` bindings silently at load; the
  guide's session-level `k` examples get renamed; users can no
  longer keep a session-level `k` (the spring constant) or `j`
  under those spellings. That cost was accepted explicitly.
- Store serialization extends for the new value; history and `Ans`
  carry quaternions.
- The error surface grows by the refused combinations; the refusal
  list bounds it, and each line is a decision, not an omission.
- epher becomes the only calculator-class tool in the comparison
  with quaternions; the audience is the spacecraft-adjacent niche
  the research identified, and the docs say the element order
  loudly because CCSDS and GMAT files list the scalar last.
- Deferred: biquaternions, quaternion elements in lists and
  matrices, WCS or frame machinery, and CK-style Hermite
  interpolation (`qslerp` covers the pointing-timeline ask).
