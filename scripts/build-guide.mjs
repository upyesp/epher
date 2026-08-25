// Build the user guide: site/guide/<lang>.md → site/guide/<lang>/index.html
// (one static, localized, theme-capable HTML page per language).
//
// Run: npm run build:guide  (CI: pages.yml runs this before assembling _site)
//
// The pages share the landing page's styles.css + guide.css and follow the
// same accessibility requirements (WCAG 2.2 AA — skip link, labels, contrast,
// lang/dir per locale, focus-visible via the shared stylesheet).
import { marked } from "marked";
import { readFileSync, mkdirSync, writeFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..");
const GUIDE = join(ROOT, "site", "guide");

const LANGS = ["en", "zh-CN", "hi", "es", "fr", "ar", "de", "pt"];

// Per-language chrome strings (mirror the landing page dictionaries in
// site/app.js; the guide page itself is single-language so no runtime i18n).
// copy/copied label the example-block copy button (and its announcement).
const CHROME = {
  en: { title: "epher — User guide", app: "App", back: "Back to home", contents: "Contents", themeDark: "Use dark theme", themeLight: "Use light theme", footer: "epher user guide", copy: "Copy", copied: "Copied", about: "About", privacy: "Privacy", menu: "Menu" },
  "zh-CN": { title: "epher — 用户指南", app: "应用", back: "返回主页", contents: "目录", themeDark: "使用深色主题", themeLight: "使用浅色主题", footer: "epher 用户指南", copy: "复制", copied: "已复制", about: "关于", privacy: "隐私", menu: "菜单" },
  hi: { title: "epher — उपयोगकर्ता गाइड", app: "ऐप", back: "मुख्य पृष्ठ पर वापस जाएँ", contents: "विषय-सूची", themeDark: "गहरी थीम का उपयोग करें", themeLight: "हल्की थीम का उपयोग करें", footer: "epher उपयोगकर्ता गाइड", copy: "कॉपी करें", copied: "कॉपी हो गया", about: "परिचय", privacy: "गोपनीयता", menu: "मेनू" },
  es: { title: "epher — Guía de usuario", app: "App", back: "Volver al inicio", contents: "Contenido", themeDark: "Usar tema oscuro", themeLight: "Usar tema claro", footer: "Guía de usuario de epher", copy: "Copiar", copied: "Copiado", about: "Acerca de", privacy: "Privacidad", menu: "Menú" },
  fr: { title: "epher — Guide de l'utilisateur", app: "App", back: "Retour à l'accueil", contents: "Sommaire", themeDark: "Utiliser le thème sombre", themeLight: "Utiliser le thème clair", footer: "Guide de l'utilisateur de epher", copy: "Copier", copied: "Copié", about: "À propos", privacy: "Confidentialité", menu: "Menu" },
  ar: { title: "epher — دليل المستخدم", app: "التطبيق", back: "العودة إلى الصفحة الرئيسية", contents: "المحتويات", themeDark: "استخدام المظهر الداكن", themeLight: "استخدام المظهر الفاتح", footer: "دليل مستخدم epher", copy: "نسخ", copied: "تم النسخ", about: "حول", privacy: "الخصوصية", menu: "القائمة" },
  de: { title: "epher — Benutzerhandbuch", app: "App", back: "Zurück zur Startseite", contents: "Inhalt", themeDark: "Dunkles Design verwenden", themeLight: "Helles Design verwenden", footer: "epher-Benutzerhandbuch", copy: "Kopieren", copied: "Kopiert", about: "Über", privacy: "Datenschutz", menu: "Menü" },
  pt: { title: "epher — Guia de utilizador", app: "App", back: "Voltar ao início", contents: "Índice", themeDark: "Usar tema escuro", themeLight: "Usar tema claro", footer: "Guia de utilizador do epher", copy: "Copiar", copied: "Copiado", about: "Sobre", privacy: "Privacidade", menu: "Menu" },
};

// --- example code blocks ------------------------------------------------
//
// Guide fenced blocks come in two kinds (see docs/website.md):
//   ```epher / ```sh → what the reader types: a code block with lightweight
//                      syntax highlighting and a copy button (below)
//   ```text          → what epher answers, REPL transcripts, URLs, paths:
//                      the plain box, unchanged
// The highlighter is a tiny epher tokenizer (keywords, constants, numbers,
// strings, function calls); epher has no comment syntax.

const KEYWORDS = new Set([
  "def", "const", "if", "then", "else", "while", "do", "and", "or", "not",
  "graph", "save", "language", "quit",
]);
const CONSTANTS = new Set(["pi", "e", "tau", "phi", "true", "false"]);

// chrome strings of the language currently being rendered
let currentChrome = CHROME.en;

function escapeHtml(s) {
  return s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

const TOKEN = /(\d+(?:\.\d*)?(?:[eE][+-]?\d+)?|\.\d+)|([A-Za-z_][A-Za-z0-9_]*)|("[^"\n]*"|'[^'\n]*')|([\s\S])/g;

function highlightEpher(code) {
  let out = "";
  for (const m of code.matchAll(TOKEN)) {
    const [text, num, ident, str, ch] = m;
    if (num !== undefined) {
      out += `<span class="tok-num">${escapeHtml(text)}</span>`;
    } else if (str !== undefined) {
      out += `<span class="tok-str">${escapeHtml(text)}</span>`;
    } else if (ident !== undefined) {
      const after = code[m.index + text.length]; // function call: ident directly before (
      const cls = KEYWORDS.has(text)
        ? "tok-kw"
        : CONSTANTS.has(text)
          ? "tok-num"
          : after === "("
            ? "tok-fn"
            : null;
      out += cls ? `<span class="${cls}">${escapeHtml(text)}</span>` : escapeHtml(text);
    } else {
      out += escapeHtml(ch);
    }
  }
  return out;
}

function exampleBlock(code, info) {
  return `<div class="example">
<pre tabindex="0"><code class="language-${info}">${highlightEpher(code)}</code></pre>
<div class="example-bar">
<button type="button" class="copy-btn">
<svg class="icon-copy" aria-hidden="true" viewBox="0 0 24 24"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>
<svg class="icon-check" aria-hidden="true" viewBox="0 0 24 24"><path d="M20 6 9 17l-5-5"/></svg>
<span class="copy-label">${currentChrome.copy}</span>
</button>
</div>
</div>`;
}

marked.use({
  renderer: {
    code(code, infostring) {
      const info = (infostring || "").trim().split(/\s+/)[0];
      // tabindex: long lines scroll horizontally — keyboard users must be
      // able to focus and scroll the region (axe scrollable-region-focusable)
      if (info === "epher" || info === "sh") return exampleBlock(code, info);
      return `<pre tabindex="0"><code class="language-text">${escapeHtml(code)}\n</code></pre>`;
    },
  },
});

const usedIds = new Set();
function slugify(text) {
  let slug = text
    .toLowerCase()
    .replace(/[^a-z0-9\u0600-\u06FF\u4E00-\u9FFF\u0900-\u097F\s-]/g, "")
    .trim()
    .replace(/\s+/g, "-");
  if (!slug) slug = "section";
  let id = slug;
  let n = 2;
  while (usedIds.has(id)) id = `${slug}-${n++}`;
  usedIds.add(id);
  return id;
}

function postprocess(html) {
  // wrap tables for horizontal scroll (mobile + 200% zoom); the wrap gets
  // tabindex="0" so the scrollable region is keyboard-focusable
  // (WCAG 2.1.1 / axe scrollable-region-focusable)
  html = html.replace(
    /<table>/g,
    '<div class="table-wrap" tabindex="0"><table>'
  ).replace(
    /<\/table>/g,
    "</table></div>"
  );

  // heading ids + TOC entries
  const toc = [];
  html = html.replace(/<h([1-4])>(.*?)<\/h\1>/g, (_, level, inner) => {
    const text = inner.replace(/<[^>]*>/g, "");
    const id = slugify(text);
    if (level >= 2 && level <= 3) {
      toc.push(`<li class="toc-l${level}"><a href="#${id}">${text}</a></li>`);
    }
    return `<h${level} id="${id}">${inner}</h${level}>`;
  });
  return { html, toc };
}

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

function page(lang, body, toc) {
  const c = CHROME[lang];
  const dir = lang === "ar" ? ' dir="rtl"' : "";
  return `<!DOCTYPE html>
<html lang="${lang}"${dir}>
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>${c.title}</title>
  <meta name="theme-color" media="(prefers-color-scheme: light)" content="#ffffff" />
  <meta name="theme-color" media="(prefers-color-scheme: dark)" content="#141416" />
  <link rel="icon" href="../../icon.svg?v=3" type="image/svg+xml" />
  <link rel="stylesheet" href="../../styles.css" />
  <link rel="stylesheet" href="../../guide.css" />
  ${themeScript()}
  <noscript>
      <style>
        /* without JavaScript the disclosure button cannot work: show the
           links as a plain stacked row instead (progressive enhancement) */
        .menu-toggle { display: none; }
        .site-nav { display: flex !important; flex-direction: column; position: static;
                    border: 0; box-shadow: none; padding: 0.5rem 0; }
      </style>
    </noscript>
</head>
<body>
  <a class="skip-link" href="#main">${c.back}</a>
  <header class="site-header guide-header">
    <a class="brand" href="../../">
      <img class="brand-icon" id="brand-icon" src="../../icon.svg?v=3" alt="" width="32" height="32" />
      <span>epher</span>
    </a>
    <nav class="site-nav" id="site-nav" hidden aria-label="epher">
      <a href="../../about.html">${c.about}</a>
      <a href="../../">${c.back}</a>
      <a href="/pwa/">${c.app}</a>
      <a href="../../privacy.html">${c.privacy}</a>
    </nav>
    <div class="header-controls">
      <button type="button" id="theme-toggle" class="icon-btn" aria-pressed="false" aria-label="${c.themeDark}">
        <svg class="icon-moon" aria-hidden="true" viewBox="0 0 24 24"><path d="M21 12.8A9 9 0 1 1 11.2 3 7 7 0 0 0 21 12.8Z" /></svg>
        <svg class="icon-sun" aria-hidden="true" viewBox="0 0 24 24"><circle cx="12" cy="12" r="4.5" /><path d="M12 2v2.5M12 19.5V22M2 12h2.5M19.5 12H22M4.9 4.9l1.8 1.8M17.3 17.3l1.8 1.8M19.1 4.9l-1.8 1.8M6.7 17.3l-1.8 1.8" /></svg>
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
      ${body}
    </div>
  </main>

  <footer class="site-footer">
    <nav class="footer-links" aria-label="${c.footer}">
      <a href="../../about.html">${c.about}</a>
      <a href="../../">${c.back}</a>
      <a href="/pwa/">${c.app}</a>
      <a href="../../privacy.html">${c.privacy}</a>
    </nav>
    <p class="muted">${c.footer}</p>
  </footer>

  <p class="visually-hidden" id="copy-status" role="status"></p>

  <script>
    // theme toggle (guide pages are single-language; no i18n needed here)
    (function () {
      var toggle = document.getElementById("theme-toggle");
      var labels = ${JSON.stringify({ dark: CHROME[lang].themeDark, light: CHROME[lang].themeLight })};
      function setTheme(t) {
        document.documentElement.dataset.theme = t;
        try { localStorage.setItem("epher-theme", t); } catch (e) {}
        var next = t === "dark" ? "light" : "dark";
        toggle.setAttribute("aria-label", labels[next]);
        // brand mark flips tile colors with the theme; the CSS content:url
        // rule (styles.css) handles Chrome/Firefox, this keeps Safari right
        var brand = document.getElementById("brand-icon");
        if (brand) brand.src = t === "dark" ? "../../icon-light.svg?v=2" : "../../icon.svg?v=2";
      }
      toggle.addEventListener("click", function () {
        setTheme(document.documentElement.dataset.theme === "dark" ? "light" : "dark");
      });
      // disclosure nav (mobile): same pattern as app.js on the site pages
      var menuBtn = document.getElementById("menu-toggle");
      var nav = document.getElementById("site-nav");
      if (menuBtn && nav) {
        var setMenu = function (open) {
          menuBtn.setAttribute("aria-expanded", String(open));
          nav.hidden = !open;
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
    })();
  </script>

  <script>
    // copy buttons on example code blocks (epher/sh fenced blocks in the md)
    (function () {
      var strings = ${JSON.stringify({ copy: CHROME[lang].copy, copied: CHROME[lang].copied })};
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

      // older browsers / non-secure contexts
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
            live.textContent = strings.copied; // announce (role=status)
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
</html>`;
}

let built = 0;
for (const lang of LANGS) {
  usedIds.clear();
  currentChrome = CHROME[lang];
  const md = readFileSync(join(GUIDE, `${lang}.md`), "utf8");
  const { html, toc } = postprocess(marked.parse(md, { gfm: true, breaks: false }));
  const outDir = join(GUIDE, lang);
  mkdirSync(outDir, { recursive: true });
  writeFileSync(join(outDir, "index.html"), page(lang, html, toc));
  built++;
}
console.log(`built ${built} guide pages (${LANGS.join(", ")})`);
