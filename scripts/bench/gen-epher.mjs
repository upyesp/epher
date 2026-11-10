#!/usr/bin/env node
// Stage 4 epher implementation generator. Emits committed .epher files under
// implementations/epher/. Each file serves as both the timed script and the
// gate input: assignments are silent in epher, bare expressions are outputs,
// so script-verify-all prints exactly the gate values.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { join } from 'node:path';

const BENCH = new URL('./', import.meta.url).pathname;
const DATA = join(BENCH, 'data');
const OUT = join(BENCH, 'implementations', 'epher');
mkdirSync(OUT, { recursive: true });

// ---------- W3 ----------
function w3(n) {
  const A = readFileSync(join(DATA, `W3-A${n}.ephlit`), 'utf8').trim();
  const B = readFileSync(join(DATA, `W3-B${n}.ephlit`), 'utf8').trim();
  const acc = JSON.parse(readFileSync(join(DATA, 'acceptance2.json'), 'utf8'));
  const b = acc[`w3-${n}`].b;
  const blit = '[' + b.map((v) => `[${v}]`).join(', ') + ']';
  const lines = [
    `A = ${A}`,
    `B = ${B}`,
    `bc = ${blit}`,
    `P = A * B`,
    `dA = det(A)`,
    `dB = det(B)`,
    `tr = trace(P)`,
    `X = inv(A) * bc`,
    `sxa = 0`,
    `for i in 1 to ${n} do sxa = sxa + abs(X[i][1])`,
    `R = A * X - bc`,
    `rs = 0`,
    `for i in 1 to ${n} do rs = rs + abs(R[i][1])`,
    `dA`,
    `dB`,
    `tr`,
    `sxa`,
    `rs`,
  ];
  writeFileSync(join(OUT, `w3-${n}.epher`), lines.join('\n') + '\n');
}
w3(100);
w3(500);

// ---------- W4 ----------
{
  const D = readFileSync(join(DATA, 'W4-samples.ephlit'), 'utf8').trim();
  // sample stdev from the population variance builtin: sqrt(var*n/(n-1));
  // percentiles are Type-7 via sort + 1-based index reads (epher's quartile
  // is the Excel median-of-halves method, verified 2026-11-08, so the
  // builtin is not gate-comparable for p10/p90); least squares over indexed
  // reads (native linreg returns text, not gate-comparable).
  const lines = [
    `D = ${D}`,
    `n = 10000`,
    `mu = mean(D)`,
    `sd = sqrt(variance(D) * n / (n - 1))`,
    `md = median(D)`,
    `S = sort(D)`,
    `p10 = S[1000] + 0.9 * (S[1001] - S[1000])`,
    `p90 = S[9000] + 0.1 * (S[9001] - S[9000])`,
    `sxy = 0`,
    `sxx = 0`,
    `sx = 0`,
    `sy = mu * n`,
    `for i in 1 to n do sxx = sxx + (i - 5000.5) * (i - 5000.5)`,
    `for i in 1 to n do sxy = sxy + (i - 5000.5) * (D[i] - mu)`,
    `for i in 1 to n do sx = sx + (i - 5000.5)`,
    `slope = sxy / sxx`,
    `intercept = mu - slope * 5000.5`,
    `syy = variance(D) * n`, // sum((y-mean)^2): population variance * n
    `r = sxy / sqrt(sxx * syy)`,
    `mu`,
    `sd`,
    `md`,
    `p10`,
    `p90`,
    `slope`,
    `intercept`,
    `r`,
  ];
  writeFileSync(join(OUT, 'w4.epher'), lines.join('\n') + '\n');
}

