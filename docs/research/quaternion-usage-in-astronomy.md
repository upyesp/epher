# Quaternion representation and use in astronomy: a primary-source survey

**Goal:** decide how epher should treat quaternions (notation, storage order,
convention, and which astronomy workflows actually need them), by reading the
owning primary sources: NAIF SPICE, CCSDS, FITS/WCS papers and standard, ERFA,
astropy, GMAT, Basilisk, Skyfield. Ends with a verdict on where quaternions are
native in astronomy and where matrices/vectors dominate.

**Method (all claims sourced, not assumed):** every factual claim below carries a
URL checked on **2026-11-18**. Quotes are verbatim from the cited document
(minus HTML/PDF extraction artifacts). Items that could NOT be confirmed from a
primary source are quarantined in section 5.

---

## 1. Notation and conventions

### 1.1 Hamilton form and algebra

- A quaternion is q = w + x*i + y*j + z*k, or scalar+vector q = s + v. SPICE:
  "We can represent Q using 'scalar + vector' notation... s = q0 =
  cos(theta/2), v = (q1, q2, q3)". Conjugate/norm/inverse: SPICE works with
  "unit-length SPICE-style quaternion... ||q|| = 1"; Geometric Tools states the
  conjugate q* and "q_hat" for the unit quaternion, with vector rotation
  R(v_hat) = q_hat * v_hat * q*.
  https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/q2m_c.html
  https://www.geometrictools.com/Documentation/Quaternions.pdf
- Hamilton product rule (sign of the cross term is the convention): SPICE:
  "Q1*Q2 = s1*s2 - <v1,v2> + s1*v2 + s2*v1 + (v1 x v2)" and "If Q1 and Q2
  represent the rotation matrices M1 and M2 respectively, then the quaternion
  product Q1*Q2 represents the matrix product M1*M2". SPICE attributes this
  style to Hamilton ("Invented by Sir William Rowan Hamilton / Frequently used
  in mathematics and physics textbooks") versus the "Engineering" style "Widely
  used in aerospace engineering applications". The i^2 = j^2 = k^2 = ijk = -1
  algebra itself is only implied by these sources (section 5).

### 1.2 Storage order: scalar-first vs scalar-last

- **Scalar-first (w,x,y,z):** SPICE/CSPICE, always: "CSPICE function
  interfaces ALWAYS use SPICE quaternions", q = (cos(theta/2), sin(theta/2)a1,
  sin(theta/2)a2, sin(theta/2)a3). Same q2m_c URL as above.
- **Scalar-last (x,y,z,w), aerospace:** SPICE's "engineering" quaternions:
  "(+/-) (-sin(theta/2)A(1), -sin(theta/2)A(2), -sin(theta/2)A(3),
  cos(theta/2))", with explicit conversion "QSPICE = ( q3, -q0, -q1, -q2 )".
  GMAT (flight dynamics): "GMAT's quaternion representation includes the three
  'vector' components as the first three elements in the quaternion and the
  'rotation' component as the last element", Q4 default 1. CCSDS (message
  standard): AEM/ACM fields EPOCH, Q1, Q2, Q3, QC with "QC = cos(phi/2)" last,
  default identity "0 0 0 1". Robotics: ROS 2 tf2 lists tf2::Quaternion
  alongside Matrix3x3/Transform (tf2/Eigen storage is scalar-last).
  https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/q2m_c.html
  https://raw.githubusercontent.com/nasa/GMAT/main/doc/help/src/Resource_SpacecraftAttitude.xml
  https://public.ccsds.org/Pubs/504x0b2.pdf
  https://docs.ros.org/en/rolling/p/tf2/
- Practical rule: element order is a per-tool/per-format convention that must
  be documented; SPICE warns "multiple styles of quaternions are in use. This
  routine will not work properly if the input quaternions do not conform to
  the SPICE convention."
  https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/qxq_c.html

### 1.3 Hamilton vs JPL/Engineering convention

- The substance of the "Hamilton vs JPL" split is SPICE's SPICE-vs-Engineering
  paragraph: the two styles differ in element order AND sign; the engineering
  quaternion is effectively the negated conjugate of the SPICE one, so the two
  compose rotations in opposite orders (Hamilton right-handed: Q1*Q2 =
  M1*M2). Math texts and SPICE: Hamilton scalar-first. Aerospace practice
  (GMAT, CCSDS, SPICE's own "engineering" style): scalar-last. (The literature
  name "JPL/Shuster convention" is attributed to Shuster's survey; see
  section 5.) Source: q2m_c URL above.

### 1.4 Notation seen in the surveyed documents

