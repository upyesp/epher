# ADR-0067: Stage five brings the editors beyond VS Code

Date: 2026-09-17

Status: Accepted. Executes the last planned rollout stages of
ADR-0066 (the JetBrains plugin, the Zed extension, the Neovim, Vim,
and Sublime configs, and the disposition of Visual Studio); changes
nothing about the server or the language.

## Context

ADR-0066 fixed the architecture (one `epher-lsp` per platform, thin
universal clients, versions locked to the 0.5.x train, website page
as the distribution until marketplaces) and the order: VS Code, then
JetBrains, then Zed, then Neovim, Vim, and Sublime as configuration
and documentation, then Visual Studio last. Stages one through four
shipped: core spans, trace, and catalog enrichment; the server; the
VS Code pilot with CI packaging; and the localized IDE Extensions
page with the `epher-vscode.vsix` riding the releases.

Facts verified in editor sources and APIs before deciding:

- The JetBrains platform ships a built-in LSP client
  (`com.intellij.platform.lsp.api`, public since 2024.2) and a
  bundled TextMate engine, so the plugin needs no parser and no
  marketplace to be useful: install from disk.
- Zed languages register without a tree-sitter grammar
  (`grammar` is optional in the language config), and Zed renders
  LSP semantic tokens on their own under
  `semantic_tokens: "full"`, so a grammar-less language keeps full
  coloring from the server. Zed compiles extensions itself
  (`wasm32-wasip1`), so a dev extension ships as source, not as a
  binary artifact.
- Sublime Text loads TextMate grammars in the plist XML form, not
  the JSON form the repository shares.
- Neovim's native LSP client (`vim.lsp.config`/`enable`) covers the
  server wiring; its syntax engine does not consume LSP semantic
  tokens, so the static regex syntax remains the highlighting there.
- Visual Studio's LSP surface lives behind the VSSDK, a C# extension
  project of its own, with no configuration-level shortcut.

## Decision

1. **JetBrains ships as a first-class plugin** (`clients/jetbrains`):
   Kotlin on the platform's LSP client API, the shared grammar as a
   TextMate bundle, the same first-run download and cache layout as
   the VS Code pilot. Distributed as `epher-jetbrains.zip`, a stable
   release-asset name built by CI, installed from disk. JetBrains
   Marketplace publication stays out of scope, like every other
   marketplace.
2. **Zed ships as a dev extension** (`clients/zed`): a small Rust
   crate that resolves the release matching its own version through
   the extension API's GitHub helpers, downloads the platform
   binary, and hands the path to Zed. Distributed as source: users
   install via `zed: install dev extension`, and Zed compiles it.
   The language registers grammar-less; coloring rides the server's
   semantic tokens behind the documented per-language setting. A
   tree-sitter-epher grammar may come later for outline and
   tree-sitter-only features; it is not a v1 gate.
3. **Neovim, Vim, and Sublime stay configuration** (ADR-0066's own
   words), delivered as `clients/vim` (shared runtime: ftdetect,
   syntax, ftplugin), `clients/nvim` (native LSP glue), and
   `clients/sublime` (the shared grammar converted to plist XML by a
   committed script, plus the LSP-package client definition). No
   plugins, no package registries; docs point at the same release
   assets.
4. **Visual Studio stays last and stays future work.** The VSSDK
   route is a real extension project on its own; nothing about it
   blocks the other editors, and the website page says so in every
   language. This closes the stage-five scope rather than deferring
   the family silently.
5. **The shared grammar gains clients, not copies of truth.** The
   JSON file remains the single source; JetBrains and Sublime carry
   generated plist forms in their own directories, produced by
   committed scripts, the same discipline the VS Code pilot set.

## Consequences

- Every editor family in ADR-0066's brief except Visual Studio now
  has a working path to the same server, the same features, and the
  same version lock; an epher release carries `epher-vscode.vsix`,
  `epher-jetbrains.zip`, and the four `epher-lsp` binaries under
  stable names.
- The Zed extension's distribution needs Rust on the user's machine
  (Zed compiles dev extensions); registry publication removes that
  step later, mechanically, with no source change.
- The JetBrains plugin cannot be exercised on a headless runner
  beyond packaging and smoke checks; first real interaction testing
  happens in the IDE, and bugs there are fixed on the same train.
- The static syntax in `clients/vim` approximates the parser's unit
  rule by design; the server's semantic tokens remain the exact
  rule, and the vim README says so.
- Version bumps for clients remain part of the release train: the
  VS Code `package.json`, the JetBrains `-PpluginVersion` default
  with its manifest, and the Zed `extension.toml`/`Cargo.toml` pair
  all move in the same bump commit, or the extensions break loudly
  (missing release assets), which is the ADR-0066 trade-off kept.
