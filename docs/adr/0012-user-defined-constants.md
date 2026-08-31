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

- **Evaluated once, immediately** — `const area = pi * r ^ 2` captures the
  value, not the expression (JavaScript semantics).
- **Immutable** — assigning with `=` after `const` is an error
  (`cannot assign to constant tax`), and so is redefining it with a
  different value (`constant already defined: tax`); re-declaring it with
  the value it already has is a no-op (see the amendment below). A name is
  either a variable or a constant, never both, so lookups stay unambiguous.
- **Visible inside functions, like `pi`** — the built-in constants are visible
  in function bodies; session variables are not. User constants follow the
  built-ins (`new_child` copies them), which is what makes them useful:
  `const g = 9.81; def weight(m) = m * g` works. A parameter still shadows a
  constant.
- **Persisted like functions** — `save tax` stores the `const` source line in
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
