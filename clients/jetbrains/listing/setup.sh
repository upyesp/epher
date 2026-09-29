#!/usr/bin/env bash

# Prepares a Linux desktop (written for the Linux Mint VM) for the
# JetBrains Marketplace captures: downloads IntelliJ IDEA Ultimate
# 2024.2.4 (the exact IDE line the plugin builds against; free 30-day
# trial, and the JetBrains Account login it asks for at first start is
# the same account used for the marketplace portal), installs the epher
# plugin into the IDE, and creates the demo project with demo.epher.
#
#   ./setup.sh [path/to/epher-jetbrains-0.5.57-listing.zip]
#
# Without an argument the script looks for the listing zip in the home
# directory, then falls back to the latest release zip from GitHub.
# Idempotent: safe to rerun. Artifacts land in the home directory.

set -euo pipefail

IDE_VERSION="2024.2.4"
IDE_DIR="$HOME/ideaIU-captures"
PROJECT_DIR="$HOME/epher-captures"
TARBALL="$HOME/ideaIU-$IDE_VERSION.tar.gz"
URL="https://download.jetbrains.com/idea/ideaIU-$IDE_VERSION.tar.gz"

# --- 1. the plugin zip --------------------------------------------------
PLUGIN_ZIP="${1:-}"
if [ -z "$PLUGIN_ZIP" ]; then
  for candidate in "$HOME/epher-jetbrains-0.5.57-listing.zip" \
                   "$HOME/epher-jetbrains.zip" \
                   "$HOME/Desktop/epher-jetbrains.zip"; do
    if [ -f "$candidate" ]; then PLUGIN_ZIP="$candidate"; break; fi
  done
fi
if [ -z "$PLUGIN_ZIP" ]; then
  PLUGIN_ZIP="$HOME/epher-jetbrains.zip"
  echo "no plugin zip found; downloading the latest release"
  curl -fL -o "$PLUGIN_ZIP" \
    "https://github.com/upyesp/epher/releases/latest/download/epher-jetbrains.zip"
fi
[ -f "$PLUGIN_ZIP" ] || { echo "plugin zip not found: $PLUGIN_ZIP" >&2; exit 1; }
echo "plugin zip: $PLUGIN_ZIP"

# --- 2. the IDE ---------------------------------------------------------
if [ -x "$IDE_DIR/bin/idea.sh" ]; then
  echo "IDE already installed: $IDE_DIR"
else
  if [ ! -s "$TARBALL" ]; then
    echo "downloading IntelliJ IDEA $IDE_VERSION (~1.4 GB, once)"
    curl -fL --retry 3 -o "$TARBALL" "$URL"
  fi
  echo "extracting into $IDE_DIR"
  rm -rf "$IDE_DIR"
  mkdir -p "$IDE_DIR"
  tar -xzf "$TARBALL" -C "$IDE_DIR" --strip-components=1
fi

# --- 3. the plugin, straight into the IDE's plugins directory -----------
echo "installing the plugin into $IDE_DIR/plugins"
unzip -oq "$PLUGIN_ZIP" -d "$IDE_DIR/plugins"
ls -d "$IDE_DIR/plugins/epher" >/dev/null && echo "plugin installed"

# --- 4. the demo project ------------------------------------------------
mkdir -p "$PROJECT_DIR"
cat > "$PROJECT_DIR/demo.epher" <<'EOF'
// the earth, measured
const radius = 6371 km
const diameter = 2 * radius
circumference = pi * diameter

// the same numbers in miles
circumference in mile

// a right triangle
const legs = 3 m
const hypotenuse = 5 m
height = sqrt(hypotenuse^2 - legs^2)

// the discriminant
def disc(a, b, c) = b^2 - 4*a*c
disc(1, -5, 6)

graph sin(x)
EOF
echo "epher demo project" > "$PROJECT_DIR/notes.txt"

# --- 5. next steps ------------------------------------------------------
cat <<EOF

Ready. Next steps (manual):

  1. Start the IDE and open the project folder:
       "$IDE_DIR/bin/idea.sh" "$PROJECT_DIR" &
     Sign in (or start the trial) when asked; the trial is free.
  2. Open demo.epher and wait: the first open downloads the language
     server (a few MB), then every line shows its inline answer.
  3. Follow CAPTURE-GUIDE.md for the four screenshots.

To start over, delete "$IDE_DIR" and "$TARBALL".
EOF