- SPICE: q = (q0, q1, q2, q3), "unit quaternion"; q paired with its time
  derivative in qdq2av_c ("Quaternion and derivative to angular velocity").
  https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/rotation.html
- CCSDS ADM: Q1, Q2, Q3, QC; derivatives Q1_DOT..QC_DOT.
  https://public.ccsds.org/Pubs/504x0b2.pdf
- Basilisk calls quaternions "EP" (Euler parameters): C2EP, EP2C, BmatEP,
  addEP, alongside MRP variants.
  https://raw.githubusercontent.com/AVSLab/basilisk/master/src/architecture/utilities/rigidBodyKinematics.h
- Geometric Tools: q_hat, q*, Slerp(t; q0, q1) (URL above). The spacecraft
  term "Euler symmetric parameters" (Wertz) was not verified (section 5).

---

## 2. Quaternions as rotations: the mechanics astronomers rely on

- **Unit quaternion = rotation (axis-angle):** "The four numbers of a
  quaternion represent a unit vector and an angle... Q = (q0, q1, q2, q3) =
  ( cos(T/2), sin(T/2)*u1, sin(T/2)*u2, sin(T/2)*u3 )".
  https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/ck.html
- **Double cover:** "if a quaternion q represents a matrix M, then so does the
  quaternion -q"; m2q_c fixes sign ("always returns a quaternion with scalar
  part greater than or equal to zero") and warns that q2m then m2q "might
  preserve q or convert q to -q".
  https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/cspice/q2m_c.html
- **Sign continuity for interpolation:** "Although q1 and -q1 represent the
  same rotation, the values of Slerp(t; q0, q1) and Slerp(t; q0, -q1) are not
  the same. It is customary to choose the sign sigma on q1 so that q0 .
  (sigma*q1) >= 0... This choice avoids extra spinning."
  https://www.geometrictools.com/Documentation/Quaternions.pdf
- **Rotating a vector (sandwich):** "The rotated vector, represented as a
  quaternion, is R(v_hat) = q_hat v_hat q*" (same URL); for unit q, inverse =
  conjugate, i.e. v' = q v q*.
- **Matrix conversion, angle, rates:** q2m_c gives the element-by-element 3x3
  formula for M from q; m2q_c inverts it with "the rotation angle of m to
  lie in [0, pi]... Q(0) is always in [0, 1]" (so theta = 2*acos(q0)); and
  qdq2av_c converts quaternion + time derivative to angular velocity.
  https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/rotation.html
- **Interpolation in flight products:** SPICE CK type 5 subtype 1: "Sliding-
  window Lagrange interpolation of quaternions... The interpolated quaternion
  is then unitized"; subtype 0: Hermite on "four quaternion components
  followed by four quaternion time derivative components".
  https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/ck.html
- **SLERP vs nlerp:** slerp is Slerp(t;q0,q1) = (q0 sin((1-t)theta) +
  q1 sin(t theta)) / sin(theta); scipy documents its Slerp class as "a
  rotation around a fixed axis with a constant angular velocity... the
  shortest path". Normalized LERP is the cheap small-angle approximation
  (constant-speed claim is the verified slerp property; the nlerp trade-off is
  textbook, section 5).
  https://www.geometrictools.com/Documentation/Quaternions.pdf
  https://docs.scipy.org/doc/scipy/reference/generated/scipy.spatial.transform.Slerp.html
