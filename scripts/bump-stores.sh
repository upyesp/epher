#!/usr/bin/env bash
# ADR-0061: version-bump one store repository for a release. Called by
# the release workflow's store-bumps job:
#   scripts/bump-stores.sh flathub v0.5.33 flathub
# The clone is already present as ./<name> (the caller fetched it with
# the right SSH key) and the working directory is the repo root. Skips
# with a notice when the target is already at this version (a re-run).
set -euo pipefail

TARGET=$1
VERSION=$2       # like v0.5.33
DIR=$3
VERSION_NUM=${VERSION#v}
DATE=$(date +%F)

cd "$DIR"
git config user.name "epher release"
git config user.email "release@epher.org"

case "$TARGET" in
  flathub)
    # The manifest's git tag and the metainfo release entry.
    sed -i "s|^        tag: v.*|        tag: $VERSION|" com.epher.Desktop.yml
    if grep -q "release version=\"$VERSION_NUM\"" com.epher.Desktop.metainfo.xml; then
      echo "flathub already at $VERSION_NUM"
      exit 0
    fi
    python3 - "$VERSION_NUM" "$DATE" <<'PY'
import sys
v, date = sys.argv[1], sys.argv[2]
p = "com.epher.Desktop.metainfo.xml"
s = open(p).read()
entry = f'    <release version="{v}" date="{date}">\n      <url type="details">https://github.com/upyesp/epher/releases</url>\n    </release>\n'
marker = "  <releases>\n"
s = s.replace(marker, marker + entry, 1)
open(p, "w").write(s)
PY
    git add -A
    git commit -m "epher $VERSION_NUM"
    git push origin master
    ;;
  *)
    echo "unknown target: $TARGET (flathub)" >&2
    exit 1
    ;;
esac
