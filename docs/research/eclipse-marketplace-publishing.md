# Publishing epher for Eclipse on the Eclipse Marketplace: the listing, the p2 update site, and what CI can and cannot do

Research of 2026-09-29, in preparation for publishing the Eclipse IDE plugin
(`clients/eclipse`, `epher-eclipse.jar`) on the marketplace the IDE itself
queries: marketplace.eclipse.org, the site behind Help → Eclipse Marketplace
Client. Four questions: what the channel is and who reaches it, how a listing
is created and moderated today, what the listing carries and how updates flow,
and what the most-favorited dev-tool listings do that ours should too.

## Goal

Decide what it takes to put epher on the Eclipse Marketplace with a listing
that matches what the top plugins ship: the account work, the listing form's
fields (including the p2 update-site and Feature-ID fields the Marketplace
Client requires), the media and logo specs, and the honest gap between what CI
builds today (a dropins jar) and what the MPC install path needs (a p2 update
site with a feature).

## Method (all claims verified)

Everything below was fetched directly with curl on 2026-09-29, no blog posts:

- **The marketplace site itself**: the home page, the Quickstart page, the
  Marketplace Client Content Inclusion Policy (v1.1, updated 14 February
  2024), the "How do I specify Feature IDs" page, the Add Content page, the
  login flow, and the installs metrics page [1][2][3][4][5][6][7].
- **The live REST API**: the catalog endpoint, the single-listing node
  endpoints for the five top plugins plus DBeaver and WindowBuilder, and the
  favorites/popular rankings [8][9][10][11].
- **The Marketplace Client source** (`org.eclipse.epp.mpc`, the Eclipse
  Foundation's own client, Yatta mirror at `master`): the header comment that
  enumerates every public API endpoint [12].
- **Top-5 listing pages and their API records**, for the media and structure
  analysis [9][13].
- **Our own artifacts**: `clients/eclipse/README.md`, `plugin.xml`,
  `EpherConnectionProvider.java`, the release workflow, and
  `clients/vscode/README.md` (repo files, no fetch needed).

## The channel

**Eclipse Marketplace** (marketplace.eclipse.org) is the Eclipse Foundation's
listing site for "Eclipse-based solutions"; its install front door is the
**Marketplace Client (MPC)**, an IDE plugin included in every Eclipse IDE
package, which "retrieves listing data by contacting the Eclipse Marketplace
website" [3]. The site counts 1,319 solutions and 54.7M installs served
directly from listings on the day of research [1].

URL shapes:

- Listing page: `https://marketplace.eclipse.org/content/<slug>` (e.g.
  `/content/subclipse`) [1][9].
- Category browse: `/listings/category/<category>` (Editor has 324 listings,
  Programming Languages 109) [1][4].
- Market browse: `/listings/market/<market>` (Tools is our market) [1].
- Per-listing XML API: `/content/<slug>/api/p` [9].
- Installs leaderboard: `/metrics/successful_installs/last30days` [7].

The marketplace site is a Drupal application; a "solution" listing is a node
of type `resource` (the API answers `<type>resource</type>`) [9], and the Add
Content page links "Add a new Solutions Listing" to `/node/add/resource` [4].

One policy line shapes everything else: "The Eclipse Marketplace Client (MPC)
only returns listings with an open source license unless it was submitted by
an Eclipse Member company" [3]. epher is MIT, so it qualifies through the
open-source door.

## First publication, step by step

### The human-only steps (everything that needs the account)

1. **Eclipse Foundation account.** Registration is at
   accounts.eclipse.org/user/register (fetched live, 200) [5]. The
   marketplace does not have its own logins: its Log in link redirects to the
   Foundation's Keycloak at `auth.eclipse.org/realms/community` with client
   `drupal_marketplace` — one Foundation account for everything [6]. (The
   Quickstart still says "same accounts from Eclipse Bugzilla"; that is the
   same account system, unified since.) The Quickstart's advice stands: an
   account is needed for adding and editing listings [2].
2. **Create the listing.** While logged in, Add Content in the top navbar →
   "Add a new Solutions Listing" → `/node/add/resource` [4]. Anonymous
   requests to that path 404, so the form itself is only visible logged in;
   its field inventory is exactly the data model every listing carries (next
   section), which is fully observable through the public API [9].
3. **Fill the fields** (see below): identity, description, taxonomy, media,
   and the p2 install pointers.
4. **Submit into moderation.** "Once you've submitted your listings it will
   be placed into the Moderation Queue and should appear on Marketplace in
   the next 24 business hours" [2]. The moderation criteria are written down:
   content must relate to and work with Eclipse technologies, be in English,
   have working links, and "installable solutions must function as described
   in their description"; DSA rules add no-illegal-content and
   PII-transparency requirements, and the Foundation reserves the right to
   unpublish without notice [3]. Contact: marketplace@eclipse-foundation.org
   [3].
5. **Keep the login for edits.** "In order to edit your listings, you can
   visit the 'My Marketplace' link" [2]. Every later edit (description text,
   Eclipse-versions compatibility, screenshots, the version string shown on
   the listing) is a manual web-form edit by a logged-in human.

