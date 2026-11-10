/* epher website: i18n + theme + disclosure nav.
 *
 * i18n mirrors the Localizer in crates/i18n: device locale auto-detection,
 * a stored user preference, English fallback. The catalogs live in
 * i18n/<lang>.js (loaded before this file as plain scripts; no fetch, so
 * the page also works offline from any mount point); this file applies
 * them by data-i18n / data-i18n-aria / data-i18n-href attributes.
 *
 * Theme: light/dark, defaults to prefers-color-scheme, toggle persisted in
 * localStorage ("epher-theme"). Language preference: "epher-lang".
 *
 * Nav: below 880px the header links collapse into a disclosure menu
 * (WAI-ARIA APG pattern): the button carries aria-expanded, Escape closes
 * it and restores focus, a click outside closes it, and the links are
 * removed from the tab order while closed (the `hidden` attribute).
 */
"use strict";

const SUPPORTED = ["en", "zh-CN", "hi", "es", "fr", "ar", "de", "pt"];

const MESSAGES = window.EPHER_I18N || { en: {} };

function normalize(code) {
  return code.replace("_", "-").toLowerCase();
}

/** Negotiate a supported locale from the device's languages. The static
 *  twin of `Localizer::resolve` in crates/i18n: exact match first, then
 *  language-prefix match, English fallback. */
function detect() {
  const wanted = (navigator.languages || [navigator.language || "en"]).map(normalize);
  for (const w of wanted) {
    const hit = SUPPORTED.find((s) => s.toLowerCase() === w);
    if (hit) return hit;
  }
  for (const w of wanted) {
    const prefix = w.split("-")[0];
    const hit = SUPPORTED.find((s) => s.toLowerCase() === prefix);
    if (hit) return hit;
  }
  return "en";
}

let currentLang = "en";

function applyLang(lang) {
  currentLang = lang;
  const dict = MESSAGES[lang] || MESSAGES.en || {};
  document.querySelectorAll("[data-i18n]").forEach((el) => {
    const key = el.getAttribute("data-i18n");
    // An empty-string value clears the element (a fragment that only
    // exists in some locales, such as a link's surrounding words);
    // a missing key keeps the element's static English text.
    if (dict[key] !== undefined) el.textContent = dict[key];
  });
  document.querySelectorAll("[data-i18n-aria]").forEach((el) => {
    const key = el.getAttribute("data-i18n-aria");
    if (dict[key]) el.setAttribute("aria-label", dict[key]);
  });
  // links whose target depends on the active locale (the user guide).
  // Swap ONLY the locale segment, keeping the original depth prefix:
  // the authored hrefs are correct per page ("guide/en/" at the root,
  // "../../guide/en/" on nested pages), and a blanket relative reset
  // broke every nested page after navigating.
  document.querySelectorAll("[data-i18n-href]").forEach((el) => {
    el.href = el.getAttribute("href").replace(/guide\/[A-Za-z-]+\//, `guide/${lang}/`);
  });
  // WCAG 3.1.1: lang (and dir for Arabic) must track the active locale.
  document.documentElement.lang = lang;
  document.documentElement.dir = lang === "ar" ? "rtl" : "ltr";
  document.getElementById("lang-select").value = lang;
}

function setTheme(theme) {
  document.documentElement.dataset.theme = theme;
  try {
    localStorage.setItem("epher-theme", theme);
  } catch (e) {
    /* private mode: ignore */
  }
  // The toggle's label names the theme it switches TO.
  const next = theme === "dark" ? "light" : "dark";
  const key = next === "dark" ? "theme-dark" : "theme-light";
  const label = (MESSAGES[currentLang] || MESSAGES.en || {})[key];
  const toggle = document.getElementById("theme-toggle");
  if (label) toggle.setAttribute("aria-label", label);
  const hidden = toggle.querySelector(".visually-hidden");
  if (hidden && label) hidden.textContent = label;
  // The brand mark flips tile colors with the theme (the CSS content:url
  // rule handles Chrome/Firefox; this keeps the src right for Safari).
  const brand = document.getElementById("brand-icon");
  if (brand) brand.src = theme === "dark" ? "icon-light.svg?v=3" : "icon.svg?v=3";
}

/** Disclosure nav (mobile): open/close the collapsed header links. */
function initMenu() {
  const button = document.getElementById("menu-toggle");
  const nav = document.getElementById("site-nav");
  if (!button || !nav) return;

  const setMenu = (open) => {
    button.setAttribute("aria-expanded", String(open));
    // `hidden` takes the links out of the tab order and the a11y tree;
    // the desktop stylesheet overrides it back to visible (author rules
    // beat the UA's [hidden] { display: none }).
    nav.hidden = !open;
    if (!open) setDocs(false);
  };

  button.addEventListener("click", () => {
    setMenu(button.getAttribute("aria-expanded") !== "true");
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && button.getAttribute("aria-expanded") === "true") {
      setMenu(false);
      button.focus();
    }
  });

  document.addEventListener("click", (e) => {
    if (
      button.getAttribute("aria-expanded") === "true" &&
      !nav.contains(e.target) &&
      !button.contains(e.target)
    ) {
      setMenu(false);
    }
  });

  // following a link closes the menu (same-page anchors included)
  nav.addEventListener("click", (e) => {
    if (e.target.closest("a")) setMenu(false);
  });
}

