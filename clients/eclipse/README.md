# epher for Eclipse

A plugin for the Eclipse IDE that wires the shared `epher-lsp`
language server (ADR-0066) into any `.epher` file: live diagnostics,
hover, and completion through LSP4E, plus the same TextMate grammar
every other client uses for baseline highlighting (through TM4E).
On recent Eclipse releases the inline answers — the value each
statement produces — appear as inlay hints next to the code.

There is nothing to configure. The plugin registers the `.epher`
content type, starts `epher-lsp` when an epher editor opens, and
stops it when the last one closes. Like the emacs/vim/neovim
clients, it never downloads anything: you install the server binary
once, the plugin finds it on `PATH`.

## Install

Two pieces, both one-time:

**1. The server binary.** Download the asset for your platform from
the releases page (`https://github.com/upyesp/epher/releases`),
uncompress it, and put it on your `PATH`:

```sh
# linux x86_64 (arm64 and macos-aarch64 analogous)
curl -LO https://github.com/upyesp/epher/releases/latest/download/epher-lsp-linux-x86_64.gz
gunzip epher-lsp-linux-x86_64.gz && mv epher-lsp-linux-x86_64 ~/.local/bin/epher-lsp
chmod +x ~/.local/bin/epher-lsp

# windows (powershell): epher-lsp-windows-x86_64.zip -> epher-lsp.exe
```

**2. The plugin.** Drop `epher-eclipse.jar` into the `dropins`
folder of your Eclipse install and (re)start Eclipse:

```sh
mv ~/Downloads/epher-eclipse.jar /opt/eclipse/dropins/
```

Then open any `.epher` file — the generic editor picks it up with
highlighting, and the language server starts behind it.

## Troubleshooting

- **No highlighting / no server**: the IDE needs LSP4E and TM4E.
  Both ship in the standard Eclipse IDE packages (2023-12 or
  newer); if yours lacks them, install them from the marketplace
  (Help → Eclipse Marketplace → "LSP4E").
- **"Unable to start language server"**: `epher-lsp` is not on the
  `PATH` Eclipse sees. Check with `which epher-lsp` inside
  *Help → Show shell* or a terminal launched the same way; the
  server definition is visible under *Preferences → Language
  Servers*.
- **macOS, Eclipse launched from the Dock**: the Finder/Dock
  environment does not inherit your shell `PATH`. Start Eclipse
  from a terminal, or symlink the binary:
  `sudo ln -s ~/.local/bin/epher-lsp /usr/local/bin/epher-lsp`.

## Uninstall

Delete the jar from `dropins/` and restart. Nothing else is touched.
