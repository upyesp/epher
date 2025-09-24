# Microsoft Store publishing for the Windows builds: process, assets, listings, CI

Research of 2026-09-19. Goal: publish the Windows versions of epher (NSIS today:
console `epher.exe` + GUI-subsystem `epher-gui.exe`, eight UI languages, offline)
to the Microsoft Store under an ordinary personal MSA (upyesp@gmail.com, the same
account as the VS Code Marketplace publisher), so the Store listing exists and
winget's `msstore` source can install it. Companion to
[store-ci-cd-publishing.md](store-ci-cd-publishing.md) §7 (which already priced
the submission REST API); this doc adds the account, packaging, asset and
listing detail that a human submission needs.

## Method

Fetched the primary sources on 2026-09-19: the learn.microsoft.com publishing
docs (many pages updated 2026-05…2026-08), the MSIX manifest schema pages, the
MSIX packaging docs' GitHub source (MicrosoftDocs/msix-docs), the Tauri v2 docs'
Microsoft Store page and its source (tauri-apps/tauri-docs, branch `v2`), the
winget docs plus winget-cli source, and the Microsoft Store Policies. For
listing anatomy, queried the first-party DisplayCatalog API
(`https://displaycatalog.mp.microsoft.com/v7.0/products?bigIds=<ID>&market=US&languages=en-us&MS-CV=x`,
no auth) for six popular apps whose product IDs were resolved from first-party
pages (see §2 — three IDs from the research brief turned out to be wrong or
dead and are called out below). Every load-bearing claim carries its URL;
where a number could not be verified, the text says so.

---

## 1. The process, end to end

### 1.1 Developer account: free, individual, single MSA

