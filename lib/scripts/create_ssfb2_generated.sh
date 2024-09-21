#!/bin/bash -ex

SCRIPTDIR=$(dirname $0)
SCRIPTDIR=$(cd $SCRIPTDIR && pwd -P)
BASEDIR=${SCRIPTDIR}/..
BASEDIR=$(cd ${BASEDIR} && pwd -P)

# Build Converter
pushd ${BASEDIR}

arr=("common_ssfb2" "ssfb2" "ssc2" "ssa2" "sse2" "ssq2" "ssf2" "sss2")
for i in "${arr[@]}"
do
    flatc --cpp-std 'c++17' -c ${BASEDIR}/../utils/fbs/${i}.fbs --gen-compare --gen-object-api --gen-mutable --cpp-ptr-type std::shared_ptr
    /bin/mv ${i}_generated.h include/SSConverter3Lib/ssfb2/${i}_generated.h
done

popd > /dev/null # ${BASEDIR}
