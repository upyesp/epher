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

import { execFile } from 'node:child_process';

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
    await osaOut('set volume output volume 25');
    console.log(`  info: system volume ${cur} -> 25 for the run`);
    return true;
  }
  return false;
}

const captured = [];
const seen = new Set();
let poller = null;

try {
  const { voiceOver } = await import('@guidepup/guidepup');

  console.log('starting VoiceOver (harness: native-desktop-vo) ...');
  await voiceOver.start();
  await clampVolume();
  await sleep(800);

  await new Promise((res) =>
    execFile('/usr/bin/osascript', ['-e', `tell application "${APP}" to activate`], () => res()));
  await sleep(2500);

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

  // Type an evaluation and submit it, wherever focus sits after the walk
  // (if focus reached the expression field this announces the result via
  // the live region; the dump shows where focus actually landed).
  await voiceOver.type('2+3*4', { delay: 60 });
  await sleep(400);
  await voiceOver.press('Enter');
  await sleep(2000);

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

const resultAnnounced = captured.some((p) => /=\s*14/.test(p));
const entrySeen = captured.some((p) => /expression|edit text|text field/i.test(p));
const menuSeen = captured.some((p) => /\bmenu\b/i.test(p));

if (resultAnnounced) console.log('PASS  the result was announced (live region carries "= 14")');
else failures.push('no "= 14" announcement after typing the evaluation');
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
