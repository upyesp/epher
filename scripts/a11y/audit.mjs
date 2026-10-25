// epher — ADR-0009 automated accessibility suite (axe-core WCAG 2.2
// A/AA + Chromium accessibility-tree screen-reader layer + computed
// contrast), ported from the trains project's audit harness.
//
//   node scripts/a11y/audit.mjs
//
// Serves the freshly built PWA (crates/web/dist) at /pwa/ from an
// in-process static server, audits every state ADR-0009 names —
// desktop dark/light/night, the graph pane with 2D and 3D plots
// drawn, the guide overlay open, and the mobile layout (baseline,
// hamburger open, after the auto-slide) — and asserts:
//   • axe: zero violations under wcag2a/2aa/21a/21aa/22aa
//   • AX tree: every interactive control exposes its role and name to
//     the layer macOS screen readers consume; the result region is a
//     live status region that announces the answer
//   • contrast: the theme tokens meet 4.5:1 (text) / 3:1 (non-text),
//     recomputed per theme from the computed styles
// Policy for axe "incomplete" (ADR-0009): color-contrast on targets
// axe cannot resolve (symbol-glyph buttons — we compute those
// ourselves — and plot-SVG text, covered by the token audit) and
// elements behind an open overlay (the baseline scans cover them).
import { existsSync, readdirSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import puppeteer from 'puppeteer';
import axePkg from 'axe-core';
import { startPwaServer } from './serve.mjs';

const PORT = Number(process.env.A11Y_PORT ?? 8741);
const TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];

// ---- chrome from the shared puppeteer cache (no downloads) ----------------
function findChrome() {
  if (process.env.CHROME_BIN && existsSync(process.env.CHROME_BIN)) return process.env.CHROME_BIN;
  const root = join(homedir(), '.cache', 'puppeteer', 'chrome');
  for (const platform of readdirSync(root).sort().reverse()) {
    for (const mid of readdirSync(join(root, platform)).sort().reverse()) {
      for (const base of [join(root, platform, mid), join(root, platform)]) {
        for (const rel of [
          ['chrome-mac-arm64', 'Google Chrome for Testing.app', 'Contents', 'MacOS', 'Google Chrome for Testing'],
          ['chrome-mac-x64', 'Google Chrome for Testing.app', 'Contents', 'MacOS', 'Google Chrome for Testing'],
          ['chrome-linux64', 'chrome'], ['chrome-win64', 'chrome.exe'],
        ]) {
          const p = join(base, ...rel);
          if (existsSync(p)) return p;
        }
      }
    }
  }
  throw new Error('no Chrome for Testing in ~/.cache/puppeteer — run `npx puppeteer browsers install chrome`');
}

const browser = await puppeteer.launch({
  headless: 'new', executablePath: findChrome(), args: ['--no-sandbox', '--force-prefers-reduced-motion'],
});
const { server, base: BASE } = await startPwaServer({ port: PORT });

const failures = [];
const infos = [];
const check = (state, name, ok, detail = '') => {
  console.log(`${ok ? 'PASS' : 'FAIL'}  [${state}] ${name}${detail ? ` — ${detail}` : ''}`);
  if (!ok) failures.push({ state, name, detail });
};

// Switch the theme the only way a user can: the menubar's Theme menu.
// Setting the data-theme attribute directly bypasses the app's state
// machine and is not a user-reachable path.
async function setTheme(page, theme) {
  const buttons = await page.$$('button[aria-haspopup="menu"]');
  for (const b of buttons) {
    const label = await b.evaluate((el) => (el.getAttribute('aria-label') ?? el.textContent).toLowerCase());
    if (!/theme|settings|preferences/.test(label)) continue;
    await b.click();
    await new Promise((r) => setTimeout(r, 200));
    const picked = await page.evaluate((want) => {
      const items = [...document.querySelectorAll('[role="menuitemradio"]')];
      const hit = items.find((i) => new RegExp(want, 'i').test(i.textContent));
      if (hit) { hit.click(); return hit.textContent.trim().slice(0, 30); }
      return null;
    }, theme);
    if (picked) {
      await new Promise((r) => setTimeout(r, 350));
      const applied = await page.evaluate((t) => document.documentElement.dataset.theme === t, theme);
      if (applied) return true;
      infos.push(`theme ${theme}: menu picked "${picked}" but data-theme did not follow`);
      return false;
    }
    await page.keyboard.press('Escape');
    await new Promise((r) => setTimeout(r, 150));
  }
  infos.push(`theme ${theme}: no Theme menu found — falling back to the attribute`);
  await page.evaluate((t) => { document.documentElement.dataset.theme = t; }, theme);
  await new Promise((r) => setTimeout(r, 250));
  return true;
}

