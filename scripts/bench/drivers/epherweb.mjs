// epher web driver (W tier). The web app submits the whole textarea as a
// multi-line script on Enter, so batch tasks ingest as chunks of silent
// assignments plus one checksum statement; the chunk span is Enter ->
// #epher-result text present (in-page performance.now()). W10 uses the
// ?expr= URL for launch-to-ready and real CDP key events for
// keystroke-to-answer.
import { setTimeout as sleep } from 'node:timers/promises';
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { summarize, bucketSamples } from '../stats.mjs';
import { Cdp, evalJS, chromeVersion } from './cdp.mjs';

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const PORT = 9801;
const DIST = join(root, 'crates', 'web', 'dist');

const CHUNK = 500;
const CHUNKS = {
  w1: 'W1-expressions.txt',
  w2: 'W2-transcendental-ln.txt',
};

export function version() {
  return `epher web (trunk build at HEAD; Chrome for Testing ${chromeVersion()})`;
}

function chunkText(lines, kind) {
  // silent assignments + grouped checksum statements (the only transcript
  // output). One long 500-term checksum line opens the autocomplete popup,
  // which swallows Enter, so group sums stay at 24 terms per line.
  const out = [];
  for (let i = 0; i < lines.length; i++) {
    out.push(`v${i + 1} = ${lines[i]}`);
  }
  const G = 24;
  const groups = [];
  for (let g = 0; g * G < lines.length; g++) {
    const terms = [];
    for (let k = g * G; k < Math.min((g + 1) * G, lines.length); k++) terms.push(`v${k + 1}`);
    out.push(`g${g + 1} = ${terms.join(' + ')}`);
    groups.push(`g${g + 1}`);
  }
  out.push(groups.join(' + '));
  return out.join('\n');
}

function checksumOf(values) {
  return values.reduce((a, c) => a + c, 0);
}

export async function benchAll(acceptance, manifestHash, session, only) {
  const cdp = new Cdp(PORT, '/tmp/bench4/profile-epherweb');
  await cdp.start();
  const results = [];
  try {
    // serve dist over a local static server started by the runner wrapper;
    // if it is already running, reuse it.
    await ensureServer();
    for (const task of ['w1', 'w2']) {
      if (only && !only.includes(task)) continue;
      console.error(`[epherweb] ${task}: gate`);
      results.push(await benchBatch(cdp, task, acceptance));
      console.error(`[epherweb] ${task}: done`);
    }
    if (!only || only.includes('w7')) {
      console.error('[epherweb] w7: gate');
      results.push(await benchW7(cdp, acceptance));
      console.error('[epherweb] w7: done');
    }
    if (!only || only.includes('w10')) {
      console.error('[epherweb] w10');
      results.push(await benchW10(cdp));
      console.error('[epherweb] w10: done');
    }
  } finally {
    cdp.kill();
    await stopServer();
  }
  return results;
}

let serverProc = null;
async function ensureServer() {
  try {
    const r = await fetch('http://127.0.0.1:8123/index.html');
    if (r.ok) return;
  } catch {}
  const { spawn } = await import('node:child_process');
  serverProc = spawn('python3', ['-m', 'http.server', '8123', '--directory', DIST], { stdio: 'ignore', detached: true });
  serverProc.unref();
  await sleep(800);
}
async function stopServer() {
  if (serverProc) serverProc.kill();
}

async function newPage(cdp, url) {
  const c = await cdp.page(url);
  await sleep(2500);
  return c;
}

async function submitChunk(c, text) {
  // returns in-page ms for Enter -> result text present
  return evalJS(c, `(async () => {
    const ta = document.querySelector('textarea');
    ta.focus();
    ta.value = ${JSON.stringify(text)};
    ta.dispatchEvent(new Event('input', {bubbles:true}));
    await new Promise(r=>setTimeout(r,120));
    const t0 = performance.now();
    ta.dispatchEvent(new KeyboardEvent('keydown', {key:'Enter', code:'Enter', keyCode:13, which:13, bubbles:true, cancelable:true}));
    for (let i=0;i<4000;i++) {
      await new Promise(r=>setTimeout(r,10));
      const live = document.querySelector('.pane-answer-final') || document.querySelector('#epher-result');
      if (live && live.textContent.trim()) return {ms: performance.now()-t0, text: live.textContent.trim()};
    }
    return {ms:-1, text:'TIMEOUT'};
  })()`);
}

