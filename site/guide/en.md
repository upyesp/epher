# epher user guide

Welcome! epher is a programmable, scriptable calculator. You can use it for a
quick calculation, or build up your own functions and small programs. Everything
is available in eight languages.

This guide is for complete beginners. It starts with the simplest possible
calculation and builds up to the full power of the language. Every example
shows what you type and what epher answers.

There are five ways to use epher. Pick whichever suits you:

| Version | What it is | Best when |
|---|---|---|
| **Command line** (CLI) | Text commands in a terminal | You live in a terminal and like scripts |
| **REPL** | An interactive `epher` session at the `epher>` prompt | You want quick back-and-forth without leaving the terminal |
| **Terminal UI** (TUI) | A full-screen program inside the terminal | You want a terminal app with graphs and history on screen |
| **Desktop app** | A normal desktop program with its own window | You want a regular application |
| **Web app** (PWA) | Runs in your browser, installable, works offline | You want the fastest start; no installation |

The desktop app, the command line, the REPL, and the terminal UI are one
program: a single download installs the `epher` command, which does all
four. The web app is the exception: it needs no download at all.

All five versions understand exactly the same language. Learn it once, use it
anywhere.

## 1. The epher language

This chapter teaches the language shared by every version of epher. In the web
app or desktop app, type an expression and press **Enter** (or click the
**=** button). In the CLI, start the session with `epher repl` and type after
the `epher>` prompt. In the TUI (`epher tui`), just type and press **Enter**. In the CLI you can also write
`epher "expression"` to evaluate one expression directly.

### 1.1 Your first calculation

Type this:

```epher
2 + 3 * 4
```

epher answers:

```text
14
```

Multiplication is done before addition, exactly like in mathematics. That
rule is called *operator precedence*.

### 1.2 Order of operations

The full precedence order, from strongest to weakest:

1. `!` factorial and `%` percent (both postfix)
2. `^` power
3. `*` and `/` multiplication and division
4. `+` and `-` addition and subtraction

Use parentheses to change the order:

```epher
(2 + 3) * 4
```

```text
20
```

The `^` operator computes powers, and it works right-to-left:

```epher
2 ^ 10
```

```text
1024
```

```epher
2 ^ 3 ^ 2
```

```text
512
```

(`2 ^ 3 ^ 2` means `2 ^ (3 ^ 2)`, which is `2 ^ 9` = 512.)

Powers can be fractional. `2 ^ 0.5` is the square root of 2:

```epher
2 ^ 0.5
```

```text
1.4142135623730951
```

Subtraction and division work left-to-right:

```epher
10 - 3 - 2
```

```text
5
```

The `%` sign is a postfix operator that means "divided by 100": `5%` is
0.05. It never looks at the operators around it, so `200 + 10%` is
200.1. To increase 200 by 10%, spell the multiplication:

```epher
200 * (1 + 10%)
```

```text
220
```

### 1.3 The special numbers pi, e, tau and phi

The famous constants are built in:

```epher
pi
```

```text
3.141592653589793
```

```epher
2 * pi
```

```text
6.283185307179586
```

```epher
e
```

```text
2.718281828459045
```

Two more: `tau` is a full turn (2 pi), and `phi` is the golden ratio:

```epher
tau
```

```text
6.283185307179586
```

```epher
phi
```

```text
1.618033988749895
```

### 1.4 Comparing and logic

You can compare numbers. The result is either `true` or `false`:

| Comparison | Meaning |
|---|---|
| `a > b` | a is greater than b |
| `a < b` | a is less than b |
| `a >= b` | a is greater than or equal to b |
| `a <= b` | a is less than or equal to b |
| `a == b` | a equals b (note the double `=`) |
| `a != b` | a does not equal b |

```epher
3 > 2
```

```text
true
```

```epher
1 != 2
```

```text
true
```

Combine comparisons with `and`, `or` and `not`:

```epher
3 > 2 and 2 < 3
```

```text
true
```

```epher
not 3 > 2
```

```text
false
```

### 1.5 Variables

Give a name to a value with a single `=`:

```epher
x = 5
```

```text
5
```

epher repeats the value back to you. From now on, `x` can be used anywhere:

```epher
x ^ 2
```

```text
25
```

You can change a variable whenever you like. It keeps its value until you
change it:

```epher
x = x + 1
```

```text
6
```

> Names can contain letters and underscores, like `radius` or `my_total`.
> They cannot contain spaces or start with a number.

The special variable `ans` always holds the previous answer, like the
`Ans` key on a pocket calculator, handy for chained calculations:

```epher
2 + 3
ans * 2
```

```text
5
10
```

### 1.6 Constants: names that never change

A *constant* is a name for a value that never changes, like the built-in
`pi`, but chosen by you. Define one with `const`:

```epher
const tax = 0.2
```

```text
0.2
```

Use it anywhere a number can go:

```epher
100 * (1 + tax)
```

```text
120
```

The value is fixed: changing it with `=` is an error,

```epher
tax = 0.25
```

```text
error: cannot assign to constant tax
```

and so is redefining it with a different value:

```epher
const tax = 0.25
```

```text
error: constant already defined: tax
```

Constants are different from variables in one more way: like `pi`, they
work inside your own functions.

```epher
const g = 9.81
```

```text
9.81
```

```epher
def weight(m) = m * g
```

```epher
weight(80)
```

```text
784.8000000000001
```

Save a constant for future sessions with `save tax`, exactly like a
function (chapter 4.4).

> A variable and a constant cannot share a name: after
> `const tax = 0.2`, `tax = ...` is always an error. Pick a fresh name or
> start a new session.

### 1.7 Decisions with if

`if` chooses between two values:

```epher
if 3 > 2 then 10 else 20
```

```text
10
```

The shape is always `if condition then value_if_true else value_if_false`.
The `else` part is required.

A more useful example with a variable:

```epher
price = 100
if price > 50 then 2 else 1
```

```text
2
```

> epher does not have text values: both branches of an `if` must be numbers
> (or the results of comparisons).

### 1.8 Loops with while

`while` repeats a statement as long as a condition holds:

```epher
x = 0; while x < 5 do x = x + 1; x
```

```text
5
```

Read that script as: *start x at 0; while x is less than 5, add 1 to x; then
show x.* The result is 5 because the loop ran five times.

> **Safety net:** epher stops any loop after 100,000 steps and shows
> `error: step limit exceeded`. That protects you from loops that would
> never end. If you see it, your condition probably never became false.

### 1.9 Your own functions with def

A function is a calculation with a name and parameters:

```epher
def f(x) = x ^ 2
```

Then use it:

```epher
f(7)
```

```text
49
```

Functions can take several parameters:

```epher
def area(w, h) = w * h
area(3, 4)
```

```text
12
```

You can also define a function with no parameters:

```epher
def answer() = 42
answer()
```

```text
42
```

### 1.10 Recursion: a function that calls itself

The most famous example is the Fibonacci numbers:

```epher
def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
```

```epher
fib(10)
```

```text
55
```

`fib(10)` is the 10th Fibonacci number. The function calls itself with
smaller arguments until it reaches `n <= 1`. This works because the
`if ... then ... else ...` form only calculates the branch it needs.

> A function's body is a single expression, one line. Combine several
> calculations with `;` in a script instead (next section).

### 1.11 Scripts: several statements at once

A *script* is several statements joined with `;` or with newlines,
which mean exactly the same thing, executed one after another:

```epher
x = 10; y = x + 5; x + y
```

```text
25
```

Scripts are how you build small programs: set up variables, loop, and show a
final result.

Newlines and `;` are the same separator, and you can mix them freely. The
**Copy** button above a multi-line example copies the whole script, and you
can paste it straight into epher: the entry field on the web app and the
desktop app, the terminal UI, and `epher repl` all run each line in order,
exactly as if you had typed them one by one. Joining several statements with
`;` on one line works everywhere too, including the one-shot command line
(section 4.1).