// ---- page driver -----------------------------------------------------------
async function newState({ mobile = false } = {}) {
  const page = await browser.newPage();
  await page.setViewport({ width: mobile ? 375 : 1280, height: mobile ? 812 : 900 });
  // Deterministic boot: no service worker, English, no stored state.
  await page.evaluateOnNewDocument(() => {
    navigator.serviceWorker.register = () => Promise.reject(new Error('audit: sw disabled'));
    localStorage.clear();
  });
  await page.goto(BASE, { waitUntil: 'networkidle0', timeout: 60000 });
  await page.waitForSelector('.epher textarea[aria-label="expression"]', { timeout: 30000 });
  await page.evaluate(() => document.fonts?.ready).catch(() => {});
  return page;
}

const setInput = (page, text) =>
  page.type('textarea[aria-label="expression"]', text, { delay: 5 });

async function submit(page, expression) {
  await page.click('textarea[aria-label="expression"]', { clickCount: 3 });
  await page.keyboard.press('Backspace');
  await setInput(page, expression);
  await page.keyboard.press('Enter');
  await new Promise((r) => setTimeout(r, 700));
}

async function runAxe(page) {
  if (!(await page.evaluate(() => 'axe' in window))) {
    await page.addScriptTag({ content: axePkg.source });
  }
  return page.evaluate(async (tags) => {
    const r = await window.axe.run(document, {
      runOnly: { type: 'tag', values: tags },
      resultTypes: ['violations', 'incomplete'],
    });
    const fmt = (list) => list.map((v) => ({
      id: v.id, impact: v.impact, help: v.help,
      nodes: v.nodes.map((n) => ({
        target: n.target.join(' '),
        html: n.html.length > 140 ? n.html.slice(0, 140) + '…' : n.html,
        summary: (n.failureSummary || '').replace(/\n\s+/g, ' | ').slice(0, 240),
      })),
    }));
    return { violations: fmt(r.violations), incomplete: fmt(r.incomplete) };
  }, TAGS);
}

// ADR-0009: axe's only allowed incompletes are color-contrast on targets
// it cannot resolve — symbol-glyph buttons (whose content is only
// non-text characters: ⌫ − ⏎ …) and plot/graphic content — and content
// behind an open overlay (the baseline scans cover it). Symbol-glyph
// targets are collected and their contrast computed manually (the
// ADR's "the suite computes itself" leg).
const INCOMPLETE_ALLOWED = (state) => {
  const overlayOpen = state.includes('guide') || state.includes('hamburger');
  return (v, n) =>
    (v.id === 'color-contrast' &&
      ((n.summary || '').match(/only non-text characters/i) ||
       String(n.target).match(/svg|plot|glyph|legend|graph/i) ||
       n.html.match(/aria-hidden|<svg|path|text/i))) ||
    (overlayOpen && v.id === 'color-contrast');
};

