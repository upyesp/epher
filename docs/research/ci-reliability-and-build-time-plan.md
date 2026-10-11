# CI reliability and build-time plan

Date: 2026-10-10

Status: Implemented 2026-10-10 in the same train (ADR-0073): P1, P2,
P4, P8 as mechanisms; P3 as process (merge-commit promotions, the
merge queue still open); P5 folded into the release train job; P6
completed for flatpak-build (the other check workflows already had
concurrency). P7 (carry-forward assets) remains deferred, as the plan
sequences it.

## Scope and method

Every workflow run from the last three days (64 runs), every failure and
cancellation, and the human-visible failure chain from the release
sessions of 2026-10-10: seven release trains driven in one day, five of
which failed at least once. The point of this document is to turn that
day's lessons into mechanisms, so reliability stops depending on who is
driving the train.

## The ledger

**Failed release drives: 5 of 8 attempts.**

| train | what happened | root cause |
|-------|---------------|------------|
| v0.5.73 | release failed, no release published; mirror tag shipped anyway | jetbrains `DEFAULT_VERSION` outside the bump sweep; the version-lock guard failed correctly |
| v0.5.74 | tag pushed at a wrong commit by a `;`-chained command; release cancelled; fix edits wiped by a hard reset before commit | local-vs-origin ref confusion; uncommitted work + branch switching |
| v0.5.75 | release failed again (same jetbrains guard; 73 and 74 had failed the same way, unnoticed for two trains) | guard runs only at release time, not on the PR that broke it |
| v0.5.78 | tag landed on a stale local branch head; all four desktop builds compiled 0.5.77 code and failed the binary version check; 24 min of matrix time wasted; nvim and sublime publishes failed too | `git tag` executed against local refs without verifying the tree |
| v0.5.79 | success, on the second drive (the first promotion attempt hit the recurring merge conflict) | see P3 |

**Recurring friction, per train:**

