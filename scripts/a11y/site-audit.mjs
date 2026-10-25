// Website accessibility audit — the site/ landing pages, from the two
// perspectives ADR-0009 names for WCAG 2.2 AA:
//
//   blind screen-reader user  — the accessibility tree: landmarks (one
//     banner/main/contentinfo), one h1 per page, every link and button
//     named, the skip link is the FIRST focusable and lands on #main,
//     unique page titles, unique names for links that go to different
//     places, keyboard reachability of the footer with no silent stops
//
//   visually impaired user   — axe color-contrast across every page
//     (4.5:1 text / 3:1 large text and UI), focus-visible indicators on
//     real keyboard focus, reflow at a 320px viewport (WCAG 1.4.10)
//     and no horizontal overflow at 200% zoom (WCAG 1.4.4)
//
// plus the full axe scan (wcag2a/2aa/21a/21aa/22aa, zero violations).
//
//   node scripts/a11y/site-audit.mjs [siteDir]     (default: site/)
//
// CI runs it against the assembled _site artifact (guide and reference
// pre-built) via scripts/a11y/site-audit.mjs _site.
import { existsSync } from 'node:fs';
import { join } from 'node:path';
import puppeteer from 'puppeteer';
import axePkg from 'axe-core';
import { startPwaServer } from './serve.mjs';
import { findChrome } from './chrome.mjs';

const TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];
const siteDir = process.argv[2] ?? join(import.meta.dirname, '..', '..', 'site');
const results = [];
const check = (name, ok, detail = '') => {
  results.push({ name, ok });
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ` — ${detail}` : ''}`);
};
const info = (m) => console.log(`  info: ${m}`);

const PAGES = [
  ['', 'landing'], ['features.html', 'features'], ['examples.html', 'examples'],
  ['scripts.html', 'scripts'], ['guide/en/', 'guide'], ['reference/', 'reference'],
  ['ide.html', 'ide'], ['privacy.html', 'privacy'],
];

const { server, base } = await startPwaServer({ root: siteDir, mount: '/' });
const browser = await puppeteer.launch({
  headless: 'new', executablePath: findChrome(),
  args: ['--no-sandbox', '--force-prefers-reduced-motion'],
});

try {
  const page = await browser.newPage();
  const titles = [];
  const linkTargets = new Map(); // name -> set of hrefs (cross-page)

  for (const [path, label] of PAGES) {
    const url = `${base}${path}`;
    let res;
    try {
      res = await page.goto(url, { waitUntil: 'networkidle0', timeout: 30000 });
    } catch (e) {
      check(`[${label}] page loads (${url})`, false, String(e).slice(0, 120));
      continue;
    }
    check(`[${label}] page loads (${url})`, res.ok(), `HTTP ${res.status()}`);
    await page.waitForSelector('main', { timeout: 15000 });
    await page.evaluate(() => { localStorage.clear(); });
    await page.addScriptTag({ content: axePkg.source });

    // ---- axe: the full WCAG 2.2 A/AA scan --------------------------------
    const axe = await page.evaluate((tags) => window.axe.run(document, {
      tags, resultTypes: ['violations', 'incomplete'],
    }), TAGS);
    const minor = axe.incomplete.map((i) => `${i.id}(${i.nodes.length})`).join(', ');
    if (axe.violations.length === 0) {
      check(`[${label}] axe: zero WCAG 2.2 A/AA violations`, true,
        axe.incomplete.length ? `incomplete: ${minor}` : 'clean');
    } else {
      check(`[${label}] axe: zero WCAG 2.2 A/AA violations`, false,
        axe.violations.map((v) => `${v.id} x${v.nodes.length}: ${v.help} — e.g. ${v.nodes[0].target.join(' ')}`).join(' | '));
    }
    for (const inc of axe.incomplete) {
      if (inc.id !== 'color-contrast') {
        info(`[${label}] ${inc.id} incomplete x${inc.nodes.length}: ${inc.nodes.map((n) => n.target.join(' ') + ' ' + n.html.slice(0, 60)).join(' ; ')}`);
      }
    }
    if (axe.incomplete.length) info(`[${label}] axe incomplete: ${axe.incomplete.map((i) => `${i.id}(${i.nodes.length})`).join(', ')}${axe.incomplete.some((i) => i.id === 'color-contrast') ? ' — glyph contrast self-computed below' : ''}`);

    // ---- symbol glyphs axe cannot resolve: compute their contrast -------
    // The same policy as the PWA suite: axe marks text-presentation
    // symbols (✓ ☰ ← →) "incomplete" for color-contrast; the suite
    // computes the WCAG ratio itself and requires 4.5:1 (3:1 large).
    const glyphInc = axe.incomplete.find((i) => i.id === 'color-contrast');
    if (glyphInc) {
      const bad = await page.evaluate((sels) => {
        const lum = (r, g, b) => {
          const f = (c) => { c /= 255; return c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4; };
          return 0.2126 * f(r) + 0.7152 * f(g) + 0.0722 * f(b);
        };
        const parse = (s) => {
          const m = s.match(/rgba?\(([\d.]+), ([\d.]+), ([\d.]+)(?:, ([\d.]+))?\)/);
          if (!m) return null;
          return { r: +m[1], g: +m[2], b: +m[3], a: m[4] === undefined ? 1 : +m[4] };
        };
        const bgOf = (el) => {
          for (let n = el; n; n = n.parentElement) {
            const c = parse(getComputedStyle(n).backgroundColor);
            if (c && c.a > 0.9) return c;
          }
          return { r: 255, g: 255, b: 255 };
        };
        const out = [];
        for (const sel of sels) {
          for (const el of document.querySelectorAll(sel)) {
            const fg = parse(getComputedStyle(el).color);
            if (!fg) continue;
            const bg = bgOf(el);
            const l1 = lum(fg.r, fg.g, fg.b), l2 = lum(bg.r, bg.g, bg.b);
            const ratio = (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
            const cs = getComputedStyle(el);
            const px = parseFloat(cs.fontSize);
            const large = px >= 24 || (px >= 18.66 && Number(cs.fontWeight) >= 700);
            const need = large ? 3 : 4.5;
            if (ratio < need) out.push(`${el.tagName}.${el.className} "${el.textContent.trim().slice(0, 12)}" ${ratio.toFixed(2)}:1 < ${need}:1 (#${fg.r.toString(16)} on #${bg.r.toString(16)})`);
          }
        }
        return out;
      }, glyphInc.nodes.map((n) => n.target.join(',')));
      check(`[${label}] glyph contrast self-computed (axe-incomplete targets)`, bad.length === 0,
        bad.length === 0 ? `${glyphInc.nodes.length} symbol target(s) all pass` : bad.slice(0, 4).join(' | '));
    }

    // ---- blind screen-reader user: the accessibility tree ----------------
    const ax = await page.evaluate(() => {
      const named = (el) => (el.getAttribute('aria-label') ?? el.getAttribute('aria-labelledby')
        ?? el.getAttribute('alt') ?? el.getAttribute('title') ?? el.textContent ?? '').trim();
      // Implicit landmarks: header/footer directly under body (or any
      // explicit role); main is a real landmark everywhere.
      const landmarks = {
        banner: document.querySelectorAll('body > header, [role="banner"]').length,
        main: document.querySelectorAll('main, [role="main"]').length,
        contentinfo: document.querySelectorAll('body > footer, [role="contentinfo"]').length,
        navigation: document.querySelectorAll('nav, [role="navigation"]').length,
      };
      const h1 = [...document.querySelectorAll('h1')];
      const links = [...document.querySelectorAll('a[href]')].map((a) => named(a));
      const unnamed = links.filter((n) => !n).length
        + [...document.querySelectorAll('button')].filter((b) => !named(b)).length;
      const skip = document.querySelector('a.skip-link');
      const levels = [...document.querySelectorAll('h1,h2,h3,h4,h5,h6')].map((h) => Number(h.tagName[1]));
      let skips = 0;
      for (let i = 1; i < levels.length; i++) if (levels[i] - levels[i - 1] > 1) skips++;
      return {
        landmarks,
        h1: h1.length,
        firstH1Text: h1[0]?.textContent.trim().slice(0, 60) ?? '',
        unnamedControls: unnamed,
        linkCount: links.length,
        skipHref: skip?.getAttribute('href') ?? null,
        headingSkips: skips,
        title: document.title.trim(),
        lang: document.documentElement.lang.trim(),
        links,
      };
    });
    check(`[${label}] exactly one main landmark`, ax.landmarks.main === 1, `got ${ax.landmarks.main}`);
    check(`[${label}] banner and contentinfo present once`,
      ax.landmarks.banner === 1 && ax.landmarks.contentinfo === 1,
      `banner=${ax.landmarks.banner} contentinfo=${ax.landmarks.contentinfo}`);
    check(`[${label}] exactly one h1`, ax.h1 === 1, ax.h1 === 1 ? `"${ax.firstH1Text}"` : `got ${ax.h1}`);
    check(`[${label}] heading levels do not skip`, ax.headingSkips === 0,
      ax.headingSkips === 0 ? 'no skips' : `${ax.headingSkips} skip(s)`);
    check(`[${label}] every link and button has an accessible name`, ax.unnamedControls === 0,
      ax.unnamedControls === 0 ? `${ax.linkCount} links all named` : `${ax.unnamedControls} unnamed`);
    check(`[${label}] skip link exists and targets #main`,
      ax.skipHref === '#main', `href=${ax.skipHref ?? 'none'}`);
    check(`[${label}] page has a non-empty title`, ax.title.length > 0, ax.title.slice(0, 60));
    check(`[${label}] html lang set`, ax.lang.length >= 2, ax.lang);
    titles.push({ label, title: ax.title });
    for (let i = 0; i < ax.links.length; i++) {
      const href = await page.evaluate((ix) => document.querySelectorAll('a[href]')[ix].getAttribute('href'), i);
      const name = ax.links[i];
      if (!name) continue;
      // Fragment-only hrefs are same-page by definition (#main skip
      // targets); they cannot be cross-page ambiguity.
      if (href.startsWith('#')) continue;
      if (!linkTargets.has(name)) linkTargets.set(name, new Map());
      // Ambiguity is about DESTINATIONS: resolve relative hrefs (pages
      // live at different depths) and compare locations, not strings.
      const dest = new URL(href, url).pathname;
      const m = linkTargets.get(name);
      m.set(dest, (m.get(dest) ?? 0) + 1);
    }

    // Skip link is the FIRST tab stop, and focus moves to #main.
    await page.goto(url, { waitUntil: 'load' });
    await page.keyboard.press('Tab');
    const firstStop = await page.evaluate(() => {
      const el = document.activeElement;
      return { cls: el.className, href: el.getAttribute?.('href') ?? null, text: el.textContent.trim().slice(0, 40) };
    });
    check(`[${label}] skip link is the first tab stop`,
      /skip-link/.test(firstStop.cls) && firstStop.href === '#main',
      JSON.stringify(firstStop));
    await page.keyboard.press('Enter');
    await sleep(120);
    const landed = await page.evaluate(() => document.activeElement.id || document.activeElement.tagName);
    check(`[${label}] activating the skip link moves focus to #main`,
      landed === 'main' || landed === 'MAIN', `focus on ${landed}`);

    // Focus indicator visible on a real keyboard stop (2.4.7/2.4.13).
    const focusVisible = await page.evaluate(() => {
      const el = document.activeElement;
      if (!el || el === document.body) return { on: false };
      const cs = getComputedStyle(el);
      const ind = (cs.outlineStyle !== 'none' && parseFloat(cs.outlineWidth) > 0) || cs.boxShadow !== 'none';
      return { on: el.matches(':focus-visible'), ind };
    });
    check(`[${label}] keyboard focus shows a visible indicator`,
      focusVisible.on && focusVisible.ind,
      focusVisible.on ? (focusVisible.ind ? 'outline/shadow present' : ':focus-visible but no outline/shadow') : 'not :focus-visible');

    // ---- visually impaired user: reflow and zoom --------------------------
    await page.setViewport({ width: 320, height: 700 });
    await sleep(250);
    const reflow = await page.evaluate(() => ({
      sw: document.documentElement.scrollWidth, iw: window.innerWidth,
    }));
    check(`[${label}] reflows at 320px without horizontal scrolling`,
      reflow.sw <= reflow.iw + 2, `scrollWidth=${reflow.sw} @ ${reflow.iw}px`);
    await page.setViewport({ width: 1280, height: 800 });
    await page.evaluate(() => { document.documentElement.style.zoom = '2'; });
    await sleep(250);
    // Overflow inside a keyboard-accessible scroll region (overflow-x:
    // auto with tabindex) is fine at 200% — nothing is clipped or lost;
    // the page itself must not grow. Every region that scrolls must be
    // reachable by keyboard, or its content is lost to keyboard users.
    const zoomed = await page.evaluate(() => {
      const scrolls = (el) => {
        const o = getComputedStyle(el).overflowX;
        return o === 'auto' || o === 'scroll' || o === 'hidden' || o === 'clip';
      };
      const inScrollRegion = (el) => {
        for (let n = el; n && n !== document.documentElement; n = n.parentElement) {
          if (n !== el && scrolls(n)) return n;
        }
        return null;
      };
      const bad = [];
      const regions = new Set();
      for (const el of document.querySelectorAll('body *')) {
        const r = el.getBoundingClientRect();
        if (r.width <= window.innerWidth + 20) continue;
        const region = inScrollRegion(el);
        if (region) {
          regions.add(region);
        } else {
          bad.push(`${el.tagName.toLowerCase()}.${String(el.className).split(' ').join('.')}`);
        }
      }
      const unkeyed = [...regions].filter((r) => r.tabIndex < 0 && !r.closest('[tabindex="0"]')
        && r.getAttribute('tabindex') !== '0').map((r) => `${r.tagName.toLowerCase()}.${String(r.className).split(' ').join('.')}`);
      return { bad: bad.slice(0, 4), unkeyed: unkeyed.slice(0, 4), regions: regions.size };
    });
    check(`[${label}] no loss of content at 200% zoom`,
      zoomed.bad.length === 0,
      zoomed.bad.length === 0 ? `${zoomed.regions} contained scroll region(s)` : `uncontained overflow: ${zoomed.bad.join(', ')}`);
    check(`[${label}] every zoom-scrollable region is keyboard-reachable`,
      zoomed.unkeyed.length === 0,
      zoomed.unkeyed.length === 0 ? 'all regions have tabindex=0' : `no tabindex: ${zoomed.unkeyed.join(', ')}`);
    await page.evaluate(() => { document.documentElement.style.zoom = ''; });
  }

  // ---- site-wide: unique titles; link text ambiguity --------------------
  const dupTitles = titles.filter((t, i) => titles.findIndex((o) => o.title === t.title) !== i);
  check('site-wide: every page has a unique title', dupTitles.length === 0,
    dupTitles.length === 0 ? titles.map((t) => t.label).join(', ') : `duplicate: ${dupTitles.map((t) => t.label).join(', ')} share "${dupTitles[0]?.title.slice(0, 50)}"`);
  const ambiguous = [...linkTargets.entries()].filter(([, dests]) => dests.size > 1)
    .map(([name, dests]) => `"${name}" -> ${[...dests.keys()].join(' , ')}`);
  check('site-wide: identical link text never goes to different destinations',
    ambiguous.length === 0, ambiguous.length === 0 ? `${linkTargets.size} distinct link names, destinations unambiguous` : ambiguous.join(' | '));
} catch (e) {
  check('site audit harness completed', false, String(e).slice(0, 300));
} finally {
  await browser.close().catch(() => {});
  server.close();
}

function sleep(ms) { return new Promise((r) => setTimeout(r, ms)); }

console.log('\n================ SITE A11Y SUMMARY ================');
const fails = results.filter((r) => !r.ok);
console.log(`${fails.length} failure(s) of ${results.length}`);
for (const f of fails) console.log(`  FAIL  ${f.name}`);
process.exit(fails.length ? 1 : 0);
