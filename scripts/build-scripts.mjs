// Build the website Scripts pages' data: walk "epher scripts/" (the
// repository of example scripts, README there explains the layout) and
// write:
//   site/scripts-data.json  — the field/area/script tree with the full
//                             text of every script (fetched by scripts.js)
//   site/scripts.html       — the browser page, built from the index.html
//                             chrome exactly like build-examples.mjs
//
// Run: npm run build:scripts  (CI: pages.yml runs this before assembling)
import { readFileSync, writeFileSync, readdirSync, statSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SITE = join(ROOT, "site");
const SCRIPTS = join(ROOT, "epher scripts");

/** The one-line purpose from the header: the standard writes the file's
 *  second line as `// name.epher -- short purpose`. */
function purpose(path, name) {
  const head = readFileSync(path, "utf8").split("\n").slice(0, 12);
  for (const line of head) {
    const m = line.match(/^\/\/ .*\.epher -- (.*)$/);
    if (m) return m[1].trim();
  }
  return "";
}

const fields = [];
for (const field of readdirSync(SCRIPTS).sort()) {
  const fieldDir = join(SCRIPTS, field);
  if (!statSync(fieldDir).isDirectory()) continue; // README.md etc.
  const areas = [];
  for (const area of readdirSync(fieldDir).sort()) {
    const areaDir = join(fieldDir, area);
    if (!statSync(areaDir).isDirectory()) continue;
    const scripts = [];
    for (const file of readdirSync(areaDir).sort()) {
      if (!file.endsWith(".epher")) continue;
      const abs = join(areaDir, file);
      const rel = `epher scripts/${field}/${area}/${file}`;
      const name = file.slice(0, -".epher".length);
      scripts.push({
        name,
        title: purpose(abs, name),
        path: rel,
        text: readFileSync(abs, "utf8"),
      });
    }
    if (scripts.length) areas.push({ name: area, scripts });
  }
  if (areas.length) fields.push({ name: field, areas });
}

const data = { fields };
writeFileSync(join(SITE, "scripts-data.json"), JSON.stringify(data));
console.log(
  `built site/scripts-data.json (${fields.length} fields, ` +
    `${fields.reduce((n, f) => n + f.areas.length, 0)} areas, ` +
    `${fields.reduce((n, f) => n + f.areas.reduce((m, a) => m + a.scripts.length, 0), 0)} scripts)`
);

// scripts.html from the index.html chrome (header, docs nav, theme, i18n
// catalogs + app.js) with the browser main content and page markers.
const template = readFileSync(join(SITE, "index.html"), "utf8");
const page = template
  .replace(
    "<!--\n  epher landing page (served at epher.org, see docs/website.md).",
    "<!--\n  epher Scripts page. Built by scripts/build-scripts.mjs from the\n  index.html chrome: header, docs nav, theme, i18n catalogs + app.js.\n  The browser (scripts.js) fetches scripts-data.json, which the same\n  builder writes from the `epher scripts` folder of the repository.\n  -->\n<!--"
  )
  .replace('<title>epher: a programmable, scriptable calculator</title>', '<title>Scripts: epher</title>')
  .replace(
    '<meta name="description" content="A programmable, scriptable calculator: command line, terminal UI, desktop app, and offline web app. Graphs in 2D and 3D, exact numerics, eight languages, no accounts." />',
    '<meta name="description" content="Browse and copy the epher scripts repository: hundreds of ready-to-run scripts for astronomy, finance, trigonometry, statistics, and every field of mathematics." />'
  )
  .replace(
    '      <a href="scripts.html" data-i18n="nav-scripts">Scripts</a>',
    '      <a href="scripts.html" aria-current="page" data-i18n="nav-scripts">Scripts</a>'
  )
  .replace(
    /<main id="main">[\s\S]*?<\/main>/,
    `<main id="main">
    <article class="prose scripts-page">
      <h1 data-i18n="scripts-title">Scripts</h1>
      <p class="lede" data-i18n="scripts-lede">Ready-to-run scripts for the epher calculator, organized by field and topic. Open a folder to browse, click a script to read it, and copy any script to the clipboard.</p>
      <div class="scripts-toolbar">
        <label for="scripts-search" data-i18n="scripts-search">Search scripts (names and content)</label>
        <input id="scripts-search" type="search" autocomplete="off" spellcheck="false" />
      </div>
      <nav id="scripts-crumbs" class="scripts-crumbs" aria-label="Path"></nav>
      <div id="scripts-browser" class="scripts-browser"></div>
      <noscript>
        <p class="note">The scripts browser needs JavaScript. The same
        scripts live in the repository folder
        <code>epher scripts/</code> on GitHub.</p>
      </noscript>
    </article>
  </main>`
  )
  .replace(
    "</body>\n</html>",
    `<p class="visually-hidden" id="copy-status" role="status"></p>
  <script src="scripts.js"></script>
</body>
</html>`
  );

writeFileSync(join(SITE, "scripts.html"), page);
console.log("built site/scripts.html");
