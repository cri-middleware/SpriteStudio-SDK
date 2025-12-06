#!/bin/bash -ex
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
BASE_DIR=$(cd "$SCRIPT_DIR/.." && pwd)

pushd $BASE_DIR > /dev/null

# runtime
FLATC="flatc --rust"
FEATURE="runtime"
/bin/mkdir -p ./src/${FEATURE}
for f in ./fbs/*.fbs;
do
  name=$(basename "$f" .fbs)
  ${FLATC} $f
  mv "${name}_generated.rs" "./src/${FEATURE}/$name.rs"
done

# converter
FLATC="flatc --rust --gen-compare --gen-object-api --gen-mutable"
FEATURE="converter"
/bin/mkdir -p ./src/${FEATURE}
for f in ./fbs/*.fbs;
do
  name=$(basename "$f" .fbs)
  ${FLATC} $f
  mv "${name}_generated.rs" "./src/${FEATURE}/$name.rs"
done

cargo fmt --all
popd > /dev/null
