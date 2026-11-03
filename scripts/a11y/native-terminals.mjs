#!/usr/bin/env node
// Native terminal surfaces: CLI + TUI accessibility walkthrough (ADR-0009
// amendment 2026-11-03). "Run the accessibility tests" covers every app
// surface; the TUI and CLI ship from one shared source tree on all three
// desktop systems, so what passes here is what Windows and Linux users get.
//
// What this asserts, per docs/accessibility.md's TUI/CLI tables:
//   CLI  plain text in/out, zero ANSI escapes, textual errors with real
//        exit codes (never color-only), stdin scripting mode.
//   TUI  full keyboard operability evidence (hints footer names every
//        key), textual result line, the y = <source> graph caption above
//        the ASCII plot (the screen-reader story), menu bar as text, and
//        no em-dash anywhere in user-visible text (ADR-0081).
//
// The REPL ships only in the unified `epher` binary (ADR-0011); its text
// IO is the same dispatch layer this script verifies, and its GUI-hosted
// sibling is covered by scripts/a11y/native-desktop-vo.mjs.

import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = join(HERE, '..', '..');
const CLI = join(ROOT, 'target', 'debug', 'epher-cli');
const TUI = join(ROOT, 'target', 'debug', 'epher-tui');
const DRIVE = join(HERE, 'tui_pty_drive.py');

const failures = [];
function check(name, ok, detail = '') {
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}${detail ? `; ${detail}` : ''}`);
  if (!ok) failures.push(name);
}

function ensureBuilt() {
  if (existsSync(CLI) && existsSync(TUI)) return;
  console.log('building epher-cli and epher-tui (debug) ...');
  const r = spawnSync('cargo', ['build', '-q', '-p', 'epher-cli', '-p', 'epher-tui'], {
    cwd: ROOT, stdio: 'inherit', env: { ...process.env },
  });
  if (r.status !== 0) {
    console.error('cargo build failed; run `cargo build -p epher-cli -p epher-tui`');
    process.exit(1);
  }
}

function run(bin, args, input) {
  const r = spawnSync(bin, args, { input, encoding: 'utf8', timeout: 30000 });
  return { out: (r.stdout || '') + (r.stderr || ''), status: r.status };
}

ensureBuilt();

// ---- CLI ---------------------------------------------------------------
{
  const r = run(CLI, ['2 + 3 * 4']);
  check('cli one-shot prints the result on stdout', r.out.trim() === '14' && r.status === 0,
    JSON.stringify(r.out.trim()));
  const e = run(CLI, ['1 +']);
  check('cli error is textual with a non-zero exit',
    /error:/.test(e.out) && e.status !== 0, JSON.stringify(e.out.trim()));
  const s = run(CLI, ['-'], '2+2\n');
  check('cli stdin scripting mode', /=\s*4/.test(s.out), JSON.stringify(s.out.trim()));
  const ansi = run(CLI, ['1 +']);
  check('cli emits zero ANSI escapes (theme-safe by construction)',
    !//.test(ansi.out));
  const h = run(CLI, ['--help']);
  check('cli --help lists usage text', /USAGE|usage/i.test(h.out) && h.status === 0);
}

// ---- TUI (via the pty driver) ------------------------------------------
{
  const r = spawnSync('/usr/bin/python3', [DRIVE, TUI], { encoding: 'utf8', timeout: 60000 });
  const text = (r.stdout || '').replace(/\x00/g, '');
  const squashed = text.replace(/[ \t]+/g, '');
  check('tui renders the keyboard hints footer',
    /Enterevaluate/.test(squashed) && /qquit/.test(squashed));
  check('tui renders the result textually', /2\+3\*4\s*=\s*14/.test(text) || /2\+3\*4=14/.test(squashed));
  check('tui carries the y = <source> graph caption (screen-reader story)',
    /y=sin\(x\)/.test(squashed));
  check('tui menu bar is text (File Edit Graph Help Settings)',
    /FileEditGraphHelpSettings/.test(squashed));
  check('tui keypad banks are text tabs',
    /123trigfnnumdata/.test(squashed) || /trigfnnumdata/.test(squashed));
  check('tui shows no em-dash anywhere (ADR-0081)', !text.includes('—'));
  check('tui examples hint names the route by keyboard',
    /Menu\(F9\)/.test(squashed) || /Menu\(F9\)/i.test(text.replace(/[ \t]+/g, ' ')));
  check('tui quit via q exits cleanly', r.status === 0, `status=${r.status}`);
}

console.log('');
if (failures.length) {
  console.log(`NATIVE TERMINALS SUMMARY: ${failures.length} failure(s): ${failures.join('; ')}`);
  process.exit(1);
}
console.log('NATIVE TERMINALS SUMMARY: all checks passed');
