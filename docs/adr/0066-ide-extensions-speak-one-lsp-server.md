# ADR-0066: IDE extensions speak one LSP server

Date: 2026-09-17

Status: Accepted. Decided in a grilling session with the user
(seventeen questions over three rounds); this record is the outcome.
Resolves how the language reaches editors; changes nothing about the
language itself.

## Context

epher runs in a browser and PWA, a desktop shell, a terminal, and a
CLI, but not in the editors where people already write code. The plan
is extensions for seven IDE families (VS Code, the JetBrains suite,
Zed, Sublime, Open VSX for Cursor and VSCodium, Neovim and Vim, and
Microsoft Visual Studio), installable at first from a new website
page, marketplace publication following once the extensions mature.
Six features were named up front: live inline evaluation, syntax
highlighting with bracket matching, completion, ranged diagnostics,
snippets, and hover signatures.

Facts verified in the code before deciding:

- The lexer and parser carry no source positions: parse errors are
  location-less strings, so squiggly underlines, hover ranges, and
  inline results have nothing to anchor to.
- The catalog is name and kind only: hover has no canonical
  signatures or descriptions to serve.
- `run(script, env)` already evaluates a statement list against an
  environment, so a per-statement results pass has a seam to grow on.

The user's key requirement: one language server build per platform
with epher compiled in, shared by every extension (six builds, not
six times seven), so an extension is zero-config: activate it, and
the language works.

## Decision

One Rust language server, `epher-lsp`, embeds `epher-core` and speaks
LSP over stdio; thin universal extensions deliver it.

1. **Server.** A new `crates/lsp` in this workspace, built on
   `lsp-server` and crossbeam: synchronous like the core, no async
   runtime. The binary is standalone rather than an `epher`
   subcommand, so an extension never requires epher to be installed.
2. **Delivery.** Every extension is a universal thin package:
   manifest, shared TextMate grammar, shared snippets, and
   download-and-spawn glue. On first activation it downloads the
   platform binary from stable-named assets attached to the promoted
   v0.5.x GitHub release (a URL template keyed to the extension's
   own version) and caches it. Always downloads: no PATH search.
   Builds cover today's four installer platforms (linux-x86_64,
   linux-aarch64, macos-aarch64, windows-x86_64); Intel macOS and
   Windows ARM64 legs join later, and the extensions do not change
   when they do.
3. **Core prerequisites land first.** Source spans through the
   lexer, parser, and error variants; a per-statement evaluation
   trace (span and value), built on `run`; catalog entries gain
   signatures and descriptions. All three serve every frontend, not
   only the server.
4. **All six features in v1.** Inline evaluation re-runs the whole
   file on a fresh Session with a short debounce: evaluation is
   deterministic and step-bounded, so no incremental machinery.
   Highlighting is a TextMate grammar baseline plus semantic tokens
   from the server, so `i`, unit suffixes, and functions color by
   meaning. Snippets are shared static assets in the repository.
5. **Clients and rollout.** Clients live in this repository under
   `clients/`, version-locked to epher's 0.5.x train. The order is
   VS Code (pilot), then JetBrains, then Zed, then Neovim, Vim, and
   Sublime (configuration and documentation, not shipped plugins),
   then Visual Studio last. The website gains a top-level
   "IDE Extensions" menu item just after Features, one page
   localized in all eight languages at launch, linking the release
   assets until marketplace publication begins.

## Consequences

- Squiggles point at the offending tokens, inline results sit on the
  statement that produced them, and hover reads canonical
  signatures; the web and TUI result panes can adopt spans later.
- Spans, the evaluation trace, and catalog descriptions become core
  surface. The reference and the drift-guard test extend to
  descriptions when they land, so hover cannot silently rot.
- First activation needs the network once. Machines that never go
  online keep using epher through the installers and frontends that
  exist today; the PATH-first fallback was offered and declined.
- Marketplace packaging stays uniform across all seven IDE families
  (universal extension plus first-run download); per-marketplace
  platform-picking machinery is never built.
- The locked version line makes currency checkable at a glance:
  extension 0.5.x speaks epher 0.5.x.
- Build order: core prerequisites, then the server, then the VS Code
  pilot, then the website page, then the remaining IDE families.
