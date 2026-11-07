// Stage 3 pilot runner: preflight, then epher -> Octave -> SpeedCrunch
// sequentially (one tool at a time, 30 s cooldown between tools, plan rule 8).
// Per-tool results land incrementally so a late failure keeps earlier tools.
//
//   node scripts/bench/run-pilot.mjs <session> [--tools epher,octave,speedcrunch]
//
// Exit criterion: the same command run twice the same day (session a, b)
// reproduces medians within 10 percent.
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

const date = new Date().toISOString().slice(0, 10);
const outDir = join(root, "results", "raw", date, `session-${session}`);

console.log(`[pilot] preflight for session ${session}`);
const pf = spawnSync("node", [join(here, "preflight.mjs"), session], { encoding: "utf8" });
if (pf.status !== 0) {
  console.error(pf.stdout + pf.stderr);
  process.exit(1);
}
const { sha256: manifestHash, date: manifestDate } = JSON.parse(pf.stdout);

const acceptance = JSON.parse(readFileSync(join(root, "scripts", "bench", "data", "acceptance.json"), "utf8"));
const cooldown = (s) => {
  console.log(`[pilot] cooldown ${s}s`);
  spawnSync("sleep", [String(s)]);
};

async function runTool(name, fn) {
  console.log(`[pilot] ${name}: start ${new Date().toISOString()}`);
  const t0 = Date.now();
  const results = await fn();
  const payload = {
    tool: name,
    tier: "N",
    session,
    machine_manifest_sha256: manifestHash,
    timestamp: new Date().toISOString(),
    wall_seconds: Math.round((Date.now() - t0) / 1000),
    tasks: results,
  };
  if (name === "epher") payload.version = epher.version();
  if (name === "octave") payload.version = octave.version();
  if (name === "speedcrunch")
    payload.version = `SpeedCrunch ${spawnSync("defaults", ["read", "/Applications/SpeedCrunch.app/Contents/Info", "CFBundleShortVersionString"], { encoding: "utf8" }).stdout.trim()}`;
  const file = join(outDir, `${name}.json`);
  writeFileSync(file, JSON.stringify(payload, null, 1));
  console.log(`[pilot] ${name}: wrote ${file} (${payload.wall_seconds}s)`);
}

const allTasks = [];

if (tools.has("epher")) {
  allTasks.push(() => runTool("epher", async () => [
    await epher.bench("w1", acceptance),
    await epher.bench("w2", acceptance),
    await epher.bench("w9a", acceptance),
  ]));
  allTasks.push(() => cooldown(30));
}
if (tools.has("octave")) {
  allTasks.push(() => runTool("octave", async () => [
    await octave.bench("w1", "single", acceptance),
    await octave.bench("w2", "single", acceptance),
    await octave.bench("w9a", "idiomatic", acceptance),
    await octave.bench("w9a", "equivalent", acceptance),
  ]));
  allTasks.push(() => cooldown(30));
}
if (tools.has("speedcrunch")) {
  for (const t of ["w1", "w2"]) {
    allTasks.push(async () => {
      console.log(`[pilot] speedcrunch ${t}: start ${new Date().toISOString()}`);
      const out = join(outDir, `speedcrunch-${t}.json`);
      const res = spawnSync("python3", [
        join(here, "drivers", "speedcrunch.py"),
        t,
        "--data",
        join(root, "scripts", "bench", "data", t === "w1" ? "W1-expressions.txt" : "W2-transcendental-ln.txt"),
        "--acceptance",
        join(root, "scripts", "bench", "data", "acceptance.json"),
        "--out",
        out,
      ], { encoding: "utf8", timeout: 7200_000, maxBuffer: 16 * 1024 * 1024 });
      if (res.status !== 0) {
        console.error(`[pilot] speedcrunch ${t} failed:\n${res.stdout}\n${res.stderr}`);
      }
      return JSON.parse(res.stdout || "{}");
    });
  }
  allTasks.push(async () => {
    // Merge the per-task files into one tool result file.
    const tasks = ["w1", "w2"].map((t) => {
      const f = join(outDir, `speedcrunch-${t}.json`);
      return JSON.parse(readFileSync(f, "utf8"));
    });
    const pfData = JSON.parse(readFileSync(join(outDir, "env-manifest.json"), "utf8"));
    await runTool("speedcrunch", async () => tasks);
  });
}

for (const step of allTasks) {
  await step();
}

// Thermal state at session end (plan rule 8).
const thermEnd = spawnSync("pmset", ["-g", "therm"], { encoding: "utf8" }).stdout.trim();
writeFileSync(join(outDir, "thermal-end.txt"), thermEnd);
console.log(`[pilot] session ${session} complete ${new Date().toISOString()}`);
