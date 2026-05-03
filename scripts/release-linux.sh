#!/bin/bash

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

pushd ${ROOTDIR} > /dev/null
cargo build --release
cargo build --release -p ssruntime --features libc_alloc,panic-handler

popd > /dev/null
