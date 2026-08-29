# Rust crates for astronomy and ephemeris in `epher`

**Goal:** pick established Rust crates so epher can compute astronomy and
ephemeris quantities (Sun, Moon, planet positions and phenomena) fully offline,
with an MIT-licensed core (crates/core, f64 numerics) compiled for five
frontends (CLI, TUI, desktop via Tauri, web/PWA via trunk to
wasm32-unknown-unknown where bundle size matters, currently a few MB).

**Method (all facts checked, not assumed):** crates.io metadata (versions,
dates, licenses, downloads, dependency trees) pulled from the crates.io API on
**2026-08-29**. Source-level claims (features, accuracy notes, license texts,
CI targets) were read directly in shallow clones of the GitHub repositories:
astro-xao/sofars, ssmichael1/satkit, Protonmatter/sol, IsomorphicAlgo/ephemerust,
omkarium/astronav, qrichert/moontool, oliverkwebb/pracstro, nyx-space/anise,
nyx-space/hifitime, Razican/vsop87-rs, saurvs/astro-rust, coquinekitty/astronomy,
mourner/suncalc, Stellarium/stellarium. Kernel file sizes were verified with HTTP
HEAD requests against the hosting servers on 2026-08-29. wasm compatibility was
**verified by actually compiling**: cargo 1.97.1, target
wasm32-unknown-unknown, in a scratch project on 2026-08-29. No more than a
couple of minutes was spent on any compile.

---

## 1. What epher needs (constraints recap)

- Entirely offline at runtime: no network API calls when computing.
- MIT project: dependencies must be license-compatible (MIT, Apache-2.0,
  BSD, MPL-2.0 are fine per the mission brief; anything else needs a verdict
  below).
- Must work on wasm32-unknown-unknown (web/PWA) as well as native targets,
  so no libc, no rayon, no filesystem assumptions, no C compilation
  unless unavoidable.
- Bundle size matters for the web frontend: a +32 MB data kernel in a PWA
  is a different league from the current few MB bundle.
- Calculator-grade features first: JD/MJD, LST, alt/az, rise/set/transit,
  moon phase, planet positions; phenomena like conjunctions and eclipses are
  secondary (see section 7 for the feature survey this is grounded in).

---

## 2. Summary table

"DL" is total / recent-90-day downloads from the crates.io API (2026-08-29).
"wasm" reports the result of the actual compile test or, where not tested,
the documented dependency analysis.

| Crate | Version (date) | License (SPDX) | DL total/recent | wasm | Verdict for epher |
|---|---|---|---|---|---|
| solar-ephemeris | 0.2.0 (2026-07-20) | MIT OR Apache-2.0 | 57 / 57 | PASS (tested) | Strong candidate |
| anise | 0.10.6 (2026-08-12) | MPL-2.0 | 186,308 / 44,041 | PASS (tested, default-features off) | Strong candidate, heavy data |
| hifitime | 4.3.1 (2026-08-07) | MPL-2.0 | 1,145,100 / 273,915 | PASS (tested) | Supporting: time scales |
| satkit | 0.20.4 (2026-08-28) | MIT OR Apache-2.0 | 130,973 / 38,339 | FAIL (tested) | Native only |
| vsop87 | 3.0.0 (2023-10-22) | MIT OR Apache-2.0 | 44,834 / 6,378 | PASS (tested) | Candidate (planets only) |
| astro | 2.0.0 (2016-05-22) | MIT | 166,395 / 41,997 | PASS (tested) | Stale but useful Meeus set |
| sofars | 0.6.1 (2026-04-17) | MIT label + IAU SOFA terms | 39,070 / 25,554 | PASS (tested) | License needs human decision |
| astronav | 0.2.5 (2024-05-25) | MIT OR Apache-2.0 | 9,663 / 157 | PASS (tested) | Featherweight fallback |
| pracstro | 1.1.1 (2026-03-15) | 0BSD | 8,132 / 531 | PASS (tested) | Featherweight fallback |
| moontool | 1.4.0 (2026-07-12) | 0BSD | 12,392 / 160 | PASS (tested) | Featherweight fallback (Moon) |
| ephemerust | 0.7.0 (2026-07-29) | MIT | 124 / 124 | not tested (chrono dep, compiles on wasm in principle) | Too early |
| rust-jpl | 0.0.1-alpha (2026-01-19) | MIT | 103 / 62 | not tested | Alpha, not ready |
| nyx-space | 2.5.1 (2026-08-12) | AGPL-3.0-or-later | 76,874 / 2,149 | n/a | Disqualified (license) |
| meeus | does not exist | - | - | - | Not on crates.io (404) |
| meealgi | 0.0.6 (2017-10-01) | Apache-2.0 OR MIT | 8,460 / 26 | not tested | Stale |

Sources: [1] crates.io API, per-crate endpoints; repo URLs in section 3 and 4.

---

## 3. Deep dives: the crates named in the mission

### 3.1 anise

- Version 0.10.6, published 2026-08-12; license MPL-2.0. Repo:
  https://github.com/nyx-space/anise [1].
