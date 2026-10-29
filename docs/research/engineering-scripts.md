# Engineering category: sources and verification

The `engineering/` field (2026-10-29): six areas, 55 scripts, every
script carrying a source citation in its header and an expected-output
block checked mechanically by `scripts/check-scripts.mjs`. Every
script's printed values were computed independently before shipping:
a Python reference sheet (`float` arithmetic on the same formulas) was
produced for each area, the script's numbers were compared against it,
and only then were the expected blocks blessed with
`check-scripts.mjs --update`. External anchors were fetched live where
the web allows; the rest cite standard texts whose values the sheets
reproduce.

## Areas and counts

| Area | Scripts | Core text |
|---|---|---|
| structural | 10 | Hibbeler, *Mechanics of Materials*, 11th ed. (Pearson); Beer, Johnston, DeWolf, Mazurek, *Mechanics of Materials*, 8th ed. (Mohr's circle, ch. 7) |
| electrical | 10 | Horowitz and Hill, *The Art of Electronics*, 3rd ed.; Boylestad, *Introductory Circuit Analysis*, 13th ed.; Floyd, *Electronic Devices* |
| thermal | 9 | Incropera, DeWitt, Bergman, Lavine, *Fundamentals of Heat and Mass Transfer*; Cengel and Boles, *Thermodynamics: An Engineering Approach*; ASHRAE Handbook—Fundamentals |
| fluids | 10 | Munson et al., *Fundamentals of Fluid Mechanics*; White, *Fluid Mechanics*; Chow, *Open-Channel Hydraulics*; Wylie and Streeter, *Fluid Transients*; Crane, *Technical Paper 410* |
| mechanical | 10 | Shigley's *Mechanical Engineering Design* (Budynas and Nisbett), 11th ed.; *Machinery's Handbook*; Hibbeler, *Engineering Mechanics: Dynamics* |
| control | 6 | Ogata, *Modern Control Engineering*, 5th ed.; Nise, *Control Systems Engineering*; Franklin, Powell, Emami-Naeini, *Feedback Control of Dynamic Systems*; Ziegler and Nichols (1942) |

## Live-fetched external checks

Values below were confirmed against the cited live pages during
authoring (curl, 2026-10-29), not from memory:

- NIST CODATA 2022 (physics.nist.gov/cuu/Constants/Table/allascii.txt):
  c = 299 792 458 m/s exact, k_B = 1.380 649e-23 J/K exact, R =
  8.314 462 618 J/mol K, sigma = 5.670 374 419e-8 W/m2K4 exact,
  g0 = 9.806 65 exact — all match epher's builtin constants
  (`c`, `k_b`, `r_gas`, `sigma_sb`, `g`) digit for digit. Used by
  `stefan-boltzmann-net.epher` and the thermal set.
- Engineering ToolBox, "Young's Modulus of Elasticity for Metals and
  Alloys" (engineeringtoolbox.com/young-modulus-d_417.html): structural
  steel E = 200 GPa, aluminum 69 GPa — the E values used across the
  structural set.
- Engineering ToolBox, "Beams Supported at Both Ends - Continuous and
  Point Loads" (…/beam-stress-deflection-d_1312.html): the closed-form
  stress and deflection formulas reproduced by
  `beam-center-deflection.epher` (24.5 mm mid-span check).
- Engineering ToolBox, "Resistivity and Conductivity - Temperature
  Coefficients for Common Materials" (…/resistivity-conductivity-
  d_418.html): copper 1.724e-8 ohm m — the wire-run value in
  `ohms-law-voltage-drop.epher`.

## Textbook anchors with published numbers

Classic checks where the formula's value is set by the source, not by
the script's inputs: Mohr's-circle stress invariants (Beer ch. 7:
principal sum equals sigma_x + sigma_y — printed as a check),
Carnot's eta = 1 − T_L/T_H (Cengel ch. 6), LMTD's counterflow-over-
parallel advantage (Incropera ch. 11), the bearing cube law
(Shigley ch. 11: 20 percent less load, 95 percent more life), the
affinity laws (ASHRAE ch. 21), Ziegler–Nichols' original 1942 table
(Trans. ASME 64), and Euler's 63.2 percent/5-tau RC landmarks
(Horowitz and Hill ch. 1). Where a script asserts a self-consistency
check (M/S against M c/I, power in against power out, drag against
weight), the expected block shows it passing.

## Honest limits

- Pressure vessels, bolt groups and channel design in practice follow
  code methods (ASME VIII, AISC J3, eurocodes) on top of these
  classroom formulas; the scripts state the classroom formula and,
  where a design factor is applied (35 percent of yield, 400 MPa
  bolt shear), say so in the header.
- Hazen-Williams applies to water only, in its ordinary temperature
  and velocity ranges; Manning's n is a table value, not a constant
  of nature. Both caveats are printed or commented.
- Temperature in epher is kelvin-only; the thermal scripts convert
  from degrees Celsius by hand (+273.15) and say so.

## House notes

- Area names are fleet-unique (`area_label` shows bare names), so the
  browser labels read "structural", "electrical", and so on without
  the field prefix.
- `delta_t` is a builtin (astronomy Δt), so thermal scripts name the
  knob `temp_rise`; `integral`, `len`, `phase`, `s`, `cd` are likewise
  taken — `lint-scripts.mjs` enforces the builtin list and warns on
  unit-name identifiers.
- The engine prints exact fractions where a decimal would repeat
  (`1333466645/541 mm^4`); that is the display's exact voice, shared
  with the birthday-problem script's `1/221`.
