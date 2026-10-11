# ADR-0073: the release is cut by CI from a verified tree, and the channels follow it

Date: 2026-10-10

Status: Accepted (amends ADR-0062's "a tag push is the last act of a
promotion"; extends ADR-0061's channel story with the process that
keeps every channel honest).

## Context

The release trains of 2026-10-10 failed five times out of eight
attempts, and the full ledger (docs/research/
ci-reliability-and-build-time-plan.md) reduces to five mechanisms, not
to flaky infrastructure:

1. **Tags were cut from local state.** A `;`-chained command pushed
   v0.5.74 at the wrong commit (caught, cancelled), and v0.5.78 landed
   on a stale local branch head, spending the whole 25-minute desktop
   matrix building 0.5.77 code before the binary version checks failed.
2. **The train bump was a hand-maintained sweep.** The jetbrains
   fallback's `DEFAULT_VERSION` sat outside every bump list; the
   release-time version-lock guard failed v0.5.73, v0.5.74, and
   v0.5.75 correctly, but only after each tag was burned, and nobody
   noticed for two trains.
3. **Promotion squashes forced a merge conflict every train** (#121,
   #129, #131): main's squash lineage and staging's lineage both touch
   the version lines, so each promotion needed a conflict-resolution
   push that re-ran the whole check suite.
4. **Channel publishes raced the release.** The tag-triggered channel
   workflows fired whether or not a release would ever exist; the
   mirrors carry 0.5.73/0.5.74/0.5.75 tags whose releases never
   published.
5. **Discovery was human and late.** The verification gauntlet
   (release, assets, mirrors, rock, tap) ran in a person's head after
   each train, and the jetbrains gap surfaced two trains late only
   because someone read the logs.

## Decision

### The release is dispatch-only, and the tag is cut by CI at the end

`release.yml` no longer triggers on `v*` tag pushes. It is a
`workflow_dispatch` with a `version` input (and a `dry_run` flag that
builds and preflights while publishing nothing). The `train` job
verifies, on the dispatched ref (main in production):

- the `VERSION` file equals the requested version, and every copy of
  the train version (the check-version-lock manifest: the two crate
  manifests, the lock entries, the zed manifests, the vscode
  package.json, the jetbrains fallback, the sublime placeholder)
  agrees with it;
- a fresh `epher-lsp` build reports that version (a tree that cannot
  produce the right binary fails in minutes, not after the desktop
  matrix);
- the tag is free, or points at the pinned sha (which makes re-running
  a failed release a no-op for the tag).

Every later job builds from that pinned sha. The tag is cut only in
the release job, after the installers exist, and the GitHub release
attaches to it. Local `git tag` + push is no longer a release act at
all: a tag pushed by hand publishes nothing.

### The version has one source, and the lock is checked on the PR

`VERSION` is the single train source. `scripts/train <version>`
rewrites every copy from one manifest; `scripts/check-version-lock`
fails on any drift; `version-lock.yml` runs the check on every pull
request into staging or main, so the class of failure that burned
three trains now fails the PR that would have caused it. The
release-time guard stays as defense in depth.

### The channels follow the release

The four tag-triggered channel workflows (sublime, epher.nvim,
luarocks, vim.org) now begin by waiting for a published, non-draft
release for their tag; after ten minutes they fail rather than ship.
With the tag cut only after the installers exist and the wait guard on
top, a failed release produces zero channel writes by construction.

### Verification is a job, not a ritual

`scripts/verify-release`, run as the release run's final `verify` job:
the release is published, the stable asset names are all attached, the
four channel workflows concluded success for the tag, and both mirror
repositories carry the new version's stamp. Red here means the train
is not out, whatever the earlier green jobs said.

### Promotions merge instead of squashing

Promotion pull requests (staging into main) merge with a merge commit
from now on; squash stays for feature PRs. Squashed promotions put the
same content on main under a different ancestry, which is what forced
the per-train version-line conflicts. With a shared ancestry,
conflicts surface at PR time, where they are fixable before any check
runs. Feature history stays squashed; promotion history is one merge
commit per train.

### Superseded check runs cancel themselves

`concurrency` groups with `cancel-in-progress` on the check workflows
(macos-silicon and preview already had them; flatpak-build joined), so
a superseded run stops burning minutes instead of finishing late as a
stale "latest completed" a merge attempt then trips over.

### Deliberately deferred: carry-forward assets

Skipping the rebuild of channels whose subtree did not change between
releases (reusing the previous release's artifacts under the new tag)
would take a typical single-channel train from ~37 minutes to roughly
10 to 15. It is deferred until it matters enough to build carefully:
it must carry honestly (release notes say which assets were carried)
and never carry a channel whose build failed.

## Consequences

- The human release step is: dispatch `release.yml` on main with the
  version, watch one run end green with the verify job's summary.
  Re-driving is re-running the same run or dispatching the same
  version; the tag check makes both safe.
- Tags are immutable in practice: the dispatcher refuses a tag that
  exists at any other sha, and versions are never re-pointed (the
  standing rule from the nvim listing work).
- The burn ledger stops growing at v0.5.78: a stale or wrong tree
  cannot be tagged, a wrong version cannot pass the train job, a failed
  release cannot reach the channels, and a missed channel cannot pass
  the verify job.
- The version bump is one audited command (`scripts/train`) plus one
  CI check; the eight copies cannot drift silently again.
- Promotion history changes shape (merge commits instead of squash),
  which is visible but contained: promotion commits were already
  one-logical-unit events.
- v0.5.73 through v0.5.78 stay burned exactly as they are; this ADR
  changes how the next train goes out, not how the old ones are
  remembered.

## Amendment 2026-10-10, same day: enabling changes

Merge-commit promotions needed two settings flipped on the repository
and its main branch: `allow_merge_commit` at the repository level (it
was disabled outright), and main's `required_linear_history` dropped
(it forbids merge commits whatever the repository allows). Everything
else in main's protection is untouched: the required Apple-silicon
check, enforce-admins, the force-push and deletion guards.
