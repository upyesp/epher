// Smoke: the examples browser opens from the File menu and loads a
// script into the entry field (ADR-0070).
import puppeteer from "puppeteer";
import { startPwaServer } from "./a11y/serve.mjs";

const { server, base } = await startPwaServer({ port: Number(process.env.A11Y_PORT ?? 8741) });
const browser = await puppeteer.launch({ headless: true });
try {
  const page = await browser.newPage();
  await page.setViewport({ width: 1280, height: 900 });
  await page.goto(base, { waitUntil: "networkidle0" });

  // Open the File menu, click Example scripts.
  const fileBtn = await page.waitForSelector('button.menu-top[aria-label="File"]');
  await fileBtn.click();
  const exampleItem = await page.waitForSelector(
    'xpath/.//button[@role="menuitem" and contains(normalize-space(), "Example scripts")]'
  );
  await exampleItem.click();

  // The dialog is up, searchable, grouped.
  await page.waitForSelector('div[role="dialog"][aria-label*="Example scripts"]');
  const groups = await page.$$eval("h3.constants-group", (els) => els.map((e) => e.textContent));
  const rows = await page.$$eval('div[role="list"] button[role="listitem"]', (els) => els.length);
  if (rows < 400) throw new Error(`expected the whole fleet, saw ${rows} rows`);
  console.log(`dialog up: ${rows} rows, first groups: ${groups.slice(0, 3).join(" | ")}`);

  // Search filters.
  await page.type('div[role="dialog"] input[type="search"]', "compound");
  await new Promise((r) => setTimeout(r, 150));
  const filtered = await page.$$eval('div[role="list"] button[role="listitem"]', (els) => els.length);
  if (filtered === 0 || filtered >= rows) throw new Error(`search did not filter: ${filtered}`);
  console.log(`search "compound" -> ${filtered} row(s)`);

  // Pick the first hit; the dialog closes and the entry carries the script.
  await page.click('div[role="list"] button[role="listitem"]');
  await page.waitForSelector('div[role="dialog"]', { hidden: true });
  const text = await page.$eval("textarea", (el) => el.value);
  if (!text.startsWith("/* ====")) throw new Error(`entry did not get the script: ${text.slice(0, 40)}`);
  console.log(`loaded into the entry: ${text.split("\n")[1].slice(0, 60)}`);

  // The welcome hint stood down (the entry is no longer empty).
  const hint = await page.$("p.examples-hint");
  if (hint) throw new Error("the welcome hint should stand down once the entry is used");
  console.log("welcome hint stood down");
  console.log("SMOKE OK");
} finally {
  await browser.close();
  server.close();
}
