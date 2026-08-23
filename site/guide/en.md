# epher user guide

Welcome! epher is a programmable, scriptable calculator. You can use it for a
quick calculation, or build up your own functions and small programs — and
everything is available in eight languages.

This guide is for complete beginners. It starts with the simplest possible
calculation and builds up to the full power of the language. Every example
shows what you type and what epher answers.

There are four ways to use epher — pick whichever suits you:

| Version | What it is | Best when |
|---|---|---|
| **Web app** (PWA) | Runs in your browser, installable, works offline | You want the fastest start; no installation |
| **Desktop app** | A normal desktop program with its own window | You want a regular application |
| **Command line** (CLI) | Text commands in a terminal; also an interactive session | You live in a terminal and like scripts |
| **Terminal UI** (TUI) | A full-screen program inside the terminal | You want a terminal app with graphs and history on screen |

The desktop app, the command line, and the terminal UI are one program: a
single download installs the `epher` command, which does all three. The web
app is the exception — it needs no download at all.

All four versions understand exactly the same language. Learn it once, use it
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

1. `!` factorial
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

Powers can be fractional — `2 ^ 0.5` is the square root of 2:

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

You can change a variable whenever you like — it keeps its value until you
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
`Ans` key on a pocket calculator — handy for chained calculations:

```epher
2 + 3
ans * 2
```

```text
5
10
```

### 1.6 Constants: names that never change

A *constant* is a name for a value that never changes — like the built-in
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

and so is defining the same constant twice:

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

> epher does not have text values — both branches of an `if` must be numbers
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

The most famous example — the Fibonacci numbers:

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

> A function's body is a single expression — one line. Combine several
> calculations with `;` in a script instead (next section).

### 1.11 Scripts: several statements at once

A *script* is several statements joined with `;` — or with newlines,
which mean exactly the same thing — executed one after another:

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
`;` on one line works everywhere too — including the one-shot command line
(section 4.1).

### 1.12 Exact results: frac, dec and big

Normally epher calculates with decimal numbers like a pocket calculator.
Some numbers look better exact.

**frac(n, d)** makes an exact fraction:

```epher
1 / 3
```

