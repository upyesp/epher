// Check every script in "epher scripts/": run it with the CLI (fresh
// store, so no saved session can leak in), compare the transcript to
// the expected-output block at the end of the file, and report.
//
// Expected-output convention (see "epher scripts/README.md"): the last
// comment section of a script starts with a line beginning
// "// ---- expected output"; every following line is a "// = ..."
// comment whose text is one line of the CLI transcript. The checker
// treats that block as the oracle.
//
// Run: cargo build -p epher-cli && node scripts/check-scripts.mjs
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";

const ROOT = resolve(join(dirname(fileURLToPath(import.meta.url)), ".."));
const SCRIPTS = join(ROOT, "epher scripts");
const CLI = process.env.EPHER_CLI || join(ROOT, "target", "debug", "epher-cli");

function walk(dir, out = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, out);
    else if (name.endsWith(".epher")) out.push(p);
  }
  return out;
}

/** Expected transcript from the footer block, or null when the file has
 *  no "// ---- expected output" section. */
function expectedTranscript(path) {
  const text = readFileSync(path, "utf8");
  const lines = text.split("\n");
  let start = -1;
  for (let i = lines.length - 1; i >= 0; i--) {
    if (lines[i].startsWith("// ---- expected output")) {
      start = i;
      break;
    }
  }
  if (start < 0) return null;
  const out = [];
  for (let i = start + 1; i < lines.length; i++) {
    const line = lines[i];
    if (line.trim() === "") continue;
    if (!line.startsWith("// ")) break; // footer must run to EOF
    out.push(line.slice(3));
  }
  return out;
}

const files = walk(SCRIPTS).sort();
const store = mkdtempSync(join(tmpdir(), "epher-scripts-"));
let passed = 0;
let failed = 0;
let missing = 0;
const problems = [];

for (const file of files) {
  const rel = relative(SCRIPTS, file);
  const expected = expectedTranscript(file);
  if (!expected) {
    missing++;
    problems.push(`no expected-output block: ${rel}`);
    continue;
  }
  const run = spawnSync(CLI, [file], {
    encoding: "utf8",
    env: { ...process.env, EPHER_STORE_DIR: store, LANG: "C", LC_ALL: "C" },
    timeout: 60_000,
  });
  const transcript = (run.stdout || "").replace(/\n$/, "").split("\n").filter((l) => l !== "");
  const err = (run.stderr || "").trim();
  let ok = run.status === 0 && err === "" && transcript.length === expected.length;
  if (ok) {
    for (let i = 0; i < expected.length; i++) {
      if (transcript[i] !== expected[i]) {
        ok = false;
        break;
      }
    }
  }
  if (ok) {
    passed++;
  } else {
    failed++;
    problems.push(
      `mismatch: ${rel} (exit ${run.status})` +
        (err ? `\n  stderr: ${err.slice(0, 200)}` : "") +
        (transcript.length !== expected.length
          ? `\n  transcript ${transcript.length} lines, expected ${expected.length}`
          : "") +
        transcript
          .map((l, i) =>
            l !== expected[i]
              ? `\n  line ${i + 1}: got  ${l}\n            want ${expected[i]}`
              : ""
          )
          .join("")
    );
  }
}

console.log(`scripts: ${files.length} files, ${passed} passed, ${failed} failed, ${missing} missing expected-output blocks`);
for (const p of problems) console.log(p);
process.exit(failed + missing > 0 ? 1 : 0);
