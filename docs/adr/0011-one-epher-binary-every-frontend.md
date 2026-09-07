# 0011 — One `epher` Binary Hosts Every Frontend

Date: 2026-08-15 · Status: accepted · Supersedes: nothing (extends ADR-0001's
frontend layout)

## Context

Releases shipped three downloads per platform (CLI, TUI, desktop), each with
its own binary. Users had to choose before trying anything, and the desktop
install put an `epher-desktop` on disk that could not do what the terminal
binary could. The ask: **one download, one install, one executable** —
`epher` on every platform — that is simultaneously the one-shot CLI, the
REPL, the TUI, and the desktop GUI. Example flow for a Windows user: run one
installer, then in PowerShell `epher "2 + 2"`, `epher repl`, `epher tui`,
and a bare `epher` for the GUI.

The frontends already share everything through the workspace crates
(ADR-0001); only the entry points were separate binaries.

## Decision

**The Tauri binary is the unified `epher` executable.** It dispatches on
arguments to the frontends' own library entry points — no frontend logic
lives in the dispatcher:

| Invocation | Mode | Implementation |
| --- | --- | --- |
| `epher "2 + 2"` | one-shot evaluation | `epher_cli::run_one_shot` |
| `epher -` | piped script (stdin, line by line) | `epher_cli::run_stdin` |
| `epher repl` | interactive REPL | `epher_cli::run_repl` |
| `epher tui` | full-screen terminal UI | `epher_tui::run` |
| `epher gui`, bare `epher` | desktop GUI | the Tauri loop (`app_lib::run`) |

- **Bare `epher` opens the GUI.** Double-click, Start Menu, and Finder
  launches pass no arguments — the no-args case must be the GUI. Terminal
  users get the same: a bare `epher` is the windowed app, `epher repl`/`tui`
  are the terminal modes.
- **The dispatch decision is pure** (`app_lib::dispatch`): `Args → Action`,
  tested without launching anything. Subcommands conflict with the
  expression positional (`args_conflicts_with_subcommands`) so
  `epher "1+1" repl` errors instead of silently merging meanings;
  `allow_hyphen_values` keeps `-` (stdin convention, like `sh -`) and `-5`
  (negative literals) working while `--help`/`--version` stay flags.
- **Windows ships two subsystem builds of the same program** (the console
  flash problem, revisited in 2026): the PE subsystem is a compile-time
  attribute, and Windows creates the console window before `main()` runs,
  so one file cannot be both a flash-free double-click target *and* a
  first-class terminal citizen (wait/redirect/pipe/exit-code semantics —
  CMD and PowerShell don't wait for GUI-subsystem processes, and don't
  inherit redirected handles to them). Therefore:
  - `epher.exe` — console subsystem, installed on `PATH`: the terminal
    product (`epher "2 + 2"`, `epher -`, `epher repl`, `epher tui`).
  - `epher-gui.exe` — GUI subsystem (`windows_subsystem = "windows"`,
    the Tauri template's default), the double-click/Start Menu target: no
    console window ever exists for this process.
  - The console binary hands GUI launches to its GUI sibling
    (`EPHER_GUI_CHILD=1`, detached, null stdio) and exits, so a
    double-click on `epher.exe` or a `epher gui` from a terminal both
    leave no lingering console. The installer carries both binaries — the
    GUI one as `mainBinaryName`, the console one File'd by the NSIS
    PREINSTALL hook in `nsis-hooks.nsh` (the PATH hook is unchanged). The
    hook Files the console exe from the installer's compile directory,
    where the Windows overlay's `beforeBundleCommand` copies it right
    after cargo build — config-level `bundle.resources` cannot (tauri-build
    copies resources *during* cargo build, before the sibling binary
    exists). A matching PREUNINSTALL hook deletes it so `RMDir $INSTDIR`
    succeeds.
    Cargo-side, `default-run = "epher-gui"` selects which bin tauri
    bundles; `mainBinaryName` names the installed file (macOS/Linux keep
    the single `epher` name, so the same GUI-subsystem source compiles
    into the one shipped binary there — the subsystem attribute is a no-op
    off Windows).
  - macOS/Linux keep the literal single unified binary: their OSes have no
    subsystem split — the `.app` bundle and the `Terminal=false` desktop
    entry decide GUI vs terminal by launch context, not by the file.
- **macOS/Linux run the GUI in-process in the foreground**, like any GUI
  binary launched from a terminal (bare `epher` from a terminal opens the
  window and blocks, as documented).
- **The dev binaries stay** for fast iteration (`epher-cli`, `epher-tui`)
  but releases ship only the unified binary (macOS/Linux) or the
  two-build pair (Windows) inside each platform installer (NSIS on
  Windows, dmg on macOS, deb/rpm/AppImage on Linux). The Windows installer
  adds the install directory to the user `PATH` so `epher` works from any
  terminal; macOS offers an in-GUI "install the `epher` command" action
  (symlink into `/usr/local/bin`, osascript fallback for permission).
- **The PWA is unchanged** — a browser cannot host native frontends.

## Consequences

- One install per platform; every mode shares the Native Store (`~/.epher`)
  by construction, not convention (ADR-0002, ADR-0010).
- On Windows the install directory holds two binaries
  (`epher.exe` + `epher-gui.exe`); both are the same program, differing
  only in PE subsystem. `mainBinaryName` points at the GUI build (shortcut
  and uninstaller targets), `PATH` resolves `epher` to the console build.
- The Tauri package (webkit dependencies) becomes a dependency of *every*
  mode for installed users; headless-server users still have the release
  archives of old versions or build from source. Acceptable: epher's
  installed audience is desktop users.
- Frontend crates must keep their entry points callable as libraries
  (`run_*` functions, thin binaries) — the seam that makes this ADR cheap
  is the same one tests use.
- Bare `epher` changing from REPL (v0.2.0 CLI) to GUI is a breaking UX
  change → version 0.3.0.
