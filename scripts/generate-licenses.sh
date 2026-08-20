#!/bin/bash

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd ${BASEDIR} && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd ${ROOTDIR} && pwd -P)

pushd ${ROOTDIR} > /dev/null

# Output directory for the generated third-party notices. These are throwaway
# build artifacts bundled into release packages, NOT committed to the repo
# (default lives under the gitignored target/). Override by passing a path.
OUT_DIR="${1:-${ROOTDIR}/target/licenses}"

# Check if cargo-about is installed
if ! command -v cargo-about &> /dev/null; then
    echo "cargo-about is not installed. Installing it now..."
    cargo install cargo-about --locked --features cli
fi

TEMPLATE="about-markdown.hbs"

# List of crates to generate licenses for, as "path|cargo-about-flags" pairs.
# The flags pin each crate to the feature set it actually ships with, so the
# notices match the distributed binaries rather than the default-feature build.
#
# ssruntime: union of the shipped features (mobile/desktop FFI, wasm, web).
# Deliberately omit `std` — it pulls in ssconverter (tests only) and would
# pollute the runtime's minimal notices. So do NOT use --all-features here.
#
# ssconverter: native + web. No self-coupling, so --all-features is fine.
#
# Every entry MUST contain a "|" even when there are no flags.
CRATES=(
    "cli|"
    "libs/ssconverter|--all-features"
    "libs/sspj|"
    "libs/ssab|"
    "libs/ssruntime|--no-default-features --features ffi,libc_alloc,wasm_alloc,emscripten_alloc,web,panic-handler"
)

/bin/mkdir -p "${OUT_DIR}"

# 1. Sync the root license into each crate. cargo-about (and `cargo publish`)
# read the license text from a file inside the crate; without it, cargo-about
# falls back to the generic SPDX template and drops the real copyright holder.
# These per-crate LICENSE.md copies ARE committed (publish requirement).
for entry in "${CRATES[@]}"; do
    path="${entry%%|*}"
    /bin/cp "LICENSE.md" "${path}/LICENSE.md"
done

# 2. Generate the third-party notices into OUT_DIR as
# THIRD-PARTY-LICENSES.<crate>.md. Output goes outside the crate dirs so that
# cargo-about never mistakes a previously generated file for a license file.
for entry in "${CRATES[@]}"; do
    path="${entry%%|*}"
    flags="${entry#*|}"
    name=$(basename "${path}")

    echo "Generating third-party licenses for ${path}..."
    # ${flags} is intentionally unquoted so it splits into separate arguments
    # (empty when the entry has no flags).
    cargo about generate ${flags} \
        --manifest-path "${path}/Cargo.toml" \
        --config about.toml \
        "${TEMPLATE}" \
        -o "${OUT_DIR}/THIRD-PARTY-LICENSES.${name}.md"
done

echo "License generation complete. Notices written to ${OUT_DIR}"

popd > /dev/null
