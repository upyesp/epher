# epher for Sublime Text

**epher** is a calculator language: you write ordinary math, with units
that convert, and every statement's answer appears inline, right next
to the line that produced it. The package is called `LSP-epher`
because the LSP package resolves its client configuration by that
name.

Download the [epher calculator](https://epher.org), and a large
selection of [ready-made scripts](https://epher.org/scripts.html) from
epher.org.

![A script computing Earth's circumference, the discriminant of a quadratic, and a speed converted from miles to kilometers per hour, each line's answer shown inline](https://github.com/upyesp/epher/raw/HEAD/clients/sublime/images/editor.png)

![The demo script typed live, each line's answer appearing as it completes](https://github.com/upyesp/epher/raw/HEAD/clients/sublime/images/demo.gif)

Type a formula and the answer is already there. No runnable repl in a
side panel, no print statements: the editor *is* the calculator.

## What you get

- **Answers inline**: each statement's result renders next to its
  line: `x = 40 + 2` shows `= 42`. The LSP package renders inlay hints
  off by default; the settings section below turns them on.
- **Units that convert**: `6371 km`, `55 mile/hr`, `30 deg` are
  quantities, not comments. `speed in km/hr` converts; the answer
  carries the right unit.
- **Live diagnostics**: syntax errors point at the exact token, and
  evaluation errors carry the same message the epher calculator shows.
- **Hover signatures**: hover any name for its canonical signature;
  your own functions show their definitions, catalog functions show
  their docs.

![Hovering a defined name shows its signature and definition](https://github.com/upyesp/epher/raw/HEAD/clients/sublime/images/hover.png)

- **Completion**: the whole catalog (math, astronomy, statistics),
  your own definitions, keywords, and snippets for the common
  statement shapes.

![Completion offers a catalog name with its documentation](https://github.com/upyesp/epher/raw/HEAD/clients/sublime/images/completion.png)

- **Highlighting**: the shared TextMate grammar colors the epher
  syntax before the server even attaches: comments, strings, numbers,
  the keywords, and the conservative unit rule. Semantic-token
  coloring from the server is off by default and needs the two steps
  under Settings; custom color schemes need one extra rule.
- **Running scripts**: **LSP-epher: Run Script** in the command
  palette (there is no keybinding out of the box: see the suggested
  one under Settings). The results view beside the script lists one
  row per statement, error rows in red; every graph the run produced
  is written as an SVG file under Sublime's cache folder and opened
  with the system viewer.
- **Unsaved tabs**: a buffer that has never been saved works like a
  file, diagnostics, answers, completion and the run command
  included, as long as its syntax is epher.

![The results view after a run, with the per-statement transcript and the graph path](https://github.com/upyesp/epher/raw/HEAD/clients/sublime/images/results.png)

## Install

From Package Control, once the package is listed:

1. Open the command palette and run **Package Control: Install
   Package**.
2. Search for **LSP** and install it. This is a real manual step:
   Package Control has no way for one package to install another
   (dependencies there are Python libraries, not packages), so LSP is
   never pulled in for you.
3. Search for **LSP-epher** and install it.

From this repository, today: copy or symlink this directory into your
`Packages` folder under the name `LSP-epher` (a direct child of
`Packages`, not inside `User`):

- linux: `~/.config/sublime-text/Packages/LSP-epher`
- macOS: `~/Library/Application Support/Sublime Text/Packages/LSP-epher`
- windows: `%APPDATA%\Sublime Text\Packages\LSP-epher`

Then install the **LSP** package from Package Control. Open any
`.epher` file: Sublime applies the syntax automatically (the grammar
carries `fileTypes`), and the LSP package reads
`LSP-epher.sublime-settings` and starts `epher-lsp` for epher views.

## Getting the server binary

Download the asset for your platform from the
[releases page](https://github.com/upyesp/epher/releases/latest),
uncompress it, and put it on your PATH (or write the full path into
the `command` array in `LSP-epher.sublime-settings`):

```sh
# linux x86_64 (arm64 and macos-aarch64 analogous)
curl -LO https://github.com/upyesp/epher/releases/latest/download/epher-lsp-linux-x86_64.gz
gunzip epher-lsp-linux-x86_64.gz && mv epher-lsp-linux-x86_64 ~/.local/bin/epher-lsp
chmod +x ~/.local/bin/epher-lsp

# windows (powershell): epher-lsp-windows-x86_64.zip -> epher-lsp.exe
```

Everything runs on your machine after that one download.

## Settings

The LSP package ships inlay hints off, so the inline answers need one
setting in `Packages/User/LSP.sublime-settings`:

```json
{ "show_inlay_hints": true }
```

If the answers still do not appear, run **LSP: Toggle Inlay Hints**
once from the command palette in that window. The per-window flag is
what drives the phantom set in current LSP builds.

### Semantic highlighting

Semantic-token coloring is also off by default. In the same file, add:

```json
{ "semantic_highlighting": true }
```

LSP adds the rule that semantic tokens need to the built-in color
schemes itself. With a custom color scheme, add it to that scheme
(**UI: Customize Color Scheme**) or the tokens render with the
background they already have:

```json
{
  "rules": [
    { "scope": "meta.semantic-token", "background": "#00000101" }
  ]
}
```

Two known LSP limitations apply: with `highlight_line` enabled there
are visible artifacts on highlighted lines, and bold or italic font
styles do not apply to semantic regions.

### Suggested keybinding

This package deliberately ships no keybinding, so nothing of yours
gets shadowed. To run a script from the keyboard, open
**Preferences: Key Bindings** from the Command Palette and add:

```json
[
  {
    "keys": ["ctrl+alt+r"],
    "command": "lsp_epher_run",
    "context": [{ "key": "selector", "operand": "source.epher" }]
  }
]
```

`ctrl+alt+r` is unbound in Sublime Text 4's default keymaps on Linux,
macOS and Windows.

## Requirements

- Sublime Text 4 (build 4132 or newer, the floor the LSP package
  sets).
- The LSP package from Package Control, installed separately (see
  Install), version 2.11 or newer: the client uses LSP's `LspPlugin`
  API, introduced in that release.
- The `epher-lsp` binary for your platform, on PATH or named in
  `LSP-epher.sublime-settings`.

## Data and telemetry

None. Everything evaluates on your machine.

## License

[MIT](https://github.com/upyesp/epher/blob/main/LICENSE)

---

## Maintainer note (not part of the listing copy)

Package Control renders the README of the standalone
`upyesp/LSP-epher` repository. Everything above this divider is the
listing copy; the sync copies just that part to the repository's
`README.md`. The package root is assembled from this folder and must
stay flat:

- `plugin.py`, `LSP-epher.sublime-settings`,
  `LSP-epher.sublime-commands`
- `epher.tmLanguage` (regenerated from `clients/shared` by
  `sync-assets.py`)
- `README.md` (the listing copy above) and `LICENSE`
- `.python-version` containing `3.8`, so the plugin loads on the
  modern host like the other LSP-* packages

No keymap ships, on the LSP team's review: a package should not bind
keys, and the earlier ctrl+c ctrl+c binding shadowed Copy. The
suggested binding lives in the README instead.

The Package Control entry lives in `sublimelsp/repository`, with
`"tags": true`, so each `v<version>` train tag becomes a release
within about an hour. See
`docs/research/sublime-packagecontrol-publishing.md` for the full
picture.
