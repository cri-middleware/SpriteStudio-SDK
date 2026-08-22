#!/usr/bin/env bash
#
# Build the GitHub Pages site locally -- everything .github/workflows/pages.yml
# publishes, in the same order: the documentation in both locales into site/,
# then the workspace rustdoc into site/api/.
#
#   scripts/build-pages.sh              # docs + rustdoc
#   scripts/build-pages.sh serve=yes    # ... and serve site/ on :8000
#   scripts/build-pages.sh api=no       # documentation only, much faster
#   scripts/build-pages.sh docs=no      # rustdoc only, into the site already there
#
# Two builds that know nothing about each other, and one tree they both land in.
# scripts/build-docs.sh generates the per-locale guide trees and runs zensical
# over them; `cargo doc` renders the workspace. Neither is aware of the other,
# which is exactly why the order matters and why it is written down in one place:
# the English documentation build clears site/, and both the Japanese site
# (site/ja) and the rustdoc (site/api) live inside it -- so the rustdoc goes in
# last, exactly as in pages.yml.
#
# Until this script existed, that graft lived only in the workflow, as a `run:`
# block with a heredoc in it. A locally built site therefore had no /api/ at all
# and the nav entry pointing into it went nowhere: the one part of the published
# site nobody could look at before publishing it.
#
# What this does NOT reproduce is the deploy job (actions/deploy-pages) -- there
# is no local equivalent of publishing. Serving site/ over HTTP is as close as a
# local run gets, and it is enough to check every link, both locales, the
# language selector and the rustdoc the nav points at.
#
# Rerunning is safe; nothing here is incremental (`cargo doc` is, and is left to
# manage its own target directory).
set -euo pipefail

APP="$(basename "$0")"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# --- options --------------------------------------------------------------
DOCS="yes"
API="yes"
PREPARE="auto"
VENV=".venv"
SERVE="no"
PORT="8000"

usage () {
  echo "Usage: $APP [options]"
  echo "$APP options:"
  echo "  docs=<yes|no>           Build the documentation, both locales (default: $DOCS)."
  echo "                          no keeps whatever site/ already holds -- handy when"
  echo "                          only the rustdoc is being re-checked."
  echo "  api=<yes|no>            Build the workspace rustdoc and graft it onto the site"
  echo "                          at site/api/ (default: $API). api=no leaves the nav's"
  echo "                          Rust API entry pointing at nothing, which is what a"
  echo "                          plain build-docs.sh run gives you."
  echo "  prepare=<auto|yes|no>   Run prepare-docs.sh first (default: $PREPARE)."
  echo "                          auto = only when no zensical is found."
  echo "  venv=<dir>              Virtualenv for the docs toolchain, relative to the"
  echo "                          repo root (default: $VENV). Passed to prepare-docs.sh"
  echo "                          and build-docs.sh."
  echo "  serve=<yes|no>          Serve site/ over HTTP when the build is done"
  echo "                          (default: $SERVE). Foreground; Ctrl-C to stop."
  echo "  port=<n>                Port for serve=yes (default: $PORT)"
  echo "  -h | --help             Show this help"
  echo ""
  echo "Output: site/ (en) + site/ja/ (ja) + site/api/ (rustdoc) -- the exact tree"
  echo "pages.yml uploads. Both documentation builds are --strict, so a broken internal"
  echo "link or a nav entry pointing at a missing page fails the build here rather than"
  echo "on a release publish."
  echo ""
  echo "The rustdoc is deliberately built WITHOUT RUSTDOCFLAGS=-D warnings, matching"
  echo "pages.yml: that gate belongs on the pull request (ci.yml), not on a publish."
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
        # Lower-cased where the option is a keyword; venv=/port= are not.
        docs)    DOCS="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')" ;;
        api)     API="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')" ;;
        prepare) PREPARE="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')" ;;
        venv)    VENV="$value" ;;
        serve)   SERVE="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')" ;;
        port)    PORT="$value" ;;
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

case "$DOCS" in yes|no) ;; *) echo "$APP: docs must be yes or no (got '$DOCS')" >&2; exit 2 ;; esac
case "$API" in yes|no) ;; *) echo "$APP: api must be yes or no (got '$API')" >&2; exit 2 ;; esac
case "$PREPARE" in auto|yes|no) ;; *) echo "$APP: prepare must be auto, yes or no (got '$PREPARE')" >&2; exit 2 ;; esac
case "$SERVE" in yes|no) ;; *) echo "$APP: serve must be yes or no (got '$SERVE')" >&2; exit 2 ;; esac
case "$PORT" in ''|*[!0-9]*) echo "$APP: port must be a number (got '$PORT')" >&2; exit 2 ;; esac

# Nothing to build at all is a typo, not a request: it would leave site/ exactly
# as it was and report success.
if [ "$DOCS" = "no" ] && [ "$API" = "no" ]; then
  echo "$APP: docs=no api=no builds nothing (see $APP --help)" >&2
  exit 2
fi

PREPARE_NOTE=""
[ "$DOCS" = "yes" ] || PREPARE_NOTE=" (unused: docs=no)"
SERVE_NOTE=""
[ "$SERVE" = "no" ] || SERVE_NOTE=" (port $PORT)"

echo "options"
echo "  docs    => $DOCS"
echo "  api     => $API"
echo "  prepare => $PREPARE$PREPARE_NOTE"
echo "  venv    => $VENV"
echo "  serve   => $SERVE$SERVE_NOTE"
echo ""

# --- preflight ------------------------------------------------------------
if [ ! -f "$ROOT_DIR/mkdocs.base.yml" ] || [ ! -f "$ROOT_DIR/Cargo.toml" ]; then
  echo "$APP: mkdocs.base.yml or Cargo.toml not found -- is this the SpriteStudio-SDK repo?" >&2
  exit 1
