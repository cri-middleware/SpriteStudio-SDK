#!/bin/bash

# # prepare
# $ rustup target add aarch64-apple-darwin
# $ rustup target add x86_64-apple-darwin

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
cargo build --target aarch64-apple-darwin ${CARGO_FLAGS}
cargo build --target aarch64-apple-darwin ${CARGO_FLAGS} -p ssruntime --features libc_alloc,panic-handler
cargo build --target x86_64-apple-darwin ${CARGO_FLAGS}
cargo build --target x86_64-apple-darwin ${CARGO_FLAGS} -p ssruntime --features libc_alloc,panic-handler

OUTPUT=target/universal-apple-darwin/${BUILD_MODE}
/bin/mkdir -p ${OUTPUT}

LIBS=(libssruntime.a libssruntime.dylib libssconverter.a libssconverter.dylib ssconverter-cli)
for lib in ${LIBS[@]}; do
    lipo -create target/aarch64-apple-darwin/${BUILD_MODE}/${lib} target/x86_64-apple-darwin/${BUILD_MODE}/${lib} -output ${OUTPUT}/${lib}
done

popd > /dev/null
