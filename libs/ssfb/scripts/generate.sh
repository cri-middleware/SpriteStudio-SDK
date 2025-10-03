#!/bin/bash -ex
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
BASE_DIR=$(cd "$SCRIPT_DIR/.." && pwd)

pushd $BASE_DIR > /dev/null
flatc --rust --gen-compare --gen-object-api --gen-mutable ./fbs/type_base.fbs

for f in ./fbs/*.fbs;
do
    name=$(basename "$f" .fbs)
    flatc --rust --gen-compare --gen-object-api --gen-mutable $f
    mv "${name}_generated.rs" "./src/$name.rs"
done

popd > /dev/null
