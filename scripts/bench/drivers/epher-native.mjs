// epher native driver (stage 4). In-engine boundary: the harness binary
// links epher-core and times parse+eval in-process (plan rule 3). Gate
// before timing (rule 2), warm-up 3 + bucket samples (rules 4-5). Per-task
// gate tolerances come from acceptance2.json and were declared before any
// stage 4 measurement.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { summarize, bucketSamples } from "../stats.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const benchEval = join(root, "target", "release", "examples", "bench_eval");
const dataDir = join(root, "scripts", "bench", "data");
const implDir = join(root, "scripts", "bench", "implementations", "epher");

function relOk(tool, ref, tol) {
  if (ref === 0) return Math.abs(tool) < tol;
  return Math.abs(tool - ref) / Math.abs(ref) <= tol;
}

const w3Gate = (n) => (vals, acc) => {
  const a = acc[`w3-${n}`];
  let bad = 0;
  // det gates compare magnitude only (sign flips under float64 pivoting
  // near-ties between independent LU implementations)
  const checks = [
    [Math.abs(vals[0]), Math.abs(a.detA), a.det_rel_tol, "rel"],
    [Math.abs(vals[1]), Math.abs(a.detB), a.det_rel_tol, "rel"],
    [vals[2], a.traceAB, a.traceAB_rel_tol, "rel"],
    [vals[3], a.xhat_expected_sum_abs, a.xhat_sum_abs_rel_tol, "rel"],
  ];
  for (const [tool, ref, tol] of checks) if (!relOk(tool, ref, tol)) bad++;
  if (!(vals[4] <= a.resid_l1_gate)) bad++;
  return { checked: 5, failed: bad };
};

const TASKS = {
  w1: { mode: "expr", data: join(dataDir, "W1-expressions.txt"), label: "W1 scalar arithmetic", gate: "flat1e9" },
  w2: { mode: "expr", data: join(dataDir, "W2-transcendental-ln.txt"), label: "W2 transcendental microbench", gate: "flat1e9", note: "uses the committed ln-rendered copy (epher's log() is base-10; canonical log( means natural log)" },
  "w3-100": { mode: "script", data: join(implDir, "w3-100.epher"), label: "W3 linear algebra, 100x100", gate: w3Gate(100), trackNote: "native matrix literals, det/inv/trace builtins" },
  "w3-500": { mode: "script", data: join(implDir, "w3-500.epher"), label: "W3 linear algebra, 500x500 (pre-scaled)", gate: w3Gate(500), trackNote: "matrices pre-scaled by n^(-1/2) so det stays in float64 range (declared before measurement)" },
  w4: { mode: "script", data: join(implDir, "w4.epher"), label: "W4 statistics", gate: (vals, acc) => {
    const refs = [acc.w4.mean, acc.w4.stdev, acc.w4.median, acc.w4.p10, acc.w4.p90, acc.w4.lin_slope, acc.w4.lin_intercept, acc.w4.lin_r];
    let bad = 0;
    for (let i = 0; i < 8; i++) if (!relOk(vals[i], refs[i], 1e-9)) bad++;
    return { checked: 8, failed: bad };
  }, trackNote: "mean/median native; sample stdev from the population variance; percentiles Type-7 via sort+index reads (epher's quartile is the Excel method); least squares over indexed reads (native linreg returns text)" },
  w5: { mode: "script", data: join(implDir, "w5.epher"), label: "W5 numerical methods", gate: (vals, acc) => {
    // file statement order: the three bisection values print first, then i1..i3
    let bad = 0;
    const I = [acc.w5.integrals.I1, acc.w5.integrals.I2, acc.w5.integrals.I3];
    for (let i = 0; i < 3; i++) if (!relOk(vals[3 + i], I[i], 1e-9)) bad++;
    const R = [acc.w5.roots.r1, acc.w5.roots.r2, acc.w5.roots.r3];
    for (let i = 0; i < 3; i++) if (!(Math.abs(vals[i] - R[i]) <= 1e-12)) bad++;
    return { checked: 6, failed: bad };
  }, trackNote: "integrals via native integral(); roots via explicit bisection (epher's solve is statement-form with display output only, verified 2026-11-08)" },
  w6: { mode: "w6", label: "W6 unit conversions", coverageRule: "a tool publishes a time only at 50/50 unit support; otherwise partial coverage with the supported rows gated" },
  w7: { mode: "expr", data: join(implDir, "w7.epher"), label: "W7 complex arithmetic", gate: "flat1e9", trackNote: "re()/im() emitted per expression; complex output itself is not gate-comparable as a float" },
  "w8-lunar": { mode: "script", data: join(implDir, "w8-lunar.epher"), label: "W8 lunar phase, 100 dates", gate: intGate("w8-lunar", 100, 1.0), trackNote: "native phase()/jd()" },
  "w8-sunrise": { mode: "script", data: join(implDir, "w8-sunrise.epher"), label: "W8 sunrise minutes, 20 pairs", gate: intGate("w8-sunrise", 20, 3.0), trackNote: "native rise()/jd()" },
  "w8-jd": { mode: "script", data: join(implDir, "w8-jd.epher"), label: "W8 Julian date round trips", gate: intGate("w8-jd", 20, 1.0), trackNote: "native jd()" },
  "w9a": { mode: "script", data: join(implDir, "w9a.epher"), label: "W9a prime count", gate: "count", trackNote: "idiomatic: isprime() loop; the flag-array sieve needs list index assignment (ADR-0044 reads only) and is not expressible" },
  "w9a-trial": { mode: "script", data: join(implDir, "w9a-primes.epher"), label: "W9a prime count (trial division, equivalent track)", gate: "count", trackNote: "the equivalent-algorithm implementation mirrors the same trial-division structure other tools use" },
  w9b: { mode: "script", data: join(implDir, "w9b.epher"), label: "W9b fixed-point iteration", gate: (vals, acc) => ({ checked: 1, failed: Math.abs(vals[0] - acc.w9.w9b.x1e12) <= 1.0 ? 0 : 1, note: `iters not collected (single-statement loop body); referee count ${acc.w9.w9b.iters}` }) },
  w9c: { mode: "script", data: join(implDir, "w9c.epher"), label: "W9c string processing", gate: (vals, acc) => {
    const strOk = vals[0] === "STR:" + acc.w9.w9c.expected_string.split("\n").join("\\n");
    const balOk = Math.round(vals[1]) === acc.w9.w9c.balanced;
    return { checked: 2, failed: strOk && balOk ? 0 : 1, note: `string ${strOk ? "exact" : "MISMATCH"}, bal=${vals[1]} vs ${acc.w9.w9c.balanced}` };
  } },
  w9d: { mode: "script", data: join(implDir, "w9d.epher"), label: "W9d lunar phase loop", gate: intGate("w9d", 100, 1.0) },
  w9e: { mode: "script", data: join(implDir, "w9e.epher"), label: "W9e branchy accumulator", gate: (vals, acc) => ({ checked: 1, failed: Math.round(vals[0]) !== acc.w9.w9e.s ? 1 : 0 }) },
};

