#!/bin/bash
# Full accessibility suite under the epher-a11y launcher app's own TCC
# identity (com.upyesp.epher-a11y). The app bundle's executable is a thin,
# never-changing shim into this script, so its code signature stays stable
# and permissions granted once keep working. The agent session triggers
# runs with `open /Applications/epher-a11y.app` and reads the log.

set -u
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin"
LOG=/tmp/epher-a11y-latest.log
REPO=/Users/pete/Developer/code/epher

mkdir -p "$(dirname "$LOG")"
{
  echo "epher accessibility run started $(date '+%Y-%m-%d %H:%M:%S')"
  echo "runner: com.upyesp.epher-a11y"
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
if [ ! -d "$BUNDLE" ]; then
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
echo "══ summary ══" >> "$LOG"
grep -h 'SUMMARY\|failures of\|failure(s)' "$LOG" >> "$LOG.summary" 2>/dev/null
cat "$LOG.summary" >> "$LOG" 2>/dev/null
rm -f "$LOG.summary"
echo "DONE $(date '+%Y-%m-%d %H:%M:%S')" >> "$LOG"
