# ADR-0082: the macOS build runs when the Linux and Windows builds run

- **Status:** accepted (supersedes ADR-0075, which was never
  implemented; amends ADR-0073's promotion-gate mechanism)
- **Deciders:** epher maintainers
- **Date:** 2026-11-10

## Context

Since ADR-0073 the macOS Apple-silicon workflow
(`.github/workflows/macos-silicon.yml`) has been two things at once:
the only arm64 test floor (ADR-0025 dropped the Intel build, so CI is
the only place the hardware the macOS dmg ships for ever runs epher),
and the promotion gate: branch protection on main required the check
"build and test on Apple silicon" on every pull request into main.

Keeping a required check reporting forced its `pull_request:` trigger
to stay unfiltered (ADR-0075 records why: a path-filtered required
check sits on "Expected; waiting for status" forever). The result was
a standing trigger asymmetry:

- macOS ran on every qualifying push to main and staging, on every
  pull request (including docs-, site-, and clients-only ones), and on
  every dispatch: up to 60 minutes of arm64 runner time each, often
  several times a day;
- the Windows leg and the Linux distribution legs ran only when a
  release or a staging build was dispatched, through the shared
  build-installers recipe.

ADR-0075 proposed keeping the gate and making it cheap (an ubuntu
passthrough that reports the required name on PRs that cannot change
what the floor tests). It stayed proposed. The premise it preserved,
that every pull request must prove arm64 correctness before merge, is
the part the maintainer has now dropped: what matters is that the
macOS build happens when the Linux and Windows builds happen, so all
platforms share one trigger.

## Decision

### The Apple-silicon workflow is a reusable workflow

`macos-silicon.yml` triggers on `workflow_call` and
`workflow_dispatch` only; the `push` and `pull_request` triggers and
their path lists are gone. There is no passthrough job and no filter
to keep in sync, so ADR-0075's filter-drift failure mode cannot occur.
`concurrency` keeps its group but loses `cancel-in-progress`: that
setting earned its keep when every push ran the floor, while now runs
are deliberate events attached to a release or a staging build, and
cancelling a release's verification leg mid-run would fail a train.

### It is called exactly where the Linux and Windows legs are called

- `release.yml` gains a `silicon` job beside `build`, pinned to the
  same verified sha, and the `release` job now waits for it: the tag
  is cut only after the suite, the script transcripts, the guide
  checks, the REPL and TUI sessions, and the desktop launch have
  passed on arm64. ADR-0073's contract, that the release is cut from
  a verified tree, keeps holding at the one place it still applies.
- `staging-build.yml` gains the same call beside its installer build,
  so a staging draft (ADR-0062) is arm64-tested before promotion. The
  draft job does not wait for it: a red arm64 run must not hide the
  installers a human is about to hand-test; the failure is visible in
  the same run.

### The promotion gate is retired

The branch-protection rule on main no longer requires "build and test
on Apple silicon" (only the required-checks section was removed; the
force-push and deletion guards stay). Pull requests into main wait on
no macOS check, and most merges wait on no check at all: no other
branch is protected. Promotion discipline is procedural again:
dispatch a staging build, hand-test the draft, merge, then dispatch
the release, whose silicon job re-verifies arm64 on the pinned sha
before any tag exists.

## Consequences

- macOS runner activity drops from several runs a day to a handful per
  release cycle (staging dispatches, one per release, manual runs).
  The repository is public, so this buys queue latency and merge
  responsiveness, not money.
- Pull requests, including code PRs, merge without arm64 evidence. A
  regression only the arm64 floor catches surfaces at the next staging
  build or at the release instead of at PR time. The staging build
  before every promotion (ADR-0062) is the compensating habit; the
  release-time silicon job is the hard stop.
- The merge race ADR-0075 was written against (a merge BLOCKED while a
  second required run was in flight) is gone, because there is no
  required run.
- The check name survives only as a job display name; nothing keys on
  it any more.

## Rejected alternatives

- Implementing ADR-0075's passthrough: keeps per-PR macOS machinery
  and the single-source path list forever, for a gate the maintainer
  no longer wants.
- Running the floor on staging pushes only (path-filtered): a third
  trigger to remember, duplicating what the preview build already
  covers cheaply; the dispatch-based staging build is the deliberate,
  batch-sized event.
- Folding the floor into build-installers.yml as a matrix leg: couples
  a test floor to an artifact recipe and drags it into every caller's
  intent; two call sites beside the existing one are simpler.
