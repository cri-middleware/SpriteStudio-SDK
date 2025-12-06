#!/bin/bash -ex
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
BASE_DIR=$(cd "$SCRIPT_DIR/.." && pwd)

pushd $BASE_DIR > /dev/null

FLATC=flatc

${FLATC} --rust fbs/initialize.fbs
/bin/mv initialize_generated.rs src/initialize.rs

${FLATC} --rust fbs/preparation.fbs
/bin/mv preparation_generated.rs src/preparation.rs

${FLATC} --rust fbs/framedata.fbs
/bin/mv framedata_generated.rs src/framedata.rs


cargo fmt --all
popd > /dev/null
