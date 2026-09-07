# ADR-0061: Linux installs come from apt, dnf, Flathub, Snap, and AUR

Date: 2026-09-05

Status: Accepted (extends ADR-0058's installer story with repository and
store channels; amends the website's Linux downloads card).
Amended 2026-09-06: the AUR channel is removed before its first
publish — see the Amendment at the end of this record.

## Context

Linux shipped as three direct downloads — `.deb`, `.rpm`, `.AppImage` —
x86_64 only (ADR-0025 dropped the Intel mac build; Linux never had an
ARM build at all). Each new machine meant: visit the site, download a
file, install it by hand, and repeat for every release. Four audiences
had no good answer:

- **Debian/Ubuntu and Fedora users** who expect `apt install epher` and
  `dnf install epher`, with updates arriving through the package
  manager they already trust.
- **ARM64 desktops and servers** (Asahi, Graviton, ARM laptops) — no
  artifact existed for their architecture.
- **Immutable distros** (Fedora Silverblue, openSUSE MicroOS, NixOS) —
  the distro's own package managers are the only sanctioned install
  path; direct downloads fight the OS.
- **Arch users**, served only by a raw AppImage.

GitHub's free arm64 Linux runners (public repos) make native aarch64
builds nearly free, and the project accepts a high per-release
maintenance budget for store presence.

## Decision

**One package, every Linux channel.** The same full desktop app (GUI,
CLI, TUI — one binary, ADR-0011) ships through five channels (six at
acceptance; the AUR was removed before going live — see the Amendment);
the direct downloads stay exactly as they are.

- **apt repository at https://epher.org/apt** — hosted on GitHub Pages
  (the same Pages deployment that serves the site: the release workflow
  publishes the repository trees to the `gh-pages` branch, and the site
  build checks them out into the deployment). Single `stable` suite,
  `main` component, `binary-amd64` and `binary-arm64`. Signed with one
  RSA-4096 GPG key whose private half lives in Actions secrets and
  whose public half is served at `https://epher.org/apt/KEY.gpg` and
  pushed to keyservers. Users add a downloaded deb822 `.sources` file —
  no curl-piped scripts.
- **dnf repository at https://epher.org/rpm** — one repository keyed by
  `$basearch` (x86_64, aarch64); the rpms depend on webkit2gtk-4.1 by
  name, which every current Fedora provides, so there are no
  per-release directories until a Fedora actually breaks the
  dependency. Signed with the same key. Users drop a `.repo` file.
- **Retention: latest only.** Each publish replaces the repository's
  contents with the current release; the repository is an install
  path, not an archive — GitHub Releases remain the permanent archive
  and there is no in-repo rollback.
- **Flathub** — `com.epher.Desktop` (the Tauri identifier), MIT
  metainfo, WebKitGTK from the Flathub shared modules on the GNOME
  runtime, aarch64 built by Flathub from the same manifest. Inside the
  sandbox the app cannot write to PATH, so the "Install the epher
  command" button's job is done by exporting an `epher` command from
  the Flatpak instead; the guide documents that the script
  collection's installed paths differ inside the sandbox. The manifest
  lives in-repo (`packaging/flatpak/`); after the one-time human
  submission PR is accepted, the release workflow commits version
  bumps straight to the Flathub repository (direct pushes trigger
  Flathub's bot builds).
- **Snap** — name `epher`, strict confinement (the `home` interface
  covers `~/.epher`, portals cover files), the GNOME extension carries
  the webview runtime; amd64 and arm64. The snap repacks the release
  `.deb` (the store allows binary repacks) so the two artifacts cannot
  drift; published with store credentials in Actions secrets.
- **ARM64 builds** — a `ubuntu-22.04-arm` matrix leg builds the deb,
  rpm, and AppImage for aarch64 with the same glibc 2.35 floor as the
  x86_64 leg (the x86_64 leg pins ubuntu-22.04 for exactly that
  floor), so both architectures support the same distro set: Debian
  12, Ubuntu 22.04+, and every newer release.
- **Website** — the Linux downloads card becomes five tabs (Debian/Ubuntu,
  Fedora, Any distro, Flatpak, Snap); the direct downloads stay
  inside their tabs and the macOS/Windows cards are unchanged.
- **Automation and gates** — the release workflow publishes the apt and
  dnf repositories and the snap automatically (GPG and Snap Store
  credentials in secrets) and scripts the Flathub bump. The
  repository publish is gated: before repodata goes live, a Debian 12
  and a Fedora container add the freshly built repository, install
  `epher` from it, and run `epher "2 + 2"`. Jobs whose secrets or
  remote repositories do not exist yet skip with a notice instead of
  failing the release.

## Consequences

- `apt install epher` and `dnf install epher` work on Debian 12+,
  Ubuntu 22.04+, and current Fedora, on x86_64 and aarch64, with
  updates delivered by the package manager. `snap refresh` and
  `flatpak update` deliver store installs the same way.
- The site's Linux story is tab-per-distro; the guide's Linux sections
  name the repository setup first and the direct downloads second.
- The GPG key becomes critical infrastructure: losing it means
  rotating users' `.sources`/`.repo` files. It exists in exactly one
  place (Actions secrets) plus the published public half.
- Latest-only repositories trade in-repo rollback for simplicity; the
  GitHub Releases page is the rollback path.
- Flathub's first submission is human work (account, PR, review
  latency, screenshots); everything after it is automated.
- The snap-repacks-the-deb coupling means a broken deb is a broken
  snap for that release — the container smoke gate now guards both.
- More CI legs and three more store/registrar relationships to keep
  healthy; each degrades gracefully (skip-with-notice) when its
  credentials are absent, so packaging work never blocks a release.
EOF
echo written
## Amendment — the AUR channel is removed (2026-09-06)

The AUR never went live: the submission flow demands one-time human
steps on machines the maintainer does not routinely use (an AUR
account, an SSH key minted for CI, a first manual push), and each
release would then carry a third-party store relationship to keep
healthy. Arch users are already served by the channels that remain —
the AppImage explicitly targets "any distro, Arch included" (x86_64
and aarch64), and Flatpak and Snap cover package-manager-minded
installs — so a community package maintained out-of-tree no longer
earns its place in this project's release surface.

Removed with this amendment:

- `packaging/aur/` (the PKGBUILD) — deleted; `epher-bin` is not
  published and the name stays unregistered.
- The `store-bumps` AUR leg, the `AUR_SSH_KEY` secret, and the `aur`
  mode of `scripts/bump-stores.sh` — the job now bumps Flathub only.
- The website's Arch tab and the `linux-aur` /
  `linux-tab-arch` strings in all eight locale catalogs; the Linux
  card has five tabs (Debian/Ubuntu, Fedora, Any distro, Flatpak,
  Snap).
- The guide's AUR sentence and `paru -S epher-bin` command in all
  eight locales.

Revisiting this is a new decision: if Arch users ask for a
package-manager install, the honest paths are Flathub/Snap first, and
an out-of-tree community `epher-bin` (maintained by others under AUR
policy) second.
