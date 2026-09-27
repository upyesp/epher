# Changelog - epher for Visual Studio Code and VSCodium

All notable changes to the extension are listed here. epher the
language changes independently; the extension bundles the shared
grammar, snippets, and the WebAssembly language server of whatever
epher release built it.

## 0.5.55 - 2026-09-27

The same extension now installs in VSCodium straight from Open VSX.
Listing refresh: new screenshots and animated capture shot in
VSCodium, changelog tab, repository links for the marketplace footer.

## 0.5.40 - 2026-09-16

Initial release.

- Inline answers: every statement's result renders next to its line
  as an inlay hint.
- Units that convert: quantities with unit suffixes, `in`/`->`
  conversions, and unit-aware semantic coloring.
- Live diagnostics: syntax and evaluation errors point at the exact
  token.
- Hover signatures for catalog functions and your own definitions.
- Completion: the full catalog, user definitions, keywords, and
  statement snippets.
- Script running through a run-only debug adapter: **Run script**
  CodeLens, F5, or the command palette, with a results pane that
  links back to the source lines and renders every graph inline.
- Works on the web: vscode.dev and github.dev, language server and
  all, via a WebAssembly build of `epher-lsp`.