- What it is: "a modern replacement of NAIF SPICE" (crate description). Pure
  Rust reader/writer for SPK (.bsp), BPC (.bpc) and ANISE PCK (.pca) kernels;
  frame transformations; aberrated light-time; Sun, Moon (DE440 lunar gravity
  field data), planet and spacecraft positions straight from JPL DE kernels.
  The anise-gui subproject builds for wasm via trunk, documented in
  anise-gui/README.md of the repo [2].
- Bodies: Sun, Moon, 8 planets, Pluto, Earth-Moon barycenter from DE kernels;
  body constants (radii etc.) from PCK.
- Time: built on hifitime (TAI, TT, ET, TDB, UTC and more; see 4.2).
- Frames: ICRF/J2000 equatorial, IAU body-fixed frames, ecliptic frames via
  transforms; horizontal (alt/az) and rise/set/transit are NOT provided - that
  is observer-level logic epher would add itself.
- Accuracy: whatever the kernel gives; DE440/DE441 are the JPL reference
  ephemerides (sub-arcsecond class in the modern era; kernel header cites
  Park, Folkner, Williams, Boggs, "The JPL Planetary and Lunar Ephemerides
  DE440 and DE441", Astronomical Journal, DOI 10.3847/1538-3881/abd414;
  verified by reading the de440s.bsp header [3]).
- Wasm: compiles for wasm32-unknown-unknown with `default-features = false`
  (verified 2026-08-29, 27 s build). The default features `metaload` (pulls
  `ureq`, a network client, for remote kernel URLs) and `analysis` (pulls
  `rayon`, which does not target wasm) must both be switched off. There are
  explicit `cfg(not(target_arch = "wasm32"))` paths in the kernel I/O code
  (anise/src/naif/mod.rs, anise/src/naif/daf/file_record.rs) [2].
- Data/kernel size: needs a JPL kernel file at runtime or embedded. Sizes
  verified via HTTP HEAD: de440s.bsp = 32,726,016 bytes (about 32.7 MB, short
  span, roughly 1850-2150); de440.bsp = 119,799,808 bytes (about 120 MB,
  1550-2650), both at https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/ [3].
  The optional `embed_ephem` feature uses `rust-embed` to embed kernels into
  the binary (anise/Cargo.toml features section) [2].
- Network calls: only through the optional `metaload`/`embed_ephem` features
  (ureq); with `default-features = false` there is no network dependency.
- License caveat: anise 0.10.x depends on `sofars` 0.6.1 (non-optional, used
  in anise/src/orientations/dynamic.rs for SOFA-based orientation math) [2].
  That drags the IAU SOFA license terms (section 5.1) into any anise user.
- Maintenance: very active (multiple releases per month in 2026); same
  author/ecosystem as nyx-space and hifitime.

### 3.2 satkit

- Version 0.20.4, published 2026-08-28; license MIT OR Apache-2.0. Repo:
  https://github.com/ssmichael1/satkit, docs https://satkit.dev [1].
- What it is: satellite astrodynamics toolkit: SGP4/TLE, orbit propagation,
  gravity models, frame transforms (GCRF, ITRF), time systems.
- Bodies and phenomena (checked in source): `jplephem` module reads JPL
  DE440/DE441/de440s binary files directly (Chebyshev decoding, no SPICE),
  giving high-precision Sun, Moon, planets (src/jplephem.rs) [4]. `lpephem`
  gives low-precision analytic versions: sun position via Vallado Algorithm 29
  plus `riseset`, moon phase/illumination/phase_name, low-precision
  heliocentric planets (src/lpephem/) [4].
- Time scales: UTC, TT, UT1, TAI, GPS, TDB (src/time/timescale.rs) [4]. UT1
  needs EOP data files.
- Data files: gravity models (EGM96.gfc, 5.6 MB), the JPL ephemeris
  (linux_p1550p2650.440, 102,272,352 bytes, about 102 MB), EOP-All.csv
  (2.3 MB), SW-All.csv space weather (2.8 MB) and more, downloaded one-time
  from https://storage.googleapis.com/astrokit-astro-data by
  `update_datafiles()` (verified via bucket listing) [4]. The `download`
  default feature pulls `ureq`; with `default-features = false` the crate is
  offline but you must provision the data directory yourself.
- Wasm: FAILS to compile for wasm32-unknown-unknown (tested 2026-08-29): the
  `process_path` 0.1.4 dependency does not build for that target (its OS-gated
  `nix` module is empty there). There is no wasm target in its CI build
  workflow (.github/workflows/build.yml) [4].
- Verdict: excellent for native-only high-precision needs, but it cannot serve
  the web frontend, and its center of gravity (satellites, drag, gravity
  models) is wider than epher needs.

### 3.3 rust-jpl

- Version 0.0.1-alpha, published 2026-01-19; license MIT; repo
  https://github.com/chinmayvivek/rust-jpl [1].
- Reads NASA JPL DE441 ephemeris for planetary positions. Alpha quality: 103
  total downloads, single release, depends on the heavyweight `config` crate
  plus serde (crates.io dependency listing) [1].
- Verdict: not production ready; watch-list only.

### 3.4 sofars

- Version 0.6.1, published 2026-04-17; crates.io license field says MIT; repo
  https://github.com/astro-xao/sofars [1].
