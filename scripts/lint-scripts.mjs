// Lint the scripts fleet for names that collide with the language.
//
// Fails when a script binds (name = / const name =) or defines (def name)
// a built-in function's name from site/reference.md section 10 — with the
// session's persistence a shadowing variable survives the script and
// changes what the builtin means in later runs (see
// docs/scripts-audit-2026-10-26.md, the extra-payment-savings incident).
//
// Unit names (reference.md section 11) only warn: legal today, a
// readability trap. Pass --units to promote the warnings to failures.
//
// Run: node scripts/lint-scripts.mjs [--units]
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, dirname, relative } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SCRIPTS = join(ROOT, "epher scripts");
const REFERENCE = readFileSync(join(ROOT, "site", "reference.md"), "utf8");
const FAIL_ON_UNITS = process.argv.includes("--units");

// Built-ins: the call-shaped names in section 10 tables' first cell only —
// answer cells can hold formulas like `p(1+r)^n - p` that are not callables.
const sec10 = REFERENCE.split("## 10. Built-in functions")[1].split("## 11.")[0];
const builtins = new Set();
for (const row of sec10.matchAll(/^\| `([^`]+)` \|/gm)) {
  for (const m of row[1].matchAll(/([a-z_][a-z0-9_]*)\(/g)) builtins.add(m[1]);
}
// Units: the `| `name` | factor | dimensions |` rows of section 11.
const sec11 = REFERENCE.split("## 11. Units")[1].split("## 12.")[0];
const units = new Set(
  [...sec11.matchAll(/^\| `([^`]+)` \| [\d.e+-]+ \|/gm)].map((m) => m[1])
);

function walk(dir, out = []) {
  for (const name of readdirSync(dir).sort()) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, out);
    else if (name.endsWith(".epher")) out.push(p);
  }
  return out;
}

function stripComments(text) {
  return text.replace(/\/\*[\s\S]*?\*\//g, "").replace(/\/\/.*$/gm, "");
}

const failures = [];
const warnings = [];
for (const path of walk(SCRIPTS)) {
  const code = stripComments(readFileSync(path, "utf8"));
  const rel = relative(ROOT, path);
  // Top-level statements start at column 0 (the fleet is one statement per
  // line); destructuring `{a, b} = ...` starts with a brace and cannot match.
  const bindings = [
    ...code.matchAll(/^(?:const\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=(?!=)/gm),
  ].map((m) => ({ name: m[1], kind: m[0].startsWith("const") ? "const" : "variable" }));
  const defs = [...code.matchAll(/^def\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(/gm)].map(
    (m) => ({ name: m[1], kind: "def" })
  );
  for (const { name, kind } of [...bindings, ...defs]) {
    const where = `${rel}: ${kind} ${name}`;
    if (builtins.has(name)) failures.push(`${where} — shadows builtin ${name}()`);
    else if (units.has(name)) warnings.push(`${where} — shares a unit name (${name})`);
  }
}

for (const w of warnings) console.log(`warn: ${w}`);
for (const f of failures) console.log(`FAIL: ${f}`);
console.log(
  `lint-scripts: ${failures.length} failure(s), ${warnings.length} unit-name warning(s)` +
    (FAIL_ON_UNITS ? " (--units: warnings count as failures)" : "")
);
if (failures.length || (FAIL_ON_UNITS && warnings.length)) process.exit(1);
