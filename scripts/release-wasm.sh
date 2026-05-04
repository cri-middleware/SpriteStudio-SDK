#!/bin/bash
#!/bin/bash

# # prepare
# $ rustup toolchain install nightly
# $ rustup target add wasm32-unknown-unknown --toolchain nightly

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

BUILD_MODE="release"
CARGO_FLAGS="--release"
if [ "$1" = "debug" ]; then
    BUILD_MODE="debug"
    CARGO_FLAGS=""
fi

pushd ${ROOTDIR} > /dev/null

# Godot's GDExtension for Web loads modules dynamically, requiring Position Independent Code (PIC).
# The pre-compiled standard library for wasm32-unknown-unknown lacks PIC support.
# Therefore, we must use the nightly toolchain and -Z build-std to recompile core/alloc with PIC.
# Note: We specifically DO NOT include +atomics here to keep the build single-threaded (nothreads).
export RUSTFLAGS="-C relocation-model=pic"

cargo +nightly rustc -Z build-std=core,alloc \
    --target wasm32-unknown-unknown \
    -p ssruntime \
    ${CARGO_FLAGS} \
    --no-default-features \
    --features "wasm_alloc,panic-handler" \
    --crate-type staticlib,rlib

popd > /dev/null


