# Visual Studio 2022 extension best practices for LSP-backed language support

Date: 2026-09-25 · Status: research notes, all claims traced to primary sources
(learn.microsoft.com Visual Studio extensibility docs, the Microsoft extensibility
GitHub repos, official NuGet package pages). Everything was fetched live on this
date; where a page carries its own `ms.date`, it is quoted. One archived page was
unreachable and is flagged as such; nothing in this note is filled in from memory.

Context: epher plans a VS extension that launches the standalone `epher-lsp`
binary over stdio (see ide-extensions-roadmap.md, decision #1, #14, #17).

## 1. The VSSDK language-server framework (Microsoft.VisualStudio.LanguageServer.Client)

**Still the supported way to back an editor language with a stdio LSP server in
VS 2022 17.x.** Support for the common LSP is built into Visual Studio itself
since VS 2017 15.8 ("Starting with Visual Studio 2017 version 15.8, support for
the common Language Server Protocol is built into Visual Studio"); the preview
Language Server Client VSIX was retired at that point
(https://learn.microsoft.com/en-us/visualstudio/extensibility/adding-an-lsp-extension?view=vs-2022,
`ms.date: 05/27/2026` — reviewed after the VS 17.14 era and still current). The
client side ships as the
[Microsoft.VisualStudio.LanguageServer.Client](https://www.nuget.org/packages/Microsoft.VisualStudio.LanguageServer.Client)
NuGet package; its latest stable version is **17.14.60** (published 2025-05-14,
following 17.13.33 / 2025-02-12 and 17.12.48 / 2024-11-12 — versions track the
VS 17.x train). The page explicitly scopes the framework: it exists "to onboard
language services that aren't part of Visual Studio product. It's not intended
to extend existing language services (like C#)" — exactly the epher case. The
same page documents the allowed transports: standard input/output streams,
named pipes, and TCP sockets.

Documented lifecycle (from the
[ILanguageClient API remarks](https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.languageserver.client.ilanguageclient?view=visualstudiosdk-2022)
and the how-to page):

1. Visual Studio calls `OnLoadedAsync()`.
2. The extension calls and awaits `InvokeAsync` on the `StartAsync` event
   (this may be deferred; the docs say "**To activate your language server, you
   must call StartAsync at some point.**").
3. Visual Studio calls and awaits `ActivateAsync(CancellationToken)`.
4. The extension starts the LSP server process and returns a `Connection`
   holding the server-side read/write streams; exceptions are surfaced as an
   InfoBar.
5. Visual Studio asynchronously performs the LSP `initialize` / `initialized`
   exchange.
6. Visual Studio calls `OnServerInitializedAsync()` (or
   `OnServerInitializeFailedAsync` on failure).

Full `ILanguageClient` member set (API reference above): properties `Name`,
`ConfigurationSections`, `InitializationOptions`, `FilesToWatch`,
`ShowNotificationOnInitializeFailed`; events `StartAsync`, `StopAsync`; methods
`ActivateAsync(CancellationToken)`, `OnLoadedAsync()`,
`OnServerInitializedAsync()`, `OnServerInitializeFailedAsync(Exception)`, and
`OnServerInitializeFailedAsync(ILanguageClientInitializationInfo)`. The class is
discovered via MEF: `[Export(typeof(ILanguageClient))]` plus
`[ContentType("...")]`, and the content type must derive from
`CodeRemoteContentDefinition.CodeRemoteContentTypeName`
([how-to page](https://learn.microsoft.com/en-us/visualstudio/extensibility/adding-an-lsp-extension?view=vs-2022)).
Activation is file-content-type driven only: "Currently, the only way to load
your LSP-based language server extension is by file content type." The framework
targets open-folder/open-file scenarios; with a custom project system "some
features, such as settings, might not work" (same page, FAQ).

**Custom messages and middle layers** come from `ILanguageClientCustomMessage2`
([API](https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.languageserver.client.ilanguageclientcustommessage2?view=visualstudiosdk-2022)):
properties `CustomMessageTarget` (object receiving non-LSP notifications/requests
from the server) and `MiddleLayer`; method
`AttachForCustomMessageAsync(JsonRpc)` (invoked after the server is started, before
the connection is established; keep the `JsonRpc` to send custom messages).
Messages are carried by VS-StreamJsonRpc
(https://github.com/microsoft/vs-streamjsonrpc).

`ILanguageClientMiddleLayer` exact member set
([API](https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.languageserver.client.ilanguageclientmiddlelayer?view=visualstudiosdk-2022);
signatures as shown in the how-to sample):

```csharp
public interface ILanguageClientMiddleLayer
{
    bool CanHandle(string methodName);
    Task HandleNotificationAsync(string methodName, JToken methodParam, Func<JToken, Task> sendNotification);
    Task<JToken> HandleRequestAsync(string methodName, JToken methodParam, Func<JToken, Task<JToken>> sendRequest);
}
```

Two caveats straight from the sources: the how-to says "The middle layer feature
is still under development and not yet comprehensive", and the API reference
marks the interface `[Obsolete("Will be removed in a future version.")]` in both
the visualstudiosdk-2019 and visualstudiosdk-2022 reference views (verified on
both pages) — while the current how-to still documents it as the interception
mechanism. There is no documented replacement; treat middle layers as usable but
at-risk API surface.

**Inlay hints are not handled by the built-in framework.** The how-to page's
"Language Server Protocol supported features" table is the authoritative list of
what the VS LSP client renders itself; it covers (rows marked "yes"): initialize,
initialized, shutdown, exit, `$/cancelRequest`, window/logMessage /
showMessage / showMessageRequest, `workspace/didChangeConfiguration`,
`workspace/didChangeWatchedFiles`, `workspace/symbol`, `workspace/executeCommand`,
`workspace/applyEdit`, `textDocument/publishDiagnostics`, didOpen/didChange/
didSave/didClose, completion + completion/resolve, hover, signatureHelp,
references, documentHighlight, documentSymbol, formatting, rangeFormatting,
definition, codeAction, rename. **The table ends at `textDocument/rename`;
`textDocument/inlayHint` has no row, and a case-insensitive search of the entire
MicrosoftDocs/visualstudio-docs checkout (docs/extensibility + docs/ide) finds
zero occurrences of "inlayhint".** Inlay hints are therefore left to the
extension: a server may send them, but the extension must intercept the messages
(via a middle layer or the custom-message channel) and render them itself; no
inlay-hint UI is documented in the VSSDK LSP framework. (Note that
`textDocument/semanticTokens` is likewise absent from the table — relevant to
roadmap decision #17, which already budgets for that.)

Also documented on the how-to page, useful for epher-lsp: settings via
`ConfigurationSections` + a pkgdef registering
`[$RootKey$\OpenFolder\Settings\VSWorkspaceSettings\...]`; wire-level tracing via
`"<section>.trace.server": "Verbose"` writing to `%temp%\VisualStudio\LSP`;
and the sample extension at
https://github.com/microsoft/VSSDK-Extensibility-Samples/tree/master/LanguageServerProtocol.

## 2. VisualStudio.Extensibility ("new extensibility") status for LSP extensions

The new model ships LSP support, but the model itself is still labelled preview.
The doc set is titled "VisualStudio.Extensibility documentation (Preview)" and
the overview states: "VisualStudio.Extensibility is in active development and is
available as a preview" and "The current VisualStudio.Extensibility preview
works with Visual Studio 2022 version 17.9 Preview 1 or higher"
(https://learn.microsoft.com/en-us/visualstudio/extensibility/visualstudio.extensibility/visualstudio-extensibility?view=vs-2022).

Language-server support is documented as a first-class area of the preview SDK:
"Extensibility Language Server Provider"
(https://learn.microsoft.com/en-us/visualstudio/extensibility/visualstudio.extensibility/language-server-provider/language-server-provider?view=vs-2022,
`ms.date: 10/23/2023`). The pattern: subclass
`Microsoft.VisualStudio.Extensibility.LanguageServer.LanguageServerProvider`
([API page](https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.extensibility.languageserver.languageserverprovider?view=visualstudiosdk-2022),
package Microsoft.VisualStudio.Extensibility v17.14.2088, page carries the
"relates to prerelease product" disclaimer), apply `[VisualStudioContribution]`,
override `LanguageServerProviderConfiguration` (display name +
`DocumentFilter.FromDocumentType(...)` over `LanguageServerBaseDocumentType` or
custom `DocumentTypeConfiguration`s), override
`CreateServerConnectionAsync` returning an `IDuplexPipe` to the server process,
and `OnServerInitializationResultAsync(ServerInitializationResult,
LanguageServerInitializationFailureInfo, ...)`; `Enabled` stops/starts the
server. The GitHub repo's 17.9 announcement (2024-02-20) lists "Leverage your
LSP servers in your extensions" as a shipped 17.9 feature
(https://github.com/microsoft/VSExtensibility/blob/main/docs/announcements.md),
and the RustLanguageServerProvider sample exists at
https://github.com/microsoft/VSExtensibility/tree/main/New_Extensibility_Model/Samples/RustLanguageServerProvider.

Packaging reality check: the latest Microsoft.VisualStudio.Extensibility on
NuGet is **17.14.2098** (2025-10-02), published as a stable (non-prerelease)
version (https://www.nuget.org/packages/Microsoft.VisualStudio.Extensibility/),
yet the docs and API reference still carry preview wording. A VSIX manifest can
declare `<ExtensionType>` = `VSSDK`, `VisualStudio.Extensibility`, or
`VSSDK+VisualStudio.Extensibility`
(https://learn.microsoft.com/en-us/visualstudio/extensibility/vsix-extension-schema-2-0-reference?view=vs-2022),
and the in-proc/hybrid doc describes mixing both models in one VSIX
(https://learn.microsoft.com/en-us/visualstudio/extensibility/visualstudio.extensibility/get-started/in-proc-extensions?view=vs-2022).
Net assessment for a language extension today: the classic VSSDK LSP framework
(section 1) is the model with non-preview status and the deepest documentation;
VisualStudio.Extensibility's `LanguageServerProvider` exists and is usable but
sits inside a framework Microsoft itself still calls a preview.

## 3. Command interception: responding to Debug.Start (F5) / Debug.StartWithoutDebugging (Ctrl+F5) from the active text view

The documented mechanism for view-scoped interception of arbitrary (including
shell/debug) commands is the **view command filter**:
`IVsTextViewCreationListener` + `IVsTextView.AddCommandFilter`. The current doc
that walks through it is
[Walkthrough: Use a shortcut key with an editor extension](https://learn.microsoft.com/en-us/visualstudio/extensibility/walkthrough-using-a-shortcut-key-with-an-editor-extension?view=vs-2022):
a class exporting `IVsTextViewCreationListener` receives
`TextViewCreated(IWpfTextView)`, obtains the view adapter via
`IVsEditorAdaptersFactoryService.GetViewAdapter(textView)`, and calls
`view.AddCommandFilter(commandFilter, out next)` where the filter implements
`IOleCommandTarget` (`QueryStatus`/`Exec`) and must forward everything it does
not handle to the returned `next` target.

Chain/order rules, from primary sources:

- [IVsTextView.AddCommandFilter remarks](https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.textmanager.interop.ivstextview.addcommandfilter?view=visualstudiosdk-2022)
  (from textmgr.idl): "The text view uses a chain architecture for command
  filters. Call AddCommandFilter to add a new command filter to the chain and to
  handle commands for the text view. The environment then returns a pointer to
  another command filter. Use this second command filter to handle all of the
  commands that you do not want to send to your command filter."
  `RemoveCommandFilter` detaches.
- [Command routing algorithm](https://learn.microsoft.com/en-us/visualstudio/extensibility/internals/command-routing-algorithm?view=vs-2022):
  commands are resolved innermost-to-outermost: add-ins → priority command
  targets (`IVsRegisterPriorityCommandTarget`) → context-menu target → toolbar
  targets → tool window → document window (only if the command carries the
  `RouteToDocs` flag) → current hierarchy → global owner VSPackage. At the
  global level "If the VSPackage has not been loaded already, it is not loaded
  when Visual Studio calls QueryStatus. The VSPackage is loaded only when the
  Exec method is called." A view command filter sits inside the text-view
  adapter's chain, i.e. at the document-window level of this routing.

The command IDs themselves: `Debug.Start` and `Debug.StartWithoutDebugging` are
in the standard VS 97 command set — `VSConstants.VSStd97CmdID.Start` = 295 and
`VSStd97CmdID.StartNoDebug` = 368
(https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.vsconstants.vsstd97cmdid?view=visualstudiosdk-2022),
with `GUID_VSStandardCommandSet97` = {5EFC7975-14BC-11CF-9B2B-00AA00573819}
(https://learn.microsoft.com/en-us/visualstudio/extensibility/ide-guids?view=vs-2022 —
docs page listing GUIDs; the GUID/ID lookup procedure is documented at
https://learn.microsoft.com/en-us/visualstudio/extensibility/internals/guids-and-ids-of-visual-studio-commands?view=vs-2022,
which points at the SDK's `SharedCmdDef.vsct` / `VsDbgCmdUsed.vsct`). No current
page specifically demonstrates intercepting `Debug.Start` from a view filter;
the documented building blocks are exactly those above. For editor-internal
commands the same walkthrough documents the modern alternative
(`ICommandHandler<T>` via `Microsoft.VisualStudio.Commanding`, VS 15.6+), but
that path only sees editor commands (e.g. TYPECHAR), not `Debug.Start`.

Note on a stale page: the old "Command routing in VSPackages" article now lives
only at
https://learn.microsoft.com/en-us/previous-versions/visualstudio/visual-studio-2017/extensibility/internals/command-routing-in-vspackages?view=vs-2017
(per the redirect from the vs-2022 URL, and per the "Related content" link in
making-commands-available). That archived page was **unreachable** during this
research (the previous-versions host served an anti-bot challenge instead of
content), so no claims here rely on it; the current routing algorithm page and
the AddCommandFilter remarks carry the needed rules.

## 4. AsyncPackage vs Package, load-time guidance, autoload, lazy commands

- **AsyncPackage** (since VS 2015) is the documented base for background
  loading: derive from `AsyncPackage`, mark
  `[PackageRegistration(UseManagedResourcesOnly = true, AllowsBackgroundLoading = true)]`,
  mark services `[ProvideService(..., IsAsyncQueryable = true)]`, override
  `InitializeAsync` (the sync `Initialize()` is sealed), and avoid RPCs
  (including `GetService`) inside `InitializeAsync` — marshal with
  `JoinableTaskFactory.SwitchToMainThreadAsync` instead
  (https://learn.microsoft.com/en-us/visualstudio/extensibility/how-to-use-asyncpackage-to-load-vspackages-in-the-background?view=vs-2022).
  If UI-context autoload is unavoidable, use
  `[ProvideAutoLoad(UIContextGuid, PackageAutoLoadFlags.BackgroundLoad)]`; the
  package loads synchronously anyway if a caller uses sync `GetService`.
- **Synchronous autoload is deprecated/blocked**: "Synchronously autoloaded
  extensions have a negative impact on the performance of Visual Studio and
  should be converted to use asynchronous autoload instead. By default, Visual
  Studio 2019 blocks synchronously autoloaded packages from any extension and
  notifies the user" — with a Performance Manager dialog and a group-policy
  escape hatch
  (https://learn.microsoft.com/en-us/visualstudio/extensibility/synchronously-autoloaded-extensions?view=vs-2022).
  Microsoft's migration guide for authors is the AsyncPackageMigration sample:
  https://github.com/Microsoft/VSSDK-Extensibility-Samples/tree/master/AsyncPackageMigration.
- **Delayed loading is the documented default philosophy**: "VSPackages are
  loaded into Visual Studio only when their functionality is required... This
  feature is called delayed loading, which is used whenever possible"
  (https://learn.microsoft.com/en-us/visualstudio/extensibility/loading-vspackages?view=vs-2022);
  `ProvideAutoLoad` exists for UI-context autoload but is the thing to avoid in
  favour of command/UI activation.
- **Lazy command activation**: commands declared in the .vsct (registered via
  `ProvideMenuResourceAttribute`) are visible without loading the package; the
  package loads only when a user executes the command
  (https://learn.microsoft.com/en-us/visualstudio/extensibility/internals/making-commands-available?view=vs-2022
  and
  https://learn.microsoft.com/en-us/visualstudio/extensibility/internals/command-design?view=vs-2022).
  Visibility can be toggled **without loading the package** using the
  `DefaultDisabled` + `DefaultInvisible` + `DynamicVisibility` command flags and
  `VisibilityConstraints`/`VisibilityItem` entries tied to UI-context GUIDs
  (same making-commands-available page). Runtime state changes (text changes,
  visible/checked) go through `OleMenuCommand.BeforeQueryStatus`, where
  `Visible`, `Enabled`, `Checked` may be set
  (https://learn.microsoft.com/en-us/visualstudio/extensibility/changing-the-text-of-a-menu-command?view=vs-2022).
  There is a dedicated performance diagnostics doc
  (https://learn.microsoft.com/en-us/visualstudio/extensibility/how-to-diagnose-extension-performance?view=vs-2022).

## 5. VSIX packaging (2026)

- **Format**: a .vsix follows the Open Packaging Conventions (OPC) — a ZIP
  container holding the payload binaries/files, `extension.vsixmanifest`, and
  `[Content_Types].xml`; file names inside the package "must not include spaces,
  nor characters that are reserved in Uniform Resource Identifiers". Per-user
  install target is `%LocalAppData%\Microsoft\VisualStudio\{version}\Extensions`;
  `AllUsers=true` installs under `Common7\IDE\Extensions`
  (https://learn.microsoft.com/en-us/visualstudio/extensibility/anatomy-of-a-vsix-package?view=vs-2022).
- **Manifest assets**: the VSIX schema 2.0 reference lists the known asset
  `Type` values as `Microsoft.VisualStudio.VsPackage`,
  `Microsoft.VisualStudio.MefComponent`, `Microsoft.VisualStudio.ToolboxControl`,
  `Microsoft.VisualStudio.Samples`, `Microsoft.VisualStudio.ProjectTemplate`,
  `Microsoft.VisualStudio.ItemTemplate`, `Microsoft.VisualStudio.Assembly`;
  custom types are allowed, and each `<Asset>` carries a `Path` (plus optional
  `TargetVersion` for per-VS-version assets)
  (https://learn.microsoft.com/en-us/visualstudio/extensibility/vsix-extension-schema-2-0-reference?view=vs-2022).
  **`Microsoft.VisualStudio.VsIcon` is not in that documented list** — a
  repo-wide search of MicrosoftDocs/visualstudio-docs (main, 2026-09-25) finds no
  current documentation of a VsIcon asset type; do not rely on it.
- **`ExtensionType` and `ProductArchitecture`** (VS 2022+ schema features):
  `<ExtensionType>` ∈ {`VSSDK`, `VisualStudio.Extensibility`,
  `VSSDK+VisualStudio.Extensibility`}; `<ProductArchitecture>` (child of
  `InstallationTarget`) ∈ {`amd64`, `arm64`}; installation target e.g.
  `<InstallationTarget Id="Microsoft.VisualStudio.Community" Version="[17.0,18.0)">`
  (schema reference above).
- **Shipping a native exe inside the vsix is documented as supported** — as
  plain content. The LSP how-to states: "the extensions created to support
  LSP-based language servers in Visual Studio don't contain the language servers
  themselves or the runtimes... Extension developers are responsible for
  distributing the language servers and the runtimes... Language servers can be
  embedded in the VSIX as content files" (alternatives: an MSI, or download
  instructions). Mechanically: files marked Build Action = **Content** +
  Include in VSIX = True land in the package folder
  (adding-an-lsp-extension page, and the same pattern for pkgdef/grammar files
  in the language-configuration page). The official sample launches the server
  from a **`Server` subfolder** of the extension install directory:
  `Path.Combine(Path.GetDirectoryName(Assembly.GetExecutingAssembly().Location),
  "Server", @"MockLanguageServer.exe")` (sample code on the how-to page).
  **No documented size limits or mandated folder layout exist** — the docs only
  show conventions ("Grammars" for TextMate files, `Server` in the sample);
  the only packaging constraint is the OPC/URI-safe file-name rule above.
- **VSIX Installer version rules**: schema 2.0 requires VS 2012+; "You can
  target earlier versions of Visual Studio with a Visual Studio 2012 or later
  VSIXInstaller, but only by using the later versions of the installer"
  (schema reference above). For VS 2022: "the VSIX installer in Visual Studio
  2022 has been updated. Developers need to use the version of the VSIX
  installer that comes with Visual Studio 2022 to install extensions to that
  version of Visual Studio", and the VS 2022 installer also installs extensions
  targeting previous VS versions when they sit side by side
  (https://learn.microsoft.com/en-us/visualstudio/extensibility/migration/update-visual-studio-extension?view=vs-2022,
  `ms.date: 11/5/2025`; the same page documents that VS 2022 will not load
  extension DLLs compiled against earlier VS SDKs, that managed extensions must
  be AnyCPU/x64, that 17.x SDK assemblies come only from NuGet
  ([Microsoft.VisualStudio.Sdk](https://www.nuget.org/packages/Microsoft.VisualStudio.Sdk/)),
  and that VS 2026 installs VS 2022-targeted extensions as-is).
  Signing: https://learn.microsoft.com/en-us/visualstudio/extensibility/signing-vsix-packages?view=vs-2022.

## 6. TextMate grammars: still the supported baseline highlighting path

Yes — documented as current on both the IDE and extensibility sides:

- Visual Studio itself provides "syntax colorization and basic statement
  completion" for ~37 languages via TextMate grammars, and users can drop
  additional grammars under `%userprofile%\.vs\Extensions\<lang>\{Syntaxes,Snippets}`
  (https://learn.microsoft.com/en-us/visualstudio/ide/adding-visual-studio-editor-support-for-other-languages?view=vs-2022,
  `ms.date: 5/02/2025`). The same page links the official sample "Ship TextMate
  grammars inside Visual Studio extensions":
  https://github.com/microsoft/VSSDK-Extensibility-Samples/tree/master/TextmateGrammar.
- The LSP how-to is explicit that highlighting is out of LSP scope in VS: "The
  LSP doesn't include specification on how to provide text colorization for
  languages. To provide custom colorization for languages in Visual Studio,
  extension developers can use a TextMate grammar file": create a folder in the
  extension, add `*.tmlanguage` / `*.plist` / `*.tmtheme` / `*.json`, and
  register it with a pkgdef line `[$RootKey$\TextMate\Repositories]
  "MyLang"="$PackageFolder$\Grammars"` (Build Action = Content, Include in
  VSIX = True). Grammars registered this way "supersede the built-in grammars"
  (https://learn.microsoft.com/en-us/visualstudio/extensibility/adding-an-lsp-extension?view=vs-2022).
- **Language Configuration** layers declarative editing behavior on top
  (comment toggling, bracket matching/auto-closing, auto-indent, snippets) so
  the editor handles these synchronously without an LSP round-trip; when no
  language service exists it "falls back to the TextMate grammar". The pkgdef
  keys are
  `[$RootKey$\TextMate\LanguageConfiguration\GrammarMapping]` (grammar
  scope-name → language-configuration.json) and
  `[$RootKey$\TextMate\LanguageConfiguration\ContentTypeMapping]` (content-type
  name → language-configuration.json), plus `TextMate\Repositories` for the
  grammar folder
  (https://learn.microsoft.com/en-us/visualstudio/extensibility/language-configuration?view=vs-2022,
  `ms.date: 11/06/2025`; sample:
  https://github.com/microsoft/VSExtensibility/tree/main/LSP/Samples/Language%20Configuration%20Setup%20Example).
  The matching registry-key constants live in `Microsoft.VisualStudio.Editor`
  (`CommonEditorConstants.TextMateRepositoryKey`,
  `CommonEditorConstants.TextMateLanguageConfigurationContentTypeMappingKey`,
  `TextMateConstants.RepositoryKey`;
  https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.editor.commoneditorconstants?view=visualstudiosdk-2022 —
  note the API pages document the members but not their literal string values).
- One correction to the premise in the question: no type named
  `IAssetTaggerProvider` appears anywhere in the current docs (zero hits in the
  MicrosoftDocs/visualstudio-docs checkout); the documented grammar integration
  points are the pkgdef registry keys and the constants above, not a tagged
  asset-provider API.

## Unreachable / not documented (explicit)

- https://learn.microsoft.com/en-us/previous-versions/visualstudio/visual-studio-2017/extensibility/internals/command-routing-in-vspackages?view=vs-2017 —
  served an anti-bot challenge, body unreadable on 2026-09-25. Superseded for
  current guidance by the command-routing-algorithm page and the
  AddCommandFilter API remarks (section 3).
- `Microsoft.VisualStudio.VsIcon` as a VSIX asset type — not present in current
  primary documentation (schema reference lists seven known types; repo-wide
  search negative).
- No learn/NuGet page states a size limit for VSIX payloads, nor a required
  subfolder name for bundled native executables.

## Sources

- https://learn.microsoft.com/en-us/visualstudio/extensibility/language-server-protocol?view=visualstudio (overview; `ms.date: 11/14/2017`)
- https://learn.microsoft.com/en-us/visualstudio/extensibility/adding-an-lsp-extension?view=vs-2022 (`ms.date: 05/27/2026`)
- https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.languageserver.client.ilanguageclient?view=visualstudiosdk-2022
- https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.languageserver.client.ilanguageclientcustommessage2?view=visualstudiosdk-2022
- https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.languageserver.client.ilanguageclientmiddlelayer?view=visualstudiosdk-2022 (also checked with `view=visualstudiosdk-2019&preserve-view=true`)
- https://www.nuget.org/packages/Microsoft.VisualStudio.LanguageServer.Client
- https://learn.microsoft.com/en-us/visualstudio/extensibility/visualstudio.extensibility/visualstudio-extensibility?view=vs-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/visualstudio.extensibility/language-server-provider/language-server-provider?view=vs-2022 (`ms.date: 10/23/2023`)
- https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.extensibility.languageserver.languageserverprovider?view=visualstudiosdk-2022
- https://www.nuget.org/packages/Microsoft.VisualStudio.Extensibility
- https://github.com/microsoft/VSExtensibility (README, fetched via raw.githubusercontent.com/microsoft/VSExtensibility/main/README.md)
- https://github.com/microsoft/VSExtensibility/blob/main/docs/announcements.md
- https://github.com/microsoft/VSExtensibility/tree/main/New_Extensibility_Model/Samples/RustLanguageServerProvider
- https://github.com/microsoft/VSExtensibility/tree/main/LSP/Samples/Language%20Configuration%20Setup%20Example
- https://learn.microsoft.com/en-us/visualstudio/extensibility/walkthrough-using-a-shortcut-key-with-an-editor-extension?view=vs-2022
- https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.textmanager.interop.ivstextview.addcommandfilter?view=visualstudiosdk-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/internals/command-routing-algorithm?view=vs-2022
- https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.vsconstants.vsstd97cmdid?view=visualstudiosdk-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/internals/guids-and-ids-of-visual-studio-commands?view=vs-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/how-to-use-asyncpackage-to-load-vspackages-in-the-background?view=vs-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/synchronously-autoloaded-extensions?view=vs-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/loading-vspackages?view=vs-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/internals/making-commands-available?view=vs-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/internals/command-design?view=vs-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/changing-the-text-of-a-menu-command?view=vs-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/how-to-diagnose-extension-performance?view=vs-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/anatomy-of-a-vsix-package?view=vs-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/vsix-extension-schema-2-0-reference?view=vs-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/migration/update-visual-studio-extension?view=vs-2022 (`ms.date: 11/5/2025`)
- https://learn.microsoft.com/en-us/visualstudio/extensibility/signing-vsix-packages?view=vs-2022
- https://learn.microsoft.com/en-us/visualstudio/ide/adding-visual-studio-editor-support-for-other-languages?view=vs-2022 (`ms.date: 5/02/2025`)
- https://learn.microsoft.com/en-us/visualstudio/extensibility/language-configuration?view=vs-2022 (`ms.date: 11/06/2025`)
- https://learn.microsoft.com/en-us/dotnet/api/microsoft.visualstudio.editor.commoneditorconstants?view=visualstudiosdk-2022
- https://github.com/microsoft/VSSDK-Extensibility-Samples/tree/master/LanguageServerProtocol
- https://github.com/microsoft/VSSDK-Extensibility-Samples/tree/master/TextmateGrammar
- https://github.com/Microsoft/VSSDK-Extensibility-Samples/tree/master/AsyncPackageMigration
- https://learn.microsoft.com/en-us/visualstudio/extensibility/visualstudio.extensibility/get-started/in-proc-extensions?view=vs-2022
- https://learn.microsoft.com/en-us/visualstudio/extensibility/visualstudio.extensibility/extensibility-models?view=vs-2022
- https://github.com/microsoft/VSExtensibility/blob/main/docs/lsp/lsp-extensions-specifications.md (VS-specific LSP extensions: VSDiagnostic, GetProjectContexts, VSServerCapabilities; implementation on NuGet Microsoft.VisualStudio.LanguageServer.Protocol)
