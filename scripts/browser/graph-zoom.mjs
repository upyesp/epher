// Browser regression for the 2D zoom controls (the "graph disappears"
// incident of 2026-11-12): the site front page's planck example defines
// the function and graphs it in ONE submitted script. The submit's
// script scope closes before the session is published and its defs do
// not cross back (ADR-0012 amendment), so the zoom/reset re-samples
// must run against the pane's draw-time environment — or every
// re-sample silently wiped the curve (wheel: dead; Reset zoom: the
// whole pane vanished).
//
//   A11Y_DIST=_site/pwa node scripts/browser/graph-zoom.mjs
//
// Drives the real PWA in headless Chrome: opens `?expr=…`, submits,
// wheel-zooms over the plot, clicks the zoom reset, and asserts the
// ticks moved (the zoom applied) and the pane kept its graph. Cases:
//   • graph sin(x)                  — no defs, the control
//   • def f … \n graph f …          — the script-scoped def regression
//   • def f … \n share f \n graph f — the documented escape hatch
// Exits nonzero on any failure.
import puppeteer from 'puppeteer';
import { startPwaServer } from '../a11y/serve.mjs';
import { findChrome } from '../a11y/chrome.mjs';

const PORT = Number(process.env.A11Y_PORT ?? 8743);

const cases = [
  {
    name: 'plain graph (no defs)',
    expr: 'graph sin(x)',
  },
  {
    name: 'script-scoped def + graph (the front-page planck shape)',
    expr: 'def f(x) = x ^ 2 - 3\ngraph f(x) from -5 to 5',
  },
  {
    name: 'shared def + graph',
    expr: 'def f(x) = x ^ 2 - 3\nshare f\ngraph f(x) from -5 to 5',
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
  args: ['--no-sandbox', '--force-prefers-reduced-motion'],
});
const { server, base: BASE } = await startPwaServer({ port: PORT });

// Snapshot the graph pane: the svg (a missing svg IS the incident — the
// pane renders html! {}), and the x tick labels (the zoom window's
// fingerprint: a zoom moved the window iff the ticks moved).
const pane = (page) =>
  page.evaluate(() => {
    const svg = document.querySelector('svg[role=img]');
    const ticks = svg
      ? [...svg.querySelectorAll('text')]
          .map((t) => t.textContent)
          .filter((t) => /^-?[\d.e+]+$/i.test(t))
      : null;
    return { svg: !!svg, ticks };
  });

const ticksDiffer = (a, b) => JSON.stringify(a) !== JSON.stringify(b) && b.length > 0;

for (const { name, expr } of cases) {
  const page = await browser.newPage();
  await page.setViewport({ width: 1280, height: 900 });
  await page.goto(BASE + '?expr=' + encodeURIComponent(expr), { waitUntil: 'networkidle0' });
  await new Promise((r) => setTimeout(r, 600));
  const input = await page.$('textarea');
  await input.focus();
  await page.keyboard.press('Enter');
  await new Promise((r) => setTimeout(r, 900));

  const drawn = await pane(page);
  check(`[${name}] the graph draws`, drawn.svg && drawn.ticks.length > 0);

  const box = await page.evaluate(() => {
    const r = document.querySelector('svg[role=img]').getBoundingClientRect();
    return { x: r.x + r.width * 0.5, y: r.y + r.height * 0.5 };
  });

  // Two wheel notches (deltaY < 0 = zoom in). The app's wheel listener
  // fires on the svg element; the page may log a benign
  // "preventDefault inside passive listener" warning.
  for (let i = 0; i < 2; i++) {
    await page.evaluate(({ x, y }) => {
      document
        .querySelector('svg[role=img]')
        .dispatchEvent(
          new WheelEvent('wheel', { bubbles: true, cancelable: true, clientX: x, clientY: y, deltaY: -120 }),
        );
    }, box);
    await new Promise((r) => setTimeout(r, 300));
  }
  const zoomed = await pane(page);
  check(
    `[${name}] the wheel zooms`,
    zoomed.svg && ticksDiffer(drawn.ticks, zoomed.ticks),
    `ticks ${JSON.stringify(drawn.ticks.slice(0, 4))} -> ${JSON.stringify(zoomed.ticks.slice(0, 4))}`,
  );

  // Reset zoom: the pane must keep its graph (the regression removed
  // the whole svg).
  await page.evaluate(() => {
    const b = [...document.querySelectorAll('button.tune-reset')].find((b) =>
      /zoom/i.test(b.getAttribute('aria-label') || ''),
    );
    b.click();
  });
  await new Promise((r) => setTimeout(r, 500));
  const reset = await pane(page);
  check(
    '[name] reset keeps the graph'.replace('[name]', `[${name}]`),
    reset.svg && reset.ticks.length > 0,
  );
  await page.close();
}

await browser.close();
server.close();

if (failures.length > 0) {
  console.error(`graph-zoom: ${failures.length} failure(s)`);
  process.exit(1);
}
console.log('graph-zoom: all cases pass');
