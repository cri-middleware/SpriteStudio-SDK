#!/usr/bin/env bash
#
# Build the documentation site -- one run per locale, in the order that works.
#
#   scripts/build-docs.sh              # English then Japanese, both strict
#   scripts/build-docs.sh locale=ja    # Japanese only, into site/ja
#
# Three things this script exists to get right:
#
#   1. The generated trees. The Player Porting Guide is authored under
#      libs/ssruntime/docs with each page next to its .ja.md translation, which
#      is what makes it readable on GitHub; Zensical builds one locale per
#      config, from one directory. scripts/build-docs-tree.py materialises
#      build/docs/{en,ja}, and both configs' docs_dir points there -- so it runs
#      first, every time, or the site is built from a stale tree.
#   2. Order. The Japanese site is written *inside* the English one (site/ja),
#      and a build clears its own output directory first -- so `zensical build`
#      deletes site/ja on its way in. English first, Japanese second; the other
#      order publishes a site with no Japanese pages at all.
#   3. --strict, always. mkdocs.base.yml sets strict: true as well; the flag is
#      spelled out here so the build stays strict if the config ever loses it.
#
# What this does not build is the rustdoc the published site carries at /api/:
# that is `cargo doc --workspace --no-deps --all-features`, copied into site/api
# after both locale builds. The nav's Rust API entry is a site-relative path, so a
# site built by this script alone has no /api/ for it to reach and it 404s --
# scripts/build-pages.sh is this script plus that graft, in that order, and is
# what pages.yml runs.
#
# The -docs suffix keeps this apart from the scripts that build the code itself;
# the whole SpriteStudio family spells the documentation pair the same way.
#
# `zensical serve` is deliberately not wrapped: it validates nothing (--strict is
# accepted and does nothing), it serves one locale at a time, and it is a
# foreground process with no arguments worth defaulting.
set -euo pipefail

APP="$(basename "$0")"
ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

# --- options --------------------------------------------------------------
LOCALE="all"
VENV=".venv"

usage () {
  echo "Usage: $APP [options]"
  echo "$APP options:"
  echo "  locale=<all|en|ja>      Which locale to build (default: $LOCALE)"
  echo "                          all = English then Japanese, the only safe order"
  echo "  venv=<dir>              Virtualenv to run zensical from, relative to the repo"
  echo "                          root (default: $VENV). Falls back to zensical on PATH,"
  echo "                          where a plain pip install -r leaves it."
  echo "  -h | --help             Show this help"
  echo ""
  echo "Output: site/ (English) and site/ja/ (Japanese). Both builds are --strict, so a"
  echo "broken internal link fails the build; a nav entry pointing at a missing page is"
  echo "caught a step earlier, by build-docs-tree.py, which --strict does not catch at all."
  echo ""
  echo "locale=en on its own DELETES site/ja: the English build clears site/, and the"
  echo "Japanese site lives inside it. Use locale=all before serving or publishing."
  echo ""
  echo "Set the venv up first with scripts/prepare-docs.sh."
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
        locale) LOCALE="$(printf '%s' "$value" | tr '[:upper:]' '[:lower:]')" ;;
        venv)   VENV="$value" ;;
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

case "$LOCALE" in
  all|both) LOCALE="all" ;;
  en|english) LOCALE="en" ;;
  ja|japanese) LOCALE="ja" ;;
  *) echo "$APP: locale must be all, en or ja (got '$LOCALE')" >&2; exit 2 ;;
esac

echo "options"
echo "  locale => $LOCALE"
echo "  venv   => $VENV"
echo ""

# --- preflight ------------------------------------------------------------
# Same probe as prepare-docs.sh: bin/ everywhere, Scripts/ for a venv built by
# the Windows launcher.
VENV_DIR="$ROOT_DIR/$VENV"
if [ -d "$VENV_DIR/Scripts" ]; then
  VENV_BIN="$VENV_DIR/Scripts"
else
  VENV_BIN="$VENV_DIR/bin"
fi

ZENSICAL=""
for candidate in "$VENV_BIN/zensical" "$VENV_BIN/zensical.exe"; do
  if [ -x "$candidate" ]; then
    ZENSICAL="$candidate"
    break
  fi
done

# A plain `pip install -r docs/requirements.txt` leaves zensical on PATH with no
# venv. Fall back to it rather than demanding a venv.
if [ -z "$ZENSICAL" ] && command -v zensical > /dev/null 2>&1; then
  ZENSICAL="$(command -v zensical)"
  echo "$APP: no venv at $VENV; using zensical from PATH"
fi

if [ -z "$ZENSICAL" ]; then
  echo "$APP: zensical not found in $VENV or on PATH" >&2
  echo "$APP: run 'scripts/prepare-docs.sh' first" >&2
  exit 1
fi

# The generator imports nothing outside the standard library, so any Python 3
# runs it. Prefer the venv's, so prepare-docs.sh is enough on a machine whose
# PATH has no python3 at all.
PYTHON="$VENV_BIN/python"
[ -x "$PYTHON" ] || PYTHON="$VENV_BIN/python.exe"
if [ ! -x "$PYTHON" ]; then
  if command -v python3 > /dev/null 2>&1; then
    PYTHON="python3"
  else
    echo "$APP: no Python 3 in $VENV or on PATH, and the guide tree has to be generated" >&2
    echo "$APP: run 'scripts/prepare-docs.sh' first" >&2
    exit 1
  fi
fi

# Every path in the configs is relative to the repo root, so build from there
# however the script was invoked.
cd "$ROOT_DIR"

# --- generate -------------------------------------------------------------
# Both locale trees, before either build and regardless of locale=: the configs
# read build/docs/en and build/docs/ja, and neither exists in a fresh checkout.
# It also fails loudly on a guide page that lost its language switcher line, and on a
# nav entry and a generated page that do not agree -- the dead nav link --strict misses.
echo ">> $(basename "$PYTHON") scripts/build-docs-tree.py -> build/docs/{en,ja}"
"$PYTHON" scripts/build-docs-tree.py

# --- build ----------------------------------------------------------------
if [ "$LOCALE" = "all" ] || [ "$LOCALE" = "en" ]; then
  echo ">> zensical build --strict (en) -> site/"
  "$ZENSICAL" build --strict
fi

if [ "$LOCALE" = "all" ] || [ "$LOCALE" = "ja" ]; then
  echo ">> zensical build -f mkdocs.ja.yml --strict (ja) -> site/ja/"
  "$ZENSICAL" build -f mkdocs.ja.yml --strict
fi

echo ""
case "$LOCALE" in
  all)
    echo "$APP: done -> site/ (en) + site/ja/ (ja)"
    echo "$APP: serve both, so the language selector resolves:"
    echo "$APP:   python3 -m http.server 8000 -d site"
    ;;
  en)
    echo "$APP: done -> site/ (en)"
    echo "$APP: WARNING: this build cleared site/, so site/ja no longer exists." >&2
    echo "$APP:          run '$APP locale=ja' before serving or publishing." >&2
    ;;
  ja)
    echo "$APP: done -> site/ja/ (ja)"
    echo "$APP: site/ still holds whatever the last English build left there."
    ;;
esac
