#!/bin/bash
set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

pushd ${ROOTDIR} > /dev/null
cargo build --target aarch64-apple-darwin --release
cargo build --target x86_64-apple-darwin --release

OUTPUT=target/universal-apple-darwin
/bin/mkdir -p ${OUTPUT}

LIBS=(libssruntime.a libssruntime.dylib libssconverter3.a libssconverter3.dylib ssconverter3-cli)
for lib in ${LIBS[@]}; do
    lipo -create target/aarch64-apple-darwin/release/${lib} target/x86_64-apple-darwin/release/${lib} -output ${OUTPUT}/${lib}
done

popd $ROOTDIR > /dev/null
