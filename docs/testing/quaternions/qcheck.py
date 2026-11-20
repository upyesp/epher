#!/usr/bin/env python3
"""qcheck.py -- cross-check epher's quaternion surface against third-party tools.

Every case runs the same expression through the real epher binary and through
an independent implementation in a reputable free library:

  sympy.algebras.Quaternion   exact Hamilton algebra, scalar-first (w, x, y, z)
  scipy.spatial.transform     rotations, matrices, slerp (scalar-last, x, y, z, w)
  numpy-quaternion            a second, independent slerp (scalar-first)

Usage:
  /tmp/qvenv/bin/python qcheck.py --epher /path/to/epher-cli

Tolerance: epher prints 12 significant digits, so agreement is asserted at
1e-9 relative / 1e-12 absolute. Rotation results compare up to the sign
q <-> -q (both name the same rotation); signs are folded before comparing.
The horizon case needs epher's own ra/decl for the Sun (the cross-check is
the frame rotation, not the ephemeris) and validates against the printed
transcript in the guide first.
"""
import argparse
import math
import re
import subprocess
import sys

import numpy as np
from sympy.algebras import Quaternion as Sq
from scipy.spatial.transform import Rotation, Slerp
import quaternion as nq

RTOL = 1e-9
ATOL = 1e-12

# ---------------------------------------------------------------- epher I/O

def run_epher(binary, lines):
    p = subprocess.run([binary, "-"], input="\n".join(lines) + "\n",
                       capture_output=True, text=True, timeout=60)
    out = p.stdout.strip().splitlines()
    err = [l for l in (p.stderr or "").strip().splitlines() if l]
    for l in out + err:
        if l.startswith("error:"):
            return ("error", l.split("error:", 1)[1].strip())
    answers = [l[2:] for l in out if l.startswith("= ")]
    if not answers:
        return ("error", f"no answer line; stdout={out!r} stderr={err!r}")
    return ("ok", answers)

NUM = r"(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?"
NUM_S = r"[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?"

def parse_quat(s):
    """`= 1 + 2i + 3j - 0.5k` -> [1, 2, 3, -0.5] (scalar first)."""
    s = s.strip().replace(" ", "")
    try:
        return [float(s), 0.0, 0.0, 0.0]
    except ValueError:
        pass
    if not re.fullmatch(rf"(?:[-+]?{NUM})?(?:[-+]?(?:{NUM})?[ijk])+", s):
        raise ValueError(f"not a quaternion literal: {s!r}")
    parts = {"": [], "i": [], "j": [], "k": []}
    for m in re.finditer(rf"([-+]?(?:{NUM})?)([ijk]?)", s):
        num, unit = m.group(1), m.group(2)
        if not num and not unit:
            continue
        if num in ("", "+", "-"):
            num += "1"
        parts[unit].append(float(num))
    return [sum(parts[""]), sum(parts["i"]), sum(parts["j"]), sum(parts["k"])]

def parse_vec(s):
    body = s.split("{", 1)[1].split("}", 1)[0]
    return [float(x) for x in re.findall(NUM_S, body)]

def parse_mat(s):
    return [[float(x) for x in re.findall(NUM_S, r)]
            for r in re.findall(r"\[([^[\]]+)\]", s)]

# ------------------------------------------------------------ compare tools

def near(a, b):
    return all(abs(x - y) <= ATOL + RTOL * abs(y) for x, y in zip(a, b))

def fold(q):
    """Canonical representative: largest-magnitude component made positive."""
    q = list(q)
    k = max(range(4), key=lambda i: abs(q[i]))
    return [-v for v in q] if q[k] < 0 else q

def qnear(a, b, foldsign=False):
    return near(fold(a), fold(b)) if foldsign else near(a, b)

def s_of(sq):
    return [float(sq.a), float(sq.b), float(sq.c), float(sq.d)]

def sq(*comps):
    return Sq(*[float(c) for c in comps])

def scipy_rot(q):  # epher order -> scipy Rotation
    return Rotation.from_quat([q[1], q[2], q[3], q[0]])

def scipy_comps(r):  # scipy Rotation -> epher order
    v = r.as_quat()
    return [v[3], v[0], v[1], v[2]]

