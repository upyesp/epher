// Compares two pilot sessions' medians (stage 3 exit criterion: within 10%).
import { readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const date = process.argv[2] || new Date().toISOString().slice(0, 10);
const [a, b] = [process.argv[3] || "a", process.argv[4] || "b"];

function medians(session) {
  const out = {};
  for (const tool of ["epher", "octave", "speedcrunch"]) {
    let file;
    try {
      file = JSON.parse(readFileSync(join(root, "results", "raw", date, `session-${session}`, `${tool}.json`), "utf8"));
    } catch {
      continue;
    }
    for (const t of file.tasks) {
      for (const r of t.results) {
        const key = `${tool}:${t.task}${t.track && t.track !== "single" ? ":" + t.track : ""}`;
        if (r.median != null) out[key] = r.median;
      }
    }
  }
  return out;
}

const ma = medians(a);
const mb = medians(b);
let worst = 0;
console.log("cell                     session-a(ms)   session-b(ms)   delta%");
for (const k of Object.keys(ma)) {
  const va = ma[k];
  const vb = mb[k];
  const delta = ((vb - va) / va) * 100;
  worst = Math.max(worst, Math.abs(delta));
  console.log(
    `${k.padEnd(24)} ${va.toFixed(3).padStart(12)}  ${vb.toFixed(3).padStart(12)}  ${delta.toFixed(1).padStart(7)}`
  );
}
console.log(`\nworst absolute delta: ${worst.toFixed(1)}% (exit criterion: within 10%)`);
process.exit(worst <= 10 ? 0 : 1);
