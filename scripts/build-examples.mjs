// Build the website Examples page: site/examples.html, one static page
// with the same chrome as index.html (theme, i18n catalogs + app.js,
// disclosure nav), whose prose is data-i18n keys in site/i18n/<lang>.js.
// The epher code blocks are NEVER localized (ADR-0007) and ship identical
// in every language; they get the same copy-button treatment as the user
// guide's example blocks (ADR-0036). On touch devices a tap on an example
// copies it and opens the app with it staged in the entry field
// (ADR-0035 amendment).
//
// Run: npm run build:examples  (CI: pages.yml runs this after build:guide)
import { readFileSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { escapeHtml, highlightEpher } from "./build-guide.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const SITE = join(ROOT, "site");

// Each example: a data-i18n caption key, the code, and the fence kind
// (epher → token colors; sh → same colors, shell commands; text → plain).
// The TUI/desktop/PWA section comes first: those frontends share the
// entry field, so their examples are the plain expressions to paste.
// The command-line section follows, then the REPL section closes the
// page (ADR-0036 amendment: the REPL is the most interactive frontend,
// so its session-style blocks read best at the end).
const SECTIONS = [
  {
    heading: "ex-h-app",
    headingText: "TUI, desktop app, and web app",
    intro: "ex-app-intro",
    introText: "The same language runs in the terminal UI, the desktop app, and the web app. Copy/paste an example into the entry field and press Enter.",
    examples: [
      { cap: "ex-a8", capText: "A straightforward calculation. Multiplication happens before addition.", kind: "epher", code: `2 + 3 * 4` },
      { cap: "ex-a9", capText: "Powers and roots work like the rest of the language.", kind: "epher", code: `sqrt(2 ^ 10)` },
      { cap: "ex-a10", capText: "Exact fractions where binary floats would round.", kind: "epher", code: `frac(1, 3) + frac(1, 6)` },
      { cap: "ex-a11", capText: "Defining a function and calling it in the same line.", kind: "epher", code: `def sq(x) = x * x; sq(9)` },
      { cap: "ex-a12", capText: "Arithmetic across number bases: `0xff` is hex, `0b1` is binary, and `hex(...)` spells the answer as hex.", kind: "epher", code: `0xff + 0b1\nhex(ans)` },
      { cap: "ex-a1", capText: "A multi-line script. Shift+Enter starts a new line, Enter runs the whole script as one history item.", kind: "epher", code: `x = 10\ny = x + 5\ny ^ 2` },
      { cap: "ex-a2", capText: "A basic 2D curve.", kind: "epher", code: `graph x ^ 2` },
      { cap: "ex-a3", capText: "The region below the curve, shaded. `y >` shades above instead.", kind: "epher", code: `graph y < x ^ 2` },
      { cap: "ex-a4", capText: "The play button beside `a` animates the wave.", kind: "epher", code: `const a = 1\ngraph sin(x * a)` },
      { cap: "ex-a5", capText: "Two curves on one plot. Every `graph` line adds another.", kind: "epher", code: `graph x ^ 2\ngraph sin(x)` },
      { cap: "ex-a6", capText: "A 3D surface. Drag, swipe, or use the sliders to orbit.", kind: "epher", code: `graph3d sin(x) * cos(y)` },
      { cap: "ex-a7", capText: "A paraboloid bowl.", kind: "epher", code: `graph3d x ^ 2 + y ^ 2` },
    ],
  },
  {
    heading: "ex-h-cli",
    headingText: "The command line",
    intro: "ex-cli-intro",
    introText: "One-shot calculations, piped scripts, and SVG exports. Copy an example from those below into any shell.",
    examples: [
      { cap: "ex-4c", capText: "Piping a script into the epher command. stdin becomes the script, one line at a time.", kind: "sh", code: `printf 'x = 10\\nx ^ 2\\n' | epher -` },
      { cap: "ex-5c", capText: "Piping epher's output into another command.", kind: "sh", code: `epher "1 / 3" | tee third.txt` },
      { cap: "ex-6c", capText: "Several statements on one line, separated by semicolons.", kind: "sh", code: `epher "x = 7; x ^ 3"` },
      { cap: "ex-8c", capText: "A recursive factorial, defined inside a piped script.", kind: "sh", code: `printf 'def fact(n) = if n <= 1 then 1 else n * fact(n - 1); fact(10)' | epher -` },
      { cap: "ex-9c", capText: "A 2D graph saved as an SVG image file.", kind: "sh", code: `epher "graph x ^ 2; graph save plot.svg"` },
      { cap: "ex-10c", capText: "A 3D surface saved as an SVG image file.", kind: "sh", code: `epher "graph3d x ^ 2 - y ^ 2; graph3d save saddle.svg"` },
    ],
  },
  {
    heading: "ex-h-repl",
    headingText: "The REPL",
    intro: "ex-repl-intro",
    introText: "Start an interactive session with `epher repl` and type after the `epher>` prompt. These blocks run one after another in the same session. Each answer feeds the next line's `ans`.",
    examples: [
      { cap: "ex-r1", capText: "The first calculation of the session.", kind: "epher", code: `epher> 2 + 3\n= 5` },
      { cap: "ex-r2", capText: "`ans` holds the previous answer. Build on it.", kind: "epher", code: `epher> ans * 10\n= 50` },
      { cap: "ex-r3", capText: "`ans` follows every new answer down the session.", kind: "epher", code: `epher> ans + 2\n= 52` },
      { cap: "ex-r4", capText: "Use `ans` inside a larger expression.", kind: "epher", code: `epher> (ans - 12) ^ 2\n= 1600` },
    ],
  },
];

// English fallback strings live in the HTML itself (data-i18n); the
// per-language catalogs override them at runtime.
function exampleBlock(code, kind) {
  const cls = kind === "text" ? "language-text" : `language-${kind}`;
  const body =
    kind === "text" ? escapeHtml(code) : highlightEpher(code);
  return `<div class="example">
<pre tabindex="0"><code class="${cls}">${body}</code></pre>
<div class="example-bar">
<button type="button" class="copy-btn">
<svg class="icon-copy" aria-hidden="true" viewBox="0 0 24 24"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
<svg class="icon-check" aria-hidden="true" viewBox="0 0 24 24"><path d="M20 6 9 17l-5-5"/></svg>
<span class="copy-label" data-i18n="copy">Copy</span>
</button>
</div>
</div>`;
}

const body = SECTIONS.map(
  (s) => `<section class="ex-section">
<h2 data-i18n="${s.heading}">${escapeHtml(s.headingText)}</h2>
<p data-i18n="${s.intro}">${escapeHtml(s.introText)}</p>
${s.examples
  .map(
    (ex) => `<figure class="example-item">
<figcaption data-i18n="${ex.cap}">${escapeHtml(ex.capText)}</figcaption>
${exampleBlock(ex.code, ex.kind)}
</figure>`
  )
  .join("\n")}
</section>`
).join("\n");

const template = readFileSync(join(SITE, "index.html"), "utf8");
// index.html is the chrome donor: header, footer, catalogs, app.js.
// Replace the main content and the page markers.
const page = template
  .replace(
    "<!--\n  epher landing page (served at epher.org, see docs/website.md).",
    "<!--\n  epher Examples page. Built by scripts/build-examples.mjs from the\n  index.html chrome: header, disclosure nav, theme, i18n catalogs + app.js.\n  The prose is data-i18n keys (ex-*, nav-examples); the epher code blocks\n  are never localized (ADR-0007) and carry the guide's copy buttons; on\n  touch devices a tap on an example opens the app with it staged\n  (ADR-0035 amendment).\n  -->\n<!--"
  )
  .replace('<title>epher: a programmable, scriptable calculator</title>', '<title>Examples: epher</title>')
  .replace('<link rel="stylesheet" href="styles.css" />', '<link rel="stylesheet" href="styles.css" />\n  <link rel="stylesheet" href="guide.css" />')
  .replace(
    '<meta name="description" content="A programmable, scriptable calculator: command line, terminal UI, desktop app, and offline web app. Graphs in 2D and 3D, exact numerics, eight languages, no accounts." />',
    '<meta name="description" content="Copyable examples for every version of epher: app expressions, REPL sessions with ans, CLI commands, piped scripts, and 2D and 3D graphs." />'
  )
  // index.html's header nav already carries the Examples link (all site
  // pages do); mark it as the current page.
  .replace(
    '      <a href="examples.html" data-i18n="nav-examples">Examples</a>',
    '      <a href="examples.html" aria-current="page" data-i18n="nav-examples">Examples</a>'
  )
  .replace(
    /<main id="main">[\s\S]*?<\/main>/,
    `<main id="main">
    <article class="prose">
      <h1 data-i18n="examples-title">Examples</h1>
      <p class="lede" data-i18n="examples-intro">Copyable examples for every version of epher. Paste them into a terminal, a REPL session, or the app's entry field. The user guide explains the language in detail; this page is for quick copying and pasting.</p>
      <p class="note" data-i18n="ex-tap">On a phone, tap an example to open the app with it ready to run.</p>
${body}
    </article>
  </main>`
  )
  .replace(
    "</body>\n</html>",
    `<p class="visually-hidden" id="copy-status" role="status"></p>
<script>
  // Copy buttons (ADR-0036): the guide's pattern, reading the strings
  // from the active language catalog so they follow the page's locale.
  (function () {
    var live = document.getElementById("copy-status");
    if (!live) return;
    var strings = function () {
      var cat = window.EPHER_I18N && window.EPHER_I18N[document.documentElement.lang] || window.EPHER_I18N && window.EPHER_I18N.en || {};
      return { copy: cat.copy || "Copy", copied: cat.copied || "Copied" };
    };
    function copyToClipboard(text) {
      if (navigator.clipboard && navigator.clipboard.writeText) {
        return navigator.clipboard.writeText(text).then(function () { return true; }, function () { return fallback(text); });
      }
      return Promise.resolve(fallback(text));
    }
    function fallback(text) {
      try {
        var ta = document.createElement("textarea");
        ta.value = text;
        ta.setAttribute("readonly", "");
        ta.style.position = "fixed";
        ta.style.opacity = "0";
        document.body.appendChild(ta);
        ta.select();
        var ok = document.execCommand("copy");
        ta.remove();
        return ok;
      } catch (e) {
        return false;
      }
    }
    document.querySelectorAll(".copy-btn").forEach(function (btn) {
      var timer = null;
      btn.addEventListener("click", function (e) {
        e.stopPropagation();
        var example = btn.closest(".example");
        var code = example && example.querySelector("code");
        if (!code) return;
        copyToClipboard(code.textContent).then(function (ok) {
          if (!ok) return;
          var s = strings();
          var label = btn.querySelector(".copy-label");
          btn.classList.add("copied");
          label.textContent = s.copied;
          live.textContent = s.copied; // announce (role=status)
          clearTimeout(timer);
          timer = setTimeout(function () {
            btn.classList.remove("copied");
            label.textContent = strings().copy;
            live.textContent = "";
          }, 2000);
        });
      });
    });
    // On touch devices (ADR-0035 amendment): tapping an example copies it
    // and opens the app with the code staged in the entry field, ready to
    // run. The copy button itself stops propagation above, so tapping it
    // still only copies.
    var touch = "ontouchstart" in window &&
      window.matchMedia && window.matchMedia("(pointer: coarse)").matches;
    if (touch) {
      var KEY = "epher-example";
      document.querySelectorAll(".example").forEach(function (ex) {
        ex.addEventListener("click", function () {
          var code = ex.querySelector("code");
          if (!code) return;
          var text = code.textContent;
          try { localStorage.setItem(KEY, text); } catch (e) {}
          if (navigator.clipboard && navigator.clipboard.writeText) {
            navigator.clipboard.writeText(text).catch(function () {});
          }
          window.location.href = "pwa/";
        });
      });
    }
  })();
</script>
</body>
</html>`
  );

writeFileSync(join(SITE, "examples.html"), page);
console.log("built site/examples.html");
