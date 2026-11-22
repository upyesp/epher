// Browser regression for the guide's solar3d time animation (ADR-0090,
// the "the slider appears but play does not move the planets" report of
// 2026-11-22, and the ADR-0091 amendment): the guide hands out
// `const t = now()` + `solar3d t`, a multi-piece submit whose
// script-scoped constant evaporates from the session when its scope
// closes (ADR-0012 amendment). The knob resolves against the pane's
// draw-time environment, the solar time knob spans the OUTERMOST
// DISPLAYED orbit — one ~24 s play cycle completes exactly that orbit
// (ADR-0091) — and a legend toggle re-spans the knob and resets the
// zoom to fit what is still shown. Before, the knob's ±2-day window
// moved Earth ~2° across the whole track and a play tick stepped 2.4
// hours of orbital time: the graphic read as frozen (and under
// prefers-reduced-motion a press stepped once, invisibly).
//
//   A11Y_DIST=_site/pwa node scripts/browser/solar-animate.mjs
//
// Drives the real PWA in headless Chrome: submits the example exactly
// as printed (two lines, one Enter), then asserts the slider row and
// play button exist with the outermost-orbit sweep, a drag moves the
// planets, playback steps the value, animates the scene, is announced
// (aria-pressed, 'Stop animating' label), survives a mid-play legend
// toggle, and that toggling re-spans the knob and refits the frame.
// Exits nonzero on any failure.
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
      // The frame: the plot SVG's viewBox ("x y w h") — the zoom fit.
      viewW: (() => {
        const vb = pane.querySelector('.plot-box svg')?.getAttribute('viewBox');
        return vb ? Number(vb.split(/\s+/)[2]) : null;
      })(),
      // The legend checkboxes: one per body, aria-label = name.
      checks: [...pane.querySelectorAll('.legend-solar input[type=checkbox]')].map((c) => ({
        name: c.getAttribute('aria-label'),
        checked: c.checked,
      })),
      span: (() => {
        const range = pane.querySelector('.sliders .slider input[type=range]');
        return range ? Number(range.max) - Number(range.min) : 0;
      })(),
    };
  });

const toggleBody = async (name) => {
  await page.evaluate((body) => {
    const pane = [...document.querySelectorAll('section.graph3d')].find((s) =>
      /solar/i.test(s.querySelector('.graph3d-title')?.textContent ?? ''),
    );
    const box = [...pane.querySelectorAll('.legend-solar input[type=checkbox]')].find(
      (c) => c.getAttribute('aria-label') === body,
    );
    box.click();
  }, name);
  await new Promise((r) => setTimeout(r, 600));
};

const p0 = await pane();
check('the solar pane and its knob render', !!p0?.row, `knob ${p0?.name}, value ${p0?.value}`);
if (!p0?.row) {
  await page.screenshot({ path: '/tmp/solar-animate-fail.png' });
  await browser.close();
  server.close();
  console.error('solar-animate: the knob never rendered');
  process.exit(1);
}

// The outermost displayed orbit governs the sweep: everything shown
// means Pluto's ~248-year orbit (the ±2-day generic span would be 4
// wide, the ADR-0090 month 60).
const p0span = p0.span;
check(
  'the knob spans the outermost displayed orbit',
  p0span > 88_000 && p0span < 94_000,
  `span ${p0span} days (Pluto ~90.6k)`,
);

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

// A legend toggle mid-play re-spans the animation without stopping it
// (ADR-0091): Pluto off hands the sweep to Neptune (~165 y) and the
// planets keep moving.
await toggleBody('Pluto');
const p5 = await pane();
check(
  'a mid-play toggle keeps playback running',
  p5?.pressed === 'true' && /stop/i.test(p5?.label ?? ''),
  `aria-pressed ${p5?.pressed}`,
);
check(
  'a mid-play toggle re-spans the sweep',
  p5.span > 58_000 && p5.span < 62_000,
  `span ${p5.span} days (Neptune ~60.2k)`,
);
await new Promise((r) => setTimeout(r, 1200));
const p6 = await pane();
check('the re-spanned animation still moves', JSON.stringify(p5.dots) !== JSON.stringify(p6.dots));

// Stop keeps the pane.
await page.evaluate(() => {
  const pane = [...document.querySelectorAll('section.graph3d')].find((s) =>
    /solar/i.test(s.querySelector('.graph3d-title')?.textContent ?? ''),
  );
  pane?.querySelector('.sliders .slider button.play-btn')?.click();
});
await new Promise((r) => setTimeout(r, 400));
const p7 = await pane();
check('stop keeps the pane', !!p7?.row);

// Toggling refits the frame: the viewBox narrows when the outermost
// orbit leaves and widens when it returns (the zoom resets to fit).
const beforeFit = p7.viewW;
await toggleBody('Neptune');
const p8 = await pane();
check(
  'hiding the outermost orbit narrows the frame',
  p8.viewW < beforeFit * 0.75,
  `viewBox width ${beforeFit} -> ${p8.viewW}`,
);
await toggleBody('Neptune');
const p9 = await pane();
check(
  'restoring the orbit restores the same fit',
  Math.abs(p9.viewW - beforeFit) < beforeFit * 0.05,
  `viewBox width ${beforeFit} -> ${p9.viewW}`,
);

// Every orbit hidden: the month fallback keeps the knob and the pane
// alive (the ADR-0039 empty pane). Pluto is still hidden from the
// mid-play toggle, so the loop covers the other eight.
for (const body of ['Mercury', 'Venus', 'Earth', 'Mars', 'Jupiter', 'Saturn', 'Uranus', 'Neptune']) {
  await toggleBody(body);
}
const p10 = await pane();
check(
  'nothing displayed falls back to the month window',
  p10.span > 50 && p10.span < 70,
  `span ${p10.span} days`,
);
check(
  'the empty pane keeps its frame and legend (ADR-0040)',
  !!p10.row && !!p10.viewW && p10.checks.length === 11,
  `frame ${p10.viewW}, ${p10.checks.length} legend entries`,
);
for (const body of ['Mercury', 'Venus', 'Earth', 'Mars', 'Jupiter', 'Saturn', 'Uranus', 'Neptune', 'Pluto']) {
  await toggleBody(body);
}
const p11 = await pane();
check(
  're-checking everything restores the outermost sweep',
  p11.span > 88_000 && p11.span < 94_000,
  `span ${p11.span} days`,
);

await browser.close();
server.close();

if (failures.length > 0) {
  console.error(`solar-animate: ${failures.length} failure(s)`);
  process.exit(1);
}
console.log('solar-animate: all cases pass');
