#!/usr/bin/env bash
# Generates all packaged icon sizes from the single master SVG at
# crates/clipforge-app/icons/src/app-icon.svg (DESIGN.md section 6: SVG is
# the source of truth, everything else is a generated export).
#
# Requires: rsvg-convert (PNG export) and ImageMagick's `convert` (ICO
# bundling). Re-run this any time app-icon.svg changes.
set -euo pipefail

cd "$(dirname "$0")/.."

SRC="crates/clipforge-app/icons/src/app-icon.svg"
OUT="packaging/shared/icons"
LINUX_SIZES=(16 24 32 48 64 128 256 512)
ICO_SIZES=(16 32 48 256)

command -v rsvg-convert >/dev/null 2>&1 || { echo "error: rsvg-convert not found" >&2; exit 1; }
if command -v magick >/dev/null 2>&1; then
  IMAGEMAGICK="magick"
elif command -v convert >/dev/null 2>&1; then
  IMAGEMAGICK="convert"
else
  echo "error: ImageMagick (magick or convert) not found" >&2
  exit 1
fi

mkdir -p "$OUT"

echo "Exporting PNGs to $OUT/ ..."
for size in "${LINUX_SIZES[@]}"; do
  rsvg-convert -w "$size" -h "$size" "$SRC" -o "$OUT/clipforge-${size}.png"
done

echo "Building packaging/windows/icon.ico ..."
ico_inputs=()
for size in "${ICO_SIZES[@]}"; do
  ico_inputs+=("$OUT/clipforge-${size}.png")
done
"$IMAGEMAGICK" "${ico_inputs[@]}" packaging/windows/icon.ico

echo "Done. The app window/taskbar icon (crates/clipforge-app/ui/app.slint)"
echo "loads app-icon.svg directly, so no separate copy is needed there."
