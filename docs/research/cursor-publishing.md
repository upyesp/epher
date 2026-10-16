# Publishing epher to Cursor: what "the Cursor marketplace" is, and our gap

Research of 2026-09-27, in preparation for putting the epher extension in
front of Cursor users. The headline finding changes the shape of the work:
**Cursor has no marketplace of its own to publish to.** Cursor is a VS Code
fork that consumes the Open VSX Registry through Anysphere's own proxy, so
the "publish" half of this workstream already happened when 0.5.56 went to
open-vsx.org. What remains is verification, a badge, and wording.

## Goal

Decide what it takes for a Cursor user to find, install, and trust
`upyesp.epher`: what registry Cursor actually queries, whether our extension
and its `ms-vscode.wasm-wasi-core` dependency are reachable through it, what
Cursor's publisher-verification badge costs, and what (if anything) must
change in the repo or on epher.org.

## Method (all claims verified)

Everything below was fetched directly with curl, no blog posts:

- **Cursor's own docs**, pulled as markdown from their llms.txt mirror
  (`cursor.com/docs/llms.txt` indexes every page as `path.md`): the
  Extensions help page [1] and the Marketplace Security page [2].
- **The Cursor download endpoint list**, scraped from cursor.com/download,
  and the client build itself (`api2.cursor.sh/updates/download/golden/
  linux-x64/cursor/3.22`), for the product-level gallery configuration [3].
- **The live open-vsx.org API**: the `upyesp` namespace record, the `epher`
  extension record, and the `ms-vscode.wasm-wasi-core` dependency record [4].
- **The Cursor forum** (Discourse JSON API), the Extension Verification
  category where badge requests are filed [5].
- **Cursor's proxy host** `marketplace.cursorapi.com` itself, probed for
  route shapes (it 404s every open-vsx and Microsoft-gallery path I tried;
  its root now answers with a bare backend greeting, the old gallery mirror
  is gone) [6].

## What Cursor actually queries

Cursor's Extensions help page states it in one sentence: "Cursor uses the
Open VSX extension registry for third-party extensions" [1]. There is no
Anysphere-hosted registry to publish into; when Microsoft blocked forks from
the Microsoft marketplace, the forks converged on Open VSX rather than
building publisher portals of their own.

The flow has one Cursor-specific layer on top: search and download requests
are routed through Anysphere's proxy (`marketplace.cursorapi.com`), which
runs "automated malware and supply-chain analysis using commercial security
tooling" before an extension is shown or served; extensions that fail review
are blocked [1][2]. The proxy is a filter, not a separate catalog - it serves
Open VSX content with a blocklist.

Consumer-side controls wrapped around the same pipeline: team admins can set
a Marketplace Install Cooldown (hours before a freshly published version is
installable, default off), enterprise allowlists of publishers/extension IDs,
and optional Open VSX signature verification [1][2]. None of these are
publish-side steps; they only shape what users see.

## Why epher is already there

The chain, each link verified live on 2026-09-27:

1. `upyesp.epher` 0.5.56 is published on open-vsx.org with readme, changelog,
   license, and icon [4].
2. The `upyesp` namespace passed ownership review the same day: claim
   EclipseFdn/open-vsx.org#13501, closed with the `granted` label by Eclipse
   ops; the namespace API now reports `verified: true` [4].
3. Cursor queries Open VSX through its proxy [1], so the same record is what
   a Cursor user's Extensions panel searches.
4. The hard dependency survives the trip: `ms-vscode.wasm-wasi-core`
   (the WASI runtime our wasm server needs, per `clients/vscode/package.json`)
   exists on Open VSX at 1.0.2 with a verified namespace [4].

So the install path needs **no new publish, no new account, no new listing**.
Any work beyond this is trust polish, not availability.

## The trust polish: Cursor's verified-publisher badge

Cursor shows a verification badge next to publisher names in its marketplace
UI. Their docs define the exact four-step request [1]:

1. Add a link to the Open VSX listing on a public website **on its own
   domain** ("a GitHub readme is not supported") - epher.org qualifies.
2. Point the Open VSX listing's `homepage` at that website - already done:
   `clients/vscode/package.json` has `"homepage": "https://epher.org"`
   since the Open VSX listing work.
3. Keep the same extension ID across marketplaces - already true:
   `upyesp.epher` on both the Microsoft Marketplace and Open VSX.
4. Post the request in the forum's Extension Verification category with the
   extension name and the website URL [5].

Only steps 1 (site edit) and 4 (forum post) are outstanding. The forum
category shows requests waiting weeks to ~two months for approval, so the
badge is a slow-moving nicety; the extension installs fine without it.

One caveat the docs are blunt about: Open VSX and the Microsoft marketplace
are separate namespaces, and "the same `publisher.extension` name can point
to different publishers or code" in each [1]. Our identical ID and identical
vsix across both registries is exactly the defense they ask publishers for.

## Verified end to end in a real Cursor build (2026-09-28)

The client's `product.json` (extracted from the official linux-x64 3.22
AppImage) pins the gallery [3]:

- `serviceUrl`: `https://marketplace.cursorapi.com/_apis/public/gallery`
  (the Microsoft gallery protocol, Anysphere-hosted; note `_apis`, not
  `_api`),
- `controlUrl`: `https://api2.cursor.sh/extensions-control`, answering
  `{"malicious":[]}` - a blocklist, currently empty [6],
- `galleryId`: `cursor`, and an `extensionReplacementMapForImports` covering
  only eight big Microsoft extensions (remote-ssh, pylance, csharp, ...);
  nothing touches epher or `ms-vscode.wasm-wasi-core` [3].

The smoke install through Cursor's own CLI, against their production proxy:

```
$ cursor --install-extension upyesp.epher
Extension 'ms-vscode.wasm-wasi-core' v1.0.2 was successfully installed.
Extension 'upyesp.epher' v0.5.56 was successfully installed.
```

The client reports Cursor 3.22.7 / `vscodeVersion 1.128.0`, so our
`engines.vscode ^1.88.0` clears easily. The first install attempt returned
"Extension not found" once and succeeded on retry - a cold-cache blip on
their proxy, worth knowing when debugging, not a publish problem. After
install, `epher-lsp.wasm` is present in the extension directory [6].

## What we must do

Ranked, with the zero-cost items first:

1. ~~Smoke test in a real Cursor~~ **done**: installed cleanly through the
   production proxy, dependency and all; see above. A GUI pass (search in
   the panel, open a `.epher` file, run a script) remains as an optional
   deeper check for listing screenshots.
2. **Site edit**: link the Open VSX listing from epher.org/ide.html (badge
   requirement 1, and generally useful). Deploys via the pages workflow.
3. **Badge request** (needs the user's forum account): post in the
   Extension Verification category per the template above. Expect weeks of
   queue. Optional: without the badge the extension installs exactly the
   same; it is a trust signal only.
4. **Wording rides the next train**: README and ide.html can name Cursor
   alongside VSCodium ("works in VS Code, VSCodium, and Cursor") with no
   functional change.

Items 1-2 are agent-doable; 3 is a user click-and-post; 4 is a commit that
can join any future release.
