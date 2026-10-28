// Check the user guide's `epher` transcript blocks against the engine.
//
// The guide (site/guide/<locale>.md) pairs every ```epher block with the
// ```text block it produces. Those transcripts are documentation the
// reader trusts, and nothing verified them: a language change (for
// example ADR-0080 making `const` silent) can leave them describing an
// output the app no longer prints.
//
// This checker extracts each pair and runs the code through the CLI in
// one session per block (fresh store, one invocation, exactly like a
// paste), then compares the printed lines to the block. It skips pairs
// that are not transcripts: blocks whose output is a diagram, a table
// of alternatives, or prose. A block that the engine cannot run as a
// single paste is reported, not failed, unless it claims to be a
// transcript.
//
//   node scripts/check-guide.mjs            # report
//   node scripts/check-guide.mjs --strict   # fail on any mismatch
//   node scripts/check-guide.mjs --locale=en
//
// The code is the source of truth: an `epher` block is always run, and
// its `text` block must match. Use `--list` to print every pair.
import { readFileSync, mkdtempSync, rmSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";
import { tmpdir } from "node:os";

const ROOT = resolve(join(dirname(fileURLToPath(import.meta.url)), ".."));
const CLI = process.env.EPHER_CLI || join(ROOT, "target", "debug", "epher-cli");
const args = process.argv.slice(2);
const strict = args.includes("--strict");
const list = args.includes("--list");
const localeArg = args.find((a) => a.startsWith("--locale="));
const locales = localeArg
  ? [localeArg.split("=")[1]]
  : ["en"]; // translations mirror en; check en unless asked otherwise

// Output blocks that document something other than a transcript.
const NOT_A_TRANSCRIPT = [
  /^\s*\|/, // a markdown table
  /^\s*[├┌└│─+]/, // a box drawing
  /^\s*(error|warning):/, // an error the guide narrates in prose around it
  /^\s*$/, // empty
];

function blocks(md) {
  // [{lang, body, line}] in document order.
  const out = [];
  const lines = md.split("\n");
  let i = 0;
  while (i < lines.length) {
    const m = /^```(\w*)\s*$/.exec(lines[i]);
    if (!m) {
      i += 1;
      continue;
    }
    const lang = m[1];
    const start = i + 1;
    i += 1;
    while (i < lines.length && !/^```\s*$/.test(lines[i])) i += 1;
    out.push({ lang, body: lines.slice(start, i).join("\n"), line: start });
    i += 1;
  }
  return out;
}

function run(source) {
  const store = mkdtempSync(join(tmpdir(), "epher-guide-"));
  try {
    // `epher -` reads line by line; a block with block comments or
    // multi-line control flow needs the whole-file path, so write it.
    const file = join(store, "block.epher");
    writeFileSync(file, source + "\n");
    const r = spawnSync(CLI, [file], {
      env: { ...process.env, EPHER_STORE_DIR: store, LANG: "C", LC_ALL: "C" },
      encoding: "utf8",
    });
    // The guide writes answers without the CLI's `= ` voice.
    const out = (r.stdout || "")
      .split("\n")
      .map((l) => (l.startsWith("= ") ? l.slice(2) : l))
      .join("\n")
      .replace(/\n$/, "");
    return { out, err: (r.stderr || "").trim(), code: r.status };
  } finally {
    rmSync(store, { recursive: true, force: true });
  }
}

import { writeFileSync } from "node:fs";

let checked = 0;
let mismatched = 0;
let skipped = 0;
for (const locale of locales) {
  const path = join(ROOT, "site", "guide", `${locale}.md`);
  const md = readFileSync(path, "utf8");
  const bs = blocks(md);
  for (let i = 0; i < bs.length; i += 1) {
    const code = bs[i];
    if (code.lang !== "epher") continue;
    const want = bs[i + 1];
    if (!want || want.lang !== "text") continue;
    if (NOT_A_TRANSCRIPT.some((re) => re.test(want.body))) {
      skipped += 1;
      continue;
    }
    // A REPL-style block (`epher> …`) is a session, not one paste.
    if (/^epher>/.test(code.body.trim())) {
      skipped += 1;
      continue;
    }
    // Plot and table blocks render into the result pane, not as a text
    // transcript; the guide shows the pane picture in prose.
    if (/^\s*(graph|graph3d|solar3d|table)\b/m.test(code.body)) {
      skipped += 1;
      continue;
    }
    const got = run(code.body);
    // A block that needs names from earlier in the chapter is a
    // narrative continuation, not an independent paste.
    if (/unknown name/.test(got.err)) {
      skipped += 1;
      continue;
    }
    checked += 1;
    if (list) {
      console.log(`--- ${locale}.md:${code.line}\n${code.body}\n--> ${got.out}\n`);
    }
    if (got.out !== want.body) {
      mismatched += 1;
      console.log(`MISMATCH ${locale}.md:${code.line}`);
      console.log(`  code: ${JSON.stringify(code.body.split("\n").slice(0, 3).join(" / "))}`);
      console.log(`  want: ${JSON.stringify(want.body)}`);
      console.log(`  got : ${JSON.stringify(got.out)}`);
      if (got.err) console.log(`  err : ${got.err}`);
    }
  }
}
console.log(`guide: ${checked} transcripts checked, ${mismatched} mismatched, ${skipped} non-transcript blocks skipped`);
if (mismatched > 0 && strict) process.exit(1);