- Pure Rust implementation of the IAU SOFA library (iausofa.org): time scale
  conversions (UTC, TAI, TT, TDB, TCG, TCB), precession/nutation/polar motion
  (IAU 2000/2006), fundamental astrometry, calendar/JD conversions. Zero
  dependencies (crates.io) [1]. Compiles for wasm32-unknown-unknown (tested).
- What it does NOT provide: planet/moon/sun ephemeris positions, rise/set, or
  moon phase. It is the precision "plumbing" layer (time scales and frames)
  that Meeus-style algorithms or DE kernels sit on top of.
- License: this is the crate the mission flagged, and the concern is real.
  The LICENSE file starts with an MIT text ("Copyright 2025 Astro XAO") but
  then appends the full SOFA Software License (six terms, copyright Standards
  of Fundamental Astronomy Board of the IAU). The README states plainly:
  "Since the core algorithms are derived from the IAU SOFA source code, any
  use of this project must also comply with the SOFA license terms" and asks
  for IAU SOFA acknowledgment in published work or commercial products [5].
  Details in section 5.1.
- Maintenance: active, regular 2026 releases; also used by anise 0.10.x.

### 3.5 nyx-space

- Version 2.5.1, published 2026-08-12; license AGPL-3.0-or-later; repo
  https://github.com/nyx-space/nyx [1].
- Full astrodynamics/mission-design framework (orbit determination,
  propagation, rayon-parallel). GPL-family license is incompatible with
  embedding in a permissive MIT project without turning the whole thing
  AGPL. Also depends on anise, arrow/parquet, rayon - heavy.
- Verdict: disqualified on license. Not needed functionally either: it is
  mission-design machinery, not an astronomy calculator.

### 3.6 ephemerust

- Version 0.7.0, published 2026-07-29; license MIT; repo
  https://github.com/IsomorphicAlgo/ephemerust [1].
- "Teaching-grade" astronomy and satellite-tracking library and CLI, self-described
  middle ground between sgp4 and nyx-space, with a roadmap toward data
  services ("Phase 2 - API-based data access"). Provides (per its readme
  status table): Julian date and sidereal time, Sun/Moon position and
  rise/set, RA/Dec to alt/az, planet positions from truncated VSOP87D at
  about arcminute accuracy, planet rise/set [6].
- Dependencies include chrono, clap, sgp4, optional ureq (crates.io) [1].
- Verdict: charming and well-documented, but brand new (June 2026), 124
  downloads, single maintainer, teaching-grade accuracy, and a stated network
  direction. Not a foundation for epher.

### 3.7 vsop87

- Version 3.0.0, published 2023-10-22; license MIT OR Apache-2.0; repo
  https://github.com/Razican/vsop87-rs [1].
- Pure Rust VSOP87 implementation, all six variants (VSOP87, A, B, C, D, E):
  heliocentric positions of the 8 planets (plus Earth-Moon barycenter; VSOP87E
  is barycentric including the Sun). Accuracy documented in its README:
  under 1 arcsecond for 4000 years around J2000 for Mercury through Mars,
  2000 years for Jupiter/Saturn, 6000 years for Uranus/Neptune [7].
- No Moon (that is ELP territory), no Sun-as-seen-from-Earth convenience
  wrappers (trivial: Sun is at the origin of heliocentric frames; geocentric
  Sun is minus Earth), no rise/set, no moon phase, no time scales - you pass
  a Julian day.
- `no_std` feature via libm (Cargo.toml: `no_std = ["libm"]`) [7]; zero other
  deps. Compiles for wasm (tested). Crate source is large (4.9 MB compressed
  on crates.io) because the VSOP87 tables are embedded as Rust code; the
  compiled contribution is table-dominated but each planet's tables only link
  if used (per-function table constants), and LTO/`-C opt-level=z` shrink it
  further.
- Maintenance: last release 2023; acceptable because VSOP87 is a fixed,
  published theory - there is nothing to maintain except API polish.

### 3.8 astro (astro-rust)

- Version 2.0.0, published 2016-05-22 (ten years old); license MIT; repo
  https://github.com/saurvs/astro-rust [1].
- The most complete Meeus port in stable Rust: Julian day, sidereal time,
  dynamical time and Delta-T approximation, nutation, obliquity, equinoxes,
  rising and setting times, times of lunar phases, geocentric ecliptic
  positions of Sun and Moon, heliocentric positions of all planets (complete
  VSOP87 element set) plus Pluto, orbital elements, geodesic distances.
  README: "The main reference used as the source of algorithms is the famous
  book Astronomical Algorithms by Jean Meeus, whose almost every chapter has
  been addressed here", with tests using the book's example data [8].
- Zero dependencies; compiles for wasm (tested).
- Accuracy: Meeus-algorithm grade (arcminute class for most things; the README
  notes more accurate methods were implemented for Delta-T and planetary
  heliocentric positions) [8].
- Verdict: unmaintained since 2016, which for pure math is a low-dynamics
  risk (no CVE surface, no deps), but also means no wasm/no_std polish, no
  new Meeus chapters, and issue backlog forever. Usable, but epher would be
  absorbing a frozen codebase.

### 3.9 meeus (and similar)

- There is no crate named `meeus` on crates.io (API returns 404, checked
  2026-08-29) [1].
