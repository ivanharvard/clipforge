#!/usr/bin/env bash
# Locally exercises the same steps as .github/workflows/release.yml's
# aur-publish job, using your own SSH key (whatever's registered with your
# AUR account) instead of the CI secret — lets you validate the AUR publish
# pipeline without waiting on a real release.
#
# Defaults to a dry run: clones/inits the AUR repo, regenerates PKGBUILD +
# .SRCINFO, and commits locally, but stops short of pushing. Pass --push to
# actually publish.
#
# Usage:
#   ./scripts/test-aur-publish.sh [VERSION] [--push]
# VERSION defaults to the workspace's current Cargo.toml version. That tag
# (vVERSION) must already exist on GitHub, since the sha256sum is computed
# from its release tarball, same as the real CI job.
set -euo pipefail

cd "$(dirname "$0")/.."

push=false
version=""
for arg in "$@"; do
  case "$arg" in
    --push) push=true ;;
    *) version="$arg" ;;
  esac
done
if [ -z "$version" ]; then
  version=$(grep -m1 '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
fi

workdir=$(mktemp -d)
trap 'rm -rf "$workdir"' EXIT
repo="$workdir/aur-repo"

echo "==> Testing AUR publish for v$version (push=$push)"

# No "clone fails -> init an empty repo" fallback: AUR clones a brand-new,
# not-yet-published package name as an empty repo just fine, so a real
# clone failure here means something's actually wrong (e.g. AUR's own
# maintenance flakiness) — worth seeing directly rather than silently
# working around it, same reasoning as release.yml's aur-publish job.
for attempt in 1 2 3; do
  if git clone ssh://aur@aur.archlinux.org/clipforge.git "$repo"; then
    break
  fi
  if [ "$attempt" -eq 3 ]; then
    echo "==> Cloning the AUR repo failed after 3 attempts." >&2
    exit 1
  fi
  rm -rf "$repo"
  sleep $((attempt * 5))
done

cp packaging/linux/aur/PKGBUILD "$repo/PKGBUILD"
sed -i "s/^pkgver=.*/pkgver=$version/" "$repo/PKGBUILD"
sed -i "s/^pkgrel=.*/pkgrel=1/" "$repo/PKGBUILD"
sha=$(curl -sL "https://github.com/ivanharvard/clipforge/archive/refs/tags/v$version.tar.gz" | sha256sum | cut -d' ' -f1)
sed -i "s/^sha256sums=.*/sha256sums=('$sha')/" "$repo/PKGBUILD"

(cd "$repo" && makepkg --printsrcinfo > .SRCINFO)

cd "$repo"
git config user.name "ClipForge CI (local test)"
git config user.email "actions@github.com"
git add PKGBUILD .SRCINFO

if git diff --cached --quiet; then
  echo "==> PKGBUILD/.SRCINFO unchanged for v$version — nothing to publish."
  exit 0
fi

git commit -m "Update to $version"
echo "==> Commit created locally:"
git show --stat HEAD

if [ "$push" = true ]; then
  echo "==> Pushing to AUR..."
  git push origin HEAD:master
  echo "==> Published."
else
  echo "==> Dry run — not pushed. Re-run with --push to actually publish."
fi
