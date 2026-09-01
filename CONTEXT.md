# epher

epher is a programmable, scriptable calculator: users evaluate Expressions,
define reusable Functions, and write multi-line Scripts. It can graph
Expressions and accept LaTeX as an input form. (Some terms below are still
being sharpened during design.)

## Language

### Core domain

**Expression**:
A piece of mathematics that evaluates to a Value.
_Avoid_: formula, equation, calculation, term

**Value**:
The result of evaluating an Expression.
_Avoid_: result, answer, output, number

**Constant**:
A named, immutable Value defined by the user with `const`; like the built-in
pi, it is visible inside Functions (ADR-0012).
_Avoid_: literal, fixed number, named number

**Function**:
A named, parameterized, reusable computation that returns a Value.
_Avoid_: macro, routine, procedure

**Script**:
A sequence of statements (assignments, Function definitions, control flow)
executed in order. Statements are separated by `;` or newlines, the same
separator everywhere (ADR-0001).
_Avoid_: program, macro, routine

**Graph**:
A visual representation of one or more Expressions (for example, the curve
y = f(x)); the plot overlays curves, regions, and points of interest
(ADR-0014).
_Avoid_: plot, chart, figure

**Trace**:
Reading coordinates off a plotted curve by moving a cursor along it (pointer
or arrow keys).
_Avoid_: hover, cursor readout, inspect

