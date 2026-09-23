#!/usr/bin/env bash
#
# Assemble the release artifacts: take the per-platform trees the build matrix
# produced, lay them out the way every consumer expects, and zip them.
#
#   scripts/build-release.sh                  # assemble from artifacts/
#   scripts/build-release.sh licenses=no      # reuse target/licenses (no cargo-about)
#   scripts/build-release.sh out=/tmp/rel     # somewhere else
#
# release.yml's package job runs this script for the assets it uploads, so CI
# and a local run cannot drift. The workflow keeps only what is GitHub's -- the
# checkout, the toolchain, the artifact download and the Release itself.
#
# This is the phase AFTER it has been decided what is being released. Which
# commit, which tag and which build's artifacts are all settled before this
# script runs -- by the tag that was pushed, the ref the workflow was dispatched
# from, and the download that populated in=. So there is no option here that
# re-decides any of them: the tag is read from the checkout this is running in,
# and the artifacts are read from the directory they were put in.
#
#   git checkout v7.0.0-beta.1                # which release
#   gh run download <id> -D artifacts         # which build's artifacts
#   scripts/build-release.sh                  # this
#
# What this does NOT do is build anything. The seven platform jobs need seven
# operating systems between them, so there is no local equivalent of the matrix
# and pretending otherwise would only produce a half-empty release. What it
# replaces is the part that never needed a matrix in the first place: some two
# hundred lines of `mv` and `zip` that decide the layout of every archive the
# SDK ships -- and that six Players' scripts/download-sdk.sh read. Until this
# script existed, the only way to try a change to that layout was to push it and
# watch seven platforms rebuild; against a downloaded run the same check takes
# seconds, because the artifacts are already built and the layout is all that is
# being tested.
#
# Rerunning is safe, and re-running against the same download is the point:
# unlike the workflow this replaces, the input tree is copied from, never moved
# out of, so one download serves any number of attempts.
#
# The zips are byte-for-byte what a consumer downloads, so verify=yes (the
# default) opens them again afterwards and checks the handful of things that
# have actually broken releases before: a missing archive, a tree that lost the
# root layout download-sdk.sh navigates, a macOS framework whose version
# symlinks the zip dereferenced, a checksum that covers nothing.
set -euo pipefail

APP="$(basename "$0")"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# --- options --------------------------------------------------------------

IN="artifacts"
OUT="release-dist"
LICENSES="yes"
VERIFY="yes"
CLEAN="yes"

usage () {
  echo "Usage: $APP [options]"
  echo "$APP options:"
  echo "  in=<dir>                Where the per-platform artifact trees are (default:"
  echo "                          $IN). One subdirectory per upload-artifact name, which"
  echo "                          is exactly what 'gh run download -D <dir>' produces."
  echo "  out=<dir>               Where the zips land (default: $OUT). Relative paths"
  echo "                          resolve against the current directory."
  echo "  licenses=<yes|no>       Regenerate the third-party notices into"
  echo "                          target/licenses with cargo-about (default: $LICENSES)."
  echo "                          no reuses what is already there, which is how to run"
  echo "                          this without cargo-about installed."
  echo "  verify=<yes|no>         Re-open the finished zips and check them (default:"
  echo "                          $VERIFY)."
  echo "  clean=<yes|no>          Delete <out> before assembling (default: $CLEAN)."
    echo "  sdk= skip=              Accepted and ignored here. The family's build-release"
  echo "                          scripts share out= clean= verify= in= sdk= skip=,"
  echo "                          so one command line drives all of them."
echo "  -h | --help             Show this help"
  echo ""
  echo "Output: <out>/ holds the three SDK archives, one ssconverter-cli archive per"
  echo "platform, and SHA256SUMS over all of them -- the exact set release.yml attaches"
  echo "to the draft Release."
  echo ""
  echo "There is no option for which release this is: the tag comes from the checkout"
  echo "this runs in (the ref the workflow was dispatched from, or 'git checkout <tag>'"
  echo "locally), and the artifacts come from whatever populated in=. Both decisions"
  echo "belong to the step before this one."
  echo ""
  echo "Every artifact the layout names must be present: a partial release is not a"
  echo "thing worth producing, so a missing one is an error that lists everything else"
  echo "missing alongside it rather than failing on the first."
}

