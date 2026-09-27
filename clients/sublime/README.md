# epher for Sublime Text

A package folder that teaches Sublime Text the epher syntax and
wires the shared `epher-lsp` language server through the popular
[LSP](https://packagecontrol.io/packages/LSP) package (sublimelsp).

## What each file does

- `epher.tmLanguage`: the shared TextMate grammar in the plist XML
  form Sublime loads (scope `source.epher`). Generated from
  `clients/shared/epher.tmLanguage.json` by `sync-assets.py`; edit
  the shared file, never this copy.
- `LSP-epher.sublime-settings`: the LSP-package client definition.
- `epher.py`: the run command (`lsp_epher_run`, the chord
  ctrl+c ctrl+c, or "epher: run this script" in the command
  palette). Sends the same `epher/run` request the VS Code results
  pane uses (ADR-0069) and shows every answer and error in a
  results view; graphs are written as SVG files under Sublime's
  cache folder (`epher/runs/`) and opened with the system viewer.
- `LSP-epher.sublime-commands`: the palette entry for the run
  command.
- `Default (<platform>).sublime-keymap`: the run chord, ctrl+c
  ctrl+c on Windows and Linux, super+c super+c on macOS, bound only
  in epher views.
- `sync-assets.py`: the grammar generator (needs only Python 3).

## Install

1. copy (or symlink) this directory into your `Packages` folder
   under the name `LSP-epher` (a direct child of `Packages`, not
   inside `User`). The `LSP-` prefix is required: the LSP package
   resolves the client configuration at
   `Packages/LSP-epher/LSP-epher.sublime-settings`, so the folder
   and file names must match exactly:
   - linux: `~/.config/sublime-text/Packages/LSP-epher`
   - macOS: `~/Library/Application Support/Sublime Text/Packages/LSP-epher`
   - windows: `%APPDATA%\Sublime Text\Packages\LSP-epher`
2. install the `LSP` package from Package Control;
3. open any `.epher` file. Sublime applies the syntax automatically
   (the grammar carries `fileTypes`); the LSP package reads
   `LSP-epher.sublime-settings` and starts `epher-lsp` for epher
   views.

## Getting the binary

Download the asset for your platform from the releases page
(`https://github.com/upyesp/epher/releases`), uncompress it, and put
it on your PATH (or write the full path into the `command` array in
`LSP-epher.sublime-settings`):

```sh
# linux x86_64 (arm64 and macos-aarch64 analogous)
curl -LO https://github.com/upyesp/epher/releases/latest/download/epher-lsp-linux-x86_64.gz
gunzip epher-lsp-linux-x86_64.gz && mv epher-lsp-linux-x86_64 ~/.local/bin/epher-lsp
chmod +x ~/.local/bin/epher-lsp

# windows (powershell): epher-lsp-windows-x86_64.zip -> epher-lsp.exe
```

Everything runs on your machine after that one download.

## What you get

- TextMate baseline highlighting (the same grammar the other
  editors share), bracket matching, and comment toggling;
- live diagnostics, inline answers, hover signatures, completion
  with the shared snippets, and semantic-token coloring, from the
  server (LSP package >= 1.16 renders semantic tokens on top of the
  grammar);
- running scripts: ctrl+c ctrl+c (or the palette entry) sends the
  same `epher/run` request the other clients use and opens the
  results view beside the script; `<CR>`-style jumps are not part
  of the Sublime pane, but every graph row carries the SVG path it
  was written to, and each graph opens with the system viewer as
  the run completes.