function intGate(key, n, tol) {
  return (vals, acc) => {
    // referee values are in natural units (degrees, minutes, days); the
    // implementations print scaled integers (x100 phase, x1e5 jd)
    const refs = (key === "w9d" ? acc["w8-lunar"] : acc[key]).values;
    const scale = key === "w8-lunar" || key === "w9d" ? 100 : key === "w8-jd" ? 100000 : 1;
    let bad = 0;
    for (let i = 0; i < n; i++) {
      if (Math.abs(vals[i] / scale - refs[i]) > tol) bad++;
    }
    return { checked: n, failed: bad };
  };
}

function runBinary(args, opts = {}) {
  return execFileSync(benchEval, args, { encoding: "utf8", maxBuffer: 64 * 1024 * 1024, ...opts });
}
function runJson(args, opts = {}) {
  // script-run emits literal NaN for non-numeric finals (not valid JSON)
  const raw = runBinary(args, opts).replace(/\bNaN\b/g, "null");
  return JSON.parse(raw);
}

export function version() {
  const crateVersion = readFileSync(join(root, "crates", "core", "Cargo.toml"), "utf8").match(/version = "([^"]+)"/)[1];
  const commit = execFileSync("git", ["rev-parse", "--short=12", "HEAD"], { encoding: "utf8" }).trim();
  return `epher-core ${crateVersion} (harness bench_eval, git ${commit})`;
}

