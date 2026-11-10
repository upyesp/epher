// GNU Octave driver (stage 3 pilot). In-engine boundary: tic/toc inside the
// implementation file covers parse + evaluation of the block; process spawn
// is excluded (declared). Verify first, then warm-up 3 + bucket samples.
import { execFileSync, spawnSync } from "node:child_process";
import { readFileSync, writeFileSync, unlinkSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { summarize, bucketSamples } from "../stats.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const implDir = join(root, "scripts", "bench", "implementations", "octave");

function octaveVersion() {
  const out = execFileSync("/opt/homebrew/bin/octave-cli", ["--version"], { encoding: "utf8" });
  return out.split("\n")[0].trim();
}

function runFile(mFile, verifyOut) {
  const env = { ...process.env };
  if (verifyOut) env.BENCH_VERIFY_OUT = verifyOut;
  const timerExpr = verifyOut
    ? `run('${mFile}')`
    : `t0 = tic; run('${mFile}'); __elapsed = toc(t0); printf('BENCH_ELAPSED_S %.6f\\n', __elapsed);`;
  const res = spawnSync("/opt/homebrew/bin/octave-cli", ["--quiet", "--eval", timerExpr], {
    encoding: "utf8",
    env,
    timeout: 900_000,
    maxBuffer: 64 * 1024 * 1024,
  });
  const text = res.stdout || "";
  const elapsed = text.match(/BENCH_ELAPSED_S ([0-9.]+)/);
  const count = text.match(/W9A_COUNT (\d+)/);
  const written = text.match(/VERIFY_WRITTEN (\d+)/);
  return {
    elapsed_s: elapsed ? Number(elapsed[1]) : null,
    count: count ? Number(count[1]) : null,
    written: written ? Number(written[1]) : null,
    stderr: (res.stderr || "").slice(0, 500),
  };
}

function relOk(toolValue, refereeValue, tol = 1e-9) {
  const t = Number(toolValue);
  const r = Number(refereeValue);
  if (!Number.isFinite(t) || !Number.isFinite(r)) return false;
  if (r === 0) return Math.abs(t) < tol;
  return Math.abs(t - r) / Math.abs(r) <= tol;
}

async function verifyBatch(taskKey, dataFile, acceptance) {
  const verifyOut = `/tmp/bench-octave-verify-${taskKey}.txt`;
  if (existsSync(verifyOut)) unlinkSync(verifyOut);
  const res = runFile(join(implDir, `${taskKey}-verify.m`), verifyOut);
  if (!res.written || !existsSync(verifyOut)) {
    return { checked: 0, failed: 1, note: `verification run failed: ${res.stderr}` };
  }
  const values = readFileSync(verifyOut, "utf8").trim().split("\n");
  const ref = acceptance[taskKey];
  let failed = 0;
  for (let i = 0; i < ref.length; i++) {
    if (!relOk(values[i], ref[i])) failed++;
  }
  return { checked: ref.length, failed, note: `values=${values.length}` };
}

const W3 = (n) => ({
  file: `w3-${n}`,
  tracks: ["native", "equivalent"],
  values: 5,
  gate: (vals, acc) => {
    const a = acc[`w3-${n}`];
    // det gates compare magnitude only: the sign flips under float64
    // pivoting near-ties between independent LU implementations (observed
    // between the referee and the explicit-loop Octave track, 2026-11-08)
    const spec = [
      [Math.abs(vals[0]), Math.abs(a.detA), a.det_rel_tol],
      [Math.abs(vals[1]), Math.abs(a.detB), a.det_rel_tol],
      [vals[2], a.traceAB, a.traceAB_rel_tol],
      [vals[3], a.xhat_expected_sum_abs, a.xhat_sum_abs_rel_tol],
    ];
    let bad = 0;
    for (const [tool, ref, tol] of spec) {
      if (!(Math.abs(tool - ref) / Math.abs(ref) <= tol)) bad++;
    }
    // residual: absolute gate
    if (!(vals[4] <= a.resid_l1_gate)) bad++;
    return { checked: 5, failed: bad };
  },
});

const GATES = {
  "w1": { file: "w1-batch", verifyFile: "w1-verify", tracks: ["single"], values: 10000, gate: flatGate("w1", 1e-9) },
  "w2": { file: "w2-batch", verifyFile: "w2-verify", tracks: ["single"], values: 10000, gate: flatGate("w2", 1e-9) },
  "w3-100": W3(100),
  "w3-500": W3(500),
  "w4": { file: "w4", tracks: ["native", "equivalent"], values: 8, gate: arrGate(8, (acc) => [acc.w4.mean, acc.w4.stdev, acc.w4.median, acc.w4.p10, acc.w4.p90, acc.w4.lin_slope, acc.w4.lin_intercept, acc.w4.lin_r]) },
  "w5": { file: "w5", tracks: ["native", "equivalent"], values: 6, gate: (vals, acc) => {
    let bad = 0;
    const I = [acc.w5.integrals.I1, acc.w5.integrals.I2, acc.w5.integrals.I3];
    for (let i = 0; i < 3; i++) if (!(Math.abs(vals[i] - I[i]) / Math.abs(I[i]) <= 1e-9)) bad++;
    const R = [acc.w5.roots.r1, acc.w5.roots.r2, acc.w5.roots.r3];
    for (let i = 0; i < 3; i++) if (!(Math.abs(vals[3 + i] - R[i]) <= 1e-12)) bad++;
    return { checked: 6, failed: bad };
  } },
  "w7": { file: "w7", tracks: ["single"], values: 400, gate: (vals, acc) => {
    const flat = [];
    for (const pair of acc.w7.values) flat.push(pair[0], pair[1]);
    let bad = 0;
    for (let i = 0; i < flat.length; i++) {
      const ref = flat[i];
      if (ref === 0 ? Math.abs(vals[i]) > 1e-9 : !(Math.abs(vals[i] - ref) / Math.abs(ref) <= 1e-9)) bad++;
    }
    return { checked: flat.length, failed: bad };
  } },
  "w8-lunar": { file: "w8-lunar", tracks: ["single"], values: 100, gate: intGate("w8-lunar", 100, 100.0) },
  "w8-sunrise": { file: "w8-sunrise", tracks: ["single"], values: 20, gate: intGate("w8-sunrise", 20, 3.0) },
  "w8-jd": { file: "w8-jd", tracks: ["single"], values: 20, gate: intGate("w8-jd", 20, 10.0) },
  "w9a": { file: "w9a", tracks: ["idiomatic", "equivalent"], special: "w9a" },
  "w9b": { file: "w9b", tracks: ["single"], values: 2, gate: (vals, acc) => {
    const bad = !(Math.abs(vals[0] - acc.w9.w9b.x1e12) <= 1.0) ? 1 : 0;
    return { checked: 1, failed: bad, note: `iters=${vals[1]} (informational; epher cannot count iterations in a single-statement loop body)` };
  } },
  "w9c": { file: "w9c", tracks: ["single"], special: "w9c" },
  "w9d": { file: "w9d", refsFrom: "w8-lunar", tracks: ["single"], values: 100, gate: intGate("w8-lunar", 100, 100.0) },
  "w9e": { file: "w9e", tracks: ["single"], values: 1, gate: (vals, acc) => ({ checked: 1, failed: vals[0] !== acc.w9.w9e.s ? 1 : 0 }) },
};

function flatGate(key, tol) {
  return (vals, acc) => {
    const refs = acc[key];
    let bad = 0;
    const n = Math.min(vals.length, refs.length);
    for (let i = 0; i < n; i++) {
      const r = Number(refs[i]);
      if (!(Math.abs(vals[i] - r) / Math.abs(r) <= tol)) bad++;
    }
    return { checked: n, failed: bad };
  };
}

function arrGate(n, pick) {
  return (vals, acc) => {
    const refs = pick(acc);
    let bad = 0;
    for (let i = 0; i < n; i++) {
      if (!(Math.abs(vals[i] - refs[i]) / Math.abs(refs[i]) <= 1e-9)) bad++;
    }
    return { checked: n, failed: bad };
  };
}

const SCALES = { "w8-lunar": 100, "w9d": 100, "w8-sunrise": 1, "w8-jd": 100000 };
function intGate(key, n, tol) {
  return (vals, acc) => {
    const refs = acc[key].values;
    const s = SCALES[key] ?? 1;
    let bad = 0;
    for (let i = 0; i < n; i++) {
      if (Math.abs(vals[i] / s - refs[i]) > tol) bad++;
    }
    return { checked: n, failed: bad };
  };
}

export async function bench(taskKey, variant, acceptance, octaveVersionCache) {
  const spec = GATES[taskKey];
  if (!spec) throw new Error(`unknown octave task ${taskKey}`);
  if (!spec.tracks.includes(variant)) throw new Error(`${taskKey} has no ${variant} track`);
  const mFile = join(implDir, `${spec.file}${variant === "single" ? "" : `-${variant}`}.m`);
  const out = { task: taskKey, track: variant, results: [] };

  const gate = { tolerance: "per-task tolerances from acceptance2.json (declared before measurement)", checked: 0, failed: 0 };
  if (spec.verifyFile) {
    const verifyOut = `/tmp/bench-octave-verify-${taskKey}.txt`;
    if (existsSync(verifyOut)) unlinkSync(verifyOut);
    const res = runFile(join(implDir, `${spec.verifyFile}.m`), verifyOut);
    if (!res.written || !existsSync(verifyOut)) {
      gate.failed = 1;
      gate.note = `verification run failed: ${res.stderr}`;
    } else {
      const vals = readFileSync(verifyOut, "utf8").trim().split("\n").map(Number);
      Object.assign(gate, spec.gate(vals, acceptance));
      gate.note = `values=${vals.length}`;
    }
  } else if (spec.special === "w9a") {
    const res = runFile(mFile);
    gate.checked = 1;
    if (res.count !== acceptance.w9a_count) gate.failed++;
    gate.note = `count=${res.count}, expected ${acceptance.w9a_count}`;
  } else if (spec.special === "w9c") {
    const verifyOut = `/tmp/bench-octave-verify-w9c.txt`;
    if (existsSync(verifyOut)) unlinkSync(verifyOut);
    if (existsSync("/tmp/bench-w9c-string.txt")) unlinkSync("/tmp/bench-w9c-string.txt");
    const res = runFile(mFile, verifyOut);
    const vals = existsSync(verifyOut) ? readFileSync(verifyOut, "utf8").trim().split("\n").map(Number) : [];
    const strOk = existsSync("/tmp/bench-w9c-string.txt")
      ? readFileSync("/tmp/bench-w9c-string.txt", "utf8") === acceptance.w9.w9c.expected_string
      : false;
    gate.checked = 2;
    if (!strOk) gate.failed++;
    if (!(vals[0] === acceptance.w9.w9c.balanced)) gate.failed++;
    gate.note = `string ${strOk ? "exact" : "MISMATCH"}, bal=${vals[0]} vs ${acceptance.w9.w9c.balanced}`;
  } else {
    const verifyOut = `/tmp/bench-octave-verify-${taskKey}-${variant}.txt`;
    if (existsSync(verifyOut)) unlinkSync(verifyOut);
    const res = runFile(mFile, verifyOut);
    if (!existsSync(verifyOut)) {
      gate.failed = 1;
      gate.note = `verification run failed: ${res.stderr}`;
    } else {
      const vals = readFileSync(verifyOut, "utf8").trim().split("\n").map(Number);
      Object.assign(gate, spec.gate(vals, acceptance));
      gate.note = `values=${vals.length}`;
    }
  }
  out.gate = gate;
  if (gate.failed > 0) {
    out.results.push({ verdict: "incorrect: no time published (plan rule 2)" });
    return out;
  }

  const probe = runFile(mFile);
  const samplesWanted = bucketSamples(probe.elapsed_s);
  for (let i = 0; i < 3; i++) runFile(mFile);
  const samples = [];
  let censored = false;
  for (let i = 0; i < samplesWanted; i++) {
    const res = runFile(mFile);
    if (res.elapsed_s == null) {
      censored = true;
      break;
    }
    samples.push(res.elapsed_s * 1000);
  }
  const summary = summarize(samples);
  out.results.push({
    boundary: "in-engine: tic/toc around run() from outside the file, covering read + parse + evaluation; process spawn excluded",
    warmups: 3,
    requested_samples: samplesWanted,
    bucket_rule: `probe pass ${probe.elapsed_s?.toFixed(2)} s -> ${samplesWanted} samples`,
    censored_at_cap: censored,
    ...summary,
    raw_passes_ms: samples,
  });
  return out;
}

export function version() {
  return octaveVersion();
}
