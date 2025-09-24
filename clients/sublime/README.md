# epher for Sublime Text

A package folder that teaches Sublime Text the epher syntax and
wires the shared `epher-lsp` language server through the popular
[LSP](https://packagecontrol.io/packages/LSP) package (sublimelsp).
ADR-0066 ships Sublime as configuration and documentation, not a
plugin; this folder is that configuration.

## What each file does

- `epher.tmLanguage`: the shared TextMate grammar in the plist XML
  form Sublime loads (scope `source.epher`). Generated from
  `clients/shared/epher.tmLanguage.json` by `sync-assets.py`; edit
  the shared file, never this copy.
- `LSP-epher.sublime-settings`: the LSP-package client definition.
- `sync-assets.py`: the generator (needs only Python 3).

## Install

1. copy (or symlink) this directory into your `Packages/User` folder
   under the name `epher`:
   - linux: `~/.config/sublime-text/Packages/User/epher`
   - macOS: `~/Library/Application Support/Sublime Text/Packages/User/epher`
   - windows: `%APPDATA%\Sublime Text\Packages\User\epher`
2. install the `LSP` package from Package Control;
3. copy `LSP-epher.sublime-settings` into `Packages/User/` (that is
   the file name the LSP package looks for);
4. open any `.epher` file. Sublime applies the syntax automatically;
   set it once by hand via the syntax menu if it was already open.

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
  grammar).