- **Composition order and direction bookkeeping:** q1 first then q2 composes
  as q2*q1 (from "Q1*Q2 represents M1*M2" with SPICE's V_TO = M * V_FROM).
  CCSDS annex F: "The quaternion called 'from frame A to frame B' is defined
  as the quaternion of the rotation that transforms the basis vectors of
  frame A into the basis vectors of frame B", X_B = M_BA * X_A, with the full
  M_BA matrix given in terms of Q1..QC. URLs: q2m_c and CCSDS PDF above.

---

## 3. Astronomy software, one by one

### 3.1 NASA/NAIF SPICE (quaternion-native for spacecraft attitude)

- C-kernels (CK) ARE the attitude files; every data type stores quaternions.
  Type 1 record: q0..q3 then optionally angular velocity a1..a3. "Quaternions
  are widely used to represent rotation matrices. They require less than half
  the space of 3x3 matrices and finding the rotation matrix given by a
  quaternion is faster and easier than finding it from, say, RA, Dec, and
  Twist." Order: q0 is the scalar (cos(T/2)).
  https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/ck.html
- Core API: q2m_c, m2q_c, qxq_c, qdq2av_c ("Quaternions are used in CSPICE as
  a compact representation for rotations. They may be used to compose
  rotations more efficiently than can be done with matrix multiplication.").
  https://naif.jpl.nasa.gov/pub/naif/toolkit_docs/C/req/rotation.html
- Missions really ship CKs: Cassini's public NAIF area lists hundreds of
  attitude files (*.bc):
  https://naif.jpl.nasa.gov/pub/naif/CASSINI/kernels/ck/
- Historical note: the pre-quaternion representation was RA/Dec/Twist Euler
  angles ("Most projects, including Voyager and Cassini define the
  relationship as c = [Twist][pi/2-Dec][pi/2+RA]"; Galileo differed);
  rotation.req URL above.

### 3.2 SOFA (IAU) and ERFA (matrix/angle only)

- ERFA (canonical open implementation of the SOFA algorithms): the entire
  public header (589 lines, all routines) contains ZERO occurrences of
  "quaternion"; every rotation entity is a 3x3 matrix or angles.
  https://raw.githubusercontent.com/liberfa/erfa/master/src/erfa.h SOFA's own
  site is currently a JS-driven page; its routine list was not fetchable, so
  "SOFA itself has no quaternion routine" is verified via ERFA only
  (section 5). https://www.iausofa.org/current-software

### 3.3 astropy (matrix-native)

- astropy.coordinates transformations: zero "quaternion" mentions; the
  machinery is AffineTransform/BaseCoordinateFrame plus explicit
  rotation-matrix helpers; matrix_utilities.py: "Utilities used for
  constructing and inspecting rotation matrices". No quaternion type in core
  docs; no affiliated quaternion package referenced.
  https://docs.astropy.org/en/stable/coordinates/transforming.html
  https://raw.githubusercontent.com/astropy/astropy/main/astropy/coordinates/matrix_utilities.py

### 3.4 FITS WCS (matrix-native; no quaternion keywords)

- WCS rotations are the linear transformation matrix PC_i_ja (with CDELTi
  scales) or the equivalent CD_i_j; the legacy AIPS-style CROTA2 keyword is
  translated to a PC matrix (Paper II Eqs. 186-188; "defines a rotation if
  and only if lambda = +/- 1"). No quaternion mention in the paper.
  Calabretta & Greisen 2002, A&A 395, 1077:
  https://www.atnf.csiro.au/people/mcalabre/WCS/ccs.pdf
- FITS 4.0 standard: "Use of the CROTA2 keyword is now deprecated, and
  instead the newer PCi_j or CDi_j keywords are recommended"; "quaternion"
  appears exactly once in the whole standard, speculatively: complex data
  "future revisions might admit other elements such as quaternions or general
  tensors". Spacecraft attitude is not standardized in FITS at all; missions
  use external products (SPICE CK, CCSDS messages).
  https://fits.gsfc.nasa.gov/standard40/fits_standard40aa-le.pdf

### 3.5 Spacecraft attitude practice

- **CCSDS Attitude Data Messages 504.0-B-2 (Blue Book, January 2024):** the
  interagency standard for exchanging attitude. AEM/ACM rows: EPOCH, Q1, Q2,
  Q3, QC ("QC = cos(phi/2)", scalar last) plus optional Q1_DOT..QC_DOT;
  annex F defines the quaternion-to-matrix relation; ACM TARGET_ATTITUDE
  quaternions default "0 0 0 1". https://public.ccsds.org/Pubs/504x0b2.pdf
- **GMAT (NASA GSFC):** attitude set via Q1..Q4 (scalar last), and GMAT
  reads/writes CCSDS AEM files (CCSDS-AEM attitude model; sample script
  Ex_CCSDSAEMAttitude.script).
  https://raw.githubusercontent.com/nasa/GMAT/main/doc/help/src/Resource_SpacecraftAttitude.xml
  https://github.com/nasa/GMAT/tree/main/application/samples
- **Basilisk (CU AVSLab):** attitude library supports Euler parameters (EP =
  quaternions: C2EP, EP2C, addEP, BmatEP...) and MRPs (C2MRP, MRP2EP,
  mrpFeedback controllers); MRPs favored for control, quaternions fully
  supported. https://raw.githubusercontent.com/AVSLab/basilisk/master/src/architecture/utilities/rigidBodyKinematics.h
- **JWST and HST:** their public NAIF kernel areas hold only spk/ (orbit)
  directories, no public ck/; the JWST pointing docs page contains no
  quaternion mention. Flight-ops quaternion usage unverified (section 5).
  https://naif.jpl.nasa.gov/pub/naif/JWST/kernels/
  https://naif.jpl.nasa.gov/pub/naif/HST/kernels/
  https://jwst-docs.stsci.edu/jwst-observatory-characteristics/jwst-pointing-and-position-angle
- **Gaia (ESA):** not verified today (section 5). **poliastro:** main branch
  has no quaternion or attitude code at all (366 paths checked); purely
  orbital dynamics on vectors/matrices. https://github.com/astrojuanlu/poliastro

### 3.6 Skyfield (vector-native)

- "Skyfield stores the locations of celestial bodies as (x, y, z) vectors
  that it calls positions." No quaternion anywhere in the positions docs;
  frames handled with vector operations.
  https://rhodesmill.org/skyfield/positions.html

### 3.7 Verdict

The quaternion-native niche in astronomy is precisely **spacecraft attitude
engineering**: time-tagged attitude products (SPICE CK, CCSDS AEM/ACM),
flight-dynamics tools (GMAT), simulators/controllers (Basilisk). Sky and frame
astrometry, WCS handling, catalog positions: all **matrix- or vector-native**
(SOFA/ERFA and astropy: rotation matrices; Skyfield: xyz vectors; FITS WCS:
PC/CD matrices, no quaternion keywords). Even inside the spacecraft niche the
quaternion is a storage/composition format converted to a matrix or MRP for
the actual math (q2m_c in SPICE pipelines, EP2C/MRP in Basilisk). And
scalar-first (SPICE/Hamilton) vs scalar-last (GMAT, CCSDS) differs across the
quaternion-native world itself, so any epher quaternion support must document
element order and Hamilton composition explicitly.

---

## 4. What astronomers actually compute with quaternions

1. Read a mission attitude file: load a CK; SPICE interpolates the stored
   quaternion (Hermite/Lagrange per type) and unitizes it; q2m_c yields the
   C-matrix mapping inertial to spacecraft frame. (ck + q2m_c URLs above.)
2. Exchange attitude between systems: produce/consume CCSDS AEM rows (EPOCH,
   Q1, Q2, Q3, QC, optional rates); GMAT ingests them as "CCSDS-AEM"
   attitude. (CCSDS + GMAT URLs above.)
3. Compose successive attitudes cheaply: qxq_c so quaternion products track
   matrix products without building matrices. (rotation.req URL above.)
4. Get rates from attitude histories: qdq2av_c from quaternion + derivative;
   CK type 5 stores derivative packets precisely for this. (URLs above.)
5. Interpolate a pointing timeline: shortest-path slerp with the
   q . q' >= 0 sign fix (Geometric Tools formula; scipy Slerp reference
   implementation); SPICE CK files do the polynomial-then-unitize variant.
6. Slew targeting: ACM TARGET_ATTITUDE quaternions (default 0 0 0 1) specify
   maneuver end states; GMAT Spinner attitude seeded by Q1..Q4.
7. Map a catalog direction into an instrument frame: RA/Dec to unit vector,
   apply attitude as v' = q v q* (or M from q2m_c then matrix multiply), as
   in every SPICE-based pipeline. (q2m_c + Geometric Tools URLs above.)

---

## 5. Unverified (could not be confirmed from a primary source today)

- **Hamilton's original algebra** i^2 = j^2 = k^2 = ijk = -1 from a primary
  text (Hamilton 1844/1866): not fetched; only SPICE's product formula and
  attribution verified.
- **The name "JPL/Shuster convention"** for the scalar-last family: Shuster's
  "A Survey of Attitude Representations" (J. Astronaut. Sci. 41, 439, 1993)
  is the usual citation, but ADS article PDF and NTRS search refused scripted
  access. The underlying convention difference IS verified via SPICE (1.3).
- **"Euler symmetric parameters"** as the synonym used in spacecraft
  literature (Wertz 1978): not verified. Basilisk's "EP" naming IS verified.
- **Shoemake 1985** ("Animating rotation with quaternion curves") as slerp's
  origin: scipy cites a reference but its text did not extract; the formula
  itself is verified from Geometric Tools. **SOFA proper having no quaternion
  routines:** verified for ERFA only; iausofa.org is a JS shell today.
- **JWST and HST flight ops using quaternions:** no public primary doc
  fetched; public NAIF areas for both hold only spk kernels; the JWST
  pointing page never says "quaternion". **Gaia attitude quaternions
  (OAM/AHF/AGIS):** Gaia DR3 documentation chapters fetched (astrometric
  processing, calibration) contain no "quaternion" mention; deeper pages
  unreachable.
- **Left- vs right-multiplication body-rate equations** (q_dot = 1/2 omega q
  vs 1/2 q omega): no primary source fetched states the distinction in
  words; only qdq2av_c behavior is verified. **Normalized LERP vs SLERP
  accuracy claims:** treated as textbook knowledge; no fetched primary
  states it explicitly.