Scripts can carry **comments** - notes for you that epher skips, written the PHP way. `//` or `#` comments to the end of the line; `/* ... */` comments out a block, across lines or inline between tokens:

```epher
// a small script with notes
r = 3 # radius in metres
area = /* pi r squared */ pi * r ^ 2
area
```
### 1.12 Exact results: frac, dec and big

Normally epher calculates with decimal numbers like a pocket calculator,
but exact fractions are on by default: any result with a good
small-denominator fraction shows as one. `1 / 3` displays as `1/3`
without asking:

```epher
1 / 3
```

```text
1/3
```

With **exact fractions off** in the Results settings (chapter 2.2) the
same division shows `0.3333333333333333`. **frac(n, d)** makes an exact
fraction that stays exact through calculations:

```epher
frac(1, 3)
```

```text
1/3
```

Fractions stay exact through calculations:

```epher
frac(1, 3) * 3
```

```text
1
```

**dec(x)** makes an exact decimal. Compare these two:

```epher
0.1 + 0.2
```

```text
3/10
```

```epher
dec(0.1) + dec(0.2)
```

```text
0.3
```

The first result is the exact fraction behind the answer and the
second the decimal itself; with exact fractions off, the first instead
shows the tiny rounding error every computer makes with decimal
numbers (`0.30000000000000004`).

**big(x)** makes an exact whole number, for values too large for a pocket
calculator:

```epher
big(10 ^ 20)
```

```text
100000000000000000000
```

**Number bases** write integers the way the math community spells them:
`0b` for binary, `0o` for octal, `0x` for hex (the prefix changes the
spelling, never the value):

```epher
0b1010 + 0xFF
```

```text
265
```

Convert back with **bin(x)**, **oct(x)** and **hex(x)**. These give the
prefixed spelling of a whole number, ready to feed straight back in:

```epher
hex(255)
bin(10)
```

```text
0xff
0b1010
```

**exact(x)** reconstructs the exact fraction behind a decimal result:
any value with a good small-denominator fraction shows it. This is the
same reconstruction the apps use for their default display, so `1 / 3`
usually shows as `1/3` without asking:

```epher
exact(0.3333333333333333)
exact(0.30000000000000004)
```

```text
1/3
3/10
```

An irrational value like `pi` has no good fraction, so `exact()` leaves
it alone.

The display verbs spell a number in other notations. **scientific(x)**
uses one digit before the exponent, **engineering(x)** uses exponents in
steps of three (the mantissa stays between 1 and 1000), and
**grouped(x)** inserts thin-space thousands separators:

```epher
scientific(12345)
engineering(12345)
engineering(0.5)
grouped(1234567.89)
```

```text
1.2345e4
12.345e3
500e-3
1 234 567.89
```

The web app and TUI also offer these as display settings (see
chapter 2.2 and 5.2): exact fractions on or off, Auto/Scientific/
Engineering notation, and thousands separators. The settings only change
how results are shown; the values stay decimal numbers underneath.

### 1.13 Built-in functions

epher has the functions of a scientific calculator, grouped by family.

Trigonometry works in radians. Use `deg` and `rad` to convert:

| Function | Meaning | Example | Result |
|---|---|---|---|
| `sin(x)`, `cos(x)`, `tan(x)` | trigonometric functions | `sin(pi / 2)` | `1` |
| `asin(x)`, `acos(x)`, `atan(x)` | inverse trigonometric | `atan(1)` | `0.7853981633974483` |
| `atan2(y, x)` | angle of the point (x, y) | `atan2(1, 1)` | `0.7853981633974483` |
| `deg(x)` | radians → degrees | `deg(pi)` | `180` |
| `rad(x)` | degrees → radians | `rad(180)` | `3.141592653589793` |
| `sinh(x)`, `cosh(x)`, `tanh(x)` | hyperbolic functions | `sinh(1)` | `1.1752011936438014` |
| `asinh(x)`, `acosh(x)`, `atanh(x)` | inverse hyperbolic | `acosh(1)` | `0` |

Powers, roots and logarithms (on a calculator `log` is base 10):

| Function | Meaning | Example | Result |
|---|---|---|---|
| `sqrt(x)` | square root | `sqrt(16)` | `4` |
| `cbrt(x)` | cube root | `cbrt(-27)` | `-3` |
| `root(n, x)` | nth root | `root(3, 8)` | `2` |
| `exp(x)` | e to the power x | `exp(1)` | `2.718281828459045` |
| `ln(x)` | natural logarithm | `ln(e)` | `1` |
| `log(x)` | base-10 logarithm | `log(100)` | `2` |
| `log2(x)` | base-2 logarithm | `log2(8)` | `3` |
| `logb(b, x)` | logarithm in base b | `logb(2, 8)` | `3` |
| `hypot(a, b)` | hypotenuse | `hypot(3, 4)` | `5` |
| `5!` (also `fact(n)`) | factorial | `5!` | `120` |

Rounding, signs and whole numbers:

| Function | Meaning | Example | Result |
|---|---|---|---|
| `abs(x)` | absolute value | `abs(-3)` | `3` |
| `floor(x)` / `ceil(x)` | round down / up | `floor(2.7)` | `2` |
| `round(x)` | nearest, half away from zero | `round(2.5)` | `3` |
| `trunc(x)` | drop the fraction | `trunc(-2.9)` | `-2` |
| `sign(x)` | -1, 0 or 1 | `sign(-5)` | `-1` |
| `ncr(n, r)` | combinations | `ncr(52, 5)` | `2598960` |
| `npr(n, r)` | permutations | `npr(5, 2)` | `20` |
| `gcd(a, b)` / `lcm(a, b)` | common divisors and multiples | `gcd(12, 18)` | `6` |
| `mod(a, b)` | remainder | `mod(7, 3)` | `1` |

Primes and divisors work on whole numbers:

| Function | Meaning | Example | Result |
|---|---|---|---|
| `isprime(n)` | true when n is prime | `isprime(97)` | `true` |
| `nextprime(n)` / `prevprime(n)` | nearest primes | `nextprime(10)` | `11` |
| `factors(n)` | prime factorization | `factors(360)` | `2^3 * 3^2 * 5` |
| `totient(n)` | Euler's totient | `totient(12)` | `4` |
| `ndivisors(n)` | how many divisors | `ndivisors(360)` | `24` |
| `modpow(b, e, m)` | b to the e, mod m, exactly | `modpow(2, 10, 1000)` | `24` |

Statistics take any number of arguments:

| Function | Meaning | Example | Result |
|---|---|---|---|
| `sum(...)` / `product(...)` | totals | `sum(1, 2, 3)` | `6` |
| `mean(...)` | average | `mean(1, 2, 3)` | `2` |
| `median(...)` | middle value | `median(1, 2, 3, 4)` | `2.5` |
| `min(...)` / `max(...)` | smallest / largest | `max(4, 1, 3)` | `4` |
| `variance(...)` / `stdev(...)` | spread of the values | `stdev(2, 4)` | `1` |

The exact layers from section 1.12 stay:

| Function | Meaning | Example | Result |
|---|---|---|---|
| `frac(n, d)` | exact fraction | `frac(1, 3)` | `1/3` |
| `dec(x)` | exact decimal | `dec(0.1)` | `0.1` |
| `big(x)` | exact whole number | `big(10 ^ 20)` | `100000000000000000000` |
| Binary, octal, hex | `0b…`, `0o…`, `0x…` | `0xFF + 0b1` |
| Base spelling | `bin(x)`, `oct(x)`, `hex(x)` | `hex(255)` |
| `bin(x)` / `oct(x)` / `hex(x)` | prefixed spelling in base 2 / 8 / 16 | `hex(255)` | `0xff` |

They combine like everything else:

```epher
min(sqrt(16), 5)
```

```text
4
```

