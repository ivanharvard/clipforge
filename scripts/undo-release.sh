#!/usr/bin/env bash
# Deletes a release tag (locally and on origin), and the GitHub Release it
# produced if one was actually published — recovery for a tag whose build
# failed, so you can fix the problem and re-run `make release VERSION=x.y.z`
# against the same version afterward.
#
# Usage:
#   make undo-release VERSION=1.0.0
set -euo pipefail
cd "$(dirname "$0")/.."

if [ -z "${VERSION:-}" ]; then
  echo "error: VERSION is required. Usage: make undo-release VERSION=1.0.0" >&2
  exit 1
fi

tag="v$VERSION"

local_exists=false
if git rev-parse "$tag" >/dev/null 2>&1; then
  local_exists=true
fi

remote_exists=false
if [ -n "$(git ls-remote --tags origin "refs/tags/$tag" 2>/dev/null)" ]; then
  remote_exists=true
fi

if [ "$local_exists" = false ] && [ "$remote_exists" = false ]; then
  echo "Tag $tag doesn't exist locally or on origin — nothing to undo."
  exit 0
fi

release_exists=false
if gh release view "$tag" >/dev/null 2>&1; then
  release_exists=true
fi

echo "About to undo release $tag:"
[ "$local_exists" = true ] && echo "  - delete local tag $tag"
[ "$remote_exists" = true ] && echo "  - delete origin tag $tag"
if [ "$release_exists" = true ]; then
  echo "  - delete the PUBLISHED GitHub Release for $tag — this has real (possibly already-downloaded) assets attached"
else
  echo "  - (no GitHub Release was ever published for $tag, so nothing public is affected)"
fi

read -r -p "Proceed? [y/N] " confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
  echo "Aborted, nothing changed."
  exit 0
fi

if [ "$release_exists" = true ]; then
  gh release delete "$tag" --yes
  echo "Deleted GitHub Release $tag."
fi

if [ "$remote_exists" = true ]; then
  git push origin ":refs/tags/$tag"
fi

if [ "$local_exists" = true ]; then
  git tag -d "$tag"
fi

echo
echo "Done. Once whatever broke the build is fixed, re-run:"
echo "  make release VERSION=$VERSION"
