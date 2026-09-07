# ADR-0047: bitwise operations

- Status: accepted
- Date: 2026-09-02
- Roadmap: feature-gap analysis round 6 (T2.8 bitwise operations —
  the parallel programmer track that rounds out the unit system)

## Context

Six of the nine researched apps ship bitwise operations, and the
programmer-mode staple is a small, well-understood surface: and, or,
xor, not, and shifts. epher already owns the base literals (0b, 0o,
0x; ADR-0022) and the exact whole-number type (`big`); what is missing
is the operation set. The report also names word-size settings, as
SpeedCrunch, HiPER, and HP Prime have them.

## Decision

### The operators

- Infix `&` (and), `|` (or), `xor` (exclusive or), `<<` (shift left),
  `>>` (shift right), and unary `~` (not). The boolean `and`/`or`
  keep their names; `^` stays the power operator, so xor is spelled
  out — the word `xor` is reserved like `and`/`or`.
- Precedence, C-style, below the comparisons: shift binds tighter than
  `&`, which binds tighter than `|`/`xor`; everything else binds
  tighter than them. `5 & 3 == 1` is `(5 & 3) == 1`, and
  `1 | 2 << 3` is `1 | (2 << 3)` = 17.
- The operands are integers: floats must be whole (like the base
  conversions), and rationals/decimals must be integral too; anything
  else is a type error. The result is always a `Big` exact whole
  number, so `1 << 60` stays exact where an f64 would round.

### Word size

- The working word size is a session state, `bits()` reports it and
  `bits(n)` sets it to 8, 16, 32, or 64 (anything else is a domain
  error); the default is 64 — SpeedCrunch parity, no settings panel.
  Like `randseed`, it lives in the environment and shares through
  function bodies, so a script can pin it and restore it.
- Every bitwise result is interpreted as a signed n-bit two's
  complement word: the mathematical result is masked to n bits and
  the top bit decides the sign. `bits(8)` then `~0` is -1, `255 & 1`
  is 1, and `1 << 8` is 0 — the word wraps like the researched
  calculators. Right shift is arithmetic: `-8 >> 1` is -4.
- The base-conversion displays (bin/oct/hex) are untouched: they
  show the exact integer, and a negative shows its sign, as today.

## Consequences

- Six new token kinds and six expression forms; the keypad stays
  frozen (`&`, `|`, `~`, `<<`, `>>` are typed or copied from the
  guide, and autocomplete offers `bits` and the `xor` hint).
- Bitwise results are `Big` values: exact arithmetic, displayable in
  any base, and promotable in arithmetic like every `big` today.
- One new session verb (`bits`) and one hint key, plus the guide
  section and quick-reference rows in all eight locales.
