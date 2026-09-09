// Build the language reference: site/reference.md → site/reference/index.html
//
// Run: npm run build:reference  (CI: site-build.yml runs this with the others)
//
// The reference is the formal, normative definition of the epher language
// (English only — it is the specification, not a tutorial; the localized
// user guide teaches). The page shares the guide's renderer, styles, and
// accessibility chrome: same code highlighting, copy buttons, table
// wrapping, heading ids, TOC, theme handling, disclosure nav.
import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { marked } from "marked";
import { postprocess, highlightEpher } from "./build-guide.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");

// chrome strings for the single-language page (mirror build-guide.mjs's
// en entry; c.reference labels the Docs-menu link back to this page)
const c = {
  title: "epher: Language reference",
  app: "App",
  back: "Back to home",
  contents: "Contents",
  themeDark: "Use dark theme",
  themeLight: "Use light theme",
  footer: "epher language reference",
  copy: "Copy",
  copied: "Copied",
  privacy: "Privacy",
  menu: "Menu",
  examples: "Examples",
  docs: "Docs",
  guide: "User guide",
  scripts: "Scripts",
  reference: "Language reference",
};

const md = readFileSync(join(ROOT, "site", "reference.md"), "utf8");
const { html, toc } = postprocess(marked.parse(md, { gfm: true, breaks: false }));

function themeScript() {
  return `<script>
  (function () {
    try {
      var theme = localStorage.getItem("epher-theme");
      if (theme !== "light" && theme !== "dark") {
        theme = window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
      }
      document.documentElement.dataset.theme = theme;
    } catch (e) {
      document.documentElement.dataset.theme = "light";
    }
  })();
</script>`;
}

