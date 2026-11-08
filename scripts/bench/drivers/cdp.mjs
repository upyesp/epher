// Shared headless-Chrome CDP helper for the W-tier drivers (no deps:
// Node >= 22 global WebSocket + fetch). One Chrome per driver process,
// a dedicated profile per tool, timing taken inside the page via
// performance.now() wherever the plan allows it.
import { spawn } from 'node:child_process';
import { setTimeout as sleep } from 'node:timers/promises';
import { readdirSync, existsSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

export function findChrome() {
  const root = join(homedir(), '.cache', 'puppeteer', 'chrome');
  for (const dir of readdirSync(root).sort().reverse()) {
    const p = join(root, dir, 'chrome-mac-arm64', 'Google Chrome for Testing.app', 'Contents', 'MacOS', 'Google Chrome for Testing');
    if (existsSync(p)) return p;
  }
  throw new Error('no chrome for testing found');
}

export function chromeVersion() {
  const root = join(homedir(), '.cache', 'puppeteer', 'chrome');
  return readdirSync(root).sort().reverse()[0].replace('mac_arm-', 'mac-arm-');
}

export class Cdp {
  constructor(port, profile) {
    this.port = port;
    this.profile = profile;
    this.chrome = null;
    this.id = 0;
    this.pending = new Map();
  }

  async start() {
    // unique profile per process: two drivers sharing a profile dir would
    // fight over the SingletonLock (observed when a smoke overlapped a
    // session run, 2026-11-08)
    this.effectiveProfile = `${this.profile}-${process.pid}`;
    this.chrome = spawn(findChrome(), [
      '--headless=new', `--remote-debugging-port=${this.port}`, '--no-first-run',
      `--user-data-dir=${this.effectiveProfile}`, '--window-size=1280,900', 'about:blank',
    ], { stdio: 'ignore' });
    const t0 = Date.now();
    while (Date.now() - t0 < 20000) {
      try { await fetch(`http://127.0.0.1:${this.port}/json/version`); return; } catch {}
      await sleep(300);
    }
    throw new Error('cdp never came up');
  }

  async page(url) {
    const tab = await (await fetch(`http://127.0.0.1:${this.port}/json/new?${encodeURIComponent(url)}`, { method: 'PUT' })).json();
    const t1 = Date.now();
    while (!tab.webSocketDebuggerUrl && Date.now() - t1 < 10000) await sleep(200);
    const ws = new WebSocket(tab.webSocketDebuggerUrl);
    await new Promise((res, rej) => { ws.onopen = res; ws.onerror = rej; });
    ws.onmessage = (ev) => {
      const m = JSON.parse(ev.data);
      if (m.id && this.pending.has(m.id)) { this.pending.get(m.id)(m); this.pending.delete(m.id); }
    };
    const send = (method, params = {}) => new Promise((res) => {
      const i = ++this.id;
      this.pending.set(i, res);
      ws.send(JSON.stringify({ id: i, method, params }));
    });
    const close = () => fetch(`http://127.0.0.1:${this.port}/json/close/${tab.id}`).catch(() => {});
    await send('Runtime.enable');
    await send('Page.enable');
    return { tab, send, close };
  }

  kill() {
    this.chrome?.kill();
  }
}

export async function evalJS(conn, expr) {
  const r = await conn.send('Runtime.evaluate', { expression: expr, returnByValue: true, awaitPromise: true });
  if (r.result?.exceptionDetails) return 'EXC: ' + String(r.result.exceptionDetails.exception?.description || r.result.exceptionDetails.text).slice(0, 300);
  return r.result?.result?.value;
}
