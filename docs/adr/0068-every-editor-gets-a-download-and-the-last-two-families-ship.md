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

## Amendment (2026-09-12): Emacs joins as a config family

The page names ten editor families now. Emacs ships as
`epher-emacs.zip`, a ready-made `epher.el` (a derived major mode
with the conservative font-lock view of the grammar, filetype
detection, and the LSP wiring) speaking to the shared server through
eglot, which Emacs 29 ships built in, with an lsp-mode registration
beside it. Same rules as the other config families: source zip under
a stable name, download link on the page, no package.el publication.
The stable-name list above grows by one: `epher-emacs.zip`.

## Amendment (2026-09-13): Eclipse joins as a dropins family

Eleven families now. Eclipse ships as `epher-eclipse.jar`, a
dropins bundle, the classic analog of VS Code's "Install from
VSIX": the user copies the jar into `<eclipse>/dropins/` and
restarts. Inside is the smallest thing LSP4E accepts: one Java
connection-provider class that launches `epher-lsp` from PATH (the
emacs/vim/neovim bring-your-own-binary contract; the plugin never
downloads), and everything else declared in plugin.xml: the `.epher`
content type, the LSP4E server definition and contentTypeMapping,
and the shared TextMate grammar registered with TM4E with a
scopeNameContentTypeBinding for baseline highlighting. CI compiles
the class against the LSP4E bundle from the project's own p2
release repo (it is not on Maven Central), packages the jar, and
smoke-checks it; no p2 update site, no marketplace (the ADR-0068
gate stands). The stable-name list above grows by one:
`epher-eclipse.jar`.

## Amendment (2026-09-18): the VS Code Marketplace ships, publication unparked for that store

The user explicitly asked (2026-09-18), exercising the item 5 gate
for the VS Code family. `upyesp.epher` v0.5.42 is published on the
Visual Studio Marketplace; the release-train `epher-vscode.vsix`
stays attached to releases as the sideload route (forks, offline,
reproducibility), unchanged.

- **Publisher**: `upyesp`, created once in the publisher management
  portal. Publisher creation is the one step no API accepts (the
  gallery rejects programmatic creation with a reCAPTCHA check and
  vsce has removed `create-publisher`); everything after it is
  automatable.
- **Identity**: the Entra ID secure automated publishing route, not
  a personal access token. Publishing runs
  `vsce publish --azure-credential` (vsce 3.9.2), whose credential
  chain takes an Entra token for the Azure DevOps resource from the
  Azure CLI's device-code login on the provisioning machine. The
  publisher's owner is the publishing identity; no secret is stored
  anywhere.
- **Listing**: the extension's own README, icon (`images/icon.png`,
  256 px), `galleryBanner` dark theme, version and CI badges, and
  the real captures under `clients/vscode/images/` (editor,
  hover, completion, and the `demo.gif` animation). Nothing was
  mocked: the captures come from a real VS Code session driving the
  packaged extension.
- **CI publishes the same way**: `vscode-publish.yml` (reusable, and
  dispatchable on its own for tests) checks out, builds the same
  wasm + vsix recipe, logs Azure in with the repository's GitHub OIDC
  token (`az login --service-principal --federated-token`), and runs
  the same `--azure-credential` flag; the release train calls it at
  the tagged version. The trust is a workload identity federation
  credential on the app whose subject names the repo's `stores`
  environment in the immutable ID-bearing form
  (`repo:upyesp@25488264/epher@1332818283:environment:stores`; the
  repo postdates the July 2026 subject-format cutover). CI holds two
  identifiers (`AZURE_TENANT_ID`, `AZURE_CLIENT_ID`) and no secret
  material; until they exist the job skips with a notice, the repo's
  standard pattern.
- **How the identity became a publisher-authorized one**: not through
  the publisher Members picker, which could not resolve the service
  principal in any form (the publisher predates the tenant; its
  member search realm has no directory identities, and the principal
  has no email address to resolve). What worked: connecting the Azure
  DevOps organization to the tenant (Organization settings →
  Microsoft Entra) and adding the service principal as an
  organization user (Stakeholder). With that, the gallery accepts the
  principal's Entra token as an authorized publisher identity, proven
  by a publish attempt of the already-published version returning a
  version conflict rather than an authorization failure.
- **The website's IDE page links the marketplace**: the VS Code
  section leads with the marketplace install, the
  `epher-vscode.vsix` download stays beside it as the sideload
  route, and all eight locale catalogs carry the change.
- **Open VSX is untouched**: a separate registry with its own
  account, still parked.
