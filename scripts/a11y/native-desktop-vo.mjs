#!/usr/bin/env node
// Desktop GUI (macOS Tauri shell) under VoiceOver — the native-surface leg
// of the accessibility walkthrough (ADR-0009 amendment 2026-11-03).
//
// The Tauri shell wraps the same page the PWA audits cover, but inside
// WKWebView (WebKit's accessibility tree), so this is genuinely new
// coverage: Chromium's AX behavior is verified by voiceover-audit.mjs,
// WebKit's only here.
//
// RUN UNDER TERMINAL (the Apple Events grants live on Terminal's ancestry;
// the agent session's node ancestry is not authorized to drive VoiceOver).
//
// The harness starts VoiceOver first (WKWebView builds its web AX tree
// only while an accessibility client is active at webview creation),
// launches the bundle, walks the app, and evaluates a UNIQUE random
// expression each run so stale history rows can never satisfy the
// checks. Everything VoiceOver says is captured on a timestamped
// timeline with phase markers (type/Enter/Tab/Escape) so the dump shows
// exactly when each phrase landed relative to the evaluation.

import { execFile, execFileSync, spawn } from 'node:child_process';

const sleep = (ms) => new Promise((res) => setTimeout(res, ms));

const APP = process.env.EPHER_APP || 'epher';
const WALK_STOPS = Number(process.env.EPHER_WALK_STOPS || 14);

// A unique sum per run: stale history ("a+b = sum" rows persist in
// WKWebView local storage) can then never speak the expected result.
const A = 7 + Math.floor(Math.random() * 90);
const B = 2 + Math.floor(Math.random() * 88);
const EXPR = `${A}+${B}`;
const SUM = A + B;
const EXPECTED = new RegExp(`=\\s*${SUM}\\b`);

function osaOut(script) {
  return new Promise((res) =>
    execFile('/usr/bin/osascript', ['-e', script], (err, stdout) =>
      res(err ? '' : String(stdout).trim())));
}

async function clampVolume() {
  const cur = Number(await osaOut('output volume of (get volume settings)'));
  if (cur > 25) {
    await osaOut(`set volume output volume 25`);
    console.log(`  info: system volume ${cur} -> 25 for the run`);
    return true;
  }
  return false;
}

async function appFrontmost() {
  const v = await osaOut('tell application "System Events" to tell process "epher" to get frontmost');
  return v === 'true';
}

// Launch the desktop app and poll until it is genuinely frontmost. A
// cold Tauri boot easily exceeds a fixed sleep, and every keystroke that
// follows lands in whatever IS frontmost (usually Terminal) — which both
// ruins the walk and executes stray text in a shell.
// EPHER_APP is either an installed .app bundle (activate via Apple Events)
// or a bare dev binary (spawn directly; focus via System Events, which the
// agent session is already authorized to drive).
async function bringAppToFront() {
  if (APP.endsWith('.app')) {
    await osaOut(`tell application "${APP}" to activate`);
  } else {
    let running = '';
    try {
      running = execFileSync('/usr/bin/pgrep', ['-x', 'epher'], { encoding: 'utf8' }).trim();
    } catch { /* no match */ }
    if (!running) {
      const child = spawn(APP, [], { detached: true, stdio: 'ignore' });
      child.unref();
    }
  }
  for (let i = 0; i < 20; i++) {
    await sleep(1000);
    await osaOut('tell application "System Events" to set frontmost of process "epher" to true');
    if (await appFrontmost()) return true;
    if (APP.endsWith('.app')) await osaOut(`tell application "${APP}" to activate`);
  }
  return false;
}

const t0 = Date.now();
const now = () => (Date.now() - t0) / 1000;
// Timeline entries: { t, kind: 'phrase' | 'phase', text }. Phrases are
// deduplicated only against the immediately previous one, so a phrase
// heard again after Enter (navigation, live repeats) is captured anew.
const timeline = [];
let lastSpoken = '';
let poller = null;
let onEntry = false;

const mark = (text) => timeline.push({ t: now(), kind: 'phase', text });
const phrases = () => timeline.filter((e) => e.kind === 'phrase');
const phrasesBetween = (fromName, toName) => {
  const from = timeline.findIndex((e) => e.kind === 'phase' && e.text === fromName);
  const to = toName ? timeline.findIndex((e) => e.kind === 'phase' && e.text === toName) : timeline.length;
  return timeline
    .filter((e, i) => e.kind === 'phrase' && i > from && (to < 0 || i < to))
    .map((e) => e.text);
};
const phrasesAfter = (fromName) => {
  const from = timeline.findIndex((e) => e.kind === 'phase' && e.text === fromName);
  return timeline.filter((e, i) => e.kind === 'phrase' && i > from).map((e) => e.text);
};

