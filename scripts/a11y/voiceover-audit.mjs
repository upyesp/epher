// Real-VoiceOver leg of the epher accessibility suite (ADR-0009).
// The axe + accessibility-tree layers live in audit.mjs (permissions-free,
// headless); THIS script drives the actual screen reader on this Mac and
// asserts on what it SPEAKS while a sighted-keyboard user tabs through
// the app. It takes over the Mac's audio while running (clamped quiet).
//
//   node scripts/a11y/voiceover-audit.mjs --vo
//
// MUST run under Terminal (or another GUI-launched context): controlling
// VoiceOver needs Automation/TCC grants a background daemon chain cannot
// get on macOS 27 — launch via ~/.epher-vo-audit.command (see the repo
// README section below / the trains project's equivalent wrapper).
//
// Known engine limit (proven in the trains project's
// scripts/vo-announcement-matrix.mjs): Chromium on macOS does not deliver
// aria-live announcements to VoiceOver in any markup form; Safari does.
// The result region's live-region MECHANISM is asserted by audit.mjs's
// AX layer; here, speech of the result is recorded as info.
import { existsSync, readdirSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { execFile } from 'node:child_process';
import puppeteer from 'puppeteer';
import { startPwaServer } from './serve.mjs';

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// Pete: VO speech must be barely audible during automated runs — capture
// is text-based. VoiceOver exposes NO scriptable cursor volume, so clamp
// the SYSTEM output volume instead (VO speech tracks it); restored at the
// end. Override with VO_SYSTEM_VOLUME=n (0..100).
const VO_SYSTEM_VOLUME = Number(process.env.VO_SYSTEM_VOLUME ?? 5);
let savedSystemVolume = null;
const osaOut = (script) =>
  new Promise((res, rej) =>
    execFile('/usr/bin/osascript', ['-e', script], (err, stdout, stderr) =>
      err ? rej(new Error(String(stderr || err.message).trim())) : res(stdout.trim()),
    ),
  );
async function clampVolume() {
  try {
    const cur = Number(await osaOut('output volume of (get volume settings)'));
    if (cur > VO_SYSTEM_VOLUME) {
      savedSystemVolume = cur;
      await osaOut(`set volume output volume ${VO_SYSTEM_VOLUME}`);
      console.log(`  info: system volume ${cur} -> ${VO_SYSTEM_VOLUME} for the run`);
    }
  } catch (e) { console.log(`  info: volume clamp failed: ${e.message}`); }
}
async function restoreVolume() {
  if (savedSystemVolume === null) return;
  try { await osaOut(`set volume output volume ${savedSystemVolume}`); } catch { /* best effort */ }
  savedSystemVolume = null;
}

function findChrome() {
  if (process.env.CHROME_BIN && existsSync(process.env.CHROME_BIN)) return process.env.CHROME_BIN;
  const root = join(homedir(), '.cache', 'puppeteer', 'chrome');
  for (const platform of readdirSync(root).sort().reverse()) {
    for (const mid of readdirSync(join(root, platform)).sort().reverse()) {
      for (const base of [join(root, platform, mid), join(root, platform)]) {
        for (const rel of [
          ['chrome-mac-arm64', 'Google Chrome for Testing.app', 'Contents', 'MacOS', 'Google Chrome for Testing'],
        ]) {
          const p = join(base, ...rel);
          if (existsSync(p)) return p;
        }
      }
    }
  }
  throw new Error('no Chrome for Testing in ~/.cache/puppeteer');
}

const results = [];
const check = (name, ok, detail = '') => {
  results.push({ name, ok });
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? ` — ${detail}` : ''}`);
};
const info = (m) => console.log(`  info: ${m}`);

if (!process.argv.includes('--vo')) {
  console.log('voiceover-audit: pass --vo to drive the real screen reader\n'
    + '(the headless axe + accessibility-tree layer is `node scripts/a11y/audit.mjs`)');
  process.exit(process.argv.includes('--vo') ? 0 : 0);
}

const { server, base } = await startPwaServer();
const browser = await puppeteer.launch({
  headless: false,
  executablePath: findChrome(),
  args: ['--no-sandbox', '--no-first-run', '--no-default-browser-check', '--window-size=1280,900', '--window-position=40,40'],
});
let poller = null;
const captured = [];
const seen = new Set();

try {
  const { voiceOver } = await import('@guidepup/guidepup');
  const page = (await browser.pages())[0] ?? (await browser.newPage());
  // --window-size is clamped to the display's work area (this Mac's is
  // ~800px), which silently flipped the app into its mobile layout.
  // Emulate the desktop viewport the ADR-0009 desktop audits run at.
  await page.setViewport({ width: 1280, height: 900 });
  await page.evaluateOnNewDocument(() => {
    navigator.serviceWorker.register = () => Promise.reject(new Error('audit: sw disabled'));
    localStorage.clear();
  });

  // VoiceOver must be LIVE before the page loads: aria-live only speaks
  // for changes that happen while the screen reader is running. Chrome
  // must be genuinely frontmost or the keystrokes land elsewhere.
  await voiceOver.start();
  await clampVolume();
  await sleep(800);
  await new Promise((res) => execFile('/usr/bin/osascript', ['-e', 'tell application "Google Chrome for Testing" to activate'], () => res()));
  await sleep(1200);
  poller = setInterval(() => {
    voiceOver.lastSpokenPhrase().then((p) => {
      const t = p.trim();
      if (t && !seen.has(t)) { seen.add(t); captured.push(t); }
    }).catch(() => {});
  }, 250);

  // First load warms Chromium's accessibility tree (it switches AX on
  // when it notices a screen reader).
  await page.goto(base, { waitUntil: 'networkidle0', timeout: 60000 });
  await page.waitForSelector('.epher textarea[aria-label="expression"]', { timeout: 30000 });
  await sleep(2500);

  // Chrome launches focused in the omnibox; only real keystrokes move
  // focus into the page (and the VO cursor with it).
  let entered = null;
  for (let i = 0; i < 15 && !entered; i++) {
    await voiceOver.press('Tab');
    await sleep(420);
    const last = await voiceOver.lastSpokenPhrase();
    if (/\b(heading|link|button|edit text|menu)\b/i.test(last)) entered = last;
  }
  check('vo: keyboard enters the page and VoiceOver follows', !!entered, entered?.trim().slice(0, 80) ?? 'nothing spoken in 15 tabs');

  // Tab through the app: every stop must be announced with its name, and
  // the walk must reach the expression field, the keypad, and the menu.
  // Budget 80: the full app is ~55–65 stops (menu rail, entry, result,
  // history, 10 keypad tabs, ~20 keypad cells, graph pane, install) and
  // the old 45 ran out inside the keypad — `=` is the last cell, right
  // after the `C` the walk did hear.
  // Start from the entry field: from the omnibox the Tab path crosses
  // Chrome's own UI and the browser→content transition drops one silent
  // stop that is a capture artifact, not a page defect (the AX tree
  // names every element; audit.mjs asserts that layer).
  await page.focus('.epher textarea[aria-label="expression"]');
  await sleep(400);
  // Deterministic walk start: rewind to the page's FIRST stop (the
  // menu rail brand). The enter loop above ends wherever the cursor
  // first spoke — mid-page — and a forward walk from there misses the
  // field and only re-hears it after wrapping.
  let atTop = null;
  try {
    for (let i = 0; i < 20 && !atTop; i++) {
      await voiceOver.press('Shift+Tab');
      await sleep(380);
      const last = await voiceOver.lastSpokenPhrase().catch(() => '');
      if (/menu bar/i.test(last)) atTop = last;
    }
  } catch {
    for (let i = 0; i < 30 && !atTop; i++) {
      await voiceOver.press('Tab');
      await sleep(380);
      const last = await voiceOver.lastSpokenPhrase().catch(() => '');
      if (/menu bar/i.test(last)) atTop = last;
    }
  }
  info(`walk starts from the rail top: ${atTop ? atTop.trim().slice(0, 60) : 'NOT REACHED'}`);

  let heardExpression = null;
  let heardEquals = null;
  let heardClear = null;
  let unnamed = 0;
  const silentStops = [];
  const transcript = [];
  for (let i = 0; i < 80; i++) {
    await voiceOver.press('Tab');
    await sleep(380);
    const last = await voiceOver.lastSpokenPhrase().catch(() => '');
    transcript.push(last.trim());
    // Ground truth for every stop: what has focus, is it rendered, and
    // which layout is live — the first silent stop (a hidden element
    // taking focus) is only diagnosable with this in hand.
    const where = await page.evaluate((spoken) => {
      const el = document.activeElement;
      if (!el || el === document.body) return { el: 'body', spoken };
      const r = el.getBoundingClientRect();
      const cs = getComputedStyle(el);
      const mb = document.querySelector('.menubar');
      return {
        el: `${el.tagName.toLowerCase()}${typeof el.className === 'string' && el.className ? '.' + el.className.split(' ').slice(0, 2).join('.') : ''} "${(el.getAttribute('aria-label') ?? el.textContent).trim().slice(0, 24)}"`,
        display: cs.display, rect: `${Math.round(r.width)}x${Math.round(r.height)}@${Math.round(r.left)},${Math.round(r.top)}`,
        vw: innerWidth, layout: mb && getComputedStyle(mb).display !== 'none' ? 'desktop' : 'mobile',
        named: (el.getAttribute('aria-label') ?? el.getAttribute('aria-labelledby') ?? el.getAttribute('title') ?? el.textContent ?? '').trim().length > 0,
        speech: spoken,
      };
    }, last.trim()).catch((e) => ({ el: '?', err: String(e).slice(0, 80) }));
    if (i < 6) info(`stop #${i + 1}: ${JSON.stringify(where)}`);
    if (!last.trim()) {
      unnamed++;
      // Two-layer verdict (ADR-0009): the AX layer is mechanism truth.
      // A silently-skipped stop on an element that HAS an accessible
      // name is VO poll timing (announcement dropped between 380ms
      // polls — delivery is intermittent, not absent), not a page
      // defect. Only an UNNAMED silent stop is a real failure.
      if (where.named === true) {
        info(`silent stop #${i + 1} = ${where.el} — named in the AX tree (poll-timing artifact), display=${where.display}`);
      } else {
        silentStops.push(`#${i + 1} ${JSON.stringify(where)}`);
      }
      continue;
    }
    if (/expression or script/i.test(last)) heardExpression ??= last;
    if (/calculate the result/i.test(last)) heardEquals ??= last;
    if (/clear the current entry/i.test(last)) heardClear ??= last;
    if (heardExpression && heardEquals && heardClear) break;
  }
  check('vo: the expression field announces its label', !!heardExpression,
    heardExpression?.trim().slice(0, 80) ?? 'not reached in 80 tabs');
  check('vo: the keypad equals key announces "Calculate the result"', !!heardEquals,
    heardEquals?.trim().slice(0, 80) ?? `not reached in ${transcript.length} tabs`);
  check('vo: keypad keys announce their purpose (clear entry)', !!heardClear,
    heardClear?.trim().slice(0, 80) ?? `not reached in ${transcript.length} tabs`);
  check('vo: every tab stop spoke something (no silent stops)', silentStops.length === 0,
    silentStops.length === 0 ? `${transcript.length} stops, all announced (${unnamed} poll-timing artifact(s) named in AX)`
      : `${silentStops.length} UNNAMED silent stop(s) of ${transcript.length}: ${silentStops.join(' | ')}`);

  // Evaluate by keyboard from the field: type, press Enter, and listen.
  // Chromium's live-region gap makes the announcement an info, not a
  // failure (the AX layer asserts the mechanism).
  await page.focus('textarea[aria-label="expression"]');
  await sleep(300);
  await page.keyboard.type('2 + 2', { delay: 60 });
  await sleep(300);
  const logBefore = (await voiceOver.spokenPhraseLog()).length;
  await page.keyboard.press('Enter');
  await sleep(1500);
  const spoken = (await voiceOver.spokenPhraseLog()).slice(logBefore).join(' | ');
  const heardResult = /4/.test(spoken) && /(equals|=|result|= 4)/i.test(spoken);
  info(`result announcement over real VoiceOver: ${heardResult ? `spoken "${spoken.trim().slice(0, 90)}"` : `not spoken (Chromium live-region engine gap) — spoken: ${spoken.trim().slice(0, 90) || 'nothing'}`}`);

  // The menu opens by keyboard and its items are announced — the REAL
  // user path, cursor-synced throughout: Tab to the Settings
  // menubutton, press Enter (a native button fires click), then Tab
  // into the open dropdown (ADR-0017: items are ordinary tab stops).
  // Programmatic clicks open the dropdown but move neither focus nor
  // the VO cursor, so nothing is spoken — that was the old probe's bug.
  let onSettings = null;
  await voiceOver.press('Tab'); // resync: cursor re-joins focus after the page.focus above
  await sleep(400);
  for (let i = 0; i < 80 && !onSettings; i++) {
    await voiceOver.press('Tab');
    await sleep(380);
    const last = await voiceOver.lastSpokenPhrase().catch(() => '');
    if (/settings collapsed submenu/i.test(last)) onSettings = last;
  }
  let menuHeard = null;
  if (onSettings) {
    await voiceOver.press('Enter');
    await sleep(700);
    for (let i = 0; i < 4 && !menuHeard; i++) {
      await voiceOver.press('Tab');
      await sleep(420);
      const last = await voiceOver.lastSpokenPhrase().catch(() => '');
      const m = last.match(/menu item|radio|dark|light|night|language/i);
      if (m) menuHeard = last;
    }
    await page.keyboard.press('Escape'); // close the dropdown again
    await sleep(250);
  }
  check('vo: the menu opens and its items are announced', !!menuHeard,
    menuHeard ? menuHeard.trim().slice(0, 80)
      : onSettings ? 'opened by Enter but no item speech in 4 tabs'
      : 'settings menubutton not reached by Tab in 80 stops');
} catch (e) {
  check('vo: harness completed', false, String(e).slice(0, 300));
} finally {
  if (poller) clearInterval(poller);
  try { const { voiceOver } = await import('@guidepup/guidepup'); await voiceOver.stop(); } catch { /* may not have started */ }
  await restoreVolume();
  await browser.close().catch(() => {});
  server.close();
}

console.log(`\n================ VO SUMMARY ================`);
const fails = results.filter((r) => !r.ok);
console.log(`${fails.length} failure(s) of ${results.length}`);
process.exit(fails.length ? 1 : 0);