### The fields a listing carries

The form is login-gated, so the inventory below is read off the public
listing data model — the same fields the API returns for every listing [9],
displayed on every listing page [13]:

- **Name** and **short description**: one paragraph, shown in MPC search
  results and cards; the top five all land at 169–190 characters [9].
- **Body**: the long description, HTML accepted (the top listings use
  `<p>`, `<ul>`, `<strong>`, `<a>`; DBeaver's is a good specimen) [9].
- **Categories** (multi-select; the marketplace's ~50-item taxonomy:
  Editor, IDE, Languages, Programming Languages, Tools, ...) and **tags**
  (free-tagging) [9][1].
- **License**: one value from the facet list (Commercial, Commercial - Free,
  EPL, EPL 2.0, GPL, LGPL, Apache 2.0, BSD, Free for non-commerical [sic],
  Other, Other Open Source, MIT) [1][9]. This value drives the MPC
  open-source rule [3]. Ours: MIT.
- **Development Status**: Alpha, Beta, Production/Stable, or Mature (the
  search facet; the API `status` field) [1][9].
- **Platform Support**: Windows, Mac, Linux/GTK [13].
- **Eclipse Versions**: multi-select of platform releases; the API stores
  them as `eclipseversion`, e.g. `4.42, 4.41, 4.40, ...` [9]. This is the
  compatibility selection MPC filters on.
- **Update Site URL** (API `updateurl`) and **Feature IDs** (API `ius`, a
  list of p2 installable units flagged required/default). These two fields
  are what make the listing installable from the IDE: "Your product needs to
  be downloadable from an Eclipse p2 update site. Your listing on Marketplace
  requires the URL to that Update Site", with default features identified by
  Feature ID read out of the update site's `site.xml` [2][14]. The IDs feed the MPC's drag-to-install banner and
  the "Install" button; the listing then also exposes a
  `eclipse.org/setups/marketplace/?id=<id>` one-click setup link [13].
- **Support URL, homepage URL, company/organization name, submitted-by
  user** (API `supporturl`, `homepageurl`, `companyname`, `owner`) [9].
- **Logo** and **screenshots** (media spec below).

### The REST API is read-only

The marketplace's public API is the client API the MPC itself consumes. The
client source enumerates it in one comment [12]:

- `/api/p` — markets + categories (verified live) [8]
- `/node/%/api/p` or `/content/%/api/p` — one listing's detail (verified
  live) [9]
- `/taxonomy/term/%/api/p` — category listings [12]
- `/featured/api/p`, `/recent/api/p` — curated and recent results [12]
- `/favorites/top/api/p` — most-favorited; `/popular/top/api/p` — most
  active (both verified live) [10]
- `/related/api/p`, `/news/api/p` [12]
- search: `/api/p/search/apachesolr_search/<query>` [12]

Every endpoint is a GET that returns XML. There is no authentication, no
token, and no write endpoint anywhere in the client's public API [12]. The
old `/marketplace-api` documentation page 404s today [11], and nothing on the
site documents a create/update route: creation is `/node/add/resource` and
edits are "My Marketplace", both session-login web forms [2][4]. **CI cannot
create or update a listing.** There is no token a repository secret could
hold; every listing mutation is a human with the Foundation account, browser
session included.

## How updates flow: the listing points, p2 delivers

The listing is a pointer, not a distribution channel: "These listings are
essentially pointers to external repositories" [3]. Once the listing carries
an update-site URL and feature IDs, plugin updates flow through Eclipse's own
p2 machinery — the marketplace listing does not need touching when only the
feature version on the update site advances. What the listing itself shows
(the version string, "Date Updated", Eclipse-versions compatibility,
screenshots) changes only when a human edits the listing [2][9].

Where the update site should live: any HTTPS host works — the top listings
point at `download.eclipse.org` (WindowBuilder), `dbeaver.io` (DBeaver),
`pydev.org` (PyDev), and, closest to our situation, **`subclipse.github.io`
— Subclipse serves its p2 update site from GitHub Pages**
(`https://subclipse.github.io/updates/subclipse/4.3.x/`) [9]. GitHub Pages
off the epher repo (or a sibling repo) is an established pattern for exactly
this.

