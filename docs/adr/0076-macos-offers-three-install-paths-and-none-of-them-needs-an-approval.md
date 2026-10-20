# ADR-0076: macOS offers three install paths, and none of them needs an approval

Date: 2026-10-20

Status: Accepted (extends ADR-0072's Homebrew decision with two more
macOS paths; the formula and the site card go live with the train that
first builds the new tarball).

Amendment 2026-10-20, same day: the landing-page card advertises two
of the three paths (install.sh and the cask) — the formula line was
dropped from the card as one option too many for a first impression.
The formula itself stays in the tap, self-updating and documented
there; the card's macOS story is script, cask, disk image.

## Context

ADR-0072 made Homebrew the macOS channel, and the release ships an
ad-hoc signed DMG (ADR-0058 amendment) because notarization needs a
paid Apple Developer account. The cost of that pairing is paid at
every install:

- Browsers stamp `com.apple.quarantine` on the DMG, and Gatekeeper
  blocks a quarantined ad-hoc bundle: the ordinary right-click → Open
  warning on older macOS, a System Settings detour on newer ones.
- Homebrew historically let a user skip cask quarantine with
  `HOMEBREW_NO_QUARANTINE`; Homebrew 7 removed the opt-out and now
  stamps quarantine on every cask download, unconditionally.
- Worst of all, a quarantined ad-hoc binary executed from a terminal
  does not show a dialog at all — the exec silently hangs while
  Gatekeeper waits for an approval that never comes (observed on
  macOS 27, 2026-10-20). A user who installs the cask and then types
  `epher` sees a frozen prompt.

The paid fix is Developer ID signing plus notarization: the Apple
Developer Program, still $99/year per Apple's enrollment page
(verified 2026-10-20; the membership fee waiver covers only nonprofit,
education, and government entities, expressly not individuals). A
notarized app passes Gatekeeper everywhere: browser download, cask,
otherwise.

The project's goal is a macOS user who installs and runs epher with no
complications and no approvals. The free options all work by one
property: **Gatekeeper only checks quarantined files.** A download
made by a tool (curl, brew formula) carries no quarantine at all.

## Decision

The macOS card on epher.org offers three install paths, all
Apple Silicon (ADR-0025):

### 1. install.sh — one line, no Homebrew, nothing to approve

```sh
curl -fsSL https://raw.githubusercontent.com/upyesp/epher/main/install.sh | sh
```

The script lives at the monorepo root and is served from the main
branch, so it exists as soon as this ADR's train merges — it does not
wait for a release asset. It downloads the latest DMG, installs
`epher.app` into /Applications (falling back to ~/Applications for
non-admin users), links `epher` into ~/.local/bin with a PATH hint,
prints the installed version, and opens the app. Because curl stamps
no quarantine, Gatekeeper never appears — the same property rustup and
Homebrew's own installer rely on. Guard rails: it refuses non-macOS
and non-arm64 machines with clear messages, and if Homebrew already
manages epher it says so and points at `brew upgrade --cask epher`
instead of writing over brew's install. `EPHER_APP_DIR` and
`EPHER_BIN_DIR` override the destinations; the repo's own smoke test
installs into a temp dir with them.

### 2. The Homebrew cask — unchanged

`brew install --cask upyesp/tap/epher` stays as documented in
ADR-0072, quarantine and all: brew users keep brew's bookkeeping, and
first-launch approval remains a one-time cost of that channel.

### 3. The Homebrew formula — just the terminal command

`brew install upyesp/tap/epher` installs the unified binary alone
(CLI, REPL, TUI) from a new release asset, `epher-macos-aarch64.tar.gz`,
which the desktop build's macOS leg tars from the same smoke-tested
binary the bundle ships (stripped, ad-hoc signed like the language
server). Homebrew formulae are never quarantined, so the terminal
command installs and runs with no approval. The tap's self-update
workflow bumps the cask and the formula together on each train, each
with its own digest. The formula's first digest is a placeholder until
the first train that builds the tarball fills it in.

The verify gauntlet (ADR-0073) gains the tarball as a stable asset, so
a train that forgets it fails the release.

## Consequences

- A macOS user with zero patience for Gatekeeper has two
  prompt-free paths (script, formula) that never show an approval —
  and the script's last act is opening the app.
- The browser-DMG path still shows the first-launch approval; only
  notarization can remove that. This ADR deliberately does not close
  the door on it: Developer ID + notarization (~$99, whose notarization
  tickets outlive the membership) would stack on top of all three
  paths and remove the cask's approval too. Revisit when the project
  can justify the fee.
- The script trusts TLS to GitHub and nothing else — no checksum step
  in v1 (the cask pins the DMG digest; the formula pins the tarball's).
  A checksum file attached to each release would harden install.sh;
  deferred until wanted.
- Two failure modes to watch: raw.githubusercontent serving lag (the
  script is served from main, so a bad merge is immediately live), and
  Homebrew changing formula quarantine policy (today it does not
  quarantine formulae).
- The cask's README advice to use `--no-quarantine` is deleted:
  Homebrew 7 removed the flag, and the advice was already stale.

## Rejected alternatives

- **Stripping quarantine inside the cask's postflight** (`xattr -dr`
  after install): works today but fights Homebrew's deliberate design,
  breaks silently on a future brew, and silently downgrades a security
  feature for users who chose brew.
- **Free Apple certificates** (personal team): development-signed
  builds are device-locked with 7-day profiles — not a distribution
  mechanism.
- **The membership fee waiver**: nonprofit/education/government
  entities only; explicitly not individuals.
- **Mac App Store**: requires the same paid account plus sandboxing
  and review.

## Deliberately deferred

- A checksum verification step in install.sh against a per-release
  digest asset.
- Developer ID + notarization (the ~$99 decision, one membership year
  covering roughly two years of releases).
- A `brew audit`/shellcheck CI leg for the tap and install.sh.
