# Publishing the JetBrains plugin to the JetBrains Marketplace

Research from 2026-09-28, before the first publication of
`io.epher.jetbrains`. Sources: the IntelliJ Platform SDK docs
(plugins.jetbrains.com/docs/intellij/publishing-plugin.html,
plugin-signing.html, language-server-protocol.html), the Marketplace
docs (plugins.jetbrains.com/docs/marketplace/best-practices-for-listing.html,
uploading-a-new-plugin.html), and the marketplace's public search and
plugin APIs. Facts verified live against the docs pages themselves.

## The process

1. **JetBrains Account** (account.jetbrains.com) — one account powers
   the marketplace portal, the IDE trial login, and the publish token.
2. **First publication is always manual**: log in at
   plugins.jetbrains.com, account menu, "Upload plugin", accept the
   Developer Agreement, create the **vendor profile** (upyesp), fill
   the plugin details (name, license, tags, category), upload the zip.
   JetBrains reviews new listings (typically a couple of business
   days) before the plugin goes public.
3. **Personal access token**: profile page, **My Tokens**, Generate
   Token. Shown once. This is `JB_MARKETPLACE_TOKEN` in the `stores`
   environment; gradle's `publishPlugin` uses it.
4. **CI from then on**: `.github/workflows/jetbrains-publish.yml`
   (called by release.yml) checks out the tag and runs
   `gradle publishPlugin -PpluginVersion=<v>`; the intellij platform
   gradle plugin (2.18.1) builds and uploads in one task. The
   marketplace **rejects same-version uploads** ("Please change
   version number"), so listing fixes ride the next train like every
   other channel.

## Signing

Optional but recommended: an unsigned plugin shows an install-time
warning. The author generates their own key (openssl, RSA 4096),
`signPlugin` (runs automatically before `publishPlugin` when the
material is present) signs the zip, and the marketplace re-signs with
its own certificate, so users see a JetBrains-trusted signature.
Uploading the public key for marketplace-side verification is not
available yet. Secrets, all optional:

- `JB_CERTIFICATE_CHAIN` (PEM chain), `JB_PRIVATE_KEY` (PEM key),
  `JB_PRIVATE_KEY_PASSWORD`.

Generate (from the plugin-signing doc):

    openssl genpkey -aes-256-cbc -algorithm RSA -out private_encrypted.pem -pkeyopt rsa_keygen_bits:4096
    openssl rsa -in private_encrypted.pem -out private.pem
    openssl req -key private.pem -new -x509 -days 365 -out chain.crt

## Listing format (top-5 analysis)

The five most-downloaded third-party listings (.env files 30.2M,
Rainbow Brackets 24.9M, IdeaVim 22.1M, .ignore 19.9M, CSV Editor
13.4M) share a shape:

- **Description**: 1000-2900 chars of HTML from plugin.xml. The first
  sentence doubles as the search-card preview (docs: the first 40
  characters must carry the short English summary). Structure: one
  short pitch paragraph, then feature bullets with bold lead-ins.
  At most one embedded image; most embed none.
- **Visuals live in the Media section** of the plugin's admin page
  (uploaded once, manually), not in the description: Rainbow Brackets
  carries 28 gallery images, CSV Editor 7. Images in the description
  cannot be zoomed; gallery images can.
- **Tags** are search filters chosen at upload (1-7).
- epher follows it: pitch + six bullets in plugin.xml, the four
  screenshots and the GIF go to Media.

Marketplace rules worth keeping: description must not start with the
homepage; avoid marketing adjectives; name 1-4 words, no "plugin" in
the name; license (EULA or open source + source link) required at
upload.

## Compatibility

The plugin needs the platform LSP API, which ships **only in the
commercial IDEs** (IDEA Ultimate, PyCharm Professional, WebStorm,
PhpStorm, RubyMine, CLion, DataGrip, GoLand, Rider, Aqua, RustRover);
IDEA Community and Android Studio cannot load it. The zip's module
declarations (`com.intellij.modules.lang`) make the marketplace mark
even Community as compatible, so after the first upload the
compatible-products list is narrowed **manually** in General
Information (the admin panel allows it; the docs recommend it).

## Captures

The IDE refuses to run without an account login (verified live: both
the 2026.3 EAP and the 2024.2.4 stable gate on "Log in to JetBrains
Account"; there is no headless or no-login path), so the screenshots
are taken on a desktop machine - the Linux Mint VM. `listing/setup.sh`
prepares it (IDE + plugin + demo project); `listing/CAPTURE-GUIDE.md`
specifies the four shots (editor, hover, completion, results). No
animated GIF: the listing uses static shots only, like the top
listings it mirrors.

## Version notes

- First marketplace upload: the 0.5.57 zip with the new description
  (server fetch stays pinned to the existing v0.5.57 release, so the
  uploaded plugin works immediately).
- Next train 0.5.59 publishes via CI (`publishPlugin` rejects 0.5.58
  nowhere — 0.5.58 was burned only on the VS gallery; but the train
  convention is one version for every client, and 0.5.59 is the next
  free number across all galleries).
- `signPlugin` runs automatically only when the certificate secrets
  exist; without them the plugin publishes unsigned (install-time
  warning only).
