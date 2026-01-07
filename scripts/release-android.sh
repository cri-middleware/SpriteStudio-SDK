#!/bin/bash
set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

pushd ${ROOTDIR} > /dev/null
cargo ndk --target aarch64-linux-android build -p ssruntime --release
cargo ndk --target armv7-linux-androideabi build -p ssruntime --release
cargo ndk --target i686-linux-android build -p ssruntime --release
cargo ndk --target x86_64-linux-android build -p ssruntime --release
popd $ROOTDIR > /dev/null
