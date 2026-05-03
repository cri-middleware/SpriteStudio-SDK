#!/bin/bash

# # prepare
# $ rustup target add wasm32-unknown-unknown

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

pushd ${ROOTDIR} > /dev/null
# Build a static library (.a / .rlib) for wasm32 consumers that statically link
# Rust into their own wasm build (Unity WebGL / Godot Web / Flutter etc. via Emscripten).
# JS-facing builds go through release-web.sh (wasm-pack). The cdylib crate-type is
# excluded here because no FFI/wasm-bindgen exports are enabled under these features,
# which would otherwise produce an empty .wasm artifact.
# Uses dlmalloc for memory management and a basic panic handler, while keeping math
# deterministic via libm.
cargo rustc --target wasm32-unknown-unknown -p ssruntime --release --no-default-features --features "wasm_alloc,panic-handler" --crate-type staticlib,rlib

popd > /dev/null
