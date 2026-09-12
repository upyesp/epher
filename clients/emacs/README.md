# epher for Emacs

The LSP client glue for the shared `epher-lsp` server (ADR-0066
ships Emacs as a ready-made config, not a package.el plugin).
Requires Emacs 29 or newer — that is where eglot ships built in.
With lsp-mode installed instead, the client is registered there too.

## Install

Put `epher.el` on your `load-path` and load it. With use-package:

```elisp
(use-package epher
  :load-path "~/code/epher/clients/emacs")
```

Or plain:

```elisp
(add-to-list 'load-path "~/code/epher/clients/emacs")
(require 'epher)
```

Opening a `.epher` file turns on `epher-mode`. When the server
binary is on your `exec-path`, eglot starts by itself; point it
elsewhere by customizing the command:

```elisp
(setq epher-server-program '("/path/to/epher-lsp"))
```

Set `epher-lsp-autostart` to nil if you start the server yourself
or run it through lsp-mode (the client is registered there as
`epher-lsp`, so `lsp` in an `epher-mode` buffer just works).

## Getting the binary

Download the asset for your platform from the releases page
(`https://github.com/upyesp/epher/releases`), uncompress it, and
mark it executable:

```sh
# linux x86_64 (arm64 and macos-aarch64 analogous)
curl -LO https://github.com/upyesp/epher/releases/latest/download/epher-lsp-linux-x86_64.gz
gunzip epher-lsp-linux-x86_64.gz && mv epher-lsp-linux-x86_64 ~/.local/bin/epher-lsp
chmod +x ~/.local/bin/epher-lsp

# windows (powershell): epher-lsp-windows-x86_64.zip -> epher-lsp.exe
```

First download needs the network once; the server runs entirely
locally after that.

## What you get

- live diagnostics from spans (parse and evaluation errors), in the
  standard eglot flymake integration;
- inline answers on the statement that produced them (inlay hints);
- hover signatures for catalog names, current values for your own
  constants (eldoc);
- completion with the shared snippets;
- `xref` definition jumps for names defined in the file.

Baseline highlighting in `epher-mode` is the conservative view of
the grammar — comments, strings, numbers, the twenty keywords, and
number-adjacent units. The language server's semantic tokens are
the exact rule; eglot does not consume them today, so the font-lock
layer is what you see.

## Verify by hand

```elisp
M-x eglot RET          ;; in an epher-mode buffer
M-x eglot-events-buffer RET
M-x eglot-reconnect RET
```
