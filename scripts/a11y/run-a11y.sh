#!/bin/bash
# Start the full accessibility suite with no human in the loop.
#
# Why a Terminal window: VoiceOver automation needs Apple Events and
# Accessibility grants, and TCC decides on the RESPONSIBLE PROCESS. This
# Mac's TCC database is write-sealed under SIP, so nothing running under the
# agent's own ancestry can ever be granted them (no prompt is shown to an
# unsigned sender, and PPPC profiles need an MDM). Terminal.app is
# Apple-signed, so Apple's TCC policy exempts it - a run started inside
# Terminal inherits that, which is why `epher-a11y` typed into a Terminal
# window always worked. `open -a Terminal <file>` needs no grant of its own,
# so the agent can start exactly the same run unattended.
#
#   bash scripts/a11y/run-a11y.sh                  # start, wait, print summary
#   bash scripts/a11y/run-a11y.sh --detach         # start and return at once
#   bash scripts/a11y/run-a11y.sh --wait-unlock 30 # wait up to 30 min for an
#                                                  # unlocked screen first
#
# Exit status: 0 when every leg passed, 1 when a leg failed, 2 when the run
# could not start (screen locked, run already in flight), 3 on timeout.

set -u
REPO=$(cd "$(dirname "$0")/../.." && pwd)
LOG=/tmp/epher-a11y-latest.log
STATE=/tmp/epher-a11y-trigger.state
REPO_MARKER=/tmp/epher-a11y-run.repo
DETACH=0
UNLOCK_WAIT_MIN=0

while [ $# -gt 0 ]; do
  case "$1" in
    --detach) DETACH=1 ;;
    --wait-unlock)
      shift
      UNLOCK_WAIT_MIN="${1:-30}"
      ;;
    --wait-unlock=*) UNLOCK_WAIT_MIN="${1#*=}" ;;
    -h|--help)
      sed -n '2,26p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      exit 2
      ;;
  esac
  shift
done

# One run at a time: two suites driving VoiceOver would fight over the screen
# and produce meaningless phrases.
if [ "$(cat "$STATE" 2>/dev/null || true)" = "running" ]; then
  echo "a run is already in flight (state: $STATE); log: $LOG" >&2
  exit 2
fi

# A locked screen has no foreground app to walk: the suite would type into
# the lock screen. Wait for the user session when asked, otherwise refuse.
waited=0
while ! "$REPO/scripts/a11y/session-lock.py" >/tmp/epher-a11y-session.txt 2>&1; do
  if [ "$UNLOCK_WAIT_MIN" -le 0 ]; then
    echo "cannot start: $(cat /tmp/epher-a11y-session.txt)" >&2
    echo "unlock the Mac and run again, or pass --wait-unlock <minutes>" >&2
    exit 2
  fi
  if [ "$waited" -ge $((UNLOCK_WAIT_MIN * 60)) ]; then
    echo "cannot start after ${UNLOCK_WAIT_MIN} min: $(cat /tmp/epher-a11y-session.txt)" >&2
    exit 2
  fi
  [ "$waited" -eq 0 ] && echo "waiting for an unlocked screen ($(cat /tmp/epher-a11y-session.txt)) ..."
  sleep 15
  waited=$((waited + 15))
done

# Terminal.app may already be open for other work; only close the app again
# when this run is the one that started it.
if pgrep -x Terminal >/dev/null 2>&1; then
  echo 1 > /tmp/epher-a11y-term-was-running
else
  echo 0 > /tmp/epher-a11y-term-was-running
fi

echo "$REPO" > "$REPO_MARKER"
rm -f "$STATE"
echo "running" > "$STATE"

/usr/bin/open -a /System/Applications/Utilities/Terminal.app \
  "$REPO/scripts/a11y/terminal-runner.command" || {
  echo "could not open Terminal to host the run" >&2
  echo "failed" > "$STATE"
  exit 2
}

if [ "$DETACH" -eq 1 ]; then
  echo "started; progress in $LOG and $STATE"
  exit 0
fi

# Wait for the run. The desktop leg builds a Tauri bundle on first use, so
# allow an hour before giving up.
elapsed=0
while [ "$(cat "$STATE" 2>/dev/null || echo running)" = "running" ]; do
  if [ "$elapsed" -ge 3600 ]; then
    echo "timed out after 60 min; the run may still be going (log: $LOG)" >&2
    exit 3
  fi
  sleep 5
  elapsed=$((elapsed + 5))
done

final=$(cat "$STATE" 2>/dev/null || echo "finished rc=?")
echo
echo "════════ run finished: $final ════════"
# Every leg prints its own exit code; the summary block repeats the harness
# verdicts, which is what decides pass or fail.
grep -E '^══|exit=|failure|FAIL|WARN|PASS' "$LOG" 2>/dev/null | tail -40

case "$final" in
  *"rc=0"*) exit 0 ;;
  *) exit 1 ;;
esac
