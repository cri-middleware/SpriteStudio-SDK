#!/bin/bash

# # prepare
# $ rustup target add wasm32-unknown-unknown

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

pushd ${ROOTDIR} > /dev/null
cargo build --target wasm32-unknown-unknown -p ssruntime --release --no-default-features --features "ffi,std"

popd > /dev/null
