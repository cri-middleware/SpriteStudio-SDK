#!/bin/bash

# # prepare
# $ rustup target add aarch64-apple-darwin
# $ rustup target add x86_64-apple-darwin

set -e

BASEDIR=$(dirname $0)
BASEDIR=$(cd $BASEDIR && pwd -P)
ROOTDIR=${BASEDIR}/..
ROOTDIR=$(cd $ROOTDIR && pwd -P)

BUILD_MODE="release"
CARGO_FLAGS="--release"
if [ "$1" = "debug" ]; then
    BUILD_MODE="debug"
    CARGO_FLAGS=""
fi

pushd ${ROOTDIR} > /dev/null
cargo build --target aarch64-apple-darwin ${CARGO_FLAGS}
cargo build --target aarch64-apple-darwin ${CARGO_FLAGS} -p ssruntime --features libc_alloc,panic-handler
cargo build --target x86_64-apple-darwin ${CARGO_FLAGS}
cargo build --target x86_64-apple-darwin ${CARGO_FLAGS} -p ssruntime --features libc_alloc,panic-handler

OUTPUT=target/universal-apple-darwin/${BUILD_MODE}
/bin/mkdir -p ${OUTPUT}

LIBS=(libssruntime.a libssruntime.dylib libssconverter.a libssconverter.dylib ssconverter-cli)
for lib in ${LIBS[@]}; do
    lipo -create target/aarch64-apple-darwin/${BUILD_MODE}/${lib} target/x86_64-apple-darwin/${BUILD_MODE}/${lib} -output ${OUTPUT}/${lib}
done

# --- Code signing & notarization (opt-in via env) ---------------------------
# Signing runs only when APPLE_SIGNING_IDENTITY is set and its Developer ID
# Application certificate is importable from the keychain search list (CI sets
# up an ephemeral keychain; locally your login keychain is used). Notarization
# of the CLI additionally needs an App Store Connect API key:
#   APPLE_API_KEY_PATH — path to the .p8 private key file
#   APPLE_API_KEY      — the key ID
#   APPLE_API_ISSUER   — the issuer ID
# These map to the SS_APPLE_* GitHub secrets — see
# .github/workflows/release.yml for the CI wiring.
if [ -n "${APPLE_SIGNING_IDENTITY:-}" ]; then
    echo "==> Codesigning macOS artifacts as: ${APPLE_SIGNING_IDENTITY}"
    # Hardened runtime (--options runtime) + a secure timestamp are prerequisites
    # for notarization and for Gatekeeper acceptance of Developer ID binaries.
    SIGN_TARGETS=(
        "${OUTPUT}/libssruntime.dylib"
        "${OUTPUT}/libssconverter.dylib"
        "${OUTPUT}/ssconverter-cli"
    )
    for f in "${SIGN_TARGETS[@]}"; do
        codesign --force --sign "${APPLE_SIGNING_IDENTITY}" \
            --options runtime --timestamp "${f}"
        codesign --verify --strict --verbose=2 "${f}"
    done

    # Notarize the CLI. A bare executable cannot be stapled (stapling targets
    # .app/.dmg/.pkg only), so we notarize a zip of the signed binary and rely on
    # Gatekeeper's online ticket check on first run.
    if [ -n "${APPLE_API_KEY_PATH:-}" ] && [ -n "${APPLE_API_KEY:-}" ] && [ -n "${APPLE_API_ISSUER:-}" ]; then
        echo "==> Notarizing ssconverter-cli"
        NOTARIZE_ZIP="${OUTPUT}/ssconverter-cli.notarize.zip"
        /bin/rm -f "${NOTARIZE_ZIP}"
        ( cd "${OUTPUT}" && /usr/bin/zip -q ssconverter-cli.notarize.zip ssconverter-cli )
        xcrun notarytool submit "${NOTARIZE_ZIP}" \
            --key "${APPLE_API_KEY_PATH}" \
            --key-id "${APPLE_API_KEY}" \
            --issuer "${APPLE_API_ISSUER}" \
            --wait
        /bin/rm -f "${NOTARIZE_ZIP}"
    else
        echo "==> Skipping notarization (APPLE_API_KEY_PATH / APPLE_API_KEY / APPLE_API_ISSUER not all set)"
    fi
else
    echo "==> Skipping macOS codesign (APPLE_SIGNING_IDENTITY not set)"
fi

popd > /dev/null
