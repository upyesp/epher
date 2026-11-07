// epher native driver (stage 3 pilot). In-engine boundary: the harness
// binary links epher-core and times parse+eval in-process (plan rule 3).
// Verify first (rule 2), then warm-up 3 + bucket-decided samples (rules 4-5).
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { summarize, bucketSamples } from "../stats.mjs";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const benchEval = join(root, "target", "release", "examples", "bench_eval");
const dataDir = join(root, "scripts", "bench", "data");
const implDir = join(root, "scripts", "bench", "implementations", "epher");

const TASKS = {
  w1: { mode: "expr", data: join(dataDir, "W1-expressions.txt"), label: "W1 scalar arithmetic" },
  w2: { mode: "expr", data: join(dataDir, "W2-transcendental-ln.txt"), label: "W2 transcendental microbench", note: "uses the committed ln-rendered copy (epher's log() is base-10; canonical log( means natural log)" },
  w9a: { mode: "script", data: join(implDir, "w9a-primes.epher"), label: "W9a prime count (trial division)" },
};

function runBinary(args) {
  return JSON.parse(execFileSync(benchEval, args, { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 }));
}

function relOk(toolValue, refereeValue, tol = 1e-9) {
  const t = Number(toolValue);
  const r = Number(refereeValue);
  if (!Number.isFinite(t) || !Number.isFinite(r)) return false;
  if (r === 0) return Math.abs(t) < tol;
  return Math.abs(t - r) / Math.abs(r) <= tol;
}

export function version() {
  const crateVersion = readFileSync(join(root, "crates", "core", "Cargo.toml"), "utf8").match(/version = "([^"]+)"/)[1];
  const commit = execFileSync("git", ["rev-parse", "--short=12", "HEAD"], { encoding: "utf8" }).trim();
  return `epher-core ${crateVersion} (harness bench_eval, git ${commit})`;
}

export async function bench(taskKey, acceptance) {
  const task = TASKS[taskKey];
  const out = { task: taskKey, label: task.label, results: [] };

  // Correctness gate (rule 2): evaluate the whole dataset once and compare.
  const verifyArgs = task.mode === "expr" ? ["expr-verify", task.data] : ["script-verify", task.data];
  const gate = { tolerance: "relative 1e-9 vs 60-digit decimal referee", checked: 0, failed: 0 };
  if (task.mode === "expr") {
    const values = execFileSync(benchEval, verifyArgs, { encoding: "utf8", maxBuffer: 64 * 1024 * 1024 }).trim().split("\n");
    const ref = acceptance[taskKey];
    for (let i = 0; i < ref.length; i++) {
      gate.checked++;
      if (!relOk(values[i], ref[i])) gate.failed++;
    }
  } else {
    const finalStr = execFileSync(benchEval, verifyArgs, { encoding: "utf8" }).trim();
    gate.checked = 1;
    if (Math.round(Number(finalStr)) !== acceptance.w9a_count) gate.failed++;
    gate.note = `final=${finalStr}, expected count ${acceptance.w9a_count}`;
  }
  out.gate = gate;
  if (gate.failed > 0) {
    out.results.push({ track: "single", verdict: "incorrect: no time published (plan rule 2)" });
    return out;
  }

  // Probe pass to fix the sample bucket (declared rule), then the measured run.
  const probe = runBinary([task.mode === "expr" ? "expr-batch" : "script-run", task.data, "3", "1"]);
  const samplesWanted = bucketSamples(probe.passes_ms[0] / 1000);
  const final = runBinary([task.mode === "expr" ? "expr-batch" : "script-run", task.data, "3", String(samplesWanted)]);
  const summary = summarize(final.passes_ms);
  out.results.push({
    track: "single",
    track_note:
      task.mode === "expr"
        ? "W1/W2 fix the expressions, so the idiomatic and equivalent implementations coincide by construction (declared in the pilot README)."
        : "epher has no list index assignment (ADR-0044 gives reads only), so the flag-array sieve is not expressible: the equivalent track is recorded as not supported, not timed (plan rule 11). Trial division is epher's idiomatic implementation.",
    boundary: "in-engine: parse + eval, in-process Instant in the harness binary linking epher-core; no printing inside the span",
    warmups: 3,
    requested_samples: samplesWanted,
    bucket_rule: `warm-up pass ${probe.passes_ms[0].toFixed(2)} ms -> ${samplesWanted} samples`,
    ...summary,
    raw_passes_ms: final.passes_ms,
  });
  return out;
}