try {
  const { voiceOver } = await import('@guidepup/guidepup');

  // VoiceOver FIRST, then launch the app: WKWebView builds its web
  // accessibility tree only while an accessibility client is active at
  // webview creation (wry does nothing to propagate it later), so an app
  // launched before VoiceOver exposes nothing but its window shell.
  console.log('starting VoiceOver (harness: native-desktop-vo) ...');
  await voiceOver.start();
  await clampVolume();
  await sleep(800);

  // A stale instance predates VoiceOver and keeps a shell-only tree.
  try { execFileSync('/usr/bin/killall', ['epher']); } catch { /* not running */ }
  await sleep(1000);

  console.log(`launching ${APP} with VoiceOver already running ...`);
  if (!(await bringAppToFront())) {
    console.log(`FAIL  could not bring ${APP} frontmost; aborting before any keystrokes`);
    clearInterval(poller);
    await voiceOver.stop();
    process.exit(1);
  }
  await sleep(1500);

  poller = setInterval(() => {
    voiceOver.lastSpokenPhrase().then((p) => {
      const t = (p || '').trim();
      if (t && t !== lastSpoken) { lastSpoken = t; timeline.push({ t: now(), kind: 'phrase', text: t }); }
    }).catch(() => {});
  }, 250);

  // Walk the first stops: menu rail, entry, keypad, history in DOM order.
  // Real keystrokes only: VO's cursor moves with the keyboard focus here,
  // same as the Chrome audit.
  for (let i = 0; i < WALK_STOPS; i++) {
    await voiceOver.press('Tab');
    await sleep(450);
  }

  // Walk BACK to the expression entry: typing into whatever focus landed
  // on (a history button, say) would do nothing, and pressing Enter would
  // just activate that control. Shift+Tab until the entry is announced.
  onEntry = false;
  for (let i = 0; i < 24 && !onEntry; i++) {
    await voiceOver.press('Shift+Tab');
    await sleep(450);
    const p = (await voiceOver.lastSpokenPhrase().catch(() => '')) || '';
    if (/expression/i.test(p)) onEntry = true;
  }
  if (!onEntry) {
    console.log('FAIL  the walk never confirmed the expression entry; aborting (no keystrokes sent)');
    clearInterval(poller);
    await voiceOver.stop();
    process.exit(1);
  }
  console.log('  info: focus is on the expression entry');

  // Evaluate a unique expression and submit it. The live-region
  // announcement (the Answer status, role=status) must arrive AFTER
  // Enter; history rows are excluded by the unique sum, not by guesswork.
  if (!(await appFrontmost())) {
    console.log('FAIL  the desktop app lost frontmost focus before typing; aborting (no keystrokes sent)');
    clearInterval(poller);
    await voiceOver.stop();
    process.exit(1);
  }
  mark('type start');
  await voiceOver.type(EXPR, { delay: 60 });
  await sleep(400);
  mark('enter pressed');
  await voiceOver.press('Enter');
  await sleep(2500);

  // Then navigate to the Answer status itself: if WebKit does not fire
  // live announcements, the value may still be readable on arrival. Tab
  // from the entry lands on the Answer status in DOM order.
  mark('tab after enter');
  await voiceOver.press('Tab');
  await sleep(1200);

  // Escape clears; one more named stop for the dump.
  mark('escape pressed');
  await voiceOver.press('Escape');
  await sleep(800);

  clearInterval(poller);
  await voiceOver.stop();
} catch (e) {
  if (poller) clearInterval(poller);
  try { const { voiceOver } = await import('@guidepup/guidepup'); await voiceOver.stop(); } catch { /* not started */ }
  console.log(`FAIL  harness: ${e.message}`);
  process.exit(1);
}

console.log(`\n── everything VoiceOver said (expression ${EXPR}, expecting = ${SUM}) ──`);
timeline.forEach((e) => {
  const t = e.t.toFixed(1).padStart(5);
  if (e.kind === 'phase') console.log(`  ${t}s  ⋮ ${e.text}`);
  else console.log(`  ${t}s  ${e.text}`);
});

const failures = [];
const allPhrases = phrases().map((e) => e.text);
if (allPhrases.length === 0) failures.push('VoiceOver said nothing over the app');

// Live window: phrases between "enter pressed" and the navigation Tab.
// Navigable window: phrases from the Tab onward. Button-named phrases are
// history rows or pane controls, never the live announcement itself.
const liveWindow = phrasesBetween('enter pressed', 'tab after enter').filter((p) => !/button/i.test(p));
const navigableWindow = phrasesAfter('tab after enter');
const postEnter = phrasesAfter('enter pressed');
const liveAnnounced = liveWindow.some((p) => EXPECTED.test(p));
const navigableAnnounced = navigableWindow.some((p) => EXPECTED.test(p));
const resultAnnounced = allPhrases.some((p) => EXPECTED.test(p));
const entrySeen = allPhrases.some((p) => /expression|edit text|text field/i.test(p));
const menuSeen = allPhrases.some((p) => /\bmenu\b/i.test(p));

const want = `"= ${SUM}"`;
if (resultAnnounced) console.log(`PASS  ${want} was spoken somewhere (live region or history row)`);
else failures.push(`no ${want} spoken at all after the evaluation`);
if (liveAnnounced) {
  console.log('PASS  the result was live-announced after Enter (role=status, WCAG 4.1.3)');
} else if (navigableAnnounced) {
  failures.push(`WKWebView did NOT live-announce the result (WCAG 4.1.3); ${want} was only readable after navigating to the Answer status. Live window: ` + JSON.stringify(liveWindow));
} else {
  failures.push(`no ${want} in any phrase after Enter (post-Enter phrases: ` + JSON.stringify(postEnter) + ')');
}
if (onEntry) console.log('PASS  the walk reached and focused the expression entry');
else console.log('INFO  the entry was not confirmed on the walk back (see dump)');
if (entrySeen) console.log('PASS  the expression entry is announced by name/type');
else console.log('INFO  no expression-entry phrase captured; WebKit may name it differently');
if (menuSeen) console.log('PASS  a menu control was announced');
else console.log('INFO  no menu phrase captured in the first stops');

console.log('');
if (failures.length) {
  console.log(`DESKTOP VO SUMMARY: ${failures.length} failure(s): ${failures.join('; ')}`);
  process.exit(1);
}
console.log('DESKTOP VO SUMMARY: no failures');