## The update site is our gap

Today's `epher-eclipse.jar` is a dropins bundle: CI compiles the OSGi bundle
against `org.eclipse.lsp4e` and attaches it to the GitHub release; the README
says "drop `epher-eclipse.jar` into the `dropins` folder" (repo files). That
install path is fine but invisible to MPC. The marketplace listing's install
path needs:

- a **p2 update site** (site.xml + feature jar + bundle jar) at a stable URL,
- a **feature** wrapping `io.github.upyesp.epher.eclipse`, whose feature ID
  goes into the listing's Feature-ID field.

So the work items are: CI builds the p2 site each release train and publishes
it to GitHub Pages; the listing points at it once. Until then a listing could
still exist as a pointer, but with no Install button doing anything — the
MPC's whole value — so the update site belongs in the same train as the
listing, not after it.

## What the top five format their listings

From `/favorites/top/api/p` (the marketplace's own favorites ranking, fetched
today) [10], then each listing's API record and page [9][13]:

| listing | favorites | installs (total) | short desc | body | categories | tags | screenshots |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Darkest Dark Theme with DevStyle | 5,563 | 2,517,905 | 190 chars | 809 chars | 2 (IDE, UI) | 4 | 2 |
| Spring Tools (aka Spring Tool Suite) | 4,362 | 3,308,004 | 183 chars | 1,478 chars | 1 (IDE) | 5 | 0 |
| SonarQube for IDE | 3,366 | 1,482,627 | 169 chars | 4,002 chars | 5 | 5 | 4 |
| Subclipse | 2,420 | 3,162,567 | 183 chars | 1,016 chars | 5 | 5 | 0 |
| PyDev - Python IDE for Eclipse | 2,268 | 1,824,975 | 178 chars | 2,533 chars | 5 | 5 | 0 |

Reading across them:

- **The first line is a plain declarative "X is ..."** in all five: "Spring
  Tools is the next generation of Spring Boot tooling...", "An Eclipse Team
  Provider plug-in providing support for Subversion...", "PyDev is a plugin
  that enables Eclipse to be used as a Python IDE..." [9].
- **Short descriptions are one sentence, 169–190 characters** — the MPC card
  is small and they do not fight it [9].
- **Bodies are short HTML**: 0.8–4K chars of paragraphs and lists, mostly
  links out (source repo, docs, support forum). **Nobody embeds `<img>` in
  the body** — visuals live in the screenshots gallery only, and four of the
  five carry between zero and four screenshots. SonarQube (4) is the ceiling
  among the five; Spring Tools and Subclipse ship none [9][13].
- **Categories are few** (1–5; IDE is the constant for these) and **tags run
  4–5**, real search terms (`spring`, `subversion`, `eclipse`, `python`)
  [9].
- **Status is Production/Stable** (PyDev: Mature) and **licenses are the
  project licenses** (EPL, EPL 2.0, LGPL; DevStyle is Commercial - Free)
  [9].
- **Eclipse Versions are wide** — PyDev lists 4.6 through 4.37 in one field,
  Spring Tools narrows to 4.41/4.42 — a multi-select the maintainer updates
  by hand as releases are verified [9].
- The install leaderboard `/popular/top/api/p` and the 30-day metrics page
  tell the same story with the same names (GitHub Copilot and TestNG riding
  recent pushes) [7][10].

The pattern for a high-quality listing: declarative one-sentence description,
compact HTML body with links out, a screenshots gallery where the UI shows,
one to five honest categories, an up-to-date Eclipse Versions multi-select,
and a p2 update site the Install button can trust.

## Media and logo spec