while [ $# -gt 0 ]; do
  item="$1"
  shift

  case "$item" in
    -h|--help|help)
      usage
      exit 0
      ;;
    *=*)
      key="${item%%=*}"
      value="${item#*=}"
      case "$key" in
        in)       IN="$value" ;;
        out)      OUT="$value" ;;
        # Lower-cased where the option is a keyword; in=/out= are paths.
        licenses) LICENSES="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')" ;;
        verify)   VERIFY="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')" ;;
        clean)    CLEAN="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')" ;;
        # Accepted so one command line drives build-release in every repository
        # of the family; this one has no use for them.
        sdk|skip) echo "$APP: ${key}= does not apply here; ignored" >&2 ;;
        *)
          echo "$APP: unknown option '$key' (see $APP --help)" >&2
          exit 2
          ;;
      esac
      ;;
    *)
      echo "$APP: unknown argument '$item' (options are key=value; see $APP --help)" >&2
      exit 2
      ;;
  esac
done

case "$LICENSES" in yes|no) ;; *) echo "$APP: licenses must be yes or no (got '$LICENSES')" >&2; exit 2 ;; esac
case "$VERIFY" in yes|no) ;; *) echo "$APP: verify must be yes or no (got '$VERIFY')" >&2; exit 2 ;; esac
case "$CLEAN" in yes|no) ;; *) echo "$APP: clean must be yes or no (got '$CLEAN')" >&2; exit 2 ;; esac

