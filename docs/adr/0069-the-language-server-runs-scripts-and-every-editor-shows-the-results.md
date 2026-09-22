# ADR-0069: The language server runs scripts, and every editor shows the results

Date: 2026-09-19

Status: Accepted. Goal set by the user: the epher IDE extensions support
the execution and debugging of epher scripts, with a results pane in the
IDE that can show long results, 2D graphs, 3D graphs, and other plot
types.

## Context

The extensions today are editors-only. The language server (epher-lsp,
the same wasm build inside every editor's extension) evaluates each
statement to produce the inline answers, diagnostics, hover, and
completions, but nothing runs the script as a whole: running an epher
script still means leaving the editor for a terminal.

The pieces the goal needs already exist and are already inside the
language server's build:

- the statement evaluator that produces the inline answers
  (`Document::inlay_hints` in crates/lsp walks and evaluates the
  document with the same engine the calculator ships);
- the SVG renderers (`graph_svg`, `graph3d_svg`, and the data plots in
  crates/core), which the web app and the GUI call for 2D, 3D, and
  statistical plots.

## Decision

1. **Execution lives in the language server.** A custom LSP request
   (`epher/run`) runs the whole document through the same evaluator the
   inline hints use and returns structured results: per-statement
   answers and errors anchored to their lines, plus every graph
   statement's plot as an SVG captured in memory (the existing
   `graph save file.svg` syntax stays untouched). One implementation in
   the language server; every editor that speaks LSP with us gets
   execution without new native dependencies or new binaries.
2. **VS Code gets the results pane first.** A webview panel beside the
   editor: statement results grouped by line, errors linked back to the
   source, SVG graphs rendered inline and zoomable. A CodeLens "Run"
   above the first statement, plus a command and keybinding, triggers
   it.
3. **Other editors get text-first results.** Neovim, Vim, Emacs, and
   Sublime run the same LSP request and show the text results in a
   scratch or floating buffer; graphs are written to temporary SVG
   files and opened with the system viewer. JetBrains and Eclipse take
   the VS Code shape where their UI toolkits allow it; Zed within its
   extension API's limits. No editor's extension ships worse results
   than it ships today.
4. **Debugging is a later phase, honestly named.** Real stepping
   (breakpoints, step over, inspect) needs a Debug Adapter Protocol
   implementation and engine instrumentation for statement boundaries —
   a release of its own, with its own ADR. This ADR covers run plus
   results; nothing here blocks DAP later.
5. **The marketplace listing names the download and the scripts.** The
   extension README (which is the listing body) now also says:
   "Download the epher calculator, and a large selection of ready-made
   scripts from epher.org."

## Consequences

- The language server's run request is the single execution surface;
  editor extensions stay thin (commands and views, no evaluation logic
  duplicated per editor).
- No new engine work is needed for graphs: the renderers are already
  shared and already compiled into the wasm build.
- The results pane is the IDE home for outputs the inline hints cannot
  hold: long tables, multi-line dumps, and every plot type.
- Phasing: milestone 1 — the LSP run request plus the VS Code results
  pane; milestone 2 — the text-first editors; milestone 3 — DAP
  debugging.

## Amendment (2026-09-21): every start is the same start

The VS Code client grew two run roads: the play CodeLens ran
`epher/run` itself and opened the results pane, while F5 and Ctrl+F5
went through the run-only debug adapter (added after this ADR, no
amendment until now) and printed the transcript in the Debug Console,
opening the pane only when the run produced graphs. Two roads meant
two experiences for one command, and a text-only run on F5 left the
pane closed; VSCodium installs carrying the older build also showed
the adapter's earliest shape, which echoed statement source instead
of answers.

The amendment collapses the roads into one: every start (the CodeLens,
Ctrl+Enter, the command palette, F5, Ctrl+F5) launches the same
run-only debug session. The adapter sends `epher/run`, streams the
transcript to the Debug Console, and always opens the results pane
beside the editor, graphs or no graphs. The pane registers before the
language server starts, so a run never races its creation, and the
adapter's old "run the play command instead" fallback message is gone.
Other editors are unchanged: this is the VS Code family's client-side
plumbing, not the server contract.

## Amendment (2026-09-21): milestone two — the editors beyond VS Code run

Milestone two is shipped, editor by editor, within each platform's
real limits:

- **JetBrains** takes the VS Code shape. "Run Epher Script" lives in
  the Tools menu and the editor's context menu; a results tool window
  beside the editor renders the same report the VS Code pane does —
  answers and errors anchored to lines, every 2D/3D graph inline —
  through the platform's JCEF browser, with a text-only Swing
  fallback where JCEF is unavailable. The platform's built-in LSP
  client exposes no raw server handle, so a run speaks to its own
  short-lived `epher-lsp` session (initialize, didOpen, `epher/run`,
  shutdown) beside the inline-hints session; the binary is the one
  the plugin already downloads and caches.
- **Visual Studio** gets a Tools-menu command backed by a minimal
  VSPackage and .vsct. The report opens in the IDE's internal browser
  as a tab in the editor well — dockable like any document — carrying
  the same HTML; errors echo to an "epher" Output pane. The same
  one-shot server session drives the run; StreamJsonRpc frames it.
  A true dockable tool window is the named follow-up.
- **Neovim** is text-first, exactly as decision 3 prescribed:
  `:EpherRun` sends the same request through the native LSP client
  and opens a results window beside the script (a float with
  `pane = "float"`); graphs are written under the cache directory as
  SVG and opened with the system viewer; `<CR>` on a result row jumps
  to its statement, and on a graph row reopens the file. Rerunning
  from the results window re-runs the script it belongs to.
- **Zed** stays within its extension API's limits, recorded in its
  README: no commands, no panels, no webviews, so there is no pane to
  fill and no action to register. The live inline answers and
  `epher run script.epher` in Zed's terminal are the honest
  substitute until the API grows a surface. The extension's version
  lock (it downloads tag v<its own version>) is now guarded in CI,
  after riding six releases stale. After the 0.5.47 field reports,
  the optional tree-sitter grammar left the extension too: Zed's
  install-time grammar build (a downloaded wasi-sdk toolchain) fails
  in real environments — the Flatpak sandbox among them — with a bare
  ENOENT that blocks the whole install, and the server's semantic
  tokens carry the coloring instead.
- **Vim, Emacs, Sublime, and Eclipse** keep the decision-3 floor for
  now — configuration-level clients whose run story follows in a
  later release; nothing about them regressed.