Observable from the live site (the form's own help text is login-gated):

- **Logo**: uploaded PNG; originals observed at 110×80 (SonarQube, GitHub
  Copilot) and 80×80 (DBeaver, Spring Tools) [9][13]. The site renders it
  through its `badge_logo` image style at **80×80** in every listing header
  and card [9]. A square PNG in the 80px-plus range is the safe upload;
  RGBA PNG throughout.
- **Screenshots**: PNG (animated GIF also accepted — WindowBuilder's lead
  screenshot is a GIF) [13]. Uploaded originals can be large (SonarQube's is
  3456×2160); the site serves them scaled to its `resource_screenshot` style
  at **1200×750** with a lightbox gallery [13]. No hard count is documented
  publicly; four (SonarQube) is the most observed among the top listings
  [9][13]. Screenshots are a gallery field of their own, separate from the
  body HTML.
- **Our brand**: the epher monogram — `site/icon.svg`, the rounded tile +
  monogram "e" (`#1c1c1e` tile, `#fcfcfc` glyph; `icon-light.svg` and
  `icon-plain.svg` variants). The listing logo is a PNG render of it (the
  tile already carries its own rounding, so a straight raster export at
  256×256 scales cleanly to the 80×80 badge). The same render serves the
  screenshots' framing and any feature artwork. No social media links
  anywhere in the listing — repo, docs, and epher.org only.

## Listing copy source

The listing text adapts from `clients/vscode/README.md` (repo file): the
"epher is a calculator language" intro, the answers-inline / units /
diagnostics / hover / completion feature bullets, and the hero screenshot and
demo GIF. Adaptations for Eclipse: the install section becomes the p2/MPC
install plus the PATH story from `clients/eclipse/README.md` (server binary
from PATH, bring-your-own-binary, ADR-0066); the VS Code-specific sections
(Run script CodeLens, launch.json, wasm/browser) come out; the screenshots
show the Eclipse editor, not VS Code's. All links point at
github.com/upyesp/epher and epher.org; **no social media links**.

## Human vs. CI, across the release train

One version across all clients per train (extension 0.5.x speaks epher
0.5.x). With the p2 site in place:

- **CI (automatable)**: build `epher-eclipse.jar` (already does), build the
  feature + p2 update site, publish the site to GitHub Pages under the
  stable URL the listing points at, attach artifacts to the release.
  Repeat every train; the marketplace needs no notice.
- **Human (one-time)**: Foundation account; create the listing at
  `/node/add/resource`; fill fields; pass moderation (~24 business hours)
  [2].
- **Human (per train, small)**: edit the listing when the displayed version
  string, Eclipse Versions compatibility, or screenshots should advance.
  Optional — updates keep flowing through p2 without it — but a listing
  advertising 0.5.56 while p2 serves 0.5.57 erodes trust, so budget the
  two-minute edit into the release checklist.
- **CI secrets**: none possible. No API token exists for listings [12].

## Sources

All fetched 2026-09-29 unless marked as repo files.

1. Eclipse Marketplace home (counts, category/mark browse, license facet): https://marketplace.eclipse.org/
2. Eclipse Marketplace Quickstart (account, Add Content, moderation queue, p2 update-site + Feature-ID requirements, My Marketplace edits): https://marketplace.eclipse.org/quickstart
3. Marketplace Client Content Inclusion Policy v1.1 (MPC open-source rule, moderation criteria, DSA, listing-as-pointer): https://marketplace.eclipse.org/content/marketplace-client-content-inclusion-policy
4. Add Content page (Solutions Listing → /node/add/resource; links to the Feature-ID how-to): https://marketplace.eclipse.org/content/add-content
5. Eclipse Foundation account registration: https://accounts.eclipse.org/user/register
6. Marketplace login redirect to Foundation Keycloak (client drupal_marketplace): https://marketplace.eclipse.org/user/login → https://auth.eclipse.org/realms/community/protocol/openid-connect/auth?client_id=drupal_marketplace&...
7. Installs metrics, last 30 days (ranked by successful installs): https://marketplace.eclipse.org/metrics/successful_installs/last30days
8. REST catalog endpoint (live): https://marketplace.eclipse.org/catalogs/api/p and https://marketplace.eclipse.org/api/p
9. Per-listing REST records (live XML): https://marketplace.eclipse.org/content/dbeaver/api/p, /content/darkest-dark-theme-devstyle/api/p, /content/spring-tools-aka-spring-tool-suite/api/p, /content/sonarqube-ide/api/p, /content/subclipse/api/p, /content/pydev-python-ide-eclipse/api/p, /content/windowbuilder/api/p, /content/github-copilot/api/p
10. Favorites and popularity rankings (live): https://marketplace.eclipse.org/favorites/top/api/p and https://marketplace.eclipse.org/popular/top/api/p
11. The old REST-doc URL /marketplace-api (404 today): https://marketplace.eclipse.org/marketplace-api
12. MPC client source, DefaultMarketplaceService.java (the public-API endpoint list; read-only): https://github.com/YattaSolutions/org.eclipse.epp.mpc/blob/master/org.eclipse.epp.mpc.core/src/org/eclipse/epp/internal/mpc/core/service/DefaultMarketplaceService.java
13. Top listing pages (media markup, gallery, screenshots, update-site links): https://marketplace.eclipse.org/content/sonarqube-ide, /content/windowbuilder, /content/darkest-dark-theme-devstyle, /content/github-copilot, /content/spring-tools-aka-spring-tool-suite, /content/subclipse, /content/pydev-python-ide-eclipse
14. How do I specify Feature IDs for the Eclipse Marketplace Client? (site.xml example): https://marketplace.eclipse.org/content/how-specify-feature-ids
