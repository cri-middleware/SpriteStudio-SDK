#!/usr/bin/env bash
#
# Run the checks a change has to pass before it is pushed: the ones a pull
# request runs in .github/workflows/ci.yml, and on request the slower ones it
# leaves out. PowerShell twin: check.ps1.
#
#   scripts/check.sh                # ci.yml's lint job and its workspace tests
#   scripts/check.sh features=yes   # ... and ssruntime's feature sets, one at a time
#   scripts/check.sh msrv=yes       # ... and the toolchain Cargo.toml declares
#   scripts/check.sh docs=yes       # ... and the Player Porting Guide, both locales
#
# Every check runs, whatever failed before it, and the summary at the end names
# each one as passed, FAILED or skipped. A check whose prerequisite is missing
# (the wasm32 target, the MSRV toolchain) is skipped rather than failed, so read
# the summary, not the exit status: 0 says that nothing failed, not that
# everything ran.
#
# The default set is ci.yml's lint job plus its Test step; keep the two in step.
# features=yes goes further than CI does. --all-features compiles every cfg at
# once, so it hides code that only one feature set compiles, or leaves dead; CI
# builds a few of the sets and lints none of them.
set -uo pipefail

APP="$(basename "$0")"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# --- options --------------------------------------------------------------
FEATURES="no"
MSRV="no"
DOCS="no"

usage () {
  echo "Usage: $APP [options]"
  echo "$APP options:"
  echo "  features=<yes|no>   Also lint and document ssruntime with each feature set on"
  echo "                      its own, native and wasm32, and lint the browser converter"
  echo "                      (default: $FEATURES)."
  echo "  msrv=<yes|no>       Also cargo check the workspace with the rust-version in"
  echo "                      Cargo.toml, as ci.yml's msrv job does (default: $MSRV)."
  echo "  docs=<yes|no>       Also build the Player Porting Guide in both locales, with"
  echo "                      scripts/build-docs.sh (default: $DOCS). Nothing builds it"
  echo "                      on a pull request, so run this when libs/ssruntime/docs"
  echo "                      changed."
  echo "  -h | --help         Show this help"
  echo ""
  echo "Always run: cargo fmt --check, the license copies, clippy and rustdoc with"
  echo "warnings denied, and the workspace tests."
  echo ""
  echo "Exit status: 0 when no check failed, 1 when one did, 2 on a usage error. A"
  echo "skipped check does not fail the run; the summary lists it with the reason."
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
      value="$(printf '%s' "${item#*=}" | tr '[:upper:]' '[:lower:]')"
      case "$key" in
        features) FEATURES="$value" ;;
        msrv)     MSRV="$value" ;;
        docs)     DOCS="$value" ;;
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

case "$FEATURES" in yes|no) ;; *) echo "$APP: features must be yes or no (got '$FEATURES')" >&2; exit 2 ;; esac
case "$MSRV" in yes|no) ;; *) echo "$APP: msrv must be yes or no (got '$MSRV')" >&2; exit 2 ;; esac
case "$DOCS" in yes|no) ;; *) echo "$APP: docs must be yes or no (got '$DOCS')" >&2; exit 2 ;; esac

echo "options"
echo "  features => $FEATURES"
echo "  msrv     => $MSRV"
echo "  docs     => $DOCS"

# --- preflight ------------------------------------------------------------
if [ ! -f "$ROOT_DIR/Cargo.toml" ] || [ ! -f "$ROOT_DIR/libs/ssruntime/Cargo.toml" ]; then
  echo "$APP: Cargo.toml not found -- is this the SpriteStudio-SDK repo?" >&2
  exit 1
fi
if ! command -v cargo > /dev/null 2>&1; then
  echo "$APP: cargo not found -- install the Rust toolchain (https://rustup.rs)" >&2
  exit 1
fi
cd "$ROOT_DIR" || exit 1

# --- runners --------------------------------------------------------------
# Plain strings rather than arrays: macOS's bash 3.2 treats an empty array as
# unset under `set -u`.
SUMMARY=""
PASSED=0
FAILED=0
SKIPPED=0

record () {
  SUMMARY="${SUMMARY}  $1  $2"$'\n'
}

# check <label> <command> [args...]
check () {
  local label="$1"
  shift
  echo ""
  echo "==> $label"
  echo "    $*"
  if "$@"; then
    PASSED=$((PASSED + 1))
    record "passed " "$label"
  else
    FAILED=$((FAILED + 1))
    record "FAILED " "$label"
  fi
}