const out = `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>${c.title}</title>
  <meta name="description" content="The formal definition of the epher language: grammar, types, operators, statements, built-in constants and functions, units, display rules, errors, and limits." />
  <meta name="theme-color" media="(prefers-color-scheme: light)" content="#ffffff" />
  <meta name="theme-color" media="(prefers-color-scheme: dark)" content="#141416" />
  <link rel="icon" href="../icon.svg?v=3" type="image/svg+xml" />
  <link rel="stylesheet" href="../styles.css" />
  <link rel="stylesheet" href="../guide.css" />
  ${themeScript()}
  <noscript>
      <style>
        .menu-toggle { display: none; }
        .docs-toggle { display: none; }
        .site-nav { display: flex !important; flex-direction: column; position: static;
                    border: 0; box-shadow: none; padding: 0.5rem 0; }
        .docs { position: static; }
        .docs-menu { display: flex !important; position: static; padding-inline-start: 1rem;
                     border: 0; box-shadow: none; }
      </style>
    </noscript>
</head>
<body>
  <a class="skip-link" href="#main">${c.back}</a>
  <header class="site-header guide-header">
    <a class="brand" href="../">
      <img class="brand-icon" id="brand-icon" src="../icon.svg?v=3" alt="" width="32" height="32" />
      <span>epher</span>
    </a>
    <nav class="site-nav" id="site-nav" hidden aria-label="epher">
      <a href="/pwa/">${c.app}</a>
      <div class="docs">
        <button type="button" class="docs-toggle" id="docs-toggle" aria-expanded="false" aria-controls="docs-menu" aria-haspopup="true">
          <span>${c.docs}</span>
          <svg class="icon-chevron" aria-hidden="true" viewBox="0 0 24 24"><path d="M6 9l6 6 6-6" /></svg>
        </button>
        <div class="docs-menu" id="docs-menu" hidden>
          <a href="../guide/en/">${c.guide}</a>
          <a href="../examples.html">${c.examples}</a>
          <a href="../scripts.html">${c.scripts}</a>
          <a href="../reference/" aria-current="page">${c.reference}</a>
        </div>
      </div>
      <a href="../privacy.html">${c.privacy}</a>
    </nav>
    <div class="header-controls">
      <button type="button" id="theme-toggle" class="icon-btn" aria-pressed="false" aria-label="${c.themeDark}">
        <svg class="icon-moon" aria-hidden="true" viewBox="0 0 24 24"><path d="M21 12.8A9 9 0 1 1 11.2 3 7 7 0 0 0 21 12.8Z" /></svg>
        <svg class="icon-sun" aria-hidden="true" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4.5" /><path d="M12 2v2.5M12 19.5V22M2 12h2.5M19.5 12H22M4.9 4.9l1.8 1.8M17.3 17.3l1.8 1.8M6.7 6.7l1.8 1.8" /></svg>
        <span class="visually-hidden">${c.themeDark}</span>
      </button>
    </div>
    <button type="button" id="menu-toggle" class="menu-toggle" aria-expanded="false" aria-controls="site-nav">
      <svg class="icon-burger" aria-hidden="true" viewBox="0 0 24 24"><path d="M3 6h18M3 12h18M3 18h18" /></svg>
      <span class="visually-hidden">${c.menu}</span>
    </button>
  </header>

  <main id="main" class="guide">
    <nav class="toc" aria-label="${c.contents}" tabindex="0">
      <h2 class="toc-title">${c.contents}</h2>
      <ul>${toc.join("")}</ul>
    </nav>
    <div class="guide-body">
      ${html}
    </div>
  </main>

  <footer class="site-footer">
    <nav class="footer-links" aria-label="${c.footer}">
      <a href="/pwa/">${c.app}</a>
      <a href="../">${c.back}</a>
      <a href="../examples.html">${c.examples}</a>
      <a href="../scripts.html">${c.scripts}</a>
      <a href="../reference/" aria-current="page">${c.reference}</a>
      <a href="../privacy.html">${c.privacy}</a>
    </nav>
    <p class="muted">${c.footer}</p>
  </footer>

  <p class="visually-hidden" id="copy-status" role="status"></p>

  <script>
    (function () {
      var toggle = document.getElementById("theme-toggle");
      var labels = ${JSON.stringify({ dark: c.themeDark, light: c.themeLight })};
      function setTheme(t) {
        document.documentElement.dataset.theme = t;
        try { localStorage.setItem("epher-theme", t); } catch (e) {}
        var next = t === "dark" ? "light" : "dark";
        toggle.setAttribute("aria-label", labels[next]);
        var brand = document.getElementById("brand-icon");
        if (brand) brand.src = t === "dark" ? "../icon-light.svg?v=2" : "../icon.svg?v=2";
      }
      toggle.addEventListener("click", function () {
        setTheme(document.documentElement.dataset.theme === "dark" ? "light" : "dark");
      });
      var menuBtn = document.getElementById("menu-toggle");
      var nav = document.getElementById("site-nav");
      var docsBtn = document.getElementById("docs-toggle");
      var docsMenu = document.getElementById("docs-menu");
      var setDocs = function (open) {
        if (!docsBtn) return;
        docsBtn.setAttribute("aria-expanded", String(open));
        docsMenu.hidden = !open;
      };
      if (menuBtn && nav) {
        var setMenu = function (open) {
          menuBtn.setAttribute("aria-expanded", String(open));
          nav.hidden = !open;
          if (!open) setDocs(false);
        };
        menuBtn.addEventListener("click", function () {
          setMenu(menuBtn.getAttribute("aria-expanded") !== "true");
        });
        document.addEventListener("keydown", function (e) {
          if (e.key === "Escape" && menuBtn.getAttribute("aria-expanded") === "true") {
            setMenu(false);
            menuBtn.focus();
          }
        });
        document.addEventListener("click", function (e) {
          if (menuBtn.getAttribute("aria-expanded") === "true" &&
              !nav.contains(e.target) && !menuBtn.contains(e.target)) {
            setMenu(false);
          }
        });
        nav.addEventListener("click", function (e) {
          if (e.target.closest("a")) setMenu(false);
        });
      }
      if (docsBtn) {
        docsBtn.addEventListener("click", function (e) {
          e.stopPropagation();
          setDocs(docsBtn.getAttribute("aria-expanded") !== "true");
        });
        document.addEventListener("keydown", function (e) {
          if (e.key === "Escape" && docsBtn.getAttribute("aria-expanded") === "true") {
            setDocs(false);
            docsBtn.focus();
          }
        });
        document.addEventListener("click", function (e) {
          if (docsBtn.getAttribute("aria-expanded") === "true" &&
              !docsBtn.contains(e.target) && !docsMenu.contains(e.target)) {
            setDocs(false);
          }
        });
      }
    })();
  </script>

  <script>
    (function () {
      var strings = ${JSON.stringify({ copy: c.copy, copied: c.copied })};
      var live = document.getElementById("copy-status");

      function copyToClipboard(text) {
        if (navigator.clipboard && navigator.clipboard.writeText) {
          return navigator.clipboard.writeText(text).then(
            function () { return true; },
            function () { return fallback(text); }
          );
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
        btn.addEventListener("click", function () {
          var example = btn.closest(".example");
          var code = example && example.querySelector("code");
          if (!code) return;
          copyToClipboard(code.textContent).then(function (ok) {
            if (!ok) return;
            var label = btn.querySelector(".copy-label");
            btn.classList.add("copied");
            label.textContent = strings.copied;
            live.textContent = strings.copied;
            clearTimeout(timer);
            timer = setTimeout(function () {
              btn.classList.remove("copied");
              label.textContent = strings.copy;
              live.textContent = "";
            }, 2000);
          });
        });
      });
    })();
  </script>
</body>
</html>
`;

const outDir = join(ROOT, "site", "reference");
mkdirSync(outDir, { recursive: true });
writeFileSync(join(outDir, "index.html"), out);
console.log(`built site/reference/index.html (${toc.length} toc entries)`);