export async function bench(taskKey, acceptance) {
  const task = TASKS[taskKey];
  if (!task) throw new Error(`unknown epher task ${taskKey}`);
  const out = { task: taskKey, label: task.label, results: [] };
  const trackNote = task.trackNote;

  if (task.mode === "w6") {
    // Coverage pass: all 50 rows, continue on errors. Time publishes only
    // at 50/50 (declared rule).
    const lines = runBinary(["expr-verify-cont", join(implDir, "w6.epher")]).trim().split("\n");
    let supported = 0;
    let wrong = 0;
    const acc = acceptance.w6;
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (line.startsWith("ERR:") || line.startsWith("NAN")) continue;
      const m = line.match(/^QTY:(\S+) (\S+)$/);
      if (!m) continue;
      const row = acc.rows[i];
      if (!row) continue;
      supported++;
      if (row.kind === "factor") {
        const target = Number(row.target);
        if (!(Math.abs(Number(m[1]) - target) / Math.abs(target) <= acc.rel_tol)) wrong++;
      } else {
        wrong++; // temperature offset rows: epher has no offset scales, so a QTY here would be suspect
      }
    }
    out.gate = {
      tolerance: "factor rows: relative 1e-9 vs exact SI/NIST decimal targets; coverage rule declared before measurement",
      checked: 50,
      failed: wrong,
      supported,
      note: `partial: ${supported}/50 rows expressible (no cal/BTU/kWh/MJ/hp; no temperature offset scales), all supported factor rows compared`,
    };
    if (supported === 50 && wrong === 0) {
      out.results.push({ track: "single", verdict: "covered; timing would run here", coverage: supported });
    } else {
      out.results.push({
        track: "single",
        verdict: `partial: ${supported}/50 unit rows supported, ${wrong} of the supported rows wrong; no time published (declared coverage rule)`,
        coverage: supported,
      });
    }
    return out;
  }

  // Gate (rule 2)
  const gate = { tolerance: "per-task tolerances from acceptance2.json (declared before measurement)", checked: 0, failed: 0 };
  let gateVals;
  if (task.gate === "flat1e9") {
    const values = runBinary([task.mode === "expr" ? "expr-verify" : "script-verify-all", task.data]).trim().split("\n").map(Number);
    const ref = acceptance[taskKey];
    let bad = 0;
    const n = Math.min(values.length, ref.length);
    for (let i = 0; i < n; i++) if (!relOk(values[i], ref[i], 1e-9)) bad++;
    gate.checked = n;
    gate.failed = bad;
    gate.note = `values=${values.length}`;
    gateVals = values;
  } else if (task.gate === "count") {
    const finalStr = runBinary(["script-verify-all", task.data]).trim().split("\n").pop();
    gate.checked = 1;
    if (Math.round(Number(finalStr)) !== acceptance.w9a_count) gate.failed++;
    gate.note = `final=${finalStr}, expected count ${acceptance.w9a_count}`;
  } else {
    const raw = runBinary(["script-verify-all", task.data]).trim().split("\n");
    gateVals = raw.map((l) => (l.startsWith("STR:") ? l : Number(l)));
    Object.assign(gate, task.gate(gateVals, acceptance));
    gate.note = gate.note ?? `values=${raw.length}`;
  }
  out.gate = gate;
  if (gate.failed > 0) {
    out.results.push({ track: "single", track_note: trackNote, verdict: "incorrect: no time published (plan rule 2)" });
    return out;
  }

  if (task.mode === "w6") return out; // unreachable; w6 handled above

  // Probe pass for the bucket, then measured samples.
  const probe = runJson([task.mode === "expr" ? "expr-batch" : "script-run", task.data, "3", "1"]);
  const samplesWanted = bucketSamples(probe.passes_ms[0] / 1000);
  const final = runJson([task.mode === "expr" ? "expr-batch" : "script-run", task.data, "3", String(samplesWanted)]);
  const summary = summarize(final.passes_ms);
  out.results.push({
    track: "single",
    track_note: trackNote,
    boundary: "in-engine: parse + eval, in-process Instant in the harness binary linking epher-core; no printing inside the span",
    warmups: 3,
    requested_samples: samplesWanted,
    bucket_rule: `warm-up pass ${probe.passes_ms[0].toFixed(2)} ms -> ${samplesWanted} samples`,
    ...summary,
    raw_passes_ms: final.passes_ms,
  });
  return out;
}

// epher-only sweep (not cross-app): all 532 shipped scripts, one pass each,
// in-engine parse+eval per script; medians per top-level category.
export async function sweep() {
  const base = join(root, "epher scripts");
  const files = execFileSync("find", [base, "-name", "*.epher"], { encoding: "utf8" }).trim().split("\n").sort();
  const entries = [];
  const errors = [];
  for (const f of files) {
    try {
      const r = runJson(["script-run", f, "0", "1"]);
      const category = f.slice(base.length + 1).split("/")[0];
      entries.push({ path: f.slice(base.length + 1), category, ms: r.passes_ms[0] });
    } catch (e) {
      errors.push({ path: f.slice(base.length + 1), error: String(e).slice(0, 120) });
    }
  }
  const byCategory = {};
  for (const e of entries) (byCategory[e.category] = byCategory[e.category] || []).push(e.ms);
  const categories = Object.entries(byCategory).map(([cat, arr]) => {
    const s = summarize(arr);
    return { category: cat, scripts: arr.length, ...s };
  }).sort((a, b) => a.category.localeCompare(b.category));
  return {
    task: "sweep", label: "epher-only shipped-script sweep (532 scripts)",
    gate: { checked: entries.length + errors.length, failed: errors.length, note: errors.length ? `${errors.length} scripts errored` : "all scripts ran" },
    results: [{
      track: "single",
      boundary: "in-engine parse + eval per script, one pass each, in-process Instant in the harness binary; process spawn excluded (declared for all epher in-engine cells)",
      total_scripts: entries.length,
      errored: errors,
      categories,
      all_entries: entries,
    }],
  };
}
