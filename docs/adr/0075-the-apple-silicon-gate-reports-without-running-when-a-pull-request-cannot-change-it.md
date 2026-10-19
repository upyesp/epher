# ADR-0075: the Apple-silicon gate reports without running when a pull request cannot change what it tests

- **Status:** proposed
- **Deciders:** epher maintainers
- **Date:** 2026-10-19

## Context

The macOS Apple-silicon workflow (`.github/workflows/macos-silicon.yml`) is two things at once:

1. **The only arm64 test floor** (ADR-0025 dropped the Intel build): the workspace suite, the embedded web build, the unified binary, the CLI smoke, every repository script transcript, a REPL session, a pty-driven TUI smoke, and a bundled, launched, desktop app.
2. **The promotion gate** (ADR-0073): branch protection requires the check `build and test on Apple silicon` on every staging→main PR.

The two roles conflict at the trigger. A required check that does not run does not report, and a PR whose protection rule expects `build and test on Apple silicon` would sit on "Expected — waiting for status" forever if the workflow's `pull_request:` were path-filtered. So `pull_request:` is deliberately unfiltered, and every pull request — a landing-page copy edit (#178), an IDE listing sentence (#184) — runs the full Apple-silicon gauntlet, up to 60 minutes on the arm64 runner.

The cost was measured before ADR-0073: 17 Apple-silicon runs (144 runner-minutes) in three days, many superseded or duplicate (P6 in `docs/research/ci-reliability-and-build-time-plan.md`), and merges BLOCKED while a second required run was in flight. ADR-0073 removed the waste (cancel-in-progress concurrency) and the conflict-re-run amplifier (merge-commit promotions), and kept the frequency, by design: the gate must always report.

(The Actions run history was cleared after 2026-10-17 — zero runs and zero artifacts remain as of 2026-10-19, per both the authenticated and the public unauthenticated API — so the research document's ledger is the surviving record of these numbers.)

What ADR-0073 could not fix is the shape of GitHub's required checks: a protection rule understands "did not run" and "passed", and only the workflow itself can choose which to report.

## Decision

### The required check name does not change

Branch protection keeps requiring `build and test on Apple silicon`, exactly as today. No protection-rule edits; nothing changes in the merge UI.

### Reporting is split from running

The workflow gains a `changes` job on `ubuntu-latest` that answers one question using the same path list `push.paths` already encodes: *can this pull request change what this workflow tests?* Two jobs then carry the identical `name:` as today's silicon job, under mutually exclusive conditions, so exactly one runs per PR:

- **code paths touched** (or the event is not a pull request) → the full Apple-silicon run, unchanged from today;
- **otherwise** → a one-step ubuntu job under the same name that succeeds with a note ("no macOS-relevant changes; gate satisfied").

Because the passthrough genuinely runs and succeeds, this relies on no GitHub heuristic about skipped checks — the protection rule simply sees its required check report success.

### The path list is single-sourced

The PR filter and `push.paths` must stay the same list, or the gate and the push floor drift apart and coverage disappears silently. Both live in the same workflow file with cross-reference comments naming each other. `site/guide/**` stays on the list: `crates/guide` reads `site/guide/*.md` at test time and the desktop bundle carries the guide.

### The promotion gate is not weakened

Every train bumps crate versions under `crates/**`, so staging→main promotion PRs always match the filter and always run the full suite. What stops running is the PR that cannot change the answer. The release-time guarantee of ADR-0073 — main only ever merges a staging tree whose transcripts replay on arm64 — is untouched.

### Rejected alternatives

- **Path-filtering `pull_request:` directly:** the workflow then never reports on filtered PRs and merges block forever; this is precisely why it is unfiltered today.
- **A single job whose `if:` skips:** conclusion `skipped` does satisfy required checks, but that behavior is a platform heuristic with re-run caveats; a same-named passthrough job that actually runs needs no such reliance.
- **A merge queue:** would moot most of this for main (ADR-0073 left the queue open), but staging feature PRs still need the check and the queue is a larger process change; the gate split composes with a future queue rather than competing with it.
- **A `workflow_run` pair:** the required name would live in a second workflow that watches the first; naming, ordering, and re-run semantics get murkier for no added benefit.

## Consequences

- Docs-, site-, and clients-only PRs report the gate in well under a minute instead of a 20–60-minute arm64 run; the BLOCKED race window shrinks to code PRs only.
- The full suite still runs on every code-touching PR, every qualifying staging and main push, every dispatch, and every promotion.
- Two seconds-cheap ubuntu jobs per PR (the repository is public; minutes are not billed).
- **New failure mode: filter drift.** The PR list and the push list diverging would skip coverage silently. Mitigation: one file, cross-referenced comments, and a possible future conformance step that diffs the two lists.
- Duplicate job display names are legal today (the job ids differ); if GitHub ever stops honoring that, the fallback is the single-job skip whose `skipped` conclusion protection already treats as satisfied.
- "Re-run all checks" on the passthrough replays the same conclusion; a rebase that introduces code paths re-evaluates the gate on the new push regardless.

## Deliberately deferred

- The same split for the other check workflows: the Apple-silicon gate is the expensive one; revisit if another check's frequency becomes a ledger item.
- A gate-list conformance check (asserting `push.paths` and the PR filter match) — worth adding when the filter next changes, not before.