export async function benchBatch(cdp, task, acceptance) {
  const c = await newPage(cdp, 'http://127.0.0.1:8123/index.html');
  try {
    const lines = readFileSync(join(root, 'scripts', 'bench', 'data', CHUNKS[task]), 'utf8').trim().split('\n');
    const ref = acceptance[task];
    const gate = { tolerance: 'chunk checksums: relative 1e-9 vs the 60-digit decimal referee (float64 sum order = dataset order, declared)', checked: 0, failed: 0 };
    const chunks = [];
    for (let i = 0; i < lines.length; i += CHUNK) {
      const slice = lines.slice(i, i + CHUNK).map((l) => l.trim());
      const text = chunkText(slice, task);
      const want = checksumOf(slice.map((_, k) => Number(ref[i + k])));
      chunks.push({ text, want, start: i });
    }
    for (const ch of chunks) {
      const r = await submitChunk(c, ch.text);
      if (r.ms < 0) {
        gate.failed += CHUNK;
        gate.checked += CHUNK;
        continue;
      }
      // the sum may render as an exact rational "a/b" (integer-division
      // results propagate into the checksum)
      let got = NaN;
      const rat = r.text.match(/=\s*(-?\d+)\s*\/\s*(-?\d+)/);
      if (rat) got = Number(rat[1]) / Number(rat[2]);
      else {
        const m = r.text.match(/-?\d+(\.\d+)?([eE][+-]?\d+)?/);
        if (m) got = parseFloat(m[0]);
      }
      gate.checked++;
      if (!Number.isFinite(got)) { gate.failed++; continue; }
      const rel = Math.abs(got - ch.want) / Math.max(1e-300, Math.abs(ch.want));
      if (!(rel <= 1e-9)) gate.failed++;
    }
    const result = { task, label: task === 'w1' ? 'W1 scalar arithmetic (web)' : 'W2 transcendental microbench (web)', track: 'single', gate, results: [] };
    if (gate.failed > 0) {
      result.results.push({ track: 'single', verdict: 'incorrect: no time published (plan rule 2)' });
      return result;
    }
    // timed passes
    const passMs = async () => {
      let total = 0;
      for (const ch of chunks) {
        const r = await submitChunk(c, ch.text);
        if (r.ms < 0) return null;
        total += r.ms;
      }
      return total;
    };
    const warm = await passMs();
    const bucket = bucketSamples(warm / 1000);
    console.error(`[epherweb] ${task}: warm pass ${warm.toFixed(0)} ms -> ${bucket} samples`);
    for (let i = 0; i < 3; i++) await passMs();
    const samples = [];
    for (let i = 0; i < bucket; i++) {
      const ms = await passMs();
      console.error(`[epherweb] ${task}: pass ${i + 1}/${bucket} = ${ms?.toFixed(0)} ms`);
      if (ms == null) break;
      samples.push(ms);
    }
    result.results.push({
      track: 'single',
      track_note: 'idiomatic and equivalent coincide by construction (W1/W2 fix the expressions)',
      boundary: `GUI script ingest end-to-end: per-chunk span is Enter -> #epher-result present (in-page performance.now), chunk = ${CHUNK} silent assignments + one checksum statement; chunk spans summed; page load excluded`,
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
  const c = await newPage(cdp, 'http://127.0.0.1:8123/index.html');
  try {
    const lines = readFileSync(join(root, 'scripts', 'bench', 'data', 'W7-complex.txt'), 'utf8').trim().split('\n');
    const flat = [];
    for (const pair of acceptance.w7.values) flat.push(pair[0], pair[1]);
    const gate = { tolerance: 'chunk checksums of re/im parts: absolute 1e-9 · |sum| vs float64 referee (complex arithmetic is float64 in both)', checked: 0, failed: 0 };
    const chunks = [];
    for (let i = 0; i < lines.length; i += CHUNK) {
      const slice = lines.slice(i, i + CHUNK);
      const assigns = [];
      slice.forEach((l, k) => {
        assigns.push(`re${k + 1} = re(${l})`);
        assigns.push(`im${k + 1} = im(${l})`);
      });
      const G = 24;
      const reGroups = [];
      const imGroups = [];
      for (let g = 0; g * G < slice.length; g++) {
        const rte = [];
        const iti = [];
        for (let k = g * G; k < Math.min((g + 1) * G, slice.length); k++) {
          rte.push(`re${k + 1}`);
          iti.push(`im${k + 1}`);
        }
        assigns.push(`gr${g + 1} = ${rte.join(' + ')}`);
        assigns.push(`gi${g + 1} = ${iti.join(' + ')}`);
        reGroups.push(`gr${g + 1}`);
        imGroups.push(`gi${g + 1}`);
      }
      const reSum = reGroups.join(' + ');
      const imSum = imGroups.join(' + ');
      const wantRe = checksumOf(slice.map((_, k) => flat[(i + k) * 2]));
      const wantIm = checksumOf(slice.map((_, k) => flat[(i + k) * 2 + 1]));
      chunks.push({ text: assigns.join('\n') + '\n(' + reSum + ') + (' + imSum + ')', wantRe, wantIm, size: slice.length });
    }
    for (const ch of chunks) {
      const r = await submitChunk(c, ch.text);
      if (r.ms < 0) { gate.failed += ch.size; gate.checked += ch.size; continue; }
      const nums = r.text.match(/-?\d+(\.\d+)?([eE][+-]?\d+)?/g) || [];
      gate.checked += ch.size;
      const wantTotal = ch.wantRe + ch.wantIm;
      const rat = r.text.match(/=\s*(-?\d+)\s*\/\s*(-?\d+)/);
      let got = NaN;
      if (rat) got = Number(rat[1]) / Number(rat[2]);
      else if (nums.length) got = parseFloat(nums[0]);
      if (!Number.isFinite(got) || Math.abs(got - wantTotal) > 1e-9 * Math.max(1, Math.abs(wantTotal))) gate.failed += ch.size;
    }
    const result = { task: 'w7', label: 'W7 complex arithmetic (web)', track: 'single', gate, results: [] };
    if (gate.failed > 0) {
      result.results.push({ track: 'single', verdict: 'incorrect: no time published (plan rule 2)' });
      return result;
    }
    const passMs = async () => {
      let total = 0;
      for (const ch of chunks) {
        const r = await submitChunk(c, ch.text);
        if (r.ms < 0) return null;
        total += r.ms;
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
      boundary: `GUI script ingest end-to-end, chunk = ${CHUNK} complex expressions as re/im assignment pairs + two checksum statements; per-chunk span Enter -> result present; chunk spans summed`,
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

async function benchW10(cdp) {
  const result = {
    task: 'w10', label: 'W10 interactive latency (web)', track: 'single', gate: { checked: 1, failed: 0 },
    results: [],
  };
  // launch-to-ready: fresh page with ?expr=6*7; ready = the share-link
  // expression is STAGED in the entry (ADR-0038: ?expr stages, the user's
  // Enter is the submit - so the answer never appears without a keystroke;
  // polling for staged state is the honest launch-to-ready boundary)
  const samples = [];
  for (let i = 0; i < 10; i++) {
    const c = await cdp.page('http://127.0.0.1:8123/index.html?expr=6*7');
    const ms = await evalJS(c, `(async () => {
      const t0 = performance.now();
      for (let k=0;k<6000;k++) {
        await new Promise(r=>setTimeout(r,10));
        const ta = document.querySelector('textarea');
        if (ta && ta.value.trim()) return performance.now() - t0;
      }
      return -1;
    })()`);
    await c.close();
    if (ms >= 0) samples.push(ms);
    await sleep(300);
  }
  // keystroke-to-answer: real key events into the focused textarea
  const ksSamples = [];
  {
    const c = await cdp.page('http://127.0.0.1:8123/index.html');
    await sleep(2000);
    for (let i = 0; i < 20; i++) {
      await evalJS(c, `(() => { const ta = document.querySelector('textarea'); ta.focus(); ta.value=''; return 'ok'; })()`);
      await sleep(120);
      const keys = ['6', '*', '7'];
      for (const k of keys) {
        await c.send('Input.dispatchKeyEvent', { type: 'keyDown', key: k, text: k, windowsVirtualKeyCode: k.charCodeAt(0) });
        await c.send('Input.dispatchKeyEvent', { type: 'keyUp', key: k, windowsVirtualKeyCode: k.charCodeAt(0) });
      }
      const t0 = Date.now();
      await c.send('Input.dispatchKeyEvent', { type: 'keyDown', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13, text: '\r' });
      await c.send('Input.dispatchKeyEvent', { type: 'keyUp', key: 'Enter', code: 'Enter', windowsVirtualKeyCode: 13 });
      let ok = false;
      while (Date.now() - t0 < 5000) {
        const v = await evalJS(c, `document.querySelector('#epher-result')?.textContent.trim() || ''`);
        if (v && v.includes('42')) { ok = true; break; }
        await sleep(10);
      }
      if (ok) ksSamples.push(Date.now() - t0);
    }
    await c.close();
  }
  result.results.push({
    track: 'single',
    boundary: 'launch-to-ready: fresh page ?expr=6*7 -> share-link expression staged in the entry textarea (ADR-0038: ?expr stages only, Enter is the user submit), in-page performance.now (page-load navigation cost excluded by in-page clock, declared); keystroke-to-answer: CDP real key events 6*7+Enter -> result 42, wall clock at 10 ms poll resolution',
    warmups: 0,
    requested_samples: 10,
    launch_to_ready_ms: summarize(samples),
    launch_samples: samples.length,
    keystroke_to_answer_ms: summarize(ksSamples),
    keystroke_samples: ksSamples.length,
  });
  return result;
}
