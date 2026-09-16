# Epher for VS Code

The epher calculator language in VS Code — desktop, web, and the
forks: syntax highlighting, a diagnostic for every broken statement,
the answer rendered inline next to every statement that produces one,
hover with canonical signatures, and completion for the catalog, your
own names, and the shared snippets.

The Extension is a thin shell (ADR-0066). All understanding lives in
`epher-lsp`, the shared language server — built once from the same
commit as the extension and shipped inside the vsix as a
`wasm32-wasip1-threads` module (`server/epher-lsp.wasm`). Desktop and
web run that identical build; nothing is ever downloaded.

**Desktop** (`main`, `out/extension.js`): the wasm module runs in the
Node extension host through Microsoft's `ms-vscode.wasm-wasi-core`
extension (auto-installed at install time via `extensionDependencies`;
present on the Marketplace and Open VSX, and builtin on the web
hosts), with the `@vscode/wasm-wasi-lsp` pipe bridge. One artifact
covers every desktop platform VS Code ships — Windows, macOS, Linux,
x64 and arm64 — because the engine (V8) and the threading host
(worker_threads) are already there.

**Web** (`browser`, `dist/browser.js`; vscode.dev, github.dev, any web
extension host): the same module, run by the same wasm-wasi-core
extension in its workers. Nothing touches the network.

If the server cannot start, the same TextMate baseline applies:
editing keeps syntax highlighting and snippets; only the live
features (diagnostics, inline answers, hover, completion) wait for a
working host.

## The shared assets

The grammar (`syntaxes/epher.tmLanguage.json`) and the snippets
(`snippets/epher.json`) are copies of `clients/shared/`, the single
source every IDE consumes. Edit there and run `npm run compile` (which
syncs); never edit the copies.

## Building

The wasm must be in place first (CI does this before packaging; the
wasm is never checked in):

    rustup target add wasm32-wasip1-threads
    cargo build -p epher-lsp --target wasm32-wasip1-threads --release
    mkdir -p server
    cp ../../target/wasm32-wasip1-threads/release/epher-lsp.wasm server/

Then:

    npm install
    npm run compile          # syncs shared assets, typechecks, bundles both entries
    npx @vscode/vsce package --no-dependencies

`npm run compile` bundles the desktop entry to `out/extension.js` and
the web entry to `dist/browser.js` with esbuild (single file each —
the web worker allows no module loading) and typechecks both.

## Status

Pilot (stage three of the ADR-0066 plan). Marketplace publication is
deliberately out of scope until the extensions are proven; the website
page is the distribution until then.
