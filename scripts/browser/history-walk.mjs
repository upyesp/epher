// Browser regression for the history walk (ADR-0093): on an empty entry,
// Down recalls the newest history line and the arrows then walk the list
// (Down older, Up newer, the way the list renders, newest on top), the
// list highlights the recalled row and keeps it in view, Up from the
// newest returns the entry to empty, Down at the oldest stays, typing
// ends the walk, and a click on a row is a pick, not a walk.
//
//   node scripts/browser/history-walk.mjs
//   A11Y_DIST=_site/pwa node scripts/browser/history-walk.mjs
//
// Drives the real PWA in headless Chrome against the trunk build (the
// default A11Y_DIST is crates/web/dist) with real key events. Exits
// nonzero on any failure.
import puppeteer from 'puppeteer';
import { startPwaServer } from '../a11y/serve.mjs';
import { findChrome } from '../a11y/chrome.mjs';

const PORT = Number(process.env.A11Y_PORT ?? 8744);

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
const page = await browser.newPage();
await page.setViewport({ width: 1280, height: 900 });
await page.goto(BASE, { waitUntil: 'networkidle0' });
await new Promise((r) => setTimeout(r, 600));

const entry = () => page.$eval('textarea', (t) => t.value);
// The recalled row's expression (its button text minus the recorded
// answer suffix), or null when no row wears the walk's highlight.
const recalled = () =>
  page.evaluate(() => {
    const li = document.querySelector('.history-row[aria-current="true"]');
    if (!li) return null;
    return li
      .querySelector('.history-item')
      .textContent.replace(/\s{2}(=|error:|warning:)[\s\S]*$/, '');
  });
const rows = () => page.$$('.history-row');
const pause = (ms) => new Promise((r) => setTimeout(r, ms));
const press = async (key) => {
  await page.keyboard.press(key);
  await pause(120);
};
const submit = async (text) => {
  await (await page.$('textarea')).focus();
  await page.keyboard.type(text);
  await page.keyboard.press('Enter');
  await pause(350);
};
// Whether the highlighted row lies inside the history box's viewport
// (the walk keeps it in view; the sticky heading's height is excluded).
const recalledInView = () =>
  page.evaluate(() => {
    const box = document.querySelector('.history-box');
    const li = box.querySelector('.history-row[aria-current="true"]');
    if (!li) return null;
    const b = box.getBoundingClientRect();
    const head = box.querySelector('.history-head').getBoundingClientRect().height;
    const r = li.getBoundingClientRect();
    return r.top >= b.top + head - 1 && r.bottom <= b.bottom + 1;
  });

await submit('1 + 1');
await submit('2 + 2');
await submit('3 + 3');
check('three lines in the history', (await rows()).length === 3);
check('the entry is empty after a submit', (await entry()) === '');
check('no row is highlighted while idle', (await recalled()) === null);

await press('ArrowUp');
check(
  'Up on a fresh entry recalls the newest line too (either arrow starts the walk, the ADR-0093 amendment)',
  (await entry()) === '3 + 3' && (await recalled()) === '3 + 3',
  `entry "${await entry()}"`,
);
await press('ArrowUp');
check(
  'Up again returns the entry to empty',
  (await entry()) === '' && (await recalled()) === null,
);

await press('ArrowDown');
check('Down recalls the newest line', (await entry()) === '3 + 3', `entry "${await entry()}"`);
check('the newest row wears the highlight', (await recalled()) === '3 + 3');
await press('ArrowDown');
check(
  'Down again recalls the next older line',
  (await entry()) === '2 + 2' && (await recalled()) === '2 + 2',
  `entry "${await entry()}"`,
);
await press('ArrowDown');
check('Down reaches the oldest line', (await entry()) === '1 + 1');
await press('ArrowDown');
check(
  'Down at the oldest stays put',
  (await entry()) === '1 + 1' && (await recalled()) === '1 + 1',
);
await press('ArrowUp');
check('Up walks back toward newer', (await entry()) === '2 + 2');
await press('ArrowUp');
await press('ArrowUp');
check(
  'Up from the newest returns the entry to empty',
  (await entry()) === '' && (await recalled()) === null,
);

await press('ArrowDown');
check('the walk restarts at the newest', (await entry()) === '3 + 3');
await page.keyboard.type(' ');
check(
  'typing ends the walk',
  (await recalled()) === null && (await entry()) === '3 + 3 ',
  `entry "${await entry()}"`,
);
await press('ArrowDown');
check(
  'Down with edited text is the textarea\'s, not a recall',
  (await entry()) === '3 + 3 ',
);

// A click loads a row's expression (ADR-0027) and is not a walk: no
// highlight, and Down stays the textarea's.
await page.evaluate(() => {
  document.querySelectorAll('.history-item')[2].click();
});
await pause(200);
check(
  'a click picks the expression and ends the walk',
  (await entry()) === '1 + 1' && (await recalled()) === null,
  `entry "${await entry()}"`,
);
await press('ArrowDown');
check('Down after a click does not recall', (await entry()) === '1 + 1');

// A long history: the walk keeps the recalled row in view down to the
// oldest line.
await (await page.$('textarea')).focus();
await page.keyboard.down('Control');
await page.keyboard.press('KeyA');
await page.keyboard.up('Control');
await page.keyboard.press('Backspace');
for (let i = 1; i <= 40; i++) {
  await submit(`${i} * 2`);
}
check('forty-three lines in the history', (await rows()).length === 43);
for (let i = 0; i < 60; i++) {
  await press('ArrowDown');
}
check(
  'the walk reaches the oldest line of a long history',
  (await entry()) === '1 + 1',
  `entry "${await entry()}"`,
);
check(
  'the oldest row is highlighted and in view',
  (await recalled()) === '1 + 1' && (await recalledInView()) === true,
  `in view: ${await recalledInView()}`,
);

await browser.close();
server.close();

if (failures.length > 0) {
  console.error(`history-walk: ${failures.length} failure(s)`);
  process.exit(1);
}
console.log('history-walk: all cases pass');
