#!/usr/bin/env bash
# Builds a release binary and packages it as an AppImage using
# appimage-builder and the recipe in packaging/linux/appimage/.
set -euo pipefail

cd "$(dirname "$0")/.."

cargo build --release --bin clipforge-app

mkdir -p dist
appimage-builder \
  --recipe packaging/linux/appimage/AppImageBuilder.yml \
  --skip-test
mv ClipForge*.AppImage dist/ 2>/dev/null || true