- Promotion merge conflicts (`DIRTY`) three times (#121, #129, #131):
  main's squash lineage and staging's lineage both touch the version
  lines every train. Each cost a full re-run of the check suite.
- BLOCKED merges while a second required Apple-silicon run was in
  flight, three times. 17 Apple-silicon runs (144 runner-minutes) in
  three days, many superseded or duplicate.
- Manual post-release verification (mirrors, stamps, rock, tap, assets)
  after every train: 10 to 15 minutes of human time per train, and the
  only reason the jetbrains gap was found at all.
- Release wall time: 37 to 38 minutes, clean-room and cacheless by
  design (the workflow says so), rebuilding every channel even when a
  train touched one subtree.

## Failure patterns and their mechanisms

### P1: tags are cut from local state (burned 0.5.74, 0.5.78)

`git tag` runs against whatever the local checkout happens to be. Two
of five failures are exactly this. The fix is to stop tagging locally.

**Mechanism:** a `release.yml` `workflow_dispatch` (input: version)
that cuts the tag server-side: resolves `origin/main`, verifies the
tree's train version equals the requested version, verifies
`crates/lsp`, `crates/cli`, the jetbrains fallback, and the sublime
placeholder all agree, tags via the API, then runs the existing release
jobs. The human step becomes "click run, type 0.5.80". A
`scripts/cut-release` wrapper keeps the same gate for manual drives:
checkout main, reset to origin, verify the tree, tag, push, with every
step loud.

### P2: the train bump is a hand-maintained sweep (burned 73 through 75)

Eight files carry the train version; the bump is a grep over a list
that has twice missed a file (jetbrains `.kt`, sublime placeholder).
The version-lock guard catches it, but only at release time: too late,
the tag is burned.

**Mechanism:** a `VERSION` file at the repo root as the single source;
`scripts/train <version>` rewrites the eight locations from a manifest
list checked into the repo; and the lock check moves left: the same
guard that fails the release today becomes a required check on every PR
into staging, failing the PR instead of the train. Longer term, stop
shipping the train version in files consumers read at runtime where a
channel allows it (the sublime placeholder is already build-time
stamped; the jetbrains fallback can become a build-time resource the
same way).

### P3: promotion squash lineage forces a conflict every train (#121, #129, #131)

Squash-merging staging into main puts the same content on main under a
different ancestry. The next train bump edits the same lines on both
sides; the next promotion cannot merge cleanly; the conflict resolution
push invalidates the check suite and everything runs twice.

**Mechanism (pick one):**

- Cheapest: promote with a merge commit (`gh pr merge --merge`) instead
  of squash. Main then shares staging's ancestry, future promotions
  merge clean, and conflicts surface at PR time where they are fixable
  before any check runs. Squash stays for feature PRs; promotion
  history is one merge commit per train either way.
- Best: enable the GitHub merge queue on main. The queue runs checks on
  the merged result and merges atomically, which also deletes the
  BLOCKED-stale-run race (three waits this week) and the
  resolve-conflicts-then-rerun-everything loop.

### P4: channel publishes race the release instead of following it

sublime, nvim, luarocks, and vim publishing are separate tag-triggered
workflows. They fire the moment the tag lands, whether or not a release
will ever exist: the 73/74/75 mirrors carry tags whose releases never
published, and only luck (their own version checks) kept the 0.5.78
mirrors quiet.

**Mechanism:** make each publish workflow's first job wait for the
monorepo release run to conclude success (`gh run watch` on the same
tag's release.yml run, or `workflow_run`-triggered publishes gated on
the release conclusion). A failed release must produce zero channel
writes, by construction rather than by luck.

### P5: the doomed-tree matrix costs 24 minutes before it fails

The desktop matrix builds for ~25 minutes per leg and only then runs
the binary version check. On 0.5.78 that was four parallel doomed
builds. The check belongs before the matrix.

**Mechanism:** a pre-flight job at the front of `release.yml` (first in
`build`'s `needs`): checkout, `cargo metadata`, the tree greps from P1,
a host-arch `epher-lsp` build with `--version` asserted against the
tag, budgeted under three minutes. The matrix jobs `needs:` it. With P1
in place such tags stop existing, but the pre-flight also shortens any
future class of bad-tree failure from 25 minutes to 3.

### P6: superseded runs keep burning required-check minutes

Every push to a PR re-runs the Apple-silicon check; merges to staging
and main do it again. 17 runs in three days, several superseded while
in flight, which is also what produced the BLOCKED races.

**Mechanism:** `concurrency: { group: checks-${{ github.ref }},
cancel-in-progress: true }` on the check workflows. A superseded run
cancels in seconds instead of finishing at minute 11 as the "latest
completed" the protection rule then ignores. Merge queue (P3) makes
most of this moot for main.

### P7: every release rebuilds every channel (~37 min, the longest pole)

Release builds are clean-room and cacheless by explicit decision. The
decision is sound for binaries users install; its cost is that a
one-subtree train (docs, site, a single client) still rebuilds four
desktop apps, two flatpaks, and every client archive.

**Mechanism: carry-forward, not cache.** At release time compute
`git diff --name-only <previous-tag>..<tag>`; for any channel whose
subtree is unchanged, download that channel's assets from the previous
release and re-upload them under the new tag, recording in the release
notes which assets were carried. Rebuild only what changed. A typical
single-channel train drops from ~37 minutes to roughly 10 to 12 (rust
crates plus the touched channel); an all-channel train is unchanged.
This needs the release notes to be honest about carried assets, and a
guard that never carries a channel whose build failed upstream.

Deliberately not proposed: turning the clean-room cache back on for
release builds. That decision exists because a stale cache once masked
a broken arm leg; carry-forward gets the time win without touching the
clean-room property of anything that is actually rebuilt.

### P8: post-release verification is a human ritual

Finding the jetbrains gap took a human reading logs two trains late.
The verification gauntlet (release exists, assets present, mirrors
stamped, rock listed, tap bumped) is mechanical.

**Mechanism:** a final `verify` job in `release.yml` (needs: release
and every publish) that runs the gauntlet and fails loudly on any
missing piece, with a short grace window for the hourly channels
(package control, the tap) before it checks them. This also gives P4's
ordering a natural place to live.

### P9: the git hygiene class (twice in one day)

Uncommitted edits wiped by `reset --hard` before a branch switch, and
`git add -A` sweeping `build/` into a train commit (since fixed by
`.gitignore`). These are driver errors, and the mechanism is a script,
not vigilance: `scripts/cut-release` (P1) and `scripts/train` (P2) make
the dangerous sequences single, audited commands; a one-line CI check
(`git status --porcelain` empty after building) keeps artifacts
untrackable.

## Plan, in order

**This week (mechanisms for every failure that actually burned a
train):**

1. P1 server-side tag cut (`release.yml` dispatch + `scripts/cut-release`).
2. P2 `VERSION` file, `scripts/train`, lock check as a required staging check.
3. P4 publishes follow the release conclusion.

**Next (time and money):**

4. P3 merge-commit promotions now, merge queue when there is an
   afternoon to configure it.
5. P5 release pre-flight job.
6. P6 concurrency groups on the check workflows.
7. P8 the `verify` job.

**When release time matters enough to do it carefully:**

8. P7 carry-forward assets per unchanged subtree.

## What success looks like

A train is: PR into staging (version lock checked at PR time), merge,
promote (merge queue, no BLOCKED dance, no DIRTY), click "cut release"
(type the version), release goes green in 10 to 15 minutes for a
typical single-channel train, the verify job posts one green summary
covering every channel. Zero local tags. A red anything stops the
channel writes by construction. The 73-to-76 correction chain of
2026-10-10 would have been: the jetbrains miss caught on the train PR
(P2), the stamp bug caught by the verify job before any human looked
(P8), and no burned anything.
