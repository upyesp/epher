// Desmos web driver (W tier). Surface: the public calculator page's
// window.Calc API (stage 2 spikes confirmed setExpression +
// HelperExpression numeric readback + regression helpers + integral
// helpers). Boundary per cell: per-expression span = setExpression ->
// HelperExpression numericValue arrival (in-page performance.now);
// chunked, expressions removed between chunks outside the timed spans.
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { summarize, bucketSamples } from '../stats.mjs';
import { Cdp, evalJS, chromeVersion } from './cdp.mjs';
import { setTimeout as sleep } from 'node:timers/promises';

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const PORT = 9803;
const CALC = 'https://www.desmos.com/calculator';
const CHUNK = 100;

export function version() {
  return `Desmos web (public calculator page + window.Calc API; Chrome for Testing ${chromeVersion()})`;
}

// bare function names in desmos latex parse as implicit multiplication
// (s*i*n*(...)); call syntax must be converted to latex form
const FN_TO_LATEX = (s) => s.replace(/\b(sin|cos|tan|asin|acos|atan|arcsin|arccos|arctan|sinh|cosh|tanh|ln|log|exp|sqrt|abs|floor|ceil|round|mod|sign)\(/g, "\\$1(");

// every page call carries the runner: a context reload between evals
// silently drops window.__bench (observed 2026-11-09: w2 chunks lost all
// spans while w5 on a fresh page worked)
const withRunner = (expr) => RUNNER + "\n" + expr;

async function readyPage(cdp) {
  const c = await cdp.page(CALC);
  for (let i = 0; i < 60; i++) {
    const ok = await evalJS(c, `typeof window.Calc === 'object' && typeof Calc.setExpression === 'function'`);
    if (ok === true) return c;
    await sleep(500);
  }
  // a broken tab (stale/overloaded chrome) must fail the cell, not
  // benchmark nothing (observed 2026-11-09: w2 gate recorded 10000
  // silent failures against a tab whose calculator app never booted)
  throw new Error('desmos calculator app did not boot within 30s');
}

// in-page helper factory (string): chunked run with per-expression spans
const RUNNER = `
window.__bench = async (latexList, timeoutMs) => {
  // Helper-only evaluation: the HelperExpression IS the API's evaluation
  // path (assignments do not produce helper numericValues, probed
  // 2026-11-08). Each span = helper construction -> numericValue arrival.
  const out = { spans: [], values: [] };
  for (const latex of latexList) {
    const t0 = performance.now();
    const h = Calc.HelperExpression({ latex });
    const v = await new Promise((res) => {
      const check = () => {
        const v = h.numericValue;
        if (typeof v === 'number' && !Number.isNaN(v)) { res(v); return true; }
        return false;
      };
      if (check()) return;
      h.observe('numericValue', () => { if (check()) h.unobserve('numericValue'); });
      setTimeout(() => res(NaN), timeoutMs || 4000);
    });
    out.spans.push(performance.now() - t0);
    out.values.push(v);
    try { h.unobserve('numericValue'); } catch (err) {}
  }
  return out;
};
window.__clear = (first, last) => {
  const exprs = Calc.getExpressions().filter(e => e.id && e.latex);
  for (const e of exprs) { try { Calc.removeExpressions([e]); } catch (err) {} }
  return exprs.length;
};
`;

export async function benchAll(acceptance, manifestHash, session, only) {
  const cdp = new Cdp(PORT, '/tmp/bench4/profile-desmos3');
  await cdp.start();
  const results = [];
  try {
    if (!only || only.includes('w1')) results.push(await benchBatch(cdp, 'w1', acceptance));
    if (!only || only.includes('w2')) results.push(await benchBatch(cdp, 'w2', acceptance));
    if (!only || only.includes('w4')) results.push(await benchW4(cdp, acceptance));
    if (!only || only.includes('w5-integrals')) results.push(await benchW5int(cdp, acceptance));
    if (!only || only.includes('w10')) results.push(await benchW10(cdp));
  } finally {
    cdp.kill();
  }
  return results;
}

async function benchBatch(cdp, task, acceptance) {
  const c = await readyPage(cdp);
  try {
    await evalJS(c, RUNNER);
    const dataFile = task === 'w1' ? 'W1-expressions.txt' : 'W2-transcendental-ln.txt';
    const lines = readFileSync(join(root, 'scripts', 'bench', 'data', dataFile), 'utf8').trim().split('\n').map((l) => l.trim());
    const ref = acceptance[task].map(Number);
    const gate = { tolerance: 'per-expression HelperExpression value vs referee, relative 1e-9', checked: 0, failed: 0 };
    const chunks = [];
    for (let i = 0; i < lines.length; i += CHUNK) {
      const slice = lines.slice(i, i + CHUNK);
      // bare constant-only helpers (e.g. \sin(1.413)) never produce a
      // numericValue in the current desmos API; wrapping any expression in
      // \frac{...}{1} forces a full evaluation tree (probed 2026-11-08:
      // bare sin/ln time out, \frac{\sin(1.413)}{1} returns the exact value)
      const latex = slice.map((l) => `\\frac{(${FN_TO_LATEX(l)})}{1}`);
      const wants = slice.map((_, k) => ref[i + k]);
      chunks.push({ latex, wants, start: i });
    }
    let totalMs = 0;
    let chunkIdx = 0;
    for (const ch of chunks) {
      console.error(`[desmos] ${task}: gate chunk ${chunkIdx++}/${chunks.length}`);
      const r = await evalJS(c, withRunner(`__bench(${JSON.stringify(ch.latex)}, 6000)`), 120000);
      if (!r || !r.spans) { gate.failed += ch.wants.length; gate.checked += ch.wants.length; continue; }
      totalMs += r.spans.reduce((a, b) => a + b, 0);
      for (let k = 0; k < ch.wants.length; k++) {
        gate.checked++;
        const got = r.values[k];
        const want = ch.wants[k];
        if (typeof got !== 'number' || Number.isNaN(got)) { gate.failed++; continue; }
        if (want === 0 ? Math.abs(got) > 1e-9 : !(Math.abs(got - want) / Math.abs(want) <= 1e-9)) gate.failed++;
      }
      await evalJS(c, withRunner(`__clear()`));
      await sleep(150);
    }
    const result = { task, label: `${task.toUpperCase()} (Desmos web)`, track: 'single', gate, results: [] };
    if (gate.failed > 0) {
      result.results.push({ track: 'single', verdict: 'incorrect: no time published (plan rule 2)', note: `gate wall time ${Math.round(totalMs)} ms` });
      return result;
    }
    // timed passes: re-run all chunks fresh (clear between)
    // a fresh page per pass: 10k setExpression+removal cycles degrade a
    // long-lived calculator page (measured 726 s warm vs 2273 s on the
    // second pass, 2026-11-08 - undo/state bloat). The reload cost sits
    // outside the per-expression spans (same declared boundary).
    const passMs = async () => {
      const pc = await readyPage(cdp);
      try {
        await evalJS(pc, RUNNER);
        let total = 0;
        for (const ch of chunks) {
          const r = await evalJS(pc, withRunner(`__bench(${JSON.stringify(ch.latex)}, 6000)`), 120000);
          if (!r || !r.spans) return null;
          total += r.spans.reduce((a, b) => a + b, 0);
          await evalJS(pc, withRunner(`__clear()`));
          await sleep(120);
        }
        return total;
      } finally {
        await pc.close();
      }
    };
    const warm = await passMs();
    const bucket = bucketSamples(warm / 1000);
    console.error(`[desmos] ${task}: warm pass ${warm.toFixed(0)} ms -> ${bucket} samples`);
    for (let i = 0; i < 3; i++) await passMs();
    const samples = [];
    for (let i = 0; i < bucket; i++) {
      const ms = await passMs();
      console.error(`[desmos] ${task}: pass ${i + 1}/${bucket} = ${ms?.toFixed(0)} ms`);
      if (ms == null) break;
      samples.push(ms);
    }
    result.results.push({
      track: 'single',
      track_note: 'idiomatic and equivalent coincide by construction (W1/W2 fix the expressions)',
      boundary: `per-expression span = setExpression -> HelperExpression numericValue arrival (in-page performance.now); ${CHUNK}-expression chunks, removals and page housekeeping outside the spans; spans summed`,
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

async function benchW4(cdp, acceptance) {
  const c = await readyPage(cdp);
  try {
    await evalJS(c, RUNNER);
    const ys = readFileSync(join(root, 'scripts', 'bench', 'data', 'W4-samples.txt'), 'utf8').trim().split('\n').map(Number);
    const acc = acceptance.w4;
    // one list expression (10k numbers) + helper readbacks
    const listLatex = `D=[${ys.join(',')}]`;
    const r = await evalJS(c, `(async () => {
      const t0 = performance.now();
      Calc.setExpression({ latex: ${JSON.stringify(listLatex)} });
      const setMs = performance.now() - t0;
      const read = (latex) => new Promise((res) => {
        const h = Calc.HelperExpression({ latex });
        const f = () => { const v = h.numericValue; if (typeof v === 'number' && !Number.isNaN(v)) { h.unobserve('numericValue', f); res(v); } };
        h.observe('numericValue', f);
        setTimeout(() => res(NaN), 8000);
      });
      await new Promise(r2 => setTimeout(r2, 2000));
      const mean = await read('\\mean(D)');
      const sd = await read('\\stdev(D)');
      const median = await read('\\median(D)');
      const p10 = await read('\\sort(D)[\\ceil(0.1*(\\size(D)-1)+1)] + (0.1*(\\size(D)-1)+1 - \\floor(0.1*(\\size(D)-1)+1))*(\\sort(D)[\\floor(0.1*(\\size(D)-1)+1)+1]-\\sort(D)[\\floor(0.1*(\\size(D)-1)+1)+0])');
      return { setMs, mean, sd, median, p10 };
    })()`, 120000);
    const gate = { tolerance: 'relative 1e-9 vs referee for mean/stdev/median (Type-7 percentiles and linreg not expressible in Desmos functions: not supported sub-parts, declared)', checked: 3, failed: 0 };
    for (const [got, want] of [[r.mean, acc.mean], [r.sd, acc.stdev], [r.median, acc.median]]) {
      if (typeof got !== 'number' || Number.isNaN(got) || !(Math.abs(got - want) / Math.abs(want) <= 1e-9)) gate.failed++;
    }
    const result = { task: 'w4', label: 'W4 statistics (Desmos web)', track: 'single', gate, results: [] };
    if (gate.failed > 0) {
      result.results.push({ track: 'single', verdict: 'incorrect: no time published (plan rule 2)', note: `mean=${r.mean} sd=${r.sd} median=${r.median}` });
      return result;
    }
    // timed pass: set list + three stat helper arrivals, repeated
    const samples = [];
    for (let i = 0; i < 3; i++) {
      await evalJS(c, withRunner(`__clear()`));
      await sleep(200);
    }
    for (let i = 0; i < 10; i++) {
      await evalJS(c, withRunner(`__clear()`));
      await sleep(200);
      const t = await evalJS(c, `(async () => {
        const t0 = performance.now();
        Calc.setExpression({ latex: ${JSON.stringify(listLatex)} });
        const read = (latex) => new Promise((res) => {
          const h = Calc.HelperExpression({ latex });
          const f = () => { const v = h.numericValue; if (typeof v === 'number' && !Number.isNaN(v)) { h.unobserve('numericValue', f); res(v); } };
          h.observe('numericValue', f);
          setTimeout(() => res(NaN), 8000);
        });
        await new Promise(r2 => setTimeout(r2, 400));
        const a = await read('\\mean(D)');
        const b = await read('\\stdev(D)');
        const m = await read('\\median(D)');
        return performance.now() - t0;
      })()`, 120000);
      if (typeof t === 'number' && t > 0) samples.push(t);
    }
    result.results.push({
      track: 'single',
      boundary: 'list setExpression + mean/stdev/median helper value arrival (in-page performance.now); 10 samples (spec: percentiles Type-7 and linreg are not expressible in Desmos functions, so the full W4 spec is only partially expressible and no cross-tool time claim is made)',
      warmups: 3,
      requested_samples: 10,
      units: 'ms',
      ...summarize(samples),
      raw_passes_ms: samples,
    });
    return result;
  } finally {
    await c.close();
  }
}

async function benchW5int(cdp, acceptance) {
  const c = await readyPage(cdp);
  try {
    await evalJS(c, RUNNER);
    const I = acceptance.w5.integrals;
    const gate = { tolerance: 'relative 1e-9 vs the float64 adaptive-Simpson referee (root finds are not expressible in Desmos: not supported, declared)', checked: 3, failed: 0 };
    const latexes = [
      '\\int_{0}^{1}{e^{x^{2}} \\cdot e^{-2x^{2}} dx}',
      '\\int_{0}^{1}{e^{-x^{2}}dx}',
      '\\int_{0}^{\\pi}{\\sin\\left(x\\right) \\cdot e^{-x/3} dx}',
      '\\int_{1}^{2}{\\ln\\left(x\\right) / x dx}',
    ];
    const read = (latex) => withRunner(`__bench([${JSON.stringify(latex)}], 10000)`);
    const vals = {};
    for (const [key, latex] of [['I1', latexes[1]], ['I2', latexes[2]], ['I3', latexes[3]]]) {
      const r = await evalJS(c, read(latex));
      vals[key] = r?.values?.[0];
      if (typeof vals[key] !== 'number' || Number.isNaN(vals[key]) || !(Math.abs(vals[key] - I[key]) / Math.abs(I[key]) <= 1e-9)) gate.failed++;
      await evalJS(c, withRunner(`__clear()`));
      await sleep(150);
    }
    const result = { task: 'w5-integrals', label: 'W5 integrals (Desmos web)', track: 'single', gate, results: [] };
    if (gate.failed > 0) {
      result.results.push({ track: 'single', verdict: 'incorrect: no time published (plan rule 2)', note: JSON.stringify(vals) });
      return result;
    }
    const samples = [];
    for (let i = 0; i < 3; i++) {
      for (const latex of [latexes[1], latexes[2], latexes[3]]) {
        await evalJS(c, read(latex));
        await evalJS(c, withRunner(`__clear()`));
      }
    }
    for (let i = 0; i < 10; i++) {
      const t0 = Date.now();
      let total = 0;
      let ok = true;
      for (const [key, latex] of [['I1', latexes[1]], ['I2', latexes[2]], ['I3', latexes[3]]]) {
        const r = await evalJS(c, read(latex));
        if (!r?.spans) { ok = false; break; }
        total += r.spans[0];
        await evalJS(c, withRunner(`__clear()`));
        await sleep(100);
      }
      if (ok) samples.push(total);
    }
    result.results.push({
      track: 'single',
      boundary: 'per-integral span = setExpression(integral) -> HelperExpression value arrival (in-page performance.now); three fixed integrals per pass; removals outside spans',
      warmups: 3,
      requested_samples: 10,
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
  const result = { task: 'w10', label: 'W10 interactive latency (Desmos web)', track: 'single', gate: { checked: 1, failed: 0 }, results: [] };
  const samples = [];
  for (let i = 0; i < 8; i++) {
    const c = await cdp.page(CALC);
    const ms = await evalJS(c, `(async () => {
      const t0 = performance.now();
      for (let k = 0; k < 3000; k++) {
        await new Promise(r => setTimeout(r, 20));
        if (window.Calc && Calc.setExpression) {
          Calc.setExpression({ latex: '6*7' });
          for (let q = 0; q < 200; q++) {
            await new Promise(r => setTimeout(r, 10));
            const exprs = Calc.getExpressions();
            if (exprs.length && exprs[0].latex) {
              const h = Calc.HelperExpression({ latex: '6*7' });
              const v = h.numericValue;
              if (typeof v === 'number' && !Number.isNaN(v)) return performance.now() - t0;
            }
          }
        }
      }
      return -1;
    })()`, 120000);
    await c.close();
    if (ms >= 0) samples.push(ms);
    await sleep(300);
  }
  result.results.push({
    track: 'single',
    boundary: 'launch-to-ready: page load -> Calc API -> first expression value (in-page performance.now); keystroke-to-answer dropped: the mathquill input could not be driven reliably in stage 2 spikes (plan caveat)',
    warmups: 0,
    requested_samples: 8,
    launch_to_ready_ms: summarize(samples),
    launch_samples: samples.length,
  });
  return result;
}
