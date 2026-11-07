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

export async function bench(taskKey, variant, acceptance, octaveVersionCache) {
  const out = { task: taskKey, track: variant, results: [] };
  const fileMap = {
    "w1|single": "w1-batch.m",
    "w2|single": "w2-batch.m",
    "w9a|idiomatic": "w9a-idiomatic.m",
    "w9a|equivalent": "w9a-equivalent.m",
  };
  const mFile = join(implDir, fileMap[`${taskKey}|${variant}`]);

  const gate = { tolerance: "relative 1e-9 vs 60-digit decimal referee", checked: 0, failed: 0 };
  if (taskKey === "w9a") {
    const res = runFile(mFile);
    gate.checked = 1;
    if (res.count !== acceptance.w9a_count) gate.failed++;
    gate.note = `count=${res.count}, expected ${acceptance.w9a_count}`;
  } else {
    Object.assign(gate, await verifyBatch(taskKey, null, acceptance));
  }
  out.gate = gate;
  if (gate.failed > 0) {
    out.results.push({ verdict: "incorrect: no time published (plan rule 2)" });
    return out;
  }

  // Probe spawn for the bucket rule, then warm-up 3 untimed + measured runs.
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