// ---------- W5 ----------
{
  // integrals: epher's native integral (idiomatic). roots: epher's solve is
  // a bare statement with display output and cannot feed a gate value
  // (verified 2026-11-08), so roots are explicit bisections (equivalent
  // track). Loop bodies are single statements, so each while body is one
  // if/else; booleans have no == in epher, so sign agreement is spelled
  // with and/or. Sign is tested against the hi bracket end.
  const r1 = [
    `lo1 = 0`,
    `hi1 = 1`,
    `while hi1 - lo1 > 1e-13 do if (cos((lo1 + hi1) / 2) - (lo1 + hi1) / 2 < 0 and cos(hi1) - hi1 < 0) or (cos((lo1 + hi1) / 2) - (lo1 + hi1) / 2 >= 0 and cos(hi1) - hi1 >= 0) then hi1 = (lo1 + hi1) / 2 else lo1 = (lo1 + hi1) / 2`,
    `(lo1 + hi1) / 2`,
  ];
  const r2 = [
    `lo2 = 1`,
    `hi2 = 2`,
    `while hi2 - lo2 > 1e-13 do if (((lo2 + hi2) / 2)^3 - (lo2 + hi2) / 2 - 1 < 0 and hi2^3 - hi2 - 1 < 0) or (((lo2 + hi2) / 2)^3 - (lo2 + hi2) / 2 - 1 >= 0 and hi2^3 - hi2 - 1 >= 0) then hi2 = (lo2 + hi2) / 2 else lo2 = (lo2 + hi2) / 2`,
    `(lo2 + hi2) / 2`,
  ];
  const r3 = [
    `lo3 = 1`,
    `hi3 = 2`,
    `while hi3 - lo3 > 1e-13 do if (exp((lo3 + hi3) / 2) - 3 * (lo3 + hi3) / 2 < 0 and exp(hi3) - 3 * hi3 < 0) or (exp((lo3 + hi3) / 2) - 3 * (lo3 + hi3) / 2 >= 0 and exp(hi3) - 3 * hi3 >= 0) then hi3 = (lo3 + hi3) / 2 else lo3 = (lo3 + hi3) / 2`,
    `(lo3 + hi3) / 2`,
  ];
  const out = [
    `i1 = integral(exp(-x^2), 0, 1)`,
    `i2 = integral(sin(x) * exp(-x/3), 0, pi)`,
    `i3 = integral(ln(x) / x, 1, 2)`,
    ...r1,
    ...r2,
    ...r3,
    `i1`,
    `i2`,
    `i3`,
  ];
  writeFileSync(join(OUT, 'w5.epher'), out.join('\n') + '\n');
}

// ---------- W6 ----------
{
  const rows = JSON.parse(readFileSync(join(DATA, 'W6-rows.json'), 'utf8'));
  const lines = rows.map(([v, from, to]) => `${v} ${from} in ${to}`);
  writeFileSync(join(OUT, 'w6.epher'), lines.join('\n') + '\n');
}

// ---------- W7 ----------
{
  const lines = readFileSync(join(DATA, 'W7-complex.txt'), 'utf8').trim().split('\n');
  const out = [];
  for (const l of lines) {
    out.push(`re(${l})`);
    out.push(`im(${l})`);
  }
  writeFileSync(join(OUT, 'w7.epher'), out.join('\n') + '\n');
}

// ---------- W8 ----------
{
  const dates = JSON.parse(readFileSync(join(DATA, 'W8-dates.json'), 'utf8'));
  const lunar = dates.map((d) => `round(phase(11, jd(${d.y}, ${d.m}, ${d.d}, 12)) * 100)`);
  writeFileSync(join(OUT, 'w8-lunar.epher'), lunar.join('\n') + '\n');
  const pairs = JSON.parse(readFileSync(join(DATA, 'W8-sunrise.json'), 'utf8'));
  const rise = pairs.map(
    (p) => `round((rise(10, jd(${p.y}, ${p.m}, ${p.d}, 12), ${p.lat}, ${p.lon}) - jd(${p.y}, ${p.m}, ${p.d}, 0)) * 1440)`,
  );
  writeFileSync(join(OUT, 'w8-sunrise.epher'), rise.join('\n') + '\n');
  const jds = JSON.parse(readFileSync(join(DATA, 'W8-jd.json'), 'utf8'));
  const jdl = jds.map((r) => `round(jd(${r.y}, ${r.m}, ${r.d}, ${r.hr}) * 100000)`);
  writeFileSync(join(OUT, 'w8-jd.epher'), jdl.join('\n') + '\n');
}