# Resolve both paths against the invoking directory, before anything cds away.
case "$IN"  in /*) ;; *) IN="$PWD/$IN" ;; esac
case "$OUT" in /*) ;; *) OUT="$PWD/$OUT" ;; esac

# clean=yes deletes out=, so refuse the two paths that would take the repository
# (or the filesystem) with them.
if [ "$OUT" = "/" ] || [ "$OUT" = "$ROOT_DIR" ]; then
  echo "$APP: refusing to use '$OUT' as out= (it is deleted when clean=yes)" >&2
  exit 2
fi

cd "$ROOT_DIR"

# A release is a tag: one is pushed, and the workflow is then dispatched from
# it. So the ref this is running on already IS the answer, and reading it here
# is reading the decision, not making it.
#
# GITHUB_REF_NAME is the authoritative form when there is one -- GitHub knows
# which ref the dispatch chose without depending on which refs the checkout
# happened to fetch. `git describe` is the same question asked locally, and
# --match keeps a stray non-release tag on the same commit from answering it.
#
# Neither is an error when absent: a dispatch off a branch is an artifact build,
# not a release (the workflow refuses to create a Release from one), and it
# still has to stamp something true -- the manifest version, which is what that
# branch's crates actually say they are.
MANIFEST_TAG="v$(./scripts/release-version.sh)"

if [ "${GITHUB_REF_TYPE:-}" = "tag" ] && [ -n "${GITHUB_REF_NAME:-}" ]; then
  TAG="$GITHUB_REF_NAME"
  TAG_SOURCE="GITHUB_REF_NAME -- the ref this release was dispatched from"
elif TAG="$(git -C "$ROOT_DIR" describe --exact-match --match 'v[0-9]*' --tags HEAD 2>/dev/null)"; then
  TAG_SOURCE="the release tag HEAD is checked out at"
else
  TAG="$MANIFEST_TAG"
  TAG_SOURCE="Cargo.toml -- HEAD is not on a release tag, so this is an artifact build"
fi

echo "options"
echo "  in       => $IN"
echo "  out      => $OUT"
echo "  licenses => $LICENSES"
echo "  verify   => $VERIFY"
echo "  clean    => $CLEAN"
echo "  tag      => $TAG"
echo "              ($TAG_SOURCE)"

# A tag that disagrees with the manifest is an error, not a note. The archives
# would be stamped with the tag while the crates inside call themselves
# something else, and every Player pins the SDK by that tag in its own
# scripts/SDK_VERSION.txt -- so the mismatch propagates to six repositories
# before anyone notices. This cannot fire off a branch: TAG falls back to
# MANIFEST_TAG when HEAD is not on a release tag.
if [ "$TAG" != "$MANIFEST_TAG" ]; then
  echo "" >&2
  echo "$APP: tag $TAG does not match Cargo.toml ($MANIFEST_TAG)." >&2
  echo "$APP: bump [workspace.package] version, or retag -- the archives would be" >&2
  echo "$APP: stamped $TAG, which is what every Player's SDK_VERSION.txt pins." >&2
  exit 1
fi

# --- preflight ------------------------------------------------------------

if [ ! -f "$ROOT_DIR/Cargo.toml" ] || [ ! -d "$ROOT_DIR/libs/ssruntime" ]; then
  echo "$APP: Cargo.toml or libs/ssruntime not found -- is this the SpriteStudio-SDK repo?" >&2
  exit 1
fi

for tool in zip unzip tar; do
  command -v "$tool" > /dev/null 2>&1 || {
    echo "$APP: $tool not found -- it is what packages the archives" >&2
    exit 1
  }
done

# The runners have sha256sum (coreutils); macOS ships shasum instead. Both write
# the same `<hash>  <name>` lines and both take -c, so SHA256SUMS is identical
# either way -- which is the only reason a release can be assembled from either.
if command -v sha256sum > /dev/null 2>&1; then
  sha256 () { sha256sum "$@"; }
elif command -v shasum > /dev/null 2>&1; then
  sha256 () { shasum -a 256 "$@"; }
else
  echo "$APP: neither sha256sum nor shasum found -- one of them writes SHA256SUMS" >&2
  exit 1
fi

if [ "$LICENSES" = "yes" ] && ! command -v cargo-about > /dev/null 2>&1; then
  echo "$APP: cargo-about not found -- install it with 'cargo install cargo-about --locked'," >&2
  echo "$APP: or pass licenses=no to reuse whatever target/licenses already holds" >&2
  exit 1
fi

step () {
  echo ""
  echo ">> $*"
}

# The input is one subdirectory per upload-artifact name -- the shape
# actions/download-artifact gives the package job, and the shape `gh run
# download -D <dir>` gives a local one. That is what lets one assembly path
# serve both, and it is why fetching is not this script's job.
if [ ! -d "$IN" ]; then
  echo "$APP: $IN does not exist" >&2
  echo "$APP: populate it first -- 'gh run download <run-id> -D $(basename "$IN")' -- or point in= at the artifacts" >&2
  exit 1
fi

# --- what the layout needs ------------------------------------------------
#
# Every line is `<source under in=>  <destination>`; a destination ending in /
# is a directory the source is copied into, keeping its own name. Sources are
# copied, never moved, so the input tree survives for the next run.
#
# The archive root is deliberately consumer-neutral -- libs/, include/,
# bindings/, VERSION, no wrapper directory -- because each Player's
# scripts/download-sdk.sh maps it into that Player's own build layout. Changing
# a path here changes six repositories.

STATIC_MAP="
spritestudio-sdk-libraries-macos/libssconverter.a            libs/macos/
spritestudio-sdk-libraries-macos/libssruntime.a              libs/macos/
spritestudio-sdk-libraries-web/libssruntime.a                libs/web/
spritestudio-sdk-libraries-ios/static/libssruntime.xcframework  libs/ios/
spritestudio-sdk-libraries-linux/libssconverter.a            libs/linux/x86_64/
spritestudio-sdk-libraries-linux/libssruntime.a              libs/linux/x86_64/
spritestudio-sdk-libraries-linux-aarch64/libssconverter.a    libs/linux/aarch64/
spritestudio-sdk-libraries-linux-aarch64/libssruntime.a      libs/linux/aarch64/
spritestudio-sdk-libraries-android/arm64/libssruntime.a      libs/android/arm64-v8a/
spritestudio-sdk-libraries-android/armv7/libssruntime.a      libs/android/armeabi-v7a/
spritestudio-sdk-libraries-android/x86_64/libssruntime.a     libs/android/x86_64/
spritestudio-sdk-libraries-windows/x86_64/ssconverter.lib    libs/windows/x86_64/
spritestudio-sdk-libraries-windows/x86_64/ssruntime.lib      libs/windows/x86_64/
spritestudio-sdk-libraries-windows/arm64/ssconverter.lib     libs/windows/arm64/
spritestudio-sdk-libraries-windows/arm64/ssruntime.lib       libs/windows/arm64/
"

# The macOS runtime ships as two wrappers of the same universal Mach-O: a
# .bundle (Unity's native-plugin format) and a versioned .framework (Ren'Py
# dlopens it by path; Flutter / CocoaPods link it via vendored_frameworks, which
# cannot consume a loadable .bundle). The bare standalone runtime dylib is no
# longer shipped -- every consumer moved off it. The bundle has no symlinks so a
# plain copy is fine; the framework arrives as a tar and is unpacked below, so
# its version symlinks and code signature survive the trip.
DYNAMIC_MAP="
spritestudio-sdk-libraries-macos/libssconverter.dylib        libs/macos/
spritestudio-sdk-libraries-macos/ssruntime.bundle            libs/macos/
spritestudio-sdk-libraries-ios/dynamic/libssruntime.xcframework  libs/ios/
spritestudio-sdk-libraries-linux/libssconverter.so           libs/linux/x86_64/
spritestudio-sdk-libraries-linux/libssruntime.so             libs/linux/x86_64/
spritestudio-sdk-libraries-linux-aarch64/libssconverter.so   libs/linux/aarch64/
spritestudio-sdk-libraries-linux-aarch64/libssruntime.so     libs/linux/aarch64/
spritestudio-sdk-libraries-android/arm64/libssruntime.so     libs/android/arm64-v8a/
spritestudio-sdk-libraries-android/armv7/libssruntime.so     libs/android/armeabi-v7a/
spritestudio-sdk-libraries-android/x86_64/libssruntime.so    libs/android/x86_64/
spritestudio-sdk-libraries-windows/x86_64/ssconverter.dll    libs/windows/x86_64/
spritestudio-sdk-libraries-windows/x86_64/ssruntime.dll      libs/windows/x86_64/
spritestudio-sdk-libraries-windows/arm64/ssconverter.dll     libs/windows/arm64/
spritestudio-sdk-libraries-windows/arm64/ssruntime.dll       libs/windows/arm64/
"

# The three wasm-pack packages are assembled by the web build job; they share
# file names (ssruntime.js, …), so they stay as subdirectories.
WEB_MAP="
spritestudio-sdk-libraries-web/runtime-web           runtime-web
spritestudio-sdk-libraries-web/runtime-web-nosimd    runtime-web-nosimd
spritestudio-sdk-libraries-web/converter-web         converter-web
"

# Not in a map because they are the one thing collected from a single platform
# rather than per-platform: cbindgen emits one plain-C header per crate (it
# carries `extern "C"` guards, so it serves C and C++ alike -- ffigen and the
# framework module map need this one), and csbindgen emits the C# FFI bindings
# SSPlayerForUnity compiles against. Both are platform-independent, so Linux is
# the canonical source.
CANONICAL="spritestudio-sdk-libraries-linux"

# The macOS framework, tarred by the build job so the final zip cannot
# dereference its symlinks and invalidate the code signature.
FRAMEWORK_TAR="spritestudio-sdk-libraries-macos/libssruntime.framework.tar"

# One archive per CLI artifact. `zip -rj` junks paths, which is why the Windows
# arm64 CLI is its own artifact: two ssconverter-cli.exe would collide at the
# zip root.
CLI_ARTIFACTS="
ssconverter-cli-linux
ssconverter-cli-linux-aarch64
ssconverter-cli-macos
ssconverter-cli-windows
ssconverter-cli-windows-arm64
"

# --- check the input ------------------------------------------------------
# Everything at once. Finding out about the fifth missing artifact on the fifth
# run of a script that takes a matrix build to feed is not a debugging loop
# worth having.
step "checking $IN"

MISSING=""
note_missing () {
  MISSING="${MISSING}
  $1"
}

check_map () { # <map>
  local src dest
  while read -r src dest; do
    [ -n "$src" ] || continue
    [ -e "$IN/$src" ] || note_missing "$src"
  done <<EOF
$1
EOF
}

check_map "$STATIC_MAP"
check_map "$DYNAMIC_MAP"
check_map "$WEB_MAP"

[ -e "$IN/$FRAMEWORK_TAR" ] || note_missing "$FRAMEWORK_TAR"

# Globs, so tested by expansion rather than by name.
ls "$IN/$CANONICAL/include"/*.h > /dev/null 2>&1 || note_missing "$CANONICAL/include/*.h"
ls "$IN/$CANONICAL/bindings"/*.g.cs > /dev/null 2>&1 || note_missing "$CANONICAL/bindings/*.g.cs"

for cli in $CLI_ARTIFACTS; do
  [ -d "$IN/$cli" ] || note_missing "$cli/"
done

if [ -n "$MISSING" ]; then
  echo "$APP: $IN is missing pieces the release layout needs:$MISSING" >&2
  echo "" >&2
  echo "$APP: point in= at a complete matrix build -- 'gh run download <run-id> -D $(basename "$IN")'." >&2
  exit 1
fi
echo "   every artifact the layout names is present"

# --- licenses -------------------------------------------------------------
# Generated fresh from the current Cargo.lock into target/licenses (not
# committed). Each shipped archive carries the notice for the crates it holds,
# pinned to the features those crates actually ship with.
LICENSE_DIR="$ROOT_DIR/target/licenses"

if [ "$LICENSES" = "yes" ]; then
  step "generating third-party license notices"
  ./scripts/generate-licenses.sh
else
  step "reusing the third-party license notices in target/licenses"
fi

for notice in ssruntime ssconverter cli; do
  if [ ! -f "$LICENSE_DIR/THIRD-PARTY-LICENSES.$notice.md" ]; then
    echo "$APP: $LICENSE_DIR/THIRD-PARTY-LICENSES.$notice.md is missing" >&2
    [ "$LICENSES" = "no" ] && echo "$APP: licenses=no reuses these -- drop it to regenerate them" >&2
    exit 1
  fi
done

# --- assemble -------------------------------------------------------------
WORK="$ROOT_DIR/target/release-staging"

step "assembling the trees in $WORK"

rm -rf "$WORK"
mkdir -p "$WORK/static" "$WORK/dynamic" "$WORK/web"

place_map () { # <map> <tree-root>
  local map="$1" root="$2" src dest
  while read -r src dest; do
    [ -n "$src" ] || continue
    case "$dest" in
      */) mkdir -p "$root/$dest" ;;
      *)  mkdir -p "$(dirname "$root/$dest")" ;;
    esac
    cp -R "$IN/$src" "$root/$dest"
  done <<EOF
