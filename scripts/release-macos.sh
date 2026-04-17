#!/bin/bash

# # prepare
# $ rustup target add aarch64-apple-darwin
# $ rustup target add x86_64-apple-darwin

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

pushd ${ROOTDIR} > /dev/null
cargo build -p ssconverter-cli -p ssconverter -p ssruntime --target aarch64-apple-darwin --release
cargo build -p ssconverter-cli -p ssconverter -p ssruntime --target x86_64-apple-darwin --release

OUTPUT=target/universal-apple-darwin
/bin/mkdir -p ${OUTPUT}

LIBS=(libssruntime.a libssruntime.dylib libssconverter.a libssconverter.dylib ssconverter-cli)
for lib in ${LIBS[@]}; do
    lipo -create target/aarch64-apple-darwin/release/${lib} target/x86_64-apple-darwin/release/${lib} -output ${OUTPUT}/${lib}
done

popd > /dev/null
