# VS Code web extension requirements: what vscode.dev and github.dev demand

Date: 2026-09-12 · Status: research note, all claims checked against Microsoft's
own docs (code.visualstudio.com / the microsoft/vscode-docs repo), the
microsoft/vscode-languageserver-node and microsoft/vscode-extension-samples
repos, microsoft/vscode-vsce source, and microsoft/vscode-wasm. Every factual
claim carries its source URL inline. Two observations about the live services
(headers, redirects) are marked as *observed* and are not doc claims.

## What was asked

Whether our VS Code extension (clients/vscode, `main: ./out/extension.js`,
engines ^1.85.0, declarative language contributions plus a download-and-spawn
native Rust LSP server) can run in the browser versions of VS Code —
vscode.dev, github.dev (the editor behind the "." key on GitHub), and the
Codespaces-lite web configurations — and what exactly that would require.

## 1. What web VS Code is

VS Code runs in a browser in three related configurations. VS Code for the Web
at [vscode.dev](https://vscode.dev) is the free, zero-install browser editor
for browsing local files, GitHub repos (`vscode.dev/github/<org>/<repo>`), and
Azure Repos ([vscode-web doc](https://code.visualstudio.com/docs/remote/vscode-web)).
[github.dev](https://docs.github.com/en/codespaces/the-githubdev-web-based-editor)
is GitHub's own web editor; you reach it by pressing `.` (the period key) on a
repository or Pull Request — the VS Code web-extensions guide describes exactly
this ("the `github.dev` user interface reached by pressing `.` (the period key)
when browsing a repository or Pull Request in GitHub",
[web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions);
GitHub's own doc has you press `.` to open a PR in github.dev, same link). The
[1.60 release notes](https://code.visualstudio.com/updates/v1_60) describe the
same "." behavior. Finally, VS Code for the Web **with Codespaces** pairs the
web UI with a remote container; the
[extension host doc](https://code.visualstudio.com/api/advanced-topics/extension-host)
lists four configurations and which extension hosts each has: desktop (local +
web hosts), desktop with remote (local + web + remote), vscode.dev/github.dev
(web host only), and VS Code for the Web with Codespaces (web + remote hosts).

All extension code in these configurations runs in the **web extension host**,
a browser WebWorker runtime ([extension host doc](https://code.visualstudio.com/api/advanced-topics/extension-host)).
VS Code for the Web "runs entirely in your web browser's sandbox and offers a
very limited execution environment"; the terminal and debugger are not
available because you cannot compile, run, and debug a Rust or Go application
inside the browser sandbox ([vscode-web doc](https://code.visualstudio.com/docs/remote/vscode-web)).
GitHub's doc is equally blunt about github.dev: compute — "There is no
associated compute, so you won't be able to build and run your code or use the
integrated terminal"; terminal access — "None"
([docs.github.com](https://docs.github.com/en/codespaces/the-githubdev-web-based-editor)).

Two observed facts about the live services (not doc claims, checked 2026-09-12
with `curl`): `https://vscode.dev` serves
`cross-origin-opener-policy: same-origin` and
`cross-origin-embedder-policy: require-corp` (the page is cross-origin
isolated — this matters for WASM threads, below), and `https://github.dev`
currently answers with a `302` redirect to `https://vscode.dev/github/`.

## 2. What makes an extension a "web extension"

An extension that can run in the web extension host is a web extension. Its
entry file is defined by the **`browser`** property of package.json instead of
`main`; `contributes` works identically in both runtimes. The browser entry
file "runs in the web extension host in a Browser WebWorker environment" and
has full access to the VS Code API but no access to Node.js APIs or module
loading ([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).
The manifest reference documents `browser` as "The entry point to your Web
extension" alongside `main`
([extension-manifest reference](https://code.visualstudio.com/api/references/extension-manifest)).

**Declarative-only extensions are automatically web extensions.** The guide is
explicit: "Extensions with only declarative contributions (only `contributes`,
no `main` or `browser`) can be web extensions. They can be installed and run in
VS Code for the Web without any modifications by the extension author.
Examples of extensions with declarative contributions include themes,
grammars, and snippets" ([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).
The exact enablement rule: "VS Code automatically treats an extension as a web
extension if: The extension manifest (`package.json`) has `browser` entry
point. The extension manifest has no `main` entry point and none of the
following contribution points: `localizations`, `debuggers`, `terminal`,
`typescriptServerPlugins`" (same page). Conversely, "Extensions that have only
a `main` entry point, but no `browser` are not web extensions. They are
ignored by the web extension host and not available for download in the
Extensions view" (same page). Our language id, TextMate grammar, snippets, and
language-configuration fall squarely in the automatic category.

**One VSIX can carry both entry points**: "Extensions can have both `browser`
and `main` entry points in order to run in browser and in Node.js runtimes"
([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).
Where the extension runs is decided by the available extension hosts, the
extension's capabilities, where it is installed, and its `extensionKind`
preference; in web configurations there is no local (Node.js) host, and "If an
extension is web-only, it will always run on the web extension host,
regardless of the `extensionKind` setting. We recommend to not define
`extensionKind` in that case." A `ui`-kind extension in web-with-Codespaces
can load in the web extension host only "with a limitation that it cannot
instantiate a web worker"
([extension host doc](https://code.visualstudio.com/api/advanced-topics/extension-host)).

**How Marketplace/VS Code decide installability.** The mechanism on the
publishing side is a tag: "Make sure to use the latest version of `vsce` to
publish your extension. `vsce` tags all extensions that are web extension"
using the enablement rules above
([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).
In vsce source, that is the `__web_extension` tag added when the manifest is
"web kind", plus an `executesCode` flag (`!!(manifest.main ??
manifest.browser)`) sent to the gallery; extension-kind deduction is
`main`+`browser` → `['workspace', 'web']`, `browser` only → `['web']`, no code
→ `['ui', 'workspace', 'web']` filtered down by contribution kind (e.g.
`localizations`, `debuggers`, `terminal`, `typescriptServerPlugins` exclude
web; `jsonValidation`, markdown preview styles/scripts, `html.customData`,
`css.customData` allow it)
([vsce package.ts](https://github.com/microsoft/vscode-vsce/blob/main/src/package.ts)).
On the consumer side, vscode.dev shows a warning icon and a "Learn Why" link
on extensions that cannot be installed in the web; installed extensions are
saved in the browser's local storage and can sync via Settings Sync
([vscode-web doc](https://code.visualstudio.com/docs/remote/vscode-web)).

**vscode.dev vs github.dev.** vscode.dev: "Only a subset of extensions can run
in the browser. You can use the Extensions view to install extensions in the
web, and extensions that cannot be installed will have a warning icon and
Learn Why link" ([vscode-web doc](https://code.visualstudio.com/docs/remote/vscode-web)).
github.dev is narrower still — "The github.dev editor supports VS Code
extensions that have been specifically created or updated to run in the web.
These extensions are known as 'web extensions'... Extensions that can run in
github.dev will appear in the Extensions View and can be installed" — and,
contrasted with Codespaces: "Only a subset of extensions that can run in the
web will appear in the Extensions View and can be installed. With GitHub
Codespaces, you can use most extensions from the Visual Studio Code
Marketplace"
([docs.github.com](https://docs.github.com/en/codespaces/the-githubdev-web-based-editor)).
github.dev additionally sandboxes harder: "Every extension installed in
github.dev is run under an independent web worker" (same page). The web
extension runtime is supported on VS Code desktop too, so a web extension also
runs there and in GitHub Codespaces
([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).

**Minimum engine version.** No doc states a dedicated minimum for web
extensions; the gates documented are the samples and features themselves. The
original (September 2021) version of the web extensions guide used
`"engines": { "vscode": "^1.58.0" }` in its sample
([vscode-docs @ 681ff81](https://github.com/microsoft/vscode-docs/blob/681ff81924efc7dbb999facc40258be1bc740cf2/api/extension-guides/web-extensions.md)),
and the 1.60 release notes (September 2021) introduced the web-extension
authoring story ([v1.60 notes](https://code.visualstudio.com/updates/v1_60)).
The current guide's sample declares `^1.74.0` and notes that pre-1.74 targets
must list activation events explicitly, because from 1.74 activation events
are implied by contributions
([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).
Our `^1.85.0` is comfortably above all of these.

## 3. Runtime restrictions in the web extension host

The guide's list of limitations for the browser entry file
([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)):

- "Importing or requiring other modules is not supported. `importScripts` is
  not available as well. As a consequence, the code must be packaged to a
  single file."
- `require('vscode')` works through a shim, but the shim cannot load other
  extension files or node modules.
- "Node.js globals and libraries such as `process`, `os`, `setImmediate`,
  `path`, `util`, `url` are not available at runtime," though webpack/esbuild
  can polyfill some.
- The workspace is a virtual file system: "Access to workspace files needs to
  go through the VS Code file system API accessible at `vscode.workspace.fs`."
- Extension context locations — `extensionUri`, `storageUri`,
  **`globalStorageUri`** — are also on a virtual file system and need to go
  through `vscode.workspace.fs`. So yes, a web extension can read and write
  its global storage, but only through `vscode.workspace.fs`, never Node `fs`.
- "For accessing web resources, the Fetch API must be used. Accessed resources
  need to support Cross-Origin Resource Sharing (CORS)."
- "Creating child processes or running executables is not possible. However,
  web workers can be created through the Worker API" — which is exactly how
  language servers run in the web (next section).
- The runtime "only supports the execution of JavaScript and WebAssembly";
  libraries in other languages must be cross-compiled (the guide cites C/C++
  and Rust-to-WASM tooling, and vscode-anycode's tree-sitter WASM as the
  worked example).

For a language extension specifically, the vscode-web doc summarizes what is
missing at the product level: terminal and debugger are unavailable, and
executions that need compute don't apply
([vscode-web doc](https://code.visualstudio.com/docs/remote/vscode-web)).
github.dev's comparison table says the same: no associated compute, no
terminal, and it warns "If you try to access the Run and Debug View or the
Terminal, you'll be notified that they are not available in github.dev"
([docs.github.com](https://docs.github.com/en/codespaces/the-githubdev-web-based-editor)).
The virtual-workspaces guide — the same restrictions apply because "VS Code
for the Web runs entirely inside a browser and workspaces are virtual due to
the browser sandbox" — adds that if you "run executables and tasks from
commands, check whether these commands make sense in a virtual workspace"
([virtual workspaces guide](https://code.visualstudio.com/api/extension-guides/virtual-workspaces)).
The guide does not list the authentication API as restricted; I found no
Microsoft doc restricting it in web, so I make no claim either way.

Degraded-mode plumbing is documented: `when`-clause context keys
(`virtualWorkspace`, `resourceScheme`, `shellExecutionSupported`) hide
commands that cannot work in web, and it is "fine to provide less
functionality when your extension is running in the web"
([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).
An extension can also declare `"capabilities": { "virtualWorkspaces": false }`
to be disabled in virtual-workspace windows entirely, or `"limited"` with a
description shown in the Extensions view
([virtual workspaces guide](https://code.visualstudio.com/api/extension-guides/virtual-workspaces)).
To detect the environment programmatically, `env.appHost` reports "desktop,
GitHub Codespaces, github.dev, and vscode.dev"
([v1.60 notes](https://code.visualstudio.com/updates/v1_60)).

## 4. Language servers in the web

**The vscode-languageclient browser story.** vscode-languageserver-node "since
3.16.0... provide[s] a browser implementation. The server can run in a web
worker and the connection is based on the webworkers `postMessage` protocol.
The client for the browser can be found at `vscode-languageclient/browser`;
the server at `vscode-languageserver/browser`"
([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).
Current vscode-languageclient (10.1.1) ships both runtimes as package exports:
`./node` → `lib/node/main.js`, `./browser` → `lib/browser/main.js`
([npm: vscode-languageclient](https://www.npmjs.com/package/vscode-languageclient),
source [client/src/browser/main.ts](https://github.com/microsoft/vscode-languageserver-node/blob/main/client/src/browser/main.ts)).
In the browser client, `LanguageClient`'s `ServerOptions` is
`Worker | (() => Promise<Worker | MessageTransports>)`, and a `Worker` is
bridged with `BrowserMessageReader`/`BrowserMessageWriter` from
`vscode-languageserver-protocol/browser` over the worker's `postMessage`
([client/src/browser/main.ts](https://github.com/microsoft/vscode-languageserver-node/blob/main/client/src/browser/main.ts)).

**The official sample.**
[microsoft/vscode-extension-samples/lsp-web-extension-sample](https://github.com/microsoft/vscode-extension-samples/tree/main/lsp-web-extension-sample)
is the reference architecture. Its package.json declares
`"browser": "./client/dist/browserClientMain"`, activation
`"onLanguage:plaintext"`, engines `^1.100.0`, and webpack builds (no `main` at
all — it is web-only). The client entry imports `LanguageClient` from
`vscode-languageclient/browser` and constructs it with a worker:

```ts
const serverMain = Uri.joinPath(context.extensionUri, 'server/dist/browserServerMain.js');
const worker = new Worker(serverMain.toString(true));
return new LanguageClient('lsp-web-extension-sample', 'LSP Web Extension Sample', clientOptions, worker);
```

The server entry imports `createConnection, BrowserMessageReader,
BrowserMessageWriter` from `vscode-languageserver/browser` and connects over
`self` (the worker global); everything after the connection is runtime-neutral
code that "could be shared with a regular extension." The webpack config
builds **two** bundles with `target: 'webworker'` — client and server — with
`vscode` as external and `path-browserify` as a node-polyfill fallback. Both
sample sources:
[client/src/browserClientMain.ts](https://github.com/microsoft/vscode-extension-samples/blob/main/lsp-web-extension-sample/client/src/browserClientMain.ts),
[server/src/browserServerMain.ts](https://github.com/microsoft/vscode-extension-samples/blob/main/lsp-web-extension-sample/server/src/browserServerMain.ts),
[webpack.config.js](https://github.com/microsoft/vscode-extension-samples/blob/main/lsp-web-extension-sample/webpack.config.js).
The guide notes Microsoft has converted its own JSON, CSS, and HTML language
servers to run this way ([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).

**The WASM story (running a compiled server in the worker).**
[vscode-wasm](https://github.com/microsoft/vscode-wasm) provides the
`ms-vscode.wasm-wasi-core` library extension: it "provides API to run WASM
binaries in VS Code's extension host both in the desktop and the Web. The WASM
file needs to be created with a WASI Preview 1 compliant tool chain like the
WASI-SDK or Rust using the `wasm32-wasip1` target," and it "supports the
following WASI specifications: wasi_snapshot_preview1; thread support
(wasi-threads)" ([wasm-wasi-core README](https://github.com/microsoft/vscode-wasm/blob/main/wasm-wasi-core/README.md)).
The extension itself (id `ms-vscode.wasm-wasi-core`, current version 1.0.2)
has both `main` and `browser` entry points and requires engines `^1.88.0`
([wasm-wasi-core package.json](https://github.com/microsoft/vscode-wasm/blob/main/wasm-wasi-core/package.json)).
Consumer extensions depend on it via `"extensionDependencies":
["ms-vscode.wasm-wasi-core"]` and use the `@vscode/wasm-wasi` facade
(`Wasm.load()`, `wasm.createProcess(...)` with stdio and mount points)
([wasm-wasi README](https://github.com/microsoft/vscode-wasm/blob/main/wasm-wasi/README.md)).
For LSP specifically, the npm module
[`@vscode/wasm-wasi-lsp`](https://github.com/microsoft/vscode-wasm/tree/main/wasm-wasi-lsp)
("A npm package to ease implementing language servers in WebAssembly with
WASI") supplies `createStdioOptions()` (WASI pipes for in/out/err),
`startServer(process)` (bridges the WASI pipes to LSP `MessageTransports`),
and `createUriConverters()` (maps workspace folder URIs to `file:///workspace`
inside the WASM sandbox)
([wasm-wasi-lsp/src/main.ts](https://github.com/microsoft/vscode-wasm/blob/main/wasm-wasi-lsp/src/main.ts)).

**Does our shape of server fit? Microsoft built a testbed that is exactly our
shape.** [testbeds/lsp-rust](https://github.com/microsoft/vscode-wasm/tree/main/testbeds/lsp-rust)
is a plain `lsp-server`-crate stdio server — `use lsp_server::{Connection, ...}`,
`Connection::stdio()` — the same crate (0.7.6) our crates/lsp uses, compiled
with `cargo rustc --release --target wasm32-wasi-preview1-threads`. The client
wraps it in a normal `LanguageClient` whose async `ServerOptions` compiles
`server.wasm` from the extension directory, creates the process with
`stdio: createStdioOptions()`, `mountPoints: [{ kind: 'workspaceFolder' }]`,
and shared memory `{ initial: 160, maximum: 160, shared: true }`, then returns
`startServer(process)` ([testbed extension.ts](https://github.com/microsoft/vscode-wasm/blob/main/testbeds/lsp-rust/client/src/extension.ts),
[server Cargo.toml](https://github.com/microsoft/vscode-wasm/blob/main/testbeds/lsp-rust/server/Cargo.toml),
[server/package.json](https://github.com/microsoft/vscode-wasm/blob/main/testbeds/lsp-rust/server/package.json)).
Note the testbed's own client still imports `vscode-languageclient/node` and
has no `browser` entry — it demonstrates the desktop case; the pieces are the
same but the browser wiring (worker + browser client import) is the
lsp-web-extension-sample's job.

**Threads — the question our server actually turns on.** Our server uses
`std::thread::spawn` + `sleep` for debounce and crossbeam channels. Three
verified facts: (1) vscode-wasm explicitly supports wasi-threads
([wasm-wasi-core README](https://github.com/microsoft/vscode-wasm/blob/main/wasm-wasi-core/README.md));
(2) Microsoft's own [testbeds/rust-threads](https://github.com/microsoft/vscode-wasm/tree/main/testbeds/rust-threads)
is `std::thread::spawn` with `thread::sleep(Duration::from_millis(...))` —
literally our debounce pattern — built to `wasm32-wasi-preview1-threads` and
run with shared memory (`{ initial: 17, maximum: 17, shared: true }`)
([extension.ts](https://github.com/microsoft/vscode-wasm/blob/main/testbeds/rust-threads/extension.ts),
[src/main.rs](https://github.com/microsoft/vscode-wasm/blob/main/testbeds/rust-threads/src/main.rs));
(3) in the browser, threads require shared memory, and "To use shared memory
your document must be in a secure context and cross-origin isolated"
([MDN SharedArrayBuffer](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer)).
vscode.dev serves the cross-origin-isolation headers (COOP/COEP — *observed*,
2026-09-12), so wasi-threads is at least plausible there; the vscode-wasm docs
themselves don't spell out per-surface thread guarantees, so this needs a
hands-on test before we rely on it. The safe planning assumption is that the
thread-using parts (debounce timer thread, crossbeam channels, lsp-server's
io_threads) must be proven on vscode.dev, with a single-threaded fallback if
they misbehave.

**Alternatives when the server can't run**, all from Microsoft docs: run the
server logic in-process in the extension worker (the browser client's
`ServerOptions` accepts a factory returning `MessageTransports`, so the
transport need not be a worker at all —
[client/src/browser/main.ts](https://github.com/microsoft/vscode-languageserver-node/blob/main/client/src/browser/main.ts));
compile the engine to WebAssembly and drive it in-process like vscode-anycode
does with tree-sitter
([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions));
or degrade to declarative-only support. The virtual-workspaces guide defines
the expected tiers for languages in the web — **A. Basic** (TextMate
tokenization, editing support, snippets), **B. Single-file** (symbols,
completions, hovers, formatting, same-file validation), **C. Cross-file,
workspace-aware** — and says rich extensions may legitimately ship only Basic
or Single-file in the web. It also documents the split-extension pattern used
by the built-in JSON support: a basic extension (grammar, configuration,
snippets, `"virtualWorkspaces": true`) plus a rich extension (the `main`
file, `"virtualWorkspaces": false`, `extensionDependencies` on the basic one)
([virtual workspaces guide](https://code.visualstudio.com/api/extension-guides/virtual-workspaces)).

## 5. Packaging and publishing

**One VSIX, both runtimes — yes.** Documented directly: "Extensions can have
both `browser` and `main` entry points in order to run in browser and in
Node.js runtimes" ([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).
vsce operationalizes it: extension-kind deduction treats `main`+`browser` as
`['workspace', 'web']`, and packaging validates that **both** entry files are
actually in the VSIX — a missing one fails with "Extension entrypoint(s)
missing... Make sure these files exist and aren't ignored by '.vscodeignore'"
([vsce package.ts](https://github.com/microsoft/vscode-vsce/blob/main/src/package.ts)).
So the `.vscodeignore` must keep the browser bundle (and the server/worker
bundle and any `.wasm` assets) while excluding sources, tsconfig, and
node_modules — the lsp-web-extension-sample's
[.vscodeignore](https://github.com/microsoft/vscode-extension-samples/blob/main/lsp-web-extension-sample/.vscodeignore)
excludes exactly those.

**Bundling is not optional.** Because `require` of other modules doesn't
exist, "the code must be packaged to a single file"; the guide walks through
webpack (`target: 'webworker'`, `resolve.fallback` polyfills, `process`
shim) and esbuild (`platform: 'browser'`, `external: ['vscode']`,
NodeGlobalsPolyfillPlugin) with `vscode:prepublish` running the production
build ([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).

**Publishing mechanics.** Publish with the latest vsce; vsce automatically
adds the `__web_extension` tag for web-capable extensions using the enablement
rules ([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions),
[vsce package.ts](https://github.com/microsoft/vscode-vsce/blob/main/src/package.ts)).
Declarative-only extensions "don't need any modification... Republishing is
not necessary," though new versions should be packaged with current vsce
(same guide). vsce also enforces activation-events hygiene that touches the
browser field directly: a manifest with activation events needs a `main` or
`browser`, and a manifest with a `browser` needs activation events (or, on
engines ≥ 1.74, implicit ones from contributions)
([vsce package.ts](https://github.com/microsoft/vscode-vsce/blob/main/src/package.ts)).
Standard Marketplace publishing otherwise applies
([publishing guide](https://code.visualstudio.com/api/working-with-extensions/publishing-extension)).

**Open VSX.** For Microsoft's web editors it does not matter: vscode.dev and
github.dev install from the Visual Studio Marketplace (github.dev: "web
extensions" from the Marketplace; Codespaces: "most extensions from the Visual
Studio Code Marketplace" —
[docs.github.com](https://docs.github.com/en/codespaces/the-githubdev-web-based-editor)),
and web extensions are "hosted on the Marketplace along with other extensions"
([web extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).
Open VSX is "a vendor-neutral open-source alternative to the Visual Studio
Marketplace" run by the Eclipse Foundation
([eclipse-openvsx/openvsx README](https://github.com/eclipse-openvsx/openvsx),
[about page](https://open-vsx.org/)); it exists because other VS Code-like
environments may not use Microsoft's registry — VSCodium "uses open-vsx.org"
citing the Marketplace ToU ("you may only install and use Marketplace
Offerings with Visual Studio Products and Services")
([VSCodium README](https://github.com/VSCodium/vscodium)), and code-server
documents installing extensions from OpenVSX
([code-server FAQ](https://raw.githubusercontent.com/coder/code-server/main/docs/FAQ.md)).
VSCodium ships no official web build, so Open VSX matters for our website-page
distribution story only if we care about VSCodium *desktop* or code-server
users — a separate decision.

## Facts worth keeping

- `browser` entry point = web extension; `main`-only extensions are invisible
  to the web extension host ([guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).
- Purely declarative extensions (grammar, snippets, language-configuration,
  themes) are web extensions with zero changes, as long as they don't
  contribute `localizations`, `debuggers`, `terminal`, or
  `typescriptServerPlugins` ([guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).
- Web worker sandbox: no Node APIs, no child processes, no executables, no
  dynamic module loading (single-file bundle), virtual file system via
  `vscode.workspace.fs` (including `globalStorageUri`), network via `fetch`
  with CORS, `Worker` API allowed, JavaScript + WebAssembly only
  ([guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).
- LSP in web = `vscode-languageclient/browser` + server in a `Worker` over
  `postMessage`; official sample
  [lsp-web-extension-sample](https://github.com/microsoft/vscode-extension-samples/tree/main/lsp-web-extension-sample).
- WASI server in web = `ms-vscode.wasm-wasi-core` (WASI Preview 1,
  `wasm32-wasip1`, wasi-threads supported, engines ≥ 1.88) + `@vscode/wasm-wasi-lsp`
  to map stdio to `MessageTransports`
  ([vscode-wasm](https://github.com/microsoft/vscode-wasm)).
- Microsoft's lsp-rust testbed proves an `lsp-server` stdio server (our exact
  crate) compiles to `wasm32-wasi-preview1-threads` and runs; rust-threads
  proves `std::thread::spawn` + `sleep` works under wasi-threads
  ([testbeds](https://github.com/microsoft/vscode-wasm/tree/main/testbeds)).
- Browser threads need cross-origin isolation; vscode.dev serves COOP/COEP
  (*observed*), github.dev's docs promise only "web extensions" and one
  web worker per extension
  ([MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/SharedArrayBuffer),
  [docs.github.com](https://docs.github.com/en/codespaces/the-githubdev-web-based-editor)).
- One VSIX can ship `main` + `browser`; vsce validates both entry files are
  in the package and tags web extensions `__web_extension` automatically
  ([guide](https://code.visualstudio.com/api/extension-guides/web-extensions),
  [vsce](https://github.com/microsoft/vscode-vsce/blob/main/src/package.ts)).
- Open VSX is irrelevant for vscode.dev/github.dev (Marketplace only) but
  relevant for VSCodium/code-server-style environments
  ([VSCodium](https://github.com/VSCodium/vscodium),
  [open-vsx.org](https://open-vsx.org/)).

## What this means for epher (no decision made here)

- The declarative half of clients/vscode is already web-compatible by itself;
  a user installing today's VSIX on vscode.dev would get highlighting,
  snippets, and language configuration and nothing else, because our `main`-only
  manifest is ignored by the web extension host.
- The download-and-spawn of a native `epher-lsp` binary is categorically
  impossible in web: no `child_process`, no executables, and the download
  itself would be dead weight in the bundle since `fetch` + WASM is the only
  executable payload the runtime allows.
- If we ever want diagnostics/hover/completion in web, the documented paths
  are: compile crates/lsp to `wasm32-wasi-preview1-threads` and run it through
  `ms-vscode.wasm-wasi-core` + `@vscode/wasm-wasi-lsp` (engines would have to
  rise to ^1.88.0; threads need a vscode.dev hands-on test), or rebuild the
  server logic on the already-wasm-capable epher-core and run it in-process/in
  a worker via `vscode-languageclient/browser`; or ship the split
  basic/rich-extension pattern and accept Basic-tier web support.
