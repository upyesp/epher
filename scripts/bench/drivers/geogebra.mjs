// GeoGebra web driver (W tier). Surface: the Apps API embed (stage 2 spike,
// pinned deployggb.js from geogebra.org). Boundary per cell: the sum of
// per-evalCommand API latencies measured with in-page performance.now();
// object readbacks (getValue) happen only in the untimed gate pass and
// objects are deleted between chunks outside the timed spans.
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { summarize, bucketSamples } from '../stats.mjs';
import { Cdp, evalJS, chromeVersion } from './cdp.mjs';

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const PORT = 9802;
const APP_HTML = '/tmp/bench2/geogebra-app.html';
const CHUNK = 500;
const GROUP = 24;

export function version() {
  return `GeoGebra web Apps API (deployggb.js current; Chrome for Testing ${chromeVersion()})`;
}

async function readyPage(cdp) {
  const c = await cdp.page(`file://${APP_HTML}`);
  for (let i = 0; i < 90; i++) {
    const s = await evalJS(c, `(() => { try { window.ggbApplet.evalCommand('probeL = {1,2}'); return String(window.ggbApplet.getValue('Mean(probeL)')); } catch(e) { return 'E'; } })()`);
    if (s && !String(s).startsWith('E') && s !== 'NaN' && s !== 'undefined') break;
    await sleep(500);
  }
  return c;
}
import { setTimeout as sleep } from 'node:timers/promises';

// in-page helper: run a batch of commands, timing each with performance.now,
// returning total ms + values read back
const batchJS = (cmds) => evalJS(null, ''); // placeholder, replaced below

async function runChunkTimed(c, assigns) {
  // assigns: array of full command strings; times each evalCommand, sums,
  // returns { ms, cmds }
  return evalJS(c, `(async () => {
    const g = window.ggbApplet;
    const names = ${JSON.stringify(assigns.names)};
    const t0 = performance.now();
    for (const cmd of ${JSON.stringify(assigns.cmds)}) g.evalCommand(cmd);
    const ms = performance.now() - t0;
    return { ms };
  })()`);
}

async function readValues(c, names) {
  return evalJS(c, `(() => {
    const g = window.ggbApplet;
    return ${JSON.stringify(names)}.map(n => {
      const v = g.getValue(n);
      const vs = g.getValueString(n);
      return { v, vs };
    });
  })()`);
}

async function deleteObjects(c, names) {
  await evalJS(c, `(() => { const g = window.ggbApplet; for (const n of ${JSON.stringify(names)}) { try { g.deleteObject(n); } catch(e) {} } return 'ok'; })()`);
}

function groupSums(vars) {
  // grouped checksum command list (24 terms per line) over the given names
  const cmds = [];
  const gnames = [];
  for (let g = 0; g * GROUP < vars.length; g++) {
    const terms = vars.slice(g * GROUP, (g + 1) * GROUP);
    const name = `gsum${g + 1}_${vars.length}`;
    cmds.push(`${name} = ${terms.join(' + ')}`);
    gnames.push(name);
  }
  const total = `gtotal_${vars.length}`;
  cmds.push(`${total} = ${gnames.join(' + ')}`);
  return { cmds, total };
}

export async function benchAll(acceptance, manifestHash, session, only) {
  const cdp = new Cdp(PORT, '/tmp/bench4/profile-ggb');
  await cdp.start();
  const results = [];
  try {
    if (!only || only.includes('w1')) results.push(await benchBatch(cdp, 'w1', acceptance));
    if (!only || only.includes('w2')) results.push(await benchBatch(cdp, 'w2', acceptance));
    if (!only || only.includes('w7')) results.push(await benchW7(cdp, acceptance));
    if (!only || only.includes('w9a')) results.push(await benchW9a(cdp, acceptance));
    if (!only || only.includes('w10')) results.push(await benchW10(cdp));
  } finally {
    cdp.kill();
  }
  return results;
}

function parseNumber(vs, v) {
  // GeoGebra numbers: getValue float, or "a/b"?? getValueString may be
  // rational-like; prefer the float value.
  if (typeof v === 'number' && Number.isFinite(v)) return v;
  const m = String(vs).match(/-?\d+(\.\d+)?([eE][+-]?\d+)?/);
  return m ? parseFloat(m[0]) : NaN;
}

