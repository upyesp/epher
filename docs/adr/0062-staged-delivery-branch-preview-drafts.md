# ADR-0062: staged delivery — a staging branch, a preview site, and draft installers

Date: 2026-09-08

Status: Accepted.

## Context

Since the first public release, every push to `main` deployed the
website and PWA to epher.org (the `pages` workflow), and every `v*` tag
push built the installers, published them to the Releases page, rebuilt
the apt/dnf repositories, and bumped the stores (the `release`
workflow). That was fine while the only user was the author. Now the
site, the PWA, and the apps have public users: a push or a tag is an
instant, irreversible publication to everyone, and there is no place to
see or run a build before it ships. One stray `git push origin main`
or a premature tag publishes to the whole world with no review step in
between.

The project also has no staging surface at all: no preview URL, no test
installers, nothing between "committed" and "live". Releases are
infrequent and deliberate, but each one currently goes live on the
same push that creates it.

GitHub Pages serves exactly one site per repository, so a preview site
cannot live in this repo beside the live one; it needs its own Pages
repo. GitHub Releases are public the moment they are created unless
they are drafts — and drafts are visible only to the repo owner.

## Decision

Development moves to a `staging` branch, and live publication becomes a
deliberate two-step promotion. Nothing ships live before the staged
build has been seen and tested.

1. **`staging` is the development branch.** All day-to-day work lands
   there. Pushing it never touches the live site or the public
   releases.
2. **Every staging push builds the preview site.** The `preview`
   workflow assembles the exact tree the live site build produces
   (shared `site-build.yml`) and publishes it to a second Pages repo,
   `upyesp/epher-preview`, served at `upyesp.github.io/epher-preview`.
   The apt/rpm repository trees are not carried over (the preview is a
   test site, not a package mirror), and the landing page's single
   root-absolute link (`href="/pwa/"`) is rewritten to a relative one
   because the preview is mounted under `/epher-preview/`. The shared
   builder also means a staging push validates the site build before
   any main push happens.
3. **Installers build on demand, into a hidden draft.** The `staging
   build` workflow (manual dispatch, default ref `staging`) runs the
   same reusable installer builder as a public release
   (`build-installers.yml`) and drops the result into a single
   replaced-each-time **draft** GitHub Release. Drafts are invisible to
   the public and carry the same stable asset names a release uses —
   what is tested from the draft is byte-identical to what will ship.
   The draft is created tag-less through the API (GitHub creates the
   git ref only on publish), so a staging build can never trip the
   `release` workflow's tag trigger by accident.
4. **`main` is release-only and protected.** `main` moves only by
   merging a `staging → main` pull request. Branch protection requires
   that PR (with the Apple-silicon transcript/workspace check green),
   enforces linear history, and forbids force-pushes and direct pushes
   even by admins. A stray push to `main` is rejected by GitHub, not
   published.
5. **Promotion is the go-ahead.** When the staged build passes, the
   release version-bump commit lands on `staging`, the PR is merged,
   the `vX.Y.Z` tag is pushed from the merge — and only then do the
   unchanged live workflows run: `pages` deploys epher.org, `release`
   publishes the installers and rebuilds apt/dnf and the stores. The
   release choreography itself (notes, checks) is unchanged; it just
   happens after an explicit human yes.
6. **The macOS test floor covers staging.** The Apple-silicon workflow
   runs on staging pushes and on PRs into main; on PRs it is the
   required promotion gate.

One-time bootstrap exception: the workflow files and this record were
committed straight to `main` (an "ops bootstrap" commit, no version or
site content), because GitHub's Actions UI runs workflows from the
default branch — the dispatch buttons for preview and staging builds
must exist on `main`. After that commit, main obeys the promotion rule.

## Consequences

- The public surface (epher.org, Releases, apt/dnf, Flathub, Snap)
  changes only through the promotion step. "Did that go live?" has a
  single answer: was it promoted?
- A preview of every staged batch exists at
  `upyesp.github.io/epher-preview` (public URL — anything on staging is
  visible there; sensitive work must not be pushed to staging).
- Test installers are downloads the author can keep private (drafts are
  owner-only) and re-test until the batch passes.
- Costs: a second Pages repo and deploy key to maintain; a staging push
  runs the ~8-minute preview build plus the ~10-minute Apple-silicon
  suite; a staging installer build costs the same ~45 runner-minutes as
  a release; promotion now involves a PR (automated via `gh`) instead
  of a bare push. Releases are slightly slower by design.
- Drafts must never be clicked "Publish" — publishing a draft would
  create the `staging-build` tag and attach a half-baked release to the
  public page. Promotion replaces the draft with the real release.
- The old muscle memory (`git push origin main` publishes) is now an
  error message instead of a deployment — which is the point.
