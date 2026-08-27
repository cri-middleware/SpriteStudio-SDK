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
#   scripts/list-prerelease-markers.sh                # this repository
#   scripts/list-prerelease-markers.sh <dir>...       # the repositories given
#   scripts/list-prerelease-markers.sh --siblings     # and every checkout beside it
#   scripts/list-prerelease-markers.sh --count        # one line per repository
#
# It checks **this repository** by default, and nothing else. That is the only
# mode with no assumption behind it: it holds in CI, where one repository is
# checked out and no other exists, and in anyone's clone wherever they keep it.
#
# --siblings is a convenience for whoever is coordinating a release across the
# family and happens to keep all the checkouts in one directory. It is opt-in
# precisely because that layout is one person's habit, not a contract -- do not
# build anything on top of it that has to be right.
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
SIBLINGS=no
TARGETS=()

for arg in "$@"; do
  case "$arg" in
    -h|--help|help)
      sed -n '3,36p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    --count) COUNT_ONLY=yes ;;
    --siblings) SIBLINGS=yes ;;
    -*) echo "$APP: unknown option '$arg' (see $APP --help)" >&2; exit 2 ;;
    *)  TARGETS+=("$(cd "$arg" 2>/dev/null && pwd -P || echo "$arg")") ;;
  esac
done

# Default: this repository. --siblings widens it to every checkout beside this
# one -- the Players live next to the SDK rather than inside it, so there is no
# manifest to read and what is on disk is the list. Which is exactly why it is
# not the default: what is on disk differs per machine, and in CI there is
# nothing beside the checkout at all.
if [ "${#TARGETS[@]}" -eq 0 ]; then
  TARGETS+=("$ROOT_DIR")
  if [ "$SIBLINGS" = yes ]; then
    seen=":$ROOT_DIR:"
    for dir in "$(dirname "$ROOT_DIR")"/*; do
      [ -d "$dir/.git" ] || continue
      real="$(cd "$dir" && pwd -P)"
      case "$seen" in *":$real:"*) continue ;; esac
      seen="$seen:$real:"
      TARGETS+=("$real")
    done
    if [ "${#TARGETS[@]}" -eq 1 ]; then
      echo "$APP: --siblings found no other checkout beside $(dirname "$ROOT_DIR")" >&2
    fi
  fi
elif [ "$SIBLINGS" = yes ]; then
  echo "$APP: --siblings and an explicit list are mutually exclusive" >&2
  exit 2
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
