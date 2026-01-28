#!/bin/bash -ex
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
BASE_DIR=$(cd "$SCRIPT_DIR/.." && pwd)

pushd $BASE_DIR > /dev/null

# for runtime
FLATC_CMD=flatc
FLATC="${FLATC_CMD} --rust"
FEATURE="runtime"
/bin/mkdir -p ./src/${FEATURE}
for f in ./fbs/*.fbs;
do
  name=$(basename "$f" .fbs)
  ${FLATC} $f
  mv "${name}_generated.rs" "./src/${FEATURE}/$name.rs"
done

# for converter
FLATC="${FLATC_CMD} --rust --gen-compare --gen-object-api --gen-mutable"
FEATURE="converter"
/bin/mkdir -p ./src/${FEATURE}
for f in ./fbs/*.fbs;
do
  name=$(basename "$f" .fbs)
  ${FLATC} $f
  mv "${name}_generated.rs" "./src/${FEATURE}/$name.rs"
  if [ $(uname) == "Darwin" ]; then
    # bsd sed
    sed -i '' 's/pub mod ss {/pub mod ss {\nextern crate alloc;/g' "./src/${FEATURE}/$name.rs"
    sed -i '' 's/pub mod format {/pub mod format {\nextern crate alloc;/g' "./src/${FEATURE}/$name.rs"
  else
    # gnu sed
    sed -i 's/pub mod ss {/pub mod ss {\nextern crate alloc;/g' "./src/${FEATURE}/$name.rs"
    sed -i 's/pub mod format {/pub mod format {\nextern crate alloc;/g' "./src/${FEATURE}/$name.rs"
  fi
done

popd > /dev/null
