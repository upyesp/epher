#!/bin/bash
# Close the Terminal window the a11y run borrowed, and Terminal itself when
# that run is the one that started the app. Detached on purpose: it must
# outlive the window's own process, which is what makes the close clean
# instead of a "processes are running" prompt.
#
# Launched by scripts/a11y/terminal-runner.command.
set -u
window_id="${1:-}"
was_running=$(cat /tmp/epher-a11y-term-was-running 2>/dev/null || echo 1)

# Give the window's shell a moment to be gone before asking to close it.
sleep 3

case "$window_id" in
  ""|"missing value") ;;
  *)
    /usr/bin/osascript -e \
      "tell application \"Terminal\" to close (first window whose id is $window_id)" \
      >/dev/null 2>&1
    ;;
esac

sleep 1

if [ "$was_running" = "0" ]; then
  /usr/bin/osascript -e \
    'tell application "Terminal" to if (count windows) is 0 then quit' \
    >/dev/null 2>&1
fi