- Closest names: `meealgi` 0.0.6 (2017-10-01, Apache-2.0/MIT, "selective
  implementation of Jean Meeus' astronomical algorithms", 26 recent downloads,
  effectively abandoned) [1]; the Meeus content otherwise lives inside the
  crates above (astro, astronav, practical-astronomy-rust, moontool lineage,
  solar-ephemeris which cites specific Meeus chapters in its source).

---

## 4. Additional crates found on crates.io that fit the search

Search terms used: astronomy ephemeris, VSOP87, ELP2000, Meeus, sidereal,
julian date, SPICE, coordinates astronomy, moon phase, planet position, sun
position, rise set. Full candidates reviewed; the notable ones:

### 4.1 solar-ephemeris (the stand-out)

- Version 0.2.0, published 2026-07-20; license MIT OR Apache-2.0; 57 total
  downloads; compressed crate only 337 KB. Repo: https://github.com/Protonmatter/sol
  (workspace; the published crate is crates/solar-ephemeris) [1].
- Zero dependencies (empty [dependencies] section verified in its Cargo.toml;
  "Zero dependencies - it pulls in nothing else" per its README) [9].
- What it provides, from its README and source [9]:
  - VSOP2013 heliocentric positions for the Sun and 8 planets, ELP-MPP02 for
    the Moon, TOP2013 for the outer giants, valid about plus/minus 5000
    years, all as packed binary coefficient tables embedded in the crate.
  - Full topocentric reduction: light-time, aberration, Meeus Ch. 21 ecliptic
    precession, nutation, refraction, polar motion, and a complete
    Espenak-Meeus Delta-T era table (-500 to +2150, continuous to 0.26 s at
    the seams) spliced with measured IERS values near the present.
  - Rise/transit/set with body-specific thresholds, phase/illumination
    (moon), apparent visual magnitudes including Saturn's ring term (source
    cites Meeus Ch. 41), and a 108-star bright-star catalogue with proper
    motion.
  - Time scales module (timescales.rs) with TT/UT1/UTC handling.
- Accuracy: "validated against JPL Horizons (DE441) to arcsecond class", with
  a scheduled CI workflow re-checking RA/Dec, alt/az and Delta-T weekly
  (ephemeris-accuracy.yml in the repo) [9].
- Wasm: first-class. Builds as a raw cdylib for wasm32-unknown-unknown with a
  tiny extern "C" ABI (no wasm-bindgen, no bundler); README: "It powers the
  Solar Maximum Engine 'My Sky' and 'Solar System' surfaces in about 0.5 MB,
  coefficient tables included" [9]. Compile test passed (2026-08-29).
- Network: none possible (zero dependencies).
- Honest limits, stated in its README: accuracy envelopes shrink outside the
  validated era; deep-time positions are Delta-T-limited; "This is a
  research/learning engine" [9].
- Risks: brand new (first release June 2026 era), one maintainer, tiny user
  base; API is young (0.x); the crate-type includes a cdylib (harmless but
  unusual for a lib dependency). Mitigation: wrap it behind an
  `epher-core` astronomy module, pin the version, and be ready to vendor.

### 4.2 hifitime (time scales foundation)

- Version 4.3.1, published 2026-08-07; license MPL-2.0; 1.14 million total
  downloads; repo https://github.com/nyx-space/hifitime [1].
- Ultra-precise epoch/duration handling with explicit time scales (TAI, TT,
  ET, TDB, UTC with embedded leap second table, GPS), nanosecond precision
  over tens of millennia; formally verified with the Kani model checker per
  its README [10].
- Wasm is a supported target: Cargo.toml carries
  [target.wasm32-unknown-unknown.dependencies] entries for wasm-bindgen [10];
  compile test passed.
- UT1: supported via an optional `ut1` feature which pulls `ureq` (Earth
  rotation data fetch); offline builds should avoid that feature or supply
  the data themselves (Cargo.toml features: `ut1 = ["std", "ureq", "tabled"]`)
  [10].
- It is the time engine underneath anise, and the natural JD/MJD and
  TT/UTC/TDB backbone if epher wants typed time scales rather than raw f64
  days.

### 4.3 Small Meeus-grade crates (featherweight tier)

- astronav 0.2.5 (2024-05-25, MIT OR Apache-2.0, zero deps): JD, GMST/LMST,
  sun position and rise/set, alt/az of objects. Its README is refreshingly
  honest: sun rise/set and angles "typically see up to 2 mins of variation
  when compared with ... Stellarium" [11]. Compiles for wasm (tested).
- pracstro 1.1.1 (2026-03-15, 0BSD, zero deps): compact sun/planet/moon/star
  properties: horizon coordinates, rise/set (`Coord::riseset` in
  src/coord.rs), precession, moon phase age/illumination fraction/phase
  angle; benchmarks itself at microseconds per full ephemeris; the repo even
  ships a wasm build used by its sibling projects [12]. Compiles for wasm
  (tested).
