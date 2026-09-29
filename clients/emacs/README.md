# epher for Emacs

**epher** is a calculator language: you write ordinary math, with units
that convert, and every statement's answer appears inline, right next
to the line that produced it. This is the client glue for the shared
`epher-lsp` server: `epher-mode` for `.epher` files and eglot for the
language features. Requires Emacs 29 or newer, where eglot ships built
in. With lsp-mode installed instead, the client is registered there
too.

Download the [epher calculator](https://epher.org), and a large
selection of [ready-made scripts](https://epher.org/scripts.html) from
epher.org.

![A script computing Earth's circumference, the discriminant of a quadratic, and a speed converted from miles to kilometers per hour, each line's answer shown inline](https://github.com/upyesp/epher/raw/HEAD/clients/emacs/images/editor.png)

![The demo script typed live, each line's answer appearing as it completes](https://github.com/upyesp/epher/raw/HEAD/clients/emacs/images/demo.gif)

Type a formula and the answer is already there. No runnable repl in a
side panel, no print statements: the editor *is* the calculator.

## What you get

- **Answers inline**: each statement's result renders next to its
  line: `x = 40 + 2` shows `= 42`, as inlay hints.
- **Units that convert**: `6371 km`, `55 mile/hr`, `30 deg` are
  quantities, not comments. `speed in km/hr` converts; the answer
  carries the right unit.
- **Live diagnostics**: syntax errors point at the exact token, and
  evaluation errors carry the same message the epher calculator shows,
  in the standard eglot flymake integration.
- **Hover signatures**: hover any name for its canonical signature in
  eldoc; your own functions show their definitions, catalog functions
  show their docs.

![Hovering a defined name shows its signature and current value](https://github.com/upyesp/epher/raw/HEAD/clients/emacs/images/hover.png)

- **Completion**: the whole catalog (math, astronomy, statistics),
  your own definitions, keywords, and snippets for the common
  statement shapes.

![Completion offers a catalog name with its documentation](https://github.com/upyesp/epher/raw/HEAD/clients/emacs/images/completion.png)

- **Definition jumps**: `xref` jumps for names defined in the file.
- **Run the script**: `C-c C-c` sends the same `epher/run` request the
  VS Code results pane uses. The per-statement transcript lands in the
  `*epher run*` buffer, and every graph the script produced is saved
  as an SVG file and opened with the system viewer. `g` in the results
  buffer re-runs the script, `q` closes it.

![The epher run buffer with the per-statement transcript and the graph paths](https://github.com/upyesp/epher/raw/HEAD/clients/emacs/images/results.png)

- **Highlighting**: `epher-mode` is the conservative view of the
  grammar: comments, strings, numbers, the twenty keywords, and
  number-adjacent units. The language server's semantic tokens are the
  exact rule; eglot does not consume them today, so the font-lock
  layer is what you see.

## Install

From MELPA, once the package is published: add MELPA to
`package-archives` if it is not there yet, then

```
M-x package-install RET epher RET
```

From this repository, today: put `epher.el` on your `load-path` and
load it. With use-package:

```elisp
(use-package epher
  :load-path "~/code/epher/clients/emacs")
```

Or plain:

```elisp
(add-to-list 'load-path "~/code/epher/clients/emacs")
(require 'epher)
```

Opening a `.epher` file turns on `epher-mode`. When the server binary
is on your `exec-path`, eglot starts by itself; point it elsewhere by
customizing the command:

```elisp
(setq epher-server-program '("/path/to/epher-lsp"))
```

Set `epher-lsp-autostart` to nil if you start the server yourself or
run it through lsp-mode (the client is registered there as
`epher-lsp`, so `lsp` in an `epher-mode` buffer just works).

## Getting the server binary

Download the asset for your platform from the
[releases page](https://github.com/upyesp/epher/releases/latest),
uncompress it, and mark it executable:

```sh
# linux x86_64 (arm64 and macos-aarch64 analogous)
curl -LO https://github.com/upyesp/epher/releases/latest/download/epher-lsp-linux-x86_64.gz
gunzip epher-lsp-linux-x86_64.gz && mv epher-lsp-linux-x86_64 ~/.local/bin/epher-lsp
chmod +x ~/.local/bin/epher-lsp

# windows (powershell): epher-lsp-windows-x86_64.zip -> epher-lsp.exe
```

First download needs the network once; the server runs entirely
locally after that.

## Requirements

- Emacs 29.1 or newer, for the built-in eglot. lsp-mode is optional
  and registered as an alternative.
- The `epher-lsp` binary for your platform, on `exec-path` or named
  in `epher-server-program`.

## Data and telemetry

None. Everything evaluates on your machine.

## License

[MIT](https://github.com/upyesp/epher/blob/main/LICENSE)
