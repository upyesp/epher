// Preflight: environment manifest + quiet-machine checklist (plan rule 8).
// Run before every benchmark session. Aborts (exit 1) if the screen locks
// or the machine is on battery, since GUI tools need an unlocked desktop
// and thermals differ on battery.
import { execFileSync } from "node:child_process";
import { writeFileSync, mkdirSync } from "node:fs";
import { createHash } from "node:crypto";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const date = new Date().toISOString().slice(0, 10);
const session = process.argv[2] || "a";
const outDir = join(root, "results", "raw", date, `session-${session}`);
mkdirSync(outDir, { recursive: true });

const sh = (cmd, args) => execFileSync(cmd, args, { encoding: "utf8" }).trim();
// Command output can carry the local username via process paths; redact it
// before the manifest is written and hashed (committed to a public repo).
const redact = (s) => s.replaceAll("/Users/pete", "~");
const trySh = (cmd, args) => {
  try {
    return redact(sh(cmd, args));
  } catch (e) {
    return `unavailable: ${e.status ?? e.message}`;
  }
};

const lockOut = trySh("python3", [join(root, "scripts", "a11y", "session-lock.py")]);
if (/warning|(?<!un)locked/i.test(lockOut)) {
  console.error("session-lock.py reports a locked screen; aborting:\n" + lockOut);
  process.exit(1);
}

const batt = trySh("pmset", ["-g", "batt"]);
if (!/AC Power/i.test(batt)) {
  console.error("machine is not on AC power; aborting:\n" + batt);
  process.exit(1);
}

const ps = trySh("ps", ["-Ao", "pcpu,comm", "-r"]);
const topCpu = ps
  .split("\n")
  .slice(1)
  .map((l) => l.trim())
  .filter((l) => l)
  .slice(0, 8);

const manifest = {
  session,
  timestamp: new Date().toISOString(),
  host: {
    model: trySh("sysctl", ["-n", "hw.model"]),
    cpu: trySh("sysctl", ["-n", "machdep.cpu.brand_string"]),
    memory_bytes: trySh("sysctl", ["-n", "hw.memsize"]),
    macos: `${trySh("sw_vers", ["-productVersion"])} (${trySh("sw_vers", ["-buildVersion"])})`,
  },
  power: batt.split("\n")[0] ?? "",
  thermal: trySh("pmset", ["-g", "therm"]),
  screen_lock: lockOut,
  top_cpu_processes: topCpu,
  versions: {
    git: sh("git", ["rev-parse", "HEAD"]),
    octave: trySh("/opt/homebrew/bin/octave-cli", ["--version"]).split("\n")[0],
    speedcrunch: trySh("defaults", [
      "read",
      "/Applications/SpeedCrunch.app/Contents/Info",
      "CFBundleShortVersionString",
    ]),
    browser: "not used in the stage 3 pilot (N tier only)",
  },
  notes: [
    "quiet-machine checklist: no other heavy processes expected; top_cpu_processes recorded for audit",
    "thermal state recorded at session start; re-record at session end",
    "user paths redacted to ~ before hashing",
  ],
};

const file = join(outDir, "env-manifest.json");
writeFileSync(file, JSON.stringify(manifest, null, 1));
const hash = createHash("sha256").update(JSON.stringify(manifest, null, 1)).digest("hex");
console.log(JSON.stringify({ file, sha256: hash, session, date }));
