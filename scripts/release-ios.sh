#!/bin/bash

# # prepare
# $ rustup target add aarch64-apple-ios
# $ rustup target add x86_64-apple-ios

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

pushd ${ROOTDIR} > /dev/null
cargo build --target aarch64-apple-ios -p ssruntime --release --features custom-panic-handler
cargo build --target x86_64-apple-ios -p ssruntime --release --features custom-panic-handler

OUTPUT=target/universal-apple-ios
/bin/mkdir -p ${OUTPUT}

LIBS=(libssruntime.a libssruntime.dylib)
for lib in ${LIBS[@]}; do
    lipo -create target/aarch64-apple-ios/release/${lib} target/x86_64-apple-ios/release/${lib} -output ${OUTPUT}/${lib}
done

popd > /dev/null
