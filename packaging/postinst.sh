#!/bin/sh
# epher deb/rpm post-install: refresh the freedesktop icon caches so every
# desktop environment picks up the installed hicolor icon set immediately.
# Some menus keep a stale cache otherwise and show a generic icon (reported
# on Linux Mint). Also refresh the mime and desktop databases so the
# application/x-epher file association (the .desktop MimeType= and
# /usr/share/mime/packages/epher.xml) is live right after install and a
# double-clicked .epher file opens the app. Failures are non-fatal: the
# caches are optimizations.
set -e
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q /usr/share/icons/hicolor || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database -q /usr/share/applications || true
fi
if command -v update-mime-database >/dev/null 2>&1; then
  update-mime-database /usr/share/mime || true
fi
exit 0
