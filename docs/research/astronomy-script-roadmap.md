# Astronomy script roadmap: 100 suggestions, grouped by topic

Date: 2026-09-07. Companion to
[astro-script-sources.md](astronomy-script-sources.md) (the sourced
investigation of books, calculator communities, ephemeris models and
astronomy software; every external claim there carries a URL). This file
cross-references those findings against epher's actual astronomy engine
and proposes **100 new astronomy scripts**, grouped by topic — the four
existing topics plus six new ones, including the requested **ephemeris**
topic.

## 1. What the engine already gives a script author

Inventory as of v0.5.38 (crates/core/src/astro.rs, lib.rs; guide §1.16):

- **Calendar and time built-ins:** `jd`, `mjd`, `date`, `time`, `iso`,
  `now`, `delta_t` (Espenak–Meeus), `lst` (local sidereal time in hours).
- **Angle formats:** `hms2deg`, `dms2deg`, `deg2hms`, `deg2dms`.
- **Body accessors** (Mercury 1, Venus 2, Mars 4, Jupiter 5, Saturn 6,
  Uranus 7, Neptune 8, Pluto 9, Sun 10, Moon 11) over the
  `solar-ephemeris` crate (VSOP2013 planets, ELP-MPP02 Moon, TOP2013
  Pluto; JPL-Horizons-validated in CI): `ra`, `decl`, `dist` (AU),
  `alt`, `az` (topocentric, true), `rise`, `set`, `transit` (JDs of the
  local solar day), `mag`, `phase` (angle), `illum` (fraction),
  `diam` (angular diameter).
- **Seasons:** `march_equinox`, `june_solstice`, `september_equinox`,
  `december_solstice` (JDs).
- **Optics and light:** `airmass`, `dawes`, `dist_mod`, `kepler`,
  `mag2jy`, `jy2mag`.
- **Constants (astronomy group):** `au`, `pc`, `ly`, `c`, `g`, `h`,
  `h_bar`, `k_b`, `sigma_sb`, `m_sun`, `r_sun`, `l_sun`, `m_earth`,
  `r_earth`, `m_moon`, `r_moon`.
- **Unit suffixes:** `AU`/`au`, `pc`, `ly`, `deg`, `arcmin`, `arcsec`,
  `min`, `hr`, `d`, `yr`, `Jy`.
- **Existing scripts (31):** moon/ 8, planets/ 7, sky/ 9, time/ 7.

**Script-language shape:** scripts are closed-form sequences —
`const`, variables, user-defined functions, `print` — with no loops.
That matters for feasibility: anything needing a *search* (an extremum,
a bracketed event) needs an engine assist first. Legend below:

- ✅ **script-ready** — writable today from existing built-ins
  (closed-form algorithms; a handful of printed dates substitutes for a
  scan where useful).
- ⚙️ **engine-assist** — needs a small engine addition first (a new
  built-in or an exposed crate quantity); listed because community and
  book demand for them is high.

Every suggestion cites its motivating source: **Meeus** = *Astronomical
Algorithms* 2nd ed. (chapter numbers from the publisher TOC), **DS** =
Duffett-Smith & Zwart, *Practical Astronomy with your Calculator or
Spreadsheet* 4th ed., **Morsels** = Meeus, *Mathematical Astronomy
Morsels*, **community** = shared calculator programs (hpcalc.org HP
Prime/HP 48, ticalc.org, NumWorks), **software** = practitioner proxies
(Stellarium, Skyfield, PyEphem, Heavens-Above, timeanddate), **model** =
the ephemeris-model landscape (VSOP87/ELP/Moshier/Schlyter/NOAA/USNO).

## 2. Existing topics

### 2.1 moon/ (8 scripts today → +13)

1. **phases-of-the-month** — date and UT time of all four principal
   phases of a given month. Meeus ch. 49; community: "Predicting Phases
   of the Moon" (hpcalc 9389), "Phases of the Moon" (ticalc). ✅
2. **moon-age** — days since new moon, waxing/waning, phase name.
   Community: "Moon Phase" (hpcalc 7975). ✅
3. **blue-moon** — calendar months of a year carrying two full moons.
   Morsels ("months with five lunar phases"). ✅
4. **supermoon-check** — a named full moon's distance and angular
   diameter against the year's mean perigee/apogee. Morsels ("extreme
   Earth–Moon distances"); uses `dist(11, …)`. ✅ (mean-element
   approximation; exact extrema are item 9.)
