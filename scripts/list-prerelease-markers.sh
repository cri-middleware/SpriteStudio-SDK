#!/usr/bin/env bash
#
# List the pre-release markers that expire on release day.
#
# The SDK, the Players and the portal all advertise things that do not exist
# yet -- hosted documentation sites, npm and pub packages, GitHub Releases --
# behind a marker that says so. The wording is a convention, defined in
# SpriteStudio-Docs/AGENTS.md: an outbound link to something unpublished is
# neutralized to plain "coming soon" / "準備中" text, and something published
# but not yet reachable is linked with a 🚧 beside it.
#
# Every one of those becomes a lie the moment the thing it promises ships, and
# a stale one is invisible from the inside: it reads correctly to anyone who
# knows the release has not happened, and wrong only to the reader arriving
# from outside, which is the reader it was written for. They are spread across
# ten repositories, so nobody finds them all by remembering.
#
# This prints every one, so a release can walk the list instead of the memory.
#
# Usage:
#   scripts/list-prerelease-markers.sh                # this repo and its siblings
#   scripts/list-prerelease-markers.sh <dir>...       # only the repos given
#   scripts/list-prerelease-markers.sh --count        # one line per repo
#
# Exit status is 1 when any marker is found, so a release step can gate on it.
# That is the normal state before the first release -- it is not a failure until
# the release is the thing you are doing.
#
# AGENTS.md is skipped: it defines the convention, so it names the markers
# without carrying any.
set -euo pipefail

APP="$(basename "$0")"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

MARKERS=(-e '🚧' -e 'coming soon' -e 'Coming Soon' -e '準備中')
COUNT_ONLY=no
TARGETS=()

for arg in "$@"; do
  case "$arg" in
    -h|--help|help)
      sed -n '3,29p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    --count) COUNT_ONLY=yes ;;
    -*) echo "$APP: unknown option '$arg' (see $APP --help)" >&2; exit 2 ;;
    *)  TARGETS+=("$(cd "$arg" 2>/dev/null && pwd -P || echo "$arg")") ;;
  esac
done

# Default: this repository, plus every sibling checkout next to it. The Players
# live beside the SDK rather than inside it, so there is no manifest to read --
# what is on disk is the list.
if [ "${#TARGETS[@]}" -eq 0 ]; then
  seen=""
  for dir in "$ROOT_DIR" "$(dirname "$ROOT_DIR")"/*; do
    [ -d "$dir/.git" ] || continue
    # $ROOT_DIR is itself one of the siblings, so the two loops overlap.
    real="$(cd "$dir" && pwd -P)"
    case "$seen" in *":$real:"*) continue ;; esac
    seen="$seen:$real:"
    TARGETS+=("$real")
  done
fi

total=0
for dir in "${TARGETS[@]}"; do
  if [ ! -d "$dir/.git" ]; then
    echo "$APP: $dir is not a git checkout, skipping" >&2
    continue
  fi
  name="$(basename "$dir")"
  hits="$(git -C "$dir" grep -nI "${MARKERS[@]}" -- '*.md' '*.yml' 2>/dev/null | grep -v '^AGENTS\.md:' || true)"
  n=0
  [ -n "$hits" ] && n="$(printf '%s\n' "$hits" | wc -l | tr -d ' ')"
  total=$((total + n))

  if [ "$COUNT_ONLY" = yes ]; then
    printf '%-24s %s\n' "$name" "$n"
    continue
  fi
  [ "$n" -eq 0 ] && continue
  echo "=== $name ($n)"
  printf '%s\n' "$hits" | sed 's/^/  /'
done

echo
noun="repositories"; [ "${#TARGETS[@]}" -eq 1 ] && noun="repository"
echo "$APP: $total marker(s) across ${#TARGETS[@]} $noun"
[ "$total" -eq 0 ] || exit 1