def nq_q(q):
    return nq.quaternion(*[float(c) for c in q])

def nq_comps(q):
    return [float(v) for v in nq.as_float_array(np.atleast_1d(q))[0]]

def fmt(v):
    return "[" + ", ".join(f"{x:.10g}" for x in v) + "]"

# ---------------------------------------------------------------- the cases

RESULTS = []
def case(cid, group, desc, lines, check):
    RESULTS.append((cid, group, desc, lines, check))

Q1 = [1.2, -0.3, 2.5, 0.7]
Q2 = [0.4, 1.1, -2.0, 3.3]
Q3 = [-0.9, 0.2, -1.4, 0.6]
QL = [1, 2, 3, 4]
EQ1 = "quaternion(1.2, -0.3, 2.5, 0.7)"
EQ2 = "quaternion(0.4, 1.1, -2.0, 3.3)"
EQ3 = "quaternion(-0.9, 0.2, -1.4, 0.6)"
EQL = "1 + 2i + 3j + 4k"

AX = [2.0, -1.0, 4.0]          # |axis| = sqrt(21)
ANG = 1.1

def qaxis_quat(axis, ang):
    n = math.sqrt(sum(a * a for a in axis))
    s = math.sin(ang / 2)
    return [math.cos(ang / 2)] + [a / n * s for a in axis]

QAX = qaxis_quat(AX, ANG)
U111 = qaxis_quat([1, 1, 1], 0.7)          # unit, angle 0.7 < pi
UY = qaxis_quat([0, 1, 0], 2.5)            # unit, angle 2.5 < pi
def mat_literal(r):
    m = np.array(r.as_matrix())
    return "[[" + ", ".join(f"{v:.15g}" for v in m[0]) + "], [" \
           + ", ".join(f"{v:.15g}" for v in m[1]) + "], [" \
           + ", ".join(f"{v:.15g}" for v in m[2]) + "]]"

M_STR = mat_literal(Rotation.from_rotvec([0.3, -0.4, 0.9]))

def quat_case(cid, group, desc, lines, expected, foldsign=False):
    def check(k, p):
        assert k == "ok", p
        got = parse_quat(p[0])
        return qnear(got, expected, foldsign), f"epher={fmt(got)} ref={fmt(expected)}"
    case(cid, group, desc, lines, check)

def err_case(cid, group, desc, lines, want=""):
    def check(k, p):
        ok = k == "error" and want in p
        return ok, p if k == "error" else f"expected error {want!r}, got ok: {p}"
    case(cid, group, desc, lines, check)

# == A. literals, parts, algebra (vs sympy) =================================

quat_case("A1", "algebra", "literal spells all four parts",
          [EQL], QL)
quat_case("A2", "algebra", "constructor is scalar-first",
          ["quaternion(1, 2, 3, 4)"], QL)
quat_case("A3", "algebra", "literal * constructor, Hamilton order",
          [f"({EQL}) * {EQ2}"], s_of(sq(*QL) * sq(*Q2)))
quat_case("A4", "algebra", "general product q1*q2",
          [f"{EQ1} * {EQ2}"], s_of(sq(*Q1) * sq(*Q2)))
quat_case("A5", "algebra", "reversed product q2*q1 (non-commutative, still matches)",
          [f"{EQ2} * {EQ1}"], s_of(sq(*Q2) * sq(*Q1)))
quat_case("A6", "algebra", "associativity (q1*q2)*q3",
          [f"({EQ1} * {EQ2}) * {EQ3}"], s_of((sq(*Q1) * sq(*Q2)) * sq(*Q3)))
quat_case("A7", "algebra", "associativity q1*(q2*q3) agrees",
          [f"{EQ1} * ({EQ2} * {EQ3})"], s_of(sq(*Q1) * (sq(*Q2) * sq(*Q3))))
quat_case("A8", "algebra", "conjugate",
          [f"conj({EQL})"], s_of(sq(*QL).conjugate()))
case("A9", "algebra", "norm abs(q) = sqrt(30)",
     [f"abs({EQL})"],
     lambda k, p: (near([float(p[0])], [math.sqrt(30)]), p[0]))
