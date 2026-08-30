# ADR-0037: Astronomy units, constants, time functions, and ephemeris (v0.5.0)

- Status: accepted
- Deciders: epher maintainers
- Date: 2026-08-29

## Context

The project took on two overlapping feature areas for one release: astronomy
calculator functions (specialized units, constants, time and coordinate math,
optics formulas) and ephemeris (positions of the Sun, Moon and planets plus
phenomena), fully offline, on all five frontends, with minimal core
architecture change. The ground rules from the mission brief: use established
Rust crates rather than rewriting tested algorithms, keep MIT license
discipline, keep the DSL (ADR-0004) and the single `Value` number model
(ADR-0005) intact unless the maintainers explicitly sign off otherwise, and
draw on the notable chapters of Meeus's *Astronomical Algorithms*.

The crate survey (docs/research/astro-crates.md, 2026-08-29: crates.io
metadata, source-level inspection of the candidates, actual
wasm32-unknown-unknown compile tests, HTTP-verified kernel sizes) grounded the
backend choice:

- `anise` (MPL-2.0, active) is the credible SPICE path, but it needs a 32.7 MB
  `de440s.bsp` kernel, provides positions only (rise/set, phase and
  magnitudes would be ours to build), and hard-depends on `sofars`, which
  ships the six-term IAU SOFA license beneath its MIT label. SOFA-derived
  code is excluded by the project's license allowlist, so anise is out.
- `solar-ephemeris` 0.2.0 (MIT OR Apache-2.0, zero dependencies) embeds
  VSOP2013, ELP-MPP02 and TOP2013 tables (about 0.5 MB in wasm), is validated
  arcsecond-class against JPL Horizons by a recurring CI check, and already
  includes the observer-level pieces: topocentric reduction, rise/transit/set,
  moon phase and illumination, apparent magnitudes, and Espenak-Meeus
  Delta-T.
- `satkit` fails to compile for wasm; the AGPL crates are disqualified; the
  featherweight tier (`vsop87` + `astro`, arcminute grade) remains the
  documented fallback.

Language facts that shaped the surface: no string literals, no tuples, no
implicit multiplication (`2 pi` is a parse error), display-only `Str` (the
`bin`/`oct`/`hex` mechanism), and name resolution order user variable, user
constant, builtin (so builtins are shadowable, exactly like `pi` today).

## Decision

### One release

The calculator functions, the ephemeris, and the 3D solar system view (the
latter rendered by the ADR-0015 amendment's space curves and positioned
points) ship together as v0.5.0.

### Unit literals: grammar sugar onto SI factors

- A number immediately followed by a unit suffix is a **unit literal**: it
  multiplies by the unit's SI factor and evaluates to a plain `Float` in SI
  units. `3.2 AU` is metres, `30 deg` is radians (`sin(30 deg)` works), `5 hr`
  is seconds, `5 Jy` is watts per square metre hertz. No `Value` variant, no
  dimension checking, no unit-carrying display: ADR-0005's single number
  representation stands.
- Token table: length `AU`/`au`, `pc`, `ly` (to metres); angle `deg`,
  `arcmin`, `arcsec` (to radians); time `min`, `hr`, `d`, `yr` (to seconds;
  `yr` is the Julian year, 365.25 d); flux `Jy` (to 1e-26 W m^-2 Hz^-1).
- Spaced and tight spellings are the same literal (`3.2 AU` and `3.2AU`): the
  lexer is whitespace-insensitive, so both tokenize as Number then Ident.
- Suffix factors are grammar-level constants, not name lookups: a user
  `const au = 3` cannot change what `3.2 AU` means. Unit tokens are reserved
  in suffix position (after a number); an Ident followed by `(` is always a
  function call, never a suffix. Non-unit implicit multiplication stays an
  error (`2 pi` remains invalid).
- No RA hour/minute angle tokens: `12 h` meaning 180 degrees would collide
  with the time hour, and `hms2deg` already converts RA notation. The hour
  suffix is spelled `hr` because Planck's constant keeps the single letter
  `h`.
- **Functions return counts in natural units; suffixes convert counts to
  SI.** `mag2jy(20)` yields a Jy count; `mag2jy(20) Jy` converts it to SI.
  `hms2deg(6, 0, 0)` yields degrees; append ` deg` to enter radian world.

### Astronomy constants

`au`, `pc`, `ly`, `c`, `g`, `h`, `h_bar`, `k_b`, `sigma_sb`, `m_sun`, `r_sun`,
`l_sun`, `m_earth`, `r_earth`. Hybrid naming: bare short units, short
compound physical constants. Shadowable by users, exactly like `pi`.

### Time, angles, and optics

- `jd(y, m, d [, hr])` and `mjd(y, m, d [, hr])`: numeric calendar to Julian
  Date (no date strings; the language has none).
- `now()`: the current Julian Date from the host clock, the first
  non-deterministic builtin (accepted: clock calculators are the norm).
- `hms2deg(h, m, s)` / `dms2deg(d, m, s)` numeric in; `deg2hms(x)` /
  `deg2dms(x)` return the display-only `Str` (`12h 34m 56s`), the same
  mechanism as `hex`.
- `lst(jd, lon)` local sidereal time; `altaz(ra, dec, lat, lon, jd)` pair
  accessors `alt(...)`/`az(...)` style functions convert equatorial to
  horizontal; `airmass(alt)` the sec(z) estimate; `dawes(d_mm)` resolving
  power; `dist_mod(m_M)` distance modulus; `kepler(M, e)` solves Kepler's
  equation. Delta-T is applied inside ephemeris functions and exposed as
  `delta_t(jd)`.
