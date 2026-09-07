#!/bin/sh
# epher deb/rpm post-install: refresh the freedesktop icon caches so every
# desktop environment picks up the installed hicolor icon set immediately.
# Some menus keep a stale cache otherwise and show a generic icon (reported
# on Linux Mint). Failures are non-fatal — the cache is an optimization.
set -e
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database -q /usr/share/applications || true
fi
exit 0