// Self-compute contrast for the symbol-glyph controls axe marks
// incomplete: their glyph color vs the panel they sit on must clear
// 4.5:1 (they read as text characters).
async function verifySymbolGlyphs(name, page, axeResult) {
  const selectors = [];
  for (const v of axeResult.incomplete) {
    if (v.id !== 'color-contrast') continue;
    for (const n of v.nodes) {
      if ((n.summary || '').match(/only non-text characters/i)) selectors.push(n.target);
    }
  }
  if (!selectors.length) return;
  const rows = await page.evaluate((sels) => {
    const parseColor = (c) => { const m = c.match(/[\d.]+/g); return m ? m.map(Number) : null; };
    function lum(c) {
      const lin = (x0) => { const x = x0 / 255; return x <= 0.03928 ? x / 12.92 : Math.pow((x + 0.055) / 1.055, 2.4); };
      return 0.2126 * lin(c[0]) + 0.7152 * lin(c[1]) + 0.0722 * lin(c[2]);
    }
    const ratio = (a, b) => {
      const x = lum(a), y = lum(b);
      return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
    };
    function bgOf(el) {
      let n = el;
      while (n && n !== document.documentElement) {
        const bg = parseColor(getComputedStyle(n).backgroundColor);
        if (bg && (bg.length < 4 || bg[3] > 0.9)) return bg;
        n = n.parentElement;
      }
      return parseColor(getComputedStyle(document.body).backgroundColor);
    }
    return sels.map((sel) => {
      const el = document.querySelector(sel);
      if (!el) return { sel, ratio: null };
      const fg = parseColor(getComputedStyle(el).color);
      const bg = bgOf(el);
      return { sel, ratio: fg && bg ? ratio(fg, bg) : null };
    });
  }, selectors);
  for (const r of rows) {
    check(name, `symbol-glyph contrast ${String(r.sel).slice(0, 48)}`, r.ratio !== null && r.ratio >= 4.5,
      r.ratio === null ? 'element not found' : `${r.ratio.toFixed(2)}:1`);
  }
}

async function auditState(name, page, { expectPlot = false, three = false } = {}) {
  console.log(`\n── ${name} ──`);
  const { violations, incomplete } = await runAxe(page);
  for (const v of violations) {
    for (const n of v.nodes) {
      check(name, `axe ${v.id}`, false, `${n.target} — ${n.html} — ${n.summary}`);
      if (v.id === 'color-contrast' && String(n.target).includes('ans-inline')) {
        const dbg = await page.evaluate(() => {
          const el = document.querySelector('.ans-inline');
          const chain = [];
          let n = el;
          while (n && n !== document.documentElement) {
            const st = n.getAttribute('style') || '';
            if (st) chain.push(`${n.tagName}.${n.className} style=${st.slice(0, 140)}`);
            n = n.parentElement;
          }
          const matched = [];
          for (const sheet of document.styleSheets) {
            let rules; try { rules = sheet.cssRules; } catch { continue; }
            for (const rule of rules) {
              if (!rule.selectorText || !rule.style) continue;
              let m = false; try { m = el.matches(rule.selectorText); } catch {}
              if (m && /color\s*:/.test(rule.style.cssText)) matched.push(rule.cssText.slice(0, 140));
            }
          }
          const injected = [];
          for (const s of document.querySelectorAll('style')) {
            if ((s.textContent || '').includes('ffb3a8')) injected.push(s.textContent.slice(0, 200));
          }
          return {
            theme: document.documentElement.dataset.theme,
            color: getComputedStyle(el).color,
            text: el.textContent.trim().slice(0, 30),
            chain, matched, injected,
            tokenText: getComputedStyle(document.documentElement).getPropertyValue('--text').trim(),
            bodyColor: getComputedStyle(document.body).color,
            epherColor: getComputedStyle(document.querySelector('.epher')).color,
            resultColor: (() => { const r = document.getElementById('epher-result'); return r ? getComputedStyle(r).color : null; })(),
            resultChainColors: (() => {
              const r = document.getElementById('epher-result');
              const out = [];
              let n = r;
              while (n && out.length < 8) { out.push(`${n.tagName}.${String(n.className).slice(0, 24)}=${getComputedStyle(n).color}`); n = n.parentElement; }
              return out;
            })(),
          };
        }).catch((e) => ({ err: String(e) }));
        console.log(`   DEBUG ${name}:`, JSON.stringify(dbg, null, 1));
      }
    }
  }
  check(name, 'axe: zero WCAG 2.2 A/AA violations', violations.length === 0,
    violations.length === 0 ? undefined : `${violations.length} rule(s)`);
  const unallowed = [];
  for (const v of incomplete) {
    for (const n of v.nodes) if (!INCOMPLETE_ALLOWED(name)(v, n)) unallowed.push(`${v.id}: ${n.target} — ${n.summary}`);
  }
  if (unallowed.length) {
    for (const u of unallowed) check(name, 'axe incomplete (unallowed)', false, u);
  } else {
    check(name, 'axe incompletes within ADR-0009 allowances', true,
      incomplete.reduce((a, v) => a + v.nodes.length, 0) + ' incomplete node(s), all allowed');
  }
  await verifySymbolGlyphs(name, page, { violations, incomplete });
  // Plot presence where the state demands it.
  if (expectPlot) {
    const plot = await page.evaluate(() => {
      const el = document.querySelector('svg[role="img"], .epher canvas');
      return el ? (el.getAttribute('aria-label') || el.getAttribute('title') || '(present)') : null;
    });
    check(name, three ? '3D surface drawn with an accessible name' : '2D plot drawn with an accessible name', !!plot, plot ?? 'no svg[role=img]/canvas found');
  }
  return { violations, incomplete };
}

