# epher for the JetBrains IDEs

The epher calculator language in IntelliJ IDEA, PyCharm, WebStorm,
GoLand, CLion, PhpStorm, RubyMine, DataGrip, Rider, and RustRover:
syntax highlighting with bracket matching, a diagnostic for every
broken statement, the answer rendered inline next to every statement
that produces one, hover with canonical signatures, completion for the
catalog and your own names, and the shared snippets.

The plugin is a thin shell (ADR-0066). All understanding lives in
`epher-lsp`, the shared language server, which the plugin downloads on
first use and caches.

## Supported IDEs

2024.2 or newer of the commercial JetBrains IDEs: IntelliJ IDEA
Ultimate, PyCharm Professional, WebStorm, PhpStorm, RubyMine, CLion,
DataGrip, GoLand, Rider, and RustRover. The platform's built-in LSP
client ships only in the commercial IDEs, so IntelliJ IDEA Community
and Android Studio cannot load this plugin.

## Install from disk

1. Download `epher-jetbrains.zip` from the epher release page (do not
   unpack it).
2. Open Settings (Preferences on macOS), then Plugins.
3. Open the gear menu in the Plugins page and pick "Install Plugin
   from Disk".
4. Choose the zip and restart the IDE.

JetBrains Marketplace publication comes later; the website page and
the release assets are the distribution until then.

## First use

Opening a `.epher` file triggers a one-time download of the
`epher-lsp` build for your platform (about 4 to 6 MB):

- URL: `https://github.com/upyesp/epher/releases/download/v<plugin
  version>/epher-lsp-<target>`; a plugin update re-fetches a matching
  server.
- Targets: `linux-x86_64`, `linux-aarch64`, `macos-aarch64`
  (gzipped), `windows-x86_64` (zip with the exe at the archive root).
- Cache: the IDE's system directory, `epher/bin`, with a
  `server-version` marker, so the download happens once.

First activation needs the network once. If the download fails, the
error is reported cleanly and editing still works through the TextMate
baseline; only the live features are missing. Reopening the file (or
restarting the IDE) retries.

## The features

- **Highlighting**: the shared TextMate grammar (baseline spelling)
  plus semantic tokens from the server, so `i`, unit suffixes, and
  functions color by meaning.
- **Bracket matching**: from the grammar's language configuration.
- **Inline evaluation**: the server re-evaluates the whole file on a
  short debounce and the answers render above the statements that
  produce them.
- **Diagnostics**: parse and evaluation errors arrive as squiggles
  under the offending spans.
- **Completion**: the function catalog, your own names, and the
  server's snippet suggestions.
- **Hover**: canonical signatures and descriptions from the catalog.
- **Snippets**: the shared snippets ship as native live templates
  (type `def`, `for`, `solve`, and friends, then Tab).

## The shared assets

Everything the plugin knows about epher is a copy of the shared
sources; never edit the copies:

- `epher-bundle/syntaxes/epher.tmLanguage` is generated from
  `clients/shared/epher.tmLanguage.json` (the JetBrains TextMate
  plugin reads the plist form, the VS Code pilot reads the JSON form):

      python3 scripts/json2plist.py \
          clients/shared/epher.tmLanguage.json \
          src/main/resources/epher-bundle/syntaxes/epher.tmLanguage

- `epher-bundle/language-configuration.json` mirrors
  `clients/vscode/language-configuration.json` (comments, brackets,
  folding), packaged with the grammar as a TextMate bundle in VS Code
  layout.
- `liveTemplates/epher.xml` ports `clients/shared/epher-snippets.json`
  to IntelliJ live templates (the TextMate plugin's own snippet
  loading is internal at 2024.2, so the bundle ships none).

## Building

Requires a JDK 21 and Gradle 9.0.0 or newer (no wrapper is committed;
CI pins the Gradle version via `gradle/actions/setup-gradle`). The
build compiles against IntelliJ IDEA Ultimate 2024.2.4, because the
LSP API classes only exist in the commercial distributions, and it
downloads that IDE on first run, roughly 1.3 GB.

    gradle buildPlugin -PpluginVersion=0.5.40

The plugin zip lands in `build/distributions/`. CI packages it under
the stable name `epher-jetbrains.zip` for the release and the staging
draft.

## Status

Stage five of the ADR-0066 plan, after the VS Code pilot. Versions are
locked to the epher 0.5.x train: plugin 0.5.40 downloads the server
from the v0.5.40 release. Marketplace publication is deliberately out
of scope until the extensions are proven; the website page is the
distribution until then.