The physical constants use SI units, like the astronomy ones in section
1.16:

| Name | Meaning | Value |
|---|---|---|
| `G` | Newton's gravitational constant | 6.6743e-11 |
| `gamma` | Euler-Mascheroni constant | 0.5772156649015329 |
| `q_e` | elementary charge | 1.602176634e-19 |
| `ev` | electronvolt, in joules | 1.602176634e-19 |
| `eps_0` | vacuum permittivity | 8.8541878128e-12 |
| `mu_0` | vacuum permeability | 1.25663706212e-6 |
| `z_0` | impedance of free space | 376.730313668 |
| `m_e` | mass of the electron | 9.1093837139e-31 |
| `m_p` | mass of the proton | 1.67262192595e-27 |
| `m_n` | mass of the neutron | 1.67492750056e-27 |
| `m_u` | atomic mass unit | 1.66053906892e-27 |
| `a_0` | Bohr radius | 5.29177210544e-11 |
| `alpha` | fine-structure constant | 0.0072973525643 |
| `r_inf` | Rydberg constant | 10973731.568160 |
| `mu_b` | Bohr magneton | 9.2740100783e-24 |
| `n_a` | Avogadro constant | 6.02214076e23 |
| `faraday` | Faraday constant, C/mol | 96485.33212 |
| `r_gas` | molar gas constant | 8.31446261815324 |
| `atm` | standard atmosphere, in pascals | 101325 |
| `wien` | Wien wavelength constant | 0.002897771955 |
| `phi_0` | magnetic flux quantum | 2.067833848e-15 |
| `m_P` | Planck mass | 2.176434e-8 |
| `l_P` | Planck length | 1.616255e-35 |
| `t_P` | Planck time | 5.391247e-44 |
| `r_e` | classical electron radius | 2.8179403205e-15 |
| `lambda_c` | Compton wavelength | 2.42631023867e-12 |
| `mu_n` | nuclear magneton | 5.050783699e-27 |

### 1.14 Reading errors

When something goes wrong, epher tells you instead of guessing:

```epher
1 / 0
```

```text
error: division by zero
```

```epher
sqrt(-4)
```

```text
error: domain error: sqrt of negative number -4
```

```epher
unknown_name
```

```text
error: unknown name: unknown_name
```

```epher
foo(1)
```

```text
error: unknown name: foo
```

The last example is important: epher tells you exactly which name it does
not know, so you can fix your expression.

### 1.15 Quick reference

| What | Syntax | Example |
|---|---|---|
| Add, subtract, multiply, divide | `+ - * /` | `7 / 2` |
| Power | `^` (right-to-left) | `2 ^ 10` |
| Factorial | `!` (postfix) | `5!` |
| Percent | `%` (postfix) | `200 * (1 + 10%)` |
| Parentheses | `( )` | `(2 + 3) * 4` |
| Constants | `pi`, `e`, `tau`, `phi` | `2 * pi` |
| Scientific notation | `2.5e-3` | `6.02e23` |
| Compare | `> < >= <= == !=` | `3 >= 2` |
| Logic | `and or not` | `a > 1 and a < 10` |
| Variable | `name = value` | `x = 5` |
| Constant | `const name = value` | `const tax = 0.2` |
| Decision | `if c then a else b` | `if x > 0 then 1 else -1` |
| Loop | `while c do statement` | `while x < 5 do x = x + 1` |
| Function | `def name(params) = expr` | `def f(x) = x ^ 2` |
| Script | statements joined with `;` or newlines | `x = 1; x + 1` |
| Exact fraction | `frac(n, d)` | `frac(1, 3)` |
| Exact decimal | `dec(x)` | `dec(0.1) + dec(0.2)` |
| Exact whole number | `big(x)` | `big(10 ^ 20)` |
| Reconstruct a fraction | `exact(x)` | `exact(0.3333333333333333)` |
| Scientific, engineering, grouped | `scientific(x)` `engineering(x)` `grouped(x)` | `engineering(12345)` |
| Imaginary unit | `i`, or a literal `4i` | `sqrt(-1)` |
| Complex parts | `re(z)` `im(z)` `arg(z)` `conj(z)` `abs(z)` | `re(3 + 4i)` |
| Solve an equation | `solve lhs == rhs` | `solve x^2 == 9` |
| Numeric derivative | `derivative(expr, x)` | `derivative(x^2, 3)` |
| Definite integral | `integral(expr, a, b)` | `integral(x^2, 0, 3)` |
| Binary, octal, hex | `0b…`, `0o…`, `0x…` | `0xFF + 0b1` |
| Base spelling | `bin(x)`, `oct(x)`, `hex(x)` | `hex(255)` |
| Primes | `isprime(n)`, `factors(n)`, … | `factors(360)` |
| List literal | `{…}` | `{1, 2, 3}` |
| List element | `list[i]` (1-based) | `{5, 6}[2]` |
| List statistics | `mean(list)`, `median(list)`, … | `stdev(d)` |
| List shape | `len(s)`, `sort(s)`, `mode(s)`, `range(s)`, `quartile(s, k)` | `quartile(d, 1)` |
| Linear regression | `linreg(xs, ys)` | `linreg(x, y)` |
| Normal family | `normpdf` `normcdf` `invnorm` | `invnorm(0.975)` |
| t family | `tpdf` `tcdf` `invt` | `invt(0.975, 10)` |
| Chi-squared family | `chi2pdf` `chi2cdf` `invchi2` | `chi2cdf(3.84, 1)` |
| Discrete families | `binompdf` `binomcdf` `poissonpdf` `poissoncdf` | `binomcdf(2, 10, 0.5)` |
| Tests and intervals | `ztest` `ttest` `zinterval` `tinterval` `chisq_gof` | `tinterval(d, 0.95)` |
| Data plots | `graph scatter(xs, ys)` `histogram(data)` `boxplot(data)` | `graph boxplot(d)` |
| Random numbers | `random()`, `random(a, b)`, `randint(a, b)`, `randseed(n)` | `randint(1, 6)` |
| Constants browser | Help → Constants: every builtin constant, grouped | Help → Constants |
| Quantity | `5 m`, `60 mile/hr`, `1 km` | `2 m^2` |
| Convert | `expr in unit` or `expr -> unit` | `72 km/hr in m/s` |
| Prefixes | `k M G T m µ n p` scale any unit | `5 km`, `3 MPa`, `1 GHz` |
| Bitwise and, or | `a & b`, `a \| b` | `0xFF & 0x0F` |
| Bitwise xor | `a xor b` | `5 xor 3` |
| Bitwise not | `~a` | `~0` |
| Shifts | `a << n`, `a >> n` | `1 << 8` |
| Word size | `bits(n)` for 8, 16, 32, 64 | `bits(8)` |
| Implicit relation | `graph lhs == rhs` | `graph x^2 + y^2 == 1` |
| Matrix literal | `[[1, 2], [3, 4]]` | `[[1, 2], [3, 4]] * [[5, 6], [7, 8]]` |
| Matrix functions | `det` `inv` `transpose` `trace` `dim` `ref` `rref` | `rref([[2, 1, 5], [1, -1, 1]])` |
| TVM solver | `tvm_n` `tvm_i` `tvm_pv` `tvm_pmt` `tvm_fv` | `tvm_pmt(360, 0.08/12, -100000, 0)` |
| NPV and IRR | `npv(rate, flows)` `irr(flows)` | `irr({-100, 60, 60})` |
| Amortization | `amort(p, r, n, k)` | `amort(1000, 0.01, 12, 6)` |
| Interest | `simple_interest` `compound_interest` | `compound_interest(1000, 0.05, 2)` |

### 1.16 Astronomy and the solar system

epher speaks astronomy: unit suffixes, physical constants, calendar and time
functions, and a live ephemeris for the Sun, the Moon, the planets and Pluto.
Everything works offline.