- moontool 1.4.0 (2026-07-12, 0BSD): John Walker's moontool.c astronomical
  routines ported to Rust: moon age, phase name/angle, illuminated fraction,
  distance, plus sun calculations; original is public domain ("Do what thou
  wilt", per the README quoting Walker) [13]. Compiles for wasm (tested).
- astro 2.0.0 (section 3.8) also belongs to this accuracy tier.
- julian 0.7.1 (2025-06-27, MIT): Julian day number conversions for Gregorian
  and Julian calendars (repo https://github.com/jwodder/julian-rs) [1].
- sun 0.3.1 (MIT, zero deps) and suncalc 0.4.0 (MIT): single-purpose sun
  position/rise/set and sunlight phases; suncalc is a port of the SunCalc
  JavaScript library (moon position and illumination included) [1].
- flare 0.2.0 (2025-04-24, MIT, deps chrono + rand): astropy-inspired observer
  math: sun rise/set, civil/nautical/astronomical twilight, airmass, angular
  separation (repo https://github.com/boom-astro/flare) [1].
- astronomy 0.1.5 (2025-06-22, MIT OR Apache-2.0): early-stage general
  astronomy calculations (repo https://github.com/open-physics/astronomy) [1].

### 4.4 SPICE-adjacent (mostly not suitable)

- rust-spice 0.7.8 (2023-12-18, Apache-2.0 wrapper): binds the C SPICE
  toolkit (cspice-sys, libc). Last release 2023; C code, libc dependency;
  NAIF's toolkit terms are permissive but it is not a pure-Rust wasm story
  [1].
- spicekit 0.2.2 (2026-05-05, MIT): pure-Rust SPICE kernel reader (DAF, SPK,
  PCK) from B612 Asteroid Institute, but depends on memmap2 and rayon, so no
  wasm32-unknown-unknown [1].
- astrodyn_ephemeris 0.2.0 (2026-06-09, MIT OR Apache-2.0): DE4xx reader
  built on top of anise anyway [1].
- astronomy-engine-bindings 2.1.19 (2024-11-05, MIT): bindgen/cc/libc bindings
  to the C port of Astronomy Engine (upstream MIT, repo
  https://github.com/cosinekitty/astronomy, last pushed 2025-01-27). The
  upstream library itself is superb (section 7.1) but the Rust access path is
  C-compile based, which is exactly what epher's wasm build does not want
  [1].

### 4.5 Crates disqualified on license

- siderust 0.11.0 (2026-07-04): AGPL-3.0-only [1].
- tempoch 0.6.5 (2026-06-21): AGPL-3.0-only [1].
- vedaksha-ephem-core 7.6.0: BUSL-1.1 (Business Source License, not open
  source; production-use restrictions) [1].
- swisseph-rs 0.2.0 (2026-08-12): AGPL-3.0-or-later, consistent with Swiss
  Ephemeris's own licensing since 2.10 [1].
- nyx-space 2.5.1: AGPL-3.0-or-later (section 3.5) [1].

---

## 5. License compatibility verdicts for an MIT project

### 5.1 The sofars / IAU SOFA question (needs a human decision)

What the sofars package actually ships (read from its LICENSE file and README
at v0.6.1) [5]:

1. A standard MIT permission statement ("Copyright 2025 Astro XAO").
2. Appended below it, the full SOFA Software License, copyright the Standards
   of Fundamental Astronomy Board of the IAU, with six terms. The operative
   restrictions for a derived work (which sofars itself is, and which any
   epher build including it would also be, per term 3e which requires these
   conditions to be reproduced intact for downstream receivers):
   - The work must carry a statement that it (i) uses routines and
     computations derived from SOFA software and (ii) is not software
     provided by or endorsed by SOFA (term 3a).
   - The source must contain descriptions of how the derived work is based
     upon, contains and/or differs from the original SOFA software (term 3b).
   - Routine names must not include the prefix "iau" or "sofa" (term 3c).
     sofars complies (its API uses plain names like `era00`, `ts`, `pnp`).
   - Origin must not be misrepresented; no patent filings on SOFA algorithms
     (term 3d).
   - No warranty, no liability (term 5); acknowledgment appreciated in
     published works or commercial products (post-term-6 note).
3. The crate's README repeats: algorithms are derived from IAU SOFA source,
   so users must comply with SOFA terms and acknowledge IAU SOFA [5].

Assessment for epher: SOFA is free-of-charge and royalty-free including for
commercial use, but it is NOT an OSI-approved license and it is not
relabelable as MIT. The `license = "MIT"` in sofars' Cargo.toml understates
the attached SOFA conditions. Shipping sofars inside MIT-licensed epher is
probably workable (keep the SOFA notice and derived-work statements in the
distribution, add an acknowledgment), but it imposes ongoing packaging
obligations on every distribution channel (crate, GitHub, wasm bundles
shipped from a website) and it is a judgment call a human should sign off on.
Flag: YES, needs human decision.

Transitive exposure: anise 0.10.x depends on sofars 0.6.1 as a hard
dependency (anise/Cargo.toml; used in src/orientations/dynamic.rs) [2], so
adopting anise adopts the same SOFA terms transitively. There is no anise
feature to turn it off (it is the orientation-model backend).

### 5.2 Everything else

- Clean, no-decision-needed licenses: MIT (astro, moontool-adjacent crates,
  ephemerust, rust-jpl, spicekit, flare, julian, sun, suncalc, arika,
  practical-astronomy-rust); MIT OR Apache-2.0 / Apache-2.0 OR MIT
  (solar-ephemeris, vsop87, satkit, astronav, pracstro-adjacent, astronomy,
  astrodyn_ephemeris, meealgi); MPL-2.0 (anise, hifitime, skymath) - MPL-2.0
  is file-level copyleft, explicitly fine to link from MIT code.
- 0BSD (moontool, pracstro): public-domain-equivalent, unconditionally fine.
- Hard disqualifiers (copyleft or commercial-use restrictions): AGPL-3.0
  (nyx-space, siderust, tempoch, swisseph-rs), BUSL-1.1
  (vedaksha-ephem-core).
- CSPICE via rust-spice: Apache-2.0 wrapper, but the bundled C toolkit is
  US-government-work software with its own (permissive but non-standard)
  terms; adding C also breaks the no-compile-for-wasm preference. Avoid.

---

## 6. wasm32-unknown-unknown evidence

Compile tests run 2026-08-29 with cargo/rustc 1.97.1 on x86_64 Linux, target
wasm32-unknown-unknown (installed via rustup). One scratch project per group;
times are wall-clock for a debug build:

- PASS (28.9 s group build): solar-ephemeris 0.2.0, vsop87 3.0.0, sofars
  0.6.1, astro 2.0.0, astronav 0.2.5, pracstro 1.1.1, moontool 1.4.0,
  hifitime 4.3.1 (default-features = false).
- PASS (27.1 s): anise 0.10.6 with default-features = false.
- FAIL (7.3 s): satkit 0.20.4 with default-features = false: process_path
  0.1.4 fails (`mod nix` gated to OS targets, wasm has none). Non-optional
  dependency, so satkit is wasm-incompatible today.

Dependency-level red flags to keep avoiding on wasm: ureq (network; anise
metaload feature, satkit download feature, hifitime ut1 feature), rayon
(anise analysis feature, spicekit), memmap2 (spicekit; anise carries it but
compiles anyway), libc/cc/bindgen (rust-spice, astronomy-engine-bindings).

---

## 7. What do astronomy calculators actually compute? (feature grounding)

### 7.1 Consumer and prosumer tools

- SunCalc (the de-facto standard open-source sun/moon calculator for apps,
  https://github.com/mourner/suncalc): sun position (apparent altitude with
  refraction, azimuth), sunlight phases (sunrise, sunset, dawn, dusk, golden
  hour, blue hour, solar noon), moon position, moonrise/moonset, moon
  illumination fraction. Its README states it "matches the accuracy and
  conventions of timeanddate.com and the U.S. Naval Observatory" and that the
  formulas come from Meeus' Astronomical Algorithms [14].
- Astronomy Engine (https://github.com/cosinekitty/astronomy, MIT, no
  external deps, minified JS under 120 KB, claims accuracy within 1 arcminute
  of NOVAS) is effectively the maximal sensible feature list for an offline
  engine [15]:
  - Sun, Moon, Mercury through Pluto: heliocentric and geocentric vectors;
  - topocentric horizon (alt/az) positions for any observer;
  - rise, set, culmination times; civil, nautical, astronomical twilight;
  - moon phases (new/first quarter/full/third quarter date search);
  - lunar and solar eclipses; transits of Mercury and Venus;
  - lunar apogee/perigee; equinoxes and solstices;
  - apparent visual magnitudes; conjunctions, oppositions, apsides dates;
  - maximum elongations of Mercury and Venus;
  - Jupiter's four Galilean moons; lunar libration; constellation of a point;
  - coordinate frames: J2000 equatorial, equator-of-date, ecliptic J2000,
    topocentric horizontal, galactic (IAU 1958).
- Stellarium (https://github.com/Stellarium/stellarium), the reference
  desktop planetarium: full sky rendering plus precise Delta-T algorithms,
  including the Meeus (1998, Astronomical Algorithms 2nd ed.) Delta-T
  algorithm with Chapront et al. extrapolation, selectable by the user
  (src/core/StelCore.cpp, StelUtils.cpp) [16].
- The pattern: calculators cluster into (a) time primitives (JD/MJD, time
  scales), (b) Sun/moon daily phenomena (rise/set/twilight/phase), (c)
  positions in several frames (ecliptic, equatorial, horizontal; LST as the
  bridge), (d) planet positions and pairwise phenomena (conjunction,
  opposition, elongation), (e) rare events (eclipses, transits).

### 7.2 Meeus, Astronomical Algorithms: what the ecosystem actually cites

Primary-source citations found in the surveyed code (rather than a possibly
inaccurate restatement of the book's full TOC):

- Publisher page confirms the reference edition: 2nd edition, 1999, 477
  pages, hardbound (https://shopatsky.com/products/astronomical-algorithms-2nd-edition)
  [17].
- solar-ephemeris source cites, with formula numbers: Ch. 11 (observer
  geocentric quantities on the WGS84 ellipsoid), Ch. 13 (ecliptic/equatorial
  conversion and obliquity, formulas 13.1-13.4), Ch. 16 (refraction, 16.4),
  Ch. 21 (ecliptic precession), Ch. 40 (geocentric to topocentric, diurnal
  parallax), Ch. 41 (apparent visual magnitudes, including Saturn's ring
  brightening), plus Espenak-Meeus Delta-T tables [9].
- astro-rust README: "almost every chapter has been addressed", tests use
  the book's example data [8].
- Stellarium implements the book's Delta-T algorithm [16]; SunCalc builds its
  sun/moon routines on the book [14].
- For epher's scope, the convergent "most notable" chapters across these
  tools are exactly: Julian day and calendar (foundations), dynamical time
  and Delta-T, sidereal time, coordinate transformations (ecliptic,
  equatorial, horizontal, precession, nutation, refraction), rising/transit/
  setting, solar coordinates, lunar position/phase/illumination, planetary
  positions (VSOP87-based), and apparent magnitudes.

### 7.3 Implications for epher's feature scope

- Tier 1 (calculator core): JD/MJD, GMST/LMST, Delta-T/TT-UTC, alt/az from
  RA/Dec, sun rise/set/twilight, moon rise/set, moon phase and illumination.
  Every tool surveyed has these; in Rust the coverage exists in multiple
  crates.
- Tier 2: planet geocentric/ecliptic positions, apparent magnitudes,
  conjunction/opposition search. VSOP87/VSOP2013 covers positions; pairwise
  phenomena search is application code on top (no surveyed crate provides a
  ready conjunction finder).
- Tier 3: eclipses, transits, apsides, Galilean moons. Only Astronomy Engine
  provides these as a coherent set, and its Rust path is C-bindings; epher
  would implement Meeus's eclipse chapters itself if wanted.

---

## 8. Shortlist recommendation

Two candidates plus one supporting crate:

### 8.1 solar-ephemeris - recommended primary

- Why: the only surveyed crate that bundles positions AND phenomena (rise/
  transit/set, moon phase, magnitudes) with arcsecond-class validation
  against JPL Horizons, zero dependencies, tables embedded (no kernel files),
  no network, first-class wasm32-unknown-unknown with about 0.5 MB binary
  including tables (its claim; consistent with a 337 KB compressed crate),
  MIT OR Apache-2.0. It is practically shaped like epher's need: offline,
  small, multi-frontend.
- Role: full engine behind `epher-core` astronomy Expressions/Functions on
  all five frontends.
- Open risks: 5 weeks old, one maintainer, 57 downloads; 0.x API; accuracy
  envelopes shrink outside its validated era; if it stalls, epher must vendor
  or fork. Mitigate: pin exact version, isolate behind a facade module in
  epher-core, keep a vsop87+astro fallback plan.

### 8.2 anise - recommended high-precision option

- Why: the most credible long-term Rust ephemeris infrastructure (active
  maintainer, JPL DE440 kernels, pure Rust, wasm-capable when
  default-features are off, MPL-2.0). Gives epher a SPICE-grade path for
  desktop/CLI, and the embed_ephem feature can embed kernels into the binary.
- Role: optional high-precision backend, most realistic for desktop/CLI
  where a 32.7 MB de440s.bsp (or 120 MB de440.bsp) is acceptable; not for the
  PWA.
- Open risks: 32 MB+ kernel data is a non-starter for the web bundle;
  observer-level phenomena (rise/set/transit, moon phase) still have to be
  built on top; transitive sofars dependency pulls the SOFA license terms
  (section 5.1) into the dependency tree.

### 8.3 hifitime - supporting crate for time scales

- Why: if epher wants typed time scales (UTC/TT/TDB, leap seconds, JD/MJD)
  rather than raw f64 arithmetic, this is the maintained, wasm-tested,
  MPL-2.0 standard (1.1M downloads), and it is already anise's time engine.
- Caveat: its `ut1` feature wants to download Earth-rotation data via ureq;
  offline usage should stay on TT/UTC or bundle a DUT1 table. Not needed if
  solar-ephemeris's internal timescales suffice.

Featherweight fallback (if the human decision rejects young dependencies):
vsop87 (planets, MIT/Apache, no_std) + astro or astronav (Meeus algorithms:
JD, LST, rise/set, moon phase) + moontool (moon detail), all zero-dependency,
wasm-verified, total addition well under a megabyte. Accuracy arcminute-ish,
and more glue code for epher to write.

Not recommended despite activity: satkit (wasm-broken via process_path,
102 MB data file, satellite-centric), ephemerust/rust-jpl (too immature),
everything in 4.5 (licenses).

---

## 9. Questions that need a human decision

1. SOFA license (blocking for anise and sofars): is epher willing to ship
   IAU SOFA-derived routines under its MIT umbrella, keeping the SOFA notice,
   the derived-work statement and the no-endorsement disclaimer in every
   distribution? If not, anise 0.10.x and sofars are out; solar-ephemeris and
   vsop87 are unaffected.
2. Web bundle budget: is +0.5 MB (solar-ephemeris) the ceiling for the PWA,
   or is there appetite for larger? This decides whether anise (32.7 MB
   kernel minimum for planets+moon+sun) can ever serve the web frontend or
   only native ones.
3. Accuracy tier: arcminute (Meeus/calculator grade, featherweight crates) or
   arcsecond (VSOP2013/ELP-MPP02, DE440)? This sets the shortlist order and
   whether epher needs both a light and a heavy backend behind one facade.
4. Time span: 1850-2150 (de440s), 1900-2100 (solar-ephemeris's Delta-T era
   table runs -500 to +2150), or about plus/minus 5000 years (VSOP87/
   solar-ephemeris positions, Deep-time accuracy caveats)? Historical-date
   users push this.
5. Risk posture on young crates: adopt solar-ephemeris at 0.2.0 with a
   vendoring plan, or start featherweight and revisit? Conversely: is
   accepting the ten-year-old, unmaintained astro acceptable as fallback?
6. UT1 policy: skip UT1 (UTC/TT only) in v1, or bundle a DUT1/EOP table into
   the offline data? Affects hifitime feature choice and any rise/set
   accuracy claims.
7. Feature scope for v1: confirm Tier 1 (JD/MJD, LST, alt/az, rise/set,
   twilight, moon phase) from section 7.3, and whether Tier 2 planets and
   Tier 3 eclipses are roadmap or out of scope.

---

## Sources

All URLs accessed and verified 2026-08-29 unless noted.

[1] crates.io API, per-crate and search endpoints, e.g.
    https://crates.io/api/v1/crates/anise ,
    https://crates.io/api/v1/crates/satkit ,
    https://crates.io/api/v1/crates/sofars ,
    https://crates.io/api/v1/crates/vsop87 ,
    https://crates.io/api/v1/crates/astro ,
    https://crates.io/api/v1/crates/hifitime ,
    https://crates.io/api/v1/crates/solar-ephemeris ,
    https://crates.io/api/v1/crates/nyx-space ,
    https://crates.io/api/v1/crates/ephemerust ,
    https://crates.io/api/v1/crates/rust-jpl ;
    `meeus` returns 404: https://crates.io/api/v1/crates/meeus
[2] anise repository: https://github.com/nyx-space/anise
    (anise/Cargo.toml for features and the sofars dependency;
    anise/src/naif/mod.rs and anise/src/naif/daf/file_record.rs for the
    wasm32 cfg paths; anise-gui/README.md for the trunk/wasm GUI;
    README.md lines about de440s.bsp/de440.bsp/pck08.pca and 100,000-query
    DE440.bsp benchmark)
[3] JPL SSD FTP, binary kernels:
    https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/de440s.bsp (32,726,016 bytes)
    and https://ssd.jpl.nasa.gov/ftp/eph/planets/bsp/de440.bsp
    (119,799,808 bytes); DE440 header citing Park, Folkner, Williams, Boggs,
    AJ, DOI 10.3847/1538-3881/abd414
[4] satkit repository: https://github.com/ssmichael1/satkit
    (src/time/timescale.rs for TimeScale; src/lpephem/sun.rs and
    src/lpephem/moon.rs for riseset/phase/illumination; src/jplephem.rs for
    direct DE440 reading; src/utils/update_data.rs for the data bucket;
    .github/workflows/build.yml for CI targets; Cargo.toml for features)
    and https://storage.googleapis.com/astrokit-astro-data/ (bucket listing:
    linux_p1550p2650.440 = 102,272,352 bytes, EGM96.gfc, EOP-All.csv,
    SW-All.csv)
[5] sofars repository: https://github.com/astro-xao/sofars
    (LICENSE file: MIT text followed by the complete IAU SOFA Software
    License, six terms; README.md "License and SOFA Terms of Use" section)
[6] ephemerust repository: https://github.com/IsomorphicAlgo/ephemerust
    (readme.md status table: truncated VSOP87D about arcminute accuracy,
    rise/set coverage, phase 2 data-services roadmap)
[7] vsop87 repository: https://github.com/Razican/vsop87-rs
    (README.md accuracy paragraph; Cargo.toml no_std feature)
[8] astro-rust repository: https://github.com/saurvs/astro-rust (README.md
    feature list and Meeus attribution)
[9] solar-ephemeris: https://github.com/Protonmatter/sol
    (crates/solar-ephemeris/README.md, Cargo.toml, src/coords.rs and
    src/physics.rs for the Meeus chapter citations)
[10] hifitime repository: https://github.com/nyx-space/hifitime
     (README.md, Cargo.toml features and wasm target sections)
[11] astronav repository: https://github.com/omkarium/astronav (README.md,
     including the 2-minute accuracy notice)
[12] pracstro repository: https://github.com/oliverkwebb/pracstro
     (README.md benchmarks; pracstro/src/coord.rs riseset; wasm workspace
     member)
[13] moontool repository: https://github.com/qrichert/moontool (README.md,
     Walker public-domain quote; others/rust/ for the Rust port)
[14] SunCalc: https://github.com/mourner/suncalc (README.md: feature list,
     timeanddate/USNO conventions, Meeus basis)
[15] Astronomy Engine: https://github.com/cosinekitty/astronomy (README.md
     feature list, no-dependency and accuracy goals)
[16] Stellarium: https://github.com/Stellarium/stellarium
     (src/core/StelCore.cpp, src/core/StelUtils.cpp: Meeus 2nd ed. Delta-T)
[17] Astronomical Algorithms, 2nd ed.: publisher page
     https://shopatsky.com/products/astronomical-algorithms-2nd-edition
     (477 pages, 2nd edition 1999)
