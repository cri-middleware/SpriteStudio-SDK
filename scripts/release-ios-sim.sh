#!/bin/bash

# # prepare
# $ rustup target add aarch64-apple-ios-sim
# $ rustup target add x86_64-apple-ios

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

pushd ${ROOTDIR} > /dev/null
cargo build --target aarch64-apple-ios-sim -p ssruntime --no-default-features --features ffi,libc_alloc,panic-handler --release
cargo build --target x86_64-apple-ios -p ssruntime --no-default-features --features ffi,libc_alloc,panic-handler --release

OUTPUT=target/universal-apple-ios-sim
/bin/mkdir -p ${OUTPUT}

LIBS=(libssruntime.a libssruntime.dylib)
for lib in ${LIBS[@]}; do
    lipo -create target/aarch64-apple-ios-sim/release/${lib} target/x86_64-apple-ios/release/${lib} -output ${OUTPUT}/${lib}
done

popd > /dev/null
