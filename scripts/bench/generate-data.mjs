#!/usr/bin/env node
// Calculator benchmark data generator (stage 3). Deterministic: mulberry32,
// seed 1, fixed in advance (plan rule 10). The emitted datasets are the
// canonical forms, fed byte-identical to every tool; per-tool function-name
// rendering tables (log -> ln where a tool's log() is not natural log) live
// in each driver, never alter values, and are checked by the referee gate.
import { writeFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const outDir = join(root, "scripts", "bench", "data");
mkdirSync(outDir, { recursive: true });

const N = 10_000;

function mulberry32(seed) {
  let a = seed >>> 0;
  return function () {
    a |= 0; a = (a + 0x6d2b79f5) | 0;
    let t = Math.imul(a ^ (a >>> 15), 1 | a);
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t;
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
  };
}

const rnd = mulberry32(1);
// Three-decimal operands in [0.001, 999.999]; never zero, so division is safe.
const num = () => (Math.floor(rnd() * 999000) / 1000 + 0.001).toFixed(3);

// W1: 10,000 scalar expressions, mixed + - * / ^ with parens. One ^ per
// expression at most, exponent literal 2 or 3 so magnitudes stay bounded.
// Every template parses identically in epher, GNU Octave, and SpeedCrunch.
const w1 = [];
for (let i = 0; i < N; i++) {
  const a = num(), b = num(), c = num(), d = num(), e = num();
  switch (i % 8) {
    case 0: w1.push(`(${a} + ${b}) * (${c} - ${d}) / ${e}`); break;
    case 1: w1.push(`${a} * ${b} + ${c} / ${d} - ${e}`); break;
    case 2: w1.push(`(${a} - ${b}) ^ 2 + ${c} * ${d}`); break;
    case 3: w1.push(`${a} + ${b} * (${c} + ${d} - ${e})`); break;
    case 4: w1.push(`(${a} * ${b} - ${c}) / (${d} + ${e})`); break;
    case 5: w1.push(`${a} ^ 3 - ${b} * ${c} + ${d}`); break;
    case 6: w1.push(`(${a} + ${b} * ${c}) / ${d} - ${e}`); break;
    case 7: w1.push(`(${a} - ${b}) * (${c} ^ 2 + ${d})`); break;
  }
}

// W2: 10,000 transcendental microbench lines cycling sin, cos, tan, exp, log,
// sqrt on fixed arguments. `log(` means natural logarithm (the mathematical
// convention); tools whose `log()` is base-10 or absent render it to their
// natural-log name (declared per driver, gate-checked).
const funcs = ["sin", "cos", "tan", "exp", "log", "sqrt"];
const w2 = [];
for (let i = 0; i < N; i++) {
  const arg = (0.1 + rnd() * 6).toFixed(3);
  w2.push(`${funcs[i % 6]}(${arg})`);
}

writeFileSync(join(outDir, "W1-expressions.txt"), w1.join("\n") + "\n");
writeFileSync(join(outDir, "W2-transcendental.txt"), w2.join("\n") + "\n");
// Deterministic rendering for tools whose log() is not natural log (epher's
// log is base-10; SpeedCrunch has no log at all). Values identical; the
// referee gate checks it.
writeFileSync(join(outDir, "W2-transcendental-ln.txt"), w2.map((l) => l.replace("log(", "ln(")).join("\n") + "\n");
console.log(`W1: ${w1.length} lines; W2: ${w2.length} lines (+ ln-rendered copy); seed 1 (mulberry32)`);