The current onboarding (docs page last updated 2026-05-07) is the first thing
to know because it changed: **"With the new onboarding experience, there are no
registration fees for either account type"**
([open a developer account](https://learn.microsoft.com/en-us/windows/apps/publish/partner-center/open-a-developer-account)).
The FAQ on the same page answers "Do I need to pay the registration fee?" with
"No — if you're using the new flow via the Store marketing page", and the only
supported entry point is
[storedeveloper.microsoft.com](https://storedeveloper.microsoft.com) — starting
from Partner Center directly shows the legacy flow, whose fee ("$19 individual /
$99 company" in older docs and countless posts) **could not be verified against
any current primary source**; treat any fee as legacy-path only.

For epher the flow is the *individual* one, and it is exactly the single-MSA
path the brief assumed:

1. Go to storedeveloper.microsoft.com → "Get started for free" → *Individual
   developer* (individual accounts must use a personal MSA; Entra work accounts
   are company-only).
2. Sign in with the MSA (upyesp@gmail.com).
3. **Identity verification with a government-issued ID and a selfie**, captured
   on mobile ("Begin identity verification with a government-issued ID and
   selfie" — same page). This is the one human-identity step; no DUNS, no
   business documents, no tax forms.
4. Complete profile → "Go to Partner Center dashboard" → the Apps & Games
   overview appears (can take ~5–30 min to light up).

Individual vs company: individual = publishing under your own name, not in
relation to a business; company = legal business entity, verified via DUNS or
documents, with manual review of 2–5 business days. "Changing a developer
account from Individual to Company is not supported" (same page) — fine, epher
stays individual.

**Payout and tax are skippable for a free app**, explicitly:
"If you only plan to list free offers, you don't need to fill out any tax forms
or set up a payout profile. If you change your mind … you can fill out tax
forms and set up a payout account then"
([set up your payout account](https://learn.microsoft.com/en-us/partner-center/account-settings/set-up-your-payout-account)).

### 1.2 Name reservation and the package identity

1. Partner Center → Apps and Games → **New product → MSIX or PWA app** → enter
   name → Check availability → Reserve
   ([reserve your app's name](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/reserve-your-apps-name)).
   Reserved names not used within **three months** lose the reservation; you
   may reserve several names and pick later.
2. The reservation mints the app's identity, visible in Partner Center under
   **Product management → Product identity**
   ([view product identity details](https://learn.microsoft.com/en-us/windows/apps/publish/view-app-identity-details)).
   Three of those values must appear verbatim in the MSIX manifest:
   `Package/Identity/Name`, `Package/Identity/Publisher`,
   `Package/Properties/PublisherDisplayName`. The same page shows the **PFN**
   (package family name), the **Store ID** (the `9…`/`XP…` product ID used by
   winget and deep links), and the listing URL
   `https://apps.microsoft.com/detail/<Store ID>`.
3. Manifest rules: `Identity@Name` is **case-sensitive**; `Publisher` must
   match the Partner Center string exactly ("Values in the manifest are
   case-sensitive. Spaces and other punctuation must also match"
   — [app package requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements));
   `Version` is `major.minor.build.revision` with the **fourth section
   reserved for the Store and left as 0**, sections ≤ 65535, first ≠ 0
   ([Identity schema](https://learn.microsoft.com/en-us/uwp/schemas/appxpackage/uapmanifestschema/element-identity),
   [package version numbering](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements#package-version-numbering)).
   For Store submissions the identity is *not* the Tauri `identifier`
   (`com.epher.desktop`); that value is irrelevant here — Partner Center
   assigns its own Name/Publisher pair, copied verbatim into the AppxManifest.

Name choice note: the Store name must be unique Store-wide; "epher" is short
enough that a collision is plausible — check availability before planning
around it (the same page covers what to do if the name is taken but unused).

### 1.3 Packaging: the Tauri reality, and the MSIX route chosen

**Tauri's official Microsoft Store guide describes only the EXE/MSI route.**
The v2 docs
([tauri.app/distribute/microsoft-store](https://tauri.app/distribute/microsoft-store/),
source [microsoft-store.mdx](https://github.com/tauri-apps/tauri-docs/blob/v2/src/content/docs/distribute/microsoft-store.mdx))
say: "Currently Tauri only generates EXE and MSI installers, so you must create
a Microsoft Store application that only links to the unpacked application",
require the offline-Installer Webview2 mode
(`webviewInstallMode: { "type": "offlineInstaller" }` in a
`tauri.microsoftstore.conf.json` merged at bundle time), and note the Store
requires **silent installation** (NSIS: `/S`, uppercase S, entered as the
installer's silent flag) and a publisher value that differs from the product
name.

That route is Store policy **10.2.9**, and its requirements are steep
([Store Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies)):
the linked installer binary "may only be an .msi or .exe", "the binary and all
of its Portable Executable (PE) files must be digitally signed with a code
signing certificate that chains up to a certificate issued by a Certificate
Authority (CA) that is part of the Microsoft Trusted Root Program", it must be
a standalone (not web) installer that installs silently (UAC allowed), the
download URL must be versioned and never change under a given version, and the
product "may only be made available to PC devices". Updates on this route are
the developer's problem: the Tauri guide says the linked installer "must be
offline, handle auto-updates and be code signed".

**The MSIX route is the one that fits epher**, for three verified reasons:

1. **No code-signing certificate**: "Your MSIX and AppX packages don't have to
   be signed with a certificate rooted in a trusted certificate authority when
   submitting to the Microsoft Store. The Microsoft Store will automatically
   re-sign your MSIX/AppX packages with a Microsoft certificate during the
   publishing process" — and, conversely, "If you are submitting an MSI or EXE
   installer to the Store, the Store does not re-sign those files. You must
   Authenticode-sign your MSI/EXE installer yourself"
   ([app package requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements)).
   epher ships unsigned today; MSIX is the only Store route that accepts that.
2. **Store-managed updates**: on the MSIX route the Store distributes and
   updates the package — "Existing users will receive the update via the Store
   automatically"
   ([publish update](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/publish-update-to-your-app-on-store)),
   with optional gradual rollout and mandatory-update scheduling in Partner
   Center ([upload MSIX app packages](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/upload-app-packages)).
3. **No NSIS-specific policy surface**: no silent-flag, no versioned-URL, no
   PE-signing obligations; the payload is just files in a package.

Packaging routes for producing that MSIX (Tauri has no MSIX bundler — the
bundler emits only `msi` and `nsis` on Windows, see the earlier research in
[store-ci-cd-publishing.md](store-ci-cd-publishing.md) §7):

| Route | Tool | CI-fit |
|---|---|---|
| Manual: hand-written `AppxManifest.xml` + payload, packed with **MakeAppx.exe** | Windows SDK (`MakeAppx.exe pack /d <dir> /p <out>.msix`), documented for exactly the no-Visual-Studio case: "If you don't use Visual Studio to create your package, you must create your package manifest manually" ([app package requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements), [create-app-package-with-makeappx-tool](https://learn.microsoft.com/en-us/windows/msix/package/create-app-package-with-makeappx-tool), [package from the command line](https://learn.microsoft.com/en-us/windows/msix/package/manual-packaging-root)) | **best** — a GitHub Actions windows runner already builds the app; adding a manifest dir + one MakeAppx call is a small step; unsigned output is accepted |
| **MSIX Packaging Tool** repack of the NSIS installer | interactive GUI; has a CLI (`MsixPackagingTool.exe create-package --template …`) that needs a clean conversion VM/remote machine ([command line](https://learn.microsoft.com/en-us/windows/msix/packaging-tool/package-conversion-command-line)) | poor — conversion-environment plumbing on a runner, and repackaging an installer is the wrong shape when we own the source |
| **msix-packaging** (open source) | microsoft/msix-packaging — cross-platform lib/CLI for pack/unpack ([github.com/microsoft/msix-packaging](https://github.com/microsoft/msix-packaging)) | niche; MakeAppx on a Windows runner is simpler and is what the Store docs themselves assume |

**Minimal working AppxManifest shape** for a Win32 `runFullTrust` package with
two executables and PATH aliases (sketch — values from the Product identity
page, verified attribute semantics from the schema pages
[Identity](https://learn.microsoft.com/en-us/uwp/schemas/appxpackage/uapmanifestschema/element-identity),
[uap5:AppExecutionAlias](https://learn.microsoft.com/en-us/uwp/schemas/appxpackage/uapmanifestschema/element-uap5-appexecutionalias),
[desktop extensions / runFullTrust](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/desktop-to-uwp-extensions)):

```xml
<?xml version="1.0" encoding="utf-8"?>
<Package xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
         xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
         xmlns:uap5="http://schemas.microsoft.com/appx/manifest/uap/windows10/5"
         xmlns:uap10="http://schemas.microsoft.com/appx/manifest/uap/windows10/10"
         xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities">
  <Identity Name="…from Partner Center" Publisher="CN=…from Partner Center"
            Version="0.5.42.0" ProcessorArchitecture="x64"/>
  <Properties>
    <DisplayName>epher</DisplayName>
    <PublisherDisplayName>…from Partner Center</PublisherDisplayName>
    <Logo>assets\StoreLogo.png</Logo>
  </Properties>
  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.26100.0"/>
  </Dependencies>
  <Resources>
    <!-- one Resource Language per UI language: en, de, … -->
    <Resource Language="en"/>
  </Resources>
  <Applications>
    <!-- GUI: the Start-menu entry, windows subsystem -->
    <Application Id="EpherGui" Executable="epher-gui.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements DisplayName="epher" Description="scriptable graphing calculator"
        BackgroundColor="transparent" Square150x150Logo="assets\150.png" Square44x44Logo="assets\44.png"/>
      <Extensions>
        <uap5:Extension Category="windows.appExecutionAlias">
          <uap5:AppExecutionAlias uap10:Subsystem="windows">
            <uap5:ExecutionAlias Alias="epher-gui.exe"/>
          </uap5:AppExecutionAlias>
        </uap5:Extension>
      </Extensions>
    </Application>
    <!-- console: no Start tile needed, but an alias so `epher` works on PATH -->
    <Application Id="EpherCli" Executable="epher.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements DisplayName="epher console" Description="epher CLI"
        BackgroundColor="transparent" Square150x150Logo="assets\150.png" Square44x44Logo="assets\44.png"
        AppListEntry="none"/>
      <Extensions>
        <uap5:Extension Category="windows.appExecutionAlias">
          <uap5:AppExecutionAlias uap10:Subsystem="console">
            <uap5:ExecutionAlias Alias="epher.exe"/>
          </uap5:AppExecutionAlias>
        </uap5:Extension>
      </Extensions>
    </Application>
  </Applications>
  <Capabilities>
    <rescap:Capability Name="runFullTrust"/>
  </Capabilities>
</Package>
```

Design notes for epher specifically: the AppExecutionAlias is what puts `epher`
on PATH for Store installs — it lands in
`%LOCALAPPDATA%\Microsoft\WindowsApps` and replaces the NSIS hook that edits
the user PATH (each `Application` may carry up to 1000 aliases, per the
`uap5:ExecutionAlias{0,1000}` range in the schema). `AppListEntry="none"` keeps
the console build out of the Start menu. The NSIS layout already Files
`epher.exe` next to `epher-gui.exe`, so the payload directory from
`cargo tauri build` output carries both binaries unchanged. Run WACK (Windows
App Certification Kit) on the packed MSIX before submitting — the docs ask for
it repeatedly ([app package requirements](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-package-requirements),
[avoid common certification failures](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/resolve-submission-errors)).

Upload format: the Packages page accepts .msix / .msixupload / .msixbundle /
.appx / .appxbundle ([upload MSIX app packages](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/upload-app-packages));
a single-architecture unsigned .msix is the simplest thing epher can submit.
Package cap: 25 GB (irrelevant here, but the actual cap).

### 1.4 Visual assets: what the submission requires vs optional

From [app screenshots, images, and trailers](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images) (page last updated 2025-06-06) — these are the numbers a human crops to:

| Asset | Required? | Exact size | Notes |
|---|---|---|---|
| Screenshot (Desktop) | **required — exactly one minimum** (any device family) | **1366 × 768 or larger**, landscape or portrait; 4K up to 3840 × 2160 supported | PNG, ≤ 50 MB each; **up to 10 desktop screenshots** (8 for other device families); Microsoft recommends **5–8** per device family; caption per screenshot **≤ 200 chars**; keep critical content in the top two-thirds (text overlays land on the bottom third); no logos/marketing text added |
| Store logo — 1:1 app tile icon | strongly recommended; "If you don't provide this image, the Store will use the image from your app package" (and the uploaded one wins) | **300 × 300** | PNG; this is what epher should upload — the existing dark-tile mark scaled |
| 2:3 poster art | apps: optional (games: required) | **720 × 1080** or **1440 × 2160** | may include the app name |
| 1:1 box art | apps: optional (games: main logo fallback) | **1080 × 1080** or **2160 × 2160** | |
| 16:9 "Super hero" art | optional (required only to show trailers at the top) | **1920 × 1080** or **3840 × 2160** | no text in the image |
| Trailer | optional | 1920 × 1080 video (MP4/MOV, ≤ 2 GB, ≤ 15 trailers) + PNG thumbnail 1920 × 1080 + title ≤ 255 chars | skip for v1 |
| Xbox / Holographic sets | only if publishing to those device families | (584 × 800, 1920 × 1080, 1080 × 1080; 2400 × 1200) | epher: Windows.Desktop only, skip |

The in-package icon set (what `tauri icon` generates and the VisualElements
entries reference) renders as 44/66/71/88/107/142/150/225/300/310/465/620 px
tiles plus 310 × 150 / 620 × 300 wide — visible empirically in the
DisplayCatalog Tile data of published apps (§2). Submission-side only the
300 × 300 Store tile is new art; everything else exists in
`crates/tauri-app/src-tauri/icons`.

Multiple-language listings: each language gets its own listing page, and
images must be uploaded per language ("even if you are using the same images")
([screenshots page](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/screenshots-and-images),
[store listing info](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info)).
For v1: list in English only, or reuse the same screenshots across the eight
languages with translated captions; the export/import CSV flow exists for bulk
editing ([import and export](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/import-and-export-store-listings)).

### 1.5 Submission mechanics: listing fields, category, age rating, pricing, release options

Listing fields and limits
([add and edit Store listing info](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/add-and-edit-store-listing-info)):

| Field | Limit |
|---|---|
| Description (required) | **10,000 characters**, plain text |
| What's new in this version | 1,500 characters |
| Product features (bulleted list) | up to **20** items, **≤ 200 chars** each |
| Short description | 1,000 chars, but only the first **270** show — if absent, the first 500 chars of the description are used instead |
| Short title / sort title / voice title | 50 / 255 / 255 chars |
| Screenshots + captions | see §1.4 (captions ≤ 200 chars) |
| Search keywords | **≤ 7** unique terms — this is a *policy*, 10.1.3: "Not exceed seven unique terms or phrases. Be relevant … Not include pricing terms. Not use other product titles …" ([Store Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies)) |
| Copyright/trademark info, license terms | free-text (submission API fields `copyrightAndTrademarkInfo`, `licenseTerms`, [manage app submissions](https://learn.microsoft.com/en-us/windows/uwp/monetize/manage-app-submissions)) |

Category for a calculator: **Utilities + tools** is the documented home —
its examples literally list "file manager, **calculators**, barcode scanner…"
([categories and subcategories](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/categories-and-subcategories)).
No subcategory is required for non-game apps; an optional secondary category
(Partner Center uses "Utilities & tools" / "Productivity" display names —
Windows Calculator's own listing carries Category `Utilities & tools`, §2).
Categories can be changed later by a new submission (only Games locks in).

Age rating: an IARC-style multiple-choice questionnaire runs once per app and
its answers carry to all subsequent updates
([age ratings](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/age-ratings)).
For epher the answers are the boring ones: category "productivity/tools or
education", no accounts, no user-generated content, no online interactivity,
no purchases, no ads, no sharing of personal info — which generates the lowest
ratings in every market (the questionnaire's own category branch determines
which questions even appear). Microsoft shares publisher display name + email
with IARC; an appeal path exists via the rating certificate email.

Pricing and markets ([price and availability](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/price-and-availability)):
a base price is required and **Free** is one of the choices; the Store reaches
"over 240 countries and regions" and defaults to all of them — leave that.
Stop distribution with the "Stop acquisition" option under availability.

Release options ([manage submission options](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/manage-submission-options)):
default is publish-as-soon-as-certified; alternatives are "Don't publish this
submission until I select Publish now" (manual) or a scheduled date ≥ 24 h
out. The **Notes for certification** box is where a real tester reads context —
for epher: "offline app, no login, run `epher` in a terminal or `epher-gui`
from Start; the graphing calculator is the GUI, the CLI shares the engine" —
plus a privacy-policy URL if the listing references one (the common-failures
list calls out privacy policy and offline behavior explicitly,
[resolve submission errors / avoid common certification failures](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/resolve-submission-errors)).

### 1.6 Certification review: duration, scans, common rejections

[The app certification process](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/app-certification-process) (page updated 2026-08-24):

- **Duration: "up to three business days"**, "though it can be quicker"; after
  publish, the listing is typically visible "within about 15 minutes".
- Three test phases: **security** (the submitted packages are checked for
  viruses/malware — for epher's MSIX route that means the MSIX payload, our
  binaries, get scanned; the docs point to the "Trust and Security Services
  Scan"), **technical compliance** (the Windows App Certification Kit — "The
  Store installs and runs your app to verify it behaves as expected"), and
  **content compliance** (listing text, age rating, screenshots).
- Publishing phase: Microsoft re-signs the MSIX; after that starts, the
  submission can no longer be cancelled or rescheduled.
- Post-publish spot checks exist; failures come with a certification report,
  fix → new submission → certification again.

Common desktop-app rejection reasons, from the
[avoid common certification failures](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/resolve-submission-errors) list and the policies:
submitting unfinished apps; crashing without network ("Ensure that your app
doesn't crash without network connectivity" — epher is offline by design, but
the *checklist* means testers may pull the network); missing/wrong identity
("The name found in the package is not one of your reserved app names" — the
§1.2 manifest match); misleading metadata (policy 10.1); missing privacy
policy when personal info is touched (epher: state no-telemetry plainly);
declaring accessibility without engineering it; keyword padding (10.1.3);
incomplete age-rating answers. Malware policy 10.2.3 ("must not contain or
enable malware as defined by the Microsoft criteria for Unwanted and Malicious
Software") is what the security scan enforces — for the NSIS route the same
scan applies to the installer binary fetched from the developer URL, which is
one more (minor) argument for MSIX where only the store-signed package
matters.

### 1.7 Store-installed updates, and staying parallel outside the Store

Updates for the MSIX Store package flow **only through the Store**: create a
new submission, and "Existing users will receive the update via the Store
automatically"
([publish update](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/publish-update-to-your-app-on-store)),
optionally as a gradual percentage rollout or a mandatory update
([upload MSIX app packages](https://learn.microsoft.com/en-us/windows/apps/publish/publish-your-app/msix/upload-app-packages)).

Tauri's docs: the current v2 Microsoft Store guide says nothing about update
tooling, and the updater plugin page
([tauri.app/plugin/updater](https://tauri.app/plugin/updater)) does not mention
the Store at all (verified against the docs source). The v1-era guidance to
disable an in-app updater for Store builds **could not be verified in current
Tauri docs**; the mechanical reason stands on its own: the Store already owns
update distribution for the MSIX identity, and epher's NSIS channel has no
updater today, so nothing to turn off — just never wire the Tauri updater
plugin to the Store package.

Parallel distribution is fine and needs no special dispensation: the Store
install is identified by the Partner Center identity/PFN (a Store MSIX), while
the NSIS installer writes to Program Files / user PATH with no package
identity — two independent installs of independent mechanisms. The one rule:
**never let them share an identity** — the NSIS build must not adopt the Store
PFN (and Tauri's NSIS doesn't), or updates/uninstalls would collide (identity
semantics: [package identity overview](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview); Store identity values: [view product identity details](https://learn.microsoft.com/en-us/windows/apps/publish/view-app-identity-details)). Mark as inference where it goes beyond citation: the docs never discuss side-by-side NSIS+MSIX directly; the independence follows from the identity model.

### 1.8 winget

- The `msstore` source **is** the Store catalog: "WinGet specifies the
  following three default sources … **msstore — The Microsoft Store catalog**",
  endpoint `https://storeedgefd.dsx.mp.microsoft.com/v9.0`
  ([winget source](https://learn.microsoft.com/en-us/windows/package-manager/winget/source)).
  A published Store app appears there with no developer-side registration;
  winget talks to the same catalog and uses the Store **product ID** as the
  package identifier (the msstore plumbing in winget-cli is ProductId-based
  end to end, e.g. [MSStore.cpp](https://github.com/microsoft/winget-cli/blob/master/src/AppInstallerCommonCore/MSStore.cpp)).
- `winget search epher` hits msstore by display name (the docs' own example is
  `winget search "Visual Studio Code" -s msstore`
  — [winget search](https://learn.microsoft.com/en-us/windows/package-manager/winget/search));
  `winget install --source msstore <Product ID>` installs the Store package
  (free products) directly.
- **Agreements prompt**: msstore installs prompt to accept license/source
  agreements ("Some applications … will require the user to agree to the
  license or other agreements before installing. … you can auto accept …
  `--accept-package-agreements`"), and scripts should also pass
  `--accept-source-agreements` for the source terms
  ([winget install](https://learn.microsoft.com/en-us/windows/package-manager/winget/install)).
  Headless usage therefore wants both flags; interactive usage shows the prompt
  once per source/package.
- So the Store listing itself is the entire winget-msstore integration: publish
  in Partner Center, then `winget search epher` resolves once ingestion has
  propagated (no published SLA for catalog propagation — could not verify a
  number).
- **Parallel route, one paragraph**: the `winget` community source is a
  different thing entirely — a manifest PR to
  [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs) covering
  the NSIS installer / zip, no Store account involved, updated by manifest PRs
  per release. Optional, complements (does not replace) the msstore presence;
  the release train already attaches the artifacts a manifest would point at.

---

## 2. Listing anatomy of popular apps (DisplayCatalog extraction)

**Which apps, and how their IDs were resolved.** Queried
`https://displaycatalog.mp.microsoft.com/v7.0/products?bigIds=<IDs>&market=US&languages=en-us&MS-CV=x`
on 2026-09-19. Product IDs verified from first-party sources:

| App | Product ID | How resolved |
|---|---|---|
| WhatsApp | `9NKSQGP7F2NH` | detail page `apps.microsoft.com/detail/9NKSQGP7F2NH` resolves (title "WhatsApp"); displaycatalog confirms (publisher "WhatsApp Inc.") |
| Instagram | `9NBLGGH5L9XT` | detail page + displaycatalog ("Instagram") |
| Windows Notepad | `9MSMLRH6LZF3` | detail page + displaycatalog ("Microsoft Corporation") |
| Windows Terminal | `9N0DX20HK701` | detail page + displaycatalog ("Microsoft Corporation") |
| Windows Calculator | `9WZDNCRFHVN5` | microsoft/calculator README links `microsoft.com/store/apps/9WZDNCRFHVN5`; displaycatalog: "Windows Calculator" |
| Microsoft PowerToys | `XP89DCGQ3K6VLD` | aka.ms/getPowertoys → `apps.microsoft.com/store/detail/microsoft-powertoys/XP89DCGQ3K6VLD`; detail page resolves ("Microsoft PowerToys"). displaycatalog `bigIds` returns **0 products** for XP-format IDs — API limitation, so no image/dimension row for it |

**Wrong IDs caught and discarded** (worth recording — the brief's guesses were
stale): `9WZDNCRFHWH5` is *not* Windows Calculator (it resolves to a
third-party app "智机市场-精品应用推荐"); `9WZDNCRFJ2TJ` is *not* Netflix (a
game, "La chuuute by Oasis"); `9NCB6Z2P3KFX` (old Spotify) and `9N5NQ0S3BL7L`
(old Zoom) return nothing/410 — those products were removed from the Store.
Spotify/Netflix/Zoom current IDs could not be resolved from first-party pages
in this session (Store search is client-rendered; DDG/Bing/Mojeek all blocked
the scraper). Six verified apps stand.

**Extracted numbers** (description = `ProductDescription` char count; images =
`LocalizedProperties[0].Images`; dates = product `LastModifiedDate`):

| App (ID) | Category | Desc chars | Screenshots | Screenshot px | Extra art | Last modified |
|---|---|---|---|---|---|---|
| WhatsApp (9NKSQGP7F2NH) | Social | 705 | 5 | 1600 × 900 | logo 50/75/100/300 px + 15-tile set | 2026-09-16 |
| Instagram (9NBLGGH5L9XT) | Social | 784 | 3 | 2732 × 1536 | logo 300 px | 2026-08-12 |
| Windows Notepad (9MSMLRH6LZF3) | Productivity | 140 | 5 | 1366 × 768 + 4 × 1374 × 776 | logo set + 15-tile set | 2026-08-31 |
| Windows Terminal (9N0DX20HK701) | Developer tools (sub: Utilities) | 492 | 3 | 1732 × 980 | super hero 1920 × 1080, poster 1440 × 2160, box art 2160 × 2160, logo + 15-tile set | 2026-08-14 |
| Windows Calculator (9WZDNCRFHVN5) | Utilities & tools | 393 | 7 | 3840 × 2160 | logo set + 15-tile set | 2026-08-31 |

**What the listings do, structurally:**

1. **Titles are the app name, alone.** "WhatsApp", "Instagram", "Windows
   Notepad", "Windows Terminal", "Windows Calculator" — no keyword strings.
   This is policy, not taste: "Your product title or name must be unique and
   must not contain marketing or descriptive text, including extraneous use of
   keywords" (10.1.1, [Store Policies](https://learn.microsoft.com/en-us/windows/apps/publish/store-policies)).
   epher: title "epher" (or "epher — graphing calculator" is *not* better; keep
   the keyword out of the title and in the keywords/description).
2. **Descriptions are short**: 140–784 chars among these six — under 10 % of
   the 10,000 budget. First line = the pitch ("A simple yet powerful
   calculator that includes standard, scientific, programmer, and graphing
   calculator functions…" — Calculator; "WhatsApp from Meta is a 100% free
   messaging app. It's used by over 2B people…" — WhatsApp, social proof in
   line one). WhatsApp/Instagram then run bolded feature mini-headers
   ("Private messaging across the world") with short paragraphs — the
   description's structure is H3-ish headers + 1–2 sentences, not prose.
3. **Short description field**: displaycatalog serves none of these apps a
   separate `ShortDescription` (all empty) — the first ≤ 500 chars of the
   description doubles as the short text, which the listing docs say is the
   fallback. For epher: make the description's first paragraph do both jobs,
   or set an explicit short description with *different* text as the docs
   recommend.
4. **Screenshot sets**: 3–7 images, all 16:9-ish landscape, resolutions from
   the documented 1366 × 768 floor (Notepad's lead image is exactly 1366 × 768)
   to full 4K (Calculator, 7 × 3840 × 2160). Notepad/Catalog don't pad to 10 —
   five strong frames is a normal, complete-looking set. Whether individual
   shots carry caption bars/annotations can't be seen from displaycatalog
   metadata (it stores dimensions and URIs, not design); the guideline doc
   warns only against *adding* logos/marketing and asks for headroom in the
   bottom third for overlay text.
5. **Store logo**: 300 × 300 1:1 present across the set (the tile icon),
   first-party apps additionally carry the full 15-size tile ladder and
   50/75/100/300 px logo ladder — that ladder is generated from one source
   image by `tauri icon`, not drawn by hand.
6. **Localization counts: could not verify.** displaycatalog served only the
   first requested language per call (`languages=de-de,fr-fr,…` truncated to
   one entry), and scraping each Store web locale was out of scope. First-party
   Microsoft apps are visibly localized on the Store web UI; treat a specific
   count as unverified and don't cite one.

**Template for epher's listing, derived**: title "epher"; description ≤ ~700
chars with the graphing-calculator pitch in sentence one and 3–5 bold feature
headers; 5 desktop screenshots at 1920 × 1080 or 2560 × 1440 (above the
1366 × 768 floor, well under the 50 MB cap), real UI as with the VS Code
listing ([vscode-marketplace-page-anatomy.md](vscode-marketplace-page-anatomy.md)
authenticity rules apply verbatim); 300 × 300 store tile from the existing
icon; category Utilities & tools; 5–7 keywords (calculator, graphing
calculator, calculator language, units, unit conversion, plotting, math).

---

## 3. CI/CD

There **is** a REST submission API — the "Microsoft Store submission API"
(manage.devcenter.microsoft.com, a.k.a. ingestion API), authenticated with an
Entra app associated in Partner Center (Manager role, tenant + client ID +
secret, token via client-credentials, 60-minute validity), with the caveat
that the first submission must be created manually in Partner Center
including the age-rating questionnaire before the API can manage the app
([create and manage submissions](https://learn.microsoft.com/en-us/windows/uwp/monetize/create-and-manage-submissions-using-windows-store-services),
[manage app submissions](https://learn.microsoft.com/en-us/windows/uwp/monetize/manage-app-submissions);
full detail already researched in
[store-ci-cd-publishing.md](store-ci-cd-publishing.md) §7). **No OIDC/federation
route is documented for this API** — unlike the VS Code Marketplace's managed-
identity + `vsce publish --azure-credential` path we built, CI here would hold
a client secret; that's the honest asymmetry with the Entra-federated publish
already in place for the Marketplace.

The repo's side is nearly free: the Windows artifacts are already built on a
GitHub Actions `windows-latest` runner (`build-installers.yml` matrix leg,
`cargo tauri build --bundles nsis`, after the pinned trunk + tauri-cli builds;
see the workflow comments on the console/GUI double build), and release.yml
attaches `epher-windows-x86_64.exe` plus the lsp zips to the release. The
incremental CI work for the Store is exactly two steps: **(1)** an MSIX leg —
copy the built binaries + assets into a layout matching the AppxManifest and
run `MakeAppx.exe pack` (Windows SDK is preinstalled on windows runners;
unsigned output is Store-acceptable per §1.3), plus an optional WACK run;
**(2)** a manual Partner Center upload of the .msix for v1. Automating (2) via
the submission API is a v2 decision priced in the earlier doc (Entra app +
stored secret + manual-first-submission constraint); manual upload per release
is a reasonable steady state given the three-day review window dwarfs the
upload effort anyway.

---

## Draft next steps

1. **(user, portal)** Enroll at storedeveloper.microsoft.com as *Individual*,
   MSA upyesp@gmail.com, government ID + selfie on a phone; wait for the
   Partner Center Apps & Games tile. Free; no payout/tax steps needed for a
   free app (§1.1).
2. **(user, portal)** New product → MSIX or PWA app → reserve the name
   ("epher"; check availability first). Three-month reservation clock starts
   (§1.2).
3. **(user, portal)** Copy the Product identity values (Identity/Name,
   Identity/Publisher CN string, PublisherDisplayName, PFN, Store ID) from
   Product management → Product identity into a repo note; these are literal
   manifest values (§1.2).
4. **(repo)** Add `packaging/msix/`: `AppxManifest.xml` (§1.3 sketch, identity
   placeholders from step 3, two Applications + execution aliases, 8 resource
   languages) + a small pack script (`MakeAppx.exe pack` on a layout built
   from the NSIS job's binaries; icons from `crates/tauri-app/src-tauri/icons`)
   (§1.3).
5. **(repo)** Extend the `build-installers.yml` Windows leg (or add a job) to
   run the pack step and attach `epher-windows-x86_64.msix` to the release
   artifacts; run WACK locally or on the runner once before first submission
   (§1.3, §3).
6. **(user, portal)** First submission, manual: upload the .msix; paste the
   listing text (parent session drafts it — §2 template: title "epher",
   description ≤ 10,000 chars with a ≤ 500-char first paragraph, ≤ 7
   keywords, ≤ 20 feature bullets at ≤ 200 chars); crop 5 screenshots to
   ≥ 1366 × 768 PNG (1920 × 1080 recommended); upload the 300 × 300 store
   tile; category Utilities & tools; IARC questionnaire (all-no answers, §1.5);
   pricing Free, all markets; publish as soon as certified (§1.5).
7. **(system)** Certification: up to three business days, then live in ~15
   minutes; fix-and-resubmit per report if needed (§1.6).
8. **(user/my verification)** After "In the Store": check
   `https://apps.microsoft.com/detail/<Store ID>`, then `winget search epher`
   and `winget install --source msstore` with the Store ID; note the
   agreements prompt behavior (§1.8).
9. **(later, optional)** Decide on: submission REST API automation (needs an
   Entra app + stored secret; no OIDC route), winget-pkgs community manifest
   for the NSIS channel, additional listing languages (§1.5, §1.8, §3).
