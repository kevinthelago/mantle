#!/usr/bin/env bash
# update.sh — Update mantle to the latest release on the configured channel.
#
# Usage:
#   update.sh [OPTIONS]
#
# Options:
#   --channel   stable|nightly   Channel to update from (default: stable)
#   --prefix    /path            Install prefix (default: directory of current binary)
#   --dry-run                    Show what would be done without doing it
#
set -euo pipefail

REPO="kevinthelago/mantle"
CHANNEL="stable"
PREFIX=""
DRY_RUN=0
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ── Argument parsing ─────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --channel) CHANNEL="$2"; shift 2 ;;
    --prefix)  PREFIX="$2";  shift 2 ;;
    --dry-run) DRY_RUN=1;    shift   ;;
    -h|--help)
      sed -n '/^# /s/^# \?//p' "$0"
      exit 0
      ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

die()  { echo "error: $*" >&2; exit 1; }
info() { echo "  →  $*"; }

# ── Find current installation ─────────────────────────────────────────────────
CURRENT_BIN=$(command -v mantle 2>/dev/null || true)
if [[ -n "$PREFIX" ]]; then
  INSTALL_PREFIX="$PREFIX"
elif [[ -n "$CURRENT_BIN" ]]; then
  INSTALL_PREFIX="$(dirname "$(dirname "$CURRENT_BIN")")"
else
  [[ $EUID -eq 0 ]] && INSTALL_PREFIX="/usr/local" || INSTALL_PREFIX="$HOME/.local"
fi

# ── Determine current version ────────────────────────────────────────────────
CURRENT_VERSION="none"
if [[ -x "${INSTALL_PREFIX}/bin/mantle" ]]; then
  CURRENT_VERSION=$("${INSTALL_PREFIX}/bin/mantle" --version 2>/dev/null | awk '{print $NF}' || echo "unknown")
fi
info "Current version: ${CURRENT_VERSION}"

# ── Determine latest available version ───────────────────────────────────────
require_curl() { command -v curl &>/dev/null || die "curl is required"; }
require_curl

case "$CHANNEL" in
  stable)
    LATEST_TAG=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
                 | grep '"tag_name"' | head -1 | cut -d'"' -f4)
    [[ -n "$LATEST_TAG" ]] || die "could not fetch latest stable tag"
    ;;
  nightly)
    LATEST_TAG="nightly"
    ;;
  *)
    die "unknown channel '$CHANNEL'"
    ;;
esac

info "Latest ${CHANNEL}: ${LATEST_TAG}"

# ── Compare ───────────────────────────────────────────────────────────────────
if [[ "$CURRENT_VERSION" == "$LATEST_TAG" ]] || \
   [[ "v${CURRENT_VERSION}" == "$LATEST_TAG" ]]; then
  echo "mantle is already up to date (${CURRENT_VERSION})."
  exit 0
fi

echo "Update available: ${CURRENT_VERSION} → ${LATEST_TAG}"

if [[ $DRY_RUN -eq 1 ]]; then
  echo "(dry-run) Would run: $SCRIPT_DIR/install.sh --channel $CHANNEL --prefix $INSTALL_PREFIX --no-autostart"
  exit 0
fi

# ── Delegate to install.sh ────────────────────────────────────────────────────
exec "$SCRIPT_DIR/install.sh" \
  --channel "$CHANNEL" \
  --prefix  "$INSTALL_PREFIX" \
  --no-autostart
