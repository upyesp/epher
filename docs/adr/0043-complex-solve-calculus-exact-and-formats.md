# ADR-0043: complex numbers, equation solving, numeric calculus, exact display, and result formats

- Status: accepted
- Date: 2026-08-31
- Roadmap: feature-gap analysis round 2 (core block: T1.1, T1.2, T1.4, T2.5, T2.1)

## Context

Round 1 (ADR-0042) shipped the quick wins. The core block is the
calculator's mathematical depth: complex numbers (the only capability
every one of the nine researched apps has and epher lacks), numeric
equation solving (the most-requested feature after arithmetic), numeric
calculus helpers, exact fraction display, and result notation
controls. The Value type already declares a `Complex` variant; this ADR
puts it to work.

## Decision

### Complex numbers (T1.2)

- `i` is a builtin constant, exactly like `pi`: `i` alone is the
  imaginary unit, shadowable by a user variable or constant of the same
  name (resolution order unchanged).
- `4i` is a single token: a numeric literal with an imaginary suffix,
  parsed as `4 * i` in the AST. The suffix applies to decimal and
  based literals (`0xFFi` works); it is not consumed when the `i` is
  part of a longer identifier (`4it` stays a number followed by a
  name). `i` without a number stays an identifier, so `3 + 4i` is
  `3 + 4i`, `x = 2i` assigns, and `i^2` is `-1`.
- Arithmetic promotes float + complex to complex (add, subtract,
  multiply, divide, power via the principal branch). Rational, decimal,
  and big values do not promote; combining them with a complex is a
  type error.
- The real transcendental family extends mechanically: when given a
  complex argument, `sin cos tan asin acos atan sinh cosh tanh asinh
  acosh atanh exp ln log log2 sqrt cbrt abs` compute in the complex
  plane. When given a real argument outside the real domain, `sqrt ln
  log log2 asin acos atanh acosh` return the principal complex result
  instead of a domain error: `sqrt(-1)` is `i`, `ln(-1)` is `i*pi`,
  `asin(2)` is complex. `cbrt(-8)` stays real (`-2`), like the
  researched calculators.
- New builtins: `re(z)`, `im(z)`, `arg(z)`, `conj(z)`; `abs(z)` is the
  magnitude. On a real argument `re/arg/conj/abs` return the value,
  `im` returns 0.
- Display uses the shortest `a + bi` form (`3+4i`, `3-4i`, `i`, `-i`,
  and `3` when the imaginary part is zero).
- Integer-only builtins (`fact ncr npr gcd lcm mod frac floor ceil
  round trunc sign isprime ...`) reject complex arguments with a type
  error, as do comparisons and `min/max`.

### Equation solving (T1.1)

- A new statement form: `solve <equation>`, where the equation is
  `lhs == rhs` (the `==` comparison). `solve` on anything else is a
  parse/type error.
- The variable solved for: `x` when it appears, otherwise the single
  free variable of the equation; more than one free variable is an
  error, none is an error.
- Polynomial equations (built from `+ - * ^` with non-negative integer
  exponents, constants, and the builtin/user constants, up to degree
  12) get all roots, real and complex, via Durand-Kerner iteration on
  the coefficient vector — numeric root-finding with no CAS (ADR-0004
  spirit preserved). `solve x^2 == -1` prints `x = i, x = -i`.
- Anything else is scanned numerically over -100..100 (2000 samples,
  sign-change brackets, bisection safeguard then Newton polish); poles
  are rejected by a residual check; up to 16 roots are reported. `solve
  sin(x) == 0.5` prints the roots in range.
- The result is a display string (like `factors`): `x = 2, x = 3`;
  near-integer roots print without a decimal point.

### Numeric calculus (T2.1)

- Two special-form builtins whose first argument is a raw expression,
  not an evaluated value — the first lazy arguments in the language:
  - `derivative(expr, p)` differentiates `expr` numerically at `p`
    (5-point central stencil, step 1e-4 * (1+|p|)). The variable is
    the expression's free variable, bound to `p` in a child
    environment, so `derivative(x^2, 3)` is 6 and `derivative(sin(t),
    0)` is 1. A constant expression differentiates to 0; an expression
    in several variables is an error. Because the argument stays an
    expression, `graph derivative(x^3 - x, x)` plots the derivative.
  - `integral(expr, a, b)` integrates numerically (adaptive Simpson,
    tolerance 1e-9 relative, depth-capped). `a == b` is 0; `a > b`
    gives the signed integral. `graph integral(x^2, 0, x)` works the
    same way.
- User-defined functions with these names win over the special forms;
  the keypad is untouched (the names come from autocomplete and the
  guide).

### Exact display (T1.4)

- A new `exact(x)` builtin: continued-fraction rational reconstruction
  with a denominator bound of 1000 and relative tolerance 1e-9. A float
  that reconstructs returns the exact `Rational` (`exact(0.3333333...)`
  is `1/3`); otherwise the value passes through unchanged.
- The interactive frontends gain an "Exact fractions" display toggle,
  default ON: when on, every float result that reconstructs is shown as
  a fraction (`1/3`, `3/10`), including `0.1 + 0.2`. `pi`, `sqrt(2)`,
  and friends stay decimal because no small-denominator convergent is
  good enough. The toggle is display-only; the value stays float
  (ADR-0005 untouched).
- Mixed numbers stay out of scope (the `frac` builtin already makes
  explicit fractions on demand).

### Result formats (T2.5)

- New display verbs: `scientific(x)` (`1.2345e4`), `engineering(x)`
  (SI-style exponents, multiples of 3: `12.345e3`, `500e-3`), and
  `grouped(x)` (thin-space thousands separators, locale-neutral:
  `1 234 567.89`). All three return display strings.
- The interactive frontends gain a "Results" settings group: the exact
  toggle above, a format choice (Auto / Scientific / Engineering), and
  a thousands-separators toggle (default OFF — results stay
  copy-pasteable). The choice lives in the shared store settings
  (`format`, `exact`, `separators`) and in localStorage for the plain
  browser session, mirroring theme/language.
- Keypad: no changes. The digits tab remains frozen per the ADR-0042
  amendment; new functions reach users through autocomplete and the
  guide.

## Consequences

- `sqrt(-1)` and `ln(-1)` change from domain errors to complex results:
  a deliberate, documented behavior change — the headline of T1.2.
- The lazy first arguments of `derivative`/`integral` are a new
  mechanism; user functions with those names still shadow them, and
  free-variable detection ignores constants (pi, e, the catalog, user
  constants) so `derivative(pi * x^2, 2)` is `2*pi*x` numerically.
- Solve's numeric scan misses roots outside -100..100 and roots where
  the function only touches zero; polynomial equations get the full
  root set regardless.
- Durand-Kerner and adaptive Simpson are pure f64 arithmetic: no new
  dependencies, wasm-safe, deterministic.
- The settings group adds three store keys with the existing key-value
  settings mechanism — no schema migration.
