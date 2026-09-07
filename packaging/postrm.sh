#!/bin/sh
# epher postrm — clear per-user data on uninstall (ADR-0025).
#
# History, functions, constants, and settings live in ~/.epher per user.
# The user asked that uninstalling clears them so a reinstall starts with
# a clean slate; this runs for every human user's home directory.
#
# deb: invoked with $1 = remove | purge | upgrade | failed-upgrade | ...
# rpm: %postun scriptlet, $1 = number of packages of this name remaining
#      after the removal (0 means this was the last one).
case "$1" in
  remove|purge|0) ;;
  *) exit 0 ;;
esac

for home in /home/* /root; do
  [ -d "$home/.epher" ] && rm -rf "$home/.epher"
done
exit 0
