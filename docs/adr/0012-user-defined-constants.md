# User-defined constants: `const name = value`, visible like `pi`

- Status: accepted
- Date: 2026-08-16

Users can define their own constants with `const name = value`. The value is
evaluated once, at the definition, and the name is immutable afterwards.

We chose `const name = value` because it is the syntax users already know:
JavaScript, C, C++, Java (`final`), Rust, Go, Kotlin, and Julia all introduce
immutable named values with a `const`-shaped keyword statement, and the
calculator languages that don't have it (TI-BASIC, most desktop calculators)
have no immutability to express at all. Alternatives rejected: an
uppercase-naming convention (Python style) is invisible to the language and
unenforceable; overloading `def` (a zero-parameter function already fills that
niche, and "constant" and "function" are different domain nouns in
`CONTEXT.md`).

Semantics:

- **Evaluated once, immediately**: `const area = pi * r ^ 2` captures the
  value, not the expression (JavaScript semantics).
- **Immutable**: assigning with `=` after `const` is an error
  (`cannot assign to constant tax`), and so is redefining it with a
  different value (`constant already defined: tax`); re-declaring it with
  the value it already has is a no-op (see the amendment below). A name is
  either a variable or a constant, never both, so lookups stay unambiguous.
- **Visible inside functions, like `pi`**: the built-in constants are visible
  in function bodies; session variables are not. User constants follow the
  built-ins (`new_child` copies them), which is what makes them useful:
  `const g = 9.81; def weight(m) = m * g` works. A parameter still shadows a
  constant.
- **Persisted like functions**: `save tax` stores the `const` source line in
  the Store (kind `constant`, ADR-0002), and startup replay orders functions,
  then constants, then scripts, so constants may call functions and scripts
  may use both. The web/PWA has no Store yet, exactly like functions
  (ADR-0003).

## Amendment (2026-08-30): identical re-declaration is a no-op

The examples-validation sweep found the strict rule working against
its own copies: two guide examples in the web-app chapter both open
with `const a = 1`, and running the second after the first errored
with `constant already defined: a` - the copy buttons on the guide
and the Examples page invite exactly that re-paste. The slider and
animation paths already rewrite constants in place (`set_constant`),
so the entry field was the one place a constant could not be
re-stated. Re-declaring a constant with the value it already has now
succeeds as a no-op; a different value keeps the documented error,
and the guide's tax demo (0.2 then 0.25) still shows it. Constants
still never take a variable's name.

## Amendment (2026-08-31): the built-in constants grow a physics shelf (ADR-0042)

The constants catalog now carries twenty-one CODATA 2022 physical
constants alongside the astronomy shelf (ADR-0037). Nothing about the
resolution order changes: a user variable wins, then a user constant,
then the built-in - so `const g = 9.81` still shadows the standard
gravity, and every new name (`G`, `q_e`, `n_a`, ...) is shadowable the
same way. `const` redefinition of a built-in name keeps erroring with
the existing "already defined" voice.

## Amendment (2026-10-31): a script is a scope; `share` crosses it; `def` joins the exclusivity rule

The no-op amendment fixed re-pasting the *same* example, but the
underlying failure stayed: a script valid on its own still failed (or
behaved differently) because of what earlier scripts in the same
session had defined - a second script's `const r = 3` errored against
the first script's `const r = 2`, an interactive assignment blocked a
script's `const`, and `def`, `randseed`, and `bits` interfered silently
(worse: no error, a different answer). The trigger case is the product
inviting it - the Examples browser loads a script into the entry, the
guide's copy buttons hand out scripts, and worked examples open with
`const` knobs.

Three rules replace the flat shared environment, following the
precedent a `for` loop already set (its control variable is the loop's
own, ADR-0063):

1. **A multi-statement submission is a scope.** The TUI, web app,
   desktop app, CLI REPL/one-shot/stdin, and the language server's
   `epher/run` all run multi-statement scripts against a child of the
   session environment. When the scope closes, the script's variable
   bindings (`ans` included) merge back - the interactive scratchpad
   keeps its calculator flow - while the script's constants and
   functions evaporate. The generator and word size are the script's
   own copies, so `randseed(42)` inside a script reproduces its draws
   on every re-run and `bits(8)` cannot leak into the next script.
   A single-statement submission stays interactive: `const tax = 0.2`
   typed on its own still defines a session constant with today's
   immutability.
2. **`share name, name` promotes.** The one statement that crosses the
   boundary on purpose: the named definitions (constant, function, or
   binding) join the session, with their source lines, so `save` and
   the next startup replay persist them. Sharing over a session
   definition of the same kind overwrites it - the slider path already
   rewrites constants in place (ADR-0014); crossing kinds is the same
   error defining the name directly would give. Inside a scope, a
   definition that meets a name inherited from the session library (a
   shared constant, a store replay) shadows it for the run instead of
   erroring; a definition that contradicts the script's *own* earlier
   definition still errors, exactly as before.
3. **`def` joins the name-exclusivity rule.** "A name is one thing" now
   spans all three namespaces: re-defining a function the session
   defined with a different body errors (`function already defined: f`),
   identical re-definition is a no-op (the same courtesy `const` got in
   the 2026-08-30 amendment), `def` refuses a variable's or constant's
   name, `const` refuses a function's name, and assignment refuses a
   function's name. Previously a `def` silently overwrote any function
   and coexisted with same-named variables and constants.

The escape hatch the error messages name is real now: `reset` starts
the session over (definitions and generator state forgotten, the
transcript kept). The messages previously pointed at a "Clear" that no
frontend offered.

Alternatives considered: letting `const` overwrite silently (throws
away the immutability the language documents and the slider depends
on); merging script constants back and erroring only on value change
(keeps the failure - the merge is the problem); scoping every single
statement (breaks the calculator REPL flow the TUI tests pin).
