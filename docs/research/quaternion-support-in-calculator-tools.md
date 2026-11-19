# Quaternion support in the calculator tools epher compares against

**Goal:** decide what "established practice" looks like for quaternion support by
surveying the nine math tools named on the comparison page (`site/features.html`),
against primary sources only (official docs, vendor guides, official manuals). For
each tool: is there a quaternion type or function set, is it built-in or an add-on,
what is the exact construction/display syntax, which operations exist, which scalar
convention (w-first or w-last), and how the same tool spells complex numbers as
calibration. Ends in a summary table and a best-practice paragraph aimed at epher.

**Method (all claims sourced, not assumed):** every URL below was checked on
**2026-11-18** unless noted. Access notes: `mathworks.com` blocks scripts, so the
Robotics page was verified via a Wayback capture (2026-04-29) of the live URL;
TI's download-center links are JavaScript-gated, so official TI PDFs were fetched
directly from TI's media server and full-text searched; the HP Prime guide was
read from an archive.org copy of the official HP PDF (HP's site is
script-hostile). Unverified items are marked **UNVERIFIED**.

---

## 1. NumWorks

- **Quaternions: no.** All official release-notes pages (version-01 through
  version-26, plus the update index at https://www.numworks.com/calculator/update/)
  were fetched and searched: zero mentions of "quaternion" (checked 2026-11-18).
- **Complex numbers:** supported since Epsilon 10 ("Complex numbers and turtle
  module", 2019-03-13): exact complex results, pi-form arguments, Settings
  "Complex format" section with Real (default, `sqrt(-1)` returns "unreal"),
  Cartesian and Exponential options. Source:
  https://www.numworks.com/calculator/update/version-10/

## 2. Desmos

- **Quaternions: no.** The official help-center search API
  (`https://help.desmos.com/api/v2/help_center/articles/search.json?query=quaternion`)
  returns **0 results** (checked 2026-11-18).
- **Complex numbers:** "Complex numbers are written in the form a+bi";
  Scientific, Graphing and 3D calculators have a Complex Mode toggle that adds
  `i` to the keypad, with `real(...)`, `imag(...)`, conjugate, argument and
  modulus functions. Source:
  https://help.desmos.com/hc/en-us/articles/31103542590733-Complex-Numbers

## 3. GeoGebra

- **Quaternions: no.** The current manual sitemap
  (https://geogebra.github.io/docs/sitemap-manual.xml, 7977 URLs, all languages)
  contains **zero** pages mentioning "quaternion" (checked 2026-11-18). 3D
  rotations are documented as axis/angle commands (Rotate), not quaternions.
- **Complex numbers:** the manual states "GeoGebra does not support complex
  numbers directly, but you can use points and vectors to display complex
  numbers in the plane"; entering `3 + 4ί` shows the point (3, 4); plus a
  Complex Number tool and commands such as ComplexRoot. Source:
  https://geogebra.github.io/docs/manual/en/Complex_Numbers/

## 4. Texas Instruments (TI-84 Plus CE, TI-Nspire CX II)

### 4.1 TI-84 Plus CE family

- **Quaternions: no** (official guide searched). The official TI "Getting Started
  Guide" for TI-84 Plus CE Python (78 pages, PDF served by TI's own media server)
  contains **0** "quaternion" hits.
  Source: https://education.ti.com/-/media/files/download-center/guidebooks/graphing/84ce-python/5,-d-,7/gettingstartedguide/84plcepy_gsg_en.aspx
- The dedicated TI-84 Plus CE Reference Guide PDF is only reachable through TI's
  JavaScript download center, so absence in that exact volume is **UNVERIFIED**
  (linked from https://education.ti.com/en/product-resources/guidebooks/graphing-calculators).
- **Complex numbers:** dedicated CMPLX menu documented in the guide: `conj(`,
  `real(`, `imag(`, `angle(` (polar angle), `abs(` (magnitude); a+bi /
  re^(theta)i display modes.

### 4.2 TI-Nspire CX II

- **Quaternions: no** (official guide searched). The official TI-Nspire CX II
  Handhelds Guidebook (2021, 107 pages) full-text searched: **0** "quaternion"
  hits (TI media server, checked 2026-11-18):
  https://education.ti.com/-/media/files/download-center/guidebooks/ti-nspire/5,-d-,4/gb_ti-nspire_cxii_handhelds/ti-nspire_cxii-hh_guidebook_en.aspx
- The alphabetical TI-Nspire CX II Reference Guide volume is likewise
  JavaScript-gated; absence there **UNVERIFIED**.
- **Complex numbers:** Settings offer "Real or Complex: Real, Rectangular,
  Polar" (guidebook, Settings section).

## 5. WolframAlpha / Wolfram Language

- **Quaternions: yes, add-on.** The Quaternions package (now shipped as the
  Quaternions paclet) must be loaded first: `Needs["Quaternions\`"]`; "To use
  Quaternion, you first need to load the Quaternions Package". Not a core type.
- **Construction / display:** `Quaternion[a, b, c, d]` "represents the quaternion
  a + I b + J c + K d" (w-first: `a` is the scalar part). Tutorial: "all
  quaternions should be entered using the form Quaternion[a, b, c, d]" and "Only
  limited support is offered to the symbolic form a + I b + J c + K d". Sources:
  https://reference.wolfram.com/language/Quaternions/ref/Quaternion.html and
  https://reference.wolfram.com/language/Quaternions/tutorial/Quaternions.html
- **Operations:** package functions AbsIJK, AdjustedSignIJK, From/ToQuaternion,
  QuaternionQ, UnitQuaternionQ(s), IntegerQuaternionQ, ScalarQ, associates/GCD,
  primary associates; plus overloaded core Abs, Conjugate, Norm, Re (scalar
  part), Sign, Round, Mod, PrimeQ. **No** exp, log, power, rotation or slerp
  functions are documented.
  Source: https://reference.wolfram.com/language/Quaternions/guide/QuaternionsPackage.html
- **Scalar convention:** w-first (a is the scalar part); the package "implements
  Hamilton's quaternion algebra" (tutorial). **Complex (calibration):**
  `x + I y`; FullForm is `Complex[x, y]`
  (https://reference.wolfram.com/language/ref/Complex.html).
- **WolframAlpha:** the query `Quaternion[2,1,1,1]` is accepted and echoed (page
  title "Quaternion[2,1,1,1] - Wolfram|Alpha"), but results are client-rendered;
  whether WA evaluates quaternion algebra is **UNVERIFIED** by scripted access.

## 6. SpeedCrunch

- **Quaternions: no.** Zero "quaternion" mentions across the official docs site
  (docs sources: https://github.com/speedcrunch/speedcrunch.github.io, checked
  2026-11-18).
- **Complex numbers:** "SpeedCrunch supports complex-number expressions by
  default" since version 0.12; imaginary unit entered as `i` or `j`
  (`(5 + 3j) / (8 - 2j) = 0.5+0.5j`); phasor notation `r < theta` equals
  `r*cis(theta)`; functions `real(x)`, `imag(x)`, `conj(x)`, `phase(x)`,
  `abs(4+3j) = 5`; display symbol and complex form configurable. Sources:
  https://www.speedcrunch.org/userguide/syntax.html and
  https://www.speedcrunch.org/reference/basic.html

## 7. HiPER Scientific Calculator

- **Quaternions: no.** Zero "quaternion" mentions on the official help and
  features pages (hipercalc.app, checked 2026-11-18). **Complex numbers:**
  features list "Complex numbers in rectangular and polar form" and "complex
  plane"; help discusses real vs complex domains (e.g. the multivalued complex
  square root). Sources: https://hipercalc.app/features.html and
  https://hipercalc.app/help.html (developer site: https://hiperlabs.eu/)

## 8. HP Prime

- **Quaternions: no** (official User Guide searched). The HP Prime Graphing
  Calculator User Guide (official HP PDF, "Copyright 2015-2017 HP Development
  Company" edition, 1.2 MB full text from archive.org) has **0** "quaternion"
  hits; its numeric types are real, integer, complex, matrix, list, etc.
  Source: https://archive.org/details/hp-prime-graphing-calculator-user-guide
- Newer firmware editions of the guide: **UNVERIFIED** (not script-accessible).
- **Complex numbers:** Home settings offer complex display as `(a,b)` or
  `a+b*i`; complex arithmetic supported in Home and CAS.

## 9. MATLAB / GNU Octave

### 9.1 MATLAB

- **Quaternions: yes, add-on (toolbox).** The `quaternion` class is documented
  under **Robotics System Toolbox** (URL path /help/robotics/), "Introduced in
  R2018a"; base MATLAB has no quaternion type. Cross-listing for Aerospace
  Toolbox / Sensor Fusion installs: **UNVERIFIED** (pages blocked, no snapshot).
- **Construction / display:** `quat = quaternion(A,B,C,D)`; the docs say
  `quaternion(1,2,3,4)` "creates a quaternion of the form 1 + 2i + 3j + 4k", and
  display is exactly that form (`1.1 + 2.1i + 3.1j + 4.1k`, w-first). Also from
  rotation vectors (`"rotvec"`), matrices (`"rotmat"`), Euler angles
  (`"euler"`/`"eulerd"`), se3/SO(3) objects.
- **Operations (doc page's own catalog):** arithmetic, left/right division,
  `conj`, `ctranspose`, `norm`, `normalize`, `exp`, `rotateframe`,
  `rotatepoint`, `dist`, `angvel`; conversions `compact` (N-by-4), `euler`,
  `eulerd`, `rotmat`, `rotvec`, `rotvecd`; component extraction via `parts`;
  interpolation `interp1` and `slerp` (extended R2025a to natural/SQUAD/SLERP).
  Log/powers via `log`/`power` object functions on the same class page family:
  **UNVERIFIED** (not visible in the captured snapshot's excerpt).
- Source (live URL, verified via Wayback capture 2026-04-29):
  https://www.mathworks.com/help/robotics/ref/quaternion.html
- **Complex numbers (calibration):** `z = complex(a,b)` gives `z = a + bi`;
  literals `a + 1i*b` / `a + 1j*b`.
  Source: https://www.mathworks.com/help/matlab/ref/complex.html (Wayback 2025)

### 9.2 GNU Octave

- **Quaternions: no in base; yes via add-on package.** The current manual's
  Function Index (https://docs.octave.org/latest/Function-Index.html) has zero
  quaternion entries while `complex`, `conj`, `real`, `imag`, `abs` are present.
  The Octave Forge **quaternion package** adds a `@quaternion` class "with
  overloaded operators".
- **Construction / display:** `q = quaternion(w, x, y, z)` (also `quaternion(w)`
  and `quaternion(x, y, z)` with zero parts assumed); docs: "q = w + x*i + y*j +
  z*k", example display `2 + 3i + 4j + 5k` (w-first). Unit constants allow
  `1 + 2*qi + 3*qj + 4*qk`; components are fields `q.w`, `q.x`, `q.y`, `q.z`
  plus `get`/`set`. Sources:
  https://octave.sourceforge.io/quaternion/function/quaternion.html and
  https://octave.sourceforge.io/quaternion/function/qi.html
- **Operations:** `abs`, `arg`, `conj`, `exp`, `log`, `inv`, `norm`, `ispure`,
  `isreal`, `mean`, `sum`/`cumsum`, `diff`, array ops (`cat`, `blkdiag`,
  `repmat`, ...); rotation conversions `q2rot` (vector/angle), `rot2q`
  (angle+axis to unit quaternion), `rotm2q` (matrix to quaternion). Powers and
  slerp are not in the package's documented function list.
- **Complex numbers (calibration):** `complex(3,4)`, literal `3+4i`.

## Summary table

| Tool | Quaternions | How | Primary source |
|---|---|---|---|
| NumWorks | no | nothing | release notes v1-v26, numworks.com/calculator/update/ |
| Desmos | no | nothing | help.desmos.com search (0 results) |
| GeoGebra | no | nothing | manual sitemap (0 pages), geogebra.github.io/docs |
| TI-84 Plus CE | no | nothing (complex only) | official CE Getting Started Guide PDF (education.ti.com) |
| TI-Nspire CX II | no | nothing (complex only) | official CX II Handhelds Guidebook PDF, 2021 |
| WolframAlpha / WL | yes (add-on) | Quaternions package/paclet, `Quaternion[a,b,c,d]` | reference.wolfram.com/language/Quaternions/ |
| SpeedCrunch | no | nothing (complex yes) | speedcrunch.org docs (userguide + reference) |
| HiPER Scientific | no | nothing (complex yes) | hipercalc.app/help.html, features.html |
| HP Prime | no | nothing (complex only) | HP Prime User Guide PDF (2015-2017 ed.) |
| MATLAB | yes (add-on) | `quaternion(w,x,y,z)` class, Robotics System Toolbox | mathworks.com/help/robotics/ref/quaternion.html |
| GNU Octave | yes (add-on) | Forge quaternion package, `quaternion(w,x,y,z)` class | octave.sourceforge.io/quaternion/ |

## What established practice looks like

Across these nine tools, quaternion support exists only in the professional
numerical environments, never in the consumer calculators (NumWorks, TI, Desmos,
GeoGebra, SpeedCrunch, HiPER, HP Prime ship none, and every one of those ships
complex numbers as its sole non-real numeric type). Every tool that does support
quaternions agrees on three things: (1) a functional constructor with four
comma-separated components (MATLAB `quaternion(w,x,y,z)`, Octave Forge
`quaternion(w,x,y,z)`, Wolfram `Quaternion[a,b,c,d]` in a package); (2) the
Hamilton **w-first** order, scalar first, stated or shown by example in all
three; and (3) a display format matching the complex convention, literally
`a + bi + cj + dk` in MATLAB (`1 + 2i + 3j + 4k`) and Octave Forge
(`2 + 3i + 4j + 5k`), while Wolfram displays `Quaternion[a,b,c,d]` and offers
only "limited support" for symbolic `a + I b + J c + K d`. The documented
operation set varies: MATLAB's class is the most complete (arithmetic,
conjugate, norm, exp, rotation helpers, matrix/Euler conversion, slerp/interp1);
Octave Forge adds exp/log/inv plus rotation conversions; Wolfram's package stops
at abs/norm/conjugate/parts with no exp, log, powers or rotation helpers. So an
epher implementation with constructor `quat(w, x, y, z)` and Hamilton w-first
display `a + bi + cj + dk` is exactly consistent in argument order and display
with the only two tools that implement quaternions as a first-class numeric
type (MATLAB, Octave Forge); the short `quat` spelling is epher's own (surveyed
tools spell out "quaternion"), and it exceeds every consumer calculator in the
comparison set, none of which offers quaternions at all.