fi

if [ "$API" = "yes" ] && ! command -v cargo > /dev/null 2>&1; then
  echo "$APP: cargo not found -- install the Rust toolchain (https://rustup.rs), or" >&2
  echo "$APP: build the documentation on its own with '$APP api=no'" >&2
  exit 1
fi

# Every script here resolves its own paths from the repo root, and site/ is
# written relative to it, so run from there however this was invoked.
cd "$ROOT_DIR"

SITE_DIR="$ROOT_DIR/site"
API_DIR="$SITE_DIR/api"

step () {
  echo ""
  echo ">> $*"
  "$@"
}

# --- documentation --------------------------------------------------------
# Same probe as build-docs.sh: a venv laid out either way, else PATH (which is
# how a run with no venv at all still works).
have_zensical () {
  local venv_bin="$ROOT_DIR/$VENV/bin"
  [ -d "$ROOT_DIR/$VENV/Scripts" ] && venv_bin="$ROOT_DIR/$VENV/Scripts"
  [ -x "$venv_bin/zensical" ] || [ -x "$venv_bin/zensical.exe" ] \
    || command -v zensical > /dev/null 2>&1
}

if [ "$DOCS" = "yes" ]; then
  case "$PREPARE" in
    yes)
      step ./scripts/prepare-docs.sh venv="$VENV"
      ;;
    auto)
      if have_zensical; then
        echo "$APP: zensical already available; skipping prepare-docs.sh (prepare=yes forces it)"
      else
        step ./scripts/prepare-docs.sh venv="$VENV"
      fi
      ;;
    no) ;;
  esac

  # The guide trees and both locales, English first -- build-docs.sh owns that
  # order.
  step ./scripts/build-docs.sh venv="$VENV"
else
  echo "$APP: docs=no -- site/ keeps whatever the last build left there"
  if [ ! -f "$SITE_DIR/index.html" ]; then
    echo "$APP: ...and there is no site/index.html, so the tree below is a bare /api/" >&2
  fi
fi

# --- rustdoc --------------------------------------------------------------
# `cargo doc` renders into <target>/doc; the site wants it at site/api/. A local
# CARGO_TARGET_DIR is honoured, because plenty of working trees set one and the
# hardcoded `target/` pages.yml can rely on would then be empty.
build_api () {
  step cargo doc --workspace --no-deps --all-features

  local doc_dir="${CARGO_TARGET_DIR:-$ROOT_DIR/target}/doc"
  if [ ! -d "$doc_dir" ]; then
    echo "$APP: cargo doc produced no $doc_dir" >&2
    exit 1
  fi

  echo ""
  echo ">> copying $doc_dir/ into site/api/"
  rm -rf "$API_DIR"
  mkdir -p "$API_DIR"
  cp -R "$doc_dir/." "$API_DIR/"

  # `cargo doc` writes no index.html at the root of a workspace -- with several
  # crates there is no single page to land on. The nav entry in both locale
  # configs links straight to ssruntime, and this redirect covers anyone who
  # trims the URL back to /api/. Keep it identical to the one in pages.yml.
  #
  # Five of the seven workspace members land here: libs/ssruntime-ffi and
  # libs/ssconverter-ffi set `doc = false`, deliberately. They are cdylib +
  # staticlib with no rlib, so nothing can `use` them from Rust; their surface is
  # the C API, documented by the cbindgen headers and the Player Porting Guide.
  # More to the point, each renames its lib to the crate it wraps (`ssruntime`,
  # `ssconverter`), so documenting them would write two crates into one
  # target/doc/ssruntime and clobber the real one.
  cat > "$API_DIR/index.html" <<'HTML'
<!doctype html>
<meta charset="utf-8">
<title>SpriteStudio SDK — Rust API</title>
<meta http-equiv="refresh" content="0; url=ssruntime/index.html">
<link rel="canonical" href="ssruntime/index.html">
<a href="ssruntime/index.html">SpriteStudio SDK Rust API reference</a>
HTML
}

if [ "$API" = "yes" ]; then
  build_api
else
  echo ""
  echo "$APP: api=no -- site/api is not part of this build"
fi

# --- what landed ----------------------------------------------------------
echo ""
echo "$APP: done -> site/"
report () { # <path> <label>
  if [ -e "$ROOT_DIR/$1" ]; then
    echo "  $2: $1"
  else
    echo "  $2: MISSING ($1)"
  fi
}
report "site/index.html" "en  "
report "site/ja/index.html" "ja  "
if [ "$API" = "yes" ]; then
  report "site/api/ssruntime/index.html" "api "
fi

# --- serve ----------------------------------------------------------------
PYTHON_CMD="python3"
command -v "$PYTHON_CMD" > /dev/null 2>&1 || PYTHON_CMD="python"

echo ""
if [ "$SERVE" = "yes" ]; then
  command -v "$PYTHON_CMD" > /dev/null 2>&1 || {
    echo "$APP: no python on PATH to serve with -- use any static server on site/" >&2
    exit 1
  }
  echo "$APP: http://localhost:$PORT/  (docs)"
  echo "$APP: http://localhost:$PORT/ja/  (Japanese)"
  echo "$APP: http://localhost:$PORT/api/  (Rust API)"
  echo "$APP: Ctrl-C to stop"
  echo ""
  exec "$PYTHON_CMD" -m http.server "$PORT" -d site
else
  echo "$APP: serve it (one server over the whole tree, so the language selector and"
  echo "$APP: the nav's Rust API entry both resolve):"
  echo "$APP:   $APP serve=yes"
  echo "$APP:   $PYTHON_CMD -m http.server $PORT -d site"
fi
