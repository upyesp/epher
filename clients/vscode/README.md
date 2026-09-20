# Epher for Visual Studio Code

[![latest release](https://img.shields.io/github/v/release/upyesp/epher?label=release&sort=semver)](https://github.com/upyesp/epher/releases/latest)
[![release build](https://github.com/upyesp/epher/actions/workflows/release.yml/badge.svg)](https://github.com/upyesp/epher/actions/workflows/release.yml)

**epher** is a calculator language: you write ordinary math, with units
that convert, and every statement's answer appears inline, right next
to the line that produced it.

Download the [epher calculator](https://epher.org), and a large
selection of [ready-made scripts](https://epher.org/scripts.html) from
epher.org.

![A script computing Earth's circumference, the discriminant of a quadratic, and a speed converted from miles to kilometers per hour, each line's answer shown inline](https://github.com/upyesp/epher/raw/HEAD/clients/vscode/images/editor.png)

![The demo script typed live, each line's answer appearing as it completes](https://github.com/upyesp/epher/raw/HEAD/clients/vscode/images/demo.gif)

Type a formula and the answer is already there. No runnable repl in a
side panel, no print statements: the editor *is* the calculator.

## What you get

- **Answers inline**: each statement's result renders next to its
  line: `x = 40 + 2` shows `= 42`.
- **Units that convert**: `6371 km`, `55 mile/hr`, `30 deg` are
  quantities, not comments. `speed in km/hr` converts; the answer
  carries the right unit.
- **Live diagnostics**: syntax errors point at the exact token, and
  evaluation errors carry the same message the epher calculator shows.
- **Hover signatures**: hover any name for its canonical signature;
  your own functions show their definitions, catalog functions show
  their docs.

![Hovering `disc` shows the user-defined signature and its definition line](https://github.com/upyesp/epher/raw/HEAD/clients/vscode/images/hover.png)

- **Completion**: the whole catalog (math, astronomy, statistics),
  your own definitions, keywords, and snippets for the common
  statement shapes.

![Completion offers `sqrt` with its catalog documentation](https://github.com/upyesp/epher/raw/HEAD/clients/vscode/images/completion.png)

- **Unit-aware coloring**: semantic tokens color unit suffixes by
  meaning: the `m` in `2 m` is a unit, not a variable.

## Quick start

1. In VS Code: **Extensions** view → search for **Epher** →
   **Install** (or run `ext install upyesp.epher`). The
   [marketplace page](https://marketplace.visualstudio.com/items?itemName=upyesp.epher)
   has the listing.
2. Open any `.epher` file and start calculating.

Prefer to sideload, or on a fork (Cursor, VSCodium)? Get
`epher-vscode.vsix` from the
[releases page](https://github.com/upyesp/epher/releases/latest),
then **Extensions** view → the `⋯` menu → **Install from VSIX…**
(the wasm-wasi runtime it rides on installs itself as a dependency).

The language server ships **inside the extension**: a WebAssembly
build of the same `epher-lsp` server the other editors use. Nothing is
downloaded on first use, and it works offline.

## Running a script

Three ways to run the whole file:

- the **Run script** CodeLens at the top of the editor (the play
  icon),
- **Ctrl+Enter** / <kbd>Cmd+Enter</kbd>,
- **F5** or <kbd>Ctrl+F5</kbd>. epher ships a run-only debug adapter,
  so a debug start runs the script and prints the answers in the
  Debug Console instead of sending you to the marketplace.

epher has nothing to pause or step through — statements evaluate in
order and the answers are the whole transcript — so the debugger has
no breakpoints: F5 simply means "run". The CodeLens and Ctrl+Enter
open the **results pane** beside the editor: the same transcript,
clickable back to its lines, plus every graph the run produced as
inline SVG. The Debug Console is text-only. A `launch.json` entry
works too (`type: "epher"`, `request: "launch"`, `program: path to
script`); the Add Configuration menu offers the snippet.

## Works in the browser too

The same extension runs on [vscode.dev](https://vscode.dev) and
github.dev: press <kbd>.</kbd> on any GitHub repository to open it in
the browser, with the same inline answers.

## Requirements

- VS Code `1.88` or newer (and the forks: Cursor, VSCodium; or a web
  host). The `ms-vscode.wasm-wasi-core` runtime installs automatically
  with the extension.
- No other setup: no compiler, no runtime, no network after install.

## Troubleshooting

If the server cannot start, editing still works through the TextMate
baseline (highlighting and snippets); only the live features wait. A
window reload retries. The **Output** panel's **Epher** channel says
what happened.

## Data and telemetry

None. Everything evaluates on your machine.

## License

[MIT](https://github.com/upyesp/epher/blob/main/LICENSE)

---

<details>
<summary><strong>Building and contributing (extension developers)</strong></summary>

The extension is a thin shell; all understanding lives in
<a href="https://github.com/upyesp/epher/tree/main/crates/lsp">epher-lsp</a>,
the shared language server. The grammar (`syntaxes/epher.tmLanguage.json`)
and snippets (`snippets/epher.json`) are copies of
<a href="https://github.com/upyesp/epher/tree/main/clients/shared">clients/shared</a>;
edit there, never the copies.

    npm install
    npm run compile          # syncs shared assets, typechecks, bundles both entries
    npx @vscode/vsce package --no-dependencies

`npm run compile` bundles the desktop entry to `out/extension.js` and
the web entry to `dist/browser.js` with esbuild (single file each; the
web worker allows no module loading) and typechecks both.

The wasm must be in place first (CI does this before packaging; the
wasm is never checked in):

    rustup target add wasm32-wasip1-threads
    cargo build -p epher-lsp --target wasm32-wasip1-threads --release
    mkdir -p server
    cp ../../target/wasm32-wasip1-threads/release/epher-lsp.wasm server/

Marketplace publication is live (ADR-0068 amendment, 2026-09-18):
`upyesp.epher` publishes with the Entra ID secure automated route
(`vsce publish --azure-credential`, no personal access token). The
releases page keeps the vsix for sideloading.

</details>