async function benchBatch(cdp, task, acceptance) {
  console.error(`[ggb] benchBatch ${task} | acceptance keys: ${Object.keys(acceptance || {}).slice(0, 8).join(',')} | w1 len: ${acceptance?.w1?.length}`);
  const c = await readyPage(cdp);
  try {
    const dataFile = task === 'w1' ? 'W1-expressions.txt' : 'W2-transcendental-ln.txt';
    const lines = readFileSync(join(root, 'scripts', 'bench', 'data', dataFile), 'utf8').trim().split('\n').map((l) => l.trim());
    const ref = acceptance[task].map(Number);
    const gate = { tolerance: 'chunk checksums: relative 1e-9 vs the 60-digit decimal referee', checked: 0, failed: 0 };
    const chunks = [];
    for (let i = 0; i < lines.length; i += CHUNK) {
      const slice = lines.slice(i, i + CHUNK);
      const names = [];
      const cmds = slice.map((l, k) => {
        names.push(`v${i + k + 1}`);
        return `v${i + k + 1} = (${l})`;
      });
      const { cmds: sums, total } = groupSums(names);
      const want = slice.reduce((a, _, k) => a + ref[i + k], 0);
      chunks.push({ cmds: cmds.concat(sums), names: names.concat(sums.map((s) => s.split(' ')[0])), total, want, start: i, size: slice.length });
    }
    // verification pass: run chunk, read the total, compare, delete objects
    for (const ch of chunks) {
      const r = await runChunkTimed(c, { cmds: ch.cmds });
      if (!r || r.ms === undefined) { gate.failed += ch.size; gate.checked += ch.size; continue; }
      const vals = await readValues(c, [ch.total]);
      gate.checked += ch.size;
      const got = parseNumber(vals[0]?.vs, vals[0]?.v);
      if (!Number.isFinite(got)) { gate.failed += ch.size; }
      else if (!(Math.abs(got - ch.want) / Math.max(1e-300, Math.abs(ch.want)) <= 1e-9)) gate.failed += ch.size;
      await deleteObjects(c, ch.cmds.map((cmd) => cmd.split(' ')[0]));
    }
    const result = { task, label: `${task.toUpperCase()} (GeoGebra web)`, track: 'single', gate, results: [] };
    if (gate.failed > 0) {
      result.results.push({ track: 'single', verdict: 'incorrect: no time published (plan rule 2)' });
      return result;
    }
    const passMs = async () => {
      let total = 0;
      for (const ch of chunks) {
        const r = await runChunkTimed(c, { cmds: ch.cmds });
        if (!r || r.ms === undefined) return null;
        total += r.ms;
        await deleteObjects(c, ch.cmds.map((cmd) => cmd.split(' ')[0]));
      }
      return total;
    };
    const warm = await passMs();
    const bucket = bucketSamples(warm / 1000);
    console.error(`[ggb] ${task}: warm pass ${warm.toFixed(0)} ms -> ${bucket} samples`);
    for (let i = 0; i < 3; i++) await passMs();
    const samples = [];
    for (let i = 0; i < bucket; i++) {
      const ms = await passMs();
      console.error(`[ggb] ${task}: pass ${i + 1}/${bucket} = ${ms?.toFixed(0)} ms`);
      if (ms == null) break;
      samples.push(ms);
    }
    result.results.push({
      track: 'single',
      boundary: `Apps API ingest: sum of per-evalCommand latencies (in-page performance.now) over ${CHUNK}-command chunks; chunk total read via getValue only in the untimed gate pass; object deletion outside timed spans`,
      warmups: 3,
      requested_samples: bucket,
      bucket_rule: `warm-up pass ${warm.toFixed(0)} ms -> ${bucket} samples`,
      units: 'ms',
      ...summarize(samples),
      raw_passes_ms: samples,
    });
    return result;
  } finally {
    await c.close();
  }
}

async function benchW7(cdp, acceptance) {
  const c = await readyPage(cdp);
  try {
    const lines = readFileSync(join(root, 'scripts', 'bench', 'data', 'W7-complex.txt'), 'utf8').trim().split('\n');
    const flat = [];
    for (const pair of acceptance.w7.values) flat.push(pair[0], pair[1]);
    const gate = { tolerance: 'chunk checksums of re/im parts: absolute 1e-9 scaled by |sum| vs float64 referee', checked: 0, failed: 0 };
    const chunks = [];
    for (let i = 0; i < lines.length; i += CHUNK) {
      const slice = lines.slice(i, i + CHUNK);
      const names = [];
      const cmds = [];
      slice.forEach((l, k) => {
        names.push(`zr${i + k + 1}`);
        cmds.push(`zr${i + k + 1} = real((${l}))`);
        names.push(`zi${i + k + 1}`);
        cmds.push(`zi${i + k + 1} = imaginary((${l}))`);
      });
      const { cmds: sums, total } = groupSums(names);
      // total sums re and im interleaved; referee checksum over the same order
      const want = slice.reduce((a, _, k) => a + flat[(i + k) * 2] + flat[(i + k) * 2 + 1], 0);
      chunks.push({ cmds: cmds.concat(sums), names, total, want, size: slice.length });
    }
    for (const ch of chunks) {
      await runChunkTimed(c, { cmds: ch.cmds });
      const vals = await readValues(c, [ch.total]);
      gate.checked += ch.size;
      const got = parseNumber(vals[0]?.vs, vals[0]?.v);
      if (!Number.isFinite(got) || Math.abs(got - ch.want) > 1e-9 * Math.max(1, Math.abs(ch.want))) gate.failed += ch.size;
      await deleteObjects(c, ch.cmds.map((cmd) => cmd.split(' ')[0]));
    }
    const result = { task: 'w7', label: 'W7 complex arithmetic (GeoGebra web)', track: 'single', gate, results: [] };
    if (gate.failed > 0) {
      result.results.push({ track: 'single', verdict: 'incorrect: no time published (plan rule 2)' });
      return result;
    }
    const passMs = async () => {
      let total = 0;
      for (const ch of chunks) {
        const r = await runChunkTimed(c, { cmds: ch.cmds });
        if (!r || r.ms === undefined) return null;
        total += r.ms;
        await deleteObjects(c, ch.cmds.map((cmd) => cmd.split(' ')[0]));
      }
      return total;
    };
    const warm = await passMs();
    const bucket = bucketSamples(warm / 1000);
    for (let i = 0; i < 3; i++) await passMs();
    const samples = [];
    for (let i = 0; i < bucket; i++) {
      const ms = await passMs();
      if (ms == null) break;
      samples.push(ms);
    }
    result.results.push({
      track: 'single',
      boundary: 'Apps API: sum of per-evalCommand latencies; real()/imaginary() per expression; checksum gate via getValue in the untimed pass',
      warmups: 3,
      requested_samples: bucket,
      bucket_rule: `warm-up pass ${warm.toFixed(0)} ms -> ${bucket} samples`,
      units: 'ms',
      ...summarize(samples),
      raw_passes_ms: samples,
    });
    return result;
  } finally {
    await c.close();
  }
}

