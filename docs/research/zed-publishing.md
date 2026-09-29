# Publishing epher to the Zed extensions marketplace: the PR-to-submodule flow, what Zed's own CI does, and what we must change first

Research of 2026-09-29, in preparation for publishing `clients/zed` to the
official marketplace at zed.dev/extensions, the gallery the in-editor
Extension Gallery searches. Three questions: how an extension gets into the
registry in the first place, what the listing page renders (and what it does
not), and how our release train drives updates once the listing exists. The
headline: the old "zed-extension-action publishes for you" world is gone —
first publication and every update are now pull requests to
`zed-industries/extensions`, and Zed's own CI does all packaging and
publishing. Nobody outside Zed ever holds a publishing token.

## Goal

Decide what it takes to get the extension with id `epher` (currently
installed as a dev extension from `clients/zed`) into the Zed extension
registry: the repository work the first PR requires, the account/secret
surface (smaller than any other channel we publish to), the marketplace
listing's anatomy, and the CI job that turns each `v0.5.x` train tag into a
registry update.

## Method (all claims verified)

Everything below was fetched directly with curl on 2026-09-29, no blog
posts:

- **Zed's own docs**, the publishing section added under
  zed.dev/docs/extensions: overview, prerequisites, publishing-guide,
  license-requirements, updating-and-maintenance, and the publishing FAQ,
  plus the developing-extensions, installing-extensions and languages pages
  [1]-[9].
- **The `zed-industries/extensions` repository at main**: README,
  CONTRIBUTING.md, `extensions.toml`, the CI workflow, the packaging script
  and its validation library, and the PR template [10]-[16].
- **Zed's packaging code at main**: the extension builder (grammar
  compilation), the extension manifest schema, and the `zed-extension` CLI
  (crates/extension_cli), which Zed's CI downloads as a pinned binary
  [17]-[19].
- **The live registry API** (`api.zed.dev`) for version histories and
  download counts, and the live listing pages on zed.dev for the top
  extensions [20][21].
- **GitHub API checks** that `zed-industries/zed-extension-action` — the
  action every older guide names — no longer exists (404) and that the
  community action the docs now recommend is `huacnlee/zed-extension-action`
  [15][22][23].

## The marketplace and where the listing lives

The official channel is the **Zed Extension Registry**, browsed on the web
at zed.dev/extensions and in-editor via the Extension Gallery
(cmd-shift-x, or `zed: extensions`) [1][9]. Concretely:

- **Listing URL shape**: `https://zed.dev/extensions/<id>` — for us
  `https://zed.dev/extensions/epher` [21]. The "Install in Zed" button on
  the page is a deep link, `zed://extension/<id>` [21].
- **Registry API**: `https://api.zed.dev/extensions/{id}` returns the
  version history (each version with its `published_at`, `provides`, and a
  `download_count`), no authentication needed [20]. The editor itself
  installs from the registry backed by a blob store; Zed's CI uploads
  packages under `extensions/<id>/<version>/` keys [13].
- **Scale**: `extensions.toml` on main listed 1,558 extensions on the day
  of research [12]. The highest download count observed was 7.1M (the html
  extension) [20][21].

There is **no official popularity ordering**. The web gallery has a search
box and a curated front grid, no sort control (the page HTML has no sort or
"popular" UI) [21]. The only public popularity signals are the rounded
download figure on each gallery card (name, count like "1.8M", description,
authors) and the exact "Downloads:" line on each listing page [21].

## Publishing, step by step (first publication)

From the publishing guide, prerequisites, FAQ and the repository's own
tooling [2][3][5][11]-[16]. What can be automated is called out at the end.

1. **Read the prerequisites and self-check** [2]. For a language extension
   the binding ones: test at the exact submodule commit you submit; do not
   duplicate an existing extension's functionality; the id must be unique,
   kebab-cased, and must not contain `zed`, `-zed` or the word `extension`
   (validated as `/^[a-z0-9-]+$/` with prefix/suffix exception lists, staff
   only) [3][14]; all user-facing text in English; for language extensions,
   **define a grammar in `extension.toml` for every language you provide**
   — this is a hard prerequisite for language extensions and forces the
   tree-sitter decision below [2][8].
