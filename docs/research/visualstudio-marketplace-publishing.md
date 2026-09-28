# Publishing the epher VS IDE extension to the Visual Studio Marketplace: portal upload, VsixPublisher, and what the top listings do

Research of 2026-09-28, in preparation for publishing the `clients/visualstudio` VSIX to
marketplace.visualstudio.com, where the VS Code extension already lives under publisher
`upyesp`. Four questions: how publishing works (portal and CLI), what the VSIX controls
versus what the upload controls, who the top VS IDE extensions are and how their listings
are built, and what epher's bundled-server situation means for the listing wording.

## Goal

Get `upyesp.Epher.VisualStudio` (VSIX, VS 2022 17.x, amd64) onto the official Visual
Studio Marketplace listing with: a decided publish path (portal vs VsixPublisher), the
right PAT scope, the listing assets the portal needs, and Requirements wording that
matches what the extension actually ships.

## Method (all claims verified)

Everything below was fetched directly, no blog posts:

- **The official MicrosoftDocs/visualstudio-docs markdown** via raw.githubusercontent.com:
  the portal walkthrough [1], the command line walkthrough (VsixPublisher.exe, publish
  manifest JSON) [2], and the VSIX schema 2.0 reference (field limits) [3]. Plus
  microsoft/vscode-docs `publishing-extension.md`, the only official PAT-scope guidance
  for this marketplace [4].
- **The live gallery API** `POST https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery`
  with `Accept: application/json;api-version=3.0-preview.1` [5]. Filter and sort enums
  were verified against Microsoft's own served website bundle (the vss-bundle-common JS
  on marketplace.visualstudio.com), which defines `ExtensionQueryFilterType`:
  `InstallationTarget=8`, `SearchText=10`, and sort `Installs=4` [6].
- **Item pages and Details assets** of the top five VS IDE extensions plus SonarQube for
  Visual Studio: `https://marketplace.visualstudio.com/items?itemName=...` and the
  overview markdown at `{publisher}.gallerycdn.vsassets.io/extensions/{pub}/{ext}/{version}/{ts}/Microsoft.VisualStudio.Services.Content.Details` [7]-[12].
- **This repo** (`clients/visualstudio/` README, manifest, csproj, client sources) [13]
  and **actions/runner-images**: the windows-2022 image software list [14].

## 1. Publishing mechanics

### The web portal flow

Sign in at marketplace.visualstudio.com, select "Publish extensions", which lands on the
manage page for your extensions; if you have no publisher, you are prompted to create one
[1]. Then: choose the publisher on the left, select "New extension", select "Visual
Studio", and in step "1: Upload extension" either upload a VSIX file or provide a link to
your own site [1]. Step "2: Provide extension details" is a form (fields below). "Save &
Upload" returns to the manage page with the extension not yet public; right-click it and
select "Make Public" to publish [1]. The manage page also offers "View Extension",
"Reports" (acquisition numbers), and "Edit" [1].

### The VsixPublisher CLI (ships with the VS SDK)

