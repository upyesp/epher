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
