# Numerics crate options for `epher-core`

**Goal:** pick number types for a programmable calculator whose `epher-core`
must compile to **both** `wasm32-unknown-unknown` (Yew web + Tauri desktop)
**and** native targets (CLI/TUI on x86_64 + aarch64). Expressions want exact
fractions and arbitrary precision; graphing needs fast floats and complex.

**Method (all claims verified, not assumed):** `cargo 1.97.1`, stable toolchain
(`rustc 1.97.1`). Each candidate was pulled at the versions below into an
isolated throwaway workspace and actually built with
`cargo build --release --target wasm32-unknown-unknown` **and**
`cargo build --release` (native x86_64-unknown-linux-gnu). Dependency trees
were inspected with `cargo tree`. The `wasm32-unknown-unknown` rust-std target
was installed. Date of verification: 2026-08-13.

## Compatibility table

| Crate (version tested) | Number type(s) | Pure Rust? | Builds `wasm32-unknown-unknown`? | Builds native x86_64? | Notes |
|---|---|---|---|---|---|
| **`f64` (std)** | binary64 float | n/a (std) | ✅ | ✅ | Baseline. `sin/cos/tan/sqrt/ln/exp/powi` all compile on wasm32 with std — verified. Default fast path + graphing. |
| **`num-traits` 0.2.19** | traits (`Num`, `Float`, `Signed`, …) | ✅ | ✅ | ✅ | The abstraction layer everything else implements. Has `libm` feature for no_std transcendentals. MIT/Apache. |
| **`num-bigint` 0.4.8** | `BigInt`, `BigUint` | ✅ | ✅ | ✅ | Pin to **0.4** to match `num-rational` (0.4.2 pulls num-bigint 0.4; requesting 0.5 creates two versions). MIT/Apache. |
| **`num-rational` 0.4.2** | `Ratio<T>`, `BigRational` (= `Ratio<BigInt>`) | ✅ | ✅ | ✅ | Exact fractions. Version-coupled to `num-bigint` 0.4 + `num-integer`. MIT/Apache. |
| **`num-complex` 0.4.6** | `Complex<T>` (generic over `num-traits::Float`) | ✅ | ✅ | ✅ | Use `Complex<f64>` for graphing, or over the exact/decimal type. MIT/Apache. |
| **`rust_decimal` 1.42.1** | `Decimal` (96-bit int + scale → **28–29 sig. digits**) | ✅ | ✅ | ✅ | Fixed-precision decimal. `default-features=false, features=["std","serde"]` verified. Optional `wasm-bindgen` feature only for JS interop — not needed. MIT. |
| **`bigdecimal` 0.4.10** | `BigDecimal` (arbitrary-precision, `BigInt` mantissa + `i64` scale) | ✅ | ✅ | ✅ | True arbitrary precision, unlike rust_decimal. Slower, scaling can surprise. MIT/Apache. |
| **`malachite` 0.10.0** | `Natural`, `Integer`, `Rational`, `malachite-float` (MPFR-derived algorithms) | ✅ | ✅ | ✅ | Heavyweight pure-Rust bignum; competes with GMP-class perf. Needs rustc ≥ 1.90 (we have 1.97 ✅). **LGPL-3.0-only** — licensing caveat. |
| **`ibig` 0.3.6** | `UBig`, `IBig` | ✅ | ✅ | ✅ | Big-integer **only** (no rational/complex/float). MIT. Use `default-features=false` to drop the `rand`→`getrandom` path (see below). *(There is no `ugly` numerics crate — `cargo search` returns only an unrelated Solana tool; `ibig` is the intended crate.)* |
| **`rug` 1.30.0 / `gmp-mpfr-sys` 1.7.1** | `Integer`, `Rational`, `Float`, `Complex` via GMP/MPFR/MPC | ❌ C (GMP/MPFR/MPC) | ❌ **FAILS** | ✅ | **Confirmed the key negative result — see below.** |

**The verified negative result (rug / gmp-mpfr-sys):** On `wasm32-unknown-unknown`
it fails at the C build layer, two ways:

1. *Default:* the `gmp-mpfr-sys` build script refuses outright:
   `Cross compilation from x86_64-unknown-linux-gnu to wasm32-unknown-unknown not supported!`
2. *With `force-cross`:* it invokes GMP's own `configure --host wasm32-unknown-unknown`,
   which rejects the target: `Invalid configuration 'wasm32-unknown-unknown': OS 'unknown' not recognized`.
   GMP's autoconf cannot target wasm at all, so forcing cannot rescue it.

Native build **does** succeed (compiles `libgmp.a`/`libmpfr.a`/`libmpc.a` from
C source, ~2 min), but requires a host C toolchain including **`cc`, `m4`,
`make`** (this sandbox lacked `m4` until installed) — a real distribution burden.
**Conclusion: `rug`/`gmp-mpfr-sys` cannot be a `epher-core` dependency; the wasm
constraint is hard.** If GMP-class numbers are ever wanted, they must live in a
native-only crate `epher-core` does not link — and `malachite` is the wasm-safe
pure-Rust substitute.

## Flagged-dependency scan (the wasm32-unknown-unknown killers)

Scanned every candidate's `cargo tree` for the usual wasm pitfalls:

