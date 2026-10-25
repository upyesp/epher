// Chrome-for-Testing resolution for the a11y suites: the shared
// puppeteer download cache (no re-downloads across projects; CI
// installs with `npx puppeteer browsers install chrome` first).
import { existsSync, readdirSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';

export function findChrome() {
  if (process.env.CHROME_BIN && existsSync(process.env.CHROME_BIN)) return process.env.CHROME_BIN;
  const roots = [
    process.env.PUPPETEER_CACHE_DIR,
    join(homedir(), '.cache', 'puppeteer', 'chrome'),
  ].filter(Boolean);
  for (const root of roots) {
    if (!existsSync(root)) continue;
    for (const dir of readdirSync(root).sort().reverse()) {
      for (const rel of [
        // macOS layout
        join(dir, 'chrome-mac-arm64', 'Google Chrome for Testing.app', 'Contents', 'MacOS', 'Google Chrome for Testing'),
        // linux layout (CI)
        join(dir, 'chrome-linux64', 'chrome'),
      ]) {
        const p = join(root, rel);
        if (existsSync(p)) return p;
      }
    }
  }
  // Fall through: let puppeteer resolve its own pinned download.
  return undefined;
}
