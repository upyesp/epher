# Astronomy script sources for epher

**Goal:** decide which NEW astronomy scripts ("epher scripts") are worth adding, by
surveying four bodies of evidence: (1) the leading practical-astronomy calculation
books and their real tables of contents, (2) the astronomy programs people actually
share in calculator communities, (3) the ephemeris model landscape (accuracy/range
trade-offs), and (4) the quantities astronomy software computes daily as a proxy for
user demand. Ends in a flat, deduplicated list of candidate script topics.

**Method (all claims sourced, not assumed):** every claim below carries a URL checked
on **2026-09-07**. Several publisher/community sites block scripted access or are
defunct, so verified Wayback Machine snapshots are cited where needed (snapshot URL =
the exact capture consulted). Two live-access notes: `willmannbell.com` and
`uscibooks.com` currently serve other companies' pages (the WB store was in
maintenance; University Science Books was acquired by MIT Press effective 2025-07-01,
per https://www.uscibooks.com/urban.htm as served on 2026-09-07), and
`hpcalc.org`/`hpmuseum.org` returned anti-bot responses to direct fetches — all
resolved via the Internet Archive.

---

## 1. Leading books on practical astronomy calculations

### 1.1 Jean Meeus, *Astronomical Algorithms* (2nd ed.)

- **Edition facts (publisher page):** 6"×9", 477 pages, hardbound, 2nd edition
  published 1999. The 2nd edition added chapters on the **Jewish and Moslem
  calendars** and the **satellites of Saturn**, plus **Appendix IV**: polynomial
  coefficients for the heliocentric coordinates of Jupiter–Neptune, 1998–2025.
  Source: Willmann-Bell product page,
  http://web.archive.org/web/20200105175035/https://www.willbell.com/math/mc1.HTM
- **Correction to our planning note:** the brief said "47 chapters". The 2nd
  edition's publisher table of contents lists **58 numbered chapters** plus 4
  appendices (same URL). For reference, the **1st edition (1991) has 56 chapters**
  (ends at ch. 56 "Calculation of a Planar Sundial"); scanned copy with readable
  TOC: https://archive.org/details/astronomicalalgorithmsjeanmeeus1991
- **Full TOC of the 2nd edition** (same Willmann-Bell page) — every chapter is a
  self-contained, calculator-sized algorithm; this is the single richest source of
  candidate scripts:

  1 Hints and Tips · 2 About Accuracy · 3 Interpolation · 4 Curve Fitting ·
  5 Iteration · 6 Sorting Numbers · 7 Julian Day · 8 Date of Easter ·
  9 Jewish and Moslem Calendars · 10 Dynamical Time and Universal Time ·
  11 The Earth's Globe · 12 Sidereal Time at Greenwich · 13 Transformation of
  Coordinates · 14 The Parallactic Angle · 15 Rising, Transit and Setting ·
  16 Atmospheric Refraction · 17 Angular Separation · 18 Planetary Conjunctions ·
  19 Bodies in a Straight Line · 20 Smallest Circle Containing Three Celestial
  Bodies · 21 Precession · 22 Nutation and the Obliquity of the Ecliptic ·
  23 Apparent Place of a Star · 24 Reduction of Ecliptical Elements from One
  Equinox to Another · 25 Solar Coordinates · 26 Rectangular Coordinates of the
  Sun · 27 Equinoxes and Solstices · 28 Equation of Time · 29 Ephemeris for
  Physical Observations of the Sun · 30 Equation of Kepler · 31 Elements of the
  Planetary Orbits · 32 Positions of the Planets · 33 Elliptic Motion ·
  34 Parabolic Motion · 35 Near-Parabolic Motion · 36 The Calculation of some
  Planetary Phenomena · 37 Pluto · 38 Planets in Perihelion and Aphelion ·
  39 Passages through the Nodes · 40 Correction for Parallax · 41 Illuminated
  Fraction of the Disk and Magnitude of a Planet · 42 Ephemeris for Physical
  Observations of Mars · 43 Ephemeris for Physical Observations of Jupiter ·
  44 Positions of the Satellites of Jupiter · 45 The Ring of Saturn ·
  46 Positions of the Satellites of Saturn · 47 Position of the Moon ·
  48 Illuminated Fraction of the Moon's Disk · 49 Phases of the Moon ·
  50 Perigee and Apogee of the Moon · 51 Passages of the Moon through the Nodes ·
  52 Maximum Declinations of the Moon · 53 Ephemeris for Physical Observations of
  the Moon · 54 Eclipses · 55 Semidiameters of the Sun, Moon and Planets ·
  56 Stellar Magnitudes · 57 Binary Stars · 58 Calculation of a Planar Sundial ·
  Appendix I Constants · Appendix II Some Astronomical Terms · Appendix III
  Planets: Periodic Terms · Appendix IV Coefficients for the Heliocentric
  Coordinates of Jupiter to Neptune, 1998–2025.

  USNO independently confirms the book's scope and utility for practitioners:
  "a general source of algorithms for performing a wide variety of celestial
  calculations … Chapter 15 covers the computation of times of rise, set, and
  transit. This book also contains algorithms for low-precision ephemerides of
  major celestial bodies." — https://aa.usno.navy.mil/faq/rs_algor

### 1.2 Peter Duffett-Smith & Jonathan Zwart, *Practical Astronomy with your Calculator or Spreadsheet* (4th ed.)