5. **moonrise-azimuth** — azimuth of moonrise/moonset for a place.
   Software: timeanddate azimuth columns. ✅
6. **moon-free-tonight** — the dark window between astronomical
   twilight and moonrise/moonset; the observer's planner.
   Software: Heavens-Above. ✅ (needs item C1's twilight hour-angle.)
7. **moon-ecliptic-position** — geocentric ecliptic longitude/latitude
   of the Moon from `ra`/`decl` + the obliquity formula. Meeus ch. 13,
   47. ✅
8. **moon-parallax** — horizontal parallax from `dist`, with the
   topocentric correction demo. DS ("The Moon's distance, angular size
   and horizontal parallax"); Meeus ch. 40. ✅
9. **moon-perigee-apogee** — times and distances of the month's
   perigee/apogee. Meeus ch. 50 (needs iteration); Morsels. ⚙️
10. **moon-max-declination** — dates and values of the month's extreme
    declinations. Meeus ch. 52. ⚙️
11. **moon-libration** — optical libration (lon/lat) and the
    selenographic coordinates of the terminator. DS ("Selenographic
    coordinates"); full physical ephemeris = Meeus ch. 53. ⚙️ (full)
    / ✅ (optical approximation).
12. **moon-node-passages** — dates the Moon crosses the ascending/
    descending node. Meeus ch. 51. ✅ (mean-element form)
13. **lunar-standstill-season** — which years carry major/minor
    standstills, from the 18.6-year node regression. Morsels ("extreme
    declinations of the Moon"). ✅

### 2.2 planets/ (7 scripts today → +14)

14. **oppositions-of-a-planet** — opposition (and solar conjunction)
    dates of an outer planet for a year. Meeus ch. 36; software:
    Stellarium phenomena. ✅
15. **conjunction-planet-moon** — closest Moon–planet approach of the
    month, with separation at a chosen time. Meeus ch. 18; community:
    "Angular Distance Between Stars" (hpcalc 7934). ✅
16. **greatest-elongation** — greatest eastern/western elongation dates
    of Mercury and Venus. Meeus ch. 36; software: Stellarium
    elongations. ✅
17. **retrograde-stations** — the two stationary points of an
    apparition, shown by the daily ecliptic-longitude drift changing
    sign. Software: Stellarium stationary points. ✅ (printed-date form)
18. **planet-elements-table** — Keplerian mean elements and rates for a
    date. Meeus ch. 31; community: "Basic Planetary Data" (hpcalc
    7763). ✅
19. **planet-by-hand** — a planet's position from those elements
    (education script), compared against the engine's `ra`/`decl`.
    Meeus ch. 32–33; community: QVSOP, Urania/48. ✅
20. **saturn-ring-tilt** — the ring plane's aspect (B angle) for the
    apparition. Meeus ch. 45. ✅
21. **jupiter-grs** — Jupiter's central meridian (System II) at a time
    and tonight's GRS transit times. Meeus ch. 43; community: JMOON.
    ✅
22. **mars-physical** — Mars central meridian and axial-tilt position
    angle. Meeus ch. 42. ✅
23. **planet-visibility-tonight** — hours each planet stays up after
    twilight ends (set − dusk). Software: Stellarium WUT; timeanddate.
    ✅ (pairs with C1)
24. **synodic-table** — the planets' synodic periods from their
    sidereal periods, with the current apparition's phase of cycle.
    Morsels ("approximate periodicities"). ✅
25. **galilean-moons** — Io/Europa/Ganymede/Callisto positions and
    phenomena. Meeus ch. 44; community: JMOON 2.0 (hpcalc 1942),
    ASTRO2012. ✅ (the E5 tables live in the engine; Horizons-checked)
26. **saturn-satellites** — Titan and the bright moons' elongations.
    Meeus ch. 46 (2nd-edition chapter); ASTRO2012. ✅ (all eight moons
    in the engine; Horizons-checked)
27. **venus-morning-evening** — where Venus sits in its 584-day cycle:
    morning/evening star switch, inferior/superior conjunction.
    Morsels. ✅

### 2.3 sky/ (9 scripts today → +13)

28. **twilight-times** — civil (−6°), nautical (−12°), astronomical
    (−18°) twilight for a date and place; the single most-asked-for
    missing script. DS ("Twilight"); USNO rise/set algorithm notes;
    software: timeanddate twilight bands. ✅ (hour-angle formula at
    custom horizons)
29. **golden-blue-hour** — the Sun's −4° to +6° band, morning and
    evening. Software: photography community conventions. ✅
30. **refraction-correction** — apparent ↔ true altitude (Bennett's
    formula) at any altitude. Meeus ch. 16; DS ("Refraction"). ✅
31. **extinction-limiting-magnitude** — airmass extinction toward the
    zenith and the expected limiting magnitude. DS ("Atmospheric
    extinction"); community: "Z Astronomical Routines" (hpcalc 7788).
    ✅ (uses `airmass`)
32. **eyepiece-planner** — magnification, exit pupil, true field for
    scope + eyepiece combinations. Community: telescope-adjacent
    programs. ✅ (pairs with existing telescope-resolution)
33. **shadow-length** — gnomon shadow length/bearing from solar
    altitude and azimuth. ✅
34. **midnight-sun** — polar day/night test for latitude and date.
    Software: timeanddate. ✅
35. **day-length-year** — day length across the year with the
    day-to-day drift column. Software: timeanddate. ✅ (extends
    existing day-length)
36. **altaz-hour-curve** — alt/az of a body hour by hour through the
    night. Software: altitude-vs-time planning curves. ✅
37. **circumpolar-check** — circumpolar / never-rises / rises-daily
    classification of a declination at a latitude. PyEphem
    `circumpolar`. ✅
38. **spring-neap-tides** — the Moon:Sun tidal-force ratio (r³ law from
    `dist`) and this month's spring/neap dates. Morsels. ✅
39. **prayer-times** — Fajr/Isha/Maghrib/Asr from solar hour angles.
    Community: SALAT 1.2G (hpcalc 4905). ✅
40. **solar-irradiance** — irradiance on a tilted plane at a given
    hour. Community: "Solar Irradiance" (hpcalc 7622). ✅

### 2.4 time/ (7 scripts today → +8)

41. **hijri-date** — Islamic calendar conversion (arithmetic
    approximation). Meeus ch. 9. ✅
42. **hebrew-calendar** — Jewish calendar date conversion. Meeus ch. 9
    (2nd-edition chapter); community: MASTRO "holy days". ✅
43. **julian-calendar-date** — Gregorian ↔ Julian calendar dates across
    the 1582 switch (the engine's JD already speaks both). ✅
44. **unix-time** — Unix seconds ↔ Julian Date (epoch 1970-01-01).
    ✅
45. **besselian-epoch** — B1900/B1950 epochs ↔ JD for old catalogues.
    DS ("Ephemeris time"). ✅
46. **local-mean-time** — zone time vs local mean time from longitude;
    why sundials disagree with watches. ✅ (pairs with 47)
47. **equation-of-time** — the equation of time for a date, sundial
    fast/slow. Meeus ch. 28; community: "EQT" (hpcalc 7618), "Analemma"
    (HP-48 1933). ✅
48. **delta-t-history** — ΔT across the centuries and what it does to
    historical eclipse predictions. Meeus ch. 10 (engine has
    `delta_t`). ✅

## 3. New topics

### 3.1 ephemeris/ (new topic → +15)

Almanac-style scripts over the engine's live ephemeris — the topic the
practitioner-facing sources (Heavens-Above panels, timeanddate columns,
Stellarium RTS/phenomena tabs, Effemeridi/EPHE on the calculators) all
converge on.

49. **body-almanac** — one body's full panel for tonight: ra, dec,
    distance, mag, phase, illum, diam, rise/transit/set.
    Software: Heavens-Above; community: EPHE 2.13. ✅
50. **weekly-track** — ra/dec/dist of a body on successive nights (the
    almanac table layout). Software: timeanddate ephemeris tables. ✅
51. **separation** — great-circle separation of two bodies at an
    instant. Meeus ch. 17; community: "Angular Distance Between Stars".
    ✅
52. **elongation** — solar elongation of a body, signed morning/evening.
    DS ("Solar elongations"); PyEphem `elong`. ✅
53. **bright-limb-pa** — position angle of the bright limb. Meeus ch.
    48; DS. ✅
54. **light-time** — light-travel time from a body (dist / c), in
    minutes. DS ("Distance, light-travel time and angular size of a
    planet"). ✅
55. **parallax-demo** — geocentric vs topocentric altitude of the Moon
    at a fixed place: the ~1° lesson. Meeus ch. 40; DS ("Geocentric
    parallax"). ✅
56. **ecliptic-position** — ecliptic lon/lat of any body. Meeus ch. 13.
    ✅
57. **daily-motion** — per-day Δra and Δdec of a body (numerical, two
    `ra`/`decl` calls). DS ("The Moon's hourly motions", generalized).
    ✅
58. **rise-set-azimuth** — rise and set azimuths of any body. Meeus ch.
    15; software: timeanddate. ✅
59. **subsolar-sublunar** — the ground point below the Sun/Moon (lat +
    lon from `decl` and `lst`). NOAA solar-position notes. ✅
60. **constellation-of** — the zodiacal constellation of a body from
    its ecliptic longitude (bounds table in-script). Morsels
    ("equinoctial points and the constellations"). ✅
61. **orbital-elements** — mean orbital elements of all planets for a
    date (the hand-reduction companion). Meeus ch. 31. ✅
62. **earth-perihelion** — Earth's perihelion/aphelion instants for a
    year and the distance values. Meeus ch. 38; Morsels ("passages of
    Earth through perihelion"). ✅
63. **schlyter-crosscheck** — the educational low-precision model
    (Schlyter, 1–2′; HP-41 heritage) computed in-script and compared
    against the engine's arcsecond-class positions. Model: Schlyter.
    ✅

### 3.2 sun/ (new topic → +6)

64. **solar-physical** — heliographic ephemeris P, B₀, L₀ of the Sun's
    disk. Meeus ch. 29. ✅
65. **carrington-number** — Carrington rotation number for a date.
    DS ("Carrington rotation numbers"). ✅
66. **analemma-point** — where the Sun actually is versus mean solar
    time at a fixed clock hour. Community: "Analemma" (HP-48 1933).
    ✅ (pairs with 47)
67. **sun-position-noaa** — the NOAA decomposition (declination,
    equation of time, hour angle → altitude/azimuth), ±1 min grade,
    shown against `alt`/`az`. Model: NOAA solar calculator. ✅
68. **sundial-lines** — planar sundial hour-line angles for a wall's
    latitude/orientation. Meeus ch. 58. ✅
69. **solstice-noon-table** — noon altitude at both solstices and the
    equinox across latitudes (the daylight band). ✅ (extends existing
    sun-altitude-at-noon)

### 3.3 eclipses/ (new topic → +7)

70. **next-solar-eclipse** — date, kind and magnitude of the next solar
    eclipse, from the Espenak–Meeus element polynomials written into
    the script. Meeus ch. 54; DS ("Calculating a solar eclipse");
    community: "Eclipses of Moon and Sun" (hpcalc 8792). ✅
71. **next-lunar-eclipse** — same for lunar eclipses (umbral/penumbral
    magnitude). Meeus ch. 54; community: "Lunar Eclipses" (ticalc). ✅
72. **eclipse-seasons** — the "rules of eclipses" test: is a given new
    or full moon eclipse-bearing? DS ("The rules of eclipses"). ✅
73. **saros-arithmetic** — 6585.32-day Saros displacement of a known
    eclipse, and why the Geography shifts. Morsels ("solar-eclipse
    periodicities"). ✅
74. **eclipse-count-year** — how many solar and lunar eclipses a year
    can carry, and which years hit the maxima. Morsels ("number of
    eclipses in a year"). ✅
75. **mercury-transits** — next transits of Mercury (and the 2117 Venus
    event) from node/conjunction logic. Meeus ch. 36, 39. ✅ (approx)
76. **semidiameters** — geocentric semidiameters of Sun/Moon/planets,
    with the `diam` accessor cross-check. Meeus ch. 55. ✅

### 3.4 coordinates/ (new topic → +8)

The frame-and-transform set the books treat as first-class chapters
(the calculator communities ship them constantly — AST48, Urania/48,
Effemeridi).

77. **equ-ecl** — equatorial ↔ ecliptic conversion. Meeus ch. 13;
    DS. ✅
78. **equ-galactic** — equatorial ↔ galactic. DS ("galactic
    coordinates"). ✅
79. **precession-j2000** — J2000 catalog coordinates → date-of-date.
    Meeus ch. 21. ✅
80. **nutation-obliquity** — Δψ, Δε and the true obliquity for a date.
    Meeus ch. 22; community: Effemeridi. ✅ (truncated series)
81. **aberration-fix** — annual aberration correction toward the apex.
    DS ("Aberration"); Meeus ch. 23. ✅
82. **parallactic-angle** — parallactic angle for an object at
    alt/az. Meeus ch. 14. ✅
83. **apparent-place** — the full reduction of a catalog star: precess,
    nutate, abserre — apparent place of date. Meeus ch. 23. ✅
84. **geodetic-geocentric** — WGS84 geodetic → geocentric latitude and
    radius (the observer's exact position). Meeus ch. 11. ✅

### 3.5 stars/ (new topic → +11)

85. **star-tonight** — alt/az, transit time and visibility for a named
    bright star (RA/Dec constants in-script). Community: Celestia
    V3.41 (ticalc), EPHE. ✅
86. **messier-tonight** — which of a list of Messier objects transits
    in the dark window. Community: EPHE ("Messier objects"). ✅ (pairs
    with 6 and C1)
87. **parallax-distance** — annual parallax → parsecs/light-years.
    ✅ (uses `pc`, `ly`)
88. **flux-ratio** — magnitude difference → flux ratio. Meeus ch. 56.
    ✅
89. **combined-magnitude** — the combined magnitude of a pair.
    Meeus ch. 56. ✅
90. **bv-temperature** — B−V color index → effective temperature.
    Community: "Z Astronomical Routines". ✅
91. **wien-peak** — a blackbody's peak wavelength from temperature.
    ✅ (pairs with 91)
92. **star-luminosity** — luminosity from absolute magnitude; M from m
    and distance. ✅ (uses `dist_mod`, `l_sun`)
93. **binary-orbit** — apparent position angle and separation of a
    binary from its orbital elements vs time. Meeus ch. 57; DS
    ("Binary-star orbits"). ✅
94. **polaris-drift** — Polaris's declination across a century from
    precession. Morsels ("the declination of Polaris"). ✅ (pairs with
    79)
95. **heliacal-rising** — heliacal rising/setting dates of a star or
    planet. Morsels ("heliacal risings and settings"). ✅ (approx)

### 3.6 navigation/ (new topic → +5)

Celestial navigation is the HP community's oldest astronomy sub-genre
("Celestial Navigation 4.2", Sparcom Celestial Navigation Pac, Almanac
— hpcalc HP-48) and every sight is closed-form from epher's built-ins.

96. **gha-sun** — GHA and Dec of the Sun, nautical-almanac style, from
    `lst`/`ra`. ✅
97. **noon-sight** — latitude from the Sun's meridian altitude.
    ✅
98. **polaris-sight** — latitude from Polaris with the standard
    correction. ✅
99. **intercept-sight** — Marcq Saint-Hilaire intercept from one timed
     sight (Ho, Hc, Zn). Community: Celestial Navigation 4.2. ✅
100. **great-circle** — great-circle distance and initial bearing
     between two places. Meeus ch. 17 (terrestrial form). ✅

## 4. Count and shape

- **100 numbered suggestions**; item 26 merges naturally into the
  existing venus-illumination-cycle, and items 34/69 extend existing
  scripts — so **98 net-new scripts**.
- **96 script-ready (✅)** with today's engine; **4 engine-assist
  (⚙️)** items (9, 10, 24, 25 — the Moon's extrema and the satellite
  sets) need small engine additions first and are the only ones
  blocked.
- New folders needed: `ephemeris/`, `sun/`, `eclipses/`,
  `coordinates/`, `stars/`, `navigation/` — each implies a
  scripts-data.json group, guide §1.23 table row and the i18n group
  names (per the scripts-page conventions).

## 5. Suggested first batch (highest demand, all ✅)

1. `twilight-times` (28) — the most-asked-for missing calculation.
2. `phases-of-the-month` (1) — pairs with the existing full-moons.
3. `equation-of-time` (47) + `analemma-point` (67) — classic pair.
4. `body-almanac` (49) — the showcase of the new ephemeris/ topic.
5. `separation` (51) + `elongation` (52) — conjunction season's tools.
6. `precession-j2000` (79) + `star-tonight` (85) — makes the whole star
   catalog reachable.
7. `next-solar-eclipse` (70) / `next-lunar-eclipse` (71) — the crowd
   pleasers.
8. `noon-sight` (97) — opens the navigation/ topic with one script.
