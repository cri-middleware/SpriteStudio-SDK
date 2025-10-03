#!/bin/bash -ex
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
BASE_DIR=$(cd "$SCRIPT_DIR/.." && pwd)

pushd $BASE_DIR > /dev/null
flatc --rust --gen-compare --gen-object-api --gen-mutable ./fbs/type_base.fbs

for f in ./fbs/type_base.fbs ./fbs/Data/*.fbs;
do
    name=$(basename "$f" .fbs)
    flatc --rust --gen-compare --gen-object-api --gen-mutable $f
    mv "${name}_generated.rs" "./src/$name.rs"
done

/bin/rm -f temp_file
for f in ./src/common_ssfb2.rs ./src/ss*.rs;
do
awk '
/^[ \t]*use crate::type_base_generated::/ {
  count++;
  if (count == 1) {
    $0 = "use crate::type_base::*;"
  } else if (count == 2) {
    $0 = "  use crate::type_base::ss::*;"
  } else if (count == 3) {
    $0 = "  use crate::type_base::ss::basetype::*;"
  }
}
{
  print
}' $f > temp_file && mv temp_file $f
done

for f in ./src/ss*.rs;
do
awk '
/^[ \t]*use crate::common_ssfb2_generated/ {
  count++;
  if (count == 1) {
    $0 = "use crate::common_ssfb2::*;"
  } else if (count == 2) {
    $0 = "  use crate::common_ssfb2::ss::*;"
  } else if (count == 3) {
    $0 = "  use crate::common_ssfb2::ss::ssfb_2::*;"
  }
}
{
  print
}' $f > temp_file && mv temp_file $f
done


popd > /dev/null