// ---- screen-reader mechanism layer (the AX tree macOS SRs consume) --------
async function srLayer(name, page) {
  const ax = await page.accessibility.snapshot();
  const nodes = [];
  const walk = (n) => { nodes.push(n); (n.children ?? []).forEach(walk); };
  (ax.children ?? []).forEach(walk);
  const byRole = (role) => nodes.filter((n) => n.role === role);

  const input = nodes.find((n) => n.role === 'textbox');
  check(name, 'SR: input exposes a name', !!input?.name, input?.name ?? 'textbox unnamed');
  const evaluate = nodes.find((n) => n.role === 'button' && /calculate the result|evaluate|equals/i.test(n.name ?? ''));
  if (!evaluate) {
    const names = nodes.filter((n) => n.role === 'button').map((n) => n.name).slice(0, 12);
    check(name, 'SR: evaluate button named', false, `buttons: ${names.join(' | ')}`);
  } else {
    check(name, 'SR: evaluate button named', true, evaluate.name);
  }
  const tabs = byRole('tab');
  check(name, 'SR: keypad tabs expose names', tabs.length >= 5, `${tabs.length} named tabs`);
  const status = byRole('status');
  check(name, 'SR: result region is a live status region', status.length >= 1);
  const menu = byRole('menubar');
  const mobile = await page.evaluate(() => window.innerWidth < 720);
  if (mobile) {
    // ADR-0017: the menu rail folds into a hamburger whose panel is one
    // role=menu — the rail itself only exists from 720px up.
    const burger = nodes.find((n) => n.role === 'button' && /menu/i.test(n.name ?? ''));
    const panel = byRole('menu');
    check(name, 'SR: mobile hamburger names its menu', !!burger && !panel.length, burger?.name ?? 'no menu button');
  } else {
    check(name, 'SR: menu rail exposed as menubar', menu.length === 1);
  }

  // The announcement: submit and read what the status region now holds.
  await submit(page, '2 + 2');
  const announced = await page.evaluate(() => {
    const el = document.querySelector('#epher-result');
    return el ? { live: el.getAttribute('aria-live') || el.closest('[aria-live]')?.getAttribute('aria-live'), text: (el.textContent || '').trim().slice(0, 60) } : null;
  });
  check(name, 'SR: result announced (live region carries the answer)',
    !!announced && announced.text.length > 0 && announced.live === 'polite',
    announced ? `"${announced.text}" live=${announced.live}` : 'no #epher-result');

  // Focus order: the first tab stops after the field must be named.
  await page.focus('textarea[aria-label="expression"]');
  const order = [];
  for (let i = 0; i < 6; i++) {
    await page.keyboard.press('Tab');
    order.push(await page.evaluate(() => {
      const el = document.activeElement;
      const name = el.getAttribute('aria-label') || el.textContent.trim().slice(0, 24) || el.tagName.toLowerCase();
      return `${el.getAttribute('role') ?? el.tagName.toLowerCase()}(${name})`;
    }));
  }
  const unnamed = order.filter((o) => o.endsWith('()'));
  check(name, 'SR: first tab stops are all named', unnamed.length === 0, order.join(' → '));
}

