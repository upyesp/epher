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
// This is the harness's first iteration: it walks the app with VO, types
// one evaluation, and dumps EVERYTHING VoiceOver says so the assertion set
// can be tightened against what WebKit actually exposes. Hard-fails only
// on: VO silent over the app, or the result never announced.

import { execFile, execFileSync, spawn } from 'node:child_process';

const sleep = (ms) => new Promise((res) => setTimeout(res, ms));

const APP = process.env.EPHER_APP || 'epher';
const WALK_STOPS = Number(process.env.EPHER_WALK_STOPS || 14);

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

const captured = [];
const seen = new Set();
let poller = null;
let onEntry = false;
let beforeEnter = 0;

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
      if (t && !seen.has(t)) { seen.add(t); captured.push(t); }
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

  // Type the evaluation and submit it. The live-region announcement (the
  // Answer status, role=status) must arrive AFTER Enter; phrases that
  // mention "button" are history rows, not the live announcement.
  if (!(await appFrontmost())) {
    console.log('FAIL  the desktop app lost frontmost focus before typing; aborting (no keystrokes sent)');
    clearInterval(poller);
    await voiceOver.stop();
    process.exit(1);
  }
  await voiceOver.type('2+3*4', { delay: 60 });
  await sleep(400);
  beforeEnter = captured.length;
  await voiceOver.press('Enter');
  await sleep(2500);

  // Then navigate to the Answer status itself: if WebKit does not fire
  // live announcements, the value may still be readable on arrival. Tab
  // from the entry lands on the Answer status in DOM order.
  await voiceOver.press('Tab');
  await sleep(1200);

  // Escape clears; one more named stop for the dump.
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

console.log('\n── everything VoiceOver said ──');
captured.forEach((p, i) => console.log(`${String(i + 1).padStart(3)}  ${p}`));

const failures = [];
if (captured.length === 0) failures.push('VoiceOver said nothing over the app');

// The live-region check considers only phrases spoken after Enter and
// before the deliberate navigation, ignoring anything that names a button
// (history rows) or echoes typed characters. navigableAnnounced accepts
// the phrase spoken when VO lands on the Answer status after the Tab.
const afterEnter = captured.slice(beforeEnter).filter((p) => !/button/i.test(p));
const livePhrases = afterEnter.slice(0, -1);
const liveAnnounced = livePhrases.some((p) => /=\s*14/.test(p));
const navigableAnnounced = afterEnter.some((p) => /=\s*14/.test(p));
const resultAnnounced = captured.some((p) => /=\s*14/.test(p));
const entrySeen = captured.some((p) => /expression|edit text|text field/i.test(p));
const menuSeen = captured.some((p) => /\bmenu\b/i.test(p));

if (resultAnnounced) console.log('PASS  "= 14" was spoken somewhere (live region or history row)');
else failures.push('no "= 14" spoken at all after the evaluation');
if (liveAnnounced) {
  console.log('PASS  the result was live-announced after Enter (role=status, WCAG 4.1.3)');
} else if (navigableAnnounced) {
  failures.push('WKWebView did NOT announce the live result (WCAG 4.1.3); the value was only readable after navigating to the Answer status. Post-Enter phrases: ' + JSON.stringify(afterEnter));
} else {
  failures.push('no "= 14" in any phrase after Enter (post-Enter phrases: ' + JSON.stringify(afterEnter) + ')');
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
