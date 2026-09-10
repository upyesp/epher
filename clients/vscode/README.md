# Epher for VS Code

The epher calculator language in VS Code: syntax highlighting, a
diagnostic for every broken statement, the answer rendered inline next
to every statement that produces one, hover with canonical signatures,
and completion for the catalog, your own names, and the shared
snippets.

The Extension is a thin shell (ADR-0066). All understanding lives in
`epher-lsp`, the shared language server, which this Extension
downloads on first activation and caches:

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

## The shared assets

The grammar (`syntaxes/epher.tmLanguage.json`) and the snippets
(`snippets/epher.json`) are copies of `clients/shared/`, the single
source every IDE consumes. Edit there and run `npm run compile` (which
syncs); never edit the copies.

## Building

    npm install
    npm run compile          # syncs shared assets, then tsc
    npx @vscode/vsce package # produces epher-<version>.vsix

The Windows zip layout assumed by the downloader: the binary at the
archive's root as `epher-lsp.exe` (the CI packaging stage fixes the
layout; keep them in step).

## Status

Pilot (stage three of the ADR-0066 plan). Marketplace publication is
deliberately out of scope until the extensions are proven; the website
page is the distribution until then.
