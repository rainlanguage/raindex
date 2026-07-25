#!/usr/bin/env bash
# Render one harness scene to a PNG using a real headless Chromium.
#
# Provisions Chromium + DejaVu fonts + Node via nix, then runs the driver
# (screenshot.mjs). This is the one-command entry point:
#
#   packages/ui-components/harness/screenshot.sh <scene> <out.png> [width] [height]
#
# Examples:
#   harness/screenshot.sh toast-error-decoded /tmp/toast.png
#   harness/screenshot.sh toast-error-decoded /tmp/toast.png 760 240
#
# Scene names are the keys in scenes/index.ts. List them by rendering an unknown
# scene (the page prints the available scenes).
set -euo pipefail

SCENE="${1:?usage: screenshot.sh <scene> <out.png> [width] [height]}"
OUT="${2:?usage: screenshot.sh <scene> <out.png> [width] [height]}"
WIDTH="${3:-900}"
HEIGHT="${4:-360}"

HARNESS_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HARNESS_DIR/../../.." && pwd)"

echo "[harness] resolving chromium + fonts via nix..."
CHROMIUM_OUT="$(nix build --no-link --print-out-paths "nixpkgs#chromium")"
DEJAVU_OUT="$(nix build --no-link --print-out-paths "nixpkgs#dejavu_fonts")"

export CHROMIUM="$CHROMIUM_OUT/bin/chromium"
export HARNESS_FONT_DIR="$DEJAVU_OUT/share/fonts/truetype"

# Node/npm come from the repo's webapp devShell (nodejs_20). Run the driver
# there so `vite` resolves from the repo-root node_modules.
exec nix develop "$REPO_ROOT#webapp-shell" --command \
	node "$HARNESS_DIR/screenshot.mjs" \
	--scene "$SCENE" --out "$OUT" --width "$WIDTH" --height "$HEIGHT"
