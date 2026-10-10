# ADR-0072: macOS installs come from Homebrew

Date: 2026-10-10

Status: Accepted (extends ADR-0061's channel story to macOS; amends the
website's macOS downloads card).

## Context

Linux ships through real package channels: apt and dnf repositories,
Snap, Flatpak, and direct packages (ADR-0061). macOS had none of that.
The only distribution was the DMG on the releases page: visit the site,
download `epher-macos-aarch64.dmg`, drag the app to Applications, use
the button inside to install the `epher` terminal command, and repeat
the whole ritual for every release.

Two earlier decisions shape the space:

- **ADR-0025** dropped the Intel mac build. macOS is Apple Silicon
  only, so a package channel can be single-architecture.
- **ADR-0058 (amendment)** ships the app ad-hoc signed, because
  notarization needs a paid Apple account. A quarantined unsigned
  bundle fails Gatekeeper hard ("damaged", no override); an ad-hoc
  signature passes and leaves the ordinary right-click-to-open
  warning. The Sublime work (the auto-managed server, 2026-10) proved
  the same point from the other side: a download made by a tool
  instead of a browser sidesteps quarantine entirely.

Homebrew is where macOS users look first, and a tap is the channel a
small project can run itself: no review, no notability bar, no fees.

## Decision

A Homebrew **cask** in a new tap repository, `upyesp/homebrew-tap`:

```
brew install --cask upyesp/tap/epher
```

- The cask wraps the DMG we already build (`epher-macos-aarch64.dmg`).
  No new artifact, no new build job: the release pipeline is unchanged.
- The cask installs `epher.app` into Applications **and** links the
  `epher` binary onto the PATH (`binary "#{appdir}/epher.app/Contents/
  MacOS/epher"`), which makes the button inside the app redundant but
  keeps it harmless for users who never touch brew.
- `depends_on arch: :arm64`: an Intel mac gets Homebrew's clean
  "no build exists" answer instead of a broken install, mirroring the
  Sublime plugin's arch-map behavior.
- The tap updates itself: a scheduled workflow *inside the tap repo*
  compares the cask version against the monorepo's latest release,
  recomputes the DMG sha256, and commits the bump. No cross-repo
  secret, no monorepo-side workflow, at most an hour of lag behind a
  train tag (the same lag Package Control has).
- The website's macOS card leads with the brew command; the DMG stays
  one click below as the fallback for no-Homebrew machines.
- The first-launch note stays: Homebrew quarantines cask downloads
  like a browser does, so the ad-hoc-signature right-click-to-open
  dance applies to the brew path too. (Users who prefer it can run
  `brew install --cask --no-quarantine upyesp/tap/epher`.)

## Consequences

- macOS gains `brew upgrade` as the update story, at parity with the
  Linux channels.
- The tap is a second repository that must not drift: its self-update
  workflow is the drift guard, and the cask is generated from the
  release tag, never hand-edited.
- Promotion to homebrew-core is deliberately not pursued: it needs
  notability we do not have and a build-from-source formula story we
  do not want. The tap is the channel.
- The `epher-lsp` server binary keeps its own distribution (the .gz
  release asset, the editor plugins that manage it themselves). The
  cask does not carry it: the DMG does not contain it.

## Amendment

2026-10-10, same day: the site's macOS card, the IDE extension page's
Sublime and Neovim sections, and all eight locale catalogs updated in
the same train (0.5.78) so no page advertises a channel that does not
exist or omits one that does.
