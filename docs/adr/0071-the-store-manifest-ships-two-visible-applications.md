# The store manifest ships two visible applications

Date: 2026-10-04.

## Context

ADR-0011's store shape made the console build the one visible
application: the store rejects a hidden Application entry
(`AppListEntry="none"` reads as a headless app and demands the
HeadlessAppBypass waiver), and an execution alias can only launch its
own Application's Executable. That shape carries two costs. The Start
tile launches a console build, which shows conhost for a split second
before the GUI sibling spawns. And a file type association is bound to
its Application's Executable, so a `.epher` association on that
application would launch the console build, which evaluates the script
into its terminal transcript (ADR-0069) instead of staging it in the
desktop app (ADR-0070), in a console window that closes before anyone
can read it.

## Decision

The manifest declares two visible applications, mirroring what the
NSIS install puts on disk. `EpherGuiApp` (`epher-gui.exe`) carries the
`epher` tile and the `.epher` file type association, so a double-click
opens the desktop app and stages the script, exactly like the NSIS
install (ADR-0070), and the tile opens the window directly, with no
console flash. `EpherApp` (`epher.exe`) carries the `epher CLI` tile
and the `epher.exe` execution alias, which keeps the CLI at `epher` in
every terminal: the alias resolves through
`%LOCALAPPDATA%\Microsoft\WindowsApps`, which is on the user PATH by
default, and an MSIX cannot edit PATH itself. Both applications declare
`uap10:SupportsMultipleInstances`: the window opens one instance per
launch (ADR-0070), the console alias allows parallel terminals.

## Consequences

- The Start menu shows two entries, `epher` and `epher CLI`.
- A double-clicked `.epher` file opens the desktop app on a store
  install exactly like on an NSIS install (ADR-0070).
- The HeadlessAppBypass waiver ticket is moot: nothing is hidden, so
  the store rule it exists for is not touched.
- Two visible applications are ordinary store practice (Office and
  python each ship several); the rule the store enforces is against
  hidden entries, not against a count.
- The NSIS install is unchanged: it already registers the association
  against `epher-gui.exe` and manages the user PATH itself.