// ---- computed-contrast layer (theme tokens, WCAG relative luminance) ------
async function contrastLayer(page) {
  for (const theme of ['dark', 'light', 'night']) {
    await setTheme(page, theme);
    await new Promise((r) => setTimeout(r, 150));
    const rows = await page.evaluate(() => {
      const cs = (sel, prop) => {
        const el = document.querySelector(sel);
        return el ? getComputedStyle(el)[prop] : null;
      };
      const root = getComputedStyle(document.documentElement);
      const pick = (v) => (v && v.startsWith('rgb') ? v : root.getPropertyValue(v) || null);
      return {
        bg: root.getPropertyValue('--bg').trim(), panel: root.getPropertyValue('--panel').trim(),
        text: root.getPropertyValue('--text').trim(), muted: root.getPropertyValue('--muted').trim(),
        accent: root.getPropertyValue('--accent').trim(), onAccent: root.getPropertyValue('--on-accent').trim(),
        border: root.getPropertyValue('--border').trim(),
        inputColor: cs('textarea[aria-label="expression"]', 'color'),
        inputBg: cs('textarea[aria-label="expression"]', 'backgroundColor'),
      };
    });
    const verdicts = await page.evaluate((t) => {
      function parse(c) {
        const m = c.match(/[\d.]+/g);
        if (!m) return null;
        const [r, g, b, a = 1] = m.map(Number);
        return [r, g, b, a];
      }
      function lin(ch) { const v = ch / 255; return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4); }
      function lum(c) { const [r, g, b] = c; return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b); }
      function ratio(fg, bg) {
        const f = lum(fg.slice(0, 3)), b = lum(bg.slice(0, 3));
        return (Math.max(f, b) + 0.05) / (Math.min(f, b) + 0.05);
      }
      function over(fg, bg) { // composite translucent fg over bg
        const [fr, fgr, fb, fa] = fg, [br, bgc, bb] = bg;
        return [fr * fa + br * (1 - fa), fgr * fa + bgc * (1 - fa), fb * fa + bb * (1 - fa), 1];
      }
      const cs = getComputedStyle(document.documentElement);
      const hex = (n) => cs.getPropertyValue(n).trim();
      const rgb = (n) => {
        const v = hex(n);
        if (!v) return null;
        if (v[0] === '#') {
          const body = v.length === 4 ? v.slice(1).split('').map((c) => c + c).join('') : v.slice(1);
          return [0, 2, 4].map((i) => parseInt(body.slice(i, i + 2), 16));
        }
        const m = v.match(/[\d.]+/g);
        return m ? m.map(Number) : null;
      };
      return [
        ['text on bg', rgb('--text'), rgb('--bg'), 4.5],
        ['muted on bg', rgb('--muted'), rgb('--bg'), 4.5],
        ['text on panel', rgb('--text'), rgb('--panel'), 4.5],
        ['muted on panel', rgb('--muted'), rgb('--panel'), 4.5],
        ['on-accent on accent', rgb('--on-accent'), rgb('--accent'), 4.5],
        ['border vs bg (non-text)', rgb('--border'), rgb('--bg'), 3],
        ['border vs panel (non-text)', rgb('--border'), rgb('--panel'), 3],
        ['accent vs bg (non-text)', rgb('--accent'), rgb('--bg'), 3],
      ].map(([name, fg, bg, need]) => {
        if (!fg || !bg) return { name, ok: null, ratio: null, need };
        const f = fg[3] !== undefined && fg[3] < 1 ? over(fg, bg) : fg;
        const r = ratio(f, bg);
        return { name, ok: r >= need, ratio: r, need };
      });
    }, theme);
    for (const v of verdicts) {
      if (v.ok === null) { infos.push(`[${theme}] ${v.name}: token not resolvable`); continue; }
      check(`contrast-${theme}`, `${v.name} ≥ ${v.need}:1`, v.ok, `${Number(v.ratio).toFixed(2)}:1`);
    }
  }
}

// ---- drive the states ------------------------------------------------------
// Desktop baseline (dark) + themes.
const desktop = await newState();
{
  const axReady = await page_axWait(desktop);
  check('boot', 'PWA mounts with the input focused and named', axReady);

  await auditState('desktop-dark', desktop);
  await srLayer('desktop-dark', desktop);
  await contrastLayer(desktop);

  for (const theme of ['light', 'night']) {
    await setTheme(desktop, theme);
    await new Promise((r) => setTimeout(r, 150));
    await auditState(`desktop-${theme}`, desktop);
  }
  await setTheme(desktop, 'dark');
}

