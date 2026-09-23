# epher for Zed

A Zed extension that brings epher into Zed through the shared
`epher-lsp` language server (ADR-0066). On language-server start it
downloads the `epher-lsp` binary for this platform from the GitHub
release matching the extension's own version, caches it in the
extension's work directory, and starts it over stdio.

## What works in v1

- live diagnostics: parse errors point at the exact token; evaluation
  errors carry the same message the calculator shows;
- inline answers: every statement's result renders next to its line
  (`x = 40 + 2` shows `= 42`);
- hover signatures: canonical signature and description for catalog
  names, current values for your own constants;
- completion: catalog names, your own definitions, keywords, and the
  shared snippets (served by the server itself);
- semantic-token coloring once you enable it (next section): numbers,
  variables, functions, keywords, operators, strings, and unit
  suffixes colored by meaning. Without that one settings change, Zed
  colors nothing here.

## Highlighting: no colors until one setting is on

A freshly installed extension recognizes `.epher` files (comments,
brackets, completion, the language server all work), but it colors
nothing. That is Zed's default, not a broken install: Zed ships its
`semantic_tokens` setting as "off", and this extension carries no
tree-sitter grammar to fall back on (why is the next section). All
coloring comes from the language server's semantic tokens, so turn
them on — this is step four of installing, not an optional
refinement:

```json
{
  "languages": {
    "epher": { "semantic_tokens": "full" }
  }
}
```

`full` makes the server's tokens the only source of coloring, which
is the right mode for a grammar-less language. The extension ships
`semantic_token_rules.json` for the custom `unit` token type; the
standard token types use Zed's built-in rules.

## Installing

Distribution is by dev extension until the extension is proven
(marketplace publication comes later, per ADR-0066):

1. clone this repository (or download and unzip `epher-zed.zip`
   from the release);
2. in Zed, open the command palette and run `zed: install dev
   extension`;
3. select this directory (`clients/zed`);
4. turn semantic tokens on, or nothing will ever be colored:

```json
{
  "languages": {
    "epher": { "semantic_tokens": "full" }
  }
}
```

Zed compiles the extension itself, which needs Rust installed; Zed
runs the build against the `wasm32-wasip2` target and adds it
through rustup when it is missing. First use needs the network once,
to fetch the server binary; after that everything is local.

Platforms: linux x86_64 and ARM64, macOS Apple silicon, Windows
x86_64. Intel macOS and Windows ARM64 join when ADR-0066's second
platform wave lands.

## The grammar, and why it is not here

The [tree-sitter-epher](https://github.com/upyesp/tree-sitter-epher)
grammar (written from the same source as this extension,
`grammars/tree-sitter-epher` in the epher repository; it parses the
whole shipped `epher scripts` corpus - all 433 scripts - without
errors) still exists for tooling, but it cannot ride in a dev
extension. In Zed's installer, any declared grammar makes the
install resolve a wasi-sdk toolchain before it even looks at a
prebuilt `grammars/<name>.wasm`, and a grammar absent from the
manifest is never loaded — so there is no way to ship the parser to
dev-extension users without the exact toolchain step that fails in
sandboxes (the 0.5.47 field reports; checked against Zed's
extension_builder source). The grammar comes back with marketplace
publication: registry installs download a finished package, no
toolchain touches the user's machine, and that package can carry
the parser compiled once in epher's own CI from the pinned
revision.

## Running scripts, honestly

Running a whole script and a results pane (ADR-0069) is the one thing
this extension cannot bring to Zed: Zed's extension API has no
commands, no panels, and no webviews, so there is nowhere to put a
pane and nothing to hang a run action on. What Zed users have instead:

- the inline answers stay live beside every statement as you type, so
  short scripts show their answers without any run step at all;
- whole-script runs happen in Zed's integrated terminal with the
  calculator: `epher run my-script.epher` prints the same transcript
  and writes `graph save` plots exactly as everywhere else.

If Zed's extension API grows a surface for commands or panels, the
extension picks up `epher/run` like every other editor did.

## Verifying by hand

```
cargo build --target wasm32-wasip2 --release
```

`wasm32-wasip2` is the target Zed compiles dev extensions with
(`wasm32-wasip1` answers only an older Zed). Zed runs this build
itself at install time and adds the target through rustup when it
is missing.

## If it does not work

A failed install shows an error card in Zed ("Failed to install dev
extension"). When the install succeeds but the extension seems dead,
the reasons live in Zed's log:

- Windows: `%LOCALAPPDATA%\Zed\logs\Zed.log`
- macOS: `~/Library/Logs/Zed/Zed.log`
- Linux: `~/.local/share/zed/logs/Zed.log`
  (Flatpak: `~/.var/app/dev.zed.Zed/data/zed/logs/Zed.log`)

Open it and search for `epher`. The lines that matter:

- `compiling Rust extension` / `finished compiling extension` —
  the install itself;
- `Failed to load extension` — the install finished but Zed refused
  the compiled extension (version skew; file an issue with the line);
- `epher-lsp` — the server download and start, which happens the
  first time a `.epher` file opens.

The installed extension also sits on disk as a link named
`extensions/installed/epher` under Zed's data directory
(`%LOCALAPPDATA%\Zed` on Windows, `~/Library/Application Support/Zed`
on macOS, `~/.local/share/zed` on Linux) pointing back at this
folder. If the link is there but the Extensions page (Installed
section, the card reads `epher` with a dev tag) disagrees, restart
Zed: the page re-scans that directory at startup.

## Versioning

The version here (`extension.toml` and `Cargo.toml`, kept in
lockstep) is the release the extension resolves: tag `v<version>`.
It rides epher's 0.5.x train, so bumping epher bumps this in the
same batch; a CI job fails the build when the locks drift.
