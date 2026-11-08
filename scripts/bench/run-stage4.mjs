// Stage 4 runner: preflight, then tools sequentially (one at a time, 30 s
// cooldown between tools, plan rule 8). Per-tool results land incrementally
// so a late failure keeps earlier tools.
//
//   node scripts/bench/run-stage4.mjs <session> [--tools epher,octave,speedcrunch,epherweb,geogebra,desmos,numworks]
//
// Two sessions on separate days are the stage 4 exit requirement; the
// within-day reproduction bar from stage 3 still applies to any same-day
// re-run.
import { spawnSync } from "node:child_process";
import { writeFileSync, readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import * as epher from "./drivers/epher-native.mjs";
import * as octave from "./drivers/octave.mjs";

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, "..", "..");
const args = process.argv.slice(2);
const session = args[0] || "a";
const toolsArg = args.includes("--tools") ? args[args.indexOf("--tools") + 1] : "epher,octave,speedcrunch";
const tools = new Set(toolsArg.split(","));
const only = args.includes("--only") ? args[args.indexOf("--only") + 1].split(",") : null;

const date = new Date().toISOString().slice(0, 10);
const outDir = join(root, "results", "raw", date, `session-${session}`);

console.log(`[stage4] preflight for session ${session}`);
const pf = spawnSync("node", [join(here, "preflight.mjs"), session], { encoding: "utf8" });
if (pf.status !== 0) {
  console.error(pf.stdout + pf.stderr);
  process.exit(1);
}
const { sha256: manifestHash } = JSON.parse(pf.stdout);

const acceptance4 = JSON.parse(readFileSync(join(root, "scripts", "bench", "data", "acceptance2.json"), "utf8"));
const acceptance3 = JSON.parse(readFileSync(join(root, "scripts", "bench", "data", "acceptance.json"), "utf8"));
// merged: the stage-3 file holds w1/w2 and w9a_count; the stage-4 file the rest
const acceptance = { ...acceptance3, ...acceptance4 };
const cooldown = (s) => {
  console.log(`[stage4] cooldown ${s}s`);
  spawnSync("sleep", [String(s)]);
};

const epherTasks = ["w1", "w2", "w3-100", "w3-500", "w4", "w5", "w6", "w7", "w8-lunar", "w8-sunrise", "w8-jd", "w9a", "w9a-trial", "w9b", "w9c", "w9d", "w9e"];
const octaveTasks = [
  ["w1", "single"], ["w2", "single"], ["w3-100", "native"], ["w3-100", "equivalent"],
  ["w3-500", "native"], ["w3-500", "equivalent"], ["w4", "native"], ["w4", "equivalent"],
  ["w5", "native"], ["w5", "equivalent"], ["w7", "single"],
  ["w8-lunar", "single"], ["w8-sunrise", "single"], ["w8-jd", "single"],
  ["w9a", "idiomatic"], ["w9a", "equivalent"], ["w9b", "single"], ["w9c", "single"], ["w9d", "single"], ["w9e", "single"],
];
// acceptance.json (stage 3 referee) holds w9a_count for the Octave driver.

const dynamic = [];
for (const mod of ["epherweb", "geogebra", "desmos", "numworks", "speedcrunch"]) {
  if (!tools.has(mod)) continue;
  try {
    dynamic.push({ name: mod, mod: await import(`./drivers/${mod}.mjs`) });
  } catch (e) {
    // speedcrunch is a .py driver handled separately below
    if (mod !== "speedcrunch") console.error(`[stage4] driver ${mod} failed to load: ${e.message}`);
  }
}

function withTimeout(promise, ms, name) {
  let timer;
  return Promise.race([
    promise,
    new Promise((_, rej) => { timer = setTimeout(() => rej(new Error(`${name} timed out after ${Math.round(ms / 60000)} min`)), ms); }),
  ]).finally(() => clearTimeout(timer));
}