**Units that speak astronomy.** Write a number followed by a unit suffix and
epher converts it to SI units on the spot:

| Suffix | Unit | Converts to |
|---|---|---|
| `AU` or `au` | astronomical unit | metres |
| `pc` | parsec | metres |
| `ly` | light year | metres |
| `deg` | degree | radians |
| `arcmin`, `arcsec` | arcminute, arcsecond | radians |
| `min`, `hr`, `d`, `yr` | minute, hour, day, Julian year | seconds |
| `Jy` | jansky | W m-2 Hz-1 |

```epher
3.2 AU in m
```

```text
478713186240 m
```

```epher
sin(30 deg)
```

```text
0.5
```

The suffixes are part of the grammar, so no user constant can change what
`3.2 AU` means, and `h` stays Planck's constant: hours are written `hr`.
Functions return counts in natural units; a suffix converts a count to SI,
so `mag2jy(20)` is a jansky count and `mag2jy(20) Jy` is the same flux in
watts per square metre hertz.

**Astronomy constants.** `au`, `pc`, `ly`, `c`, `g`, `h`, `h_bar`, `k_b`,
`sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`, `r_earth`, `m_moon`, `r_moon` work like `pi`,
and you can shadow them with your own constants.

**Dates and time.** `jd(y, m, d [, hr])` and `mjd(...)` turn a calendar date
into a Julian Date, `now()` reads the current instant:

```epher
jd(2000, 1, 1, 12)
```

```text
2451545
```

`delta_t(jd)` is the TT - UT1 correction, and `lst(jd, lon)` is the local
sidereal time in hours for a longitude in degrees east.

**Hours, minutes and seconds.** `hms2deg(h, m, s)` converts right ascension
to degrees, `dms2deg(d, m, s)` converts a sexagesimal angle, and
`deg2hms(x)` / `deg2dms(x)` spell an angle back as text:

```epher
deg2hms(90)
```

```text
6h 0m 0s
```

**The sky, quantified.** Give each accessor a body number: Mercury 1,
Venus 2, Mars 4, Jupiter 5, Saturn 6, Uranus 7, Neptune 8, Pluto 9,
Sun 10, Moon 11 (Earth is 3, the observer, never a target).

| Function | Meaning |
|---|---|
| `ra(b, jd)`, `decl(b, jd)` | geocentric right ascension and declination (degrees) |
| `dist(b, jd)` | distance in AU |
| `alt(b, jd, lat, lon)`, `az(b, jd, lat, lon)` | topocentric altitude and azimuth (degrees, true) |
| `rise(b, jd, lat, lon)`, `set(...)`, `transit(...)` | events of that local solar day, as Julian Dates |
| `mag(b, jd)` | apparent magnitude |
| `phase(b, jd)`, `illum(b, jd)` | phase angle (degrees) and illuminated fraction |
| `diam(b, jd)` | angular diameter (degrees) |

```epher
decl(10, jd(2000, 6, 21, 1.8))
```

```text
23.437882351
```

Latitudes and longitudes are degrees, east positive. Positions are
geocentric unless an observer is given. Pluto rides an approximate
orbit that is honest to about an arcminute, far below the accuracy of
the other bodies; eclipses and conjunction searches are not included.

**Optics and light.** `kepler(M, e)` solves Kepler's equation,
`airmass(alt)` is the sec(z) airmass, `dawes(d)` is the resolving power of
a d-millimetre aperture in arcseconds, and `dist_mod(mu)` turns a distance
modulus into parsecs.

**Seasons.** `march_equinox(year)`, `june_solstice(year)`,
`september_equinox(year)` and `december_solstice(year)` return the Julian
Date of each season boundary:

```epher
march_equinox(2000)
```

```text
2451623.8159797275
```

**The solar system in 3D.** The `solar3d` command draws the whole system:
every orbit as a curve, every body as a labelled dot, with a trail showing
where it just was:

```epher
solar3d jd(2020, 7, 1)
```

Give the time as a constant and press the play button to watch the planets
move: `const t = now(); solar3d t`. Drag or use the arrow keys to orbit,
`clear` to empty, and `solar3d save file.svg` to export.

The ephemeris is computed by the solar-ephemeris crate
(github.com/Protonmatter/sol), validated against JPL Horizons; thank you to
its author. Accuracy is arcsecond-class for the Sun, Moon and planets over
roughly 5000 years around the present.

### 1.17 Complex numbers

epher calculates with complex numbers automatically. The imaginary
unit is **i**, exactly like `pi`:

```epher
i ^ 2
sqrt(-1)
```

```text
-1
i
```

Write a complex number with the `i` suffix, no multiplication sign
needed: `3 + 4i` is one literal, `2.5i` works, and so do the based
literals (`0xFFi`). The usual arithmetic extends: add, subtract,
multiply, divide, and powers all work, and `i` follows the normal
precedence (`i ^ 2` binds like any power).

The real functions extend too. Given a complex argument they compute in
the complex plane; given a real argument outside their real domain they
return the principal complex result instead of an error:

```epher
ln(-1)
asin(2)
exp(i * pi)
```

```text
3.141592653589793i
1.5707963267948966-1.3169578969248166i
-1+0.00000000000000012246467991473532i
```

