#!/bin/bash

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd ${BASEDIR} && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd ${ROOTDIR} && pwd -P)

pushd ${ROOTDIR} > /dev/null

# Check if cargo-about is installed
if ! command -v cargo-about &> /dev/null; then
    echo "cargo-about is not installed. Installing it now..."
    cargo install cargo-about --locked --features cli
fi

TEMPLATE="about-markdown.hbs"

# List of crates to generate licenses for
CRATES=(
    "cli"
    "libs/ssconverter"
    "libs/sspj"
    "libs/ssab"
    "libs/ssruntime"
    "libs/ssinterpolate"
)

# Create a temporary directory for outputs
TEMP_DIR=$(mktemp -d)

# Ensure temp directory is cleaned up on exit
trap '/bin/rm -rf "${TEMP_DIR}"' EXIT

# 1. Clean existing generated files in workspace to prevent cargo-about recursive matching
for path in "${CRATES[@]}"; do
    /bin/rm -f "${path}/DEPENDENCIES.md"
done

# 2. Run cargo about to generate outputs in the temp directory
for path in "${CRATES[@]}"; do
    # Replace slashes in path for temp filename
    temp_name=$(echo "${path}" | tr '/' '_')
    output_temp="${TEMP_DIR}/${temp_name}.md"
    
    echo "Generating third-party licenses for ${path}..."
    cargo about generate \
        --manifest-path "${path}/Cargo.toml" \
        --config about.toml \
        "${TEMPLATE}" \
        -o "${output_temp}"
done

# 3. Move generated files to their final locations in each crate directory
for path in "${CRATES[@]}"; do
    temp_name=$(echo "${path}" | tr '/' '_')
    /bin/mv "${TEMP_DIR}/${temp_name}.md" "${path}/DEPENDENCIES.md"
done

echo "License generation complete."

popd > /dev/null
