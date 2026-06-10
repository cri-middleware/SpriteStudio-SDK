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

# ssruntime (playback Brain) -> pkg/runtime-web
OUT_DIR="../../pkg/runtime-web/${BUILD_MODE}"
/bin/rm -rf "pkg/runtime-web/${BUILD_MODE}"
$WASM_PACK build libs/ssruntime ${WASM_PACK_FLAGS} --target web --out-dir ${OUT_DIR} -- --features web --no-default-features

# ssconverter (sspj -> .ssab) -> pkg/converter-web. Sequential conversion;
# JS supplies project files in-memory (no rayon, no filesystem on wasm).
# The converter does not need SIMD, so override the repo-wide `+simd128`
# (.cargo/config.toml, intended for the runtime) for this build.
CONVERTER_OUT_DIR="../../pkg/converter-web/${BUILD_MODE}"
/bin/rm -rf "pkg/converter-web/${BUILD_MODE}"
RUSTFLAGS="-C target-feature=-simd128" $WASM_PACK build libs/ssconverter ${WASM_PACK_FLAGS} --target web --out-dir ${CONVERTER_OUT_DIR} -- --features web --no-default-features

popd > /dev/null