/** Docs disclosure (desktop dropdown, and inside the mobile menu): the
 *  button toggles its panel; Escape and outside clicks close it. */
function initDocs() {
  const button = document.getElementById("docs-toggle");
  const panel = document.getElementById("docs-menu");
  if (!button || !panel) return;

  const setDocs = (open) => {
    button.setAttribute("aria-expanded", String(open));
    panel.hidden = !open;
  };

  button.addEventListener("click", (e) => {
    e.stopPropagation();
    setDocs(button.getAttribute("aria-expanded") !== "true");
  });

  document.addEventListener("keydown", (e) => {
    if (e.key === "Escape" && button.getAttribute("aria-expanded") === "true") {
      setDocs(false);
      button.focus();
    }
  });

  document.addEventListener("click", (e) => {
    if (
      button.getAttribute("aria-expanded") === "true" &&
      !button.contains(e.target) &&
      !panel.contains(e.target)
    ) {
      setDocs(false);
    }
  });
}

/* --- Linux downloads card: the five install tabs (ADR-0061) -----------
 * A WAI-ARIA tabs pattern like the nav disclosure: click or arrow-key
 * selects; the selected tab shows its panel and hides the others (the
 * `hidden` attribute keeps them out of the tab order). No library, no
 * framework; the card works with JS disabled only as far as the first
 * panel, which is the recommended Debian/Ubuntu path. */
function initLinuxTabs() {
  const card = document.querySelector(".linux-tabs");
  if (!card) return;
  const tabs = Array.from(card.querySelectorAll("[role=tab]"));
  const panels = Array.from(card.querySelectorAll(".tabpanel"));

  const select = (tab, focus) => {
    for (const t of tabs) {
      const on = t === tab;
      t.setAttribute("aria-selected", String(on));
      t.tabIndex = on ? 0 : -1;
    }
    for (const p of panels) {
      p.hidden = p.id !== tab.getAttribute("aria-controls");
    }
    if (focus) tab.focus();
  };

  for (const tab of tabs) {
    tab.addEventListener("click", () => select(tab, false));
    tab.addEventListener("keydown", (e) => {
      const at = tabs.indexOf(tab);
      let next = null;
      if (e.key === "ArrowRight" || e.key === "ArrowDown") {
        next = tabs[(at + 1) % tabs.length];
      } else if (e.key === "ArrowLeft" || e.key === "ArrowUp") {
        next = tabs[(at - 1 + tabs.length) % tabs.length];
      } else if (e.key === "Home") {
        next = tabs[0];
      } else if (e.key === "End") {
        next = tabs[tabs.length - 1];
      }
      if (next) {
        e.preventDefault();
        select(next, true);
      }
    });
  }
}

