#!/usr/bin/env bash
# Builds release binaries and packages them as AppImages using
# appimage-builder and the recipes in packaging/linux/appimage/: the default
# build, and a second one compiled with the qt-style Cargo feature so its
# std-widgets controls render via the user's actual KDE QStyle (Breeze,
# Kvantum, etc.) — see packaging/linux/appimage/AppImageBuilder-kde.yml for
# why that one needs appimage-builder to bundle Qt6 rather than relying on a
# system dependency the way ffmpeg/mpv do.
set -euo pipefail

cd "$(dirname "$0")/.."

stage_appdir() {
  local appdir="$1" binary="$2" desktop_file="$3" desktop_name="$4"
  rm -rf "$appdir"
  mkdir -p "$appdir/usr/bin" "$appdir/usr/share/applications"
  cp "$binary" "$appdir/usr/bin/clipforge-app"
  cp "packaging/linux/appimage/$desktop_file" \
    "$appdir/usr/share/applications/$desktop_name"
  for size in 16 24 32 48 64 128 256 512; do
    icon_dir="$appdir/usr/share/icons/hicolor/${size}x${size}/apps"
    mkdir -p "$icon_dir"
    cp "packaging/shared/icons/clipforge-${size}.png" "$icon_dir/clipforge.png"
  done
}

mkdir -p dist

cargo build --release --bin clipforge-app
stage_appdir AppDir target/release/clipforge-app \
  clipforge.desktop com.clipforge.ClipForge.desktop
appimage-builder \
  --recipe packaging/linux/appimage/AppImageBuilder.yml \
  --appdir AppDir \
  --skip-test
mv ./*.AppImage dist/ 2>/dev/null || true

cargo build --release --bin clipforge-app --features qt-style \
  --target-dir target/qt-style
stage_appdir AppDir-kde target/qt-style/release/clipforge-app \
  clipforge-kde.desktop com.clipforge.ClipForge.KDE.desktop
appimage-builder \
  --recipe packaging/linux/appimage/AppImageBuilder-kde.yml \
  --appdir AppDir-kde \
  --skip-test
mv ./*.AppImage dist/ 2>/dev/null || true

echo "Done. The AppImages are in dist/."
