# The epher language reference

*Version 0.5.41 (staging). This page is the formal, normative definition
of the epher language: its lexical rules, grammar, types, operators,
statements, built-in constants and functions, units, display rules,
errors, and limits. Every name and every signature here is extracted
from the evaluating engine itself (the `epher-core` crate), not from
prose.*

The [user guide](../guide/en/) teaches the language by example; this
reference defines it exactly. Where the two appear to disagree, this
page wins, and the disagreement is a bug worth
[filing](https://github.com/upyesp/epher/issues).

## Notation

- `code` spells a token, expression, or statement exactly as written.
- *italic* names in the grammar name non-terminals.
- "number" without qualification means a real value; "complex" includes
  reals as the special case with zero imaginary part.
- Signatures write built-in calls the way you type them:
  `hypot(a, b)`.
- Every example in a code block is engine-verified: the `epher` block
  evaluates, the `text` block is what the engine answers.

## Program structure

An epher *program* (a script file, a pasted block, a piped stdin, or a
REPL line) is a sequence of *statements* separated by `;` or by a
newline. Both separators are equivalent; `;` never terminates a
statement, it separates statements, so an empty statement is not an
error and leading, trailing, and doubled separators are allowed.

The value of a program is the value of its last statement that produced
one. A statement that produces no value (for example an `if` without an
`else` whose condition is false, or `break`) is skipped when the
program's value is collected; a program whose statements all produce no
value answers nothing.

Comments come in three kinds:

```epher
# a line comment, from # to end of line
2 + 2      // another line comment, from // to end of line
/* an inline comment */ 4 * 4
/* a block comment may span
   several lines */
```

```text
4
16
```

`#`, `//`, and `/* ... */` may appear between any two tokens. Newlines
inside a block comment do not separate statements; the comment's
closing `*/` is required, and its absence is a parse error.

A statement is one of: an assignment, a constant definition, a function
definition, a destructuring, `if`, `while`, `for`, `solve`, `return`,
`break`, `continue`, or an expression. The full grammar is given in
[The complete grammar](#the-complete-grammar-ebnf).

## Lexical grammar

### Numbers

- Decimal: digits with an optional fractional part (`3`, `3.5`, `.5`)
  and an optional decimal-exponent suffix (`2e3`, `1.5E-3`, `2e+2`).
  The exponent needs at least one digit, so `2e` and `2eggs` tokenize
  as the number `2` followed by a name.
- Based integers (ADR-0022): `0b` binary, `0o` octal, `0x`
  hexadecimal — `0b101`, `0o17`, `0xFF`. Digits after the prefix must
  suit the base. A base prefix changes the spelling, never the value:
  `0xFF` *is* `255`.
- Imaginary suffix (ADR-0043): a number directly followed by `i`, with
  no letter or `_` after the `i`, is one imaginary literal: `4i`,
  `2.5i`, `0xFFi`. `3 + 4i` is a sum of two literals. `4it` is
  *not* imaginary — it is the number `4` followed by the name `it`,
  which the grammar then rejects.
- All numbers are 64-bit binary floats. Integers are exact up to
  2^53; beyond that, spelling a literal keeps at most the 53 most
  significant bits.

### Strings

A string literal is double quotes around any run of characters. Five
escapes exist (ADR-0064): `\n` newline, `\t` tab, `\r` carriage
return, `\\` backslash, `\"` quote. Any other escape is a parse error
that names the mistake, and a string cannot end on a lone backslash.

```epher
"line one\nline two"
"quote: \" backslash: \\"
```

```text
line one
line two
quote: " backslash: \
```

### Names

A name starts with a letter or `_` and continues with letters, digits,
or `_`. Letters are Unicode; `atan2`, `log10`, and `x2` are names.
Names are case-sensitive: `G` and `g` are different constants.

## Names and keywords

The following words are keywords. They cannot be written where a value
is expected, and they are recognized by exact spelling:

```
break   const   continue   def   do   else   end   for   if
in      not     or         and   return   solve   step
then    to      while      xor
```

`true` and `false` are **not** literals: booleans are produced only by
comparisons and predicates (`1 < 2`, `isprime(7)`), and a bare `true`
is an unknown-name error.

One further name is reserved: `i`, the imaginary unit. It cannot be
re-bound by assignment, by `const`, or by a destructuring pattern; see
[Names, scope, and the session store](#names-scope-and-the-session-store).
`i` is not a keyword — it resolves as a built-in constant and can be
shadowed by nothing.

## Types and values

| Type | Written as | Examples |
|---|---|---|
| Float (real number) | 64-bit binary float | `3`, `2.5`, `1e9` |
| Rational (exact fraction) | produced by `frac(n, d)`, `exact(x)` | `frac(1, 3)` |
| Decimal | produced by `dec(x)` | `dec(0.1)` |
| Big (exact integer/decimal) | produced by `big(x)`, bitwise results | `big(2) ^ 100` |
| Complex | imaginary literals, complex results | `3+4i`, `sqrt(-1)` |
| Boolean | comparisons and predicates only | `1 < 2`, `isprime(7)` |
| String | `"..."` | `"hello"` |
| List | `{a, b, c}` — a data column of reals, or a list of strings | `{1, 2, 3}`, `split("a b", " ")` |
| Matrix | `[[row], [row]]` — reals, row-major | `[[1, 2], [3, 4]]` |
| Quantity | number with a unit suffix | `2 m`, `60 mile/hr` |

Notes:

- Lists hold numbers, or strings — not a mix. A complex element is
  rejected at construction.
- Matrices are reals only, stored row-major; matrix functions are in
  the [function index](#built-in-functions).
- A quantity is an SI value plus its seven base dimensions and an
  optional display unit; see [Units](#units).
- `ans` is a normal variable the frontend rebinds after each
  computation; it holds the previous answer.

The answer panel spells values as [Number display](#number-display)
defines.

## Expressions and operators

### Precedence, tightest to loosest

| Level | Operators | Associativity |
|---|---|---|
| Primary | literals, names, calls, `(…)`, `{…}`, `[[…]]` | — |
| Postfix | `!` factorial, `%` percent, `[…]` index | left |
| Power | `^` | right |
| Unary | `-` negate, `~` bit-not | — |
| Multiplicative | `*`, `/`, unit division `60 mile/hr` | left |
| Additive | `+`, `-`, unit conversion `in`, `->` | left |
| Shift | `<<`, `>>` | left |
| Bit-and | `&` | left |
| Bit-or | `\|`, `xor` | left |
| Comparison | `>`, `<`, `>=`, `<=`, `==`, `!=` | non-chaining |
| Not | `not` | — |
| And | `and` | left |
| Or | `or` | left |
| Conditional | `if c then a else b` (expression form) | — |

`-2 ^ 2` is `-4` (unary minus binds looser than the power);
`2 ^ -2` is `0.25` (the exponent may be unary);
`3! ^ 2` is `36`; `d[2] ^ 2` indexes first;
`not x > 3` is `not (x > 3)`;
`5 & 3 == 1` is `(5 & 3) == 1`.

### Arithmetic

`+`, `-`, `*`, `/` work over reals and complex; a complex operand
computes in complex. `/` by zero is a division error. `^` is
right-associative; `0 ^ 0` is `1`.

- List arithmetic is elementwise: two same-length lists combine
  element by element, and a list combined with a scalar maps over the
  list. Different lengths are a type error.
- String `+` concatenates two strings; a string with a non-string is a
  type error (spell the number first with `str` or `fixed`).
- Unit suffixes ride through `+ - * /` with dimension checking; see
  [Units](#units).

### Factorial and percent

`n!` is the factorial of a non-negative whole number. `x%` is exactly
`x / 100` — a transparent suffix, so `200 + 10%` is `200.1`; the
Casio add-on reading (`220`) is deliberately not a grammar rule.

### Indexing

`expr[i]` indexes a list or string, **1-based**; the index is any
expression. A matrix indexed with one expression gives one whole row
as a list: `m[2]` is the second row.

### Comparisons

`==` and `!=` compare any two values of the same kind; `<`, `<=`,
`>`, `>=` order numbers and strings (strings in dictionary/code-point
order). Complex values refuse ordering. Lists and matrices do not
compare. Comparisons return booleans, and comparisons do not chain:
`1 < 2 < 3` is a type error, not `(1 < 2) < 3`.

### Booleans

`and`, `or`, `not` take booleans only and produce booleans. `and` and
`or` short-circuit: the right side is not evaluated when the left
decides. There is no truthiness: numbers are not booleans.

### Bitwise operations

`&`, `|`, `xor`, `~`, `<<`, `>>` take integers (whole floats, exact
Big, integral rationals/decimals) and produce the exact result masked
to the session's word size — a signed two's-complement word of 8, 16,
32, or 64 bits, set with `bits(w)` and defaulting to 64. Right shift
is arithmetic; a negative shift amount reverses the direction.

## Statements

### Assignment: `name = expr`

Evaluates the expression and binds the name in the session. Rebinding
is allowed; binding `i` is refused (see below).

### Constant definition: `const name = expr`

Binds an immutable name. Re-declaring with the *same* value succeeds —
examples get pasted twice; a different value is a
`constant already defined` error. Naming an existing variable is a
`cannot define constant x: the name is already a variable` error.

### Destructuring: `{a, b} = expr` (ADR-0064)

The expression must be a list whose length equals the number of
pattern names; each name binds to its position's element. `_` skips a
position: `{x, _} = {1, 2}` binds only `x`. Names in the pattern
follow assignment rules (`i` refused, constants refused). The pattern
is recognized only when the statement *begins* with `{names} =`; a
`{1, 2}` at the start of a line stays the list expression it always
was.

### Function definition: `def name(params) = expr` and `def name(params) do … end`

Two body forms. `= expr` evaluates the expression in a child
environment seeded with the arguments. `do … end` runs its statements
one after another in that child environment; the last statement's
value is the call's answer, and a body whose statements produce no
value is a named error. Recursion works in both forms. A call carries
its own 100,000-step budget. Parameters shadow session values only
inside the call.

### `return`, `break`, `continue` (ADR-0064)

`return expr` leaves the enclosing function now, with the value;
`return` at top level (outside any function) is a named error.
`break` leaves the innermost enclosing `for` or `while`; `continue`
skips to its next pass. Outside a loop both are named errors. In a
`for` loop, a pass ended by `continue` contributes no value to the
collected list.

### `if` as a statement: `if cond then stmt` / `if cond then stmt else stmt`

The condition must be a boolean. Without `else`, a false condition
produces no value — which is how a `for` loop filters: a pass whose
body produces no value adds nothing to the loop's list.

### `while cond do stmt`

Evaluates the condition (which must be a boolean) and runs the
one-statement body until it is false, `break` fires, or the step
budget runs out. A `while` loop produces no value.

### `for name in iterable do stmt` (ADR-0054, scoped per ADR-0063)

Two iterable forms: a range `start to end` (optionally `step s`), or
any expression evaluating to a list. The variable is bound per pass
and **removed afterwards**: the loop never leaves its variable in the
session, and a binding that existed before the loop is restored after
it. The loop's value is the list of the body's per-pass values.

### `solve lhs == rhs` (ADR-0043)

Numeric equation solving — no CAS. The equation must use `==`. The
unknown is `x` when `x` appears, otherwise the single other name;
constants (built-in and user) are parameters, never unknowns. Real
roots in a wide search window are reported, as a display string:
`solve x^2 == 9` answers `x = -3, x = 3`.

### Expression statements

Any expression, typically a call. Its value, when there is one,
becomes the statement's value.

## Names, scope, and the session store

Name resolution tries, in order: session variables, then session
constants, then built-in constants, then built-in functions (a call),
then the error `unknown name`. A user function or constant shadows a
built-in of the same name.

- Session variables persist for the session and — in interactive
  frontends — across sessions and frontends of the same installation
  through the shared store (the `.epher` folder; the PWA keeps the
  same shape in browser storage).
- **The imaginary unit is reserved.** `i = 5`, `const i = …`, and a
  destructuring pattern naming `i` are all refused with
  `cannot assign to i: that name is the imaginary unit`, so no
  session, store, or frontend can shadow `3+4i`. A stored `i` from an
  older release is dropped silently when the store loads.
- A `for` loop's variable exists only for the loop: it is removed
  afterwards, and a prior binding is restored (nothing a loop does
  reaches the store).
- A `def`'s parameters and body live in a child environment: they
  shadow the session inside the call and cannot leak out. Assignments
  inside a function body are local to that call.
- `save name` asks the interactive frontend to persist a function or
  constant into the store; saving a *script* is a frontend command,
  not language (see
  [Beyond the grammar](#beyond-the-grammar-interactive-commands)).

## Built-in constants

Values are SI throughout. A name's value is fixed by the engine; a
user `const` of the same name shadows it by the resolution order.

### Mathematics

| Name | Value | Meaning |
|---|---|---|
| `pi` | `3.141592653589793` | the circle constant |
| `tau` | `6.283185307179586` | two pi |
| `e` | `2.718281828459045` | Euler's number, the natural-log base |
| `phi` | `1.618033988749895` | the golden ratio, (1+√5)/2 |
| `gamma` | `0.5772156649015329` | the Euler–Mascheroni constant |
| `i` | `0+1i (the imaginary unit)` | the imaginary unit — the one reserved name (assignment refuses it) |

### Astronomy

| Name | Value | Meaning |
|---|---|---|
| `au` | `1.495978707e11` | astronomical unit (m) |
| `pc` | `3.0856775814913673e16` | parsec (m) |
| `ly` | `9.4607304725808e15` | light year (m) |
| `c` | `2.99792458e8` | speed of light in vacuum (m/s) |
| `g` | `9.80665` | standard gravity (m/s²) |
| `l_sun` | `3.828e26` | solar luminosity (W) |
| `m_sun` | `1.98847e30` | solar mass (kg) |
| `r_sun` | `6.957e8` | solar radius (m) |
| `m_earth` | `5.9722e24` | Earth mass (kg) |
| `r_earth` | `6.371e6` | Earth mean radius (m) |
| `m_moon` | `7.342e22` | lunar mass (kg) |
| `r_moon` | `1.7374e6` | lunar mean radius (m) |
| `h` | `6.62607015e-34` | Planck constant (J·s) |
| `h_bar` | `h/tau = 1.0545718176461565e-34` | reduced Planck constant (J·s) |
| `k_b` | `1.380649e-23` | Boltzmann constant (J/K) |

### Physics

| Name | Value | Meaning |
|---|---|---|
| `G` | `6.6743e-11` | gravitational constant (m³/(kg·s²)) |
| `m_e` | `9.1093837139e-31` | electron mass (kg) |
| `m_p` | `1.67262192595e-27` | proton mass (kg) |
| `m_n` | `1.67492750056e-27` | neutron mass (kg) |
| `m_u` | `1.66053906892e-27` | atomic mass unit (kg) |
| `q_e` | `1.602176634e-19` | elementary charge (C) |
| `ev` | `1.602176634e-19` | electronvolt (J) |
| `eps_0` | `8.8541878128e-12` | vacuum permittivity (F/m) |
| `mu_0` | `1.25663706212e-6` | vacuum permeability (H/m) |
| `z_0` | `376.730313668` | vacuum impedance (Ω) |
| `mu_b` | `9.2740100783e-24` | Bohr magneton (J/T) |
| `mu_n` | `5.050783699e-27` | nuclear magneton (J/T) |
| `phi_0` | `2.067833848e-15` | magnetic flux quantum (Wb) |
| `a_0` | `5.29177210544e-11` | Bohr radius (m) |
| `alpha` | `7.2973525643e-3` | fine-structure constant |
| `r_inf` | `10973731.56816` | Rydberg constant (1/m) |
| `r_e` | `2.8179403205e-15` | classical electron radius (m) |
| `lambda_c` | `2.42631023867e-12` | electron Compton wavelength (m) |
| `l_P` | `1.616255e-35` | Planck length (m) |
| `m_P` | `2.176434e-8` | Planck mass (kg) |
| `t_P` | `5.391247e-44` | Planck time (s) |

### Chemistry and thermodynamics

| Name | Value | Meaning |
|---|---|---|
| `atm` | `101325.0` | standard atmosphere (Pa) |
| `n_a` | `6.02214076e23` | Avogadro constant (1/mol) |
| `faraday` | `96485.33212` | Faraday constant (C/mol) |
| `r_gas` | `8.31446261815324` | molar gas constant (J/(mol·K)) |
| `sigma_sb` | `5.670374419e-8` | Stefan–Boltzmann constant (W/(m²·K⁴)) |
| `wien` | `2.897771955e-3` | Wien displacement constant (m·K) |

The imaginary unit's value is the complex number with real part 0 and
imaginary part 1; its literal spellings `i` and `4i` tokenize as
[literals](#lexical-grammar).

## Built-in functions

Every callable the engine knows, grouped by domain. Argument shapes are
normative: `x` is a real number, `z` any number (complex allowed),
`n` a whole number, `s` a string, `L` a list. A call with the wrong
count or kinds is a type error that names the function, e.g.
`rad expects 1 number, got 1 argument(s)`.

### Angles and logarithms

| Function | Answers |
|---|---|
| `deg(x)` | `x` radians to degrees |
| `rad(x)` | `x` degrees to radians |
| `ln(z)` | natural logarithm; `ln(-1)` is `i*pi` |
| `log(z)` | base-10 logarithm (calculator convention) |
| `log2(z)` | base-2 logarithm |
| `logb(base, x)` | logarithm of `x` to the given `base` |
| `exp(z)` | e to the `x` |

### Powers, roots, rounding

| Function | Answers |
|---|---|
| `sqrt(q)` | square root; negative reals fall back to complex; quantities need even dimensions |
| `cbrt(z)` | real cube root (principal complex root for complex) |
| `root(n, x)` | real `n`-th root; odd roots of negatives are negative |
| `abs(z)` | magnitude — for complex, distance from the origin |
| `floor(x)`, `ceil(x)`, `trunc(x)` | round down, up, toward zero |
| `round(x)` | half away from zero, like a calculator |
| `sign(x)` | `-1`, `0`, or `1` |
| `hypot(a, b)` | `sqrt(a^2 + b^2)` without the overflow |
| `min(x…)`, `max(x…)` | extreme of any number of reals |
| `gcd(a, b)`, `lcm(a, b)` | whole-number gcd and lcm |

### Trigonometry (complex where the domain needs it)

| Function | Answers |
|---|---|
| `sin(z)`, `cos(z)`, `tan(z)` | circular functions |
| `asin(z)`, `acos(z)`, `atan(z)` | inverse circular functions; real `asin(2)` falls back to complex |
| `atan2(y, x)` | angle of the point `(x, y)`, quadrant-correct |
| `sinh(z)`, `cosh(z)`, `tanh(z)` | hyperbolic functions |
| `asinh(z)`, `acosh(z)`, `atanh(z)` | inverse hyperbolic functions |

### Complex parts

| Function | Answers |
|---|---|
| `re(z)`, `im(z)` | real and imaginary part (0 for a real) |
| `arg(z)` | principal argument |
| `conj(z)` | complex conjugate (a real passes through) |

### Exact and display spellings

| Function | Answers |
|---|---|
| `exact(x)` | the rational behind a float, when one agrees through all twelve displayed digits (`exact(0.3333333333333333)` is `1/3`); irrationals pass through |
| `frac(n, d)` | the exact fraction `n/d` |
| `bin(n)`, `oct(n)`, `hex(n)` | spelling with prefix — `bin(10)` is `0b1010`; negatives keep the sign on the prefix |
| `dec(x)` | the value as an exact decimal |
| `big(x)` | the value as an exact (big) integer |
| `scientific(x)` | the value in scientific notation, as text |
| `engineering(x)` | the value in engineering notation (exponent a multiple of 3), as text |
| `grouped(x)` | the value with thin-space thousands grouping, as text |
| `fixed(x, d)` | the value with exactly `d` decimals (0–15), as text |
| `str(v)` | one value spelled the way the answer panel spells it |
| `print(v…)` | the arguments joined with spaces, as one string |

### Number theory

Integers the float type reaches exactly (`|n| < 2^53`).

| Function | Answers |
|---|---|
| `fact(n)` | `n!` |
| `ncr(n, r)`, `npr(n, r)` | combinations and permutations |
| `isprime(n)` | whether `n` is prime (deterministic Miller–Rabin) |
| `nextprime(n)`, `prevprime(n)` | nearest prime above/below |
| `modpow(b, e, m)` | `b` to the `e` modulo `m`, exact via big integers |
| `mod(a, b)` | truncated remainder — the sign of the dividend |
| `totient(n)` | Euler's totient |
| `ndivisors(n)` | how many whole numbers divide `n` |
| `factors(n)` | the prime factorization, as text: `factors(360)` is `2^3 * 3^2 * 5` |

### Statistics and probability

`L` is a list of reals; variance is population variance (divide by `n`).

| Function | Answers |
|---|---|
| `sum(L)`, `product(L)` | total and product (lists or bare numbers) |
| `mean(L)`, `median(L)`, `mode(L)` | centers |
| `variance(L)`, `stdev(L)` | spread |
| `range(L)` | max minus min |
| `quartile(L, k)` | quartile `k` in `1..3` |
| `sort(L)` | the list, ascending |
| `linreg(xs, ys)` | least-squares line and `r`, as text |
| `quadreg(xs, ys)` | quadratic fit and `r`, as text |
| `expreg(xs, ys)` | exponential fit and `r`, as text |
| `powreg(xs, ys)` | power fit and `r`, as text |
| `logreg(xs, ys)` | logarithmic fit and `r`, as text |
| `normpdf(x)`, `normcdf(x)`, `invnorm(p)` | standard normal (1 argument) or general (`x, mu, sigma`) |
| `tpdf(x, df)`, `tcdf(x, df)`, `invt(p, df)` | Student's t |
| `chi2pdf(x, df)`, `chi2cdf(x, df)`, `invchi2(p, df)` | chi-square |
| `binompdf(k, n, p)`, `binomcdf(k, n, p)` | binomial |
| `poissonpdf(k, lambda)`, `poissoncdf(k, lambda)` | Poisson |
| `ztest(L, mu0, sigma)` | one-sample z test, `z = …, p = …` |
| `ttest(L, mu0)` | one-sample t test |
| `ttestpaired(A, B)` | paired t on the differences |
| `anova(G1, G2, …)` | one-way ANOVA, `F = …, p = …` |
| `zinterval(L, sigma, level)` | confidence interval, as text |
| `tinterval(L, level)` | confidence interval, as text |
| `chisq_gof(observed, expected)` | chi-square goodness of fit |

### Lists, strings, matrices

| Function | Answers |
|---|---|
| `len(v)` | length of a list, or character count of a string |
| `upper(s)`, `lower(s)`, `trim(s)` | case and whitespace |
| `substr(s, start[, len])` | 1-based slice; no `len` means the rest |
| `split(s, sep)` | the parts, as a list of strings (non-empty `sep`) |
| `join(L, sep)` | elements spelled as `print` would, joined |
| `find(s, sub)` | 1-based position of `sub`, or 0 |
| `replace(s, old, new)` | every `old` replaced |
| `det(M)`, `trace(M)` | square-matrix determinant and trace |
| `inv(M)` | square-matrix inverse |
| `transpose(M)` | rows and columns swapped |
| `dim(M)` | `{rows, cols}` |
| `ref(M)`, `rref(M)` | row echelon and reduced row echelon form |

### Calculus (numeric)

| Function | Answers |
|---|---|
| `derivative(expr, at)` | 5-point central difference of `expr` at `at`; `expr` stays symbolic, and the unknown is `x` when `x` appears, else the single free name |
| `integral(expr, a, b)` | definite integral of `expr` from `a` to `b` |

### Randomness (seeded, reproducible)

| Function | Answers |
|---|---|
| `random()` | uniform draw in `[0, 1)` |
| `random(a, b)` | uniform draw in `[a, b)` |
| `randint(a, b)` | whole number in the closed range `[a, b]` |
| `randn()` | standard normal draw (Box–Muller) |
| `randseed(n)` | re-seed the generator and report `n`; a fresh session seeds from the clock, `randseed` makes draws reproducible |

### Word size

| Function | Answers |
|---|---|
| `bits()` | the current bitwise word size (8, 16, 32, or 64) |
| `bits(w)` | set the word size to `w` and report it |

### Finance

Sign convention: money you pay out is negative. `r` is the rate per
period.

| Function | Answers |
|---|---|
| `tvm_n(r, pv, pmt, fv)` | periods needed |
| `tvm_i(n, pv, pmt, fv)` | periodic rate that makes the amounts balance |
| `tvm_pv(n, r, pmt, fv)` | present value |
| `tvm_pmt(n, r, pv, fv)` | payment per period |
| `tvm_fv(n, r, pv, pmt)` | future value |
| `npv(r, flows)` | net present value of a flow list |
| `irr(flows)` | internal rate of return of a flow list |
| `amort(p, r, n, k)` | balance after period `k` of an `n`-period loan |
| `simple_interest(p, r, t)` | `p*r*t` |
| `compound_interest(p, r, n)` | interest earned: `p(1+r)^n − p` |

### Astronomy and time

Bodies are whole numbers: Mercury 1 … Neptune 8, Pluto 9, Sun 10,
Moon 11. `jd` is a Julian Date. Observer functions take terrestrial
latitude and longitude in degrees.

| Function | Answers |
|---|---|
| `jd(y, m, d[, hr])` | Julian Date of a calendar date (Gregorian from 1582-10-15, Julian before) |
| `mjd(y, m, d[, hr])` | Modified Julian Date |
| `date(jd)` | `{y, m, d}` |
| `time(jd)` | `{h, min, s}` |
| `iso(jd)` | the instant as `YYYY-MM-DDTHH:MM:SS` text |
| `now()` | the current Julian Date |
| `hms2deg(h, m, s)`, `dms2deg(d, m, s)` | sexagesimal to decimal degrees |
| `deg2hms(x)`, `deg2dms(x)` | decimal degrees to sexagesimal text |
| `lst(jd, lon)` | local sidereal time in **hours** at longitude `lon` |
| `delta_t(jd)` | TT − UT1 in seconds |
| `kepler(M, e)` | Kepler's equation: eccentric anomaly for mean anomaly `M` |
| `ra(body, jd)`, `decl(body, jd)` | geocentric equatorial coordinates, degrees |
| `dist(body, jd)` | distance, km |
| `mag(body, jd)` | apparent magnitude |
| `phase(body, jd)` | phase angle, degrees |
| `illum(body, jd)` | illuminated fraction |
| `diam(body, jd)` | angular diameter, arcseconds |
| `alt(body, jd, lat, lon)`, `az(body, jd, lat, lon)` | topocentric horizon coordinates, degrees |
| `airmass(body, jd, lat, lon)` | relative airmass along the line of sight |
| `rise(body, jd, lat, lon)`, `set(…)`, `transit(…)` | the event's JD for the local day containing `jd`; a named error when the event never happens that day |
| `dawes(aperture)` | Dawes limit in arcseconds for an aperture in mm |
| `dist_mod(mu)` | distance modulus `mu` to distance in parsecs |
| `mag2jy(m)`, `jy2mag(f)` | Jansky ↔ magnitude (AB, 3631 Jy zero point) |
| `satx(moon, jd)`, `saty(…)`, `satz(…)` | a Jupiter satellite's offset from Jupiter, in Jupiter radii (moon 1..4 = Io..Callisto) |
| `satsep(m1, m2, jd)` | separation of two satellites, in Jupiter radii |
| `satphen(moon, jd)` | the satellite's next phenomenon (occultation, transit, eclipse, shadow), as text |
| `march_equinox(y)`, `june_solstice(y)`, `september_equinox(y)`, `december_solstice(y)` | the JD of the Earth's season mark in year `y` |

`i` the loop variable and `i` the imaginary unit coexist inside a
`for` loop exactly because the loop scopes its variable; see
[Statements](#statements).

## Units

A number directly followed by a unit name is a **quantity** (ADR-0046):
the number times the unit's SI factor, carrying its dimensions.
`2 m` stores 2 with length dimensions; `60 mile/hr` stores
26.8224 m/s. Units may carry a whole-number power (`2 m^2`, powers to
±127) and a `/` may continue with another unit (`5 m/s^2`) — that is
unit syntax, not division, whenever a unit name follows. An SI prefix
may start a unit: `30 cm`, `5 kHz`, `3 ns`.

`x in unit` (or `x -> unit`) converts: `5 km in mile` answers
`3.10685596119 mile`, and the display unit sticks. Arithmetic checks
dimensions: adding `2 m` to `30 cm` gives `2.3 m`; adding a metre to
a second is a dimension error.

### Prefixes

The 21 SI prefixes, as spellings before a unit: `da` (×10¹), `h` (×10²),
`k` (×10³), `M` (×10⁶), `G` (×10⁹), `T` (×10¹²), `P` (×10¹⁵),
`E` (×10¹⁸), `Z` (×10²¹), `Y` (×10²⁴), `d` (×10⁻¹), `c` (×10⁻²),
`m` (×10⁻³), `µ` or `u` (×10⁻⁶), `n` (×10⁻⁹), `p` (×10⁻¹²),
`f` (×10⁻¹⁵), `a` (×10⁻¹⁸), `z` (×10⁻²¹), `y` (×10⁻²⁴).

### Base and derived units

| Unit | SI factor | Dimensions |
|---|---|---|
| `m` | 1 | length |
| `s` | 1 | time |
| `g` | 1e-3 | mass |
| `kg` | 1 | mass |
| `A` | 1 | current |
| `K` | 1 | temperature |
| `mol` | 1 | amount |
| `cd` | 1 | luminous intensity |
| `Hz` | 1 | 1/time |
| `N` | 1 | kg·m/s² |
| `Pa` | 1 | kg/(m·s²) |
| `J` | 1 | kg·m²/s² |
| `W` | 1 | kg·m²/s³ |
| `C` | 1 | A·s |
| `V` | 1 | kg·m²/(A·s³) |
| `F` | 1 | A²·s⁴/(kg·m²) |
| `ohm`, `Ohm` | 1 | kg·m²/(A²·s³) |
| `S` | 1 | A²·s³/(kg·m²) |
| `Wb` | 1 | kg·m²/(A·s²) |
| `T` | 1 | kg/(A·s²) |
| `H` | 1 | kg·m²/(A²·s²) |
| `lm` | 1 | cd·sr (cd) |
| `lx` | 1 | cd/m² |
| `Bq` | 1 | 1/s |
| `Gy` | 1 | m²/s² |
| `Sv` | 1 | m²/s² |
| `L`, `l` | 1e-3 | m³ |
| `t` | 1e3 | mass |
| `bar` | 1e5 | kg/(m·s²) |
| `atm` | 101325 | kg/(m·s²) |
| `torr` | 133.32236842105263 | kg/(m·s²) |
| `psi` | 6894.757293168361 | kg/(m·s²) |
| `eV` | 1.602176634e-19 | kg·m²/s² |
| `min` | 60 | time |
| `hr` | 3600 | time |
| `d` | 86400 | time |
| `yr` | 31557600 | time |
| `rad` | 1 | angle (dimensionless) |
| `deg` | pi/180 | angle (dimensionless) |
| `arcmin` | pi/10800 | angle (dimensionless) |
| `arcsec` | pi/648000 | angle (dimensionless) |
| `mile` | 1609.344 | length |
| `yd` | 0.9144 | length |
| `ft` | 0.3048 | length |
| `inch` | 0.0254 | length |
| `nmi` | 1852 | length |
| `lb` | 0.45359237 | mass |
| `oz` | 0.028349523125 | mass |
| `gal` | 3.785411784e-3 | m³ |
| `qt` | 9.46352946e-4 | m³ |
| `pt` | 4.73176473e-4 | m³ |
| `mph` | 0.44704 | m/s |
| `knot` | 0.5144444444444445 | m/s |
| `AU`, `au` | 1.495978707e11 | length |
| `pc` | 3.0856775814913673e16 | length |
| `ly` | 9.4607304725808e15 | length |
| `Jy` | 1e-26 | kg/s² |

A quantity displays in its display unit when it has one (typed or
converted), otherwise in the SI spelling of its dimensions (`m/s^2`,
or the exact derived name when the dimensions match one: `N`, `W`,
`Pa`, …).

## Number display

The answer panel's automatic spelling (ADR-0051):

- A float prints with up to **12 significant digits**, exact integers
  keeping every digit.
- **Exact fractions**: when a rational with denominator ≤ 1000 agrees
  with the value through all twelve displayed digits *and* is not a
  terminating decimal, the answer shows as a fraction — `1/3` stays
  `1/3`, while `0.1 + 0.2` shows `0.3`, not `3/10`.
- Complex answers print in `a+bi` form. Booleans print `true`/`false`.
  Lists print `{1, 2, 3}`; matrices print row by row: `[[1, 2], [3, 4]]`.
- The display verbs (`scientific`, `engineering`, `grouped`, `fixed`,
  `bin`, `oct`, `hex`) return text, ready to `print` or `join`.

## Errors

An error stops evaluation of the statement in progress; in a script
the whole run stops and the exit carries the message. Error classes,
with the verbatim shape of each message:

| Class | Example message |
|---|---|
| Parse | `unexpected character: '@'`; `expected digits after 0x`; `unknown escape \d in a string: the escapes are \n, \t, \r and \\`; `unterminated string: a literal needs its closing quote`; `unterminated block comment: expected */` |
| Type | `rad expects 1 number, got 1 argument(s)`; `if condition must be a boolean, got 3`; `and expects booleans, got 2`; `lists have different lengths: 3 and 1`; `cannot compare Complex(Complex { re: 3.0, im: 4.0 })` |
| Domain | `sqrt of negative number -4` (as a real); `asin of 2 outside -1..1`; `no prime below 2`; `factors of 0 must be a positive integer` |
| Division | `division by zero` |
| Name | `unknown name: nmae` |
| Constant/variable conflicts | `cannot assign to constant TAX`; `cannot define constant x: the name is already a variable` |
| Reserved name | `cannot assign to i: that name is the imaginary unit` |
| Control flow | `break outside a loop`; `continue outside a loop`; `return outside a function` |
| Limits | the step budget exhausted; `for runs at most 100000 iterations, got 100001` |
| Dimension | `cannot add m and s` (dimension mismatch); `cannot take the square root of 2 m: the dimensions do not divide evenly` |

## Limits and determinism

- **Step budget**: every program run carries a 100,000-step budget;
  each function call gets its own fresh budget. Exhausting it stops
  the run.
- **For iterations**: a single `for` loop is capped at 100,000 passes.
- **Floats**: 64-bit binary; integers exact to 2^53.
- **Reproducibility**: the random generator is SplitMix64 with a
  Box–Muller step for `randn`; `randseed(n)` makes a sequence
  repeatable. Scripts evaluated by the checker pin a seed, so a
  script's transcript is byte-stable.
- **Display**: 12 significant digits, fractions only through
  denominator 1000.

## The complete grammar (EBNF)

The grammar is written in EBNF, the Extended Backus–Naur Form. Each rule
names a construct and says how it is built: `,` joins parts in sequence,
`|` separates alternatives, `[ … ]` marks a part that may be absent,
`{ … }` a part that may repeat, and `"quoted text"` is a token written
exactly so; rules refer to one another until every chain ends in a
literal token, and any input that a chain of these rules accepts parses,
and anything else does not.

```text
program     = statement , { separator , statement } ;
separator   = ";" | newline ;

statement   = "break"
            | "continue"
            | "return" , expression
            | "if" , expression , "then" , statement , [ "else" , statement ]
            | "while" , expression , "do" , statement
            | "for" , name , "in" , expression , [ "to" , expression ,
                [ "step" , expression ] ] , "do" , statement
            | "solve" , expression
            | "const" , name , "=" , expression
            | "def" , name , "(" , [ params ] , ")" , defbody
            | pattern , "=" , expression
            | name , "=" , expression
            | expression ;
params      = name , { "," , name } ;
defbody     = "=" , expression
            | "do" , block , "end" ;
pattern     = "{" , name , { "," , name } , "}" ;
block       = statement , { separator , statement } ;

expression  = ifexpr ;
ifexpr      = "if" , expression , "then" , expression ,
              "else" , expression | orexpr ;
orexpr      = andexpr , { "or" , andexpr } ;
andexpr     = notexpr , { "and" , notexpr } ;
notexpr     = "not" , notexpr | comparison ;
comparison  = bitor , [ cmpop , bitor ] ;
cmpop       = ">" | "<" | ">=" | "<=" | "==" | "!=" ;
bitor       = bitand , { ( "|" | "xor" ) , bitand } ;
bitand      = shift , { "&" , shift } ;
shift       = additive , [ ( "<<" | ">>" ) , additive ] ;
additive    = term , { ( "+" | "-" ) , term } ;
term        = unary , { ( "*" | "/" ) , unary } ;
unary       = "-" , unary | "~" , unary | power ;
power       = postfix , [ "^" , unary ] ;
postfix     = primary , { "!" | "%" | index } ;
index       = "[" , expression , "]" ;
primary     = number | string | name | call
            | "(" , expression , ")"
            | list | matrix ;
call        = name , "(" , [ expression , { "," , expression } ] , ")" ;
list        = "{" , [ expression , { "," , expression } ] , "}" ;
matrix      = "[" , row , { "," , row } , "]" ;
row         = "[" , expression , { "," , expression } , "]" ;
```

Grammar notes the EBNF cannot say:

- The additive level also admits **unit conversion** after a term:
  `expression "in" unitpath` and `expression "->" unitpath`, looping
  so `3 m in ft in inch` chains.
- Inside a multiplicative chain, a `/` directly followed by a unit
  name (and not a call) continues a **unit path** rather than
  dividing: `60 mile/hr`, `5 m/s^2`.
- A statement beginning `{ name , … } =` is a destructuring; any
  other `{` begins a list expression (the parser rewinds).
- `end` closes a `def`'s `do` block; there is no standalone `do` statement.
- `in` is a keyword both in `for … in` and in unit conversion; the
  position disambiguates.
- Number literals absorb an `i` suffix into an imaginary token; based
  literals (`0b`, `0o`, `0x`) too.

## Beyond the grammar: interactive commands

The language is what this page defines. The frontends add **commands**
around it, recognized before the parser sees the line; they are
conveniences, not part of the language definition:

| Command | Where | Meaning |
|---|---|---|
| `save name` | interactive frontends | persist a function or constant to the store |
| `save script name` | interactive frontends | persist the session as a named script |
| `graph expr` | REPL, TUI, PWA, desktop | draw a 2D plot of `expr` over the session's window |
| `graph3d expr` | REPL, TUI, PWA, desktop | draw a 3D surface |
| `language` | interactive frontends | switch the interface language |
| `quit`, `exit` | REPL | end the session |

Everything else — expression entry, scripts, the store — behaves as
this reference defines, identically on every frontend.

---

*This reference corresponds to epher 0.5.41 (staging). The function
and constant tables are extracted from `epher-core`'s dispatch and
catalog; a core test keeps them from drifting: renaming or removing a
builtin without updating this page fails the build.*
