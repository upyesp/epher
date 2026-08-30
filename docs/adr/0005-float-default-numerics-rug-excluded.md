# Float-by-default numerics behind one Value enum; GMP/rug excluded

- Status: accepted
- Date: 2026-08-13

`epher-core` uses `f64` as the default fast path, with opt-in exact-rational
(`num-rational`/`BigRational`), decimal (`rust_decimal`, escalating to
`bigdecimal`), and complex (`num-complex`) layers. Every number representation is
a variant of a single `Value` enum behind a `Numeric` trait, so crate choices
never leak into the evaluator grammar or the Store schema.

We excluded `rug` / `gmp-mpfr-sys`: it requires the C GMP/MPFR/MPC libraries,
which cannot target `wasm32-unknown-unknown` (verified — GMP's `configure`
rejects the target). This is the hard constraint that makes the `num-*` family
the default and reserves `malachite` as the wasm-safe bignum fallback if
GMP-class performance is ever needed. `f64` transcendentals compile on wasm with
std, so graphing's fast path needs no `libm`. Claims verified by building each
crate to both targets — see `docs/research/numerics-options.md`.

## Amendment (2026-08-30): `^` behaves per layer, and refuses honestly

The same Horizons-driven validation sweep found the exponentiation
operator implemented only in the float layer; every exact layer
answered "not supported yet", and a float power of a negative base
with a fractional exponent displayed a bare `NaN`. The layers now
agree on one rule: an **integer exponent stays exact** (rationals via
`Ratio::pow`, decimals via checked multiplication with the
reciprocal for negative exponents, big decimals via `powi`, whose
negative-exponent context scale is then normalized), a **fractional
exponent is a type error** telling the user to work in floats -
the exact layers refuse rather than silently lose their exactness -
and the float layer's one genuinely undefined case (negative base,
fractional exponent) is a **domain error** pointing at `root(n, x)`,
where the real root is computed correctly. Ordinary float powers,
`0^0 = 1`, and IEEE overflow (`2^1024 = inf`) are unchanged.