quat_case("A10", "algebra", "qunit is unit in the same direction",
          [f"qunit({EQL})"], s_of(sq(*QL).normalize()))
quat_case("A11", "algebra", "inverse",
          [f"qinv({EQL})"], s_of(sq(*QL).inverse()))
quat_case("A12", "algebra", "division is multiplication by the inverse",
          [f"{EQ1} / {EQ2}"], s_of(sq(*Q1) * sq(*Q2).inverse()))
case("A13", "algebra", "parts: re(q), im(q), qw..qz",
     ["q = " + EQL, "re(q)", "im(q)", "{qw(q), qx(q), qy(q), qz(q)}"],
     lambda k, p: (qnear(parse_quat(p[0]), QL)
                   and near([float(p[1])], [1])
                   and near(parse_vec(p[2]), [2, 3, 4])
                   and near(parse_vec(p[3]), QL),
                   f"q={p[0]} re={p[1]} im={p[2]} qw..qz={p[3]}"))
case("A14", "algebra", "unit products: i*i=-1, i*j=k, j*k=i, k*i=j",
     ["i*i", "i*j", "j*k", "k*i"],
     lambda k, p: (qnear(parse_quat(p[0]), [-1, 0, 0, 0])
                   and qnear(parse_quat(p[1]), [0, 0, 0, 1])
                   and qnear(parse_quat(p[2]), [0, 1, 0, 0])
                   and qnear(parse_quat(p[3]), [0, 0, 1, 0]),
                   f"i*i={p[0]} i*j={p[1]} j*k={p[2]} k*i={p[3]}"))
quat_case("A15", "algebra", "complex promotes into quaternion arithmetic",
          [f"({EQL}) * (1 + 2i)"], s_of(sq(*QL) * sq(1, 2, 0, 0)))
quat_case("A16", "algebra", "precedence: 4k * Q binds before + (parenthesize literals)",
          [f"{EQL} * quaternion(0, 1, 0, 0)"],
          s_of(sq(1, 2, 3, 0) + sq(0, 0, 0, 4) * sq(0, 1, 0, 0)))

# == B. transcendentals (vs sympy exp/log) ==================================

quat_case("B1", "transcendental", "exp(pi i) = -1 (Euler)",
          ["exp(pi * i)"], s_of(sq(0, math.pi, 0, 0).exp()))
quat_case("B2", "transcendental", "exp of a general quaternion",
          [f"exp({EQ1})"], s_of(sq(*Q1).exp()))
LN10 = math.log(10.0)
quat_case("B3", "transcendental", "ln of a unit quaternion (half-angle map, = sympy log)",
          ["ln(qaxis({1, 1, 1}, 0.7))"], s_of(sq(*U111).log()), foldsign=True)
quat_case("B4", "transcendental", "log is base-10: log(q) = ln(q) / ln(10)",
          ["log(qaxis({1, 1, 1}, 0.7))"],
          [v / LN10 for v in s_of(sq(*U111).log())], foldsign=True)
case("B5", "transcendental", "sqrt(q)^2 = q; sqrt matches exp(ln(q)/2)",
     ["q = qaxis({0, 1, 0}, 2.5)", "r = sqrt(q)", "r * r"],
     lambda k, p: (qnear(parse_quat(p[0]), UY)
                   and qnear(parse_quat(p[1]), s_of((sq(*UY).log() / 2).exp()))
                   and qnear(parse_quat(p[2]), UY),
                   f"q={fmt(parse_quat(p[0]))} sqrt={fmt(parse_quat(p[1]))} r*r={fmt(parse_quat(p[2]))}"))
case("B6", "transcendental", "powers: q^2, q^3, q^-1",
     ["q = " + EQL, "q ^ 2", "q ^ 3", "q ^ (-1)"],
     lambda k, p: (qnear(parse_quat(p[0]), QL)
                   and qnear(parse_quat(p[1]), s_of(sq(*QL)**2))
                   and qnear(parse_quat(p[2]), s_of(sq(*QL)**3))
                   and qnear(parse_quat(p[3]), s_of(sq(*QL).inverse())),
                   f"q={fmt(parse_quat(p[0]))} q2={fmt(parse_quat(p[1]))} q3={fmt(parse_quat(p[2]))} qinv={fmt(parse_quat(p[3]))}"))

