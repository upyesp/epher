# Publishing epher to Open VSX: the registry VSCodium uses, the process, and our gap

Research of 2026-09-27, in preparation for publishing the epher extension to
the marketplace that VSCodium actually queries. Four questions: what the
marketplace is and who consumes it, how publishing works end to end, what the
listing page renders and from where, and what the top extensions do that we
do not.

## Goal

Decide what it takes to get `upyesp.epher` onto open-vsx.org with a listing
that matches what the top extensions ship: the account and namespace work, the
CI wiring, the package.json fields the registry honors (and ignores), and the
concrete gaps in our extension package.

## Method (all claims verified)

Everything below was fetched directly with curl, no blog posts:

- **VSCodium's own docs** (`docs/extensions.md` in the VSCodium repo) for why
  and how VSCodium uses Open VSX [1].
- **Microsoft's own terms** (the PDF that `aka.ms/vsmarketplace-ToU` resolves
  to, September 2025 edition) for the marketplace restriction [3].
- **The eclipse-openvsx wiki and source**: the Publishing-Extensions,
  Namespace-Access, Deleting-Extensions, Trusted-Publishing and
  Using-Open-VSX-in-VS-Code wiki pages [4][5][12][25][30], the `ovsx` CLI
  README (cli/README.md on main) [11], and the server/webui Java and
  TypeScript sources at commit `a6cfaf71b2a1` (main, 2026-09-25), cited by
  file path below [13]-[22][28][29].
- **The live open-vsx.org API**: `/api/version`, the search endpoint, and the
  metadata + README of the five highest-download extensions [6][7][9].
- **The HaaLeo/publish-vscode-extension action README** (recommended by the
  openvsx wiki) at commit `d42ddad3982e` [24].
- **Consumer docs**: the code-server FAQ and the Gitpod `openvsx-proxy`
  component README in gitpod-io/gitpod [26][27].

## The marketplace VSCodium uses

VSCodium's default `product.json` extension gallery is **open-vsx.org, the
Open VSX Registry**, served by the open-source Eclipse Open VSX project and
run by the Eclipse Foundation [1][4]. VSCodium states the reason plainly:
Microsoft "prohibits usages of the Microsoft marketplace by any other
products" or redistribution of `.vsix` files from it, so non-Microsoft builds
must get extensions elsewhere [1]; whether VS Code forks may use the Microsoft
marketplace at all is, in Microsoft's own tracker, an open question [2].

The restriction is contractual, not technical. The Microsoft Visual Studio
Marketplace and NuGet.org Terms of Use define "In-Scope Products and Services"
as "Microsoft Visual Studio, Visual Studio Code, GitHub Codespaces, Azure
DevOps, Azure DevOps Server, and successor products and services", and say:
"Marketplace Offerings are intended for use only with In-Scope Products and
Services and you may not install, reverse-engineer, import or use Marketplace
Offerings in products and services except for the In-Scope Products and
Services" [3]. VSCodium quotes this same clause and adds that it cannot help
anyone intending to infringe it [1].

Who consumes Open VSX, from each consumer's own documentation or source:

- **VSCodium**: pre-set default gallery [1].
- **code-server** (Coder): "we use the Open-VSX extension gallery ... which is
  also used by various other forks" [26].
- **Gitpod**: its IDE serves extension requests through an `openvsx-proxy`
  component that caches the Open VSX registry, present in the monorepo today
  [27].
- **Any VS Code fork or Code-OSS build**: the registry ships a Marketplace API
  adapter; a fork points its `product.json` `extensionsGallery` at
  `https://open-vsx.org/vscode/gallery` and friends [30].

Reach, measured the same day on both registries (2026-09-27): Open VSX
indexes 18,470 extensions, and its top extension has 86.8M downloads [9].
For the same extension, `ms-python.python` shows 59,267,042 downloads on
Open VSX [7] versus 238,607,512 installs on the Microsoft marketplace [8]:
roughly a factor of four. VSCodium and the other forks are the audience; the
Microsoft marketplace is still the larger pond.