function init() {
  let stored = null;
  try {
    stored = localStorage.getItem("epher-lang");
  } catch (e) {
    /* ignore */
  }
  applyLang(stored && SUPPORTED.includes(stored) ? stored : detect());

  const theme =
    document.documentElement.dataset.theme === "dark" ? "dark" : "light";
  setTheme(theme);

  document.getElementById("theme-toggle").addEventListener("click", () => {
    const current = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
    setTheme(current);
  });

  document.getElementById("lang-select").addEventListener("change", (e) => {
    applyLang(e.target.value);
    try {
      localStorage.setItem("epher-lang", e.target.value);
    } catch (err) {
      /* ignore */
    }
    setTheme(document.documentElement.dataset.theme); // refresh toggle label
  });

  initMenu();
  initDocs();
  initLinuxTabs();
}


document.addEventListener("DOMContentLoaded", init);

// Copy buttons on the install-command blocks (ide.html): one click
// copies the whole block, and the button flips to a check mark for a
// moment - no dialog, no selection, no i18n strings (icon only, with
// an aria-label).
function enhanceCommandBlocks() {
  const COPY_ICON =
    '<svg class="copy-icon" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">' +
    '<rect x="5.5" y="5.5" width="8" height="8" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.4"/>' +
    '<path d="M3.5 10.5h-1A1.5 1.5 0 0 1 1 9V2.5A1.5 1.5 0 0 1 2.5 1H9a1.5 1.5 0 0 1 1.5 1.5v1" fill="none" stroke="currentColor" stroke-width="1.4"/></svg>';
  const CHECK_ICON =
    '<svg class="copy-check" viewBox="0 0 16 16" width="14" height="14" aria-hidden="true">' +
    '<path d="M2.5 8.5 6.5 12.5 13.5 3.5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/></svg>';

  async function writeClipboard(text) {
    if (navigator.clipboard && window.isSecureContext) {
      await navigator.clipboard.writeText(text);
      return;
    }
    // file:// and old browsers: the selection fallback.
    const area = document.createElement("textarea");
    area.value = text;
    area.setAttribute("readonly", "");
    area.style.position = "fixed";
    area.style.opacity = "0";
    document.body.appendChild(area);
    area.select();
    try {
      document.execCommand("copy");
    } finally {
      area.remove();
    }
  }

  document.querySelectorAll("pre.path-cmd, pre.repo-cmd").forEach((pre) => {
    if (pre.querySelector(".copy-btn") || pre.parentElement.classList.contains("cmd-wrap")) return;
    const commands = pre.textContent;
    // The button lives on a wrapper OUTSIDE the scrolling pre: it stays
    // put while long lines scroll underneath, and never covers code text
    // on the multi-line blocks (the Linux repo commands). Same look and
    // behavior for every block - the macOS ones and these.
    const wrap = document.createElement("div");
    wrap.className = "cmd-wrap";
    pre.replaceWith(wrap);
    wrap.appendChild(pre);
    // the pre scrolls horizontally: with the button no longer inside it,
    // the region needs its own keyboard access (WCAG 2.1.1, axe
    // scrollable-region-focusable)
    pre.setAttribute("tabindex", "0");
    const button = document.createElement("button");
    button.type = "button";
    button.className = "copy-btn";
    button.setAttribute("aria-label", "Copy commands");
    button.setAttribute("title", "Copy");
    button.innerHTML = COPY_ICON + CHECK_ICON;
    button.addEventListener("click", async () => {
      try {
        await writeClipboard(commands);
        wrap.classList.add("copied");
        setTimeout(() => wrap.classList.remove("copied"), 1600);
      } catch (e) {
        /* clipboard unavailable (permissions): leave the block as-is */
      }
    });
    wrap.appendChild(button);
  });
}

enhanceCommandBlocks();