- **Edition facts:** 4th edition, Cambridge University Press, 2011
  (Open Library record, ISBN 9780521146548:
  https://openlibrary.org/books/OL25023016M).
- **Real TOC** (Google Books record
  https://books.google.com/books/about/Practical_Astronomy_with_your_Calculator.html?id=MTGYxQyW998C)
  — organized as five chapter-topics, each with many numbered section-level
  calculations. Section-level items (one line each) are directly script-sized:

  *Time:* Calendars · The date of Easter · Converting the date to the day number ·
  Julian dates · Converting the Julian date to the Greenwich calendar date ·
  Finding the name of the day of the week · Decimal hours ↔ h/m/s · Local time ↔ UT ·
  Sidereal time (ST) · UT → Greenwich sidereal time (GST) · GST → UT · Local
  sidereal time (LST) · LST → GST · Ephemeris time (ET) and terrestrial time (TT).
  *Coordinate systems:* Horizon, equatorial, ecliptic, galactic coordinates ·
  Decimal degrees ↔ DMS · Degrees ↔ hours · Conversions between all of the above ·
  Generalised coordinate transformations · The angle between two celestial objects ·
  Rising and setting · Precession · Nutation · Aberration · Refraction ·
  Geocentric parallax and the figure of the Earth · Corrections for parallax ·
  Heliographic coordinates · Carrington rotation numbers · Selenographic
  coordinates · Atmospheric extinction.
  *The Sun:* Orbits · The apparent orbit of the Sun · Calculating orbits more
  precisely · The Sun's distance and angular size · Sunrise and sunset · Twilight ·
  The equation of time · Solar elongations.
  *The planets, comets and binary stars:* The planetary orbits · Calculating the
  coordinates of a planet · Approximate positions of the planets · Perturbations in
  a planet's orbit · Distance, light-travel time and angular size of a planet ·
  The phases of the planets · Position angle of the bright limb · Apparent
  brightness of a planet · Comets · Parabolic orbits · Binary-star orbits.
  *The Moon and eclipses:* The Moon's orbit · Calculating the Moon's position ·
  The Moon's hourly motions · The phases of the Moon · Position angle of the Moon's
  bright limb · The Moon's distance, angular size and horizontal parallax ·
  Moonrise and moonset · Eclipses · The rules of eclipses · Calculating a lunar
  eclipse · Calculating a solar eclipse · The astronomical calendar.

### 1.3 Oliver Montenbruck & Thomas Pfleger, *Astronomy on the Personal Computer* (4th ed., Springer 2000)

- **Real TOC (12 chapters + appendices)** from the Springer product page (ISBNs
  978-3-540-67221-0 hardcover / 978-3-642-03436-7 eBook, © 2000):
  http://web.archive.org/web/20190329083614/https://www.springer.com/gp/book/9783540672210
  1. Introduction · 2. Coordinate Systems · 3. Calculation of Rising and Setting
  Times · 4. Cometary Orbits · 5. Special Perturbations · 6. Planetary Orbits ·
  7. Physical Ephemerides of the Planets · 8. The Orbit of the Moon · 9. Solar
  Eclipses · 10. Stellar Occultations · 11. Orbit Determination · 12. Astrometry ·
  Appendices.
- The publisher's description names its concrete outputs: "determining and
  predicting the positions of the Sun, Moon, planets, minor planets and comets,
  solar eclipses, stellar occultations by the Moon, phases of the Moon and much
  more" (same URL). Distinctive topics vs. the other books (per the same TOC):
  special perturbations (numerical integration), orbit determination from
  observations, and stellar occultations by the Moon.

### 1.4 Jean Meeus, *Mathematical Astronomy Morsels* (Willmann-Bell, 1997) and sequels

- **Edition facts:** 6"×9", 400 pages, hardbound, $24.95 (publisher page,
  http://web.archive.org/web/20200513223125/https://www.willbell.com/math/mc16.htm).
- **Sequels:** *More Mathematical Astronomy Morsels* (2002), *Morsels III*, *IV*,
  *V* are all listed on the Willmann-Bell computational-astronomy index page:
  http://web.archive.org/web/20200513223125/https://www.willbell.com/math/index.htm
- **Real TOC of Morsels I** (same mc16.htm URL) — 62 essays grouped in sections;
  unlike the two books above it is a collection of *investigations*, but each
  essay suggests a checkable calculation: **The Moon:** the instantaneous lunar
  orbit · extreme Earth–Moon distances · distribution of perigee/apogee distances ·
  mean Earth–Moon distance · extreme declinations of the Moon · librations of the
  Moon · months with five lunar phases. **Eclipses and occultations:** number of
  eclipses in a year · solar-eclipse periodicities · regions of visibility ·
  frequency of total/annular eclipses for a given place · total penumbral lunar
  eclipses · the half-Saros · series of occultations · occultations of bright
  stars/planets by the Moon. **Planetary motions:** the solar-system barycenter ·
  passages of Earth through perihelion · periheloids/apheloids · planetary
  quadrants and sectors · how often are the planets aligned · mean-motion
  "resonances" of the planets · asteroid couples · comet Encke and Jupiter ·
  orbital inclinations of the Galilean satellites. **Planetary phenomena:**
  approximate periodicities · opposition loops · opposition places · triple
  conjunctions · planetary groupings · periodicities in the phenomena of Jupiter's
  satellites · Jupiter without satellites · triple shadow transits.
  **On the celestial sphere:** heliacal risings and settings · positions of
  Uranus/Neptune/Pluto at discovery · ecliptic and galactic equator · equinoctial
  points and the constellations · the declination of Polaris. **Statistics:**
  sunspots and weather · solar activity and lunar-eclipse brightness. **Varia:**
  the equation of time · equinoxes and solstices · the weekday of Christmas ·
  the distribution of Easter Sundays · rounding numbers · predicting sunspot
  activity.

### 1.5 *Explanatory Supplement to the Astronomical Almanac*, 3rd ed. (Urban & Seidelmann eds.)

- **Edition facts:** ISBN 978-1-891389-85-6, 734 pages, copyright 2013, University
  Science Books (publisher page:
  http://web.archive.org/web/20190107004944/http://www.uscibooks.com:80/urban.htm).
- **Real TOC (15 chapters + appendices)** (publisher contents page:
  http://web.archive.org/web/20190419061000/http://www.uscibooks.com:80/urbancon.htm):
  1. Introduction to Positional Astronomy · 2. Relativity for Astrometry, Celestial
  Mechanics and Metrology · 3. Time · 4. The Fundamental Celestial Reference
  System · 5. Terrestrial Coordinates and the Rotation of the Earth · 6. Precession,
  Nutation, and Earth Rotation · 7. Positions · 8. Orbital Ephemerides of the Sun,
  Moon, and Planets · 9. Planetary Satellites and Rings · 10. Physical Ephemerides ·
  11. Eclipses of the Sun and Moon · 12. Astronomical Phenomena · 13. Stars and
  Stellar Systems · 14. Computational Techniques · 15. Calendars · Appendices ·
  Glossary · Index.
- **Practice anchor:** USNO's own guidance says the Explanatory Supplement is "the
  authoritative source describing the methods used to compute astronomical
  phenomena in The Astronomical Almanac. Chapter 12 presents algorithms for
  computing times of rise, set, and transit."
  (https://aa.usno.navy.mil/faq/rs_algor)

**Books → takeaway:** Meeus's 58 chapters and Duffett-Smith's ~65 section-level
calculations are, almost verbatim, a backlog of calculator-sized astronomy scripts.
The Explanatory Supplement marks the same topics at professional rigor (calendars,
time scales, precession/nutation, rise/set/transit, eclipses, physical ephemerides).

---

## 2. Calculator script communities: what people actually share

### 2.1 hpcalc.org — HP Prime (science category, Wayback capture 2022-12-09; live: https://www.hpcalc.org/prime/science/)

Astronomy entries with names + what each computes (detail URLs are
`https://www.hpcalc.org/details/<id>`):

- **Astro Lab 4** (7593) — "Astronomy program with many built-in features".
- **Effemeridi** (7617) — "ephemeris for Sun, Moon and planets, transit, sidereal
  time, nutation and obliquity, Julian date, ascendant and many others data".
- **Eclipses of Moon and Sun 1.0** (8792) — for a given date, finds next full/new
  moon and checks for lunar/solar eclipse incl. kind, magnitude and shadow size.
- **Moon Date** (8984) — next start date of a chosen lunar phase for a month/year.
- **Moon Phase** (7975) / **Moon Phase 1.0** (7508) — phase percent/graphic for a date.
- **Predicting Phases of the Moon** (9389) — date and UTC time of each lunar phase
  in a month.
- **Planetary Positions** (8834) — "positions of the planets, plus one dwarf planet".
- **EQT** (7618) — equation of time approximation.
- **Angular Distance Between Stars** (7934) — great-circle separation from two
  RA/Dec pairs (J2000).
- **Basic Planetary Data** (7763) — planet physical/orbital data lookup.
- **Distance from the Sun & Orbital Speed** (7632) — heliocentric distance and
  speed days from perihelion (Kepler ellipse).
- **Solar Irradiance** (7622) — incidence angle/irradiance on a tilted panel.
- **Z Astronomical Routines** (7788) — planet temperature estimate, spacecraft
  drag, magnitudes.
- **Programs for Astronomy and Orbital Mechanics** (9388) — routines + source.
- **Real Time Ephemeris** (9113) — real-time ephemeris (Indonesian).
- **Lunar Astronomy 1.0** (9046) — Moon/Earth orbit illustration.
- **MSSTARS 1.0** (9167) — main-sequence star model from mass.
- **Observatories** (7787) — parses Minor Planet Center observatory file, maps sites.

### 2.2 hpcalc.org — HP 48 astronomy category (55 entries; Wayback capture 2015-12-30)

Full listing at
http://web.archive.org/web/20151230225549/http://www.hpcalc.org:80/hp48/science/astronomy/
(detail URLs `https://www.hpcalc.org/details/<id>`). Highlights:

- **AST48 1.01** (5279) — astronomical library: ephemeris of Sun, Moon, planets,
  comets, asteroids; mean/apparent/geocentric/topocentric coordinates; accurate
  precession and nutation; mean elements of major bodies.
- **ASTRO2012 1.0** (7365) — Sun/Moon/9 planets+Xena via **VSOP87B + TOP2010A +
  corrections from DE422**; date of Easter; positions of the main satellites of
  Jupiter, Saturn, Uranus; comet orbits (parabolic/hyperbolic/elliptic).
- **EPHE 2.13** (1941) — ephemeris of sun, planets, moon, stars, Messier objects,
  comets, asteroids; distance, magnitude, apparent diameter, phase and
  rising/transit/setting.
- **JPLEPH 1.0** (7370) — JPL DE421/422/423/424/406/408 barycentric/heliocentric/
  geocentric coordinates of Sun and planets.
- **JMOON 2.0** (1942) — positions of Jupiter's moons (Io, Europa, Ganymede,
  Callisto), past/present/future.
- **Urania/48 2.00.01** (1965) — "an almost complete and expandable implementation
  of the book *Astronomical Algorithms* written by … Jean Meeus."
- **QVSOP 1.01** (1954) — faster planet positions for **1998–2025** (i.e., the
  Meeus 2nd-edition Appendix-IV polynomials' window).
- **MASTRO** (1944) — position, rise, culmination, set for sun, moon, planets;
  moon phases; calendar, holy days, Julian day; all coordinate and time conversions.
- **Celestial Navigation 4.2** (1937) — position fix from observed altitudes of
  multiple bodies by least squares, with dip/refraction corrections; **Sparcom
  Celestial Navigation Pac** (7082) commercial pack; **Almanac** (6678) celestial
  navigation fix program; **Celestial and DR Navigation** (6714).
- **SunCalc** (1961) / **Suncalc 1.0** (4192) / **Sunrise/Sunset** (1962) —
  sunrise/sunset times from position, timezone, date.
- **Analemma** (1933) — Sun's position vs "where it is supposed to be" (declination
  and right-ascension offset).
- **Eclipse 1.1** (1940) — calculates and animates solar and lunar eclipses.
- **Moon 2.0/3.0**, **Moon Phase**, **Phase Of Moon 1.0** — lunar phase date/time
  (Moon 3.0 valid "from 1582-10-15 to the future").
- **HPlanétarium 3.08** (4897) — planetarium with equatorial/azimuthal coordinates,
  rise/transit/set times, elongation, magnitude.
- **Digital Setting Circles 2.0** (1939) — telescope pointing without polar
  alignment; **Starplotter**, **StarMaHP**, **Sky**, **Ciel 1.5** — star maps and
  observation helpers.
- **Tyko 3.1** (1964) / **Plasy** (1953) — solar-system object data sheets.
- **Orbit Determination 1.0** (7577) — Gauss-Herrick-Gibbs / Herget orbit
  determination from observations.
- **XTime 1.1** (6044) — extended time/date/calendar/astronomical routines,
  partially ported from HP-41C CALENDARS solutions.
- **SALAT 1.2G** (4905) — Muslim prayer times (hour-angle-based solar computation).

### 2.3 ticalc.org — TI-83/84 Plus BASIC astronomy archive (live)

Category: https://www.ticalc.org/pub/83plus/basic/science/astronomy/ — five
programs, each with description:

- **Celestia V3.41** (`celestia.zip`) — "Converts celestial co-ords for Messier,
  NGC, Named Stars, and planets to horizon co-ords. Planet positions are
  dynamically calculated."
  https://www.ticalc.org/archives/files/fileinfo/446/44686.html
- **Julian Date Converter** (`juliandateconv.zip`) — date ↔ Julian Date.
  https://www.ticalc.org/archives/files/fileinfo/475/47521.html
- **Lunar Eclipses** (`moonecl.zip`) — "for any given year and month, the times of
  the possible lunar eclipse."
  https://www.ticalc.org/archives/files/fileinfo/276/27615.html
- **Phases of the Moon** (`moonph.zip`) — "for any given year and month, the
  instant of the selected Moon phase."
  https://www.ticalc.org/archives/files/fileinfo/269/26939.html
- **Sunθ84+ v2.5** (`suntheta84.zip`) — "calculates solar position given UTC time
  and location."
  https://www.ticalc.org/archives/files/fileinfo/450/45056.html

(The TI-84 Plus CE Python archive has no astronomy entries:
https://www.ticalc.org/pub/84plusce/python/ — checked 2026-09-07.)

### 2.4 HP Museum (hpmuseum.org) — HP-41 software library

The live site blocks scripted access (403 "Just a moment…" on 2026-09-07); Wayback
captures used. Jean-Marc Baillard's astronomy programs:

- **Astronomical Ephemeris for the HP-41**
  (http://web.archive.org/web/20160116094313/http://www.hpmuseum.org:80/software/41/41asteph.htm)
  — solves Kepler's equation, adds periodic corrections; ephemeris over **1000–3000**
  with ~**0.01°** heliocentric-longitude accuracy (Pluto 1880–2110 only); Venus/Mars
  geocentric longitude error up to ~1 arcmin near closest approach.
- **Astronomical Refraction for the HP-41**
  (http://web.archive.org/web/20160405104923/http://www.hpmuseum.org/software/41/41astror.htm)
  — apparent↔true altitude refraction, short routine with errors < 0.34″ over
  0°–90°, multi-parameter program reproducing Pulkovo refraction tables to ~1–2″.
  Also in the library: Easter date (`41easter.htm`), lunar-landing games, tides
  (`41tides.htm`), new-moon and planet series (`41td/newmoon.htm`, `41td/planet.htm`),
  and an "astro1–3" barcode set (`41td2/astro1.htm`).

### 2.5 NumWorks workshop / "My NumWorks" library

The old `workshop.numworks.com` gallery has been folded into
`https://my.numworks.com/python` (a JS-filtered public library; its English
"Examples" category currently holds only a dozen sample scripts, none astronomy —
checked 2026-09-07). Astronomy scripts are still directly indexable; a concrete
cluster by user **steveg1cmz** (https://my.numworks.com/python/steveg1cmz):

- **aeclipse.py** — "Astronomy: Eclipse … Algorithms are based on Jean Meeus.
  Calculates eclipses (lunar eclipse and solar eclipse)" with `DTm(JDE)` and
  `eclipse(year,koff)` (koff = 0 solar, ±0.5 lunar).
  https://my.numworks.com/python/steveg1cmz/aeclipse
- **alunareclipse.py** — lunar eclipse + JDE/date conversion routines (Meeus-based)
  plus a one-year calendar. https://my.numworks.com/python/steveg1cmz/alunareclipse
- **asolardistancelo.py** — "solar distance (centre Sun–centre Earth) given a JDE.
  Based on low-precision Jean Meeus algorithm (precision maybe 1000's of km)".
  https://my.numworks.com/python/steveg1cmz/asolardistancelo
- **al0_1.py** — lunar orbital characteristics (simplified circular) with Turtle
  animation. https://my.numworks.com/python/steveg1cmz/al0_1

(Casio: planet-casio.com program search for "astronomie" surfaced no astronomy
programs in the listing fetched on 2026-09-07 — no claim made.)

### 2.6 TI-Basic Developer (tibasicdev.wikidot.com)

The site's own Program Archives page states it is no longer maintained: "its forum,
archives, and even hosting service … have been decaying for years … head over to
Cemetech" — http://tibasicdev.wikidot.com/archives (checked 2026-09-07). Its site
search returns no astronomy program pages. Conclusion: treat tibasicdev as
documentation, not a program source; the TI community now lives on
ticalc.org/Cemetech.

**Community → takeaway:** across communities the same handful of computations recur
(moon phase & next-phase dates, eclipses, sun position & sunrise/sunset, planet
positions, JD conversion, ephemeris bundles, celestial navigation, moon libraries of
Jupiter/Saturn), and the respected "big" programs are explicitly Meeus implementations
(Urania/48) or VSOP87-based (ASTRO2012) — evidence that the book-derived topics
already carry user demand.

---

## 3. Ephemeris model landscape (accuracy, date range, practice use)

| Model | What it is | Accuracy | Valid range (as stated by source) | Typical use | Feasibility as short calculator script |
|---|---|---|---|---|---|
| **VSOP87** (Bretagnon & Francou, Bureau des Longitudes; CDS catalog VI/81) | Semi-analytic planetary series; fitted to JPL **DE200**; distributed as versions: VSOP87 (elliptic elements), **A** (heliocentric rect., J2000), **B** (heliocentric spherical, J2000), **C** (rect., equinox/ecliptic of date), **D** (spherical, of date), **E** (barycentric rect., J2000) | **1″ for Mercury, Venus, Earth–Moon barycenter, Mars over ±4000 yr** around J2000; same precision for Jupiter/Saturn over ±2000 yr and Uranus/Neptune over ±6000 yr (notice §PRECISION, with per-body relative precision table p₀) | As above (thousands of years) | The standard compact planetary theory used by almanac-grade amateur software (e.g., hpcalc ASTRO2012 uses VSOP87B; Meeus chs. 32/47 follow this school) | **Feasible as a script only in truncated form**: the series are large — per-body files run from ~1,700 to ~15,000 records depending on version (ReadMe File Summary: e.g. VSOP87D.nep = 1,946 records, VSOP87D.mar = 5,501, VSOP87.sat = 12,375); keeping only the largest terms is the classic "compact VSOP87" approach. The D-version (spherical, of-date) avoids frame math |
| **ELP2000-82 / ELP 2000-85** (Chapront-Touzé & Chapront; CDS catalog VI/79) | Semi-analytic **lunar** theory: trigonometric + Poisson series for longitude/latitude (arcsec) and distance (km); constants fitted to **DE200/LE200**; files cover main problem plus Earth-figure, planetary, tidal, Moon-figure perturbations | CDS: "All this set allows to compute a high precision lunar ephemeris" (VI/79 ReadMe); the 1988 paper is titled "ELP 2000-85: a semi-analytical lunar ephemeris **adequate for historical times**" | Historical times (paper title); series include t and t² terms | High-precision lunar ephemerides; basis of Meeus ch. 47's abridged theory | **Full form: no; abridged form: yes** (Meeus ch. 47 is precisely the abridged ELP; Moshier's aa uses a modified Chapront theory, see below) |
| **Moshier ephemeris (DE404 fits)** (Stephen Moshier's `plan404` series + modified Chapront lunar theory, in `aa-56.zip`) | Trigonometric expansions for Earth and planets "adjusted to match JPL's DE404 Long Ephemeris (1995)"; Moon via "a modified version of the lunar theory of Chapront-Touzé and Chapront" | Planets: "precision ranging from about **0.1″ for the Earth to 1″ for Pluto**"; Moon: "**0.5 arc second relative to DE404** for all dates between 1369 B.C. and 3000 A.D."; librations series (`selenog.zip`) 0.05″ from −1369 to +2950 | 3000 B.C.–3000 A.D. for outer planets; inner planets strictly valid 1350 B.C.–3000 A.D. (may be used to 3000 B.C. with loss) | Self-contained almanac-grade ephemeris **with no tabulated data files** — the classic choice for embeddable calculators (the Debian `aa` package) | **Very feasible**: it exists precisely to be a closed-form, data-free ephemeris for small programs |
| **Paul Schlyter, "How to compute planetary positions"** | Tutorial at **https://stjarnhimlen.se/comp/ppcomp.html** — step-by-step: orbital elements, Sun, sidereal time, Moon & planets, position in space, precession, Moon perturbations, Jupiter/Saturn/Uranus perturbations, geocentric/equatorial/azimuthal coords, Moon topocentric correction, Pluto, elongation & physical ephemerides, asteroids, comets (parabolic/near-parabolic/hyperbolic), rise/set times, element validity; with numerical test cases | His own statement: "a **fraction of an arc minute** for the Sun and the inner planets, about **one arc minute** for the outer planets, and **1–2 arc minutes** for the Moon"; based on simplifying van Flandern & Pulkkinen, "Low precision formulae for planetary positions" (ApJ Suppl. 40, 405, 1980); "first implemented on a HP-41C" in <2 KB RAM | §22: the Sun/Moon/major-planet elements given are "valid for a long time period"; comet/asteroid elements only for limited spans | The canonical low-precision walkthrough; ideal spec for a first ephemeris script family | **Maximally feasible**: closed-form, few dozen lines per body |
| **JPL Horizons** (Solar System Dynamics Group) | Online service: "flexible production of **highly accurate ephemerides** for solar system objects (asteroids, comets, planetary satellites, planets, the Sun, L1, L2, select spacecraft, and system barycenters)" via web, command-line, email, and API | The reference standard (JPL DE integrations; DE430/431-class) | All of history per ephemeris version | Ground truth for testing scripts; not embeddable | **Not a script** — needs network + big data; use to *validate* scripts |
| **Astronomical Almanac / Nautical Almanac data (USNO + HM Nautical Almanac Office)** | Annual printed/online almanacs: "The Astronomical Almanac, The Nautical Almanac, The Air Almanac, Astronomical Phenomena, and The Astronomical Almanac Online … for use in navigation, surveying, scientific research, litigation…" (https://aa.usno.navy.mil/publications); USNO also publishes FAQ "computational notes" (Julian-date formula, approximate solar coordinates, altitude & azimuth, approximate sidereal time, equation of time, Moon illumination %) under https://aa.usno.navy.mil/faq/ (nav visible on e.g. https://aa.usno.navy.mil/faq/rs_algor) | Almanac-grade (arcsecond-class positions) | Annual tabulations | Navigation, surveying, almanac users; USNO's rise/set/twilight data services | USNO's **formulas** are script-feasible; its **tables** are not |
| **USNO rise/set/transit formulas** | USNO's live guidance for computing rise/set/twilight: points to Explanatory Supplement 3rd ed. **ch. 12** and Meeus **ch. 15** as the canonical algorithm sources; the standard hour-angle method with h₀ = −0.833° for sunrise/sunset (refraction + solar semidiameter) is also laid out by NOAA (below) and the Wikipedia *Sunrise equation* article (https://en.wikipedia.org/wiki/Sunrise_equation, citing Meeus p. 98) | Matches almanac tabulations to ~1 min under standard atmosphere | Any epoch with valid ephemerides | Sunrise/sunset/twilight/moonrise/moonset calculators everywhere | **Feasible**: ~30 lines once you have solar/lunar RA/Dec |
| **NOAA solar calculator equations** | NOAA GML's Solar Calculator: "based on equations from *Astronomical Algorithms*, by Jean Meeus"; the companion PDF "General Solar Position Calculations" gives the closed forms: fractional year γ → equation of time & declination → true solar time → hour angle → zenith/azimuth; sunrise/sunset via zenith 90.833° | "sunrise and sunset results are theoretically accurate to **within a minute** for locations between ±72° latitude, and **within 10 minutes** outside those latitudes" (page notes it is no longer maintained) | Modern epoch | The de-facto public solar-position reference (solar energy, surveying) | **Feasible**: single small script, no tables |

Primary-source URLs for the table: VSOP87 —
https://cdsarc.cds.unistra.fr/ftp/VI/81/ReadMe and
https://cdsarc.cds.unistra.fr/ftp/VI/81/vsop87.txt; ELP —
https://cdsarc.cds.unistra.fr/ftp/VI/79/ReadMe; Moshier —
https://www.moshier.net/aadoc.html and https://www.moshier.net/; Schlyter —
https://stjarnhimlen.se/comp/ppcomp.html; Horizons —
https://ssd.jpl.nasa.gov/horizons/; NOAA —
https://gml.noaa.gov/grad/solcalc/calcdetails.html and
https://gml.noaa.gov/grad/solcalc/solareqns.PDF; USNO —
https://aa.usno.navy.mil/publications and https://aa.usno.navy.mil/faq/rs_algor.

**Ephemeris → takeaway for an "ephemeris" script topic family:**
- **Closed-form / script-feasible:** Schlyter's method (arcminute class), Meeus
  low-precision chapters + Appendix IV polynomials (QVSOP-style, 1998–2025),
  Moshier expansions (arcsecond class, ±3000–6000 yr, no data tables), NOAA solar
  equations, USNO rise/set hour-angle method, truncated VSOP87 (top-N waves),
  Meeus-abridged ELP (ch. 47).
- **Need tabulated data / network (not scripts):** full VSOP87/ELP2000-82B series,
  JPL DE files via Horizons or aa200 — these are what you *validate against*, not
  what you embed.

---

## 4. Astronomy software feature proxies: quantities practitioners compute

### 4.1 Stellarium (v26.2 user guide, 452-page PDF:
https://github.com/Stellarium/stellarium/releases/download/v26.2/stellarium_user_guide-26.2-1.pdf)

The **Astronomical Calculations window (AstroCalc)** is literally a checklist of
practitioner-computed quantities (§4.6 of the guide):

- **Positions tab** — equatorial J2000 or horizontal positions, magnitudes, surface
  brightness/separation filters for what's above the horizon now; "Major planets"
  subtab with heliocentric ecliptic positions and polar plot.
- **Ephemeris tab** — positions and magnitudes over a time range, including
  **horizontal-coordinate traces (analemma of the Sun)** and twilight-constrained
  visibility (e.g., Venus at civil twilight, Saturn opposition points over decades).
- **RTS tab** — "meridian transits and rising and setting times of selected
  celestial object … for a specific date range".
- **Phenomena tab** — "conjunctions, oppositions, occultations and eclipses …
  greatest elongations for the inner planets and stationary points for all planets,
  and … perihelia and aphelia"; output columns include **solar elongation, lunar
  elongation, magnitudes**.
- **Graphs tab** — altitude vs. time with civil/nautical/astronomical twilight lines
  and Moon altitude overlay; lunar elongation graphs.
- **WUT tab** — "What's Up Tonight" above-horizon planner.
- **Planetary Calculator tab** — relations between two solar-system bodies:
  linear and angular distances, orbital resonances, orbital velocities.
- **Eclipses tab** and **Almanac tab** (solar/lunar eclipse lists; almanac data).

### 4.2 Skyfield (Python) — "Almanac Computation" documentation
(https://rhodesmill.org/skyfield/almanac.html)

One page enumerates the almanac quantities users ask of it: risings and settings ·
sunrise and sunset · **polar day and polar night detection** · moonrise and moonset ·
planet rising/setting · custom refraction angles · elevated vantage points ·
rise/set of an RA/Dec · **the seasons (equinoxes/solstices)** · phases of the Moon ·
**lunar nodes** · **opposition and conjunction** · meridian transits · **twilight** ·
solar terms · lunar eclipses.

### 4.3 PyEphem — Quick Reference (https://rhodesmill.org/pyephem/quick.html)

`body.compute()` produces the quantities an ephem-style calculator exposes:
astrometric RA/Dec (`a_ra`,`a_dec`), apparent geocentric RA/Dec (`g_ra`,`ra`),
**elongation (signed, morning/evening side)**, **magnitude**, **angular size**,
**circumpolar / neverup flags**, heliocentric longitude/latitude (`hlon`,`hlat`),
**sun_distance, earth_distance, phase (% illuminated)**; plus coordinate conversion,
transit/rising/setting, equinoxes & solstices, Moon phases, angular separation.

### 4.4 Cartes du Ciel (SkyChart) — https://www.ap-i.net/skychart/en/start

"Draw sky charts, making use of the data in many catalogs of stars and nebulae. In
addition the **position of planets, asteroids and comets** are shown … a large number
of parameters … the display of labels and coordinate grids, the superposition of
pictures, **the condition of visibility**…" — i.e., catalog charting plus
solar-system positions and visibility conditions.

### 4.5 Heavens-Above — https://www.heavens-above.com/

Site sections (nav text, 2026-09-07): **satellites** (10-day ISS predictions,
daily predictions for brighter satellites, Starlink launches/passes, amateur-radio
satellite all-passes), and **Astronomy**: Solar Eclipses · interactive sky chart ·
Sun · Moon · Planets · Solar-system chart · Comets · Asteroids · Constellations.

### 4.6 timeanddate.com astronomy pages

Live site is bot-walled; Wayback captures cited.

- **Sun page** for a city (columns of the monthly table):
  http://web.archive.org/web/20250101073259/https://www.timeanddate.com/sun/usa/new-york
  — "2025 Sunrise/Sunset · Daylength · Astronomical Twilight · Nautical Twilight ·
  Civil Twilight · Solar Noon" with per-day **sunrise/sunset azimuths (°), day
  length, day-to-day difference, twilight start/end, solar-noon time and Sun
  distance**; the summary panel shows Sun direction, altitude, distance, next
  equinox. The lookup page advertises "Sunrise, Sunset, dusk, dawn and twilight,
  Sun distance, day length, altitude, and much more"
  (http://web.archive.org/web/20241231234417/https://www.timeanddate.com/sun/).
- **Moon page** for a city:
  http://web.archive.org/web/20241220230834/https://www.timeanddate.com/moon/usa/new-york
  — per-day **Moonrise/Moonset (with azimuths) · Meridian passing (time, altitude) ·
  Moon distance (mi) · Illumination (%)**; summary panel: current phase %, direction,
  altitude, distance, next new/full moon.

### 4.7 Wikipedia "Astronomical coordinate systems" (category + article)

- Article structure — coordinate systems and their conversions, each a
  calculator-suitable formula cluster:
  https://en.wikipedia.org/wiki/Astronomical_coordinate_systems (horizontal ·
  equatorial · ecliptic · galactic · supergalactic systems; conversions: hour angle
  ↔ right ascension, equatorial ↔ ecliptic, equatorial ↔ horizontal, equatorial ↔
  galactic).
- Category members (52 articles;
  https://en.wikipedia.org/wiki/Category:Astronomical_coordinate_systems) map to
  script-sized quantities: almucantar, declination, right ascension, hour angle,
  hour circle, meridian, vertical circle, prime vertical, zenith, nadir, horizon,
  celestial pole, polar distance, parallactic angle, position angle, circumpolar
  star, subsolar point, equinox (celestial coordinates), first point of Aries,
  colures, ecliptic (system), galactic (system), supergalactic (system), barycentric
  and geocentric celestial reference systems, ICRS, Earth-centered inertial, ECEF,
  solar coordinate systems, solar longitude, zodiac, celestial sphere, and more.

**Software → takeaway:** the recurring practitioner checklist is: rise/set/transit
(with twilight bands and azimuths) · alt/az now · Sun position (declination,
equation of time, analemma) · Moon phase/illumination/libration · eclipses ·
elongation & signed morning/evening side · magnitude & angular size · distances
(sun/earth) · oppositions/conjunctions/elongations/stationary points · precession &
frame conversions · "what's up tonight" filtering.

---

## 5. Candidate script topics distilled

Flat, deduplicated list of **90 concrete calculational topics**. The tag marks the
evidence source that most directly motivates the topic: **[book]** (leading books,
§1), **[community]** (shared calculator programs, §2), **[ephemeris]** (model
landscape, §3), **[software]** (software feature proxies, §4).

**Calendars and time**

1. Julian day ↔ calendar date conversion [book]
2. Day-of-year and day-of-week computations [book]
3. Date of Easter [book]
4. Jewish calendar date conversion [book]
5. Moslem (Hijri) calendar conversion [book]
6. Dynamical time ↔ universal time (ΔT estimation) [book]
7. Local time ↔ UT conversion with time zone [book]
8. Greenwich sidereal time (UT↔GST) [book]
9. Local sidereal time (GST↔LST) [book]
10. Equation of time [book]

**Coordinates and frames**

11. Obliquity of the ecliptic & nutation (Δψ, Δε) [book]
12. Precession of equatorial/ecliptic coordinates between epochs [book]
13. Aberration correction [book]
14. Transformation: equatorial ↔ ecliptic [book]
15. Transformation: equatorial ↔ horizontal (alt/az) [book]
16. Transformation: equatorial ↔ galactic [book]
17. Parallactic angle [book]
18. Angular (great-circle) separation of two objects [book]
19. Apparent place of a star (full reduction) [book]
20. Geodetic ↔ geocentric coordinates on the Earth's globe [book]

**Sun**

21. Solar coordinates (longitude, RA/Dec, distance) [book]
22. Rectangular (XYZ) coordinates of the Sun [book]
23. Times of equinoxes and solstices (seasons) [book]
24. Physical ephemeris of the Sun (P, B₀, L₀ / heliographic) [book]
25. Heliographic coordinates and Carrington rotation number [book]
26. Analemma (Sun declination/RA-offset through the year) [community]
27. NOAA-style solar position (declination + equation of time + hour angle, ±1 min rise/set) [ephemeris]
28. Sun altitude/azimuth at a given time & place ("Sunθ84" style) [community]

**Rise, set, twilight**

29. Rising, transit and setting times (hour-angle method) [book]
30. Twilight times (civil / nautical / astronomical) [book]
31. Atmospheric refraction correction (apparent ↔ true altitude) [book]
32. Atmospheric extinction (sec-z law) [book]
33. Rise/set azimuths of Sun and Moon [software]
34. Day length and day-to-day difference table [software]
35. Polar day / polar night (midnight sun) detection [software]
36. Meridian transit times and altitudes (Sun/Moon/planets) [software]
37. Prayer-times style solar hour-angle computation [community]

**Moon**

38. Position of the Moon (abridged ELP-2000) [book]
39. Moon's distance, angular size and horizontal parallax [book]
40. Moon phase / illuminated fraction of the disk [book]
41. Times of lunar phases (new/first/full/last) [book]
42. Next full/new moon (any phase) date finder [community]
43. Moon perigee and apogee (times and distances) [book]
44. Maximum declinations of the Moon [book]
45. Node passages of the Moon [book]
46. Physical ephemeris of the Moon (librations, selenographic coordinates, terminator colongitude) [book]
47. Moon distance / altitude / direction almanac panel [software]

**Eclipses and occultations**

48. Solar and lunar eclipse circumstances (local contacts, magnitude) [book]
49. Eclipse-season "rules of eclipses" test [book]
50. Eclipse checker/finder for a given date [community]
51. Occultations of stars by the Moon [book]
52. Semidiameters of the Sun, Moon and planets [book]

**Planets**

53. Kepler's equation solver (elliptic) [book]
54. Mean orbital elements of the planets (and rates) [book]
55. Low-precision positions of the planets [book]
56. Perturbation terms for a planet's orbit (Jupiter/Saturn class) [book]
57. Opposition and conjunction dates of planets [book]
58. Greatest elongations of Mercury and Venus [software]
59. Stationary points of planets [software]
60. Perihelion and aphelion times of a planet [book]
61. Node passages of a planet [book]
62. Illuminated fraction of a planet's disk and its magnitude [book]
63. Phase angle and position angle of the bright limb [book]
64. Solar elongation of a planet (incl. signed morning/evening side) [software]
65. Pluto's heliocentric position (fit valid ~1880–2110) [book]
66. Physical ephemeris of Mars (central meridian, axial tilt) [book]
67. Physical ephemeris of Jupiter (central meridians, GRS longitude) [book]
68. Positions of the four Galilean satellites [book]
69. Saturn's ring tilt (aspect of the ring plane) [book]
70. Positions of the satellites of Saturn [book]
71. Interplanetary distance & orbital-velocity relations (two bodies) [software]

**Comets, asteroids, stars, other**

72. Elliptic motion of comets/asteroids (epoch propagation) [book]
73. Parabolic comet orbit [book]
74. Near-parabolic comet orbit [book]
75. Hyperbolic comet orbit [ephemeris]
76. Combined/differential stellar magnitudes [book]
77. Binary-star apparent orbit (position angle & separation vs time) [book]
78. Planar sundial construction (hour-line angles) [book]
79. Heliacal rising and setting of stars/planets [book]
80. Months with five lunar phases (cycle check) [book]
81. Declination of Polaris over a century [book]
82. Catalog (Messier/NGC/star) coordinates → horizon coords for tonight [community]
83. Celestial navigation fix from observed altitudes (least squares, dip/refraction) [community]
84. "What's up tonight": above-horizon, magnitude-filtered object list [software]
85. Altitude-vs-time curve data for observation planning [software]

**Ephemeris engines (script-feasible models)**

86. VSOP87 (subset D, of-date) planetary positions, truncated to top-N terms [ephemeris]
87. Moshier DE404-fit planetary ephemeris (0.1″–1″, 3000 BC–3000 AD, no tables) [ephemeris]
88. Moshier/modified-Chapront lunar ephemeris (0.5″ vs DE404) [ephemeris]
89. Lunar libration & selenographic series (0.05″, −1369…+2950) [ephemeris]
90. Schlyter low-precision Sun/Moon/planet method (1–2 arcmin, HP-41 heritage) [ephemeris]

---

## Appendix: source-access log (2026-09-07)

- Live-verified: ticalc.org, aa.usno.navy.mil (intermittent 500s on /data/docs),
  ssd.jpl.nasa.gov, gml.noaa.gov, rhodesmill.org, ap-i.net, heavens-above.com,
  my.numworks.com, tibasicdev.wikidot.com, cdsarc.cds.unistra.fr, stjarnhimlen.se,
  moshier.net, openlibrary.org, books.google.com, en.wikipedia.org.
- Bot-blocked live (Wayback used): willbell.com (store down), uscibooks.com
  (redirects to AIP), cambridge.org/core, springer.com, hpcalc.org (garbage
  response), hpmuseum.org (403), cemetech.net (Anubis check), timeanddate.com (403).
- Archive.org Wayback went briefly offline mid-session; all snapshot URLs above were
  fetched successfully during the session.