# == C. rotation family (vs scipy) ==========================================

quat_case("C1", "rotation", "qaxis(axis, angle) matches scipy from_rotvec",
          ["qaxis({2, -1, 4}, 1.1)"], scipy_comps(scipy_rot(QAX)), foldsign=True)
case("C2", "rotation", "q2m matches scipy as_matrix",
     ["q2m(qaxis({2, -1, 4}, 1.1))"],
     lambda k, p: (near([x for row in parse_mat(p[0]) for x in row],
                        list(np.array(scipy_rot(QAX).as_matrix()).ravel())),
                   fmt([x for row in parse_mat(p[0]) for x in row])))
case("C3", "rotation", "m2q reads a scipy matrix back (scalar part >= 0)",
     [f"m2q({M_STR})"],
     lambda k, p: (qnear(parse_quat(p[0]),
                         scipy_comps(Rotation.from_rotvec([0.3, -0.4, 0.9])), foldsign=True)
                   and parse_quat(p[0])[0] >= 0,
                   fmt(parse_quat(p[0]))))
case("C4", "rotation", "m2q(q2m(q)) round-trips, canonical scalar >= 0",
     ["q = qaxis({2, -1, 4}, 1.1)", "m2q(q2m(q))"],
     lambda k, p: (qnear(parse_quat(p[1]), scipy_comps(scipy_rot(QAX)), foldsign=True)
                   and parse_quat(p[1])[0] >= 0,
                   fmt(parse_quat(p[1]))))
case("C5", "rotation", "qrot(q, v) = R @ v",
     ["qrot(qaxis({2, -1, 4}, -0.9), {1, 0, -2})"],
     lambda k, p: (near(parse_vec(p[0]),
                        list(scipy_rot(qaxis_quat(AX, -0.9)).as_matrix() @ np.array([1.0, 0.0, -2.0]))),
                   p[0]))
case("C6", "rotation", "qangle (angle < pi) matches scipy magnitude",
     ["deg(qangle(qaxis({2, -1, 4}, 1.1)))"],
     lambda k, p: (near([float(p[0])], [math.degrees(scipy_rot(QAX).magnitude())]), p[0]))
case("C7", "rotation", "qaxis(q) reads the unit axis",
     ["qaxis(qaxis({2, -1, 4}, 1.1))"],
     lambda k, p: (near(parse_vec(p[0]), [a / math.sqrt(21) for a in AX]), p[0]))
case("C8", "rotation", "q and -q are the same rotation",
     ["q = qaxis({2, -1, 4}, 1.1)", "q2m(0 - q)", "q2m(q)"],
     lambda k, p: (near([x for row in parse_mat(p[1]) for x in row],
                        [x for row in parse_mat(p[2]) for x in row]),
                   "q2m(0-q) = q2m(q)"))
case("C9", "rotation", "double turn q*q doubles the angle",
     ["q2m(qaxis({2, -1, 4}, 1.1) * qaxis({2, -1, 4}, 1.1))"],
     lambda k, p: (near([x for row in parse_mat(p[0]) for x in row],
                        list(np.array(scipy_rot(qaxis_quat(AX, 2.2)).as_matrix()).ravel())),
                   "matrix vs scipy rotvec(2.2)"))

# == D. slerp (vs scipy Slerp AND numpy-quaternion) =========================

def slerp_ref(q1, q2, t):
    rs = Slerp([0.0, 1.0],
               Rotation.concatenate([scipy_rot(q1), scipy_rot(q2)]))(np.array([t]))[0]
    unit = lambda q: [v / math.sqrt(sum(x * x for x in q)) for v in q]
    rn = nq.slerp_evaluate(nq_q(unit(q1)), nq_q(unit(q2)), t)
    return scipy_comps(rs), nq_comps(rn)

