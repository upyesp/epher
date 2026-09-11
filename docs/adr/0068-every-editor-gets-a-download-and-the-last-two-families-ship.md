# ADR-0068: Every editor gets a download, and the last two families ship

Date: 2026-09-17

Status: Accepted. Extends ADR-0066 and ADR-0067: per-editor download
links on the website, Cursor and VSCodium as first-class sections,
the Visual Studio extension as a shipped artifact, the tree-sitter
grammar as a standalone repository, and marketplace publication
explicitly parked.

## Context

After stage five, every editor family except Visual Studio had a
working path to the shared server, but the website page linked only
some artifacts, the Zed extension required cloning the repository,
Visual Studio was a paragraph of future tense, and the two VS Code
forks people actually use (Cursor, VSCodium) were not mentioned at
all. Separately, the Zed extension had no tree-sitter grammar, so
baseline highlighting and outline were absent until the user flipped
a semantic-tokens setting.

## Decision

1. **Every IDE section on the website carries a direct download
   link.** Stable names, one per client, resolved through the
   `releases/latest/download/<name>` pattern: `epher-vscode.vsix`,
   `epher-jetbrains.zip`, `epher-zed.zip`, `epher-nvim.zip`,
   `epher-vim.zip`, `epher-sublime.zip`, `epher-visualstudio.vsix`.
   CI builds and attaches them all; the config families ship as
   source zips because that is what their installation consumes.
2. **Cursor and VSCodium are first-class sections.** Both are VS
   Code forks with the same Install-from-VSIX flow, so they install
   the identical `epher-vscode.vsix` with identical behavior; the
   page says so in every language rather than leaving them to be
   inferred.
3. **The Visual Studio extension ships** (`clients/visualstudio`): a
   C# VSIX for Visual Studio 2022 on the platform's language-server
   client surface, the same first-run download of `epher-lsp` as
   every other client, built and smoke-checked by a windows CI leg,
   attached as `epher-visualstudio.vsix`. It is built on Windows in
   CI and cannot be exercised headlessly beyond packaging; first
   interaction testing happens in the IDE, on the same train.
4. **The tree-sitter-epher grammar is a standalone repository**
   (`upyesp/tree-sitter-epher`), because the Zed registry pins a
   grammar by repository plus revision. Its source lives in the epher
   repository under `grammars/tree-sitter-epher` and is pushed
   verbatim; the Zed extension pins the revision in its
   `extension.toml`. The grammar parses the entire shipped
   `epher scripts` corpus (433 scripts) without errors, and its unit
   suffix rule is adjacency-correct through a small external scanner.
   Baseline highlighting and outline in Zed now work out of the box;
   the semantic-tokens setting remains the path to the server's
   exact coloring.
5. **Marketplace publication is parked until the user explicitly
   asks.** No account setup, registry submissions, or publishing
   pipelines for any marketplace ride on general keep-going
   instructions; the releases page remains the distribution.

## Consequences

- The IDE Extensions page answers "how do I get epher into my
  editor" with a link and three steps for all nine editor families
  the page names; nothing requires cloning the repository.
- A release carries nine client artifacts alongside the installers
  and the server binaries, all under stable names; the version-lock
  discipline now spans vsix, plugin, dev extension, and grammar.
- The grammar's truth is split by design: tree-sitter answers
  structure (outline, baseline coloring), the language server
  answers meaning (semantic tokens), and the reference remains the
  normative text for both; the grammar documents its one deliberate
  looseness (chained comparisons parse, the language rejects them).
- Marketplace work has an explicit gate recorded in CONTEXT.md, so
  it cannot begin by inference.
