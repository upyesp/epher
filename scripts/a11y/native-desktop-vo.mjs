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

  // Walk BACK to the expression entry: typing into whatever focus landed
  // on (a history button, say) would do nothing, and pressing Enter would
  // just activate that control. Shift+Tab until the entry is announced.
  let onEntry = false;
  for (let i = 0; i < 24 && !onEntry; i++) {
    await voiceOver.press('Shift+Tab');
    await sleep(450);
    const p = (await voiceOver.lastSpokenPhrase().catch(() => '')) || '';
    if (/expression/i.test(p)) onEntry = true;
  }
  console.log(onEntry
    ? '  info: focus is on the expression entry'
    : '  warn: never heard the entry on the way back; typing anyway');

  // Type the evaluation and submit it. The live-region announcement (the
  // Answer status, role=status) must arrive AFTER Enter; phrases that
  // mention "button" are history rows, not the live announcement.
  const beforeType = captured.length;
  await voiceOver.type('2+3*4', { delay: 60 });
  await sleep(400);
  const beforeEnter = captured.length;
  await voiceOver.press('Enter');
  await sleep(2500);

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

// The live-region check considers only phrases spoken after Enter, and
// ignores anything that names a button (history rows) or echoes typed
// characters.
const afterEnter = captured.slice(beforeEnter).filter((p) => !/button/i.test(p));
const liveAnnounced = afterEnter.some((p) => /=\s*14/.test(p));
const resultAnnounced = captured.some((p) => /=\s*14/.test(p));
const entrySeen = captured.some((p) => /expression|edit text|text field/i.test(p));
const menuSeen = captured.some((p) => /\bmenu\b/i.test(p));

if (resultAnnounced) console.log('PASS  "= 14" was spoken somewhere (live region or history row)');
else failures.push('no "= 14" spoken at all after the evaluation');
if (liveAnnounced) console.log('PASS  the result was live-announced after Enter (role=status)');
else failures.push('no live-region announcement after Enter (phrases after Enter: ' + JSON.stringify(afterEnter) + ')');
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
