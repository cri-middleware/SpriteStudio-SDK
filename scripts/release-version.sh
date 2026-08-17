#!/usr/bin/env bash
#
# Print the release version -- the single source of truth for everything a
# release is named after.
#
#   scripts/release-version.sh          # 7.0.0-beta.1
#   v$(scripts/release-version.sh)      # v7.0.0-beta.1  -- the git tag
#
# It is `[workspace.package] version` in Cargo.toml, and nothing else. Every
# crate in the workspace inherits it (`version.workspace = true`), and every
# Player pins the SDK by the matching tag in its scripts/SDK_VERSION.txt -- so
# the manifest is what a consumer is already tracking, whether or not a tag
# happens to exist yet.
#
# That is the whole reason this is a script and not a line of YAML: the release
# workflow, build-release.sh and anyone running a release by hand have to agree
# on the answer, and the way to guarantee that is for there to be one place that
# computes it.
set -euo pipefail

APP="$(basename "$0")"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

case "${1:-}" in
  -h|--help|help)
    echo "Usage: $APP"
    echo "Prints [workspace.package] version from Cargo.toml (no leading 'v')."
    exit 0
    ;;
  "") ;;
  *)
    echo "$APP: unexpected argument '$1' (see $APP --help)" >&2
    exit 2
    ;;
esac

MANIFEST="$ROOT_DIR/Cargo.toml"
if [ ! -f "$MANIFEST" ]; then
  echo "$APP: $MANIFEST not found -- is this the SpriteStudio-SDK repo?" >&2
  exit 1
fi

# Deliberately scoped to the [workspace.package] table: [workspace.dependencies]
# below it is full of `version =` lines, and a grep for the first one in the file
# would happily return a dependency's.
VERSION="$(
  awk '
    /^[[:space:]]*\[/ { in_section = ($0 ~ /^[[:space:]]*\[workspace\.package\][[:space:]]*$/); next }
    in_section && /^[[:space:]]*version[[:space:]]*=/ {
      sub(/^[^=]*=[[:space:]]*/, "")
      sub(/[[:space:]]*#.*$/, "")
      gsub(/"/, "")
      gsub(/[[:space:]]/, "")
      print
      exit
    }
  ' "$MANIFEST"
)"

if [ -z "$VERSION" ]; then
  echo "$APP: no [workspace.package] version in Cargo.toml" >&2
  exit 1
fi

printf '%s\n' "$VERSION"
