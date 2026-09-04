/* epher Scripts page browser (scripts.html): fetch scripts-data.json
 * (written by scripts/build-scripts.mjs from the repository's
 * "epher scripts" folder), browse field -> area -> script, search across
 * names and content, view any script, and copy any script with a button
 * carrying the ubiquitous copy icon: two overlapping sheets, sized to
 * the surrounding font.
 *
 * Localized strings come from the same catalogs as the rest of the site
 * (window.EPHER_I18N, applied by app.js); the current language is read
 * from document.documentElement.lang, which app.js keeps current.
 */
"use strict";

(function () {
  const I18N = window.EPHER_I18N || {};
  const dict = () => I18N[document.documentElement.lang] || I18N.en || {};
  const t = (key, fallback) => dict()[key] || fallback;

  // The ubiquitous copy icon: two overlapping sheets (rounded rects).
  const copySvg = () =>
    '<svg class="icon-copy" aria-hidden="true" viewBox="0 0 24 24"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>';
  const checkSvg =
    '<svg class="icon-check" aria-hidden="true" viewBox="0 0 24 24"><path d="M20 6 9 17l-5-5"/></svg>';

  const el = (tag, cls, text) => {
    const node = document.createElement(tag);
    if (cls) node.className = cls;
    if (text !== undefined) node.textContent = text;
    return node;
  };

  let tree = null; // { fields: [{ name, areas: [{ name, scripts: [...] }] }] }
  let flat = [];   // every script with { field, area, ...script }
  const browser = document.getElementById("scripts-browser");
  const crumbs = document.getElementById("scripts-crumbs");
  const searchInput = document.getElementById("scripts-search");

  /* --- clipboard -------------------------------------------------------- */

  const live = document.getElementById("copy-status");
  function copyText(text) {
    if (navigator.clipboard && navigator.clipboard.writeText) {
      return navigator.clipboard.writeText(text).then(
        () => true,
        () => fallbackCopy(text)
      );
    }
    return Promise.resolve(fallbackCopy(text));
  }
  function fallbackCopy(text) {
    try {
      const ta = document.createElement("textarea");
      ta.value = text;
      ta.setAttribute("readonly", "");
      ta.style.position = "fixed";
      ta.style.opacity = "0";
      document.body.appendChild(ta);
      ta.select();
      const ok = document.execCommand("copy");
      ta.remove();
      return ok;
    } catch (e) {
      return false;
    }
  }

  /** A copy button: icon-only with a localized accessible name; swaps to
   *  the check mark for a moment after a successful copy. The source is
   *  a script object (copy the whole file) or a string (copy that text,
   *  as the per-OS run commands do). */
  function copyButton(source) {
    const text = typeof source === "string" ? source : source.text;
    const btn = el("button", "copy-btn");
    btn.type = "button";
    btn.title = t("copy", "Copy");
    btn.setAttribute("aria-label", t("copy", "Copy"));
    btn.innerHTML = copySvg() + checkSvg + '<span class="copy-label"></span>';
    let timer = null;
    btn.addEventListener("click", (e) => {
      e.stopPropagation();
      copyText(text).then((ok) => {
        if (!ok) return;
        btn.classList.add("copied");
        const label = btn.querySelector(".copy-label");
        label.textContent = t("copied", "Copied");
        if (live) live.textContent = t("copied", "Copied");
        clearTimeout(timer);
        timer = setTimeout(() => {
          btn.classList.remove("copied");
          label.textContent = "";
          if (live) live.textContent = "";
        }, 2000);
      });
    });
    return btn;
  }

  /* --- state and rendering ---------------------------------------------- */

  function parseHash() {
    const parts = decodeURIComponent(location.hash || "#/")
      .replace(/^#\//, "")
      .split("/")
      .filter(Boolean);
    return { field: parts[0] || null, area: parts[1] || null, script: parts[2] || null };
  }

  function setHash(field, area, script) {
    const path = ["", field, area, script].filter((p) => p !== null && p !== undefined).join("/");
    if (location.hash !== "#" + path) location.hash = "#" + path;
    else render();
  }

  function fieldOf(name) {
    return tree.fields.find((f) => f.name === name) || null;
  }
  function areaOf(field, name) {
    const f = fieldOf(field);
    return f ? f.areas.find((a) => a.name === name) || null : null;
  }
  function scriptOf(field, area, name) {
    const a = areaOf(field, area);
    return a ? a.scripts.find((s) => s.name === name) || null : null;
  }

  function scriptCount(fieldName, areaName) {
    const f = fieldOf(fieldName);
    if (!f) return 0;
    if (areaName) {
      const a = areaOf(fieldName, areaName);
      return a ? a.scripts.length : 0;
    }
    return f.areas.reduce((n, a) => n + a.scripts.length, 0);
  }

  function crumb(label, href, current) {
    const a = el("a", current ? "crumb current" : "crumb", label);
    if (href) a.href = href;
    a.setAttribute("aria-current", current ? "page" : "false");
    return a;
  }

  function renderCrumbs(state) {
    crumbs.textContent = "";
    crumbs.appendChild(crumb("epher scripts", "#/", !state.field));
    if (state.field) {
      const sep = el("span", "crumb-sep", "/");
      crumbs.appendChild(sep);
      crumbs.appendChild(crumb(state.field, `#/${state.field}`, !state.area));
      if (state.area) {
        const sep2 = el("span", "crumb-sep", "/");
        crumbs.appendChild(sep2);
        crumbs.appendChild(
          crumb(state.area, `#/${state.field}/${state.area}`, !state.script)
        );
      }
    }
  }

  function render() {
    const state = parseHash();
    renderCrumbs(state);
    if (state.field && state.area && state.script) return renderScript(state);
    if (state.field && state.area) return renderScripts(state);
    if (state.field) return renderAreas(state);
    return renderFields();
  }

  function renderFields() {
    browser.textContent = "";
    const grid = el("div", "folder-grid");
    for (const field of tree.fields) {
      const card = el("a", "folder-card");
      card.href = `#/${field.name}`;
      card.appendChild(el("span", "folder-name", field.name));
      card.appendChild(
        el("span", "folder-meta", `${field.areas.length} topic${field.areas.length === 1 ? "" : "s"} · ${scriptCount(field.name)} scripts`)
      );
      grid.appendChild(card);
    }
    browser.appendChild(grid);
  }

  function renderAreas(state) {
    browser.textContent = "";
    const field = fieldOf(state.field);
    if (!field) return renderFields();
    const head = el("h2", "", state.field);
    browser.appendChild(head);
    const grid = el("div", "folder-grid");
    for (const area of field.areas) {
      const card = el("a", "folder-card");
      card.href = `#/${state.field}/${area.name}`;
      card.appendChild(el("span", "folder-name", area.name));
      card.appendChild(
        el("span", "folder-meta", `${area.scripts.length} script${area.scripts.length === 1 ? "" : "s"}`)
      );
      grid.appendChild(card);
    }
    browser.appendChild(grid);
  }

  function renderScripts(state) {
    browser.textContent = "";
    const area = areaOf(state.field, state.area);
    if (!area) return renderFields();
    const list = el("ul", "script-list");
    for (const script of area.scripts) {
      const row = el("li", "script-row");
      row.appendChild(copyButton(script));
      const link = el("a", "script-link", script.name);
      link.href = `#/${state.field}/${state.area}/${script.name}`;
      row.appendChild(link);
      if (script.title) row.appendChild(el("span", "script-title", script.title));
      list.appendChild(row);
    }
    browser.appendChild(list);
  }

  /** The per-script run box: the installed path, one command per
   *  operating system (the installers ship the collection beside the
   *  program), each copyable on its own. */
  function runBox(script) {
    const rel = script.rel || script.path.replace(/^epher scripts\//, "");
    const win = rel.split("/").join("\\");
    const box = el("div", "script-run");
    const commands = [
      ["Linux (deb, rpm)", `epher /usr/lib/epher/scripts/${rel}`],
      ["Windows", `epher "$env:LOCALAPPDATA\\epher\\scripts\\${win}"`],
      ["macOS", `epher /Applications/epher.app/Contents/Resources/scripts/${rel}`],
    ];
    for (const [osName, cmd] of commands) {
      const line = el("div", "run-line");
      line.appendChild(el("span", "run-os", osName));
      line.appendChild(el("code", "", cmd));
      line.appendChild(copyButton(cmd));
      box.appendChild(line);
    }
    return box;
  }

  function renderScript(state) {
    browser.textContent = "";
    const script = scriptOf(state.field, state.area, state.script);
    if (!script) return renderScripts(state);
    const panel = el("div", "script-view");
    const head = el("div", "script-view-head");
    head.appendChild(copyButton(script));
    head.appendChild(el("h2", "", script.name));
    panel.appendChild(head);
    panel.appendChild(runBox(script));
    const pre = el("pre", "script-text");
    pre.tabIndex = 0;
    pre.textContent = script.text;
    panel.appendChild(pre);
    browser.appendChild(panel);
  }

  /* --- search ------------------------------------------------------------ */

  let searchTimer = null;
  function search(query) {
    const q = query.trim().toLowerCase();
    if (!q) {
      render();
      return;
    }
    const hits = [];
    for (const s of flat) {
      if (s.name.toLowerCase().includes(q) || (s.title && s.title.toLowerCase().includes(q)) || s.text.toLowerCase().includes(q)) {
        hits.push(s);
      }
    }
    browser.textContent = "";
    if (!hits.length) {
      browser.appendChild(el("p", "note", t("scripts-no-results", "No scripts match your search")));
      return;
    }
    const list = el("ul", "script-list");
    for (const s of hits.slice(0, 200)) {
      const row = el("li", "script-row");
      row.appendChild(copyButton(s));
      const link = el("a", "script-link", s.name);
      link.href = `#/${s.field}/${s.area}/${s.name}`;
      row.appendChild(link);
      row.appendChild(el("span", "script-path", `${s.field}/${s.area}`));
      list.appendChild(row);
    }
    browser.appendChild(list);
  }

  /* --- wiring ------------------------------------------------------------ */

  function load() {
    flat = [];
    for (const field of tree.fields) {
      for (const area of field.areas) {
        for (const s of area.scripts) flat.push({ field: field.name, area: area.name, ...s });
      }
    }
    render();
  }

  fetch("scripts-data.json")
    .then((r) => (r.ok ? r.json() : Promise.reject(new Error(String(r.status)))))
    .then((data) => {
      tree = data;
      load();
    })
    .catch((err) => {
      console.error("scripts data unavailable:", err);
      if (browser) {
        browser.textContent = "";
        browser.appendChild(el("p", "note", "scripts-data.json is missing; run npm run build:scripts."));
      }
    });

  window.addEventListener("hashchange", () => {
    if (searchInput && searchInput.value) {
      searchInput.value = "";
    }
    render();
  });

  if (searchInput) {
    searchInput.addEventListener("input", () => {
      clearTimeout(searchTimer);
      searchTimer = setTimeout(() => search(searchInput.value), 150);
    });
    searchInput.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        searchInput.value = "";
        search("");
        searchInput.blur();
      }
    });
  }

  // Re-render in the new language when the page's language select changes
  // (app.js applies catalogs on the same event; it registers first, so the
  // document lang is already current here).
  const langSelect = document.getElementById("lang-select");
  if (langSelect) {
    langSelect.addEventListener("change", () => {
      document.querySelectorAll(".copy-btn").forEach((btn) => {
        btn.title = t("copy", "Copy");
        btn.setAttribute("aria-label", t("copy", "Copy"));
      });
      if (searchInput && searchInput.value) search(searchInput.value);
      else render();
    });
  }
})();
