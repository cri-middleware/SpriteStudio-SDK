#!/bin/bash

# # prepare
# $ cargo install wasm-pack

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

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

/bin/rm -rf "pkg/web"
/bin/cp "LICENSE.md" "libs/ssruntime/LICENSE.md"
$WASM_PACK build libs/ssruntime --target web --out-dir ../../pkg/web -- --features web --no-default-features
/bin/rm "libs/ssruntime/LICENSE.md"

popd > /dev/null
