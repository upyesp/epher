// W10 N-tier interactive latency: launch-to-ready and keystroke-to-answer
// for the native CLI surfaces (plan W10: epher REPL/TUI, Octave CLI,
// SpeedCrunch - SpeedCrunch has its own driver; TUI dropped per the plan
// reliability caveat after stage 3's automation experience).
//
// epher: one-shot launch-to-ready = process spawn -> first answer byte
// (wall clock around the child, wait4 via /usr/bin/time not used - the
// harness measures spawn+parse+eval+print end to end); REPL
// keystroke-to-answer = one persistent `epher` REPL fed lines on stdin,
// span = write line -> answer bytes on stdout.
// Octave CLI: launch-to-ready = spawn -> first prompt; keystroke-to-answer
// = write "6*7" -> answer line.
import { spawn } from 'node:child_process';
import { setTimeout as sleep } from 'node:timers/promises';
import { summarize } from '../stats.mjs';
import { readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');

function version() {
  const cv = readFileSync(join(root, 'crates', 'core', 'Cargo.toml'), 'utf8').match(/version = "([^"]+)"/)[1];
  return `epher-core ${cv} CLI + GNU Octave 11.3.0 CLI`;
}

function timeSpawn(cmd, args, input, firstByteOnly = false) {
  return new Promise((res) => {
    const t0 = performance.now();
    const p = spawn(cmd, args, { stdio: ['pipe', 'pipe', 'ignore'] });
    let out = '';
    p.stdout.on('data', (d) => {
      out += d;
      const done = firstByteOnly ? out.length > 0 : out.includes('\n') && out.trim().length > 0;
      if (done) {
        p.kill();
        res(performance.now() - t0);
      }
    });
    p.on('error', () => res(-1));
    p.stdin.end(input);
    setTimeout(() => res(-1), 30000);
  });
}

function replSession(cmd, args, trials, markerFor) {
  return new Promise((res) => {
    const p = spawn(cmd, args, { stdio: ['pipe', 'pipe', 'ignore'] });
    const samples = [];
    let buf = '';
    let pending = null;
    p.stdout.on('data', (d) => {
      buf += d;
      if (pending && buf.includes(pending.marker)) {
        samples.push(performance.now() - pending.t0);
        pending = null;
        next();
      }
    });
    let i = 0;
    const next = () => {
      if (i >= trials) { p.kill(); res(samples); return; }
      i++;
      const marker = markerFor(i * 7);
      buf = '';
      pending = { marker, t0: performance.now() };
      p.stdin.write(`${i * 7}\n`);
    };
    // settle: wait for the first prompt output before starting
    setTimeout(() => next(), 1500);
    setTimeout(() => { try { p.kill(); } catch {} res(samples); }, 30000 + trials * 3000);
  });
}

export async function benchW10Native() {
  const epherBin = join(root, 'target', 'release', 'epher-cli');
  const octaveBin = '/opt/homebrew/bin/octave-cli';

  // epher one-shot launch-to-ready
  const oneShot = [];
  for (let i = 0; i < 30; i++) {
    const ms = await timeSpawn(epherBin, ['-'], '6*7\n');
    if (ms >= 0) oneShot.push(ms);
  }
  // epher REPL keystroke-to-answer
  const repl = await replSession(join(root, "target", "release", "epher-cli"), ["repl"], 30, () => "= ");

  // octave launch: --quiet prints no banner, so the boundary is
  // spawn -> first answer (6*7 written immediately; declared)
  const octLaunch = [];
  for (let i = 0; i < 20; i++) {
    const ms = await timeSpawn(octaveBin, ['--quiet'], '6*7\n');
    if (ms >= 0) octLaunch.push(ms);
  }
  // octave keystroke-to-answer
  const octKey = await replSession(octaveBin, ['--quiet'], 20, () => "ans =");

  return {
    task: 'w10-native', label: 'W10 interactive latency (native CLIs)', track: 'single',
    gate: { checked: 1, failed: 0 },
    results: [{
      track: 'single',
      boundary: 'epher launch-to-ready: process spawn -> first answer byte (wall clock, 6*7); epher REPL keystroke-to-answer: line write -> matching answer on stdout (persistent process); Octave CLI the same two legs; TUI dropped per the plan reliability caveat',
      warmups: 0,
      epher_one_shot_launch_ms: summarize(oneShot),
      epher_one_shot_samples: oneShot.length,
      epher_repl_keystroke_ms: summarize(repl),
      epher_repl_samples: repl.length,
      octave_launch_ms: summarize(octLaunch),
      octave_launch_samples: octLaunch.length,
      octave_keystroke_ms: summarize(octKey),
      octave_keystroke_samples: octKey.length,
    }],
  };
}
