# ADR-0053: Crate-reuse ladder and the guide on demand

Date: 2026-09-02 · Status: accepted

## Context

The nine-work-package stretch (0042-0052) grew the source fast, and every
frontend grew with it: the web bundle passed 2.9 MB and every native binary
carried a half megabyte of markdown nobody was reading at that moment. Two
directives came out of that review:

1. **Reuse first** - before writing Rust, use what the project's own crates
   already provide; then a public crate; new code is the last resort. This
   is a standing architectural requirement, not a one-off cleanup.
2. **The guide is data, not code** - the embedded user guide must not be
   compiled into the apps. It should be stored where the app can load it
   only when the user asks for it.

## Decision

### The reuse ladder

Every need is met at the lowest possible rung, in order:

1. **Another crate in this workspace** - shared code lives behind a crate
   boundary (`epher-core` numerics, `epher-shell` command kernel,
   `epher-i18n`, `epher-guide` renderers); frontends only compose.
2. **A public crate from crates.io** - wasm32-unknown-unknown-safe
   (pure Rust, no C, no threads), MIT/Apache/BSD/MPL licensed, and it must
   beat or match what we would write. Verified by building against both
   targets, not by README claims.
3. **Our own code** - only when the ladder runs out, and the reason gets
   written down (this ADR or a research note in `docs/research/`).

### What moved down a rung

- **Special functions → `puruspe` 0.4** (MIT OR Apache-2.0; pure Rust;
  dependency tree: `lambert_w`, `num-complex`, `num-traits` - all
  wasm-safe, verified by compiling both targets). The in-house
  Numerical-Recipes ports (`gamma_series`, `gamma_cf`, `beta_cf`, the
  6-coefficient Lanczos `ln_gamma`) are deleted; `regularized_gamma_q`
  → `puruspe::gammq`, `regularized_beta` → `puruspe::betai` (behind the
  same clamping wrappers so the crate's domain asserts never fire),
  `ln_gamma` → `puruspe::ln_gamma` (Fukushima-class, tighter than our
  old ~1e-12 Lanczos), and `norm_cdf`/`inv_norm` now run on
  `puruspe::erfc` (libcerf-derived erfcx form - tail-preserving to
  underflow, better than the old gamma identity's ~1e-12). The
  inversion layer stays ours by decision: `invert_cdf`'s survivor-space
  Newton with doubling brackets is the ADR-0052 accuracy work, not
  duplicated functionality. All 194 core tests pass unchanged, including
  every ADR-0044/0052 anchor (`invt(0.9999, 1)` = 3183.09875712,
  `invnorm(0.975)` = 1.95996398454, `ttest`/`tinterval` against the
  R-validated values).
- **The guide out of every binary.** `epher-guide` keeps the renderers
  and loses the `include_str!`. Trunk's `copy-dir` asset reads the
  committed `site/guide/*.md` (the single source of truth) straight
  into `dist/guide/`, the Tauri shell embeds the same dist, so
  web/PWA and desktop fetch `guide/<locale>.md` on first open and the
  service worker runtime-caches it for offline. The installers carry the
  eight files as Tauri bundle resources; the TUI reads them at open
  (`epher_guide::load`) through a search that covers `$EPHER_GUIDE_DIR`,
  the NSIS layout, the macOS `Contents/Resources`, the Linux
  `/usr/lib/<name>/resources`, the system data dirs, and a user data
  dir. Nothing renders "the guide is missing" as a blank page: the web
  overlay shows a localized notice with the epher.org URL, the TUI pager
  lists every directory it tried.

### What stays ours, and why (the last-resort rung, recorded)

- **`num-bigint`/`num-rational`/`num-complex`/`rust_decimal`/
  `bigdecimal`** are already public crates doing the exact layers
  (ADR-0005, ADR-0043). `bigdecimal` in particular is not redundant with
  `num-rational`: `big(0.1) + big(0.2)` must print `0.3`, and
  `big(10) ^ 40` must stay exact past rust_decimal's 96-bit range -
  decimal semantics at arbitrary precision, which a rational cannot
  display.
- **`solar-ephemeris`** - the JPL-validated ephemeris core was already a
  public-crate pick; `astro.rs` adds only the phenomena layer on top
  (moon phase times, rise/set, watches). The 2026-08-29 survey
  (`docs/research/astro-crates.md`) is why nothing else qualified.
- **Seed randomness** (`splitmix`), the numeric solvers (`invert_cdf`,
  `durand_kerner`, `gauss_jordan`, `adaptive_simpson`), unit parsing,
  and the graph sampler stay in-house: each is either the product
  itself (seeded reproducibility across frontends), too small for a
  dependency to pay for, or has no wasm-safe crate match
  (`statrs`/`nalgebra` pull in far more than they replace;
  `getrandom`-based `rand` breaks both wasm and reproducibility;
  `uom` outweighs the unit grammar). The 2026-08-13 numerics survey
  (`docs/research/numerics-options.md`) holds the verified negatives.

## Consequences

- The web bundle drops from 2,990,065 to 2,434,918 bytes (-18.6%); the
  TUI binary from 4,872,416 to 4,323,944 (-11.3%). Load time improves
  for every visitor; the guide costs nothing until opened.
- A guide edit no longer requires recompiling anything: the website
  picks it up on the next pages build, the apps on their next bundle.
- The installers grow by ~560 KB of markdown resources, and the release
  workflow now verifies they shipped (dist has 8 locale files, the
  binary carries none, deb/rpm list the guide).
- Special-function quality upgrades silently: better `ln_gamma` and
  tail behavior come with the crate instead of from our hand-tuning.
- Future contributions inherit the ladder: a new feature starts at
  rung 1, and skipping to rung 3 needs a written reason.