2. **Add an accepted license inside the extension directory** [4]. As of
   2025-10-01 the PR fails CI without it. Accepted: Apache-2.0, BSD-2/3,
   CC-BY-4.0, GPLv3, LGPLv3, MIT, Unlicense, zlib. The file (any name with
   a `LICENSE`/`LICENCE` prefix) must live **in the extension's path**, not
   merely at the repository root — a symlink of the root MIT file into
   `clients/zed/` is explicitly allowed [4][14].
3. **Fork `zed-industries/extensions`** — to a personal account if
   possible, so Zed staff can push fixes to your PR [3].
4. **Add the extension as a git submodule** at exactly
   `extensions/<extension-id>` (the name and path must match the id;
   validated) with an **HTTPS URL**; the repo must be public and the pinned
   commit must be reachable from a branch (no detached commits) [3][14].
   For us the submodule is the epher monorepo itself:
   `git submodule add https://github.com/upyesp/epher.git extensions/epher`.
5. **Add the `extensions.toml` entry** [3][12]:

   ```toml
   [epher]
   submodule = "extensions/epher"
   path = "clients/zed"     # relative to the submodule root
   version = "0.5.57"       # strict semver, no leading "v", must match
                            # extension.toml at the pinned commit
   ```

   The `path` field is joined onto the submodule directory by the packaging
   script, so a monorepo extension is a first-class case (the `42nd-theme`
   entry uses it the same way) [12][13].
6. **Sort and open the PR.** `pnpm sort-extensions` keeps `extensions.toml`
   and `.gitmodules` sorted (CI enforces both) [3][11]. The PR template is
   a single checklist line: read the contribution guidelines [16]. PR
   rules: exactly one extension per PR, at most three open PRs, and a
   response to maintainer feedback within 3 weeks or the PR is closed (a
   fresh PR may be opened any time) [3][5].
7. **Review.** A human maintainer reviews; most submissions get first
   feedback within a few weeks, some take one to two months [5]. The FAQ
   states plainly that updates are held to the same standards as new
   submissions [5].
8. **After merge, Zed's CI publishes — not you.** On every push to main,
   `package-extensions` runs: it compiles each extension whose version is
   not yet in the blob store with the pinned `zed-extension` CLI (Rust to
   `wasm32-wasip2`, plus grammar compilation) and uploads the package to
   Zed's S3-compatible store (`SHOULD_PUBLISH=true` on main, S3 keys are
   Zed's secrets) [11][13][19]. No contributor credential is involved at
   any step.

**What cannot be automated:** the fork, the first PR, and the human review
conversation (including the 3-week responsiveness rule). Everything before
the PR — version lockstep, license file, grammar pinning, local test
packaging with the CLI — is ordinary repo work CI can do.

## The grammar decision this forces

`clients/zed` deliberately ships no `[grammars.*]` entry because dev
extensions resolve a wasi-sdk toolchain on the user's machine for any
declared grammar (the 0.5.47 field reports). That reasoning **flips** for
the marketplace: registry installs download a finished package, and the
packaging pipeline compiles declared grammars **on Zed's CI** — the builder
clones the grammar repository at the pinned rev and compiles it with
wasip-sdk 34 in their build environment [17][19]. So marketplace
publication means adding to `extension.toml`:

```toml
[grammars.epher]
repository = "https://github.com/upyesp/tree-sitter-epher"
rev = "<pinned commit sha>"
```

compiled once by Zed's pipeline from the pinned revision, exactly as
extension.toml's comment anticipated. The prerequisite "define a grammar
for every language you provide" makes this effectively mandatory for
acceptance [2][8][17].

## Accounts, tokens, secrets

- **No Zed account exists or is needed.** There is no developer portal, no
  API token, no publisher agreement. Publication identity is your GitHub
  account, via the PR [3][5].
- **CI needs exactly one secret**: a GitHub personal access token with
  `repo` and `workflow` scopes, able to push to our fork of
  `zed-industries/extensions` and open PRs against upstream — the
  `COMMITTER_TOKEN` of the community update action [23]. Store it as
  `ZED_EXTENSIONS_TOKEN` in the `stores` environment (our convention, per
  `.github/workflows/openvsx-publish.yml`).
- `zed-industries/zed-extension-action` is **gone** — the URL 404s
  (verified 2026-09-29). Any doc or workflow still naming it is stale [22].

## How the top five format their listings