async function benchW9a(cdp, acceptance) {
  const c = await readyPage(cdp);
  try {
    const want = acceptance.w9.w9a.count;
    const cmd = `s9a = Sum(Sequence(If(IsPrime(k), 1, 0), k, 2, 9999))`;
    const r = await evalJS(c, `(() => {
      const g = window.ggbApplet;
      const t0 = performance.now();
      g.evalCommand(${JSON.stringify(cmd)});
      return { ms: performance.now() - t0, v: g.getValue('s9a') };
    })()`);
    const gate = { tolerance: 'exact prime count vs referee', checked: 1, failed: r?.v !== want ? 1 : 0, note: `count=${r?.v}` };
    const result = { task: 'w9a', label: 'W9a prime count (GeoGebra web)', track: 'idiomatic', gate, results: [] };
    if (gate.failed > 0) {
      result.results.push({ track: 'idiomatic', track_note: 'Sum+Sequence+IsPrime (GeoGebra native list programming)', verdict: 'incorrect: no time published (plan rule 2)' });
      return result;
    }
    const samples = [];
    for (let i = 0; i < 3; i++) await evalJS(c, `window.ggbApplet.deleteObject('s9a'); window.ggbApplet.evalCommand(${JSON.stringify(cmd)});`);
    for (let i = 0; i < 30; i++) {
      const t = await evalJS(c, `(() => {
        const g = window.ggbApplet;
        g.deleteObject('s9a');
        const t0 = performance.now();
        g.evalCommand(${JSON.stringify(cmd)});
        return performance.now() - t0;
      })()`);
      samples.push(t);
    }
    result.results.push({
      track: 'idiomatic',
      track_note: 'Sum(Sequence(If(IsPrime(k),1,0),k,2,9999)) - GeoGebra native list programming; the flag-array sieve in a loop is not expressible in GeoGebra scripting',
      boundary: 'Apps API: single evalCommand latency (in-page performance.now)',
      warmups: 3,
      requested_samples: 30,
      units: 'ms',
      ...summarize(samples),
      raw_passes_ms: samples,
    });
    return result;
  } finally {
    await c.close();
  }
}

async function benchW10(cdp) {
  const result = { task: 'w10', label: 'W10 interactive latency (GeoGebra web)', track: 'single', gate: { checked: 1, failed: 0 }, results: [] };
  const samples = [];
  for (let i = 0; i < 10; i++) {
    const c = await cdp.page(`file://${APP_HTML}`);
    const ms = await evalJS(c, `(async () => {
      const t0 = performance.now();
      for (let k = 0; k < 3000; k++) {
        await new Promise(r => setTimeout(r, 10));
        if (window.ggbApplet && window.ggbApplet.evalCommand) {
          ggbApplet.evalCommand('w10a = 6*7');
          const v = ggbApplet.getValue('w10a');
          if (v === 42) return performance.now() - t0;
        }
      }
      return -1;
    })()`);
    await c.close();
    if (ms >= 0) samples.push(ms);
  }
  result.results.push({
    track: 'single',
    boundary: 'launch-to-ready: page load -> applet API ready -> first evalCommand verified (in-page performance.now)',
    warmups: 0,
    requested_samples: 10,
    launch_to_ready_ms: summarize(samples),
    launch_samples: samples.length,
    keystroke_to_answer: 'not measurable on the Apps API embed (no focused text input in this embed configuration); dropped per the plan reliability caveat',
  });
  return result;
}