Policy: publishing requires an Eclipse Foundation account and a signed
Publisher Agreement, enforced server-side (publishing is rejected with "You
must log in with an Eclipse Foundation account and sign a Publisher Agreement
before publishing any extension") [4][28]. The server validates the `license`
field as free text and does not gate publication on any particular license
type [15]; the live registry indeed hosts extensions with proprietary license
strings [7]. Extensions display a verified shield only when the namespace has
an owner and the publishing user is a namespace member [5].

## Publishing, step by step

All from the official wiki and CLI docs [4][5][11], plus server source where
noted.

1. **Eclipse account.** Register at accounts.eclipse.org; fill in the GitHub
   Username field and use exactly the GitHub account you will log in to
   open-vsx.org with [4].
2. **Sign the Publisher Agreement.** Log in to open-vsx.org (GitHub OAuth),
   then Settings, then "Log in with Eclipse", then show and agree to the
   Publisher Agreement on the profile page. The Eclipse Contributor Agreement
   is a different agreement [4].
3. **Personal access token.** Generate under Settings, then Access Tokens
   (`/user-settings/tokens`). One token can publish any number of extensions;
   generate one per environment (laptop, CI) so it can be revoked
   independently. Pass it via `--pat` / `-p` or the `OVSX_PAT` environment
   variable [4][11].
4. **Create the namespace.** The namespace is the `publisher` field, for us
   `upyesp`, not `epher`. `npx ovsx create-namespace upyesp -p <token>`. The
   creator becomes a *contributor*, not the owner: at first everyone could
   publish, extensions show as unverified with a warning icon [5].
5. **Claim ownership** (needed for verified status and exclusive rights).
   Done publicly by filing the "Claim namespace ownership" issue template in
   EclipseFdn/open-vsx.org. The requester's GitHub ID needs at least 12 months
   of public history, and the claim is validated by one of: the namespace is
   also a VS Code Marketplace publisher with an extension whose repo is owned
   by the requester (same org) or has a commit by them; or domain verification
   via TXT record or email; or, when it is not a VS Code publisher, "the
   namespace matches the GitHub ID making this request". Cases matching none
   of these fall to a slower "all other cases" review. The current template
   has GitHub-org and domain paths but no special path for Eclipse project
   names; those would land in the catch-all option [10]. Our `upyesp` claim
   fits option 3 (namespace equals the GitHub ID) if upyesp is the account
   that files it [10].
6. **Publish.** A pre-built artifact: `npx ovsx publish <file>.vsix -p
   <token>`. From source: `npx ovsx publish -p <token>`, which packages via
   vsce internally (runs `vscode:prepublish`; `--yarn` for yarn; skip the
   dependency walk entirely with `--no-dependencies`, the right choice for
   bundled extensions) [4][11]. A CI job can absolutely publish our
   already-built release `.vsix` this way; the file route does no packaging at
   all [4][11][24]. Before upload, ovsx checks the package size against the
   registry's limit; open-vsx.org reports `maxExtensionSize: 262144000`
   (250 MB) on `/api/version` [6][11].
7. **Scanning.** open-vsx.org runs extension scanning on publish: secret
   detection (rejects when enforced, `// secret-detector:ignore` marks false
   positives), a blocklist hash check, and namespace-similarity
   (typosquatting) checks [4].

**Re-publish and delete rules.** Publishing the same version and target again
is rejected: "Extension ... is already published" [13]. If a stable and a
pre-release share a semver, the registry warns and recommends bumping the
minor, but allows it [14]. `ovsx unpublish` deletes versions (registry 1.2.0
or later; open-vsx.org currently reports `v1.1.2` on `/api/version`, so the
CLI may refuse and the web UI or the direct delete API are the routes) [6][11][12].
Deletion is irreversible in a specific sense: the files are removed but the
version identity stays reserved forever, so the same version number can never
be republished; only an administrator purge frees it [12][13]. Namespace owners
may delete any version; other members only versions they published [12]. The
registry tracks a `deprecated` flag on extensions [17], but the ovsx CLI
exposes no deprecation command; it is not part of the publisher workflow [11].

**GitHub Actions wiring**, three documented shapes:

- Plain step, token in a secret:
  `run: npx ovsx publish epher-vscode.vsix` with
  `env: OVSX_PAT: ${{ secrets.OVSX_PAT }}` [11].
- The community action the wiki links, HaaLeo/publish-vscode-extension: `pat`
  input from `${{ secrets.OPEN_VSX_TOKEN }}`, `extensionFile` input to publish
  a pre-built `.vsix`, `vsixPath` output so one packaged artifact can be
  published to both registries unchanged [4][24].
- Trusted publishing (no stored secret at all): register the workflow under
  Settings, then Trusted Publishers, give the job `id-token: write`, run `npx
  ovsx publish --trusted-publishing`; the registry exchanges the OIDC ID
  token for a minutes-short token scoped to one extension. Prerequisites:
  namespace ownership, a signed Publisher Agreement, and the extension already
  existing with one active version, so the very first version always goes
  through the token route [11][25].

## Listing anatomy and assets

The listing page is rendered by the webui from the extension metadata JSON;
the README is fetched separately from the URL in `files.readme` and rendered
client-side [21]. What the registry pulls out of the `.vsix`:

- **Files that become assets**: `extension/package.json` (manifest tab),
  README (`extension/README.md`, `README`, `README.txt`), CHANGELOG
  (`extension/CHANGELOG.md`, `CHANGELOG`, `CHANGELOG.txt`), LICENSE (from the
  vsixmanifest License metadata or asset), the icon (vsixmanifest asset or the
  package.json `icon` path), and `extension.vsixmanifest` [16]. The overview
  page shows the README; the changelog gets its own section; reviews, versions
  and dependencies have their own UI [20].
- **Manifest fields honored** (extracted from the vsix manifest and
  package.json): displayName, description, engines, categories, tags
  (keywords), license, homepage, repository, bugs, sponsorLink, markdown,
  galleryBanner color and theme, localizedLanguages, qna, preview, preRelease,
  extensionKind, dependencies and packs [16]. The Resources sidebar renders
  Homepage, Repository, Bugs and Q'n'A links [20]; the Preview flag renders a
  Preview badge [29].
- **Fields the validator constrains** [15]: description at most 2048
  characters; displayName, each category/keyword, license, homepage,
  repository, bugs and qna at most 255 characters each; `galleryBanner.color`
  at most 16 characters, `galleryBanner.theme` only `dark` or `light`;
  `markdown` only `github` or `standard`; `qna` only `marketplace`, `false`,
  or a URL; versions must be semver, with `latest`, `preview` and `reviews`
  reserved; namespace names match `[\w\-\+\$~]+`. At most 30 author-declared
  keywords are kept per version [16].
- **Two MS-marketplace fields are ignored here.** The `badges` array from
  package.json is never extracted: the server has the JSON type for it but
  nothing ever calls `setBadges`, and the live API returns `badges: null` for
  extensions that declare badges, including all of the top five [18][7].
  The `sponsorLink` is extracted and served by the API, but no webui component
  renders it today, so there is no sponsor button on open-vsx.org [16][20].
- **Icon.** Taken from package.json `icon` and served as `files.icon` [16][7].
  There is no separate Open VSX size rule; the packaging convention from the
  VS Code docs is a PNG of at least 128x128 px [23]. Our 256 px icon clears
  it.

**Images in the README: relative paths do not work.** The webui fetches the
README as plain text and renders it with markdown-it (html, linkify,
typographer, GitHub-style alerts, heading anchors), sanitized with DOMPurify
[19]. Nothing rewrites relative URLs against the package contents, so an
`images/foo.png` reference resolves against the open-vsx.org page URL and
404s [19][20][21]. The registry does serve arbitrary packaged files at
`/api/{namespace}/{extension}/{version}/file/**` [22], but the website does
not map README-relative paths onto it. Screenshots and GIFs must be absolute
external URLs; that is what the top extensions do (below). There is no
README-length limit beyond the package size; the validator constrains only
metadata fields [15]. The `markdown` setting is accepted and validated, but
the renderer is the same markdown-it pipeline either way [15][19].

## How the top five format their listings

From the live API, search sorted by downloadCount, then each extension's
metadata and README [9][7]:

| extension | downloads | description | categories | links | README |
| --- | --- | --- | --- | --- | --- |
| meta.pyrefly | 86.8M | 111 chars, benefit-led | 3 | repo, bugs, homepage; MIT | 3.3K chars, no images |
| ms-python.debugpy | 60.2M | 40 chars, one sentence | 1 | repo, bugs, homepage; MIT | 5.1K chars, no images |
| ms-python.python | 59.3M | 165 chars | 5 | all four incl. qna URL; MIT | 10.7K chars, no images |
| Anthropic.claude-code | 52.8M | 82 chars | 2 | bugs only; proprietary license text | 1.5K chars, no images |
| Shopify.ruby-lsp | 43.2M | 47 chars | 5 | repo, bugs, homepage; MIT | 19.0K chars, 2 images |

Reading across them:

- **Descriptions are one short sentence** (40 to 165 chars), name the
  language, and let the README sell [7].
- **All five are verified namespaces**; keywords are mostly internal
  `__ext_*` language tags, only python and ruby-lsp carry real search terms
  [7][16].
- **Only ruby-lsp embeds visuals**, a demo GIF plus a status screenshot, both
  as absolute GitHub raw URLs, consistent with the "relative paths break"
  finding above; the other four ship text-only READMEs on this registry [7].
- **Changelogs are not universal**: python and ruby-lsp package one; the other
  three do not [7].
- **Structure of the text-only READMEs converges anyway**: H1 with the
  extension name, one-paragraph intro, a bulleted "what you get"/features
  list, a usage or quick-start heading, then commands, configuration,
  requirements, and for the Microsoft-authored ones a data-and-telemetry
  statement [7]. claude-code is pure marketing bullets with two links out [7].
- **Nobody uses badges, galleryBanner, sponsor or qna** except python (qna
  discussion URL, galleryBanner `#1e415e` dark) [7].

The pattern for a high-quality listing: verified namespace, tight description,
all repo/bugs links present, a features list with a demo GIF where the feature
moves, absolute image URLs, changelog optional.

## Gap analysis: epher's extension today

Read against `clients/vscode/package.json`, `clients/vscode/README.md`,
`clients/vscode/.vscodeignore` and `clients/vscode/LICENSE`:

Already in good shape:

- **Icon**: `images/icon.png`, 256x256 PNG, wired via `icon` [23].
- **galleryBanner** `#1c1c1e` dark; **categories** `Programming Languages`,
  `Snippets`; **keywords**, 7, under the 30 cap; **repository** set; MIT
  license in package.json and a LICENSE file that ships in the vsix.
- **README**: hero screenshot plus animated GIF already exist as absolute
  GitHub raw URLs, which is exactly the format that survives Open VSX's
  no-rewriting renderer [19]; features-with-visuals structure already matches
  the convergent pattern; version and CI badges are shields in the README,
  which still render, unlike package.json `badges` which Open VSX drops [18].
- **Description** is 154 chars, well under the 2048 limit [15].

To add or change:

1. **Namespace work, before any publish**: create `upyesp` with `ovsx
   create-namespace`, then claim ownership via the issue template (option 3,
   namespace equals the GitHub ID) so the listing shows verified [4][5][10].
2. **`bugs` is missing** from package.json, so the listing's Resources sidebar
   has no Bugs entry [20]; add the GitHub issues URL.
3. **`homepage` is missing**; adding `https://epher.org` puts a Homepage link
   in Resources [20] and surfaces the scripts page indirectly.
4. **No CHANGELOG.md ships**, so no changelog section on the listing [16][20].
   Two of the top five lack one too, but for a 0.x project a short
   "what changed" file is cheap and adds the tab [7].
5. **`qna` is unset**: pick a value explicitly, the GitHub issues or
   discussions URL, or `"qna": "false"` to say "no Q&A here", rather than an
   empty field [15][20].
6. **`sponsorLink`**: no action needed; Open VSX extracts but does not render
   it [16][20]. `badges` in package.json: harmless but dead weight here [18].
7. **Quick start copy is Microsoft-marketplace-first**: it names the VS Code
   marketplace page and sideloading, but the Open VSX audience (VSCodium,
   Cursor, code-server) installs from their preset gallery with the same
   `ext install upyesp.epher`; say so in one line [1][26].
8. **Description copy**: fine as is, though it contains one U+2014 character;
   the validator allows it, this is only a search-snippet cosmetics note [15].
9. **CI**: publish the existing release vsix with `npx ovsx publish <file>` and
   the token in a repository secret, or adopt trusted publishing after the
   first token-published version; versions are immutable once published, so
   the release job must always bump [11][13][25].

## Sources

All fetched 2026-09-27 unless noted.

1. VSCodium, "Extensions + Marketplace": https://github.com/VSCodium/vscodium/blob/master/docs/extensions.md
2. microsoft/vscode issue 31168 (fork use of the Marketplace): https://github.com/microsoft/vscode/issues/31168
3. Microsoft Visual Studio Marketplace and NuGet.org Terms of Use (resolves from https://aka.ms/vsmarketplace-ToU): https://cdn.vsassets.io/v/M264_20251020.18/_content/Microsoft-Visual-Studio-Marketplace-Terms-of-Use.pdf
4. Open VSX wiki, "Publishing Extensions" (incl. account, token, namespace, packaging, scanning): https://github.com/eclipse-openvsx/openvsx/wiki/Publishing-Extensions
5. Open VSX wiki, "Namespace Access": https://github.com/eclipse-openvsx/openvsx/wiki/Namespace-Access
6. open-vsx.org live API, `/api/version`: https://open-vsx.org/api/version
7. open-vsx.org live API, extension metadata and README files of meta.pyrefly, ms-python.debugpy, ms-python.python, Anthropic.claude-code, Shopify.ruby-lsp, e.g. https://open-vsx.org/api/ms-python/python
8. Microsoft Marketplace item page, ms-python.python install count: https://marketplace.visualstudio.com/items?itemName=ms-python.python
9. open-vsx.org live API, search by downloadCount: https://open-vsx.org/api/-/search?size=5&sortBy=downloadCount&sortOrder=desc
10. Namespace ownership claim issue template: https://github.com/EclipseFdn/open-vsx.org/blob/main/.github/ISSUE_TEMPLATE/claim-namespace-ownership.yml
11. ovsx CLI README: https://github.com/eclipse-openvsx/openvsx/blob/main/cli/README.md
12. Open VSX wiki, "Deleting Extensions": https://github.com/eclipse-openvsx/openvsx/wiki/Deleting-Extensions
13. server/src/main/java/org/eclipse/openvsx/publish/PublishExtensionVersionHandler.java @ a6cfaf71b2a1: https://github.com/eclipse-openvsx/openvsx/blob/main/server/src/main/java/org/eclipse/openvsx/publish/PublishExtensionVersionHandler.java
14. server/src/main/java/org/eclipse/openvsx/LocalRegistryService.java @ a6cfaf71b2a1: https://github.com/eclipse-openvsx/openvsx/blob/main/server/src/main/java/org/eclipse/openvsx/LocalRegistryService.java
15. server/src/main/java/org/eclipse/openvsx/ExtensionValidator.java @ a6cfaf71b2a1: https://github.com/eclipse-openvsx/openvsx/blob/main/server/src/main/java/org/eclipse/openvsx/ExtensionValidator.java
16. server/src/main/java/org/eclipse/openvsx/ExtensionProcessor.java @ a6cfaf71b2a1: https://github.com/eclipse-openvsx/openvsx/blob/main/server/src/main/java/org/eclipse/openvsx/ExtensionProcessor.java
17. server/src/main/java/org/eclipse/openvsx/entities/Extension.java @ a6cfaf71b2a1: https://github.com/eclipse-openvsx/openvsx/blob/main/server/src/main/java/org/eclipse/openvsx/entities/Extension.java
18. server/src/main/java/org/eclipse/openvsx/json/ExtensionJson.java (badges never set) @ a6cfaf71b2a1: https://github.com/eclipse-openvsx/openvsx/blob/main/server/src/main/java/org/eclipse/openvsx/json/ExtensionJson.java
19. webui/src/components/sanitized-markdown.tsx @ a6cfaf71b2a1: https://github.com/eclipse-openvsx/openvsx/blob/main/webui/src/components/sanitized-markdown.tsx
20. webui/src/pages/extension-detail/extension-detail-overview.tsx @ a6cfaf71b2a1: https://github.com/eclipse-openvsx/openvsx/blob/main/webui/src/pages/extension-detail/extension-detail-overview.tsx
21. webui/src/extension-registry-service.ts @ a6cfaf71b2a1: https://github.com/eclipse-openvsx/openvsx/blob/main/webui/src/extension-registry-service.ts
22. server/src/main/java/org/eclipse/openvsx/RegistryAPI.java (file wildcard endpoint) @ a6cfaf71b2a1: https://github.com/eclipse-openvsx/openvsx/blob/main/server/src/main/java/org/eclipse/openvsx/RegistryAPI.java
23. VS Code docs, "Publishing Extension" (icon at least 128x128 px): https://code.visualstudio.com/api/working-with-extensions/publishing-extension
24. HaaLeo/publish-vscode-extension README @ d42ddad3982e: https://github.com/HaaLeo/publish-vscode-extension#readme
25. Open VSX wiki, "Trusted Publishing": https://github.com/eclipse-openvsx/openvsx/wiki/Trusted-Publishing
26. code-server FAQ (Open VSX as its gallery): https://github.com/coder/code-server/blob/main/docs/FAQ.md
27. Gitpod openvsx-proxy component: https://github.com/gitpod-io/gitpod/tree/main/components/openvsx-proxy
28. server/src/main/java/org/eclipse/openvsx/eclipse/EclipseService.java (publisher agreement enforcement) @ a6cfaf71b2a1: https://github.com/eclipse-openvsx/openvsx/blob/main/server/src/main/java/org/eclipse/openvsx/eclipse/EclipseService.java
29. webui/src/pages/extension-detail/extension-detail.tsx (Preview badge) @ a6cfaf71b2a1: https://github.com/eclipse-openvsx/openvsx/blob/main/webui/src/pages/extension-detail/extension-detail.tsx
30. Open VSX wiki, "Using Open VSX in VS Code" (product.json adapter): https://github.com/eclipse-openvsx/openvsx/wiki/Using-Open-VSX-in-VS-Code
