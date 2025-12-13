#!/bin/bash -ex
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
BASE_DIR=$(cd "$SCRIPT_DIR/.." && pwd)

pushd $BASE_DIR > /dev/null

/bin/mkdir -p src/runtime

FLATC=flatc

${FLATC} --rust fbs/initialize.fbs
/bin/mv initialize_generated.rs src/runtime/initialize.rs

${FLATC} --rust fbs/preparation.fbs
/bin/mv preparation_generated.rs src/runtime/preparation.rs

${FLATC} --rust fbs/framedata.fbs
/bin/mv framedata_generated.rs src/runtime/framedata.rs

popd > /dev/null
