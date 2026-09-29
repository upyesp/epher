#!/usr/bin/env bash
#
# Build the JetBrains Marketplace media set from the four captures.
#
# The marketplace's listing docs put the minimum recommended screenshot
# size at 1200 x 760 pixels, 1280 x 800 ideal, and the admin panel's
# Media section is where they go
# (docs/research/jetbrains-marketplace-publishing.md). The Mint VM
# captures are 901 x 568, below that floor, so this derives the media
# copies at 1280 wide (1280 x 807, the captures' own ratio) with a
# gentle unsharp, then zips them for the upload.
#
# Scaling is the fallback, not the ideal: a native capture at 1200 x 760
# or larger is crisper, and CAPTURE-GUIDE.md carries the sizing note for
# the next shoot.
#
#   ./make-media.sh [target-dir]
#
# The target defaults to build/jetbrains-media at the repository root.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
TARGET="${1:-$ROOT/build/jetbrains-media}"

SHOTS=(editor hover completion results)
WIDTH=1280

if ! command -v convert >/dev/null 2>&1; then
  echo "ImageMagick's convert is required" >&2
  exit 2
fi

mkdir -p "$TARGET"
FILES=()
for shot in "${SHOTS[@]}"; do
  src="$SCRIPT_DIR/$shot.png"
  if [ ! -f "$src" ]; then
    echo "missing capture: $src" >&2
    exit 1
  fi
  convert "$src" -filter Lanczos -resize "${WIDTH}x" -unsharp 0x0.75+0.7+0.02 "$TARGET/$shot.png"
  identify -format '  %f %wx%h\n' "$TARGET/$shot.png"
  FILES+=("$TARGET/$shot.png")
done

zip -q -j "$TARGET/jetbrains-media-${WIDTH}.zip" "${FILES[@]}"
echo
echo "upload the four PNGs from $TARGET in this order: ${SHOTS[*]}"
echo "archive: $TARGET/jetbrains-media-${WIDTH}.zip"