async function runTool(name, fn) {
  console.log(`[stage4] ${name}: start ${new Date().toISOString()}`);
  const t0 = Date.now();
  const results = await withTimeout(Promise.resolve().then(fn), 75 * 60 * 1000, name);
  const payload = {
    tool: name,
    tier: name.endsWith("web") || ["geogebra", "desmos", "numworks"].includes(name) ? "W" : "N",
    session,
    machine_manifest_sha256: manifestHash,
    timestamp: new Date().toISOString(),
    wall_seconds: Math.round((Date.now() - t0) / 1000),
    tasks: results,
  };
  const file = join(outDir, `${name}.json`);
  writeFileSync(file, JSON.stringify(payload, null, 1));
  console.log(`[stage4] ${name}: wrote ${file} (${payload.wall_seconds}s)`);
}

const keep = (list) => (only ? list.filter((t) => only.includes(typeof t === "string" ? t : t[0])) : list);

const jobs = [];
if (tools.has("epher")) {
  jobs.push(["epher", async () => {
    const results = [];
    for (const t of keep(epherTasks)) {
      try {
        results.push(await epher.bench(t, acceptance));
      } catch (e) {
        results.push({ task: t, results: [{ verdict: `driver error: ${String(e).slice(0, 200)}` }] });
      }
    }
    return results;
  }]);
  jobs.push(["cooldown", async () => cooldown(30)]);
}
if (tools.has("epher")) {
  jobs.push(["epher-sweep", async () => runTool("epher-sweep", async () => [await epher.sweep()])]);
  jobs.push(["cooldown", async () => cooldown(30)]);
}
if (tools.has("octave")) {
  jobs.push(["octave", async () => {
    const results = [];
    const tasks = keep(octaveTasks.map(([a, b]) => b === "single" ? a : [a, b]));
    for (const t of tasks) {
      const [tk, tv] = Array.isArray(t) ? t : [t, "single"];
      try {
        results.push(await octave.bench(tk, tv, acceptance));
      } catch (e) {
        results.push({ task: tk, track: tv, results: [{ verdict: `driver error: ${String(e).slice(0, 200)}` }] });
      }
    }
    return results;
  }]);
  jobs.push(["cooldown", async () => cooldown(30)]);
}
if (tools.has("speedcrunch")) {
  jobs.push(["speedcrunch", async () => {
    const res = spawnSync("python3", [join(here, "drivers", "speedcrunch.py"), "stage4", "--acceptance", join(root, "scripts", "bench", "data", "acceptance.json"), "--acceptance4", join(root, "scripts", "bench", "data", "acceptance2.json"), "--out", join(outDir, "speedcrunch.json")], { encoding: "utf8", timeout: 21_600_000, maxBuffer: 64 * 1024 * 1024 });
    const last = (res.stdout || "").split("\n").filter((l) => l.startsWith("{")).slice(-1)[0];
    if (last) console.log(`[stage4] speedcrunch: ${last}`);
    if (res.status !== 0) console.error((res.stderr || "").slice(0, 2000));
    if (res.status === 0) {
      // the python driver wrote the full payload; return its tasks so the
      // runner's wrapper re-emits them with this session's manifest fields
      const payload = JSON.parse(readFileSync(join(outDir, "speedcrunch.json"), "utf8"));
      return payload.tasks;
    }
    return [{ task: "speedcrunch", results: [{ verdict: "driver failed; see session log" }] }];
  }]);
  jobs.push(["cooldown", async () => cooldown(30)]);
}
for (const { name, mod } of dynamic) {
  jobs.push([name, async () => mod.benchAll(acceptance, manifestHash, session)]);
  jobs.push(["cooldown", async () => cooldown(30)]);
}

for (const [name, fn] of jobs) {
  if (name === "cooldown") {
    await fn();
    continue;
  }
  try {
    await runTool(name, fn);
  } catch (e) {
    console.error(`[stage4] ${name} FAILED: ${e.stack ? e.stack.split('\n').slice(0, 4).join(' | ') : String(e).slice(0, 500)}`);
  }
}
console.log(`[stage4] session ${session} complete ${new Date().toISOString()}`);
