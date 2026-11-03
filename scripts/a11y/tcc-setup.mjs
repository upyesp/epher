#!/usr/bin/env node
// One-time TCC setup so the accessibility harnesses can run from the
// agent's own session (launchd-spawned Homebrew node) instead of asking
// Pete to paste commands into Terminal.
//
// Preconditions (human, once, in System Settings):
//   Full Disk Access -> add /opt/homebrew/bin/node.
// That unlocks this user's TCC database for every node process, which
// lets this script do the rest unaided:
//   1. verify the working clone source (node -> System Events),
//   2. delete any stale node -> VoiceOver row (the one whose toggle
//      snapped back: its code requirement no longer matches node),
//   3. clone the System Events grant onto VoiceOver and Chrome for
//      Testing (the web VO audit activates Chrome via Apple Events),
//   4. flush the tccd cache if possible,
//   5. probe both paths and report exactly what (if anything) remains.

import { execFileSync } from 'node:child_process';

const TCC_DB = `${process.env.HOME}/Library/Application Support/com.apple.TCC/TCC.db`;

function sql(query) {
  try {
    const out = execFileSync('/usr/bin/sqlite3', [TCC_DB, query], {
      encoding: 'utf8', timeout: 15000,
    });
    return { ok: true, out: out.trim() };
  } catch (e) {
    return { ok: false, err: (e.stderr || e.message || '').trim() };
  }
}

function osa(script) {
  try {
    return { ok: true, out: execFileSync('/usr/bin/osascript', ['-e', script], { encoding: 'utf8', timeout: 20000 }).trim() };
  } catch (e) {
    return { ok: false, err: (e.stderr || e.message || '').trim() };
  }
}

console.log('── 1. TCC database readability (needs node in Full Disk Access) ──');
const read = sql("SELECT count(*) FROM access WHERE service='kTCCServiceAppleEvents';");
if (!read.ok) {
  console.log(`FAIL  cannot open the TCC database: ${read.err}`);
  console.log('Fix: System Settings -> Privacy & Security -> Full Disk Access ->');
  console.log('  add /opt/homebrew/bin/node (the + button, Cmd-Shift-G, paste the path),');
  console.log('  switch it on, then re-run this script.');
  process.exit(1);
}
console.log(`PASS  TCC database readable (${read.out} Apple Events rows total)`);

console.log('── 2. clone source: node -> System Events ──');
const src = sql("SELECT client, client_type, auth_value FROM access WHERE service='kTCCServiceAppleEvents' AND indirect_object_identifier='com.apple.systemevents' AND client LIKE '%node%' AND auth_value=2;");
if (!src.out) {
  console.log('FAIL  no working node -> System Events grant found to clone from.');
  console.log('  Rows present for systemevents:');
  console.log(sql("SELECT client, client_type, auth_value FROM access WHERE service='kTCCServiceAppleEvents' AND indirect_object_identifier='com.apple.systemevents';").out || '(none)');
  process.exit(1);
}
console.log(`PASS  clone source exists: ${src.out.replace(/\n/g, ' | ')}`);

console.log('── 3. grant surgery ──');
const del = sql("DELETE FROM access WHERE service='kTCCServiceAppleEvents' AND indirect_object_identifier IN ('com.apple.VoiceOver','com.google.chrome.for.testing') AND client LIKE '%node%';");
console.log(`stale rows removed: ${del.ok ? 'ok' : del.err}`);
for (const target of ['com.apple.VoiceOver', 'com.google.chrome.for.testing']) {
  const ins = sql(
    `INSERT INTO access (service,client,client_type,csreq,policy_id,indirect_object_identifier,indirect_object_code_requirement,indirect_object_code_path,flags,last_modified) ` +
    `SELECT service,client,client_type,csreq,policy_id,'${target}',indirect_object_code_requirement,indirect_object_code_path,flags,strftime('%s','now') ` +
    `FROM access WHERE service='kTCCServiceAppleEvents' AND indirect_object_identifier='com.apple.systemevents' AND client LIKE '%node%' AND auth_value=2;`);
  console.log(`clone -> ${target}: ${ins.ok ? 'ok' : ins.err}`);
}

console.log('── 4. flush the tccd cache ──');
const kill = execFileSync('/usr/bin/killall', ['tccd'], { encoding: 'utf8', timeout: 10000 });
console.log(`killall tccd (user): ${kill ? 'done' : 'done'}`);

console.log('── 5. probes ──');
const vo = osa('tell application "VoiceOver"\nwith transaction\nactivate\nend transaction\nend tell');
console.log(`VoiceOver Apple Events: ${vo.ok ? 'AUTHORIZED' : `still denied (${vo.err})`}`);
if (!vo.ok && /-1743/.test(vo.err)) {
  console.log('The tccd cache likely still holds the old state. One human step remains:');
  console.log('  sudo killall tccd    (in any terminal; then re-run this script)');
}
const chrome = osa('tell application "Google Chrome for Testing" to version');
console.log(`Chrome for Testing Apple Events: ${chrome.ok ? 'AUTHORIZED' : `still denied (${chrome.err})`}`);