- Exact function-name table is finalized in the guide pull request under the
  naming rules here: lowercase, unambiguous, no collisions with the existing
  55 builtins (`dec` is taken by the decimal conversion, hence `decl` for
  declination if the accessor form is used).

### Ephemeris: solar-ephemeris behind a core facade

- **Backend:** `solar-ephemeris`, exact version pinned, reached only through
  a facade module in `epher-core` (frontends never import it directly), so a
  vendor-or-replace decision never touches frontends. The featherweight tier
  is the documented fallback plan.
- **License allowlist for this and future dependencies:** MIT, Apache-2.0,
  BSD, MPL-2.0, ISC. SOFA-derived code is excluded; anything outside the
  allowlist goes to the maintainers first. Credit for solar-ephemeris's
  author is recorded in the guide's astronomy section and Cargo metadata.
- **Access shape:** single-value accessor functions with a documented body
  number (Mercury 1 through Neptune 8, Pluto 9, Sun 10, Moon 11), because the
  language has no strings or tuples: `ra(body, jd)`, `decl(body, jd)`,
  `dist(body, jd)`, `mag(body, jd)`, `phase(body, jd)`, `illum(body, jd)`,
  `diam(body, jd)`, `alt(body, jd, lat, lon)`, `az(body, jd, lat, lon)`,
  `rise(body, jd, lat, lon)`, `set(body, jd, lat, lon)`, plus moon-phase and
  equinox/solstice functions. Positions are geocentric unless an observer is
  given.
- **Scope:** Sun, Moon, eight planets, Pluto; RA/Dec, distance, alt/az,
  apparent magnitude, phase/illumination, angular diameter; rise/set/transit,
  moon phases, equinoxes and solstices. Eclipses and conjunction search are
  deferred.
- **3D view:** a `solar3d [t]` command renders the solar system as a graph
  pane object: orbit trails, positioned dots at each body's current position
  (ADR-0015 amendment), rotate/zoom inherited from the 3D pane. The optional
  time argument may reference a user constant, so the existing play button
  animates the solar system (`const t = now(); solar3d t`, press play).
- **Surface:** a seventh keypad tab, Astro (the ADR-0016 tab pattern), a
  guide section in all eight locales, and Examples presets (Kepler's
  equation solver, a blackbody curve, a transit light curve). The
  keypad-must-cover-every-function rule is honored by the new tab; existing
  tabs are not reworked.

## Consequences

- The parser gains exactly one new construct (Number + unit suffix) and one
  new command family (`solar3d`); the expression grammar, the `Value` model,
  and the store format are unchanged.
- Result display stays a bare canonical number; the guide teaches "suffixes
  convert to SI" as the one rule to remember.
- `now()` introduces clock dependence: re-running a script containing it can
  give different answers. Accepted; the pure path (explicit `jd(...)`) always
  exists.
- The wasm bundle grows by about 0.5 MB from the embedded ephemeris tables;
  the PWA service worker cache grows accordingly. Native binaries grow
  similarly. This is two orders of magnitude below the anise kernel path.
- Accuracy posture: arcsecond-class over the validated era (Delta-T table
  -500 to +2150; positions about plus/minus 5000 years with shrinking
  envelopes), documented honestly in the guide rather than oversold.
- The young-crate risk (first release weeks before adoption, one maintainer,
  0.x API) is carried by the pin + facade + vendoring plan, not by hope.
- Credits: solar-ephemeris's author is credited in the guide and Cargo
  metadata; the survey's license verdicts govern future astronomy
  dependencies.

## Amendment (2026-08-29): the TUI keypad carries a condensed astro bank

The Astro keypad tab in its full form (all 31 astronomy functions, the
14 constants, and the unit-suffix insert keys) exists on the web and
desktop keypads, which have no row budget. The TUI keypad pane is a
fixed eight rows under ADR-0033 (the bank row plus five key rows), the
same budget every other bank lives in, so the TUI ships a condensed
five-row astro bank: the 25 highest-value astronomy keys (the time,
accessor, and optics families plus `solar3d`), mirroring how the TUI
has always condensed the rest of the language while the web keypad
carries the complete set (ADR-0016/0019). The language itself is
identical everywhere per ADR-0011 and ADR-0007: suffixes and every
function type identically on all five frontends; only the key
coverage of the terminal's fixed grid is narrower. The unit-suffix
keys live on the web/desktop Astro tab only; on the TUI suffixes are
typed like any other token.

## Amendment (2026-08-30): every accessor answers for every body

A validation sweep against JPL Horizons (DE441) found the accessors of
this ADR not actually universal: Pluto (body 9) answered `dist`,
`mag`, `phase`, `illum`, `diam`, `alt`, and `az` through the facade's
own elements, but `ra`/`decl` and the rise/set/transit trio errored
out ("no entry for Pluto") because the sky snapshot stops at Neptune
and only some accessors had a fallback. All twelve accessors now
answer for Pluto: `ra`/`decl` run the facade's `pluto_radec` (the
same reduction `alt`/`az` always used), and `rise`/`set`/`transit`
mirror the crate's event search exactly - 10-minute sampling of
topocentric altitude across the observer's local mean-solar day,
bisection on the horizon crossing, and a parabolic culmination fit -
with the crate's horizon convention (geometric altitude −34′;
parallax is already in the topocentric place and Pluto's
semidiameter is negligible). A body without the event that day
still errors, never NaN. The same sweep found `phase(10, jd)` and
`illum(10, jd)` erroring for the Sun; the Sun's phase angle as seen
from Earth is zero by definition (Horizons reports phi 0.0000 and
100% illuminated), so those two now answer directly. Measured
against Horizons at three epochs, Pluto's new positions land within
about 42 arcsec - well inside this amendment's stated arcminute
grade, and the events keep the crate's conventions.
