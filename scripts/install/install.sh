#!/usr/bin/env bash
# install.sh — Download and install mantle, then wire autostart.
#
# Usage:
#   install.sh [OPTIONS]
#
# Options:
#   --channel   stable|nightly     Release channel (default: stable)
#   --version   vX.Y.Z             Pin to a specific release tag (stable only)
#   --prefix    /path              Install prefix (default: ~/.local if not root, /usr/local if root)
#   --no-autostart                 Skip autostart configuration
#
# Environment ladder:
#   stable  → versioned tags on main  (production)
#   nightly → latest artifact from develop CI
#
set -euo pipefail

REPO="kevinthelago/mantle"
CHANNEL="stable"
VERSION=""
PREFIX=""
AUTOSTART=1

# ── Argument parsing ────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --channel)   CHANNEL="$2";   shift 2 ;;
    --version)   VERSION="$2";   shift 2 ;;
    --prefix)    PREFIX="$2";    shift 2 ;;
    --no-autostart) AUTOSTART=0; shift   ;;
    -h|--help)
      sed -n '/^# /s/^# \?//p' "$0"
      exit 0
      ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

# ── Defaults ─────────────────────────────────────────────────────────────────
if [[ -z "$PREFIX" ]]; then
  [[ $EUID -eq 0 ]] && PREFIX="/usr/local" || PREFIX="$HOME/.local"
fi
BIN_DIR="$PREFIX/bin"

# ── Helpers ───────────────────────────────────────────────────────────────────
die()  { echo "error: $*" >&2; exit 1; }
info() { echo "  →  $*"; }

require() {
  command -v "$1" &>/dev/null || die "required tool not found: $1"
}

require curl
require sha256sum

# ── Resolve download URL ──────────────────────────────────────────────────────
resolve_stable_url() {
  local tag
  if [[ -n "$VERSION" ]]; then
    tag="$VERSION"
  else
    tag=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
          | grep '"tag_name"' | head -1 | cut -d'"' -f4)
    [[ -n "$tag" ]] || die "could not determine latest stable tag"
  fi
  echo "https://github.com/${REPO}/releases/download/${tag}/mantle_${tag}_x86_64-linux"
  # Also expose the tag for use by the caller
  RESOLVED_TAG="$tag"
}

resolve_nightly_url() {
  # Nightly artifacts are uploaded by CI on each push to develop.
  # The nightly pre-release tag is "nightly".
  RESOLVED_TAG="nightly"
  echo "https://github.com/${REPO}/releases/download/nightly/mantle_nightly_x86_64-linux"
}

case "$CHANNEL" in
  stable)  DOWNLOAD_URL=$(resolve_stable_url) ;;
  nightly) DOWNLOAD_URL=$(resolve_nightly_url) ;;
  *)       die "unknown channel '$CHANNEL' — use 'stable' or 'nightly'" ;;
esac

# ── Download ─────────────────────────────────────────────────────────────────
TMPDIR=$(mktemp -d)
trap 'rm -rf "$TMPDIR"' EXIT

info "Downloading mantle ${RESOLVED_TAG} (${CHANNEL}) …"
curl -fL --progress-bar "$DOWNLOAD_URL" -o "$TMPDIR/mantle"

# Verify checksum if a SHA256SUMS file is available
SUMS_URL="https://github.com/${REPO}/releases/download/${RESOLVED_TAG}/SHA256SUMS"
if curl -fsSL "$SUMS_URL" -o "$TMPDIR/SHA256SUMS" 2>/dev/null; then
  info "Verifying checksum …"
  BINARY_NAME="$(basename "$DOWNLOAD_URL")"
  (cd "$TMPDIR" && grep "$BINARY_NAME" SHA256SUMS | sha256sum --check --status) \
    || die "checksum verification failed"
  info "Checksum OK."
fi

# ── Install ───────────────────────────────────────────────────────────────────
mkdir -p "$BIN_DIR"
install -m 0755 "$TMPDIR/mantle" "$BIN_DIR/mantle"
info "Installed mantle to $BIN_DIR/mantle"

# ── Autostart ────────────────────────────────────────────────────────────────
if [[ $AUTOSTART -eq 1 ]]; then
  setup_sway_autostart() {
    local cfg="${XDG_CONFIG_HOME:-$HOME/.config}/sway/config"
    if [[ -f "$cfg" ]]; then
      if ! grep -qE '^\s*exec\s+mantle\b' "$cfg"; then
        echo "" >> "$cfg"
        echo "# mantle bar — added by install.sh" >> "$cfg"
        echo "exec mantle" >> "$cfg"
        info "Added 'exec mantle' to $cfg"
      else
        info "sway autostart already present in $cfg"
      fi
    fi
  }

  setup_hyprland_autostart() {
    local cfg="${XDG_CONFIG_HOME:-$HOME/.config}/hypr/hyprland.conf"
    if [[ -f "$cfg" ]]; then
      if ! grep -qE '^\s*exec-once\s*=\s*mantle\b' "$cfg"; then
        echo "" >> "$cfg"
        echo "# mantle bar — added by install.sh" >> "$cfg"
        echo "exec-once = mantle" >> "$cfg"
        info "Added 'exec-once = mantle' to $cfg"
      else
        info "Hyprland autostart already present in $cfg"
      fi
    fi
  }

  setup_sway_autostart
  setup_hyprland_autostart
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo "mantle ${RESOLVED_TAG} installed successfully."
echo "Binary: $BIN_DIR/mantle"
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
  echo ""
  echo "Note: $BIN_DIR is not in your PATH."
  echo "  Add this to your shell profile: export PATH=\"\$PATH:$BIN_DIR\""
fi
