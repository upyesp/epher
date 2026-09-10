# Shared editor assets

One copy of each editor-facing asset, consumed by every Extension
(ADR-0066). Edit here; never in a client's private copy.

- `epher.tmLanguage.json`: the TextMate grammar. The baseline spelling
  of the language when no language server is running; the server's
  semantic tokens refine it live.
- `epher-snippets.json`: the snippets. One JSON file in the VS Code
  snippet shape; the language server embeds this same file
  (`crates/lsp/src/analysis.rs` includes it) so editors without a
  snippet engine of their own get them through completion too.