| Flag | Finding |
|---|---|
| **`getrandom`** | Pulled **only** via `rand` features: `ibig`'s default `rand`, `malachite`'s `random`, `rust_decimal`'s optional `rand`. Disabling those → **no getrandom in the tree** (verified clean). On wasm32-unknown-unknown, `getrandom` needs its `js`/`wasm_js` feature + `wasm-bindgen` or it errors/panics, so `epher-core` should keep randomness out entirely. |
| **`rayon`** | Not pulled by any candidate by default. rayon 1.12 *can* target wasm only via `web_spin_lock` **plus** `-C target-feature=+atomics,+bulk-memory,+mutable-globals` and JS glue; on stock `wasm32-unknown-unknown` it won't link. Keep `epher-core` sequential. |
| **`std::time::Instant`** | **Not** present in any candidate's default tree. (`Instant` is unavailable on wasm32-unknown-unknown — it would be a hard compile error.) Don't use it in core; take timing from the host. |
| **C / syscall deps** | Only `gmp-mpfr-sys` (and transitively `rug`). `libc` appears but compiles to empty stubs on wasm — harmless. |

## Recommended layered number model

A programmable calculator benefits from one **default fast path** plus
**opt-in exactness layers**, all surfaced as variants of a single `Value` that
the DSL evaluator produces. This keeps literals and graphing fast while letting
users opt into exactness where it matters.

**Layer 0 — Fast path: `f64`.** Default for literals, all arithmetic, and
**graphing**. Works on both targets with std (no `libm` needed — verified).
Abstracted through `num-traits` so the evaluator is generic.

**Layer 1 — Exact rational: `num-rational` (`BigRational` = `Ratio<BigInt>`)
+ `num-bigint` 0.4 + `num-traits` + `num-integer`.** Pin `num-bigint` to **0.4**
to keep the version-coupled `num-*` set unified. `1/3` stays exact. Pure Rust,
both targets, MIT/Apache, serde-friendly. Enter this layer when the user writes
a fraction literal or enables an `exact` mode.

**Layer 2 — Decimal: `rust_decimal` (default) / `bigdecimal` (when truly
arbitrary).** `rust_decimal` (96-bit, 28–29 digits) as the normal decimal mode —
fast, predictable, MIT. Escalate to `bigdecimal` only when the user asks for
arbitrary precision. Both pure Rust, both targets.

**Complex: `num-complex` (`Complex<T>`).** Generic over `num-traits::Float`, so
`Complex<f64>` for the fast path; wrap the exact/decimal types when needed.
Needed for graphing complex-valued functions and for fractals.

**Explicitly excluded from `epher-core`:** `rug` / `gmp-mpfr-sys` (wasm blocker,
see above). `malachite` is the wasm-safe fallback if GMP-class bignum performance
is ever required — accept its LGPL-3.0 caveat.

### Integration seam into the DSL evaluator

Put the crate choices behind **one** type + **one** trait so the grammar never
imports a numerics crate directly:

- `enum Value { Float(f64), Rational(BigRational), Decimal(Decimal), Big(BigDecimal), Complex(Complex<f64>) }` — the only number representation the evaluator's AST yields.
- A `Numeric` trait (extends `num_traits::Num` + `serde::Serialize/Deserialize`) that each variant implements; the AST's binary/unary nodes **promote operands to a common variant** (Float → Rational → Decimal → Big, plus Complex promotion) and then delegate to the trait.
- Every layer carries `serde` (Store persistence, single schema) — verified all chosen crates have a `serde` feature that compiles on both targets.
- Swapping `rust_decimal` ↔ `bigdecimal`, or adding `malachite` later, then touches only the `Value` enum + `Numeric` impls, not the evaluator grammar or the Store schema.

## wasm / native gotchas (affecting the choice)

1. **No GMP on wasm.** Any crate that ultimately needs a C library targeting the
   host OS is out for `epher-core`. `rug`/`gmp-mpfr-sys` is the canonical casualty.
2. **Disable `rand` features.** `ibig` (default), `malachite` (`random`),
   `rust_decimal` (optional) pull `getrandom`, which is broken on
   `wasm32-unknown-unknown` without `js`/`wasm_js` + `wasm-bindgen`. Keep
   `epher-core` deterministic; do randomness on the host.
3. **No threads / `Instant` / `rayon` in core.** `std::time::Instant` is absent
   on wasm32-unknown-unknown; rayon needs atomics + JS glue. Iterate sequentially;
   source timing from the host.
4. **f64 math is fine on wasm with std** (sin/cos/tan/sqrt/ln/exp verified) — so
   the graphing fast path needs nothing special. Only a *no_std* core would need
   `num-traits` + `libm`.
5. **`num-*` are version-coupled.** `num-rational` 0.4 → `num-bigint` 0.4;
   `num-complex` → `num-traits`. Pin `num-bigint` to 0.4 or you get two copies in
   the graph and cross-crate type mismatches.
6. **`rust_decimal` is fixed-precision (28–29 digits), not arbitrary.** It
   overflows/errors past ~7.9e28. Reach for `bigdecimal` when true arbitrary
   precision is required.
7. **`malachite` is LGPL-3.0** and needs rustc ≥ 1.90 (we have 1.97). `ibig` is
   MIT but integers only. Prefer the `num-*` set as the default; reach for
   `malachite`/`ibig` only for specific bignum needs.
8. **rug's *native* build needs `cc` + `m4` + `make`** and compiles GMP/MPFR/MPC
   from source (~2 min) — even before the wasm blocker it is a distribution cost
   on every native build host.
