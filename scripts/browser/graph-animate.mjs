// Browser regression for the tune panel's animation controls (the
// "no play button" report of 2026-11-12): the examples page opens
// scripts with `const` knobs — `const a = 1` + `graph sin(x * a)` —
// and the multi-piece submit runs in a script scope whose constants
// evaporate from the session when it closes (ADR-0012 amendment).
// The curve drew, but the tune panel gated its slider rows on the
// session's const sources, so the slider and its play button never
// appeared. The knobs must resolve against the pane's draw-time
// environment, like every re-sample does.
//
//   A11Y_DIST=_site/pwa node scripts/browser/graph-animate.mjs
//
// Drives the real PWA in headless Chrome: opens `?expr=…`, submits,
// and asserts the slider row + play button exist, playback moves the
// curve and is announced, and stopping keeps the pane. Cases:
//   • const a = 1 \n graph sin(x * a) — the script-scoped knob (the report)
//   • const a = 1, then graph sin(x * a) — interactive, the old path, control
// Exits nonzero on any failure.
import puppeteer from 'puppeteer';
import { startPwaServer } from '../a11y/serve.mjs';
import { findChrome } from '../a11y/chrome.mjs';

const PORT = Number(process.env.A11Y_PORT ?? 8744);

const cases = [
  {
    name: 'script-scoped const (the examples-page shape)',
    expr: 'const a = 1\ngraph sin(x * a)',
  },
  {
    name: 'interactive const then graph (control)',
    steps: ['const a = 1', 'graph sin(x * a)'],
  },
];

const failures = [];
const check = (name, ok, detail = '') => {
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? `; ${detail}` : ''}`);
  if (!ok) failures.push({ name, detail });
};

const browser = await puppeteer.launch({
  headless: 'new',
  executablePath: findChrome(),
  args: ['--no-sandbox'],
});
const { server, base: BASE } = await startPwaServer({ port: PORT });

const submit = async (page, text) => {
  await page.evaluate((t) => {
    const ta = document.querySelector('textarea');
    ta.focus();
    ta.value = t;
    ta.dispatchEvent(new Event('input', { bubbles: true }));
  }, text);
  await page.keyboard.press('Enter');
  await new Promise((r) => setTimeout(r, 800));
};

// The knob row for constant `a`; null when the tune panel has none.
const knob = (page) =>
  page.evaluate(() => {
    const row = [...document.querySelectorAll('.slider')].find(
      (r) => r.querySelector('.slider-name')?.textContent === 'a',
    );
    if (!row) return null;
    const btn = row.querySelector('button.play-btn');
    return {
      value: row.querySelector('.slider-value')?.textContent ?? null,
      pressed: btn?.getAttribute('aria-pressed') ?? null,
      label: btn?.getAttribute('aria-label') ?? null,
    };
  });

const clickPlay = (page) =>
  page.evaluate(() => {
    const row = [...document.querySelectorAll('.slider')].find(
      (r) => r.querySelector('.slider-name')?.textContent === 'a',
    );
    row?.querySelector('button.play-btn')?.click();
  });

for (const { name, expr, steps } of cases) {
  const page = await browser.newPage();
  await page.setViewport({ width: 1280, height: 900 });
  await page.goto(
    BASE + (expr ? '?expr=' + encodeURIComponent(expr) : ''),
    { waitUntil: 'networkidle0' },
  );
  await new Promise((r) => setTimeout(r, 600));
  for (const line of steps ?? [expr]) {
    await submit(page, line);
  }

  const drawn = await page.evaluate(() => !!document.querySelector('svg[role=img]'));
  check(`[${name}] the graph draws`, drawn);

  const k = await knob(page);
  check(
    `[${name}] the slider row and play button exist`,
    !!k,
    `value ${k?.value}`,
  );
  if (!k) {
    await page.close();
    continue;
  }

  // Play must move the curve (the tick re-samples it) and be announced.
  const snap = () =>
    page.evaluate(() => document.querySelector('svg[role=img]')?.innerHTML ?? '');
  const before = await snap();
  await clickPlay(page);
  await new Promise((r) => setTimeout(r, 1200));
  const during = await knob(page);
  const moved = before !== (await snap());
  check(`[${name}] play animates the curve`, moved);
  check(
    `[${name}] playback is announced`,
    during?.pressed === 'true' && /stop/i.test(during?.label ?? ''),
    `aria-pressed ${during?.pressed}, label ${during?.label}`,
  );

  // Stop: the pane keeps its graph (no wipe).
  await clickPlay(page);
  await new Promise((r) => setTimeout(r, 400));
  const stopped = await page.evaluate(() => !!document.querySelector('svg[role=img]'));
  check(`[${name}] stop keeps the pane`, stopped);

  await page.close();
}

await browser.close();
server.close();

if (failures.length > 0) {
  console.error(`graph-animate: ${failures.length} failure(s)`);
  process.exit(1);
}
console.log('graph-animate: all cases pass');
