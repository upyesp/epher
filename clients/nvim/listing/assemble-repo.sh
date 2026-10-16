#!/usr/bin/env bash
#
# Assemble the standalone upyesp/epher.nvim repository content from this
# monorepo.
#
# Plugin discovery in the Neovim ecosystem points at one repository per
# plugin, with the plugin root at the repository root and Neovim named
# on the landing page (docs/research/neovim-publishing.md; the
# awesome-neovim rejection of PR #2532 was exactly this: the linked
# monorepo root never mentions Neovim). clients/nvim is that plugin,
# but it lives inside the epher monorepo, so the listing needs a
# dedicated repository whose root holds exactly the plugin files. This
# script produces that tree in a target directory; it never creates a
# repository and never pushes anything. The one-time steps (create the
# repo, copy the tree in, push) are in SUBMISSION.md next to this
# script; the per-train push is .github/workflows/epher-nvim-publish.yml.
#
#   ./assemble-repo.sh [target-dir] [--force]
#
# The target defaults to build/epher.nvim at the repository root. The
# script refuses to write into a non-empty target unless --force is
# given. It verifies the synced runtime first (clients/nvim/
# sync-runtime.py --check: the ftdetect, ftplugin, and syntax files are
# clients/vim copies and must not be stale), then copies the plugin
# files the research doc prescribes, verified against clients/nvim:
#
#   ftdetect/epher.vim
#   ftplugin/epher.vim
#   syntax/epher.vim
#   lua/epher.lua
#   lua/epher/health.lua
#   doc/epher.txt
#   images/             (the README's screenshots, relative links)
#   README.md           (the listing copy from clients/nvim/README.md)
#   LICENSE
#
# The README is cut at the "## Maintainer note" divider: clients/nvim/
# README.md carries the monorepo maintenance notes below that divider,
# and only the listing copy above it belongs in the standalone plugin
# repository. The image links in the listing copy are relative
# (images/...), which resolve both inside clients/nvim on GitHub and
# at the repository root of the mirror.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
SRC="$ROOT/clients/nvim"

TARGET=""
FORCE=0
for arg in "$@"; do
  case "$arg" in
    --force) FORCE=1 ;;
    -*) echo "unknown option: $arg" >&2; exit 2 ;;
    *) TARGET="$arg" ;;
  esac
done
TARGET="${TARGET:-$ROOT/build/epher.nvim}"

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

# The ftdetect, ftplugin, and syntax files are synced copies of
# clients/vim; refuse to assemble a stale tree rather than ship one.
python3 "$SRC/sync-runtime.py" --check

cp -a "$SRC/ftdetect" "$TARGET/ftdetect"
cp -a "$SRC/ftplugin" "$TARGET/ftplugin"
cp -a "$SRC/syntax" "$TARGET/syntax"
mkdir -p "$TARGET/lua/epher"
cp "$SRC/lua/epher.lua" "$TARGET/lua/epher.lua"
cp "$SRC/lua/epher/health.lua" "$TARGET/lua/epher/health.lua"
cp -a "$SRC/doc" "$TARGET/doc"
cp -a "$SRC/images" "$TARGET/images"
cp "$ROOT/LICENSE" "$TARGET/LICENSE"

# The monorepo README ends with maintainer notes below a "## Maintainer
# note" divider; only the listing copy above it ships in the plugin
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

EXPECTED=(
  ftdetect/epher.vim
  ftplugin/epher.vim
  syntax/epher.vim
  lua/epher.lua
  lua/epher/health.lua
  doc/epher.txt
  images/editor.png
  images/demo.gif
  README.md
  LICENSE
)
for f in "${EXPECTED[@]}"; do
  if [ ! -f "$TARGET/$f" ]; then
    echo "missing expected file: $f" >&2
    exit 1
  fi
done

echo "assembled $TARGET:"
(cd "$TARGET" && find . -type f | sort)
echo
echo "next: the upyesp/epher.nvim repository carries this tree at its"
echo "root. The per-train push is .github/workflows/epher-nvim-publish.yml"
echo "(gated on the NVIM_PACKAGE_TOKEN secret); the one-time setup steps"
echo "are in SUBMISSION.md next to this script."