# skip <label> <reason>
skip () {
  echo ""
  echo "==> $1: skipped -- $2"
  SKIPPED=$((SKIPPED + 1))
  record "skipped" "$1 -- $2"
}

# `cargo package` and wasm-pack ship each crate's own copy of the root LICENSE
# files, which scripts/generate-licenses.sh keeps in sync.
license_copies_match () {
  local status=0 copy root
  while IFS= read -r copy; do
    root="$(basename "$copy")"
    if ! cmp -s "$root" "$copy"; then
      echo "$copy differs from the root $root: copy it over, or run scripts/generate-licenses.sh" >&2
      status=1
    fi
  done < <(git ls-files '*/LICENSE.md' '*/LICENSE.ja.md')
  return $status
}

# --- what ci.yml runs on a pull request -----------------------------------
check "fmt" cargo fmt --all -- --check
check "license copies" license_copies_match
check "clippy (workspace)" cargo clippy --workspace --all-targets --all-features -- -D warnings
check "rustdoc (workspace)" env RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
check "tests (workspace)" cargo test --workspace --all-features -- --test-threads=1

# --- features=yes ---------------------------------------------------------
if [ "$FEATURES" = "yes" ]; then
  # With the tests, so a test-only item that one set leaves unused is caught.
  # ${combo:+...} passes no --features at all for the empty set.
  for combo in "" ffi std ffi,std web web,std ffi,web ffi,web,std; do
    check "clippy ssruntime [${combo:-none}]" cargo clippy -p ssruntime --all-targets --no-default-features ${combo:+--features "$combo"} -- -D warnings
  done
  # The allocator sets are for the no_std binaries; a test build already has
  # std's allocator, so they are linted as a library only.
  for combo in ffi,libc_alloc,panic-handler ffi,custom_alloc,panic-handler; do
    check "clippy ssruntime [$combo]" cargo clippy -p ssruntime --no-default-features --features "$combo" -- -D warnings
  done
  for combo in "" web std ffi ffi,web; do
    check "rustdoc ssruntime [${combo:-none}]" env RUSTDOCFLAGS="-D warnings" cargo doc -p ssruntime --no-deps --no-default-features ${combo:+--features "$combo"}
  done

  WASM=wasm32-unknown-unknown
  if ! command -v rustup > /dev/null 2>&1; then
    skip "clippy for $WASM" "rustup not found, so the target cannot be checked for"
  elif ! targets="$(rustup target list --installed)"; then
    skip "clippy for $WASM" "rustup target list failed"
  elif ! printf '%s\n' "$targets" | grep -qx "$WASM"; then
    skip "clippy for $WASM" "the target is not installed (rustup target add $WASM)"
  else
    for combo in web ffi,wasm_alloc,panic-handler ffi,std; do
      check "clippy ssruntime $WASM [$combo]" cargo clippy -p ssruntime --target "$WASM" --no-default-features --features "$combo" -- -D warnings
    done
    check "clippy ssconverter $WASM [web]" cargo clippy -p ssconverter --target "$WASM" --no-default-features --features web -- -D warnings
  fi
fi

# --- msrv=yes -------------------------------------------------------------
if [ "$MSRV" = "yes" ]; then
  version="$(grep -E '^rust-version = ' Cargo.toml | sed -E 's/rust-version = "(.+)"/\1/')"
  if [ -z "$version" ]; then
    echo ""
    echo "==> msrv: rust-version not found in the workspace Cargo.toml" >&2
    FAILED=$((FAILED + 1))
    record "FAILED " "msrv -- rust-version not found in the workspace Cargo.toml"
  elif ! command -v rustup > /dev/null 2>&1; then
    skip "msrv $version" "rustup not found"
  elif ! toolchains="$(rustup toolchain list)"; then
    skip "msrv $version" "rustup toolchain list failed"
  elif ! printf '%s\n' "$toolchains" | grep -q "^${version}-"; then
    skip "msrv $version" "the toolchain is not installed (rustup toolchain install $version)"
  else
    check "msrv $version" cargo "+$version" check --workspace --all-targets --all-features --locked
  fi
fi

# --- docs=yes -------------------------------------------------------------
if [ "$DOCS" = "yes" ]; then
  check "docs (both locales)" "$ROOT_DIR/scripts/build-docs.sh"
fi

# --- summary --------------------------------------------------------------
echo ""
echo "summary"
printf '%s' "$SUMMARY"
echo "$PASSED passed, $FAILED failed, $SKIPPED skipped"
[ "$FAILED" -eq 0 ]
