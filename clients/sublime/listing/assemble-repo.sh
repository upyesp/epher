#!/usr/bin/env bash
#
# Assemble the standalone upyesp/LSP-epher repository content from this
# monorepo.
#
# Package Control requires one package per git repository, with the
# package root at the repository root
# (docs/research/sublime-packagecontrol-publishing.md). clients/sublime
# is that package, but it lives inside the epher monorepo, so the listing
# needs a dedicated repository whose root holds exactly the package
# files. This script produces that tree in a target directory; it never
# creates a repository and never pushes anything. The one-time steps
# (create the repo, copy the tree in, push, tag, open the
# sublimelsp/repository PR) are in SUBMISSION.md next to this script.
#
#   ./assemble-repo.sh [target-dir] [--force]
#
# The target defaults to build/LSP-epher at the repository root. The
# script refuses to write into a non-empty target unless --force is
# given. It regenerates epher.tmLanguage from the shared grammar first
# (clients/sublime/sync-assets.py), then copies the package files the
# research doc prescribes, verified against clients/sublime and the
# capture notes in clients/sublime/listing/NOTES.md:
#
#   plugin.py
#   LSP-epher.sublime-settings
#   LSP-epher.sublime-commands
#   epher.tmLanguage
#   README.md          (the listing copy from clients/sublime/README.md)
#   LICENSE
#   .python-version
#
# The README is cut at the "## Maintainer note" divider: clients/sublime/
# README.md carries the monorepo maintenance notes below that divider,
# and only the listing copy above it belongs in the standalone package
# repository. The package root must stay flat, so the cut is done here,
# not left to the user.
#
# `.python-version` is load-bearing: the LSP package runs on Sublime
# Text's modern python host, and without the file the client loads on
# the legacy 3.3 host where `from LSP.plugin import ...` fails
# (NOTES.md, checked live on ST build 4215).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
SRC="$ROOT/clients/sublime"

TARGET=""
FORCE=0
for arg in "$@"; do
  case "$arg" in
    --force) FORCE=1 ;;
    -*) echo "unknown option: $arg" >&2; exit 2 ;;
    *) TARGET="$arg" ;;
  esac
done
TARGET="${TARGET:-$ROOT/build/LSP-epher}"

if [ "$TARGET" = "/" ]; then
  echo "refusing to write to /" >&2
  exit 2
fi

if [ -e "$TARGET" ] && [ -n "$(ls -A "$TARGET" 2>/dev/null || true)" ]; then
  if [ "$FORCE" -ne 1 ]; then
    echo "target exists and is not empty: $TARGET" >&2
    echo "rerun with --force to replace its contents" >&2
    exit 1
  fi
  rm -rf "$TARGET"
fi
mkdir -p "$TARGET"

# The grammar is generated from clients/shared; regenerate it so the
# assembled repository can never carry a stale copy.
python3 "$SRC/sync-assets.py"

cp "$SRC/plugin.py" "$TARGET/plugin.py"
cp "$SRC/LSP-epher.sublime-settings" "$TARGET/LSP-epher.sublime-settings"
cp "$SRC/LSP-epher.sublime-commands" "$TARGET/LSP-epher.sublime-commands"
cp "$SRC/epher.tmLanguage" "$TARGET/epher.tmLanguage"
cp "$SRC/.python-version" "$TARGET/.python-version"
cp "$ROOT/LICENSE" "$TARGET/LICENSE"

# The monorepo README ends with maintainer notes below a "## Maintainer
# note" divider; only the listing copy above it ships in the package
# repository. Trim the divider and any trailing blank lines.
awk '
  { buf[NR] = $0 }
  END {
    last = NR
    for (i = 1; i <= NR; i++) {
      if (buf[i] ~ /^## Maintainer note/) { last = i - 1; break }
    }
    while (last > 0 && (buf[last] == "" || buf[last] == "---")) last--
    for (i = 1; i <= last; i++) print buf[i]
  }
' "$SRC/README.md" > "$TARGET/README.md"

# A stray __pycache__ (say, from a local syntax check) must never ship:
# Package Control's reviewer flags compiled files, and the archive is
# built with git-archive from this tree.
find "$TARGET" -type d -name __pycache__ -prune -exec rm -rf {} +

EXPECTED=(
  plugin.py
  LSP-epher.sublime-settings
  LSP-epher.sublime-commands
  epher.tmLanguage
  README.md
  LICENSE
  .python-version
)
for f in "${EXPECTED[@]}"; do
  if [ ! -f "$TARGET/$f" ]; then
    echo "missing expected file: $f" >&2
    exit 1
  fi
done

echo "assembled $TARGET:"
ls -A "$TARGET"
echo
echo "next: create the public upyesp/LSP-epher repository, copy this tree"
echo "into its root, commit, and push. Then tag a semver release"
echo "(for example: git tag v0.5.57 && git push origin main --tags) and"
echo "open the sublimelsp/repository PR. See SUBMISSION.md."
