#!/bin/bash -ex

SCRIPTDIR=$(dirname $0)
SCRIPTDIR=$(cd $SCRIPTDIR && pwd -P)
BASEDIR=${SCRIPTDIR}/..
BASEDIR=$(cd ${BASEDIR} && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd ${ROOTDIR} && pwd -P)

CPP_STD='C++17'
SSFB2_HEADER_DIR="${BASEDIR}/include/SSConverter3Lib/ssfb2"
SSFB2_FBS_DIR="${ROOTDIR}/utils/fbs"

# Build Converter
pushd ${BASEDIR}

# type_base.fbs
flatc --cpp-std ${CPP_STD} -c ${SSFB2_FBS_DIR}/type_base.fbs --gen-compare --gen-object-api --gen-mutable --cpp-ptr-type std::shared_ptr
/bin/mv type_base_generated.h ${SSFB2_HEADER_DIR}/

# Data directory
/bin/mkdir -p ${SSFB2_HEADER_DIR}/Data/
DATA_FBS_DIR=${SSFB2_FBS_DIR}/Data
for i in $(/bin/ls ${DATA_FBS_DIR})
do
    stem=$(basename -s .fbs  ${DATA_FBS_DIR}/${i})
    flatc --cpp-std ${CPP_STD} -c ${DATA_FBS_DIR}/${stem}.fbs --gen-compare --gen-object-api --gen-mutable --cpp-ptr-type std::shared_ptr
    /bin/mv ${stem}_generated.h ${SSFB2_HEADER_DIR}/Data/
done

# # Runtime directory
# /bin/mkdir -p ${SSFB2_HEADER_DIR}/Runtime
# RUNTIME_FBS_DIR=${SSFB2_FBS_DIR}/Runtime
# for i in $(/bin/ls ${RUNTIME_FBS_DIR})
# do
#     stem=$(basename -s .fbs  ${RUNTIME_FBS_DIR}/${i})
#     flatc --cpp-std ${CPP_STD} -c ${RUNTIME_FBS_DIR}/${stem}.fbs --gen-compare --gen-object-api --gen-mutable --cpp-ptr-type std::shared_ptr
#     /bin/mv ${stem}_generated.h ${SSFB2_HEADER_DIR}/Runtime/
# done

popd > /dev/null # ${BASEDIR}