```text
0.3333333333333333
```

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
0.30000000000000004
```

```epher
dec(0.1) + dec(0.2)
```

```text
0.3
```

The first result is the tiny rounding error every computer makes with
decimal numbers. `dec()` removes it.

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

Convert back with **bin(x)**, **oct(x)** and **hex(x)** — the prefixed
spelling of a whole number, ready to feed straight back in:

```epher
hex(255)
bin(10)
```

```text
0xff
0b1010
```

### 1.13 Built-in functions

epher has the functions of a scientific calculator, grouped by family.

Trigonometry works in radians — use `deg` and `rad` to convert:

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
| Binary, octal, hex | `0b…`, `0o…`, `0x…` | `0xFF + 0b1` |
| Base spelling | `bin(x)`, `oct(x)`, `hex(x)` | `hex(255)` |

## 2. The web app (PWA)

### 2.1 Opening it

The web app lives at:

```text
https://epher.org/pwa/
```

No installation is needed — it works in any modern browser on a computer,
phone, or tablet.

This guide is also built into the app: open **Help → User guide** in the
menu bar (tap **☰** on a phone) to read it inside the app, in the app's
current language. Tap any example in that guide to load it into the entry
field.

### 2.2 Your first calculation

1. Click the text field (it is already focused when the page loads).
2. Type an expression, for example `2 + 3 * 4`.
3. Press **Enter** or click the **=** button.

The result appears in large text below the field. Everything from chapter 1
works here, including variables, functions, and scripts.

### 2.3 History

Every calculation is added to the history list beneath the result, so you
can scroll back and see what you did. Newest entries appear at the top, and
the **Clear history** button above the list empties it. The history is kept
while the page is open.

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
colour and dash pattern, and a legend naming them. `graph clear` empties
the plot — and a **Clear graph** button at the top of the graph pane does
the same for curves and 3D surfaces together. The TUI keeps the command in
its **Graph** menu.

At the bottom of the graph pane, the options row can hide the list of
points of interest, hide the highlighted points drawn on the plot itself,
and set the thickness of the plotted lines with the **Line width**
slider.

```epher
graph x ^ 2
graph x ^ 3
```

Points where the expression has no value (a division by zero, for
example) are skipped, leaving a gap in the curve — and a jump that is
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

**Trace:** move the pointer over the plot — or focus it and press the
arrow keys — and the nearest point on a curve is marked, with its
coordinates announced beneath the plot.

**Points of interest:** after every graph command epher finds the roots
and turning points of each curve and the intersections between curves,
marks them on the plot, and lists them beneath it:

```text
root (-1, 0)   minimum (0, 0)   root (1, 0)
```

**Tables:** the `table` command prints a table of values (rows where the
expression has no value are blank):

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
plot — drag it (or move it with the arrow keys) and every curve redraws:

```epher
const a = 1
graph a * x ^ 2
```

**Copy SVG** copies the current plot as a self-contained SVG image for
pasting into documents — the colours are baked in, so it looks the same
anywhere. The **Line width** slider at the bottom of the pane sets how
thick every plotted line draws.

#### 2.4.4 3D surfaces

`graph3d` plots a surface z = f(x, y) over a square domain (−5 to 5, or
your `from a to b`):

```epher
graph3d x ^ 2 - y ^ 2
```

Mesh lines nearer to you are drawn stronger, so the shape reads in depth.
Several `graph3d` lines overlay, like curves, and `graph3d clear` empties
the plot. Rotate the view by dragging, or focus the plot and use the arrow
keys. The terminal UI draws the same surface as an ASCII wireframe, with
the arrow keys rotating it.

#### 2.4.5 Animation

Every slider has a play button. It steps its constant through the
slider's range and loops around — the standard way calculators animate:
you animate a parameter, and everything that uses it moves. Press the
button again to pause.

A "time" variable is just a constant you animate:

```epher
const t = 0
graph sin(x - t)
```

Playing t's slider makes the wave travel. 3D surfaces animate the same
way — define a constant first, then play its slider:

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

Once installed, launch it from your home screen or app list — it opens
instantly, even with no internet connection.

### 2.6 What the web app does not do

The web app keeps your work to the current session: it evaluates
expressions, graphs them (section 2.4), and keeps a history. The **save**,
**save script**, and **language** commands work in the desktop, command
line, and terminal versions (chapters 3, 4, and 5) — in the web app they
answer with a note that saving works there. The history is not saved
between visits.

## 3. The desktop app

The desktop app is a normal window around the same web app. Everything in
chapter 2 applies; the difference is only how you install and start it.

### 3.1 Installing

Download one installer for your system from the epher website:

- **Windows:** run `epher-windows-x86_64.exe`. The installer puts `epher` on
  your PATH — open a new CMD or PowerShell window and `epher "2 + 2"` works.
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

- **Linux (any distro, including Arch):** the AppImage — make it executable
  and run it:

```sh
chmod +x epher-linux-x86_64.AppImage
./epher-linux-x86_64.AppImage
```

Every installer contains the *whole* epher — the desktop app, the command
line (chapter 4), and the terminal UI (chapter 5) — as the single `epher`
command. On Linux, the package puts `epher` in `/usr/bin`.

### 3.2 Using it

Launch epher like any other application. You get a window with the same
interface as the web app: type an expression, press **Enter** or click
**=**, and read the result. Graphing works here too — `graph x ^ 2` draws
in the window (chapter 2.4). The window can be resized freely. The menu bar
includes **Help → User guide** — the same guide as this page, with
tap-to-load examples.

You can also open it from a terminal: a bare `epher` (or `epher gui`) starts
the desktop app. On macOS, use the **Install the epher command** button inside
the app to put `epher` on your terminal PATH.

### 3.3 Storage: one store with the CLI and TUI

The desktop app shares its storage with the command line and terminal
versions. Functions, constants, scripts, history, and the language preference live in
one place — `~/.epher` on your computer (or `EPHER_STORE_DIR`, chapter
4.6) — and everything saved in one version is available in the others:

```text
def area(w, h) = w * h
save area
```

Define `area` in the desktop app, `save` it, close the window — then open
the CLI and `area(3, 4)` just works. It works the other way too: functions
and scripts you saved in the CLI or TUI are already there when the desktop
window opens, including variables set by saved scripts. The `save`,
`save script`, and `language` commands from chapter 4 work exactly the
same here.

> The web app in the browser is the one version that does not use this
> storage — it keeps each session to itself (chapter 2.6).

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
Anything from chapter 1 is available — variables, functions, loops,
everything — and the lines share one session, like a piped script
(section 4.2).

### 4.2 Piped scripts

`epher -` reads expressions from standard input, one line at a time — the way
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
several statements with `;` — newlines and `;` mean the same thing
everywhere in epher.

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

Constants save the same way — `save` on the constant's name:

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

### 4.5 Changing the interface language

The interface language is chosen from the languages you set on your device.
To override it, type `language` followed by one of: `en`, `zh-CN`, `hi`,
`es`, `fr`, `ar`, `de`, `pt`:

```text
epher> language fr
language set to fr
```

The choice is remembered for next time. Note: the language you *type* — the
expression language — is always the same, in any interface language.

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
terminal. It is part of the same `epher` program — start it with:

```sh
epher tui
```

### 5.1 The screen

The screen is divided into panels:

- **Expression** — the input line (top).
- The current **result** right below it.
- **History** — every line you entered, with its answer.
- **Graph** — the plot from the `graph` command (bottom).
- A hint line shows the keyboard shortcuts.

### 5.2 Keys

| Key | Action |
|---|---|
| Type | add to the expression |
| **Enter** | evaluate |
| **Esc** | clear the input line |
| **Ctrl+C** | quit |
| **q** | quit (when the input is empty) |
| **Arrow keys** | rotate the 3D view (when the input is empty) |
| **Space** | start/stop the animation (when the input is empty) |
| **F10** | open the menus (File, Edit, Graph, Settings, Help) |
| **Tab** | open the function keypad; switch its banks (**Esc** closes) |
| **Ctrl+L** | clear the history |

The keypad's banks hold every function, constant, and command the
language supports: **trig**, **fn**, **num**, and **var**. Arrow keys
move the highlight, **Enter** inserts the token, and **Tab** cycles
the banks.

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

You can graph any expression, including your own functions — first define
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
the TUI lists the points of interest — roots, turning points, and
intersections — under the plot. The `table` command (section 2.4.2) works
here too.

`graph3d x ^ 2 - y ^ 2` plots a 3D surface as an ASCII wireframe — rotate
it with the arrow keys, and press the space bar to animate a slider
constant (section 2.4.5).

`graph save plot.svg` writes the current plot as the same SVG image the
web app's **Copy SVG** button yields; `graph3d save file.svg` saves the
3D wireframe from the angle you are looking at.

### 5.4 Saving and persistence

The TUI shares its storage with the CLI: everything saved in one is
available in the other. Functions, scripts, history, and the language
preference live in `~/.epher` (chapter 4.6), and the same `save`,
`save script`, and `language` commands work here.

## 6. Your data and privacy

- The **installed epher program** — desktop app, CLI, and TUI — stores
  functions, scripts, history, and the language choice locally in `~/.epher`
  (or `EPHER_STORE_DIR`). Nothing leaves your computer.
- The **web app** keeps nothing on disk: history lasts only while the page
  is open. The web app can work offline because the page itself is stored by
  your browser.

All four versions run the calculation entirely on your device — nothing is
sent anywhere.
