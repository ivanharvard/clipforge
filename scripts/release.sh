#!/usr/bin/env bash
# Bumps the workspace version, commits, tags, and pushes — which is all
# .github/workflows/release.yml needs to see (a pushed vX.Y.Z tag whose
# version matches Cargo.toml) to build and publish a release.
#
# Usage:
#   make release                # interactive: shows current version + commit
#                                # count since the last release, asks what kind
#                                # of bump this is
#   make release VERSION=1.2.3  # non-interactive: bump straight to 1.2.3
set -euo pipefail
cd "$(dirname "$0")/.."

if [ -n "$(git status --porcelain)" ]; then
  echo "error: working tree isn't clean. Commit or stash your changes before releasing." >&2
  git status --short >&2
  exit 1
fi

current_version=$(grep -m1 '^version = ' Cargo.toml | sed -E 's/version = "(.*)"/\1/')
last_tag=$(git describe --tags --abbrev=0 2>/dev/null || true)

if [ -n "$last_tag" ]; then
  commit_count=$(git rev-list "${last_tag}..HEAD" --count)
  since_text="$commit_count commits since $last_tag"
else
  commit_count=$(git rev-list --count HEAD)
  since_text="$commit_count commits (no previous release)"
fi

new_version="${VERSION:-}"

if [ -z "$new_version" ]; then
  echo "ClipForge is currently on version $current_version. There have been $since_text."
  echo
  echo "What kind of version update is this?"
  select choice in "Patch" "Minor" "Major" "None"; do
    case "$choice" in
      Patch|Minor|Major|None) break ;;
      *) echo "Pick 1-4." ;;
    esac
  done

  if [ "$choice" = "None" ]; then
    echo "No version bump — nothing to release. Exiting."
    exit 0
  fi

  IFS='.' read -r major minor patch <<<"$current_version"
  case "$choice" in
    Patch) new_version="$major.$minor.$((patch + 1))" ;;
    Minor) new_version="$major.$((minor + 1)).0" ;;
    Major) new_version="$((major + 1)).0.0" ;;
  esac
fi

tag="v$new_version"

if git rev-parse "$tag" >/dev/null 2>&1; then
  echo "error: tag $tag already exists." >&2
  exit 1
fi

echo
echo "Formatting the workspace..."
cargo fmt --all

if [ "$new_version" = "$current_version" ]; then
  # Cargo.toml is already at the target version — most likely a previous
  # release attempt's version-bump commit already landed and only the tag
  # (e.g. because its build failed) got deleted. Nothing to bump; formatting
  # above may still have found something to fix, though.
  echo "Cargo.toml is already at $new_version — no version bump needed."
else
  echo "Bumping $current_version -> $new_version"
  sed -i "0,/^version = \"$current_version\"/s//version = \"$new_version\"/" Cargo.toml

  # Member crates use version.workspace = true, so Cargo.lock's own entries
  # for them need refreshing too — this touches nothing else since they're
  # path dependencies, not registry ones, so it stays fully offline.
  cargo check --workspace --quiet
fi

# Whatever combination of fmt and the version bump actually touched files
# (either, both, or neither) gets bundled into one commit rather than
# tracking them separately — there's nothing to gain from two commits here.
made_commit=false
if [ -n "$(git status --porcelain)" ]; then
  git add -A
  git commit -m "Release $tag"
  made_commit=true
fi

git tag "$tag"

branch=$(git rev-parse --abbrev-ref HEAD)
echo
echo "About to push:"
if [ "$made_commit" = true ]; then
  echo "  - commit $(git rev-parse --short HEAD) (\"Release $tag\") to origin/$branch"
else
  echo "  - origin/$branch (no new commit — tagging the existing HEAD)"
fi
echo "  - tag $tag"
read -r -p "Proceed? [y/N] " confirm
if [[ ! "$confirm" =~ ^[Yy]$ ]]; then
  echo "Not pushing. The commit and tag are made locally — push manually when ready:"
  echo "  git push origin $branch && git push origin $tag"
  exit 0
fi

git push origin "$branch"
git push origin "$tag"

echo
echo "Pushed $tag — release workflow starting:"
echo "  https://github.com/ivanharvard/clipforge/actions"
