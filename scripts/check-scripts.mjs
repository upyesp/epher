// Check every script in "epher scripts/": run it with the CLI (fresh
// store, so no saved session can leak in), compare the transcript to
// the expected-output block at the end of the file, and report.
//
// Expected-output convention (see "epher scripts/README.md"): the last
// section of a script is a block comment whose opening line starts with
// "/* ---- expected output"; every line inside is one line of the CLI
// transcript, and the block closes with "*/" on its own line. The older
// line style ("// ---- expected output" then "// = ..." comments) is
// still accepted. The checker treats that block as the oracle.
//
// Run: cargo build -p epher-cli && node scripts/check-scripts.mjs
//      node scripts/check-scripts.mjs --update   (rewrite expected blocks
//      from fresh runs — review the diff afterwards; --update blesses
//      whatever the run prints, including regressions)
import { readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
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

/** The file's lines plus the index of the marker line and whether it is a
 *  block or // footer. */
function expectedMarker(path) {
  const lines = readFileSync(path, "utf8").split("\n");
  for (let i = lines.length - 1; i >= 0; i--) {
    if (lines[i].startsWith("/* ---- expected output")) return { lines, start: i, block: true };
    if (lines[i].startsWith("// ---- expected output")) return { lines, start: i, block: false };
  }
  return null;
}

/** Expected transcript from the footer block, or null when the file has
 *  no expected-output section. */
function expectedTranscript(path) {
  const m = expectedMarker(path);
  if (!m) return null;
  const out = [];
  for (let i = m.start + 1; i < m.lines.length; i++) {
    const line = m.lines[i];
    if (line.trim() === "") continue;
    if (m.block) {
      if (line.trim() === "*/") break; // the block closes at the end
      out.push(line);
    } else {
      if (!line.startsWith("// ")) break; // footer must run to EOF
      out.push(line.slice(3));
    }
  }
  return out;
}

const UPDATE = process.argv.includes("--update");
let updatedCount = 0;
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
  } else if (
    UPDATE &&
    run.status === 0 &&
    err === "" &&
    transcript.length > 0
  ) {
    // Splice the fresh transcript over the expected block and move on; the
    // caller reviews the diff. Block style: marker line, transcript, */.
    const m = expectedMarker(file);
    const body = transcript.map((l) => (m.block ? l : `// ${l}`));
    if (m.block) body.push("*/");
    writeFileSync(file, [...m.lines.slice(0, m.start + 1), ...body].join("\n"));
    updatedCount++;
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

console.log(
  `scripts: ${files.length} files, ${passed} passed, ${failed} failed, ${missing} missing expected-output blocks` +
    (UPDATE ? `, ${updatedCount} updated` : "")
);
for (const p of problems) console.log(p);
process.exit(failed + missing > 0 ? 1 : 0);