`VsixPublisher.exe` lives at `${VSInstallDir}\VSSDK\VisualStudioIntegration\Tools\Bin\`
[2]. Commands: `publish`, `deletePublisher`, `deleteExtension`, `login`, `logout`
(`createPublisher` is gone; publishers are created on the manage portal) [2].

```
VsixPublisher.exe login    -personalAccessToken "{PAT}" -publisherName "upyesp"
VsixPublisher.exe publish  -payload "{path to vsix}" -publishManifest "{path to vs-publish.json}" -personalAccessToken "{PAT}"
VsixPublisher.exe deleteExtension -extensionName "{name}" -publisherName "{pub}" -personalAccessToken "{PAT}"
```

`-personalAccessToken` on publish/deleteExtension is optional when that publisher is
already logged in on the machine (the `login` command caches it) [2]. For a VSIX payload
the manifest only needs `identity.internalName`; the rest of the identity (display name,
version, icon, description) is generated from the vsixmanifest [2]. Full VSIX sample [2]:

```json
{
  "$schema": "http://json.schemastore.org/vsix-publish",
  "categories": [ "build", "coding" ],
  "identity": { "internalName": "MyVsixExtension" },
  "overview": "overview.md",
  "priceCategory": "free",
  "publisher": "MyPublisherName",
  "private": false,
  "qna": true,
  "repo": "https://github.com/MyPublisherName/MyVsixExtension"
}
```

`categories` requires 1 to 3 values, `overview` (the listing markdown) is required,
`qna` defaults to true, `private` defaults to false [2]. Relative images referenced by
the overview markdown ride along via `assetFiles` entries (`pathOnDisk`, `targetPath`)
[2].

### Versions: overwrite, unpublish, rename

- **Same version can be re-uploaded.** `publish`: "If the extension already exists with
  the same version, it will overwrite the extension. If the extension does not already
  exist, it will create a new extension" [2]. The portal update flow does not state a
  must-bump rule either; the version simply comes from the new VSIX [1].
- **Unpublish**: on the manage page, extension menu, "Remove"; you must confirm by
  typing the extension name, and the action is explicitly non-reversible [1]. CLI
  equivalent: `deleteExtension` [2].
- **Rename: effectively impossible after first publish.** The Internal Name is the URL
  slug (`items?itemName=publisher.name`) and, with Display Name, Version, VSIX ID, Logo,
  Short description, supported VS versions and editions, is marked read-only for an
  extension update: "This detail can't be changed for an extension update" [1]. Editable
  on update: Overview, Type, Categories, Tags, Pricing Category, source repo, Q&A toggle
  [1].

### PAT: acquisition and scope

VsixPublisher takes a "Personal Access Token (PAT) that's used to authenticate the
publisher" and says nothing more specific [2]. The only official scope guidance for this
marketplace is in the VS Code publishing docs (same website, same publisher portal):
create the token in Azure DevOps with Organization "All accessible organizations" and
Scopes "Custom defined", then "Marketplace: Manage" [4]. Answer to the practical
question: both product lines are authenticated the same way, an Azure DevOps PAT against
the publisher account on marketplace.visualstudio.com/manage (publishers for VS Code and
VS IDE are created on that one portal [1][2][4]), and no Microsoft doc restricts a
marketplace PAT to one product line, so a token that publishes the VS Code extension
under `upyesp` meets VsixPublisher's only stated requirement. Nothing we fetched says
the tokens are separate; nothing we fetched says they are shared either, so treat
first-use as the test. Portal uploads bypass tokens entirely (browser sign-in) [1].

### Headless/CI usage

The tool is a Windows exe that ships with the VS SDK [2]. GitHub's windows-2022 hosted
runner ships VS Enterprise 2022 17.14 including the `Microsoft.VisualStudio.Component.VSSDK`
component [14], so CI can run it from `...\2022\Enterprise\VSSDK\VisualStudioIntegration\Tools\Bin\`.
Auth is one PAT secret (inline on `publish`, or `login` once first) [2]. This is plain
Azure DevOps PAT auth, a different mechanism from the Entra app permission that blocks
our VS Code CI automation; the denial there does not obviously transfer, but test
before assuming.

## 2. VSIX versus upload: what controls the listing

- **Identity** (Language, Id, Version, Publisher) and the other VSIX-controlled fields
  are autopopulated into the upload form [1]. Schema limits: `DisplayName` at most 50
  characters, `Description` at most 1000 characters [3].
- **Logo**, from the manifest if provided [1]. Schema elements: `<Icon>` (png/bmp/jpeg/
  ico; rendered at 32x32 in the IDE listview) and `<PreviewImage>` (200x200, details UI)
  [3]. Ours ships the icon as a `Microsoft.VisualStudio.VsIcon` asset instead, which the
  Extensions manager uses for the same purpose.
- **Short description**, from `<Description>` [1]. The schema also has `<MoreInfo>` (a
  URL), `<License>` (txt/rtf), `<ReleaseNotes>` (file or URL), `<GettingStartedGuide>`,
  `<Tags>` (semicolon-delimited, at most 100 characters) [3]; our manifest's `<Tags>` is
  comma-separated.

Everything else on the listing comes from the upload (portal form, or publish manifest
fields for the CLI) [1][2]: Overview (markdown), supported VS versions and editions,
Type, Categories (up to 3), Tags for search relevance, Pricing Category
(free/trial/paid), source code repository link, and the Q&A on/off switch (`qna`
defaults to true in the publish manifest) [2]. The Identity fields become the VSIX ID
and the URL slug (`items?itemName=publisher.name`) [1].

**README rendering**: the VSIX's README.md is NOT picked up. The listing body is the
separately uploaded Overview, "the 'readme' file that gets uploaded to the Marketplace"
[2]; on the portal it is a form field: "a good place to include screenshots and detailed
information" [1]. It renders as markdown with anchors (verified: SonarQube for Visual
Studio's item page embeds the rendered overview with `h1` anchors and live images) [12].

**Screenshots and GIFs**: they are markdown images inside the Overview, hosted anywhere
with absolute URLs. All visuals we found on VS IDE listings are external absolute URLs
(JetBrains serves its own; Sonar uses raw.githubusercontent.com wiki GIFs) [7][12]. The
website bundle still lists legacy marketplace-hosted screenshot asset names
(`Microsoft.VisualStudio.Services.Screenshots.1` to `.3`) [6], but every item we probed
top five, Sonar, and VS Code's vim returned 404 for them; nobody uses that channel
anymore. No count limit is documented in the sources we fetched.

**Q&A tab**: enabled by the portal checkbox ("lets users leave questions on your
extension entry page" [1]) and persisted as the item property
`Microsoft.VisualStudio.Services.EnableMarketplaceQnA` (True on four of the top five,
False on Sonar) [5][7]-[11]. **Review tab**: public star rating with review count on
the item page (linked at `#review-details`) and `averagerating`/`ratingcount` in the
API statistics [5]; the publisher manage portal additionally reports "Acquisition
Trend ... Total Acquisition counts and Ratings & Reviews" per extension [4].

