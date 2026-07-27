#!/usr/bin/env bash
# Builds the pacman package from packaging/linux/pacman/PKGBUILD.
set -euo pipefail

cd "$(dirname "$0")/../packaging/linux/pacman"

makepkg -f

mkdir -p ../../../dist
mv ./*.pkg.tar.zst ../../../dist/ 2>/dev/null || true
