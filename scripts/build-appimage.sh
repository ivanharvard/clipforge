#!/usr/bin/env bash
# Builds a release binary and packages it as an AppImage using
# appimage-builder and the recipe in packaging/linux/appimage/.
set -euo pipefail

cd "$(dirname "$0")/.."

cargo build --release --bin clipforge-app

rm -rf AppDir
mkdir -p AppDir/usr/bin AppDir/usr/share/applications
cp target/release/clipforge-app AppDir/usr/bin/
cp packaging/linux/appimage/clipforge.desktop \
  AppDir/usr/share/applications/com.clipforge.ClipForge.desktop

for size in 16 24 32 48 64 128 256 512; do
  icon_dir="AppDir/usr/share/icons/hicolor/${size}x${size}/apps"
  mkdir -p "$icon_dir"
  cp "packaging/shared/icons/clipforge-${size}.png" "$icon_dir/clipforge.png"
done

mkdir -p dist
appimage-builder \
  --recipe packaging/linux/appimage/AppImageBuilder.yml \
  --skip-test
mv ClipForge*.AppImage dist/ 2>/dev/null || true

echo "Done. The AppImage is in dist/."