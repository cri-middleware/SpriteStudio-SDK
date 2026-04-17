#!/bin/bash

# # prepare
# $ rustup target add aarch64-linux-android
# $ rustup target add armv7-linux-androideabi
# $ rustup target add x86_64-linux-android

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

pushd ${ROOTDIR} > /dev/null
cargo ndk --target aarch64-linux-android build -p ssruntime --no-default-features --features ffi,libc_alloc,panic-handler --release
cargo ndk --target armv7-linux-androideabi build -p ssruntime --no-default-features --features ffi,libc_alloc,panic-handler --release
cargo ndk --target x86_64-linux-android build -p ssruntime --no-default-features --features ffi,libc_alloc,panic-handler --release
popd > /dev/null
