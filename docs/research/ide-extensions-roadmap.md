# IDE extensions roadmap: one LSP server, seven IDE families

Date: 2026-09-17 · Status: decisions and staging plan, nothing
implemented. The architecture decision is ADR-0066; this note is the
working sequence and the decisions ledger behind it.

## What was asked

Extensions for seven IDE families, initially installable from a new
website page ("IDE Extensions", top-level menu just after Features),
marketplace publication later once mature. The named feature set:
live inline evaluation, syntax highlighting and bracket matching,
auto-completion, real-time diagnostics, snippets, and hover
signatures. Stage one is a single epher LSP server reused by every
extension, kept current with the language by building on the same
Rust core the other frontends use.

## Decisions ledger

Seventeen questions in a grilling session settled the design. The
answers, in the order asked:

| # | Question | Decision |
| --- | --- | --- |
| 1 | How the server ships | Standalone `epher-lsp` binary, epher compiled in; one build per platform, shared by every extension (user requirement: 6 × 1, not 6 × 7) |
| 2 | Server framework | `lsp-server` + crossbeam, synchronous |
| 3 | Server location | `crates/lsp` in this workspace |
| 4 | Source spans | Add to lexer, parser, and error variants now (today nothing carries positions) |
| 5 | Inline-eval data path | Per-statement trace API in core (span + value), on the `run()` seam |
| 6 | Hover and completion fuel | Catalog entries gain signatures and descriptions in core |
| 7 | v1 feature cut | All six features in v1 |
| 8 | Evaluation model | Whole-file re-evaluation, fresh Session, short debounce |
| 9 | Rollout order | VS Code, JetBrains, Zed, then Neovim/Vim/Sublime (configs and docs, not plugins), Visual Studio last |
| 10 | Client layout | Monorepo `clients/` directories |
| 11 | Versioning | Locked to epher's 0.5.x train |
| 12 | Website page | Top-level nav after Features; one page, all eight languages at launch |
| 13 | Platforms | Today's four first (linux-x86_64, linux-aarch64, macos-aarch64, windows-x86_64); Intel macOS and Windows ARM64 legs later |
| 14 | Binary hosting | Stable-named assets on the promoted v0.5.x GitHub release; URL template keyed to the extension version |
| 15 | Local detection | Always download; no PATH search (PATH-first was offered and declined) |
| 16 | Build order | Core prerequisites, then server, then VS Code pilot |
| 17 | Highlighting mechanism | Shared TextMate grammar baseline plus LSP semantic tokens |

Two decisions changed shape mid-session and are worth recording. Q1
was first posed as subcommand versus bundling; the user's answer
restated the requirement (one server per platform, epher bundled in,
shipped with all extensions) and asked for investigation, which
confirmed the requirement is the cheap path: the binary is about 4 to
6 MB stripped per platform, and the extensions stay under 1 MB. Q13
was posed as six platforms; the user chose today's four first, so the
"six platforms" of the original brief arrive as a later, additive
step.

## Stage plan

1. **Core prerequisites** (all in `epher-core`, every frontend
   benefits):
   - source spans through lexer, parser, and `EpherError` variants;
   - a per-statement evaluation trace: parse, run with `run()`,
     record each statement's span and produced value;
   - catalog enrichment: signature and description strings per
     entry, with the reference and drift-guard test extended to
     cover them.
2. **The server** (`crates/lsp`, binary `epher-lsp`):
   - LSP over stdio on `lsp-server` + crossbeam;
   - diagnostics from spans (parse and evaluation errors), hover
     from catalog descriptions, completion from the catalog plus the
     document's session, semantic tokens, inline results from the
     trace, snippets from shared assets;
   - full-file re-evaluation on a fresh Session per pass, debounced.
3. **The pilot client** (`clients/vscode`):
   - universal extension: manifest, shared TextMate grammar, shared
     snippets, download-and-spawn glue;
   - first activation downloads `epher-lsp` for the platform from
     the release asset URL and caches it;
   - exercises all six features end to end.
4. **The website page**: top-level "IDE Extensions" nav item after
   Features; per-IDE install instructions; links to the release
   assets; localized in all eight languages before promotion.
