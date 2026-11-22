// Browser regression for the guide's solar3d time animation (ADR-0090,
// the "the slider appears but play does not move the planets" report of
// 2026-11-22): the guide hands out `const t = now()` + `solar3d t`, a
// multi-piece submit whose script-scoped constant evaporates from the
// session when its scope closes (ADR-0012 amendment). The knob resolves
// against the pane's draw-time environment, and — the ADR-0090 half —
// the solar time knob spans a month-wide window with a play step that
// keeps one cycle at ~24 s, so the planets visibly move. Before, the
// knob's ±2-day window moved Earth ~2° across the whole track and a
// play tick stepped 2.4 hours of orbital time: the graphic read as
// frozen (and under prefers-reduced-motion a press stepped once,
// invisibly).
//
//   A11Y_DIST=_site/pwa node scripts/browser/solar-animate.mjs
//
// Drives the real PWA in headless Chrome: submits the example exactly
// as printed (two lines, one Enter), then asserts the slider row and
// play button exist, a drag moves the planets, playback steps the
// value, animates the scene, and is announced (aria-pressed, 'Stop
// animating' label). Exits nonzero on any failure.
import puppeteer from 'puppeteer';
import { startPwaServer } from '../a11y/serve.mjs';
import { findChrome } from '../a11y/chrome.mjs';

const PORT = Number(process.env.A11Y_PORT ?? 8745);

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

const page = await browser.newPage();
await page.setViewport({ width: 1280, height: 900 });
await page.goto(BASE, { waitUntil: 'networkidle0' });
await new Promise((r) => setTimeout(r, 600));

// The guide's example, submitted exactly as printed: two lines, one
// Enter (the multiline script path).
await page.evaluate(() => {
  const ta = document.querySelector('textarea');
  ta.focus();
  ta.value = 'const t = now()\nsolar3d t';
  ta.dispatchEvent(new Event('input', { bubbles: true }));
});
await page.keyboard.press('Enter');
await new Promise((r) => setTimeout(r, 900));

const pane = () =>
  page.evaluate(() => {
    const pane = [...document.querySelectorAll('section.graph3d')].find((s) =>
      /solar/i.test(s.querySelector('.graph3d-title')?.textContent ?? ''),
    );
    if (!pane) return null;
    const row = pane.querySelector('.sliders .slider');
    return {
      row: !!row,
      name: row?.querySelector('.slider-name')?.textContent ?? null,
      value: row?.querySelector('.slider-value')?.textContent ?? null,
      pressed: row?.querySelector('button.play-btn')?.getAttribute('aria-pressed') ?? null,
      label: row?.querySelector('button.play-btn')?.getAttribute('aria-label') ?? null,
      range: row?.querySelector('input[type=range]') ?? null,
      // The planets: every positioned dot circle in the plot.
      dots: [...pane.querySelectorAll('.plot-box circle')].map((c) => [
        c.getAttribute('cx'),
        c.getAttribute('cy'),
      ]),
    };
  });

const p0 = await pane();
check('the solar pane and its knob render', !!p0?.row, `knob ${p0?.name}, value ${p0?.value}`);
if (!p0?.row) {
  await page.screenshot({ path: '/tmp/solar-animate-fail.png' });
  await browser.close();
  server.close();
  console.error('solar-animate: the knob never rendered');
  process.exit(1);
}

// A wide month window: the ±2-day generic span would be 4 wide.
const span = await page.evaluate(() => {
  const pane = [...document.querySelectorAll('section.graph3d')].find((s) =>
    /solar/i.test(s.querySelector('.graph3d-title')?.textContent ?? ''),
  );
  const range = pane?.querySelector('.sliders .slider input[type=range]');
  return range ? Number(range.max) - Number(range.min) : 0;
});
check('the knob spans a month-wide window', span > 50, `span ${span}`);

// Drag commits per input event and must move the planets.
await page.evaluate(() => {
  const pane = [...document.querySelectorAll('section.graph3d')].find((s) =>
    /solar/i.test(s.querySelector('.graph3d-title')?.textContent ?? ''),
  );
  const range = pane?.querySelector('.sliders .slider input[type=range]');
  if (!range) return;
  range.focus();
  range.value = String(Number(range.min) + (Number(range.max) - Number(range.min)) * 0.75);
  range.dispatchEvent(new Event('input', { bubbles: true }));
  range.dispatchEvent(new Event('change', { bubbles: true }));
});
await new Promise((r) => setTimeout(r, 700));
const p1 = await pane();
const dragMoved = p1.dots.length === p0.dots.length
  && p0.dots.some((d, i) => d[0] !== p1.dots[i][0] || d[1] !== p1.dots[i][1]);
check('a drag moves the planets', dragMoved);

// Play animates the scene and is announced.
await page.evaluate(() => {
  const pane = [...document.querySelectorAll('section.graph3d')].find((s) =>
    /solar/i.test(s.querySelector('.graph3d-title')?.textContent ?? ''),
  );
  pane?.querySelector('.sliders .slider button.play-btn')?.click();
});
await new Promise((r) => setTimeout(r, 1500));
const p2 = await pane();
await new Promise((r) => setTimeout(r, 1500));
const p3 = await pane();
check(
  'playback is announced',
  p2?.pressed === 'true' && /stop/i.test(p2?.label ?? ''),
  `aria-pressed ${p2?.pressed}, label ${p2?.label}`,
);
check('play steps the value', p2?.value !== p3?.value, `${p2?.value} -> ${p3?.value}`);
const playMoved = JSON.stringify(p2.dots) !== JSON.stringify(p3.dots);
check('play animates the planets', playMoved);

// Stop keeps the pane.
await page.evaluate(() => {
  const pane = [...document.querySelectorAll('section.graph3d')].find((s) =>
    /solar/i.test(s.querySelector('.graph3d-title')?.textContent ?? ''),
  );
  pane?.querySelector('.sliders .slider button.play-btn')?.click();
});
await new Promise((r) => setTimeout(r, 400));
const p4 = await pane();
check('stop keeps the pane', !!p4?.row);

await browser.close();
server.close();

if (failures.length > 0) {
  console.error(`solar-animate: ${failures.length} failure(s)`);
  process.exit(1);
}
console.log('solar-animate: all cases pass');
