#!/bin/zsh
# The accessibility suite's authorised context, opened by run-a11y.sh.
#
# This file runs INSIDE Terminal.app on purpose. TCC judges the responsible
# process, and Terminal is Apple-signed and therefore exempt, so the System
# Events / VoiceOver Apple Events and the Accessibility UI scripting the
# harnesses need all work here without any grant of our own. See
# docs/accessibility.md ("Automated runs") for the full explanation.
set -u
STATE=/tmp/epher-a11y-trigger.state
REPO="${0:A:h:h:h}"
if [ ! -f "$REPO/scripts/a11y/automation-run.sh" ]; then
  REPO="$(cat /tmp/epher-a11y-run.repo 2>/dev/null)"
fi

echo "running" > "$STATE"

# This window's unique id: Terminal reuses pty names across windows
# (/dev/ttys000 turns up more than once), so the close has to name the
# window, not its tty.
window_id="$(/usr/bin/osascript -e 'tell application "Terminal" to get id of front window' 2>/dev/null)"
# Recorded so the run's own window can be checked afterwards (the id is the
# CoreGraphics window number, visible to tools that need no TCC grant).
echo "${window_id:-}" > /tmp/epher-a11y-window-id

# A sleeping display or a dark-wake stalls the GUI legs; hold the machine
# awake for exactly as long as this run lives.
/usr/bin/caffeinate -dims -w $$ >/dev/null 2>&1 &

bash "$REPO/scripts/a11y/automation-run.sh"
rc=$?

# No residual active sessions: VoiceOver must not outlive the run either.
/usr/bin/osascript -e 'tell application "VoiceOver" to quit' >/dev/null 2>&1
sleep 1
echo "finished rc=$rc" > "$STATE"

# Hand the borrowed window back. This script IS the window's process, so the
# close has to come from a detached process that outlives it.
/usr/bin/nohup /bin/bash "$REPO/scripts/a11y/terminal-teardown.sh" "$window_id" \
  >>/tmp/epher-a11y-teardown.log 2>&1 &

exit $rc
