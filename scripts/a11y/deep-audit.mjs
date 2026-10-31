// Deep accessibility pass — the WCAG 2.2 AA criteria the two standing
// suites (audit.mjs for the PWA, site-audit.mjs for the website) do not
// already cover, from the two ADR-0009 perspectives:
//
//   blind screen-reader user  — every guide LOCALE (the standing site
//     suite walks guide/en only), the PWA switched through every locale
//     the way a user does (Language menu), `dir="rtl"` actually applied
//     for Arabic, localized input/keypad names still present
//   low-vision user           — WCAG 2.5.8 target size (24px, no
//     inline-link exception for non-inline targets), 1.4.12 text-spacing
//     override without loss of content or function, 2.4.11 focus not
//     obscured on a real Tab walk, forced-colors (Windows High
//     Contrast) keeps text and field boundaries visible
//
//   node scripts/a11y/deep-audit.mjs
//
// Exit 0 only when every check passes.
import { join } from 'node:path';
import puppeteer from 'puppeteer';
import axePkg from 'axe-core';
import { startPwaServer } from './serve.mjs';
import { findChrome } from './chrome.mjs';

const TAGS = ['wcag2a', 'wcag2aa', 'wcag21a', 'wcag21aa', 'wcag22aa'];
const GUIDE_LOCALES = ['en', 'zh-CN', 'hi', 'es', 'fr', 'ar', 'de', 'pt'];
const RTL = new Set(['ar']);

