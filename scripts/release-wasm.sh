#!/bin/bash

# # prepare
# $ rustup target add wasm32-unknown-unknown

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

pushd ${ROOTDIR} > /dev/null
# Build for Pure Wasm (e.g. WASI or custom embedded engines without JS).
# This uses dlmalloc for memory management and a basic panic handler, while keeping math deterministic via libm.
cargo build --target wasm32-unknown-unknown -p ssruntime --release --no-default-features --features "wasm_alloc,panic-handler"

popd > /dev/null
