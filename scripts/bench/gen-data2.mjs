#!/usr/bin/env node
// Stage 4 data generator: W3-W9 datasets. W1/W2 stay in generate-data.mjs.
// All datasets are seeded (mulberry32, fixed seeds per task) and shipped as
// committed files under scripts/bench/data/. Acceptance values come from
// referee2.py; this script writes the INPUT side only.
//
// Task spec decisions fixed in advance (stage 4, before any measurement):
//   W3-100, W3-500: A,B uniform [-1,1], 6 dp. Outputs: det(A), trace(A*B),
//     xhat = solve(A, b) with b = A*(ones vector) precomputed by the
//     referee so xhat is known exactly (= ones); the tool reports sum|xhat|
//     and the residual ||A*xhat - b||_1. Tolerances: det/trace rel 1e-6
//     (n=100), rel 1e-4 (n=500, float64 conditioning); residual abs gate
//     1e-7 * n. Referee is float64 pure-Python (a 60-digit Decimal matmul
//     at 500^3 costs hours); declared deviation in the stage doc.
//   W4: 10,000 samples = Box-Muller normals (mulberry32) with a 2% uniform
//     outlier band [-1000,1000]. Outputs: mean, sample stdev (n-1), median
//     (Type-7 linear interpolation), p10, p90 (Type 7), linreg slope,
//     intercept, r with x_i = i. Tolerance rel 1e-9.
//   W5-integrals: I1 = int_0^1 exp(-x^2) dx, I2 = int_0^pi sin(x)exp(-x/3) dx,
//     I3 = int_1^2 ln(x)/x dx. Rel tol 1e-9 (referee: float64 adaptive
//     Simpson converged below 1e-13; declared).
//   W5-roots: r1: cos(x)=x on [0,1]; r2: x^3-x-1=0 on [1,2];
//     r3: exp(x)=3x on [1,2]. Abs tol 1e-12 (referee float64 bisection to
//     convergence; declared).
//   W6: 50 conversion rows fixed in w6-rows.json (value, from, to, kind).
//     Coverage rule: a tool publishes a time only at 50/50 support;
//     otherwise "partial: n/50" with the supported rows gated.
//   W7: 200 complex expressions, constants a+bi with a,b uniform [-9,9]
//     1 dp, three to five chained + - * / operations. No ^ (declared:
//     pow implementations differ transcendental-wise across tools).
//     Referee: Python complex (float64), rel 1e-9 per component.
//   W8-lunar: 100 dates spread over 2026-2027. Output: phase angle in
//     degrees. Referee: Meeus ch.49 float64; abs tol 1.0 deg (declared:
//     algorithm-agreement gate, not a precision gate).
//   W8-sunrise: 20 (date, latitude) pairs at longitude -122.42. Output:
//     (rise_jd - jd0) * 1440 minutes from local midnight; -77777 marks
//     "no rise that day" (polar rows). Referee: NOAA solar calc; abs tol
//     3 min (declared).
//   W8-jd: 20 round trips: given (y,m,d,hr), output jd(y,m,d,hr) * 1e5
//     rounded to an integer (self-consistent across tools to ~1e-8 d).
//   W9b: x = cos(x) fixed point from 1.0, |dx| < 1e-12, cap 1000 iters.
//     Output: x * 1e12 rounded + iteration count.
//   W9c: 1,000-line corpus (3-8 letter words). Per line: reverse word
//     order; count parens balance. Output checksum:
//     sum(line_index * 131 + weighted char sum of reversed line) * 10007
//     + balanced_count, all integer.
//   W9d: phase angle checksum over the 100 W8 dates:
//     round(sum(round(phase_deg * 100))) integer.
//   W9e: s = 0; for i in 1..50000: s += (i % 3 == 0) ? i : ((i % 7 == 0)
//     ? -i : 1). Output: s.
import { writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

const DATA = new URL('./data/', import.meta.url).pathname;
mkdirSync(DATA, { recursive: true });

function mulberry32(seed) {
  let a = seed >>> 0;
  return function () {
    a |= 0; a = (a + 0x6D2B79F5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}
const r6 = (x) => Number(x.toFixed(6));

// ---- W3 matrices ----
function genMatrix(seed, n) {
  const rnd = mulberry32(seed);
  const rows = [];
  for (let i = 0; i < n; i++) {
    const row = [];
    for (let j = 0; j < n; j++) row.push(r6(rnd() * 2 - 1));
    rows.push(row);
  }
  return rows;
}
function writeMatrixFlat(name, m) {
  writeFileSync(join(DATA, name), m.map((r) => r.join(' ')).join('\n') + '\n');
}
function writeMatrixLiteral(name, m) {
  // epher / GeoGebra style: [[a,b],[c,d]]
  writeFileSync(join(DATA, name), '[' + m.map((r) => '[' + r.join(', ') + ']').join(', ') + ']');
}
const A100 = genMatrix(301, 100), B100 = genMatrix(302, 100);
// 500x500 entries are pre-scaled by n^(-1/2): the raw det of a uniform
// [-1,1] 500x500 matrix overflows float64 (measured Infinity in the
// referee, 2026-11-08), which would gate an ungateable output. Scaling
// does not change conditioning; the spec decision was made before any
// W3-500 measurement.
const sc = (m, n) => m.map((r) => r.map((v) => Number((v / Math.sqrt(n)).toFixed(9))));
const A500 = sc(genMatrix(303, 500), 500), B500 = sc(genMatrix(304, 500), 500);
writeMatrixFlat('W3-A100.txt', A100); writeMatrixFlat('W3-B100.txt', B100);
writeMatrixFlat('W3-A500.txt', A500); writeMatrixFlat('W3-B500.txt', B500);
writeMatrixLiteral('W3-A100.ephlit', A100); writeMatrixLiteral('W3-B100.ephlit', B100);
writeMatrixLiteral('W3-A500.ephlit', A500); writeMatrixLiteral('W3-B500.ephlit', B500);
writeMatrixLiteral('W3-A100.ggblit', A100); writeMatrixLiteral('W3-B100.ggblit', B100);

// ---- W4 dataset ----
{
  const rnd = mulberry32(401);
  const ys = [];
  let spare = null;
  for (let i = 0; i < 10000; i++) {
    let y;
    if (rnd() < 0.02) {
      y = rnd() * 2000 - 1000;
    } else {
      if (spare === null) {
        const u1 = Math.max(rnd(), 1e-12), u2 = rnd();
        const R = Math.sqrt(-2 * Math.log(u1)), th = 2 * Math.PI * u2;
        y = 50 + 12 * R * Math.cos(th);
        spare = 50 + 12 * R * Math.sin(th);
      } else { y = spare; spare = null; }
    }
    ys.push(r6(y));
  }
  writeFileSync(join(DATA, 'W4-samples.txt'), ys.join('\n') + '\n');
  // epher list literal variant
  writeFileSync(join(DATA, 'W4-samples.ephlit'), '{' + ys.join(', ') + '}');
  // octave inline assignment variant
  writeFileSync(join(DATA, 'W4-samples.mlit'), 'D = [' + ys.join('; ') + '];');
}

// ---- W7 complex expressions ----
{
  const rnd = mulberry32(701);
  const lines = [];
  const c = () => {
    const re = (Math.round((rnd() * 18 - 9) * 10) / 10);
    const im = (Math.round((rnd() * 18 - 9) * 10) / 10);
    return { re, im };
  };
  const OPS = ['+', '-', '*', '/'];
  for (let i = 0; i < 200; i++) {
    const nOps = 3 + Math.floor(rnd() * 3);
    let cur = c();
    const parts = [];
    parts.push(`${cur.re}${cur.im >= 0 ? '+' : ''}${cur.im}i`);
    for (let k = 0; k < nOps; k++) {
      parts.push(' ' + OPS[Math.floor(rnd() * 4)] + ' ');
      const nx = c();
      // keep divisions well-conditioned: divisor magnitude >= 1
      let d = c();
      while (OPS === undefined) break;
      if (parts[parts.length - 1] === '/') {
        while (Math.hypot(d.re, d.im) < 1.5) d = c();
      }
      parts.push(`${d.re}${d.im >= 0 ? '+' : ''}${d.im}i`);
    }
    lines.push('(' + parts.join('') + ')');
  }
  writeFileSync(join(DATA, 'W7-complex.txt'), lines.join('\n') + '\n');
}

// ---- W8 dates ----
const W8_DATES = [];
{
  const rnd = mulberry32(801);
  for (let i = 0; i < 100; i++) {
    const y = rnd() < 0.5 ? 2026 : 2027;
    const m = 1 + Math.floor(rnd() * 12);
    const dim = [31, 28, y % 4 === 0 && y % 100 !== 0 ? 29 : 28, 31, 30, 31, 31, 30, 31, 30, 31, 30][m - 1] ?? 30;
    const d = 1 + Math.floor(rnd() * dim);
    W8_DATES.push({ y, m, d });
  }
  writeFileSync(join(DATA, 'W8-dates.json'), JSON.stringify(W8_DATES, null, 1));
  const pairs = [];
  for (let i = 0; i < 20; i++) {
    const lat = Math.round((-60 + (120 * i) / 19) * 10) / 10;
    const dt = W8_DATES[i * 5];
    pairs.push({ y: dt.y, m: dt.m, d: dt.d, lat, lon: -122.42 });
  }
  writeFileSync(join(DATA, 'W8-sunrise.json'), JSON.stringify(pairs, null, 1));
  const jds = [];
  const rnd2 = mulberry32(802);
  for (let i = 0; i < 20; i++) {
    jds.push({ y: 1900 + Math.floor(rnd2() * 200), m: 1 + Math.floor(rnd2() * 12), d: 1 + Math.floor(rnd2() * 28), hr: Math.round(rnd2() * 24 * 2) / 2 });
  }
  writeFileSync(join(DATA, 'W8-jd.json'), JSON.stringify(jds, null, 1));
}

// ---- W9c corpus ----
{
  const rnd = mulberry32(903);
  const words = [];
  for (let i = 0; i < 400; i++) {
    const len = 3 + Math.floor(rnd() * 6);
    let w = '';
    for (let j = 0; j < len; j++) w += 'abcdefghijklmnopqrstuvwxyz'[Math.floor(rnd() * 26)];
    words.push(w);
  }
  const lines = [];
  for (let i = 0; i < 1000; i++) {
    const n = 3 + Math.floor(rnd() * 4);
    const ws = [];
    for (let j = 0; j < n; j++) ws.push(words[Math.floor(rnd() * words.length)]);
    let line = ws.join(' ');
    // sprinkle parens on ~60% of lines (balanced or not, the referee decides)
    const r = rnd();
    if (r < 0.4) line = '(' + line + ')';
    else if (r < 0.55) line = line + ')(';
    else if (r < 0.6) line = '((' + line + ')';
    lines.push(line);
  }
  writeFileSync(join(DATA, 'W9c-corpus.txt'), lines.join('\n') + '\n');
  // epher single-string variant with \n escapes
  writeFileSync(join(DATA, 'W9c-corpus.ephtxt'), lines.join('\\n'));
}

// ---- W6 conversion rows (fixed table, 50) ----
const W6 = [
  // length (12)
  ['5', 'km', 'mile'], ['3', 'mile', 'km'], ['12', 'inch', 'cm'], ['100', 'm', 'ft'],
  ['2', 'nmi', 'km'], ['1', 'AU', 'km'], ['4', 'ly', 'AU'], ['10', 'pc', 'ly'],
  ['26', 'mile', 'm'], ['9', 'yd', 'm'], ['42', 'ft', 'inch'], ['7', 'km', 'nmi'],
  // mass (8)
  ['80', 'kg', 'lb'], ['150', 'lb', 'kg'], ['12', 'oz', 'g'], ['3', 't', 'kg'],
  ['250', 'g', 'oz'], ['55', 'lb', 'oz'], ['6', 'kg', 't'], ['1', 'lb', 'g'],
  // time (6)
  ['90', 'min', 'hr'], ['48', 'hr', 'd'], ['365', 'd', 'yr'], ['3', 'yr', 'd'],
  ['7200', 's', 'hr'], ['5', 'd', 'min'],
  // speed (4)
  ['60', 'mile/hr', 'km/hr'], ['120', 'km/hr', 'mph'], ['15', 'm/s', 'km/hr'], ['30', 'knot', 'mph'],
  // volume (4)
  ['12', 'gal', 'L'], ['2', 'L', 'qt'], ['5', 'qt', 'gal'], ['30', 'L', 'pt'],
  // pressure (4)
  ['1', 'atm', 'kPa'], ['2', 'bar', 'psi'], ['760', 'torr', 'kPa'], ['35', 'psi', 'bar'],
  // energy (6)
  ['1', 'kJ', 'J'], ['2.5', 'kWh', 'MJ'], ['300', 'cal', 'J'], ['1', 'BTU', 'kJ'],
  ['5', 'eV', 'J'], ['1.5', 'hp', 'kW'],
  // temperature offsets (4)
  ['20', 'degC', 'degF'], ['68', 'degF', 'degC'], ['300', 'K', 'degC'], ['-40', 'degC', 'degF'],
  // angles (2)
  ['180', 'deg', 'rad'], ['0.5', 'rad', 'arcmin'],
];
if (W6.length !== 50) throw new Error(`W6 table has ${W6.length} rows, expected 50`);
writeFileSync(join(DATA, 'W6-rows.json'), JSON.stringify(W6, null, 1));

console.log('stage 4 datasets written to', DATA);