No official ranking exists, so the five below are the highest
`download_count` values observed across a 205-id candidate scan of
`extensions.toml` (all language names plus popular tooling and theme
keywords), cross-checked against the live pages the same day [12][20][21].
Treat it as "the most-downloaded of the extensions a new language extension
competes with", not a registry-wide medal table.

| id | downloads | manifest description (chars) | provides | repo |
| --- | --- | --- | --- | --- |
| html | 7,122,588 | "HTML support." (13) | Languages, Grammars, Language Servers | zed monorepo submodule |
| git-firefly | 1,762,957 | "Provides Git Syntax Highlighting" (32) | Languages, Grammars | d1y/git_firefly |
| toml | 1,305,925 | "TOML support." (13) | Languages, Grammars | zed-extensions/toml |
| java | 1,262,823 | "Java support." (13) | Languages, Grammars, Language Servers, Debug Adapters | zed-extensions/java |
| catppuccin | 1,152,956 | "🦀 Soothing pastel theme for Zed" (31) | Themes | catppuccin/zed |

Reading across them:

- **Descriptions are a five-to-thirty-character fragment** ("Java
  support."), not sentences. The card and search preview render this string
  alone; nobody spends words on it.
- **The listing page renders no README, no screenshots, no GIFs, no
  icons** — checked the rendered HTML of all five pages: the only `<img>`
  on each page is Zed's own chrome logo [21]. Below the header the page
  shows a "Details" section, metadata (authors, Version, Downloads,
  Provides badges, "Last updated on …"), the `zed://extension/…` install
  button and a Visit Repository link, then site boilerplate FAQ [21].
- **The Details paragraph is not ours to write**: for most extensions it
  equals the manifest description (git-firefly, catppuccin); a few
  Zed-maintained ones show a templated sentence instead (toml, java: "This
  X extension adds support and functionality directly within Zed."), which
  matches nothing in their `extension.toml` history — it is stored on
  zed.dev's side [20][21]. The one string a submitter controls is
  `description` in `extension.toml`.
- **Categories/tags do not exist** beyond the automatic Provides badges
  (Languages, Grammars, Themes, …), derived from what the package carries
  [18][21].
- **Formatting pattern for a listing, then**: kebab id, short lowercase
  name, one-fragment description, correct provides, nothing visual. The
  long pitch lives in the repository README, one click away on the
  extension's own repo.

## Media spec

There is none. The registry stores no images and the listing page renders
no README, so there is nowhere to attach screenshots — not from the repo,
not uploaded [18][21]. Extension discovery copy is the `description` string
only. (Screenshots of epher in Zed stay where they are useful today: the
website card and the repo README.)

## Icon spec

None. The extension manifest has no icon field — the only `icon` in the
schema is per-agent-server, for in-menu display of agent extensions [18].
Gallery cards and listing pages are text-only. The epher monogram
(`site/icon.svg` in the repo, with `icon-light.svg`/`icon-plain.svg`
variants) has no Zed surface: it stays the brand on epher.org, the GitHub
org avatar, and the other channels' listings.

## Version and update mechanics

- **Versions are strict semver** `major.minor.patch`, no `v` prefix, no
  prerelease suffixes (validated; the only grandfathered exception is
  staff-managed) [14]. Our `0.5.57` qualifies as is.
- **A version publishes exactly once.** On main, the packager publishes
  every `extensions.toml` version not already in the blob store — a
  same-version re-merge is a no-op, and there is no replace or delete
  mechanism exposed to authors [11][13]. If a package is broken, the fix is
  a new version (a new PR).
- **The extension id is immutable** — CI rejects any PR that changes an id
  [14]. The `version` must never decrease [14].
- **Updates are PRs too**: bump the submodule to the new commit and the
  `version` in `extensions.toml` (`git submodule update --remote
  extensions/epher` is the documented shortcut), then PR [6]. The docs
  point to `huacnlee/zed-extension-action` as the community automation: on
  a `v*` tag push it updates the fork's submodule pointer and
  `extensions.toml` and opens the PR upstream, gated on `COMMITTER_TOKEN`
  [6][23].
- **Flow into our release train**: one tag `v0.5.57` across all clients
  (ADR-0066); the Zed extension resolves `epher-lsp` from that same release
  at runtime (lib.rs downloads by `CARGO_PKG_VERSION`). So the registry
  update job is: on tag → verify locks → open/update the upstream PR. Zed's
  merge-to-main packaging then carries the new version to every user's
  Extension Gallery update check [11][13].

## Gap analysis: `clients/zed` today

Ready:

- **id and name**: `epher` passes every id rule (kebab, unique on the day —
  no `[epher]` in `extensions.toml`, no `zed-`/`-zed`/`extension` strings);
  the name `epher` passes the name rules [12][14].
- **version**: `0.5.57` in both `extension.toml` and `Cargo.toml`,
  lockstep-enforced by CI; strict semver [14].
- **description**: one line, English, no banned words, names what it does;
  fine as the search-preview string.
- **Repository metadata**: `repository` set to the monorepo; `authors =
  ["upyesp"]`.

To change or add before the PR:

1. **License inside the extension path**: `clients/zed/` has no LICENSE
   file; the MIT LICENSE sits at the repo root, which "will not work" per
   the license rules when the extension is a subdirectory. Symlink it in
   (`LICENSE -> ../../LICENSE`) — explicitly allowed [4][14].
2. **Add `[grammars.epher]`** pinned to `upyesp/tree-sitter-epher` at the
   release's grammar rev, and re-add the prebuilt-grammar note to the
   README (dev-extension installs still cannot carry a grammar; that
   limitation is now documented as dev-only) [2][8][17].
3. **Decide the submodule target.** The natural entry is
   submodule = `upyesp/epher`, `path = "clients/zed"`. The packaging script
   supports it [13]; `huacnlee/zed-extension-action` documents only an
   `extension-path` input whose semantics for monorepo submodules are
   untested — verify on the first PR, and keep the manual
   `git submodule update --remote` + version-bump commands as the fallback
   [6][23].
4. **README split**: the current README is dev-extension-first ("install as
   dev extension" is step 1). Listing text will be adapted from
   `clients/vscode/README.md` instead — install-by-search first, the two
   settings to flip (semantic tokens, inlay hints) prominent, no social
   media links anywhere (house rule; nothing to strip today, keep it that
   way).
5. **No listing-side assets to prepare**: no screenshots, no icon, no
   categories — do not spend time [18][21].

## Human steps vs. CI

Human-only, one-time:

- fork `zed-industries/extensions` (personal or `upyesp` org fork);
- open the first publication PR (submodule + `extensions.toml` + license +
  grammar entry) and shepherd the review (respond within 3 weeks);
- create the `COMMITTER_TOKEN` PAT (`repo` + `workflow` scopes) and put it
  in the `stores` environment secret `ZED_EXTENSIONS_TOKEN`.

CI, every release train:

- verify `extension.toml`/`Cargo.toml`/`crates/cli` version lockstep and
  that the `v<version>` release exists with the `epher-lsp` assets the
  extension downloads at runtime;
- compile the wasm once as a smoke test (`wasm32-wasip2`);
- bump the submodule + `extensions.toml` in the fork and open the upstream
  update PR.

## CI workflow skeleton

```yaml
# .github/workflows/zed-publish.yml
name: zed publish

# The Zed extension registry channel (zed.dev/extensions). First
# publication is a human PR (docs/research/zed-publishing.md); this job
# only drives UPDATE PRs after each train tag. Zed's own CI packages and
# publishes after merge — no registry token exists or is needed. The one
# secret is a GitHub PAT (repo+workflow scopes) that can push to the
# upyesp fork of zed-industries/extensions; the job skips with a notice
# until it exists.

on:
  workflow_call:
    inputs:
      version:
        description: "release version without the leading v (auto = read from crates/cli/Cargo.toml)"
        type: string
        default: "auto"

jobs:
  zed-update-pr:
    name: open zed-industries/extensions update PR
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
          ZED_EXTENSIONS_TOKEN: ${{ secrets.ZED_EXTENSIONS_TOKEN }}
        run: |
          if [ -n "$ZED_EXTENSIONS_TOKEN" ]; then
            echo "skip=false" >> "$GITHUB_OUTPUT"
          else
            echo "skip=true" >> "$GITHUB_OUTPUT"
            echo "::notice::zed update PR skipped: set ZED_EXTENSIONS_TOKEN in the stores environment (see docs/research/zed-publishing.md)."
          fi

      - name: Verify version lockstep and release
        if: steps.secrets.outputs.skip == 'false'
        run: |
          version="${{ inputs.version }}"
          [ "$version" = auto ] && version=$(grep -m1 '^version' crates/cli/Cargo.toml | tr -d '" ' | cut -d= -f2)
          grep -q "version = \"$version\"" clients/zed/extension.toml
          grep -q "version = \"$version\"" clients/zed/Cargo.toml
          gh release view "v$version" -R upyesp/epher --json assets -q '.assets[].name' | grep -q epher-lsp
        env:
          GH_TOKEN: ${{ github.token }}

      # Community action recommended by zed.dev/docs (the official
      # zed-industries/zed-extension-action no longer exists). Bumps the
      # submodule + extensions.toml in the FORK and opens the PR upstream.
      # Monorepo path field: verify behaviour on the first PR; manual
      # fallback is `git submodule update --remote extensions/epher` plus
      # the version bump, per the updating-and-maintenance doc.
      - name: Open update PR
        if: steps.secrets.outputs.skip == 'false'
        uses: huacnlee/zed-extension-action@v1
        with:
          extension-name: epher
          push-to: upyesp/extensions
        env:
          COMMITTER_TOKEN: ${{ secrets.ZED_EXTENSIONS_TOKEN }}
```

## Sources

All fetched 2026-09-29.

1. Zed docs, "Extensions" overview: https://zed.dev/docs/extensions
2. Zed docs, "Extension Publishing Prerequisites": https://zed.dev/docs/extensions/publishing/prerequisites
3. Zed docs, "Publishing Guide" (submodule + extensions.toml PR flow, PR rules): https://zed.dev/docs/extensions/publishing/publishing-guide
4. Zed docs, "Extension License Requirements": https://zed.dev/docs/extensions/publishing/license-requirements
5. Zed docs, "Frequently Asked Questions" (review times, staleness, id immutability): https://zed.dev/docs/extensions/publishing/faq
6. Zed docs, "Updating an Extension": https://zed.dev/docs/extensions/publishing/updating-and-maintenance
7. Zed docs, "Publishing Extensions" overview: https://zed.dev/docs/extensions/publishing/overview
8. Zed docs, "Developing Extensions": https://zed.dev/docs/extensions/developing-extensions
9. Zed docs, "Installing Extensions": https://zed.dev/docs/extensions/installing-extensions
10. zed-industries/extensions README: https://github.com/zed-industries/extensions/blob/main/README.md
11. zed-industries/extensions CI workflow (SHOULD_PUBLISH, pinned CLI, S3 env): https://github.com/zed-industries/extensions/blob/main/.github/workflows/ci.yml
12. zed-industries/extensions `extensions.toml` (entry shape, `path` field example, no `[epher]`): https://github.com/zed-industries/extensions/blob/main/extensions.toml
13. `package-extensions.js` (path join, version match, blob-store dedupe, S3 keys): https://github.com/zed-industries/extensions/blob/main/src/package-extensions.js
14. `src/lib/validation.js` (id/name/version/license/submodule rules): https://github.com/zed-industries/extensions/blob/main/src/lib/validation.js
15. zed-industries/extensions CONTRIBUTING.md: https://github.com/zed-industries/extensions/blob/main/CONTRIBUTING.md
16. zed-industries/extensions PR template: https://github.com/zed-industries/extensions/blob/main/.github/pull_request_template.md
17. Zed extension builder at main (grammar clone + wasi-sdk compile server-side): https://github.com/zed-industries/zed/blob/main/crates/extension/src/extension_builder.rs
18. Zed extension manifest schema (no extension-level icon; agent-server icon only): https://github.com/zed-industries/zed/blob/main/crates/extension/src/extension_manifest.rs
19. `zed-extension` CLI (crates/extension_cli; packaged binary served from zed-extension-cli.nyc3.digitaloceanspaces.com, URL pinned in ci.yml): https://github.com/zed-industries/zed/blob/main/crates/extension_cli/src/main.rs
20. Zed registry API, per-id version history and download counts, e.g. https://api.zed.dev/extensions/html (also git-firefly, toml, java, catppuccin)
21. Live listing pages: https://zed.dev/extensions/html, /extensions/git-firefly, /extensions/toml, /extensions/java, /extensions/catppuccin, /extensions/sql (details/download rendering, card shape, no sort UI)
22. `zed-industries/zed-extension-action` returns 404: https://github.com/zed-industries/zed-extension-action
23. huacnlee/zed-extension-action README (COMMITTER_TOKEN, inputs): https://github.com/huacnlee/zed-extension-action