// ---------- W9 ----------
{
  // W9a: isprime loop (idiomatic epher; the classic sieve needs list index
  // assignment, which the language does not have - ADR-0044 reads only).
  writeFileSync(
    join(OUT, 'w9a.epher'),
    ['s = 0', 'for i in 2 to 9999 do if isprime(i) then s = s + 1', 's'].join('\n') + '\n',
  );
  // W9b: fixed point x = cos(x)
  writeFileSync(
    join(OUT, 'w9b.epher'),
    ['x = 1', 'n = 0', 'while abs(cos(x) - x) > 1e-12 and n < 1000 do x = cos(x)', 'x * 1e12'].join('\n') + '\n',
  );
  // W9c: reverse word order per line + balance count. epher has no
  // char-code builtin, loop bodies are single statements, and for-as-
  // expression is not parseable in argument position, so both helpers are
  // recursive defs. Gate: the joined reversed corpus as one exact string
  // (bench_eval prints STR: values) plus the balanced count.
  const corpus = readFileSync(join(DATA, 'W9c-corpus.ephtxt'), 'utf8').trim();
  const cLines = [
    `corpus = "${corpus}"`,
    `lines = split(corpus, "\\n")`,
    `def rw(s) do`,
    `  p = find(s, " ")`,
    `  if p == 0 then return s`,
    `  return rw(substr(s, p + 1, len(s))) + " " + substr(s, 1, p - 1)`,
    `end`,
    `def baln(s, i, depth) do`,
    `  if i > len(s) and depth == 0 then return 1`,
    `  if i > len(s) then return 0`,
    `  d = depth`,
    `  if substr(s, i, 1) == "(" then d = d + 1`,
    `  if substr(s, i, 1) == ")" then d = d - 1`,
    `  if d < 0 then return 0`,
    `  return baln(s, i + 1, d)`,
    `end`,
    `def rall(ls, i) do`,
    `  if i > 1000 then return ""`,
    `  if i == 1000 then return rw(ls[1000])`,
    `  return rw(ls[i]) + "\\n" + rall(ls, i + 1)`,
    `end`,
    `revall = rall(lines, 1)`,
    `bal = 0`,
    `for i in 1 to 1000 do bal = bal + baln(lines[i], 1, 0)`,
    `revall`,
    `bal`,
  ];
  writeFileSync(join(OUT, 'w9c.epher'), cLines.join('\n') + '\n');
  // W9d: the 100 phase values in a loop, rounded like the referee checksum
  const dates = JSON.parse(readFileSync(join(DATA, 'W8-dates.json'), 'utf8'));
  const lunar = dates.map((d) => `round(phase(11, jd(${d.y}, ${d.m}, ${d.d}, 12)) * 100)`);
  writeFileSync(join(OUT, 'w9d.epher'), lunar.join('\n') + '\n');
  // W9e: branchy accumulator, chunked under the per-statement step budget
  const chunks = [];
  for (let lo = 1; lo <= 50000; lo += 12500) {
    const hi = lo + 12499;
    chunks.push(`for i in ${lo} to ${hi} do if mod(i, 3) == 0 then s = s + i`);
    chunks.push(`for i in ${lo} to ${hi} do if mod(i, 3) != 0 and mod(i, 7) == 0 then s = s - i`);
    chunks.push(`for i in ${lo} to ${hi} do if mod(i, 3) != 0 and mod(i, 7) != 0 then s = s + 1`);
  }
  writeFileSync(
    join(OUT, 'w9e.epher'),
    ['s = 0', ...chunks, 's'].join('\n') + '\n',
  );
}
console.log('epher implementations written');
