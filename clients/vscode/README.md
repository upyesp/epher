# Epher for VS Code

The epher calculator language in VS Code — desktop, web, and the
forks: syntax highlighting, a diagnostic for every broken statement,
the answer rendered inline next to every statement that produces one,
hover with canonical signatures, and completion for the catalog, your
own names, and the shared snippets.

The Extension is a thin shell (ADR-0066). All understanding lives in
`epher-lsp`, the shared language server, delivered two ways:

**Desktop** (`main`, `out/extension.js`): downloaded on first
activation and cached.

- URL: `https://github.com/upyesp/epher/releases/download/v<extension
  version>/epher-lsp-<target>`; an extension update re-fetches a
  matching server.
- Targets in the pilot: `linux-x86_64`, `linux-aarch64`,
  `macos-aarch64`, `windows-x86_64` (`.gz`, Windows `.zip`).
- Cache: the workspace-independent `globalStorage/bin` directory, with
  a version marker so re-downloads only happen on extension updates.

First activation needs the network once. Offline machines keep using
the installers and today's frontends. If the download fails, editing
still works through the TextMate baseline; only the live features are
missing, and a window reload retries.

**Web** (`browser`, `dist/browser.js`; vscode.dev, github.dev, any web
extension host): the server is not downloaded — it ships inside the
vsix as a `wasm32-wasip1-threads` module (`server/epher-lsp.wasm`)
built from the same commit, and runs through Microsoft's
`ms-vscode.wasm-wasi-core` extension (auto-installed via
`extensionDependencies`, present on the Marketplace and Open VSX) with
the `@vscode/wasm-wasi-lsp` pipe bridge. Nothing touches the network.
If the server cannot start, the same TextMate baseline applies.

Both entry points ride the same one server source; the wasm build adds
a target, not an implementation.

## The shared assets

The grammar (`syntaxes/epher.tmLanguage.json`) and the snippets
(`snippets/epher.json`) are copies of `clients/shared/`, the single
source every IDE consumes. Edit there and run `npm run compile` (which
syncs); never edit the copies.

## Building

    npm install
    npm run compile          # syncs shared assets, typechecks, bundles both entries
    npx @vscode/vsce package --no-dependencies

`npm run compile` bundles the desktop entry to `out/extension.js` and
the web entry to `dist/browser.js` with esbuild (single file each —
the web worker allows no module loading) and typechecks both.

The web build needs the server wasm in place first:

    rustup target add wasm32-wasip1-threads
    cargo build -p epher-lsp --target wasm32-wasip1-threads --release
    mkdir -p server
    cp ../../target/wasm32-wasip1-threads/release/epher-lsp.wasm server/

CI (`build-installers.yml`) does exactly this before packaging; the
wasm is never checked in.

The Windows zip layout assumed by the downloader: the binary at the
archive's root as `epher-lsp.exe` (the CI packaging stage fixes the
layout; keep them in step).

## Status

Pilot (stage three of the ADR-0066 plan). Marketplace publication is
deliberately out of scope until the extensions are proven; the website
page is the distribution until then.