(`exp(i * pi)` is exactly `-1`; the last digits are the noise of
`sin(pi)` in the computer's arithmetic.)

Four functions read a complex number's parts, and `abs()` is its
magnitude:

```epher
re(3 + 4i)
im(3 + 4i)
arg(-1)
conj(3 - 4i)
abs(3 + 4i)
```

```text
3
4
3.141592653589793
3+4i
5
```

Integer-only functions (`fact`, `gcd`, `floor`, `isprime`, ...) reject
complex arguments with a type error.

### 1.18 Solving equations

**solve** finds the roots of an equation in one variable. The equation
uses `==`:

```epher
solve x^2 == 5*x + 6
```

```text
x = -1, x = 6
```

Polynomial equations (built from `+ - * ^` and constants) give every
root, real and complex:

```epher
solve x^2 == -1
solve x^2 + 2*x + 5 == 0
solve (x - 1)^2 == 0
```

```text
x = -i, x = i
x = -1-2i, x = -1+2i
x = 1
```

The variable solved for is `x` when it appears, otherwise the single
other variable. Constants and bound variables act as parameters:

```epher
const k = 3
solve k*x == 12
```

```text
x = 4
```

Any other equation is scanned numerically over -100..100: roots are
found by bracketing sign changes, so `solve sin(x) == 0.5` lists every
root in that range. Two honest limitations: a root where the function
only touches zero (like `x^2 == 0` through the numeric path) can be
missed, and equations in several unbound variables are an error.

### 1.19 Calculus: derivative and integral

**derivative(expr, p)** is the numeric derivative of `expr` at `p`. The
first argument stays an expression, and its free variable is the one
differentiated:

```epher
derivative(x^2, 3)
derivative(sin(t), 0)
```

```text
6
1
```

Because the argument stays an expression, the derivative is graphable:
`graph derivative(x^3 - x, x)` plots the slope curve.

**integral(expr, a, b)** is the definite integral from `a` to `b`,
computed by adaptive Simpson quadrature:

```epher
integral(x^2, 0, 3)
integral(sin(x), 0, pi)
```

```text
9
2
```

`integral(x^2, 3, 0)` is `-9` (the signed integral), and a graphable
upper bound works: `graph integral(x^2, 0, x)`.

Both are numeric; the expressions must be real-valued over the range,
and an expression in several variables is an error.

### 1.20 Data: lists, statistics, and regression

A list is a column of numbers in braces: `{1, 2, 3}`. Elements are
expressions, the empty list `{}` is allowed, and a list binds to a name
like any value:

```epher
d = {12, 15, 14, 16, 13, 15, 14, 17}
d[2]
len(d)
```

```text
{12, 15, 14, 16, 13, 15, 14, 17}
15
8
```

`list[i]` is the i-th element, 1-based like a calculator expects; an
out-of-range index is an error. The bracket binds tighter than `^`, so
`d[2]^2` is `(d[2])^2`.

Arithmetic over a list is elementwise, with a plain number broadcast to
every element:

```epher
{1, 2, 3} * 2
{1, 2, 3} + 10
```

```text
{2, 4, 6}
{11, 12, 13}
```

Two lists must have the same length for `+ - * / ^`. `==` and `!=`
compare whole lists; ordering comparisons reject them.

The statistics functions take a list as their one argument (they keep
their variadic form too — `mean(1, 2, 3)` still works): `sum product
mean median mode variance stdev min max range`. The new shape
functions are `len(list)`, `sort(list)` (ascending copy), `mode(list)`
(most frequent value, smallest on ties), `range(list)` (max minus
min), and `quartile(list, k)` for k in 1..3 (TI-style median of
halves):

```epher
mean(d)
median(d)
quartile(d, 1)
```

```text
14.5
14.5
13.5
```

**linreg(xs, ys)** fits the least-squares line through two same-length
lists and reports it with the correlation r:

```epher
linreg({1, 2, 3, 4}, {2.1, 4.2, 5.8, 8.1})
```

```text
y = 1.96*x + 0.15 (r = 0.9979)
```

The fitted line is a display, like solve's roots; the picture of the
fit lives on the scatter plot (section 1.22).

### 1.21 Distributions and hypothesis tests

The probability functions cover the standard normal, Student's t,
chi-squared, binomial, and Poisson families. The normal family takes
one or three arguments — one argument is the standard normal:

```epher
normcdf(1.96)
invnorm(0.975)
normcdf(12, 10, 2)
```

```text
0.975002104852
1.95996398454
0.841344746069
```

`normpdf(x[, mu, sigma])`, `normcdf(x[, mu, sigma])`, `invnorm(p[,
mu, sigma])`; `tpdf(x, df)`, `tcdf(x, df)`, `invt(p, df)`;
`chi2pdf(x, df)`, `chi2cdf(x, df)`, `invchi2(p, df)`;
`binompdf(k, n, p)`, `binomcdf(k, n, p)`; `poissonpdf(k, lambda)`,
`poissoncdf(k, lambda)`. The `inv*` functions answer the reverse
question: `invt(0.975, 10)` is the t value with 97.5% of the mass
below it.

The tests take a data list and report the statistic and the two-sided
p-value as a display string; the intervals report `(lo, hi)` at the
level you name:

```epher
d = {12, 15, 14, 16, 13, 15, 14, 17}
ttest(d, 14)
tinterval(d, 0.95)
ztest(d, 14, 1.5)
chisq_gof({20, 30, 25, 25}, {25, 25, 25, 25})
```

```text
t = 0.8819, p = 0.4071
(13.1594, 15.8406)
z = 0.9428, p = 0.3458
chi2 = 2, p = 0.5724
```

`ttest(data, mu0)` and `tinterval(data, level)` use the sample
standard deviation (n−1); `ztest(data, mu0, sigma)` and
`zinterval(data, sigma, level)` need the known sigma.
`chisq_gof(observed, expected)` is the goodness-of-fit test with k−1
degrees of freedom. The results are display strings, so they are
readable and copy-pasteable, but arithmetic cannot touch them.

### 1.22 Data plots

The graph family takes lists too: a scatter, a histogram, and a
box-and-whisker plot. A data plot owns the pane like a solar system
does — the newest command wins, and `graph clear` empties it.

```epher
x = {1, 2, 3, 4, 5}
y = {2.1, 4.2, 5.8, 8.1, 9.9}
graph scatter(x, y)
```

```epher
graph histogram({1, 2, 2, 3, 3, 3, 4, 5})
```

```epher
graph boxplot({1, 2, 2, 3, 3, 3, 9})
```

**scatter(xs, ys)** plots the points and, with two or more points,
draws the least-squares fit line, captioned `y = a*x + b (r = …)` in
the legend. **histogram(data[, bins])** draws a frequency histogram;
the bin count is optional (Sturges' rule by default) and must be a
whole number between 1 and 50. **boxplot(data)** draws the
box-and-whisker: min, Q1, median, Q3, max, with whiskers to the
extremes. The plot window always fits the data — the `from a to b`
domain keywords do not apply — and the picture exports and saves like
any other plot.

### 1.23 Random numbers

`random()` draws a uniform random number in `[0, 1)`, `random(a, b)`
one in `[a, b)`, and `randint(a, b)` a whole number from the closed
range `[a, b]` — a dice roll:

```epher
randseed(7)
randint(1, 6)
```

```text
7
3
```

The sequence is reproducible: `randseed(n)` re-seeds the generator
with `n` and reports it, so the same seed replays the same draws in
every session and every frontend.

### 1.24 Units and conversion

A number followed by a unit becomes a *quantity*: the value in SI
units plus its dimensions. The unit table covers the SI base and
derived units (`m`, `s`, `kg`, `A`, `K`, `mol`, `cd`, `Hz`, `N`, `Pa`,
`J`, `W`, `C`, `V`, `F`, `ohm`, `S`, `Wb`, `T`, `H`, `lm`, `lx`, `Bq`,
`Gy`, `Sv`), the everyday units (`min`, `hr`, `d`, `yr`, `L`, `t`,
`bar`, `atm`, `torr`, `psi`, `eV`, `mile`, `yd`, `ft`, `inch`, `nmi`,
`lb`, `oz`, `gal`, `qt`, `pt`, `mph`, `knot`), and the astronomy
suffixes from section 1.16. Compound units chain: `60 mile/hr` and
`5 m/s^2` are single units.

```epher
60 mile/hr
```

```text
60 mile/hr
```

The SI prefixes scale any of them: `k M G T m µ n p` are kilo, mega,
giga, tera, milli, micro, nano, pico — `5 km`, `3 MPa`, `1 GHz` all
work, and `2 kg` is the kilogram itself.

The dimensions are checked: adding or comparing quantities with
different units errors instead of mixing metres and seconds:

```epher
5 m + 3 s
```

```text
error: dimension error: cannot add 5 m and 3 s
```

Arithmetic composes the dimensions: `5 m * 3 m` is `15 m^2`,
`(3 m)^2` is `9 m^2`, `sqrt(4 m^2)` is `2 m`, and a whole expression
whose dimensions cancel is an ordinary number again (`5 m / 5 m` is
`1`). Results prefer the exact derived name when the dimensions match
one — `5 kg * 3 m / 1 s^2` answers `15 N`.

**Conversion.** `expr in unit` (or `expr -> unit`) shows a quantity in
the named unit; the dimensions must match. `in` binds loosest of the
operators, so `5 m + 3 m in km` converts the whole sum:

```epher
72 km/hr in m/s
```

```text
20 m/s
```

```epher
2 m^2 in cm^2
```

```text
20000 cm^2
```

Temperature scales (Celsius, Fahrenheit) are not units here — kelvins
are, and `K` works like any other.

### 1.25 Bitwise operations

The base literals from section 1.13 are made for it: `0b101`, `0o17`,
`0xFF`. The bitwise operators work on whole numbers and answer with
exact integers:

```epher
0xFF & 0x0F
```

```text
15
```

| Operator | Meaning |
|---|---|
| `a & b` | bitwise and |
| `a \| b` | bitwise or |
| `a xor b` | bitwise exclusive or |
| `~a` | bitwise not (two's complement) |
| `a << n` | shift left (multiply by 2^n) |
| `a >> n` | shift right, arithmetic (divide by 2^n, rounding down) |

The results are exact `big` integers, so `1 << 60` keeps every digit.
The working word size is 64 bits by default: results are read as
signed two's complement, so `~0` is -1 and `1 << 100` wraps to 0.
`bits(n)` changes the word size to 8, 16, 32, or 64, and `bits()`
reports it:

```epher
bits(8)
~0
```

```text
8
-1
```

Shifts by a negative amount reverse the direction (`8 << -1` is `4`).
The boolean `and` and `or` keep their meanings; `&` and `|` are the
bitwise spellings.

### 1.26 Implicit relations

An equation in two unknowns plots as a curve: the graph family samples
the relation with marching squares and draws its zero contour. The
circle, the parabola, and the vertical line are all one command each:

```epher
graph x^2 + y^2 == 1
```

```epher
graph y == x^2
```

```epher
graph x == 2
```

The relation is sampled over the square from `from a to b` (or the
default window), so `graph x^2 + y^2 == 1 from -2 to 2` fits the
circle's window. Everything a curve can do applies: the legend
captions the equation, sliders animate its constants, and the picture
zooms, pans, and exports like any other plot. The inequality fills
(`y < …`, `y > …`) stay curves with shading; a relation has no points
of interest.

### 1.27 Matrices

A matrix is a grid of numbers, spelled as rows of lists: `[[1, 2],
[3, 4]]` is the 2×2 matrix. `+` and `-` are elementwise (matching
shapes), `*` is the matrix product, a number scales elementwise, and
`^` is the whole-number matrix power (`A ^ 0` is the identity, so
powers need square matrices). `M[2][1]` is the element at row 2,
column 1 — rows index like lists, 1-based.

```epher
[[1, 2], [3, 4]] * [[5, 6], [7, 8]]
```

```text
[[19, 22], [43, 50]]
```

The matrix functions cover the classroom floor: `det(M)` (square
only), `inv(M)` (singular matrices are an error), `transpose(M)`,
`trace(M)` (square), `dim(M)` (the `{rows, cols}` list), and `ref(M)`
with `rref(M)` for row reduction. Linear systems solve through rref
on the augmented matrix:

```epher
rref([[2, 1, 5], [1, -1, 1]])
```

```text
[[1, 0, 2], [0, 1, 1]]
```

The rows read `x = 2`, `y = 1` — the last column of the reduced
augmented matrix. Exact fractions display inside matrices like lists,
so `inv([[1, 2], [3, 4]])` shows `[[-2, 1], [3/2, -1/2]]`.

### 1.28 Finance

The time-value-of-money solver (TI sign convention: money out is
negative, money in positive) solves any one of the five fields given
the other four. `i` is the per-period rate as a fraction — 0.01 is 1%
— and the optional last argument is the payment timing: 0 for end of
period (the default), 1 for beginning (annuity due).

```epher
tvm_pmt(360, 0.08/12, -100000, 0)
```

```text
327259/446
```

The classic 8% mortgage: 360 monthly payments of 733.76 against a
100,000 loan — `tvm_pmt` is the payment, `tvm_pv` the loan, `tvm_fv`
the balance, `tvm_n` the term, and `tvm_i` the rate:

```epher
tvm_i(360, -100000, 733.76, 0)
```

```text
0.006666611990680783
```

The rate here is just under 8%/12 because 733.76 is rounded. `npv(r,
flows)` discounts a cash-flow list and `irr(flows)` finds the rate
where the net present value is zero:

```epher
npv(0.1, {-100, 60, 60})
```

```text
500/121
```

`amort(p, r, n, k)` is the remaining balance after k payments of an
n-period loan, `simple_interest(p, r, t)` is `p*r*t`, and
`compound_interest(p, r, n)` is `p*(1+r)^n - p`.

## 2. The web app (PWA)

### 2.1 Opening it

The web app lives at:

```text
https://epher.org/pwa/
```

No installation is needed. It works in any modern browser on a computer,
phone, or tablet.

This guide is also built into the app: open **Help → User guide** in the
menu bar (tap **☰** on a phone) to read it inside the app, in the app's
current language. **Help → Constants** opens the constants browser: every
builtin constant in groups (Math, Astronomy, Physics, Chemistry), each
with its value and a short description; tap one to insert its name into
the entry field, and the search box narrows the list. Tap any example in
the guide to load it into the entry field.

### 2.2 Your first calculation

1. Click the text field (it is already focused when the page loads).
2. Type an expression, for example `2 + 3 * 4`.
3. Press **Enter** or click the **=** button.

The result appears in large text below the field. Everything from chapter 1
works here, including variables, functions, and scripts.

While you type a name, a suggestion list appears beneath the field: the
arrows move the highlight, **Enter** or **Tab** accepts, **Esc** closes,
and a click accepts without leaving the keyboard. Each suggestion carries
a short description of the function or constant. **F1** shows the same
description for the word under the cursor in the hint bar above the
keypad. If the first thing you type into an empty field is an operator
(`+ - * / ^ % !`), epher inserts `ans` for you, so the line continues
from the previous answer.

The **Settings** menu (the gear icon, or **☰ → Settings** on a phone)
holds three groups. **Theme** and **Language** do what their names
say. **Results** shapes how answers are shown: exact fractions (on by
default, so `1 / 3` displays as `1/3`), the notation (Auto,
Scientific, or Engineering), and thousands separators. These are
display settings only; the values underneath stay ordinary numbers.

### 2.3 History

Every calculation is added to the history list beneath the result, so you
can scroll back and see what you did. Newest entries appear at the top, and
the trash icon beside the **History** heading empties it (in the terminal,
Ctrl+L, or a click on the same icon). The history is kept while the page is
open.

Each entry sits between thin rules: a single-line expression is one row,
and a multi-line script is one entry showing all of its lines. Click an
entry to load it back into the entry field and run it again.

### 2.4 Graphing

Type `graph` followed by an expression and press **Enter**:

```epher
graph x ^ 2
```

epher draws the curve y = f(x) from x = −10 to x = 10 beneath the input,
on a grid with labelled axes. You can graph any expression, including
your own functions:

```epher
def f(x) = x ^ 3
graph f(x)
```

Every `graph` line adds another curve to the same plot, each with its own
colour, and a legend naming them. The curves are all solid, so the
legend and the captions are what tell them apart without colour. `graph clear` empties
the plot, and a **Clear graph** button at the top of the graph pane does
the same for curves and 3D surfaces together. The TUI keeps the command in
its **Graph** menu.

At the top of the graph pane, beside **Clear graph** and **Copy SVG**,
the toolbar can hide the list of points of interest and the highlighted
points drawn on the plot itself. Directly above every plot sits a strip
of icon-labelled sliders, the words in each one's tooltip: line
thickness (0 to 4 in steps of 0.1 for 2D curves, 0 to 0.2 in steps of
0.01 for 3D surfaces - only the kind in view is shown, and each kind
remembers its own value), and on 3D and solar plots the horizontal and
vertical rotation speeds and the zoom speed. Every legend entry has a
checkbox, checked by default: unchecking a curve hides it from the plot,
its points of interest, and the SVG export.

```epher
graph x ^ 2
graph x ^ 3
```

Points where the expression has no value (a division by zero, for
example) are skipped, leaving a gap in the curve. A jump that is
really a vertical asymptote is never drawn as a connecting line.

#### 2.4.1 What you can plot

A domain of your choice:

```epher
graph sin(x) from 0 to 2*pi
```

Parametric curves, with t running from 0 to 2π:

```epher
graph param 2*cos(t), 3*sin(t)
```

Polar curves:

```epher
graph polar 1 + cos(theta)
```

Regions: `y <` shades the area below the curve, `y >` shades above:

```epher
graph y < x ^ 2
```
#### 2.4.2 Reading the plot

**Trace:** move the pointer over the plot, or focus it and press the
arrow keys. The nearest point on a curve is marked, with its
coordinates announced beneath the plot.

**Points of interest:** after every graph command epher finds the roots
and turning points of each curve and the intersections between curves,
marks them on the plot, and lists them beneath it:

```text
root (-1, 0)   minimum (0, 0)   root (1, 0)
```

**Tables:** the `table` command prints a table of values (rows where the
expression has no value are blank). An optional `derivative <expr>`
clause adds a third column, the numeric derivative of that expression
at each x:

```epher
table x ^ 2 from -2 to 2 points 5 derivative x ^ 2
```

```text
         x           y          y'
        -2           4          -4
        -1           1          -2
         0           0           0
         1           1           2
         2           4           4
```

Table cells follow the Results settings: with exact fractions on (the
default), a value that is a simple fraction shows as one — `table x / 3
from 0 to 1 points 4` lists `1/3` instead of `0.333`.

```epher
table x ^ 2 from -2 to 2 points 5
```

```text
         x           y
        -2           4
        -1           1
         0           0
         1           1
         2           4
```

#### 2.4.3 Sliders and export

Define a constant, use it in a graph, and a slider appears beneath the
plot. Drag it (or move it with the arrow keys) and every curve redraws:

```epher
const a = 1
graph a * x ^ 2
```

**Copy SVG** copies the current plot as a self-contained SVG image for
pasting into documents. The colours are baked in, so it looks the same
anywhere. **Save PNG** saves the same picture as a bitmap at twice its
size, so curves stay crisp; the desktop app asks where to put it, the
browser app saves it to your downloads (or asks, where the browser
offers to). The slider rows and any animated constants sit directly
beneath the plot, above the points-of-interest list.

#### 2.4.4 3D surfaces

`graph3d` plots a surface z = f(x, y) over a square domain (−5 to 5, or
your `from a to b`):

```epher
graph3d x ^ 2 - y ^ 2
```

Mesh lines nearer to you are drawn stronger, so the shape reads in depth.
Several `graph3d` lines overlay, like curves, and `graph3d clear` empties
the plot. The pane shows one kind at a time: drawing a surface clears
the 2D curves, and drawing a curve clears the surfaces. Each plot
keeps its full size. Rotate the view by dragging, or focus the plot and use the arrow
keys. The terminal UI draws the same surface as an ASCII wireframe, with
the arrow keys rotating it.

#### 2.4.5 Animation

Every slider has a play button. It steps its constant through the
slider's range and loops around. This is the standard way calculators animate:
you animate a parameter, and everything that uses it moves. Press the
button again to pause.

A "time" variable is just a constant you animate:

```epher
const t = 0
graph sin(x - t)
```

Playing t's slider makes the wave travel. 3D surfaces animate the same
way. Define a constant first, then play its slider:

```epher
const a = 1
graph3d sin(a * (x ^ 2 + y ^ 2)) from -3 to 3
```

In the terminal UI, the space bar starts and stops the animation.

### 2.5 Installing it and using it offline

The web app is a *progressive web app*: after one visit it works fully
offline, and you can install it like a normal app.

- **Chrome, Edge, or Android:** click the install icon in the address bar
  (or *Install app* in the browser menu), then confirm.
- **iPhone / iPad (Safari):** tap **Share** → **Add to Home Screen**.
- **Other browsers:** look for *Install* or *Add to Home Screen* in the menu.

Once installed, launch it from your home screen or app list. It opens
instantly, even with no internet connection.

### 2.6 What the web app does not do

The web app keeps your work to the current session: it evaluates
expressions, graphs them (section 2.4), and keeps a history. The **save**,
**save script**, and **language** commands work in the desktop, command
line, and terminal versions (chapters 3, 4, and 5). In the web app they
answer with a note that saving works there. The history is not saved
between visits.

## 3. The desktop app

The desktop app is a normal window around the same web app. Everything in
chapter 2 applies; the difference is only how you install and start it.

### 3.1 Installing

Download one installer for your system from the epher website:

- **Windows:** run `epher-windows-x86_64.exe`. The installer puts `epher` on
  your PATH. Open a new CMD or PowerShell window and `epher "2 + 2"` works.
  Because the build is not signed, choose *More info* → *Run anyway* on the
  first launch.
- **macOS:** open `epher-macos-aarch64.dmg` and drag epher into Applications.
  Because the build is not signed, the first launch needs a right-click →
  **Open**.
- **Linux (Debian/Ubuntu):** the `.deb` package

```sh
sudo apt install ./epher-linux-x86_64.deb
```

- **Linux (Fedora/RHEL):** the `.rpm` package

```sh
sudo dnf install ./epher-linux-x86_64.rpm
```

- **Linux (any distro, including Arch):** the AppImage. Make it executable
  and run it:

```sh
chmod +x epher-linux-x86_64.AppImage
./epher-linux-x86_64.AppImage
```

Every installer contains the *whole* epher: the desktop app, the command
line (chapter 4), and the terminal UI (chapter 5), as the single `epher`
command. On Linux, the package puts `epher` in `/usr/bin`.

### 3.2 Using it

Launch epher like any other application. You get a window with the same
interface as the web app: type an expression, press **Enter** or click
**=**, and read the result. Graphing works here too. `graph x ^ 2` draws
in the window (chapter 2.4). The window can be resized freely. The menu bar
includes **Help → User guide**, the same guide as this page, with
tap-to-load examples.

You can also open it from a terminal: a bare `epher` (or `epher gui`) starts
the desktop app. On macOS, use the **Install the epher command** button inside
the app to put `epher` on your terminal PATH.

### 3.3 Storage: one store with the CLI and TUI

The desktop app shares its storage with the command line and terminal
versions. Functions, constants, scripts, history, and the language preference live in
one place, `~/.epher` on your computer (or `EPHER_STORE_DIR`, chapter
4.6), and everything saved in one version is available in the others:

```text
def area(w, h) = w * h
save area
```

Define `area` in the desktop app, `save` it, close the window. Then open
the CLI and `area(3, 4)` just works. It works the other way too: functions
and scripts you saved in the CLI or TUI are already there when the desktop
window opens, including variables set by saved scripts. The `save`,
`save script`, and `language` commands from chapter 4 work exactly the
same here.

Commands you type in the CLI, the REPL, the TUI, or the desktop app
all join the same history, and the session travels too: variables
you assign and the `ans` value follow you from one version to the
next. The shared store is live: while two versions are open at once,
a change in one is reflected in the other immediately (the desktop
app and the TUI watch the store and refresh in place).

> The web app in the browser is the one version that does not use this
> storage. It keeps each session to itself (chapter 2.6).

## 4. The command line (CLI)

The CLI is the text side of the same `epher` program as the desktop app.
It has three modes: one-shot evaluation, piped scripts, and an interactive
session for longer work.

For help at any time, run `epher --help` (all commands, with
examples) or `epher help` (the full manual; on Linux packages this is the
`man epher` page).

### 4.1 One-shot calculations

Give the expression as an argument:

```sh
epher "2 + 3 * 4"
```

```text
14
```

You can do anything from chapter 1 that is a single expression:

```sh
epher "if 3 > 2 then 10 else 20"
```

```text
10
```

An expression that starts with a minus sign works directly:

```sh
epher "-2 + 5"
```

```text
3
```

One-shot mode is for scripts, from a single expression up to a whole
program. Each statement's value prints on its own line:

```sh
epher "x = 10; x + 5"
```

```text
10
15
```

Statements joined with newlines work the same way inside the argument.
Anything from chapter 1 is available: variables, functions, loops,
everything. The lines share one session, like a piped script
(section 4.2).

### 4.2 Piped scripts

`epher -` reads expressions from standard input, one line at a time, the way
scripting languages are used in pipelines:

```sh
printf "x = 3\nx * 10\n" | epher -
```

```text
= 3
= 30
```

Everything from chapter 1 works, and the lines share one session: a function
defined on an early line is available later, and `save` writes to the same
store as always. Errors print and the script keeps going. A line may join
several statements with `;`. Newlines and `;` mean the same thing
everywhere in epher.


A file works the same way: `epher plots/sine.es` runs every line of the file in order and prints each result. The argument is treated as a file when it names an existing file and contains a `.`, `/` or `\` - so `epher x` still evaluates the name `x`.
### 4.3 The interactive session (REPL)

Start it with `epher repl`:

```sh
epher repl
```

> A bare `epher` with no arguments opens the desktop app (chapter 3).

epher prints its prompt and waits:

```text
epher>
```

Now type anything from chapter 1, one line at a time. Variables keep their
values between lines:

```text
epher> x = 5
= 5
epher> x ^ 2
= 25
```

The `table` command (section 2.4.2) prints a table of values here too:

```text
epher> table x ^ 2 from -2 to 2 points 5
         x           y
        -2           4
        -1           1
         0           0
`graph` lines plot here too: the curves build up across lines, and
`graph save plot.svg` writes the same SVG image the web app's
**Copy SVG** button yields. `graph3d save file.svg` saves a 3D surface
the same way. The same graph lines work in one-shot and piped scripts:
`epher "graph sin(x); graph save plot.svg"` is a complete plot in one
command.

         1           1
         2           4
```

Each answer is shown as `= result`. To leave, type `quit` (or `exit`):

```text
epher> quit
```

Your history is remembered: the next time you run `epher repl`, the previous
session's lines are still there.


The `load` command runs a script - a file path, or the name of a script you saved with `save script` - line by line, exactly as if you had typed it:

```text
epher> load plots/sine.es
epher> load my_setup
```
### 4.4 Saving functions, constants and scripts

Define a function, then save it:

```text
epher> def fib(n) = if n <= 1 then n else fib(n - 1) + fib(n - 2)
epher> save fib
saved fib
```

The `save fib` command stores the function on disk. Next time you start the
session, `fib` is already defined:

```text
epher> fib(10)
= 55
```

Constants save the same way. `save` on the constant's name:

```text
epher> const tax = 0.2
= 0.2
epher> save tax
saved tax
```

To save a whole script (the last line you typed) use `save script`:

```text
epher> x = 0; while x < 5 do x = x + 1; x
= 5
epher> save script count_to_five
saved script count_to_five
```

Saved scripts run automatically when epher starts, so anything they define is
ready for you.


You can also load a saved script on demand with `load count_to_five`, or keep it as a plain file and run `load count_to_five.es`; `epher count_to_five.es` runs it straight from the command line (section 4.2).
### 4.5 Changing the interface language

The interface language is chosen from the languages you set on your device.
To override it, type `language` followed by one of: `en`, `zh-CN`, `hi`,
`es`, `fr`, `ar`, `de`, `pt`:

```text
epher> language fr
language set to fr
```

The choice is remembered for next time. Note: the language you *type*, the
expression language, is always the same, in any interface language.

### 4.6 Where your data lives

Functions, scripts, history, and your language choice are stored in one
folder on your computer:

```text
~/.epher
```

Delete that folder to start completely fresh. To use a different location,
set the environment variable `EPHER_STORE_DIR` before starting epher:

```sh
EPHER_STORE_DIR=/tmp/my-epher epher repl
```

## 5. The terminal UI (TUI)

The TUI is a full-screen version of the interactive session, inside your
terminal. It is part of the same `epher` program. Start it with:

```sh
epher tui
```

### 5.1 The screen

The screen is divided into panels:

- **Expression**: the input box (top). Shift+Enter starts a new line,
  and the arrow keys or a mouse click move the cursor inside the text.
- The current **result** right below it.
- **History**: every line you entered, with its answer.
- **Graph**: the plot from the `graph` command (bottom).
- A hint line shows the keyboard shortcuts.

### 5.2 Keys

| Key | Action |
|---|---|
| Type | add to the expression at the cursor |
| **Enter** | evaluate the whole script (a multi-line entry runs as one history item) |
| **Shift+Enter** | start a new line |
| **← → ↑ ↓** | move the cursor (with empty input: rotate the 3D graph) |
| **Esc** | clear the input line |
| **F1** | describe the function under the cursor (in the answer line) |
| **Ctrl+C** | quit |
| **q** | quit (when the input is empty) |
| **Arrow keys** | rotate the 3D view (when the input is empty) |
| **Space** | start/stop the animation (when the input is empty) |
| **F10** | open the menus (File, Edit, Graph, Settings, Help) |
| **Tab** | focus the always-visible keypad (or history, from the keypad); switch its banks (**Esc** returns focus to typing) |
| **Mouse** | click menus and popup items, keypad cells and bank tabs, and history lines (loads the expression); drag the graph panel to orbit (3D) or pan (2D), the wheel zooms, a double-click resets the view |
| **Ctrl+L** | clear the history |

The **Help** menu opens the in-app guide, the keypad key help, and a
constants browser: the builtin constants in groups, arrows choose a
row, **Enter** inserts its name into the expression at the cursor, and
**Esc** closes.

The keypad's banks hold every function, constant, and command the
language supports: **trig**, **fn**, **num**, **0x**, and **var**. The
0x bank holds the exactness and base conversions (`frac`, `dec`,
`big`, `bin`, `oct`, `hex`) and the factorial `!`. Arrow keys
move the highlight, **Enter** inserts the token, and **Tab** cycles
the banks. An operator typed into an empty line (or inserted from the
keypad) adds `ans` first, so the line continues from the previous
answer.

The **Settings** menu offers the same result display choices as the
web app (exact fractions, notation, thousands separators), next to the
theme and language rows.

### 5.3 Graphing

Type `graph` followed by an expression, and press **Enter**:

```epher
graph x ^ 2
```

epher samples the curve from x = −10 to x = 10 and draws it as an ASCII
plot in the Graph panel; the legend above the plot names what is plotted.

`graph clear` empties the plot, and the **Graph** menu does the same; the
**Help** menu opens this guide inside the TUI (arrow keys scroll, **Esc**
closes). The **Settings** menu can hide the points of
interest listed under the plot.

You can graph any expression, including your own functions. First define
one, then graph it:

```epher
def f(x) = x ^ 3
graph f(x)
```

Every `graph` line adds a curve to the plot, drawn with its own symbol
(`o`, `x`, `+`, `*`); `graph clear` empties the plot. The same grammar as
the web app applies: a domain (`graph sin(x) from 0 to 2*pi`), parametric
curves (`graph param 2*cos(t), 3*sin(t)`), polar curves
(`graph polar 1 + cos(theta)`), and regions (`graph y < x ^ 2` shades the
area below the curve).

Points where the expression has no value (for example division by zero)
are simply skipped, leaving a gap in the plot. After every graph command
the TUI lists the points of interest (roots, turning points, and
intersections) under the plot. The `table` command (section 2.4.2) works
here too.

`graph3d x ^ 2 - y ^ 2` plots a 3D surface as an ASCII wireframe.
Rotate it with the arrow keys while the input is empty, and press the
space bar to animate a slider constant (section 2.4.5). The bottom hint
line shows the arrow-key and space hints only while a 3D surface or an
animatable curve is displayed.

`graph save plot.svg` writes the current plot as the same SVG image the
web app's **Copy SVG** button yields; `graph3d save file.svg` saves the
3D wireframe from the angle you are looking at.

### 5.4 Saving and persistence

The TUI shares its storage with the CLI: everything saved in one is
available in the other. Functions, scripts, history, and the language
preference live in `~/.epher` (chapter 4.6), and the same `save`,
`save script`, and `language` commands work here.

## 6. Your data and privacy

- The **installed epher program** (desktop app, CLI, and TUI) stores
  functions, scripts, history, and the language choice locally in `~/.epher`
  (or `EPHER_STORE_DIR`). Nothing leaves your computer.
- The **web app** keeps nothing on disk: history lasts only while the page
  is open. The web app can work offline because the page itself is stored by
  your browser.

All five versions run the calculation entirely on your device. Nothing is
sent anywhere.
