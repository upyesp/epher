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
