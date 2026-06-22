#!/usr/bin/env bash
# Launch a nested sway compositor and run `npm run tauri:dev` inside it.
#
# Usage:
#   ./scripts/dev-nested-sway.sh          # use default display name
#   NESTED_DISPLAY=wayland-2 ./scripts/dev-nested-sway.sh
#
# Prerequisites: sway, wayland-info (optional, for readiness check)
set -euo pipefail

NESTED_DISPLAY="${NESTED_DISPLAY:-wayland-mantle-dev}"
SWAY_PID=""

cleanup() {
  if [[ -n "$SWAY_PID" ]]; then
    echo "[dev-nested-sway] stopping nested sway (pid $SWAY_PID)"
    kill "$SWAY_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

# Minimal sway config: no bar, no status, transparent background
SWAY_CONF=$(mktemp /tmp/mantle-sway-XXXX.conf)
trap 'rm -f "$SWAY_CONF"' EXIT

cat > "$SWAY_CONF" <<'EOF'
# Minimal nested sway config for Mantle dev
output * bg #1a1a1a solid_color
default_border none
default_floating_border none
gaps inner 0
gaps outer 0
EOF

echo "[dev-nested-sway] starting nested sway on $NESTED_DISPLAY"
WAYLAND_DISPLAY="$NESTED_DISPLAY" sway --config "$SWAY_CONF" &
SWAY_PID=$!

# Wait for the nested compositor to become available
MAX_WAIT=10
WAITED=0
until WAYLAND_DISPLAY="$NESTED_DISPLAY" wayland-info >/dev/null 2>&1; do
  if (( WAITED >= MAX_WAIT )); then
    echo "[dev-nested-sway] ERROR: nested sway did not start within ${MAX_WAIT}s" >&2
    exit 1
  fi
  sleep 0.5
  (( WAITED++ )) || true
done

echo "[dev-nested-sway] nested sway ready — launching tauri dev"
export WAYLAND_DISPLAY="$NESTED_DISPLAY"
exec npm run tauri:dev