5. **Remaining IDEs in order**: JetBrains (first-class built-in LSP
   client), Zed (downloads the server per its extension API), then
   Neovim, Vim, and Sublime as configuration and documentation
   pointing at the same release assets, then Visual Studio.

## Asset naming

Release assets follow the installer pattern, one stable name per
platform, attached to the promoted release:

    epher-lsp-linux-x86_64.gz
    epher-lsp-linux-aarch64.gz
    epher-lsp-macos-aarch64.gz
    epher-lsp-windows-x86_64.zip

The extension resolves `releases/download/v<its own version>/<name>`,
so an extension update re-fetches a matching server. Test builds ride
the staging draft releases under the existing staged-delivery flow
(ADR-0062).

## Known trade-offs and open items

- First activation needs the network once. Offline machines keep
  using the installers and the frontends that exist today; a server
  path override setting could be added later without breaking the
  always-download default.
- Windows ARM64 and Intel macOS need new CI legs when the remaining
  two platforms are added; both are routine for a pure-Rust binary.
- Marketplace publication (VS Code Marketplace, Open VSX, JetBrains
  Marketplace, Zed extensions) is deliberately out of stage one; the
  website page is the distribution until the extensions are proven.
- No code exists yet; this note and ADR-0066 are the record of what
  will be built and why.

## Stage five implementation notes

The JetBrains plugin exists now (clients/jetbrains, Kotlin, packaged
by a jetbrains job in build-installers.yml as epher-jetbrains.zip),
and the build wrote itself only after the platform sources settled a
dozen guesses. The verified facts, for whoever touches this next:

- The LSP API ships only in the commercial IDEs. The ideaIC 2024.2.4
  artifact carries no com/intellij/platform/lsp classes at all (checked
  the distribution jars); IDEA Ultimate 242.23726 keeps them in
  lib/product.jar. So the plugin compiles against intellijIdeaUltimate,
  not Community as stage two assumed, and the README says which IDEs
  work.
- At 242 there is no com.intellij.modules.lsp module to depend on
  (product-info.json lists none); the docs that ask for it target
  2024.3+. The serverSupportProvider extension point of the
  com.intellij.platform.lsp namespace lives in the IDE product itself,
  so com.intellij.modules.platform plus the textmate plugin suffice.
- The 242 provider shape: LspServerSupportProvider is an interface with
  one abstract method, fileOpened(project, file, starter), and the
  starter's ensureServerStarted(descriptor) starts the server. The
  descriptor's only abstract member is isSupportedFile(file);
  startServerProcess() and createCommandLine() are open with bodies
  (the default startServerProcess logs an error, so it is overridden).
  getFilePaths() from newer docs does not exist at 242; roots plus
  isSupportedFile play that role.
- The TextMate plugin reads bundles from a directory, not a bare
  .tmLanguage file: a package.json makes it a VS Code bundle, and the
  reader consumes the same manifest shape the VS Code pilot uses
  (languages with extensions and configuration, grammars, snippets).
  The shared grammar rides generated plist XML (generated by
  clients/jetbrains/scripts/json2plist.py from the shared JSON). Files
  with no other file type fall to the
  TextMate plugin's own detector, so no custom FileType is needed.
  Snippets: the TextMate snippet registry is unit-test-only at 242, so
  the shared snippets ship as native live templates instead.
- Since 2024.2 the LSP API is public but the platform still wants
  JetBrains' blessing in practice: sinceBuild is 242 with no upper
  bound, build is IntelliJ Platform Gradle Plugin 2.18.1 (which needs
  Gradle 9.0+; CI pins 9.0.0 and temurin 21), and the plugin version
  rides the train like the vsix, pinning the server URL.
- One build-system trap cost the first staging run: IPG's
  bundledPlugin(...) helper could not see the TextMate plugin in the
  extracted 2024.2 IDE, because plugin-structure's findPluginById
  misses entries until the bundled plugin list has been materialized
  once, and IPG's first access is the lookup itself (reproduced
  locally against both IPG 2.5.0 and 2.18.1). The build sidesteps it:
  a prepareTextmateJar task picks plugins/textmate/lib/textmate.jar out
  of the resolved IDE artifact as a compileOnly file, and the declared
  plugin dependency in plugin.xml provides the classes at runtime.
