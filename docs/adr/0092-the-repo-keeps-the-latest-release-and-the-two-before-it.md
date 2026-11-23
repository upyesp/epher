# ADR-0092: the repo keeps the latest release and the two before it

- **Status:** accepted
- **Deciders:** epher maintainers
- **Date:** 2026-11-23

## Context

Every train attaches its assets to a GitHub Release and nothing ever
removed one, so the releases tab grows without bound: each release
carries twenty-odd assets (installers, lsp binaries, editor shells)
plus the tag ref that cut it, and none of it serves anyone. The
landing page, install.sh, the Homebrew formula, the apt and dnf
repositories, and the editor extensions resolve everything through
`releases/latest/download/...` or their own channel, never a pinned
old release; the store, flatpak, snap, and linux-repos jobs read only
the release the current train published. Old releases are pure weight:
a longer listing, stale download links that look authoritative, and
tags for versions nobody ships.

## Decision

When the release workflow publishes a train's release, the release job
prunes the repo to three releases: the one it just published (the
latest) and the two published before it. Ordering is by created_at,
not semver, so a re-drive cannot reorder history. Every older release
is deleted together with its tag (`gh release delete --cleanup-tag`),
right after this train's assets land and before the channel workflows
are dispatched. Drafts are never touched, and the prune is idempotent:
a re-driven release job finds nothing older than the kept three and
deletes nothing.

## Consequences

- Deep links to an old release's assets break by design once a third
  newer train lands. Every documented consumption path resolves
  `releases/latest/...`, which always points at the newest release, so
  no install path depends on a pruned release.
- The previous two releases keep their full asset sets, so the window
  a broken train leaves for grabbing the last known good installer is
  exactly two versions deep.
- GitHub's "latest" pointer follows the newest published release, so
  deleting old ones never moves it.
- The tags of pruned releases disappear with them; rebuilding an old
  version means checking out its commit sha, not a tag.
- A failed prune fails the release job after the release is already
  published. That is safe: the release exists, and re-driving the run
  re-runs the prune, which is idempotent by construction.