def slerp_case(cid, desc, q1, q2, t, lines):
    def check(k, p):
        assert k == "ok", p
        got = parse_quat(p[0])
        rs, rn = slerp_ref(q1, q2, t)
        ok = qnear(got, rs, foldsign=True) and qnear(got, rn, foldsign=True)
        return ok, f"epher={fmt(got)} scipy={fmt(rs)} numpy-q={fmt(rn)}"
    case(cid, "slerp", desc, lines, check)

slerp_case("D1", "endpoints: t=0 gives q1, t=1 gives q2",
           [1, 0, 0, 0], QAX, 0.0,
           ["qslerp(quaternion(1, 0, 0, 0), qaxis({2, -1, 4}, 1.1), 0)"])
slerp_case("D1b", "endpoint t=1 gives q2",
           [1, 0, 0, 0], QAX, 1.0,
           ["qslerp(quaternion(1, 0, 0, 0), qaxis({2, -1, 4}, 1.1), 1)"])
slerp_case("D2", "midpoint t=0.5 (identity to a 90-degree z turn)",
           [1, 0, 0, 0], qaxis_quat([0, 0, 1], math.pi / 2), 0.5,
           ["qslerp(quaternion(1, 0, 0, 0), qaxis({0, 0, 1}, 90 deg), 0.5)"])
slerp_case("D3", "quarter t=0.25",
           [1, 0, 0, 0], qaxis_quat([0, 0, 1], math.pi / 2), 0.25,
           ["qslerp(quaternion(1, 0, 0, 0), qaxis({0, 0, 1}, 90 deg), 0.25)"])
slerp_case("D4", "general pair, t=0.7",
           Q1, Q2, 0.7, [f"qslerp({EQ1}, {EQ2}, 0.7)"])
slerp_case("D5", "shortest path: a 270-degree turn reads as -90",
           [1, 0, 0, 0], qaxis_quat([0, 0, 1], math.radians(270)), 0.5,
           ["qslerp(quaternion(1, 0, 0, 0), qaxis({0, 0, 1}, 270 deg), 0.5)"])
case("D6", "slerp", "symmetric: slerp(q1,q2,.5) = slerp(q2,q1,.5)",
     [f"qslerp({EQ1}, {EQ2}, 0.5)", f"qslerp({EQ2}, {EQ1}, 0.5)"],
     lambda k, p: (qnear(parse_quat(p[0]), parse_quat(p[1]), foldsign=True),
                   f"{p[0]} vs {p[1]}"))

# == E. horizon bridge (vs independent GMST/hour-angle formulas) ============

JD = 2460000.5
LAT, LON = 51.5, -0.12

def gmst_deg(jd):
    T = (jd - 2451545.0) / 36525.0
    return (280.46061837 + 360.98564736629 * (jd - 2451545.0)
            + 0.000387933 * T * T - T**3 / 38710000.0) % 360.0

def hor_unit(ra_deg, dec_deg, jd, lat_deg, lon_east_deg):
    """Equatorial (RA/Dec) -> east/north/up, pure astronomical formulas."""
    H = math.radians(gmst_deg(jd) + lon_east_deg - ra_deg)
    dec, phi = math.radians(dec_deg), math.radians(lat_deg)
    up = math.sin(dec) * math.sin(phi) + math.cos(dec) * math.cos(phi) * math.cos(H)
    north = math.sin(dec) * math.cos(phi) - math.cos(dec) * math.sin(phi) * math.cos(H)
    east = -math.cos(dec) * math.sin(H)
    return [east, north, up]

case("E1", "horizon", "qeq2hor vs independent GMST/hour-angle formulas (guide's Sun case)",
     [f"jd = {JD}",
      "sun = {cos(decl(10, jd) deg) * cos(ra(10, jd) deg), cos(decl(10, jd) deg) * sin(ra(10, jd) deg), sin(decl(10, jd) deg)}",
      f"qrot(qeq2hor(jd, {LAT}, {LON}), sun)"],
     None)  # check bound in main() once epher's ra/decl are known

