#!/bin/bash
# Full accessibility suite. Do not run this directly: start it with
# scripts/a11y/run-a11y.sh, which hosts it inside Terminal.app because only
# a Terminal-hosted run holds the Apple Events and Accessibility grants the
# VoiceOver legs need (TCC exempts Apple-signed Terminal; see
# docs/accessibility.md, "Automated runs"). That trigger needs no human and
# no grant of its own, so the whole suite runs unattended.
#
# The log is /tmp/epher-a11y-latest.log.

set -u
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
LOG=/tmp/epher-a11y-latest.log
REPO=/Users/pete/Developer/code/epher

mkdir -p "$(dirname "$LOG")"
{
  echo "epher accessibility run started $(date '+%Y-%m-%d %H:%M:%S')"
  echo "runner: ${TERM_PROGRAM:-unknown} (responsible process for TCC)"
  echo
} > "$LOG"

cd "$REPO" || exit 1

# Rebuild the web dist first: legs 2 and 5 must always test current code.
echo "══ leg 0/5: rebuild the web dist (trunk) ══" >> "$LOG"
npm run build:pwa >> "$LOG" 2>&1
echo "trunk exit=$?" >> "$LOG"

echo "══ leg 1/5: website audit ══" >> "$LOG"
node scripts/a11y/site-audit.mjs >> "$LOG" 2>&1
echo "site-audit exit=$?" >> "$LOG"

echo "══ leg 2/5: PWA audit ══" >> "$LOG"
node scripts/a11y/audit.mjs >> "$LOG" 2>&1
echo "audit exit=$?" >> "$LOG"

echo "══ leg 3/5: native terminals (CLI + TUI) ══" >> "$LOG"
node scripts/a11y/native-terminals.mjs >> "$LOG" 2>&1
echo "native-terminals exit=$?" >> "$LOG"

echo "══ leg 4/5: VoiceOver web suite (Chrome) ══" >> "$LOG"
node scripts/a11y/voiceover-audit.mjs --vo >> "$LOG" 2>&1
VO_EXIT=$?
echo "voiceover-audit exit=$VO_EXIT" >> "$LOG"
if [ "$VO_EXIT" -ne 0 ]; then
  # The VO walk has a known timing flake: the expression field can be
  # missed when VoiceOver's speech backlog lags the tab presses on a
  # cold Chrome. One retry, both outputs kept for comparison.
  echo "══ leg 4 retry (walk timing flake) ══" >> "$LOG"
  sleep 10
  node scripts/a11y/voiceover-audit.mjs --vo >> "$LOG" 2>&1
  echo "voiceover-audit retry exit=$?" >> "$LOG"
fi

echo "══ leg 5/5: desktop GUI under VoiceOver (Tauri/WebKit) ══" >> "$LOG"
# The desktop surface can only be exercised through a real .app bundle:
# the unified console binary never opens a GUI (ADR-0011). Build the
# debug bundle so this leg always tests current code and the current
# web dist, quit any installed instance (both processes are "epher"),
# then hand the bundle to the harness.
BUNDLE="$REPO/target/debug/bundle/macos/epher.app"
if [ ! -d "$BUNDLE" ] || [ "$REPO/crates/web/dist" -nt "$BUNDLE" ]; then
  (cd crates/tauri-app/src-tauri && npx --yes @tauri-apps/cli build --debug --bundles app >> "$LOG" 2>&1)
fi
if [ ! -d "$BUNDLE" ]; then
  echo "debug bundle missing; see log above" >> "$LOG"
  echo "native-desktop-vo exit=1 (no bundle)" >> "$LOG"
else
  killall epher 2>/dev/null
  sleep 2
  export EPHER_APP="$BUNDLE"
  node scripts/a11y/native-desktop-vo.mjs >> "$LOG" 2>&1
  echo "native-desktop-vo exit=$?" >> "$LOG"
fi

echo "" >> "$LOG"
# No residual active sessions: close whatever the run left open.
sleep 1
osascript -e 'tell application "epher" to quit' >/dev/null 2>&1
sleep 2
killall epher 2>/dev/null
osascript -e 'tell application "VoiceOver" to quit' >/dev/null 2>&1
sleep 2
if pgrep -x epher >/dev/null 2>&1; then
  echo "WARN  an epher process survived the cleanup" >> "$LOG"
else
  echo "epher closed after the run" >> "$LOG"
fi
if pgrep -x VoiceOver >/dev/null 2>&1; then
  echo "WARN  VoiceOver survived the cleanup" >> "$LOG"
else
  echo "VoiceOver closed after the run" >> "$LOG"
fi

echo "══ summary ══" >> "$LOG"
grep -h 'SUMMARY\|failures of\|failure(s)' "$LOG" >> "$LOG.summary" 2>/dev/null
cat "$LOG.summary" >> "$LOG" 2>/dev/null
rm -f "$LOG.summary"
echo "DONE $(date '+%Y-%m-%d %H:%M:%S')" >> "$LOG"
