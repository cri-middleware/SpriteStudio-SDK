#!/bin/bash -e
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
BASE_DIR=$(cd "$SCRIPT_DIR/.." && pwd)

pushd $BASE_DIR > /dev/null

FLATC=flatc
FEATURE="runtime"
/bin/mkdir -p src/${FEATURE}
for f in ./fbs/*.fbs;
do
  name=$(basename "$f" .fbs)
  ${FLATC} --rust $f
  mv "${name}_generated.rs" "./src/${FEATURE}/$name.rs"
done

popd > /dev/null
