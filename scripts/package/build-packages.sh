#!/usr/bin/env bash
# build-packages.sh — Build all release bundles locally.
#
# Produces AppImage, .deb, .rpm and a raw binary under dist/.
#
# Usage:
#   scripts/package/build-packages.sh [--release-dir DIR] [--bundles LIST]
#
# Options:
#   --release-dir  Output directory (default: dist/)
#   --bundles      Comma-separated Tauri bundle targets
#                  (default: appimage,deb,rpm)
#   --no-sign      Skip Tauri code signing (useful for local testing)
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RELEASE_DIR="$REPO_ROOT/dist"
BUNDLES="appimage,deb,rpm"
SIGN_ARGS=()

# ── Argument parsing ─────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --release-dir) RELEASE_DIR="$2";  shift 2 ;;
    --bundles)     BUNDLES="$2";      shift 2 ;;
    --no-sign)     SIGN_ARGS=(--no-bundle); shift ;;
    -h|--help)
      sed -n '/^# /s/^# \?//p' "$0"
      exit 0
      ;;
    *) echo "Unknown option: $1" >&2; exit 1 ;;
  esac
done

die()  { echo "error: $*" >&2; exit 1; }
info() { echo "  →  $*"; }

cd "$REPO_ROOT"

# ── Pre-flight checks ─────────────────────────────────────────────────────────
command -v cargo  &>/dev/null || die "cargo not found — install Rust"
command -v npm    &>/dev/null || die "npm not found — install Node 20"
command -v npx    &>/dev/null || die "npx not found — install Node 20"

# ── Build ─────────────────────────────────────────────────────────────────────
info "Installing npm dependencies …"
npm ci

info "Building Tauri bundles: $BUNDLES"
npx tauri build --bundles "$BUNDLES" "${SIGN_ARGS[@]+"${SIGN_ARGS[@]}"}"

# ── Collect artifacts ─────────────────────────────────────────────────────────
BUNDLE_DIR="$REPO_ROOT/src-tauri/target/release/bundle"
BINARY="$REPO_ROOT/src-tauri/target/release/mantle"

# Derive version from Cargo.toml (single source of truth)
VERSION=$(cargo metadata \
  --no-deps \
  --manifest-path src-tauri/Cargo.toml \
  --format-version 1 \
  | python3 -c "import sys,json; pkgs=json.load(sys.stdin)['packages']; \
    print(next(p['version'] for p in pkgs if p['name']=='mantle'))")
VERSION="v${VERSION}"

mkdir -p "$RELEASE_DIR"

copy_glob() {
  local src_dir="$1" pattern="$2" dest="$3"
  local found
  found=$(find "$src_dir" -name "$pattern" 2>/dev/null | head -1)
  if [[ -n "$found" ]]; then
    cp "$found" "$dest"
    info "$(basename "$dest")"
  fi
}

copy_glob "$BUNDLE_DIR/appimage" "*.AppImage" "$RELEASE_DIR/mantle_${VERSION}_amd64.AppImage"
copy_glob "$BUNDLE_DIR/deb"      "*.deb"      "$RELEASE_DIR/mantle_${VERSION}_amd64.deb"
copy_glob "$BUNDLE_DIR/rpm"      "*.rpm"      "$RELEASE_DIR/mantle-${VERSION}-1.x86_64.rpm"
[[ -f "$BINARY" ]] && cp "$BINARY" "$RELEASE_DIR/mantle_${VERSION}_x86_64-linux" && info "mantle_${VERSION}_x86_64-linux"

# ── Checksums ─────────────────────────────────────────────────────────────────
(cd "$RELEASE_DIR" && sha256sum * > SHA256SUMS)
info "SHA256SUMS written"

echo ""
echo "Packages written to $RELEASE_DIR:"
ls -lh "$RELEASE_DIR"
