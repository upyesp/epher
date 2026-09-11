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
- semantic-token coloring: numbers, variables, functions, keywords,
  operators, strings, and unit suffixes colored by meaning.

## Baseline highlighting ships; one setting makes it exact

The extension pins the [tree-sitter-epher](https://github.com/upyesp/tree-sitter-epher)
grammar, so baseline highlighting, bracket matching, and outline work
out of the box. For the server's exact coloring (unit suffixes versus
a variable named m), turn on semantic tokens in your settings:

```json
{
  "languages": {
    "epher": { "semantic_tokens": "full" }
  }
}
```

The extension ships `semantic_token_rules.json` for the custom `unit`
token type; the standard token types use Zed's built-in rules.

## Installing

Distribution is by dev extension until the extension is proven
(marketplace publication comes later, per ADR-0066):

1. clone this repository;
2. in Zed, open the command palette and run `zed: install dev
   extension`;
3. select this directory (`clients/zed`).

Zed compiles the extension itself, which needs Rust installed
(`rustup target add wasm32-wasip1` provides the target). First use
needs the network once, to fetch the server binary; after that
everything is local.

Platforms: linux x86_64 and ARM64, macOS Apple silicon, Windows
x86_64. Intel macOS and Windows ARM64 join when ADR-0066's second
platform wave lands.

## The grammar

The tree-sitter grammar lives in its own repository (the Zed registry
pins a revision): [upyesp/tree-sitter-epher](https://github.com/upyesp/tree-sitter-epher),
written from the same source as this extension
(`grammars/tree-sitter-epher` in the epher repository). It parses the
whole shipped `epher scripts` corpus - all 433 scripts - without
errors.

## Verifying by hand

```
cargo build --target wasm32-wasip1 --release
```

The extension compiles for `wasm32-wasip1`, the target Zed loads.

## Versioning

The version here (`extension.toml` and `Cargo.toml`, kept in
lockstep) is the release the extension resolves: tag `v<version>`.
It rides epher's 0.5.x train, so bumping epher bumps this in the
same batch. When the grammar moves, bump its `rev` in
`extension.toml` in the same commit.