**Icon sizes in practice**: live icons are 128x128 or 256x256 PNG or a multi-size ICO
(16 to 256) [7]-[11]. One top-five item ships no icon at all (empty `og:image`, 404 on
the icon asset) [11]. The schema's 32x32 note is about the IDE listview, not the
marketplace page [3].

## 3. Top 5 Visual Studio IDE extensions by installs

Query used: `extensionquery` with criteria `filterType 8, value "Microsoft.VisualStudio.Ide"`,
`sortBy: 4` (Installs), `sortOrder: 2`, flags 914, 800 items paged and deduplicated,
cross-checked against each item page [5][6]. Fetched 2026-09-28. The site displays a
larger install number than the API's `install` statistic for four of the five (the
site's tooltip says "installations, not including updates"; we report both rather than
reconcile). The top five set is the same under either number.

| # | extension | publisher | installs (item page) | installs (API stat) |
| --- | --- | --- | --- | --- |
| 1 | ReSharper | JetBrains.ReSharper | 3,042,375 | 2,283,043 |
| 2 | GitHub Extension for Visual Studio | GitHub.GitHubExtensionforVisualStudio | 2,769,043 | 2,212,070 |
| 3 | Microsoft Visual Studio Installer Projects | VisualStudioClient.MicrosoftVisualStudio2017InstallerProjects | 2,649,639 | 2,533,918 |
| 4 | SQL Server Integration Services Projects | SSIS.SqlServerIntegrationServicesProjects | 2,182,941 | 2,182,941 |
| 5 | Microsoft Reporting Services Projects | ProBITools.MicrosoftReportProjectsforVisualStudio | 1,690,026 | 1,567,275 |

Listing analysis (word counts from the served markdown):

| item | overview size | structure | tone | images | icon |
| --- | --- | --- | --- | --- | --- |
| ReSharper | 5,850 chars, ~667 words | 11 headings, marketing sections (Explore, Improve, Code, Maintain...) | brand prose | 9 images, 5 GIFs, all external | 256x256 PNG |
| GitHub Extension | 2,566 chars, ~371 words | intro paragraph, `## Features`, `## Requirements` | feature list | 0 | 128x128 PNG |
| Installer Projects 2017 | 5,418 chars, ~678 words | no headings, bold note + 34 bullets | functional prose | 0 | 128x128 PNG |
| SSIS Projects | 26,405 chars, ~3,504 words | one `# Release Notes` heading, 173 bullets | release-notes dump | 0 | multi-size ICO (16-256) |
| Reporting Services Projects | 4,599 chars, ~605 words | no headings, 19 bullets | release-notes dump | 0 | none (404, empty og:image) |

