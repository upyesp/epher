# ADR-0074: The Windows CLI explains a bare run until the headless waiver

Date: 2026-10-11

Status: Accepted (temporary: amends ADR-0011's bare-run rule on the
Windows console build only, until Microsoft grants the HeadlessAppBypass
waiver for the Store listing; removal rides the waiver's follow-up).

## Context

The Store manifest ships two visible applications (ADR-0071) because the
Store reads a hidden app entry (`AppListEntry="none"`) as a headless app
that needs the HeadlessAppBypass waiver, which the listing does not have
yet. The console `epher.exe` execution alias therefore carries the full
CLI in any terminal, and by ADR-0011 a bare `epher` routes to the GUI.

In Store context that bare run reads as silence: the console process
ends without printing anything a user can point at. A terminal user who
runs the bare command deserves the examples the CLI already has
(`-h`, `help`); one command away is not the same as on screen.

Linux and macOS do not have this problem: their icon launches pass the
`gui` subcommand explicitly (the desktop template and the flatpak
entry), so a bare invocation there only ever comes from a terminal, and
it keeps opening the window (ADR-0011).

## Decision

On the Windows console build only, a bare run (no subcommand, no
expression, and not the GUI-subsystem build) prints a short banner: the
sentence naming what happened, the seven examples, and a closing line;
then it waits for one keypress (console line input off, Enter not
required) and exits 0. When stdout is not a console (a Start tile, a
pipe), the run ends quietly instead: there is no reader for the banner
and no console to wait on.

`epher-gui.exe` is untouched: it keeps opening the window on a bare
run. `epher-lsp` is untouched. The other platforms are untouched. The
banner's examples mirror the short help copy; they may drift until the
waiver removes the banner.

## Consequences

- A Windows terminal user who types `epher` gets an explanation and a
  visible end instead of silence.
- The GUI hand-off (ADR-0011's detached spawn of `epher-gui.exe`) no
  longer triggers from a bare run; it still triggers from `epher gui`.
- The banner is temporary by construction: when Microsoft grants the
  waiver, the follow-up is to remove the banner and re-decide what a
  bare `epher` does on the console build (window or examples).
- The wait reads one byte with console line input disabled
  (windows-sys `Win32_System_Console`); a redirected or absent stdin
  cannot hang the run, because it has no console mode to change and its
  reads end immediately. The mode switch is undone after the keypress:
  the console mode belongs to the console, not the process, so a
  restored mode keeps the next program in that terminal (typically
  `epher repl`) reading lines with echo as before.
