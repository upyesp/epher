#!/bin/bash
# epher's macOS installer (ADR-0076): one line, no Homebrew, no prompts.
#
#   curl -fsSL https://raw.githubusercontent.com/upyesp/epher/main/install.sh | sh
#
# Downloads the latest release DMG, installs epher.app into /Applications,
# and links the `epher` terminal command into ~/.local/bin. Because curl
# (unlike a browser) stamps no com.apple.quarantine flag, macOS Gatekeeper
# never asks the user to approve anything — this is the same property
# rustup and Homebrew itself rely on.
#
# Exits 0 only when the app is installed and launched. Apple Silicon only
# (ADR-0025). EPHER_APP_DIR and EPHER_BIN_DIR override the install
# destinations (the repo's own smoke tests install into a temp dir).

set -eu

REPO="upyesp/epher"
APP_NAME="epher.app"
APP_DIR="${EPHER_APP_DIR:-/Applications}"
BIN_DIR="${EPHER_BIN_DIR:-$HOME/.local/bin}"
DMG_URL="https://github.com/$REPO/releases/latest/download/epher-macos-aarch64.dmg"

say()  { printf '==> %s\n' "$*"; }
fail() { printf 'error: %s\n' "$*" >&2; exit 1; }

[ "$(uname -s)" = "Darwin" ] || fail "this installer is for macOS. Linux users: epher.org has apt, dnf, Snap, and Flatpak instructions."
[ "$(uname -m)" = "arm64" ] || fail "epher on macOS is Apple Silicon only (this Mac reports $(uname -m); see ADR-0025 at epher.org)."

# If Homebrew manages epher, keep it that way: writing over a cask-owned
# app leaves brew's bookkeeping pointing at bits it did not install.
# An explicit EPHER_APP_DIR overrides this routing (the repo's smoke
# tests install into a temp dir on brew-managed machines).
if [ -z "${EPHER_APP_DIR:-}" ] && command -v brew >/dev/null 2>&1 && brew list --cask epher >/dev/null 2>&1; then
  say "Homebrew already manages epher on this Mac."
  say "    Upgrade it with:  brew upgrade --cask epher"
  exit 0
fi

# /Applications is writable by admin-group users without sudo; a
# non-admin user installs into their own Applications folder instead.
if [ "$APP_DIR" = "/Applications" ] && [ ! -w /Applications ]; then
  APP_DIR="$HOME/Applications"
  say "==> /Applications is not writable for this user; installing to ~/Applications"
fi
mkdir -p "$APP_DIR"

TMP="$(mktemp -d)"
MOUNT="$TMP/mnt"
cleanup() {
  hdiutil detach "$MOUNT" -quiet >/dev/null 2>&1 || true
  rm -rf "$TMP"
}
trap cleanup EXIT
trap 'cleanup; exit 130' INT
trap 'cleanup; exit 143' TERM

say "Downloading epher (latest release)..."
DMG="$TMP/epher.dmg"
curl -fsSL -o "$DMG" "$DMG_URL" || fail "the download failed - check your connection and run the command again."

say "Installing epher.app into $APP_DIR..."
mkdir -p "$MOUNT"
hdiutil attach "$DMG" -nobrowse -readonly -mountpoint "$MOUNT" -quiet \
  || fail "macOS could not open the downloaded disk image."
[ -d "$MOUNT/$APP_NAME" ] || fail "the disk image did not contain $APP_NAME - please report this at github.com/$REPO/issues."
# Replace any pre-brew copy wholesale: a stale bundle from an older
# manual install must not leave mixed-version files behind.
rm -rf "${APP_DIR:?}/$APP_NAME"
ditto "$MOUNT/$APP_NAME" "$APP_DIR/$APP_NAME"

say "Linking the terminal command into $BIN_DIR..."
mkdir -p "$BIN_DIR"
ln -sf "$APP_DIR/$APP_NAME/Contents/MacOS/epher" "$BIN_DIR/epher"
case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *)
    say ""
    say "One more step: your terminal does not know the folder"
    say "    $BIN_DIR"
    say "yet. Run these two lines once (then open a new terminal window):"
    say "    echo 'export PATH=\"$BIN_DIR:\$PATH\"' >> ~/.zshrc"
    say "    source ~/.zshrc"
    say ""
    ;;
esac

INSTALLED_VERSION="$("$APP_DIR/$APP_NAME/Contents/MacOS/epher" --version 2>/dev/null || echo "epher")"
say "Installed $INSTALLED_VERSION."

say "Opening epher..."
open "$APP_DIR/$APP_NAME"
say "Done. epher is in $APP_DIR - no approval needed on first launch."
