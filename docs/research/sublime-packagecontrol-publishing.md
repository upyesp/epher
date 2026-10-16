# Publishing epher to Package Control: the channel PR, the LSP helper repository, and a release train that only has to push tags

Research of 2026-09-29, in preparation for publishing `clients/sublime` to
Package Control (packagecontrol.io), the package manager Sublime Text users
actually install from. Three questions: where a listing lives and how it is
fed, the exact first-publication process for an LSP-package client like
ours, and what a new release must do for the listing to follow. The
headline: `LSP-*` helper packages are not submitted to the main Package
Control channel at all — the LSP team's own repository
(`sublimelsp/repository`) is included in the default channel, and once our
entry is merged there, every semver tag we push becomes a release with no
further PR, on about an hour's delay.

## Goal

Decide what it takes to get `LSP-epher` (the package in `clients/sublime`:
the LSP plugin client, the shared TextMate grammar, the run command)
installable via Package Control: which channel repository takes it, what
metadata the entry and the GitHub repo need, what the packagecontrol.io
listing renders, and the CI job that syncs a monorepo folder into the
standalone repository Package Control requires.

## Method (all claims verified)

Everything below was fetched directly with curl on 2026-09-29, no blog
posts:

- **packagecontrol.io itself**: the home and stats pages, the legacy
  submission doc (`/docs/submitting_a_package`), and the live
  browse/popular, browse/trending, and package pages for the five
  most-installed packages plus LSP and GitGutter [1]-[4][8][9].
- **The current submission guide** the channel now points to,
  docs.sublimetext.io/guide/package-control/submitting.html [5].
- **The channel repositories on GitHub**: `sublimehq/package_control_channel`
  (the URL `wbond/package_control_channel` redirects there; verified via
  the redirect) — its readme, AGENTS.md, `channel.json`,
  `repository.json`, and PR template — and `sublimelsp/repository` — its
  readme, `repository.json` (all 83 entries parsed), release-auto-update
  script, and CI workflows [6][7][10]-[14].
- **Cross-checks**: the `sublimelsp/LSP` GitHub repo description against
  the listing's description line, and the rendered HTML of package pages
  for image handling [9][12].

## The official channel and where the listing lives

Package Control is the Sublime Text package manager: 5,721 packages
indexed, 25.53M unique users counted, ~10K installs/day, 29.1GB of channel
JSON served in a day, per the live stats page [2]. Concretely:

- **Listing URL shape**: `https://packagecontrol.io/packages/<name>`, name
  URL-encoded (spaces become `%20`) — for us
  `https://packagecontrol.io/packages/LSP-epher` [9].
- **Where listings come from**: the channel sources on GitHub are crawled
  and compiled into a single `channel_v3.json` that Package Control clients
  and the website both serve from [6]. The default channel
  (`channel.json` in `sublimehq/package_control_channel`) is a list of
  repository files; line 94 of the current one is
  `https://raw.githubusercontent.com/sublimelsp/repository/main/repository.json`
  — the LSP team's repository, alongside `./repository.json` (the main
  alphabetical per-letter files) and hundreds of third-party
  `packages.json` URLs [6][10].
- **Popularity ordering exists here**: packagecontrol.io/browse/popular is
  an official descending-by-installs list, and browse/trending is the
  official movers list; package pages show total installs split by OS plus
  a 30-day chart [2][3][9].

## Which channel takes epher: sublimelsp/repository, not the main channel

Both the submission guide and the channel readme are explicit: "any
language server protocol packages are managed via SublimeLSP" — LSP
framework plugins are submitted to https://github.com/sublimelsp/repository,
"LSP helper packages... To add your plugin to Package Control, feel free to
open a pull request" [5][7]. Our package is an LSP helper by construction:
`epher.py` subclasses `LSP.plugin.AbstractPlugin`, and the `LSP-` prefix is
technically required — the LSP package resolves the client configuration at
`Packages/LSP-epher/LSP-epher.sublime-settings`, so the folder name and the
listing name must both be `LSP-epher` (the client README already says this)
[15]. The TextMate grammar rides along in the same package, as other
LSP-* packages do (LSP-astro ships syntax; entries carry the `lsp` label
plus a language label) [12].

A pure-syntax `epher` package in the main channel was considered and
rejected: it would split the client from the syntax, and the LSP-package
resolution rule above pins the name to `LSP-epher` anyway.

## Publishing, step by step (first publication)

From the guide, the channel readme, the PR checklist, and the observed
review tooling [5][6][7][10][11].