Traits shared by the top five:

- **Text-first listings.** Four of five carry zero visuals; only ReSharper is
  image-heavy, and every image is an absolute external URL. None uses
  marketplace-hosted screenshot assets [7]-[11].
- **Short descriptions are one or two sentences** naming the tool and its function
  (e.g. SSIS: "This project may be used for building high performance data integration
  and workflow solutions, including extraction, transformation, and loading (ETL)
  operations for data warehousing."). The overview does the selling [5].
- **Structure is functional prose plus bullets; changelogs are a popular overview
  genre** (two of five are pure release notes) [10][11]. Q&A is enabled on four of
  five; all have public ratings (averages 2.2 to 4.3, 99 to 381 ratings) [5][7]-[11].
- **All five are legacy, multi-target tools** (VS 2013 through 2022); none is a
  17.0-only amd64 extension like ours. Icon discipline is loose: 128 or 256 px PNG or
  ICO, and one item ships none [7]-[11].

Reference point just outside the top five: SonarQube for Visual Studio (#14 by our API
statistic, 847,834 installs) is the model modern listing: ~416 words, two animated GIFs
as absolute wiki URLs, and links out to Requirements and Installation docs [12].

## 4. epher's situation: the first-use download premise is stale

The research brief said the VS extension "downloads its language server (epher-lsp) on
first use (no VSIX-bundled exe)". The repo says otherwise, in three places [13]:

- `clients/visualstudio/README.md`: "the server ships inside the extension: the vsix
  carries `server\epher-lsp.exe` ... No first-use network step, no cache, no version
  marker".
- `Epher.VisualStudio.csproj`: `<Content Include="server\epher-lsp.exe">` ("bundled,
  the ADR-0066 amendment"), with the version stamped from `$(EpherVersion)`.
- `EpherLanguageClient.cs`: `BundledServerPath()` resolves
  `server\epher-lsp.exe` beside the extension; the comment says the downloader was
  retired for Visual Studio. If the binary is missing, an InfoBar reports it and editing
  falls back to the TextMate baseline.

So the listing's Requirements wording today is simply: VS 2022 17.x (amd64), server
bundled, no network step, plus the manifest prerequisites
(`Microsoft.VisualStudio.Component.CoreEditor`). Nothing to disclose beyond supported
editions. For the record, no top listing we analyzed uses a "downloads on first use"
phrase: the closest patterns are GitHub Extension's one-line `## Requirements`
("Visual Studio 2015 and above") [9] and Sonar's Requirements/Installation doc links
with a "download SonarQube for IDE in the Extension Marketplace" line [12]. If a
downloader ever comes back, that Requirements spot is where the disclosure belongs.

## What we must do

Agent-doable (repo work, no accounts):

1. Write `vs-publish.json` next to the VSIX build: `publisher: "upyesp"`,
   `identity.internalName: "Epher.VisualStudio"`, `overview: "overview.md"`,
   1-3 `categories`, `qna: true`, `priceCategory: "free"`, `repo` URL [2].
2. Derive `overview.md` from `clients/visualstudio/README.md`, keeping images as
   absolute GitHub raw URLs (the top listings' pattern; ReSharper's and Sonar's visuals
   are all external URLs) [7][12]; add a short Requirements section [9].
3. Fix the manifest `<Tags>` to semicolon-delimited and at most 100 characters [3];
   consider adding `<MoreInfo>`, `<License>`, `<ReleaseNotes>` (schema elements we do
   not use yet) [3]. The manifest `<Description>` is capped at 1000 characters [3]
   (ours is ~600, fine).
4. Wire CI: on windows-2022 (ships VS 2022 Enterprise + VSSDK [14]), run VsixPublisher
   `publish` with the VSIX payload and the PAT from a repository secret; the version
   already moves with `$(EpherVersion)` [13]. Same-version overwrite is allowed by the
   CLI [2].
5. Do NOT rename after first publish: the internal name, display name, version, VSIX ID,
   logo, short description, and supported versions/editions lock on update [1].

User clicks (manual, first time):

1. Create an Azure DevOps PAT: Organization "All accessible organizations", Custom
   defined scopes, Marketplace: Manage [4]. Note this is an Azure DevOps PAT, a
   different mechanism from the Entra app permission that blocks our VS Code CI
   automation; if your tenant blocks Azure DevOps PATs too, the fallback is portal
   uploads.
2. Either publish once through the portal (sign in, "Publish extensions", New extension,
   Visual Studio, upload the VSIX, fill categories/tags/pricing/Q&A/repo, Save & Upload,
   then "Make Public") [1], or run the CLI publish and use the portal only to press
   "Make Public" (the publish manifest's `"private": false` should make it public
   directly [2]).
3. Confirm the `upyesp` publisher (created for the VS Code extension) is offered on the
   manage page's publisher list; add a CI service user later via Members, by User ID,
   with the Contributor role [1].
4. If ever needed: unpublish via the extension menu's "Remove" with typed confirmation
   (non-reversible) [1].

## Sources

All fetched 2026-09-28.

1. Walkthrough: Publish a Visual Studio extension (portal flow, autopopulated and read-only fields, remove flow, publisher roles): https://github.com/MicrosoftDocs/visualstudio-docs/blob/main/docs/extensibility/walkthrough-publishing-a-visual-studio-extension.md
2. Walkthrough: Publishing via command line (VsixPublisher commands, publish manifest, asset files, overwrite rule): https://github.com/MicrosoftDocs/visualstudio-docs/blob/main/docs/extensibility/walkthrough-publishing-a-visual-studio-extension-via-command-line.md
3. VSIX extension schema 2.0 reference (field limits, Icon/PreviewImage/MoreInfo/Tags): https://github.com/MicrosoftDocs/visualstudio-docs/blob/main/docs/extensibility/vsix-extension-schema-2-0-reference.md
4. VS Code docs, Publishing Extension (PAT: All accessible organizations, Marketplace Manage; manage portal reports): https://github.com/microsoft/vscode-docs/blob/main/api/working-with-extensions/publishing-extension.md
5. Live gallery API, extensionquery (top-N query with filterType 8 "Microsoft.VisualStudio.Ide", sortBy 4; metadata, statistics, QnA property): https://marketplace.visualstudio.com/_apis/public/gallery/extensionquery
6. marketplace.visualstudio.com served vss-bundle-common JS (ExtensionQueryFilterType InstallationTarget=8/SearchText=10, sort Installs=4, legacy Screenshots.1-3 asset names)
7. ReSharper item page + Details asset (5,850 chars, 9 images/5 GIFs): https://marketplace.visualstudio.com/items?itemName=JetBrains.ReSharper
8. Installer Projects 2017 item page + Details asset: https://marketplace.visualstudio.com/items?itemName=VisualStudioClient.MicrosoftVisualStudio2017InstallerProjects
9. GitHub Extension for Visual Studio item page + Details asset: https://marketplace.visualstudio.com/items?itemName=GitHub.GitHubExtensionforVisualStudio
10. SSIS Projects item page + Details asset (26,405 chars): https://marketplace.visualstudio.com/items?itemName=SSIS.SqlServerIntegrationServicesProjects
11. Reporting Services Projects item page + Details asset (no icon asset): https://marketplace.visualstudio.com/items?itemName=ProBITools.MicrosoftReportProjectsforVisualStudio
12. SonarQube for Visual Studio item page + Details asset (847,834 installs, 2 GIFs, Requirements links): https://marketplace.visualstudio.com/items?itemName=SonarSource.SonarLintforVisualStudio2022
13. This repo: clients/visualstudio/README.md, source.extension.vsixmanifest, Epher.VisualStudio.csproj, EpherLanguageClient.cs, EpherRun.cs
14. actions/runner-images windows-2022 readme (VS Enterprise 2022 17.14, Microsoft.VisualStudio.Component.VSSDK): https://github.com/actions/runner-images/blob/main/images/windows/Windows2022-Readme.md
