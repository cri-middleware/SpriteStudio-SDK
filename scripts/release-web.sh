#!/bin/bash

# # prepare
# $ cargo install wasm-pack

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

BUILD_MODE="release"
WASM_PACK_FLAGS=""
if [ "$1" = "debug" ]; then
    BUILD_MODE="debug"
    WASM_PACK_FLAGS="--dev"
fi

WASM_PACK="wasm-pack"
if ! command -v $WASM_PACK &> /dev/null
then
    if [ -f "$HOME/.cargo/bin/wasm-pack" ]; then
        WASM_PACK="$HOME/.cargo/bin/wasm-pack"
    else
        echo "wasm-pack could not be found. Please install it with 'cargo install wasm-pack'"
        exit 1
    fi
fi

pushd ${ROOTDIR} > /dev/null

# This script stays build-only: it must not depend on cargo-about. wasm-pack
# bundles each crate's committed LICENSE.md automatically; the third-party
# THIRD-PARTY-LICENSES.md is added downstream by the release pipeline (the
# package job runs scripts/generate-licenses.sh and copies it into each
# package). Run that script manually first if you want the notice locally.

# The browser pays for every byte it downloads, so both runtime builds override the
# workspace `[profile.release]` `opt-level = 3` with `z`. That is rustc-level size
# optimization, on top of the wasm-opt `-Oz` that `[package.metadata.wasm-pack]`
# already applies; together they cut the shipped `ssruntime_bg.wasm` by about a
# third. It is passed as `--config` rather than `RUSTFLAGS` because the no-SIMD
# build below replaces RUSTFLAGS wholesale, which would drop it. ssconverter keeps
# `opt-level = 3`: it is a one-shot conversion tool, not a per-page download.
SIZE_OPT='--config profile.release.opt-level="z"'

# ssruntime (playback Brain) -> pkg/runtime-web. Inherits the repo-wide
# `+simd128` (.cargo/config.toml); this is the primary build for modern browsers.
OUT_DIR="../../pkg/runtime-web/${BUILD_MODE}"
/bin/rm -rf "pkg/runtime-web/${BUILD_MODE}"
$WASM_PACK build libs/ssruntime-ffi ${WASM_PACK_FLAGS} --target web --out-dir ${OUT_DIR} -- --features web --no-default-features ${SIZE_OPT}

# ssruntime no-SIMD variant -> pkg/runtime-web-nosimd. For hosts whose WASM
# engine lacks simd128 — notably the NW.js bundled with RPG Maker MZ, whose
# Chromium predates stable WASM SIMD and fails to load the simd128 build. Same
# source + same wasm-bindgen as above, so the generated JS glue is interface-
# identical: a consumer can swap only the `ssruntime_bg.wasm` binary. Override
# the repo-wide `+simd128` with `-simd128` for this build.
NOSIMD_OUT_DIR="../../pkg/runtime-web-nosimd/${BUILD_MODE}"
/bin/rm -rf "pkg/runtime-web-nosimd/${BUILD_MODE}"
RUSTFLAGS="-C target-feature=-simd128" $WASM_PACK build libs/ssruntime-ffi ${WASM_PACK_FLAGS} --target web --out-dir ${NOSIMD_OUT_DIR} -- --features web --no-default-features ${SIZE_OPT}

# ssconverter (sspj -> .ssab) -> pkg/converter-web. Sequential conversion;
# JS supplies project files in-memory (no rayon, no filesystem on wasm).
# The converter does not need SIMD, so override the repo-wide `+simd128`
# (.cargo/config.toml, intended for the runtime) for this build.
CONVERTER_OUT_DIR="../../pkg/converter-web/${BUILD_MODE}"
/bin/rm -rf "pkg/converter-web/${BUILD_MODE}"
RUSTFLAGS="-C target-feature=-simd128" $WASM_PACK build libs/ssconverter-ffi ${WASM_PACK_FLAGS} --target web --out-dir ${CONVERTER_OUT_DIR} -- --features web --no-default-features

popd > /dev/null
