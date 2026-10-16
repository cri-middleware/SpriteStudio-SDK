#!/bin/bash

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

CARGO_FLAGS="--release"
if [ "$1" = "debug" ]; then
    CARGO_FLAGS=""
fi

pushd ${ROOTDIR} > /dev/null
cargo build ${CARGO_FLAGS}
cargo build ${CARGO_FLAGS} -p ssruntime-ffi --features libc_alloc,panic-handler

popd > /dev/null
