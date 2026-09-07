# ADR-0046: units with conversion — quantities, prefixes, and dimension checking

- Status: accepted
- Date: 2026-09-02
- Roadmap: feature-gap analysis round 5 (T1.3 units with conversion;
  temperature scales excluded per the report, like SpeedCrunch)

## Context

Six of the nine researched apps ship units with conversion, and both
mobile scientifics users actually carry (HiPER, SpeedCrunch) do. The
expectation shape is precise: quantities (`5 m`, `60 mile/hr`), a
conversion operator (`in` or `->`), SI prefixes, and dimension
checking (`5 m + 3 s` is an error). epher already owns the grammar
mechanism — the ADR-0037 unit suffixes multiply a number by an SI
factor at parse time and produce a plain float, so `3.2 AU` evaluates
to `478713186240`. Generalizing means the suffix must carry its
*dimensions*, and the result becomes a quantity, not a bare number.

## Decision

### Quantities as a value

- New `Value::Quantity { value: f64, dims: Dims, unit: Option<(String, f64)> }`
  (ADR-0005 stays: f64 arithmetic; the dims are the seven SI base
  dimensions `[L, M, T, I, Θ, N, J]` as i8 exponents, and `unit` is an
  optional display unit — the typed spelling plus its SI factor, so
  the display can convert back). A quantity stores its SI value; the
  display unit, when present, only rescales the *display*.
- Every existing suffix becomes a quantity: `3.2 AU` is
  `Quantity { value: 4.7871…e11, dims: [1,0,0,0,0,0,0], unit: Some(("AU", 1.4959…e11)) }`
  and therefore still displays `3.2 AU`. `deg`, `arcmin`, `arcsec`
  are dimensionless (zero dims) and behave exactly like plain numbers
  everywhere, so `sin(30 deg)` still answers `0.5`.
- The unit table grows from the ten astronomy tokens to the standard
  set: SI base and derived units (m, s, g, kg, A, K, mol, cd, Hz, N,
  Pa, J, W, C, V, F, Ω, S, Wb, T, H, lm, lx, Bq, Gy, Sv, ohm), the
  common non-SI units (L, l, t, bar, atm, torr, psi, eV, min, hr, d,
  yr, deg, arcmin, arcsec, mile, yd, ft, inch, nmi, lb, oz, gal, qt,
  pt, mph, knot, AU, pc, ly, Jy), and the SI prefixes on top of any
  table unit (Y Z E P T G M k h da d c m µ u n p f a z y, longest
  prefix first: `km`, `ms`, `µm`, `MPa`, `dam`).
- `h` is deliberately absent (Planck's constant keeps its name;
  hours are `hr`), and so is `in` (reserved for the conversion
  operator; inches are `inch`).

### Dimension rules

- `+` and `-`: the operands must carry the same dims, else a
  dimension error (`5 m + 3 s` errors; `5 m + 3 m` is `8 m`). A plain
  number is dimensionless: adding it to a dimensioned quantity is an
  error too. `*` and `/` compose dims (`5 m * 3 m` is `15 m^2`);
  powers scale them (`(3 m)^2` is `9 m^2`), and a non-whole exponent
  on a dimensioned quantity is an error.
- `== != < <= > >=` compare values when dims match and error on a
  mismatch. `sqrt` and `root` halve/divide the dims when they divide
  evenly (`sqrt(4 m^2)` is `2 m`), else error.
- The transcendental and special functions (trig, log, exp, stats,
  distributions, …) consume the SI value and return a plain number —
  SpeedCrunch's model; only arithmetic and comparisons check
  dimensions.
- Lists stay floats-only (ADR-0044); a quantity in a list literal is
  a type error.

### The conversion operator

- `expr in unit` and `expr -> unit` convert a quantity to the named
  unit (dims must match, else a dimension error) and remember it as
  the display unit: `60 mile/hr in km/hr` answers `96.56064 km/hr`,
  `1 km in m` answers `1000 m`. The unit may carry a whole-number
  power and a compound path: `in m^2`, `in km/hr`. `in` binds loosest
  of the arithmetic operators, so `5 m + 3 m in km` is
  `(5 m + 3 m) in km`; it is a reserved word now.
- The compound spelling `60 mile/hr` parses as a unit chain (the
  divisor must be a unit ident directly after a suffixed number, so
  `x / hr` still means "divide by the variable hr"); `5 m/s^2` is a
  valid chain.
- `deg`-family units are dimensionless, so `30 deg in rad` works and
  `x in deg` on a plain number constructs a quantity, like SpeedCrunch.
- Temperature scales (Celsius, Fahrenheit) are excluded, per the
  report: kelvins are the only temperature unit.

### Display

- A quantity displays its value in the remembered display unit when
  there is one (`60 mile/hr`, `96.56064 km/hr`), and in SI otherwise,
  preferring the exact derived name when the dims match one
  (`15 N`, `3.5 W`), else the composed base form (`m/s^2`, `kg m/s^2`,
  `1/s`). Dimensionless quantities display as plain numbers.

## Consequences

- `3.2 AU` and `sin(30 deg)` keep their guide outputs; the arithmetic
  tests that asserted suffix results as plain floats move to
  quantities (one fence in the guide changes: the SI-value output of
  the AU example becomes `3.2 AU` with a `… in m` companion).
- The parser gains two expression forms (`Unit` and `In`); the
  sampler, calculus, solver, and table cells unwrap quantities to
  their SI values, so graphing and deriving a quantity keeps working.
- The keypad stays frozen: units are typed, and the guide's unit
  table is the discovery surface (the constants browser lists
  constants, not units).