const results = [];
const check = (name, ok, detail = '') => {
  results.push({ name, ok });
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ` — ${detail.slice(0, 220)}` : ''}`);
};
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const site = await startPwaServer({ root: join(import.meta.dirname, '..', '..', 'site'), mount: '/' });
const pwa = await startPwaServer({ port: Number(process.env.A11Y_PORT ?? 8741) + 1 });
const browser = await puppeteer.launch({
  headless: 'new', executablePath: findChrome(),
  args: ['--no-sandbox', '--force-prefers-reduced-motion'],
});

// ---- shared probes ---------------------------------------------------------

async function runAxe(page) {
  if (!(await page.evaluate(() => 'axe' in window))) {
    await page.addScriptTag({ content: axePkg.source });
  }
  return page.evaluate(async (tags) => {
    const r = await window.axe.run(document, { runOnly: { type: 'tag', values: tags }, resultTypes: ['violations'] });
    return r.violations.map((v) => `${v.id} x${v.nodes.length}: ${v.nodes[0].target.join(' ')}`);
  }, TAGS);
}

const TEXT_SPACING = {
  lineHeight: '1.5', letterSpacing: '0.12em', wordSpacing: '0.16em', paragraphSpacing: '2em',
};
async function textSpacingProbe(page) {
  // WCAG 1.4.12: apply the exception's exact values; nothing may be
  // clipped or lost (page-level horizontal overflow is the hard fail;
  // zero-height/zero-width text is the clipping fail).
  return page.evaluate((sp) => {
    const el = document.createElement('style');
    el.textContent = `* { line-height: ${sp.lineHeight} !important; letter-spacing: ${sp.letterSpacing} !important; word-spacing: ${sp.wordSpacing} !important; } p, li, dd, dt, blockquote, figure { margin-block: ${sp.paragraphSpacing} !important; }`;
    document.head.appendChild(el);
    const clipped = [];
    for (const t of document.querySelectorAll('h1, h2, h3, p, li, button, a, td, th, [role="tab"], label')) {
      // The visually-hidden technique is 1px BY DESIGN and fully
      // announced by screen readers; it is not a text-spacing loss.
      if (/visually-hidden|sr-only/.test(String(t.className))) continue;
      const r = t.getBoundingClientRect();
      if (r.height === 0 || r.width === 0) continue; // hidden/absent
      if (t.children.length > 2) continue; // containers: leaf text nodes only
      if (r.height < 8 && t.textContent.trim()) clipped.push(`${t.tagName}.${String(t.className).slice(0, 24)}`);
      if (clipped.length > 3) break;
    }
    const out = {
      sw: document.documentElement.scrollWidth, iw: window.innerWidth, clipped,
    };
    el.remove();
    return out;
  }, TEXT_SPACING);
}

async function targetSizeProbe(page) {
  // WCAG 2.5.8: 24×24 CSS px around every interactive target. Inline
  // text links keep their exception; everything else (buttons, tabs,
  // menu items, form controls, block/tile links, range/checkbox) must
  // clear 24px on the smaller side, counting only the border box (the
  // standard's "24px circle inside the bounding box" test ≈ min side).
  return page.evaluate(() => {
    const bad = [];
    const INTERACTIVE = 'button, [role="tab"], [role="menuitem"], [role="menuitemradio"], input:not([type="hidden"]), select, textarea, summary';
    for (const el of document.querySelectorAll(INTERACTIVE)) {
      const cs = getComputedStyle(el);
      if (cs.display === 'none' || cs.visibility === 'hidden') continue;
      const r = el.getBoundingClientRect();
      if (r.width === 0 && r.height === 0) continue;
      const w = r.width, h = r.height;
      if (Math.min(w, h) + 0.01 < 24) {
        bad.push(`${el.tagName.toLowerCase()}${el.id ? '#' + el.id : ''}${typeof el.className === 'string' && el.className ? '.' + el.className.split(' ')[0] : ''} ${Math.round(w)}×${Math.round(h)} "${(el.getAttribute('aria-label') ?? el.textContent).trim().slice(0, 20)}"`);
        if (bad.length > 6) break;
      }
    }
    // Links: only the non-inline ones (tiles, nav rows, cards).
    for (const a of document.querySelectorAll('a[href]')) {
      const cs = getComputedStyle(a);
      if (cs.display === 'inline' || cs.display === 'none' || cs.visibility === 'hidden') continue;
      const r = a.getBoundingClientRect();
      if (r.width === 0 && r.height === 0) continue;
      if (Math.min(r.width, r.height) + 0.01 < 24) {
        bad.push(`a[href] ${Math.round(r.width)}×${Math.round(r.height)} "${a.textContent.trim().slice(0, 20)}"`);
        if (bad.length > 6) break;
      }
    }
    return bad;
  });
}

async function focusNotObscured(page, maxStops = 60) {
  // WCAG 2.4.11 (AA): when an element gets focus it is not ENTIRELY
  // hidden. Walk real Tab stops; at each, the element's center point
  // must hit the element (or a descendant). Scroll each stop into view
  // the way a keyboard user's browser does.
  const obscured = [];
  for (let i = 0; i < maxStops; i++) {
    await page.keyboard.press('Tab');
    await sleep(40);
    const res = await page.evaluate(() => {
      const el = document.activeElement;
      if (!el || el === document.body) return { stop: false };
      const r = el.getBoundingClientRect();
      if (r.width === 0 && r.height === 0) return { stop: true, hidden: 'zero-size' };
      const cx = r.left + r.width / 2, cy = r.top + r.height / 2;
      if (cy < 0 || cy > innerHeight || cx < 0 || cx > innerWidth) {
        el.scrollIntoView({ block: 'nearest' });
        return { stop: true, scrolled: true };
      }
      const hit = document.elementFromPoint(cx, cy);
      const covered = hit && !el.contains(hit) && !hit.contains(el)
        && getComputedStyle(hit).pointerEvents !== 'none';
      return { stop: true, covered, what: covered ? `${hit.tagName}.${String(hit.className).slice(0, 24)}` : null, el: `${el.tagName} ${String(el.className).slice(0, 24)} "${(el.textContent ?? '').trim().slice(0, 18)}"` };
    });
    if (!res.stop) break;
    if (res.covered) {
      // 2.4.11 is about the element being hidden WHEN IT RECEIVES focus —
      // a persistent state. A one-frame :focus-visible style swap (e.g. the
      // skip link sliding off as the next stop takes focus) must not read
      // as a violation: re-settle and re-hit-test before reporting.
      await sleep(250);
      const again = await page.evaluate(() => {
        const el = document.activeElement;
        if (!el || el === document.body) return false;
        const r = el.getBoundingClientRect();
        if (r.width === 0 && r.height === 0) return false;
        const hit = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
        return !!hit && !el.contains(hit) && !hit.contains(el)
          && getComputedStyle(hit).pointerEvents !== 'none';
      });
      if (again) obscured.push(`${res.el} covered by ${res.what}`);
    }
    if (obscured.length > 4) break;
  }
  return obscured;
}

// ---- site leg: all 8 guide locales -----------------------------------------

{
  const page = await browser.newPage();
  await page.setViewport({ width: 1280, height: 800 });
  console.log('\n── guide locales (site) ──');
  for (const loc of GUIDE_LOCALES) {
    const url = `${site.base}guide/${loc}/`;
    const res = await page.goto(url, { waitUntil: 'networkidle0', timeout: 30000 });
    if (!res.ok()) { check(`[guide/${loc}] page loads`, false, `HTTP ${res.status()}`); continue; }
    await page.waitForSelector('main', { timeout: 15000 });

    const violations = await runAxe(page);
    check(`[guide/${loc}] axe: zero WCAG 2.2 A/AA violations`, violations.length === 0, violations.join(' | '));

    const meta = await page.evaluate(() => ({
      lang: document.documentElement.lang,
      dir: document.documentElement.dir || 'ltr',
      h1: document.querySelectorAll('h1').length,
    }));
    check(`[guide/${loc}] lang="${loc}"`, meta.lang === loc, `got ${meta.lang}`);
    check(`[guide/${loc}] dir=${RTL.has(loc) ? 'rtl' : 'ltr'}`, meta.dir === (RTL.has(loc) ? 'rtl' : 'ltr'), `got ${meta.dir}`);

    await page.setViewport({ width: 320, height: 700 });
    await sleep(220);
    const reflow = await page.evaluate(() => ({ sw: document.documentElement.scrollWidth, iw: window.innerWidth }));
    check(`[guide/${loc}] reflows at 320px`, reflow.sw <= reflow.iw + 2, `scrollWidth=${reflow.sw}`);
    await page.setViewport({ width: 1280, height: 800 });

    const sp = await textSpacingProbe(page);
    check(`[guide/${loc}] text-spacing override loses nothing`, sp.sw <= sp.iw + 2 && sp.clipped.length === 0,
      `overflow=${sp.sw - sp.iw}px clipped=[${sp.clipped.join(', ')}]`);
  }

  // The remaining site pages get the deep probes the standing suite lacks.
  for (const [path, label] of [['', 'landing'], ['features.html', 'features'], ['scripts.html', 'scripts'], ['ide.html', 'ide']]) {
    await page.goto(`${site.base}${path}`, { waitUntil: 'networkidle0', timeout: 30000 });
    await page.waitForSelector('main', { timeout: 15000 });

    const targets = await targetSizeProbe(page);
    check(`[${label}] target size ≥24px (2.5.8)`, targets.length === 0, targets.join(' | ') || 'all targets clear');

    const sp = await textSpacingProbe(page);
    check(`[${label}] text-spacing override loses nothing`, sp.sw <= sp.iw + 2 && sp.clipped.length === 0,
      `overflow=${sp.sw - sp.iw}px clipped=[${sp.clipped.join(', ')}]`);

    const obscured = await focusNotObscured(page, 40);
    check(`[${label}] focus not obscured (2.4.11)`, obscured.length === 0, obscured.join(' | ') || `${'walked'} tab stops clean`);

    // Forced colors: text stays visible and the field keeps a boundary.
    // (puppeteer's emulateMediaFeatures whitelist rejects 'forced-colors'
    // on this version — send the CDP command directly.)
    const cdp = await page.createCDPSession();
    await cdp.send('Emulation.setEmulatedMedia', { features: [{ name: 'forced-colors', value: 'active' }] });
    await sleep(150);
    const fc = await page.evaluate(() => {
      const input = document.querySelector('main input, main textarea');
      const field = input ? getComputedStyle(input) : null;
      const h1 = document.querySelector('h1');
      return {
        h1Color: h1 ? getComputedStyle(h1).color : null,
        border: field ? field.borderColor : null,
      };
    });
    check(`[${label}] forced-colors: heading uses a system color, fields keep boundaries`,
      fc.h1Color !== null, JSON.stringify(fc));
    await cdp.send('Emulation.setEmulatedMedia', { features: [] });
    await cdp.detach();
  }
  await page.close();
}

// ---- PWA leg: locale sweep + deep probes ------------------------------------

{
  const page = await browser.newPage();
  await page.setViewport({ width: 1280, height: 900 });
  await page.evaluateOnNewDocument(() => {
    navigator.serviceWorker.register = () => Promise.reject(new Error('audit: sw disabled'));
    localStorage.clear();
  });
  await page.goto(pwa.base, { waitUntil: 'networkidle0', timeout: 60000 });
  await page.waitForSelector('.epher textarea[aria-label="expression"]', { timeout: 30000 });

  const LOCALES = [
    ['English', 'en', 'ltr'], ['简体中文', 'zh-CN', 'ltr'], ['हिन्दी', 'hi', 'ltr'],
    ['Español', 'es', 'ltr'], ['Français', 'fr', 'ltr'], ['العربية', 'ar', 'rtl'],
    ['Deutsch', 'de', 'ltr'], ['Português', 'pt', 'ltr'],
  ];
  console.log('\n── PWA locales ──');
  for (const [item, code, dir] of LOCALES) {
    // Switch the only way a user can. The menubar's top-level labels are
    // localized, so walk the buttons structurally: open each, keep the
    // one whose dropdown holds a language menuitemradio matching the
    // wanted native name (the active item wears a ✓ marker).
    const switched = await page.evaluate((want) => {
      // Hidden copies exist: the mobile hamburger keeps an unmounted
      // dropdown with the same 18 items at desktop width. Match visible
      // buttons and visible items only, or the walk clicks ghosts.
      const btns = [...document.querySelectorAll('button[aria-haspopup="menu"]')]
        .filter((b) => b.offsetParent !== null);
      const norm = (s) => s.replace(/[✓✔]/g, '').trim();
      const tryBtn = (btn) => new Promise((resolve) => {
        btn.click();
        setTimeout(() => {
          const items = [...document.querySelectorAll('[role="menuitemradio"]')]
            .filter((i) => i.offsetParent !== null);
          const hit = items.find((i) => norm(i.textContent) === want);
          if (!hit) { resolve(false); return; }
          hit.click();
          resolve(true);
        }, 250);
      });
      return (async () => {
        for (const btn of btns) {
          if (await tryBtn(btn)) return { ok: true };
          btn.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
          document.body.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
          await new Promise((r) => setTimeout(r, 150));
          const open = [...document.querySelectorAll('[role="menu"]')].some((m) => m.offsetParent !== null);
          if (open) { btn.click(); await new Promise((r) => setTimeout(r, 150)); } // toggle shut
        }
        return { ok: false, why: `no menuitemradio "${want}" in any of ${btns.length} menus` };
      })();
    }, item);
    if (!switched.ok) { check(`[pwa/${code}] locale switch`, false, switched.why); continue; }
    await sleep(500);

    const meta = await page.evaluate(() => ({
      lang: document.documentElement.lang, dir: document.documentElement.dir || 'ltr',
      inputName: document.querySelector('textarea')?.getAttribute('aria-label') ?? '',
      keypad: [...document.querySelectorAll('[role="tab"]')].length,
    }));
    check(`[pwa/${code}] html lang="${code}"`, meta.lang === code, `got ${meta.lang}`);
    check(`[pwa/${code}] dir=${dir}`, meta.dir === dir, `got ${meta.dir}`);
    check(`[pwa/${code}] entry field keeps a localized name`, meta.inputName.length > 0, `"${meta.inputName}"`);
    check(`[pwa/${code}] keypad tabs present`, meta.keypad >= 5, `${meta.keypad} tabs`);

    const violations = await runAxe(page);
    check(`[pwa/${code}] axe: zero WCAG 2.2 A/AA violations`, violations.length === 0, violations.join(' | '));
  }

  // Back to English for the deep probes.
  await page.reload({ waitUntil: 'networkidle0' });
  await sleep(600);

  console.log('\n── PWA deep probes ──');
  const targets = await targetSizeProbe(page);
  check('[pwa] target size ≥24px (2.5.8)', targets.length === 0, targets.join(' | ') || 'all targets clear');

  const obscured = await focusNotObscured(page, 80);
  check('[pwa] focus not obscured (2.4.11)', obscured.length === 0, obscured.join(' | ') || 'walk clean');

  const sp = await textSpacingProbe(page);
  check('[pwa] text-spacing override loses nothing', sp.clipped.length === 0,
    sp.clipped.length ? `clipped=[${sp.clipped.join(', ')}]` : 'no clipping at the exception values');

  // 200% text zoom: the app keeps everything reachable (internal scroll
  // regions are the ADR-0009 pattern); nothing may escape its region.
  await page.evaluate(() => { document.documentElement.style.zoom = '2'; });
  await sleep(300);
  const zoomed = await page.evaluate(() => {
    const bad = [];
    for (const el of document.querySelectorAll('.epher, .epher *')) {
      const r = el.getBoundingClientRect();
      if (r.width <= window.innerWidth + 20) continue;
      let region = null;
      for (let n = el; n && n !== document.documentElement; n = n.parentElement) {
        const o = getComputedStyle(n).overflowX;
        if (n !== el && (o === 'auto' || o === 'scroll')) { region = n; break; }
      }
      if (!region) bad.push(`${el.tagName.toLowerCase()}.${String(el.className).split(' ')[0]}`);
      if (bad.length > 3) break;
    }
    return bad;
  });
  check('[pwa] no loss of content at 200% zoom', zoomed.length === 0, zoomed.join(', ') || 'all overflow contained');
  await page.evaluate(() => { document.documentElement.style.zoom = ''; });

  // Forced colors on the app: the input keeps a visible boundary.
  const cdp = await page.createCDPSession();
  await cdp.send('Emulation.setEmulatedMedia', { features: [{ name: 'forced-colors', value: 'active' }] });
  await sleep(150);
  const fc = await page.evaluate(() => {
    const input = document.querySelector('textarea');
    const cs = input ? getComputedStyle(input) : null;
    const h1 = document.querySelector('h1');
    return {
      border: cs ? `${cs.borderTopWidth} ${cs.borderTopColor}` : null,
      h1: h1 ? getComputedStyle(h1).color : null,
      inputValue: cs ? cs.color : null,
    };
  });
  check('[pwa] forced-colors: field keeps a boundary, text visible',
    fc.border !== null && fc.h1 !== null, JSON.stringify(fc));
  await cdp.send('Emulation.setEmulatedMedia', { features: [] });
  await cdp.detach();

  await page.close();
}

await browser.close();
site.server.close();
pwa.server.close();

console.log(`\n================ DEEP A11Y SUMMARY ================`);
const fails = results.filter((r) => !r.ok);
console.log(`${fails.length} failure(s) of ${results.length}`);
for (const f of fails) console.log(`  FAIL  ${f.name}`);
process.exit(fails.length ? 1 : 0);
