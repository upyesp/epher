# vendored solar-ephemeris 0.2.0

A vendored copy of `solar-ephemeris` 0.2.0 from crates.io (MIT OR
Apache-2.0, https://github.com/Protonmatter/sol), pinned by epher's
facade rule (ADR-0037: `solar-ephemeris = "=0.2.0"`).

## Why vendored

The ephemeris facade in `crates/core/src/astro.rs` computes each body's
place directly, mirroring the crate's private `topocentric_sky_at_time`
op for op, so the astronomy scripts pay microseconds per call instead of
a full sky snapshot (every body's place plus every body's rise/transit/
set event scan) per accessor. The Sun and Moon have complete public
paths; the planets need one more piece the 0.2.0 crate keeps private:
the VSOP2013 table handle (`Body::elements` and the `vsop2013_data`
module). Upstream 0.2.0 exposes `planets::planet_apparent_ecliptic` but
no way to obtain its `&Planet` argument from outside the crate.

## The diff from crates.io 0.2.0

Visibility only, two lines in `src/lib.rs`; no algorithm, table, or
formatting byte is touched:

1. `mod vsop2013_data;` is `pub mod vsop2013_data;`
2. `impl Body`'s `fn elements` is `pub fn elements`

Everything else is byte-identical to the registry source. The
`astro_direct` test suite in `crates/core` pins the facade's output
bit-for-bit against the crate's own snapshot JSON, so any drift between
this copy and a future upstream fails loudly in tests.

If upstream publishes a release that makes the table handle public,
delete this vendor directory and return to the `=0.2.0` version pin.