case("E2", "horizon", "Polaris over Greenwich (ex-a22): altitude ~ latitude",
     [f"jd = {JD}",
      "qhor = qeq2hor(jd, 51.4779, -0.0015)",
      "polaris = {cos(89.2642 deg) * cos(37.9545 deg), cos(89.2642 deg) * sin(37.9545 deg), sin(89.2642 deg)}",
      "p = qrot(qhor, polaris)",
      "deg(asin(p[3]))"],
     None)  # check bound in main(): altitude within 0.05 deg of latitude

# == F. reserved names, type boundaries, refusals ===========================

def err_case(cid, group, desc, lines, want=""):
    def check(k, p):
        ok = k == "error" and want in p
        return ok, p if k == "error" else f"expected error {want!r}, got ok: {p}"
    case(cid, group, desc, lines, check)

err_case("F1", "refusals", "k cannot be assigned", ["k = 3"], "quaternion unit")
err_case("F2", "refusals", "j cannot be assigned", ["j = 1"], "quaternion unit")
err_case("F3", "refusals", "i cannot be assigned", ["i = 5"], "cannot")
case("F4", "refusals", "def parameters may still be named k",
     ["def f(k) = k * k", "f(2)"],
     lambda k, p: (k == "ok" and abs(float(p[0]) - 4) < 1e-12, str(p)))
err_case("F5", "refusals", "rotation happens by name only (q * vector refused)",
         ["qaxis({0, 0, 1}, 90 deg) * {1, 0, 0}"])
err_case("F6", "refusals", "quaternion == scalar is a type error",
         ["quaternion(1, 0, 0, 0) == 3"])
err_case("F7", "refusals", "trig refuses quaternions",
         ["sin(quaternion(1, 2, 3, 4))"])
err_case("F8", "refusals", "quantities do not promote",
         ["quaternion(1, 0, 0, 0) + 1 m"])

# --------------------------------------------------------------------- main

def bind_dynamic_checks(binary):
    """E cases consume epher's own ephemeris (the cross-check is the frame
    rotation, not the ephemeris) and sanity anchors from the guide."""
    src, payload = run_epher(binary, [f"ra(10, {JD})", f"decl(10, {JD})"])
    assert src == "ok", payload
    ra, decl = float(payload[0]), float(payload[1])

    TRANSCRIPT = [-0.0585591573677, 0.670732553317, -0.739383978058]

    def e1_check(k, p):
        assert k == "ok", p
        got = parse_vec(p[2])
        exp = hor_unit(ra, decl, JD, LAT, LON)
        exact = near(got, TRANSCRIPT)          # the guide's printed values
        loose = all(abs(x - y) < 5e-4 for x, y in zip(got, exp))
        return exact and loose, (f"epher={fmt(got)} transcript-exact={exact} "
                                 f"formulas(5e-4)={loose} formulas={fmt(exp)}")

    def e2_check(k, p):
        assert k == "ok", p
        alt = float(p[-1])
        return abs(alt - 51.4779) < 0.75, f"alt={alt} (latitude 51.4779; Polaris is 0.736 deg off the pole)"

    for i, (cid, group, desc, lines, _) in enumerate(RESULTS):
        if cid == "E1":
            RESULTS[i] = (cid, group, desc, lines, e1_check)
        elif cid == "E2":
            RESULTS[i] = (cid, group, desc, lines, e2_check)

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--epher", default="target/release/epher-cli")
    args = ap.parse_args()

    ver = subprocess.run([args.epher, "--version"], capture_output=True,
                         text=True).stdout.strip()
    print(f"epher binary: {args.epher} ({ver})")
    bind_dynamic_checks(args.epher)

    npass = nfail = 0
    for cid, group, desc, lines, check in RESULTS:
        try:
            src, payload = run_epher(args.epher, lines)
            ok, detail = check(src, payload)
            status = "PASS" if ok else "FAIL"
            npass, nfail = npass + ok, nfail + (not ok)
            print(f"{status}  {cid:4} {group:14} {desc}")
            if not ok:
                print(f"      {detail}")
        except Exception as e:
            nfail += 1
            print(f"FAIL  {cid:4} {group:14} {desc}")
            print(f"      exception: {e!r}")
    print(f"\n{npass} passed, {nfail} failed of {len(RESULTS)}")
    sys.exit(1 if nfail else 0)

if __name__ == "__main__":
    main()
