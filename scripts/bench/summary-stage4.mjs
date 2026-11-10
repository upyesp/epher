#!/usr/bin/env node
// Stage 4 results summary: reads results/raw/<date>/session-<a|b>/*.json and
// prints the per-cell gate status + medians side by side. The JSON files are
// the record; this table is their summary.
import { readFileSync, readdirSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = join(here, '..', '..');
const date = process.argv[2] || '2026-11-08';

function cells(dir) {
  const out = {};
  if (!existsSync(dir)) return out;
  for (const f of readdirSync(dir)) {
    if (!f.endsWith('.json') || f === 'env-manifest.json') continue;
    let d;
    try { d = JSON.parse(readFileSync(join(dir, f), 'utf8')); } catch { continue; }
    for (const t of d.tasks ?? []) {
      const key = `${f.replace('.json', '')}:${t.task}${t.track && t.track !== 'single' ? '/' + t.track : ''}`;
      const r = (t.results ?? [])[0] ?? {};
      out[key] = {
        failed: t.gate?.failed,
        checked: t.gate?.checked,
        median: r.median,
        verdict: r.verdict,
      };
    }
  }
  return out;
}

const a = cells(join(root, 'results', 'raw', date, 'session-a'));
const b = cells(join(root, 'results', 'raw', date, 'session-b'));

const keys = [...new Set([...Object.keys(a), ...Object.keys(b)])].sort();
console.log('cell'.padEnd(28), 'a gate'.padEnd(10), 'a median'.padEnd(14), 'b gate'.padEnd(10), 'b median'.padEnd(14), 'delta%');
for (const k of keys) {
  const ca = a[k], cb = b[k];
  const ga = ca ? `${ca.failed}/${ca.checked}` : '-';
  const gb = cb ? `${cb.failed}/${cb.checked}` : '-';
  const ma = ca?.median != null ? ca.median.toPrecision(4) : (ca?.verdict ? ca.verdict.slice(0, 12) : '-');
  const mb = cb?.median != null ? cb.median.toPrecision(4) : (cb?.verdict ? cb.verdict.slice(0, 12) : '-');
  let delta = '-';
  if (ca?.median != null && cb?.median != null) {
    delta = (100 * (cb.median - ca.median) / ca.median).toFixed(1) + '%';
  }
  console.log(k.padEnd(28), ga.padEnd(10), String(ma).padEnd(14), gb.padEnd(10), String(mb).padEnd(14), delta);
}