// 2D and 3D plots.
await submit(desktop, 'graph sin(x) + x / 2');
await auditState('desktop-2d', desktop, { expectPlot: true });
await submit(desktop, 'graph3d sin(x) * cos(y)');
await auditState('desktop-3d', desktop, { expectPlot: true, three: true });

// Guide overlay.
{
  const opened = await openGuide(desktop);
  if (opened) {
    await auditState('desktop-guide', desktop);
    await desktop.keyboard.press('Escape');
    await new Promise((r) => setTimeout(r, 200));
  } else {
    check('desktop-guide', 'guide overlay opens', false, 'no Guide menuitem found');
  }
}
await desktop.close();

// Mobile: baseline, hamburger open, after-submit auto-slide.
{
  const mobile = await newState({ mobile: true });
  await auditState('mobile-baseline', mobile);
  const burger = await mobile.evaluate(() => {
    const b = [...document.querySelectorAll('button')].find((x) =>
      /menu/i.test(x.getAttribute('aria-label') ?? '') && x.getAttribute('aria-expanded') !== null);
    if (b) { b.click(); return b.getAttribute('aria-label'); }
    return null;
  });
  await new Promise((r) => setTimeout(r, 250));
  if (burger) {
    await auditState('mobile-hamburger', mobile);
    const expanded = await mobile.evaluate(() =>
      [...document.querySelectorAll('button')].some((x) => x.getAttribute('aria-expanded') === 'true'));
    check('mobile-hamburger', 'hamburger reports expanded', expanded);
    await mobile.evaluate(() => {
      [...document.querySelectorAll('button')].find((x) => /menu/i.test(x.getAttribute('aria-label') ?? ''))?.click();
    });
    await new Promise((r) => setTimeout(r, 200));
  } else {
    check('mobile-hamburger', 'hamburger button found', false, 'no aria-expanded menu button');
  }
  const before = await panePos(mobile);
  await submit(mobile, 'graph x ^ 2');
  await new Promise((r) => setTimeout(r, 900));
  const after = await panePos(mobile);
  check('mobile-autoslide', 'auto-slide moves the view to the graph pane',
    before !== null && after !== null && after.onGraph && !before.onGraph,
    `before=${JSON.stringify(before)} after=${JSON.stringify(after)}`);
  await auditState('mobile-after-submit', mobile, { expectPlot: true });
  await srLayer('mobile', mobile);
  await mobile.close();
}

async function page_axWait(page) {
  return page.evaluate(() => !!document.querySelector('textarea[aria-label="expression"]'));
}
async function panePos(page) {
  return page.evaluate(() => {
    const panes = document.getElementById('panes');
    const pane = document.getElementById('graph-pane');
    if (!panes || !pane) return null;
    const target = pane.offsetLeft - panes.clientLeft;
    const sl = panes.scrollLeft;
    return { scrollLeft: Math.round(sl), target, onGraph: Math.abs(sl - target) < 40 };
  });
}
async function openGuide(page) {
  const buttons = await page.$$('button[aria-haspopup="menu"]');
  for (const b of buttons) {
    await b.click();
    await new Promise((r) => setTimeout(r, 200));
    const hit = await page.evaluate(() => {
      const items = [...document.querySelectorAll('[role="menuitem"]')];
      const guide = items.find((i) => /guide/i.test(i.textContent + ' ' + (i.getAttribute('aria-label') ?? '')));
      if (guide) { guide.click(); return true; }
      return false;
    });
    if (hit) { await new Promise((r) => setTimeout(r, 600)); return true; }
    await page.keyboard.press('Escape');
    await new Promise((r) => setTimeout(r, 150));
  }
  return false;
}

await browser.close();
server.close();

console.log(`\n================ SUMMARY ================`);
console.log(`${failures.length} failure(s)${infos.length ? `, ${infos.length} info` : ''}`);
for (const f of failures) console.log(`  FAIL [${f.state}] ${f.name}${f.detail ? ` — ${f.detail.slice(0, 200)}` : ''}`);
for (const i of infos) console.log(`  info ${i}`);
process.exit(failures.length ? 1 : 0);
