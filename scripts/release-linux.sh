#!/bin/bash

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

pushd ${ROOTDIR} > /dev/null
cargo build -p ssconverter-cli -p ssconverter --release
cargo build -p ssruntime --features panic-handler --release

popd > /dev/null
