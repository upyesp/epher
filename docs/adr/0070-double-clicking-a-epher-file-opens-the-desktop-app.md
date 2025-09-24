# Double-clicking a `.epher` file opens the desktop app and stages it

Date: 2026-09-17. Renumbered from 0069 to 0070 on 2026-09-21: that
number was double-allocated, and the run-and-results decision (the
language server runs scripts) keeps the lower number because the code
cites it.

## Context

The desktop app saves scripts as `.epher` files (ADR-0024), but the
installers never told the operating systems what a `.epher` file is.
Double-clicking one did nothing anywhere: no association was registered,
no mime type existed on Linux, and the file managers had no reason to
connect the extension to epher.

The unified binary (ADR-0011) already routes a script-file argument to
the terminal evaluator, which prints and exits. That is the right
behavior in a terminal, but it is invisible for a double-click: the
Windows association would flash a console (or, pointed at the
GUI-subsystem build, print into nothing at all), and the Linux desktop
entry would launch the evaluator with no window at all.

## Decision

The installers register the association, and every launch path that
cannot be a terminal routes the file to the desktop app, which stages
the file's contents in the expression entry box (the same thing a
history pick loads). Nothing auto-evaluates.

- All bundles: `bundle.fileAssociations` declares `.epher`
  (`application/x-epher`, role Editor). Windows NSIS registers the
  association against `epher-gui.exe` (`mainBinaryName` on Windows,
  ADR-0011). macOS gets `CFBundleDocumentTypes` from the same config,
  and file-open events arrive as Tauri `RunEvent::Opened`.
- Linux: the packages ship a shared-mime-info definition
  (`/usr/share/mime/packages/epher.xml`) and a custom desktop entry
  template whose Exec carries a field code (`epher gui %f`), because the
  bundler's default Exec passes no file argument at all. The post-install
  script refreshes the mime and desktop databases.
- Dispatch: `epher gui [SCRIPT]` now carries the script to stage. The
  GUI-subsystem build (`epher-gui.exe`) treats a bare script argument
  the same way: it has no console, so terminal output is not an option.
  The console binary's terminal behavior is unchanged (`epher
  script.epher` still evaluates and prints).

## Consequences

- A double-clicked `.epher` file opens the desktop app on all three OSes
  with the script staged in the entry box, whether or not the app is
  already running (each launch opens its own window; forwarding into an
  existing instance would need the single-instance plugin and stays out
  of scope until asked for).
- The webview consumes the file through one command (`take_open_file`)
  plus an event (`open-script`) for macOS opens that arrive while the
  app runs; the browser build implements neither.
- `epher gui` in a terminal now also accepts a script argument, which
  the Windows detach hand-off forwards across the process boundary.