$map
EOF
}

# The documents every SDK archive carries at its root.
place_docs () { # <tree-root>
  local root="$1"
  printf '%s\n' "$TAG" > "$root/VERSION"
  cp "$ROOT_DIR/LICENSE.md" "$ROOT_DIR/LICENSE.ja.md" "$root/"
  cp "$LICENSE_DIR/THIRD-PARTY-LICENSES.ssruntime.md" "$root/"
  cp "$LICENSE_DIR/THIRD-PARTY-LICENSES.ssconverter.md" "$root/"
}

place_headers () { # <tree-root>
  local root="$1"
  mkdir -p "$root/include" "$root/bindings"
  cp "$IN/$CANONICAL/include"/*.h "$root/include/"
  cp "$IN/$CANONICAL/bindings"/*.g.cs "$root/bindings/"
}

place_map "$STATIC_MAP" "$WORK/static"
place_headers "$WORK/static"
place_docs "$WORK/static"
echo "   static/"

place_map "$DYNAMIC_MAP" "$WORK/dynamic"
tar -C "$WORK/dynamic/libs/macos" -xf "$IN/$FRAMEWORK_TAR"
place_headers "$WORK/dynamic"
place_docs "$WORK/dynamic"
echo "   dynamic/  (libssruntime.framework untarred)"

place_map "$WEB_MAP" "$WORK/web"
# Named per crate, as in the static and dynamic archives -- the names the
# Players' notices point to and their build-runtime scripts write, so a web
# package that download-sdk installs carries the file its notice names.
cp "$LICENSE_DIR/THIRD-PARTY-LICENSES.ssruntime.md"   "$WORK/web/runtime-web/"
cp "$LICENSE_DIR/THIRD-PARTY-LICENSES.ssruntime.md"   "$WORK/web/runtime-web-nosimd/"
cp "$LICENSE_DIR/THIRD-PARTY-LICENSES.ssconverter.md" "$WORK/web/converter-web/"
echo "   web/"

# --- archive --------------------------------------------------------------
step "archiving into $OUT"

if [ "$CLEAN" = "yes" ]; then
  rm -rf "$OUT"
fi
mkdir -p "$OUT"

# Zip the tree at the archive root -- no wrapper directory, see the map above.
( cd "$WORK/static" && zip -qr  "$OUT/spritestudio-sdk-static-libs.zip"  . )
# -y stores symlinks rather than following them, so the macOS
# libssruntime.framework keeps its version symlinks and its code signature.
# download-sdk.sh's `unzip` restores them on extract.
( cd "$WORK/dynamic" && zip -qry "$OUT/spritestudio-sdk-dynamic-libs.zip" . )
( cd "$WORK/web" && zip -qr  "$OUT/spritestudio-sdk-web-libs.zip"     . )
echo "   spritestudio-sdk-{static,dynamic,web}-libs.zip"

for cli in $CLI_ARTIFACTS; do
  platform="${cli#ssconverter-cli-}"
  staging="$WORK/cli/$platform"
  mkdir -p "$staging"
  cp -R "$IN/$cli/." "$staging/"
  cp "$ROOT_DIR/LICENSE.md" "$ROOT_DIR/LICENSE.ja.md" "$staging/"
  cp "$LICENSE_DIR/THIRD-PARTY-LICENSES.cli.md" "$staging/THIRD-PARTY-LICENSES.md"
  # The executable bit does not survive a Windows-built artifact upload, and a
  # CLI that unzips without it is a support ticket. Absent on the Windows zips,
  # hence the guard rather than a bare chmod.
  [ -f "$staging/ssconverter-cli" ] && chmod +x "$staging/ssconverter-cli"
  ( cd "$staging" && zip -qrj "$OUT/$cli.zip" . )
  echo "   $cli.zip"
done

# Canonical checksum manifest so a consumer can verify each downloaded zip's
# integrity (`sha256sum -c SHA256SUMS --ignore-missing` for a partial download).
# Generated last, over every zip in the output directory.
( cd "$OUT" && sha256 ./*.zip | sed 's|\./||' > SHA256SUMS )
echo "   SHA256SUMS"

# --- verify ---------------------------------------------------------------
if [ "$VERIFY" = "yes" ]; then
  step "verifying $OUT"

  FAILED=""
  fail () {
    FAILED="${FAILED}
  $1"
  }

  # Every archive present and not empty. `zip` happily writes an archive with
  # nothing in it, which then unzips into nothing at all on the consumer side.
  for zipfile in spritestudio-sdk-static-libs.zip \
                 spritestudio-sdk-dynamic-libs.zip \
                 spritestudio-sdk-web-libs.zip \
                 $(for cli in $CLI_ARTIFACTS; do echo "$cli.zip"; done); do
    if [ ! -s "$OUT/$zipfile" ]; then
      fail "$zipfile is missing or empty"
      continue
    fi
    if [ "$(unzip -Z1 "$OUT/$zipfile" | wc -l | tr -d ' ')" = "0" ]; then
      fail "$zipfile contains no entries"
    fi
  done

  # The root layout download-sdk.sh navigates, in the two archives that have one.
  for zipfile in spritestudio-sdk-static-libs.zip spritestudio-sdk-dynamic-libs.zip; do
    [ -s "$OUT/$zipfile" ] || continue
    entries="$(unzip -Z1 "$OUT/$zipfile")"
    for expected in VERSION LICENSE.md LICENSE.ja.md libs/ include/ bindings/; do
      case "$expected" in
        */) printf '%s\n' "$entries" | grep -q "^$expected" || fail "$zipfile has no $expected at the archive root" ;;
        *)  printf '%s\n' "$entries" | grep -qx "$expected"  || fail "$zipfile has no $expected at the archive root" ;;
      esac
    done
  done

  # The framework's version symlinks. zipinfo marks a stored symlink with a
  # leading `l`; if -y were ever dropped from the dynamic zip above, every one
  # of these would come back as a regular file and the code signature with it.
  SYMLINKS="$(unzip -Z "$OUT/spritestudio-sdk-dynamic-libs.zip" | grep -c '^l' || true)"
  if [ "$SYMLINKS" = "0" ]; then
    fail "spritestudio-sdk-dynamic-libs.zip stores no symlinks -- libssruntime.framework was flattened (zip -y)"
  else
    echo "   dynamic: $SYMLINKS symlinks stored (libssruntime.framework intact)"
  fi

  # The VERSION stamp consumers read back.
  STAMPED="$(unzip -p "$OUT/spritestudio-sdk-static-libs.zip" VERSION | tr -d ' \r\n')"
  if [ "$STAMPED" != "$TAG" ]; then
    fail "static VERSION says '$STAMPED', expected '$TAG'"
  else
    echo "   VERSION stamp: $STAMPED"
  fi

  # The manifest covers every archive, and every hash matches.
  ( cd "$OUT" && sha256 -c SHA256SUMS > /dev/null ) || fail "SHA256SUMS does not verify"
  LISTED="$(wc -l < "$OUT/SHA256SUMS" | tr -d ' ')"
  PRESENT="$(ls -1 "$OUT"/*.zip | wc -l | tr -d ' ')"
  if [ "$LISTED" != "$PRESENT" ]; then
    fail "SHA256SUMS lists $LISTED files but $PRESENT zips are present"
  else
    echo "   SHA256SUMS: $LISTED archives, all verified"
  fi

  if [ -n "$FAILED" ]; then
    echo "" >&2
    echo "$APP: the release in $OUT is not shippable:$FAILED" >&2
    exit 1
  fi
  echo "   all checks passed"
else
  step "verify=no -- the archives are not checked"
fi

# --- done -----------------------------------------------------------------
echo ""
echo "$APP: done -> $OUT  ($TAG)"
echo ""
( cd "$OUT" && ls -l ./*.zip SHA256SUMS | sed 's|^|  |' )
echo ""
echo "$APP: this is what release.yml attaches to the draft Release for $TAG."
echo "$APP: look inside one before shipping it:"
echo "  unzip -l $OUT/spritestudio-sdk-dynamic-libs.zip"