**Point of interest**:
A notable point computed from the plotted curves (a root, a turning point
(maximum/minimum), or an intersection (ADR-0014).
_Avoid_: special point, annotation, marker

**Table of values**:
The x/y rows of an Expression over a range, blank where the expression has
no value (`table` command).
_Avoid_: value list, spreadsheet

**3D surface**:
A `z = f(x, y)` mesh sampled over a square domain (`graph3d` command),
projected to 2D lines in core and drawn by the same renderers as curves
(ADR-0015).
_Avoid_: plot3d, height field, mesh (as the product term)

**Animation (parameter playback)**:
Stepping a user-defined Constant through a bounded range while everything
referencing it re-samples, the play button on a slider (web/desktop) or
the space bar (TUI). Always user-started, one control to pause; reduced
motion degrades it to stepping (ADR-0015).
_Avoid_: movie, animated plot, time series

**Percent**:
The postfix `%` operator: a transparent "divided by 100", baked into the
grammar and blind to surrounding operators - `200 + 10%` is 200.1
(ADR-0042).
_Avoid_: add-on percent, percentage-of

**Suggestion**:
A prefix match offered while typing a name in the web/desktop entry: the
session's own functions, Constants, and variables first, then the builtin
catalog, each carrying its key hint (ADR-0042).
_Avoid_: autocomplete item, prediction, snippet

**Complex value**:
A Value with a real and an imaginary part, spelled `a + bi`; `i` is the
imaginary unit constant, `4i` is a literal, and the real transcendental
family extends to the complex plane (ADR-0043). The principal complex
result replaces the old domain error: `sqrt(-1)` is `i`.
_Avoid_: imaginary number only, complex mode

**Solving**:
The `solve lhs == rhs` statement: every root of a polynomial equation
(Durand-Kerner plus deflation), or numeric sign-change bracketing over
-100..100 for anything else (ADR-0043). The result is a display string
of roots.
_Avoid_: CAS, symbolic manipulation, root finding alone

**Calculus**:
The `derivative(expr, p)` and `integral(expr, a, b)` special forms,
whose first argument stays an unevaluated expression bound to its free
variable (numeric 5-point stencil, adaptive Simpson; ADR-0043).
_Avoid_: symbolic differentiation, antiderivative

**Exact display**:
The rendering of a result as a fraction when a small-denominator
continued-fraction convergent matches it: `1/3` for a third, plain
decimal for `pi` (ADR-0043). A display choice; the value stays float.
_Avoid_: rational arithmetic by default, "show as fraction" conversion

**List**:
A column of floats spelled `{1, 2, 3}` (ADR-0044): arithmetic is
elementwise with scalar broadcast, `list[i]` is the 1-based element,
and the statistics take a list as their one argument. A value, not a
container with methods.
_Avoid_: array, vector, collection

**Data plot**:
A graph-family command that plots lists instead of expressions:
`graph scatter(xs, ys)` (with the least-squares fit line),
`graph histogram(data[, bins])`, `graph boxplot(data)` (ADR-0044).
The core computes the primitives; frontends render, like curves.
_Avoid_: chart, plot type, data visualization

**Statistics**:
The list-taking functions `mean median mode stdev variance range
quartile` plus `linreg(xs, ys)`, whose display string reports the
least-squares line and r (ADR-0044).
_Avoid_: stats module, analysis tools

**Test**:
A hypothesis test or confidence interval as a function over a data
list — `ztest ttest chisq_gof zinterval tinterval` — reporting a
display string (`z = …, p = …`, `(lo, hi)`; ADR-0044).
_Avoid_: wizard, test editor

**Seeded random**:
The generator state in the environment (ADR-0045): `randseed(n)` pins
it, `random()`/`random(a, b)` draw uniform reals, `randint(a, b)` an
inclusive whole number. Same seed, same draws, in every frontend;
interactive sessions seed from the clock.
_Avoid_: PRNG, entropy source

**Constants browser**:
The Help → Constants surface (web dialog, TUI pager; ADR-0045) listing
every builtin constant in the four guide groups (Math, Astronomy,
Physics, Chemistry) with value and hint, inserting the chosen name into
the entry.
_Avoid_: constants dialog, lookup table UI

### Astronomy

**Unit literal**:
A number followed by a unit suffix (`3.2 AU`, `30 deg`, `5 hr`, `60
mile/hr`) becomes a Quantity (ADR-0046): the SI value, the seven
base dimensions, and the typed spelling as display unit. Functions
return counts in natural units; suffixes convert counts to SI
quantities.
_Avoid_: unit type, quantity

**Quantity**:
The value-plus-dimensions result of a unit literal or arithmetic on
one (ADR-0046): `Value::Quantity { value, dims, unit }` with the SI
value stored and the optional display unit only rescaling the
display. `+`/`-`/comparisons require matching dims; `*`/`/`/`^`
compose them; dimensionless quantities (angles) behave like plain
numbers.
_Avoid_: unit value, dimensioned number

**Conversion**:
The `expr in unit` / `expr -> unit` operator (ADR-0046): rescales a
quantity to the named unit (dims must match) and remembers it as the
display unit. Binds loosest of the arithmetic operators.
_Avoid_: unit switch, converter

**Ephemeris**:
A computed position of a Solar System body (Sun, Moon, planet) at a given
time, in a stated coordinate frame.
_Avoid_: almanac, star chart, planetarium

**Implicit relation**:
A graph-family command whose source is an equation in two unknowns
(`graph x^2 + y^2 == 1`; ADR-0048): the core samples the difference
lhs - rhs with marching squares over the square domain and returns
the zero contour as ordinary samples separated by pen-up markers.
_Avoid_: implicit function, relation plot

**Matrix**:
A rows x cols grid of floats from the `[[1, 2], [3, 4]]` literal
(ADR-0049): `+`/`-` elementwise with matching shapes, `*` the matrix
product, `^` the whole-number power, rows indexed as lists
(`M[2][1]`), and the function floor det/inv/transpose/trace/dim/
ref/rref. Linear systems solve through rref on the augmented matrix.
_Avoid_: array, grid, 2D list

### Persistence

**Store**:
The persisted collection of a user's data (Functions, Constants, Scripts,
history, and settings.
_Avoid_: database, save file, cache

**Native Store**:
The Store instance reachable by frontends that have host filesystem access
(the desktop app, CLI, and TUI, three modes of the single `epher` binary,
ADR-0011). Shared across those frontends on a single device.
_Avoid_: local store, disk store

**Bridge**:
The web frontend's storage seam: `Tauri` (the Native Store over IPC inside
the desktop shell) or `None` (the session-only PWA until the Web Store
lands).
_Avoid_: sync, backend, connector

**Web Store**:
The Store instance inside the browser/PWA sandbox, physically separate from
the Native Store but sharing the same logical schema.
_Avoid_: browser storage, cache, local storage


## Style

- No em-dashes (—) anywhere, in any language: use colons, commas,
  parentheses, or separate sentences instead (user rule, 2026-08-27).
  The website, the guides, and the in-app copy must stay free of them.