1. **Stand up the package repository.** Package Control requires **one
   package per git repository, package root at repository root** [5].
   `clients/sublime` is a folder in the epher monorepo, so the listing
   needs a dedicated repository — `upyesp/LSP-epher` — whose root holds
   exactly the package files: `epher.py`,
   `LSP-epher.sublime-settings`, `LSP-epher.sublime-commands`, the three
   `Default (<platform>).sublime-keymap` files, `epher.tmLanguage`,
   `README.md`, `LICENSE`. This repo is created once (human) and synced by
   CI thereafter.
2. **Tag a semver release before the PR.** "A valid semver numbered tag
   must exist on the repository" is an approval requirement, and tag names
   must be semantic version numbers; **branch-based releases are
   deprecated and not accepted for new packages** [5]. Our train already
   tags `v0.5.57`; the package repo carries the same tag.
3. **Prepare the GitHub repo metadata — the listing is built from it.**
   The crawler takes the listing's one-line description from the repo's
   GitHub `description` field (verified: the LSP package's listing line
   equals the `sublimelsp/LSP` repo description exactly) [9][12];
   Homepage and Issues on the listing come from the repo's homepage and
   issues URL; the README is fetched from raw.githubusercontent.com and
   rendered on the page [9]. The PR checklist additionally requires a
   README describing purpose and usage [11].
4. **Clean the repository**: no `.pyc`, no `package-metadata.json`, no
   restricted filename characters; add `.gitattributes` with
   `export-ignore` for anything the package doesn't need (our
   `sync-assets.py`, tests, images) — Package Control builds the install
   archive with git-archive, so this shrinks the download [5][11].
5. **Fork `sublimelsp/repository`** and add one entry to its single
   `repository.json`, in alphabetical position. The 83 existing entries
   use exactly these fields (census over the file): `name`, `details`
   (the GitHub URL), `releases` with `sublime_text` and `tags`, `labels`
   (82 of 83 entries), and `author` (only 8 entries) [12]. Ours:

   ```json
   {
   	"name": "LSP-epher",
   	"details": "https://github.com/upyesp/LSP-epher",
   	"labels": ["lsp", "epher", "language syntax"],
   	"releases": [
   		{
   			"sublime_text": ">=4000",
   			"tags": true
   		}
   	]
   }
   ```

   `tags: true` is the whole update mechanism: "every new tag is a
   release". The `sublime_text` floor is a real decision — LSP itself
   uses `">=4132"`, LSP-clangd and LSP-astro use `">=4070"` — pick the
   floor from the LSP version our client needs (the README names LSP >=
   1.16 for semantic tokens), not from this doc.
6. **Open the PR.** Title format "Add LSP-epher" per the channel's
   AGENTS.md conventions (alphabetical insert, no reformatting of
   existing entries, don't provide information that duplicates the
   README) [10]. The main channel's PR template is the checklist of what
   reviewers check [11]; `sublimelsp/repository` runs the same ecosystem's
   automated reviewers on every PR — `packagecontrol/st-schema-reviewer-action`
   and `packagecontrol/st-package-reviewer-action` — before a human
   looks [13].
7. **Review and merge.** The template comment says "we usually need a few
   weeks"; the guide notes it is a community project reviewed in spare
   time, one package per PR [11][5]. The channel's AGENTS.md is candid
   that a human assesses "the ability and willingness of the human author
   ... to maintain it long term", and asks that the author participate in
   the conversation personally rather than by automation [10].

**What cannot be automated:** the fork, the PR, and the human review. A
PR opened by a bot against a package the author doesn't visibly maintain
is exactly what the checklist and AGENTS.md warn against [10][11].

## Accounts, tokens, secrets

- **No Package Control account exists.** There is nothing to register on
  packagecontrol.io; the entire identity is your GitHub account, via the
  PR [5][7].
- **CI needs one secret**, and only because of the standalone package
  repository: a GitHub PAT (fine-grained, `contents: read/write` on
  `upyesp/LSP-epher`) that lets the release job push the synced package
  files and the version tag. Call it `SUBLIME_PACKAGE_TOKEN` in the
  `stores` environment (our convention). Nothing is needed on the channel
  side — updates after acceptance never touch a PR [12][5].

## How the top five format their listings

From packagecontrol.io/browse/popular, the official descending-by-installs
order, fetched 2026-09-29 [3]. "Package Control" itself heads the list
(25.53M installs, the manager counting its own bootstrap); the top five
*listings* including it, with the next package in parentheses:

| package | installs | description (chars) | labels | README media | repo |
| --- | --- | --- | --- | --- | --- |
| Package Control | 25.53M | "The Sublime Text package manager" (32) | 0 | none | sublimehq/package_control |
| Emmet | 6.14M | "The essential toolkit for web-developers" (40) | 3 (auto-complete, snippets, text manipulation) | text only | emmetio/emmet |
| SideBarEnhancements | 2.95M | "Side Bar Tools and Enhancements for Sublime Text. Files and folders." (68) | 5 | text only | titoBouzout/SideBarEnhancements |
| BracketHighlighter | 2.67M | "Bracket and tag highlighter for Sublime Text" (44) | 1 (brackets) | 4 badge shields + 1 screenshot | facelessuser/BracketHighlighter |
| SublimeLinter | 2.34M | "The code linting framework for Sublime Text" (43) | 2 (linting, SublimeLinter) | 1 badge shield + 1 wide screenshot | SublimeLinter/SublimeLinter |

Reading across them:

- **The listing description is one short sentence** — 32 to 68 characters
  — and it is the GitHub repo description, not a channel field. The
  README carries the actual pitch.
- **READMEs are rendered on the package page**, fetched from the repo and
  shown under a "Readme" heading with a "Source: raw.githubusercontent.com"
  link [9]. Structure converges: H1 name, one-paragraph what-it-does,
  Installation (Package Control: Install Package), features, usage.
