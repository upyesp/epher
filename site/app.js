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
    if (dict[key]) el.textContent = dict[key];
  });
  document.querySelectorAll("[data-i18n-aria]").forEach((el) => {
    const key = el.getAttribute("data-i18n-aria");
    if (dict[key]) el.setAttribute("aria-label", dict[key]);
  });
  // links whose target depends on the active locale (the user guide)
  document.querySelectorAll("[data-i18n-href]").forEach((el) => {
    el.href = `guide/${lang}/`;
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
  initHero3d();
}

/* --- hero 3D: the saddle `graph3d x ^ 2 - y ^ 2` rotating slowly -------
 * A faithful port of the app's 3D renderer (crates/core/src/graph.rs):
 * yaw around z, pitch around the rotated x axis, and the ORTHOGRAPHIC
 * projection (the ADR-0015 amendment): the screen point is the rotated
 * coordinate pair - the view depth is dropped, there is no perspective
 * divide and no near plane. Mesh lines draw far-to-near with depth-cued
 * opacity; line thickness 0.1 (mesh 1.2x, frame 1.4x). The view box is
 * fixed (the union of the projected bounding box over every yaw) so the
 * mesh rotates without pumping. Reduced motion renders one static frame
 * (WCAG 2.3.3). */
function initHero3d() {
  const svg = document.getElementById("hero3d");
  const meshGroup = document.getElementById("hero3d-mesh");
  const frameGroup = document.getElementById("hero3d-frame");
  if (!svg || !meshGroup || !frameGroup) return;

  const GRID = 24;              // grid x grid mesh
  const DOMAIN = [-5, 5];       // graph3d default domain (ADR-0015)
  const PITCH = 0.6;            // View3D default pose
  const WIDTH = { mesh: 1.2 * 0.1, frame: 1.4 * 0.1 };
  const NS = "http://www.w3.org/2000/svg";

  // The surface: z = x^2 - y^2 over the default square domain.
  const xs = [];
  const zs = [];
  for (let i = 0; i <= GRID; i++) {
    const v = DOMAIN[0] + (DOMAIN[1] - DOMAIN[0]) * (i / GRID);
    xs.push(v);
  }
  for (let r = 0; r <= GRID; r++) {
    const row = [];
    for (let c = 0; c <= GRID; c++) row.push(xs[c] * xs[c] - xs[r] * xs[r]);
    zs.push(row);
  }

  let rig = { sy: 0, cy: 1, sp: Math.sin(PITCH), cp: Math.cos(PITCH) };
  const rigFor = (yaw) => {
    rig = {
      sy: Math.sin(yaw),
      cy: Math.cos(yaw),
      sp: Math.sin(PITCH),
      cp: Math.cos(PITCH),
    };
  };

  // to_camera: yaw around z, then pitch around the rotated x axis.
  const toCamera = (x, y, z) => {
    const xr = x * rig.cy - y * rig.sy;
    const yr = x * rig.sy + y * rig.cy;
    return [xr, yr * rig.cp - z * rig.sp, yr * rig.sp + z * rig.cp];
  };
  // to_screen: the orthographic mapping (ADR-0015 amendment) - drop the
  // view depth, keep the rotated coordinates. Affine: no point explodes
  // and relative positions stay exact at every zoom.
  const toScreen = (xr, yp) => [xr, -yp];
  // project_clipped: a world segment to screen. The projection is
  // orthographic, so every finite segment projects - there is no camera
  // plane to clip against (the core drops the clip entirely).
  const projectSegment = (x1, y1, z1, x2, y2, z2) => {
    if (!isFinite(z1) || !isFinite(z2)) return null;
    const [xr1, yp1, zp1] = toCamera(x1, y1, z1);
    const [xr2, yp2, zp2] = toCamera(x2, y2, z2);
    const [sx1, sy1] = toScreen(xr1, yp1);
    const [sx2, sy2] = toScreen(xr2, yp2);
    if (
      !isFinite(sx1) || !isFinite(sy1) || !isFinite(sx2) || !isFinite(sy2)
    ) return null;
    return [sx1, sy1, zp1, sx2, sy2, zp2];
  };

  // One frame's geometry: mesh polylines (rows then columns, far-to-near
  // painter's order) plus the orientation frame of surface_frame.
  const buildGeometry = (yaw) => {
    rigFor(yaw);
    const lines = [];
    const pushLine = (pts) => {
      if (pts.length) {
        lines.push({
          points: pts.map(([x, y]) => [x, y]),
          depth: pts.reduce((s, p) => s + p[2], 0) / pts.length,
        });
      }
    };
    // line_runs: a grid line split into visible runs.
    const runsOf = (cx, cy, cz) => {
      const runs = [];
      let run = [];
      let started = false;
      for (let i = 0; i < cx.length; i++) {
        if (!isFinite(cz[i])) {
          runs.push(run);
          run = [];
          started = false;
          continue;
        }
        if (!started) {
          const [xr, yp, zp] = toCamera(cx[i], cy[i], cz[i]);
          const [sx, sy] = toScreen(xr, yp);
          if (isFinite(sx) && isFinite(sy)) {
            run.push([sx, sy, zp]);
            started = true;
          }
          continue;
        }
        const seg = projectSegment(
          cx[i - 1], cy[i - 1], cz[i - 1], cx[i], cy[i], cz[i]
        );
        if (seg) {
          run[run.length - 1] = [seg[0], seg[1], seg[2]];
          run.push([seg[3], seg[4], seg[5]]);
        } else {
          runs.push(run);
          run = [];
          started = false;
        }
      }
      runs.push(run);
      return runs;
    };
    for (let r = 0; r <= GRID; r++) {
      const row = zs[r];
      for (const run of runsOf(xs, new Array(GRID + 1).fill(xs[r]), row)) pushLine(run);
    }
    for (let c = 0; c <= GRID; c++) {
      const col = zs.map((row) => row[c]);
      for (const run of runsOf(new Array(GRID + 1).fill(xs[c]), xs, col)) pushLine(run);
    }
    // Painter's order: far-to-near.
    lines.sort((a, b) => b.depth - a.depth);
    let zmin = Infinity;
    let zmax = -Infinity;
    for (const line of lines) {
      zmin = Math.min(zmin, line.depth);
      zmax = Math.max(zmax, line.depth);
    }
    // surface_frame: ground square, axes through the origin, vertical
    // extent, as projected segments.
    const frame = [];
    const edge = (x1, y1, z1, x2, y2, z2) => {
      const seg = projectSegment(x1, y1, z1, x2, y2, z2);
      if (seg) frame.push(seg);
    };
    const [a, b] = DOMAIN;
    edge(a, a, 0, b, a, 0);
    edge(b, a, 0, b, b, 0);
    edge(b, b, 0, a, b, 0);
    edge(a, b, 0, a, a, 0);
    edge(a, 0, 0, b, 0, 0);
    edge(0, a, 0, 0, b, 0);
    edge(0, 0, a, 0, 0, b);
    return { mesh: lines, frame, zmin, zmax };
  };

  // A constant-size, per-frame-centered view box: the size is the
  // largest projected extent over the whole rotation (plus the app's 6%
  // pad), and each frame centers that box on its own content. The mesh
  // therefore stays centered and constant-scale while it rotates; no
  // pumping, no drift.
  const touch = (px, py, acc) => {
    acc.x0 = Math.min(acc.x0, px);
    acc.x1 = Math.max(acc.x1, px);
    acc.y0 = Math.min(acc.y0, py);
    acc.y1 = Math.max(acc.y1, py);
  };
  const bboxOf = (g) => {
    const acc = { x0: Infinity, x1: -Infinity, y0: Infinity, y1: -Infinity };
    g.mesh.forEach((l) => l.points.forEach(([x, y]) => touch(x, y, acc)));
    g.frame.forEach((s) => {
      touch(s[0], s[1], acc);
      touch(s[3], s[4], acc);
    });
    return acc;
  };
  let maxSpan = 0;
  for (let deg = 0; deg < 360; deg++) {
    const acc = bboxOf(buildGeometry((deg * Math.PI) / 180));
    maxSpan = Math.max(maxSpan, acc.x1 - acc.x0, acc.y1 - acc.y0);
  }
  const W = maxSpan + maxSpan * 0.12;
  const H = maxSpan + maxSpan * 0.12;

  // Build the elements once; each frame only updates attributes and
  // re-orders (the same patching strategy as the app's live renderer).
  const meshEls = [];
  for (let i = 0; i < 2 * (GRID + 1); i++) {
    const el = document.createElementNS(NS, "polyline");
    el.setAttribute("fill", "none");
    el.setAttribute("stroke", "currentColor");
    el.setAttribute("stroke-width", WIDTH.mesh.toFixed(2));
    meshEls.push(el);
  }
  const frameEls = [];
  for (let i = 0; i < 7; i++) {
    const el = document.createElementNS(NS, "line");
    el.setAttribute("stroke", "currentColor");
    el.setAttribute("stroke-width", WIDTH.frame.toFixed(2));
    el.setAttribute("stroke-opacity", "0.9");
    frameEls.push(el);
  }

  const render = (yaw) => {
    const g = buildGeometry(yaw);
    // Center the constant-size box on this frame's content.
    const acc = bboxOf(g);
    const cx = (acc.x0 + acc.x1) / 2;
    const cy = (acc.y0 + acc.y1) / 2;
    svg.setAttribute(
      "viewBox",
      `${(cx - W / 2).toFixed(3)} ${(cy - H / 2).toFixed(3)} ${W.toFixed(3)} ${H.toFixed(3)}`
    );
    const { mesh, frame, zmin, zmax } = g;
    const span = zmax - zmin;
    mesh.forEach((line, i) => {
      const el = meshEls[i];
      const t = span < 1e-9 ? 1 : (line.depth - zmin) / span;
      el.setAttribute(
        "points",
        line.points.map(([x, y]) => `${x.toFixed(3)},${y.toFixed(3)}`).join(" ")
      );
      // Depth cue without color: opacity 0.35 far -> 0.95 near.
      el.setAttribute("stroke-opacity", (0.35 + 0.6 * t).toFixed(3));
      meshGroup.appendChild(el);
    });
    frame.forEach((seg, i) => {
      const el = frameEls[i];
      el.setAttribute("x1", seg[0].toFixed(3));
      el.setAttribute("y1", seg[1].toFixed(3));
      el.setAttribute("x2", seg[3].toFixed(3));
      el.setAttribute("y2", seg[4].toFixed(3));
      frameGroup.appendChild(el);
    });
  };

  const REDUCE = window.matchMedia("(prefers-reduced-motion: reduce)").matches;
  if (REDUCE) {
    render(0.8); // the app's default pose, static
    return;
  }
  const REV_MS = 4000; // one slow revolution
  const tick = (now) => {
    render(0.8 + (now / REV_MS) * 2 * Math.PI);
    requestAnimationFrame(tick);
  };
  requestAnimationFrame(tick);
}

document.addEventListener("DOMContentLoaded", init);