- **Media lives in the README and just works**: BracketHighlighter's page
  shows a screenshot PNG and four shields, SublimeLinter's a 785px-wide
  screenshot; the crawler proxies every image to
  `/readmes/img/<sha>.<ext>` on packagecontrol.io, GIFs included
  (GitGutter's page serves two `.gif` files that way) [9].
- **Labels are the only taxonomy** — lowercase free-text tags shown under
  the description; the recommended set is in the guide ("language syntax"
  for syntax packages, and LSP-* packages lead with `lsp`) [5][12].
- **The search preview is the description line**; package names, authors,
  and the ST2/ST3/ST4 badge complete the card on browse pages. Long
  descriptions are simply shown; nobody writes long ones (68 chars is the
  worst offender in the top five).
- Badges like "Trending", "Top 25", "Top 100" on cards are site-computed,
  not submitted [3][9].

## Media spec

There are **no listing image fields** — no upload, no channel JSON key.
Screenshots and GIFs belong in the repository README (absolute URLs, the
same raw-GitHub-URL pattern our VS Code README already uses); the crawler
renders and proxies them onto the package page automatically [9]. The
practical consequence: the adapted README (below) is the only surface
where epher's editor screenshot and demo GIF appear, and no separate
submission step exists. Nothing to size-check: proxied images are served
as-is, so keep the GIF reasonable the way `clients/vscode` already does.

## Icon spec

None. Package Control has no package icon concept — not in the channel
schema (the repository.json fields are name/details/releases/labels/
author and nothing visual), not on package pages or browse cards [9][12].
The epher monogram (`site/icon.svg`, with `icon-light.svg`/
`icon-plain.svg` variants) has no Package Control surface; it stays the
brand on epher.org and the GitHub org avatar.

## Version and update mechanics

- **The tag is the version.** With `"tags": true`, every new semver tag on
  the package repository becomes a release; the crawler re-reads the
  channel sources about once an hour — the "Last Seen" stamp on each
  package page is that refresh time, so a pushed tag goes live within the
  hour with **no PR and no human step** [5][9].
- **Same-version republish does not exist**: a tag is a git fact; Package
  Control lists versions as crawled. Re-pointing an existing tag is the
  only way to "republish" and is a misuse — always bump, matching the
  release-train rule every other channel follows.
- **Version constraints ride in the entry**: `sublime_text` ranges
  (e.g. `">=4000"`) and optional `platforms` (e.g. `["windows"]`) filter
  which installs see the release [5][12].
- **For binary-bundling packages** there is an escape hatch —
  `sublimelsp/repository`'s `auto-update-repository.py` converts GitHub
  release events into entry updates via a `repository_dispatch`
  (`lsp-add-or-update-package`), for packages that attach per-release
  zips [12][14]. epher does **not** need it: `LSP-epher` is pure config
  plus Python; the `epher-lsp` binary is downloaded by the user (or put
  on PATH), so `tags: true` covers the whole train.
- **Flow into our release train**: one `v0.5.57` tag across all clients.
  The Sublime job syncs `clients/sublime/*` to `upyesp/LSP-epher`, tags
  `v0.5.57` there, and the listing follows within the hour. The package
  itself carries no version number — the tag is the version.

## Gap analysis: `clients/sublime` today

Ready:

- The package layout is already correct for a repository root: settings,
  commands, keymaps, grammar, and `epher.py` are flat files that can be
  copied verbatim to `upyesp/LSP-epher`; `epher.tmLanguage` is generated
  from `clients/shared/epher.tmLanguage.json` by `sync-assets.py`, so CI
  can regenerate it rather than copy it [15].
- No context menu entries — the checklist's hard rule is satisfied [11].
- No color scheme ships with the syntax — checklist satisfied [11].
- Key bindings exist but are scoped: the run chord is bound only in epher
  views via the syntax selector. The checklist discourages default
  keybindings ("There aren't enough keys for all packages"), with scoped
  bindings and README documentation as the accepted mitigation — keep the
  scoping, and justify it explicitly in the PR description [11].

To do before the PR:

1. **Create `upyesp/LSP-epher`** (human, one-time) and make the sync job
   below its only writer.
2. **Set the GitHub repo metadata deliberately**: `description` = the
   one-line listing text (adapted from `clients/vscode/README.md`'s
   opening — "A calculator language where every statement's answer appears
   inline, for Sublime Text via the LSP package"), homepage =
   `https://epher.org`, issues enabled. **No social media links anywhere**
   in the description or README (house rule).
3. **Adapt the README from `clients/vscode/README.md`**: keep the hero
   screenshot and demo GIF (absolute URLs already), swap the VS Code
   quick-start for the Sublime install path (Package Control: Install
   Package → LSP-epher; LSP dependency resolves automatically; then the
   `epher-lsp` binary download step and the settings note), keep "Data and
   telemetry: none".
4. **Add `.gitattributes`** `export-ignore` for `sync-assets.py`,
   `tests/`, images not shipped in the package [5][11].
5. **Ship a LICENSE** in the package repo (MIT, matching the monorepo
   root).
6. **Choose the `sublime_text` floor** from the LSP version actually
   required (semantic tokens need a recent LSP; verify against LSP's
   releases, not from memory), then freeze it in the entry [12].

## Human steps vs. CI

Human-only, one-time:

- create the `upyesp/LSP-epher` repository with its description/homepage;
- fork `sublimelsp/repository`, open the "Add LSP-epher" PR with the JSON
  entry, and conduct the review personally (the channel explicitly wants
  the human author in that conversation);
- create the fine-grained PAT and store it as `SUBLIME_PACKAGE_TOKEN` in
  the `stores` environment.

CI, every release train:

- regenerate `epher.tmLanguage` from `clients/shared`, sync all package
  files plus the adapted README to `upyesp/LSP-epher`, push and tag
  `v<version>`;
- nothing else — the crawler picks the tag up within the hour.

## CI workflow skeleton

```yaml
# .github/workflows/sublime-publish.yml
name: sublime publish

# The Package Control channel for LSP-* helper packages
# (sublimelsp/repository, included in the default channel). First
# publication is a human PR (docs/research/sublime-packagecontrol-publishing.md);
# after acceptance, updates are just new semver tags on upyesp/LSP-epher —
# the crawler turns each tag into a release within ~1 hour, no PR. This
# job syncs the package out of the monorepo and tags it. The one secret is
# a fine-grained PAT with contents:read/write on upyesp/LSP-epher; the job
# skips with a notice until it exists.

on:
  workflow_call:
    inputs:
      version:
        description: "release version without the leading v (auto = read from crates/cli/Cargo.toml)"
        type: string
        default: "auto"

jobs:
  sublime-package:
    name: sync upyesp/LSP-epher and tag
    environment: stores
    permissions:
      contents: read
    runs-on: ubuntu-latest
    timeout-minutes: 15
    steps:
      - uses: actions/checkout@v4

      - name: Secrets check
        id: secrets
        env:
          SUBLIME_PACKAGE_TOKEN: ${{ secrets.SUBLIME_PACKAGE_TOKEN }}
        run: |
          if [ -n "$SUBLIME_PACKAGE_TOKEN" ]; then
            echo "skip=false" >> "$GITHUB_OUTPUT"
          else
            echo "skip=true" >> "$GITHUB_OUTPUT"
            echo "::notice::sublime publish skipped: set SUBLIME_PACKAGE_TOKEN in the stores environment (see docs/research/sublime-packagecontrol-publishing.md)."
          fi

      - name: Generate the grammar and assemble the package
        if: steps.secrets.outputs.skip == 'false'
        run: |
          python3 clients/sublime/sync-assets.py
          mkdir -p package
          cp clients/sublime/epher.py \
             clients/sublime/LSP-epher.sublime-settings \
             clients/sublime/LSP-epher.sublime-commands \
             "clients/sublime/Default (Linux).sublime-keymap" \
             "clients/sublime/Default (OSX).sublime-keymap" \
             "clients/sublime/Default (Windows).sublime-keymap" \
             clients/sublime/epher.tmLanguage package/
          # README.adapted.md is the human-maintained adaptation of
          # clients/vscode/README.md (no social links; Sublime quick start)
          cp clients/sublime/README.adapted.md package/README.md
          cp LICENSE package/LICENSE

      - name: Push and tag upyesp/LSP-epher
        if: steps.secrets.outputs.skip == 'false'
        run: |
          version="${{ inputs.version }}"
          [ "$version" = auto ] && version=$(grep -m1 '^version' crates/cli/Cargo.toml | tr -d '" ' | cut -d= -f2)
          git clone https://x-access-token:${SUBLIME_PACKAGE_TOKEN}@github.com/upyesp/LSP-epher.git repo
          cp package/* repo/
          cd repo
          git config user.name "epher release bot"
          git config user.email "bot@epher.org"
          git add -A
          git diff --cached --quiet || git commit -m "sync from upyesp/epher at v$version"
          git tag "v$version" || echo "::warning::tag v$version already exists; version must move with the train"
          git push origin main "v$version"
        env:
          SUBLIME_PACKAGE_TOKEN: ${{ secrets.SUBLIME_PACKAGE_TOKEN }}
```

## Sources

All fetched 2026-09-29.

1. Package Control home: https://packagecontrol.io
2. Package Control stats (5,721 packages, 25.53M users, daily installs): https://packagecontrol.io/stats
3. Package Control browse/popular (official installs ordering): https://packagecontrol.io/browse/popular (trending: https://packagecontrol.io/browse/trending)
4. Package Control legacy submission doc (hosting rules, tags, crawler cadence, channel PR steps): https://packagecontrol.io/docs/submitting_a_package
5. Current submission guide, Sublime Text Community Documentation: https://docs.sublimetext.io/guide/package-control/submitting.html
6. `sublimehq/package_control_channel` readme (crawler → channel_v3.json; LSP packages → SublimeLSP repo; `wbond/package_control_channel` redirects here): https://github.com/sublimehq/package_control_channel
7. `sublimelsp/repository` readme ("LSP Helper Package Repository", PRs welcome): https://github.com/sublimelsp/repository
8. Package Control about page: https://packagecontrol.io/about
9. Live package pages: https://packagecontrol.io/packages/LSP, /packages/GitGutter, /packages/Emmet, /packages/SideBarEnhancements, /packages/BracketHighlighter, /packages/SublimeLinter, /packages/Package%20Control (description source, Details fields, installs, /readmes/img/ proxy incl. .gif)
10. `sublimehq/package_control_channel` AGENTS.md (PR conventions, human review of maintainership) and readme PR instructions: https://github.com/sublimehq/package_control_channel/blob/master/AGENTS.md
11. `sublimehq/package_control_channel` PR template (checklist: semver tag, README, no context menus, keybindings, .gitattributes): https://github.com/sublimehq/package_control_channel/blob/master/.github/PULL_REQUEST_TEMPLATE.md
12. `sublimelsp/repository` `repository.json` (83 entries parsed; field census; LSP entry; `tags: true` releases) and `channel.json` line 94 in the main channel: https://github.com/sublimelsp/repository/blob/main/repository.json, https://github.com/sublimehq/package_control_channel/blob/master/channel.json
13. `sublimelsp/repository` PR CI (st-schema-reviewer-action, st-package-reviewer-action): https://github.com/sublimelsp/repository/blob/main/.github/workflows/on-pull-request.yml
14. `sublimelsp/repository` auto-update script and release-event workflow (binary-bundling packages only): https://github.com/sublimelsp/repository/blob/main/auto-update-repository.py, https://github.com/sublimelsp/repository/blob/main/.github/workflows/on-new-release.yml
15. `clients/sublime/README.md` and `epher.py` in the epher repo (LSP- prefix requirement, AbstractPlugin, sync-assets.py)
