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

# Deployment target for the macOS builds. Exported so rustc stamps every darwin
# slice's LC_BUILD_VERSION minos to match the .framework Info.plist LSMinimumSystemVersion
# written below — a disagreement between the two trips App Store / notarization
# validation of an embedded framework. arm64 macOS is 11.0-only regardless, so 11.0
# is the effective floor; override via the env var if a different baseline is needed.
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}"

# Marketing/build version + bundle id for the .framework Info.plist (mirrors the iOS
# xcframework packaging). Strip any -prerelease suffix (e.g. 7.0.0-alpha.1 -> 7.0.0):
# App Store rejects a non-numeric CFBundleShortVersionString.
WORKSPACE_VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)
SHORT_VERSION="${WORKSPACE_VERSION%%-*}"
BUNDLE_ID="jp.co.cri-mw.spritestudio.ssruntime"

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

# --- macOS .framework packaging ---------------------------------------------
# Flutter on macOS consumes native code via CocoaPods `vendored_frameworks`, which
# needs a .framework bundle rather than a bare dylib. The framework binary is the
# SAME Mach-O as the universal libssruntime.dylib produced above — we only wrap it in
# a macOS *versioned* bundle (Versions/A + Current symlinks; distinct from the iOS
# flat framework layout). Ren'Py dlopens this framework by path (ctypes); Flutter /
# CocoaPods link it via vendored_frameworks. Unity consumes the separate
# ssruntime.bundle built below (macOS's native-plugin format). The bare
# libssruntime.dylib is now just an internal intermediate — the shared Mach-O the
# framework and bundle are copied from — and is no longer shipped.
#
# The packaging lives in scripts/make-macos-framework.sh (single source of truth)
# so SSPlayerForFlutter's build-runtime.sh reuses the exact same layout on its fast
# host/debug path — the mirror of make-macos-bundle.sh for Unity. The helper signs
# the framework itself: Developer ID (hardened runtime + timestamp) when
# APPLE_SIGNING_IDENTITY is set (exported before this script runs), else ad-hoc — so
# the signing block below does NOT re-sign it.
FRAMEWORK=${OUTPUT}/libssruntime.framework
# The framework's public header + module-map umbrella is the plain-C header
# (libs/ssruntime/target/ssruntime.h). A C umbrella keeps the clang module
# importable from Swift/ObjC, and its `extern "C"` guards keep it importable from
# ObjC++/C++ too.
"${BASEDIR}/make-macos-framework.sh" \
    "${OUTPUT}/libssruntime.dylib" \
    "${FRAMEWORK}" \
    "${SHORT_VERSION}" \
    "${BUNDLE_ID}" \
    "${MACOSX_DEPLOYMENT_TARGET}" \
    "${ROOTDIR}/libs/ssruntime/target/ssruntime.h"

# --- macOS .bundle packaging (Unity) ----------------------------------------
# Unity's documented macOS native-plugin format is a loadable *.bundle*. The
# packaging lives in scripts/make-macos-bundle.sh (single source of truth) so
# SSPlayerForUnity's build-runtime.sh reuses the exact same layout on its fast
# host/debug path. Same universal Mach-O as libssruntime.dylib above; the helper
# signs it itself (Developer ID when APPLE_SIGNING_IDENTITY is set, else ad-hoc) —
# so the signing block below does NOT re-sign it.
"${BASEDIR}/make-macos-bundle.sh" \
    "${OUTPUT}/libssruntime.dylib" \
    "${OUTPUT}/ssruntime.bundle" \
    "${SHORT_VERSION}" \
    "${BUNDLE_ID}" \
    "${MACOSX_DEPLOYMENT_TARGET}"

# --- Code signing & notarization (opt-in via env) ---------------------------
# Signing runs only when APPLE_SIGNING_IDENTITY is set and its Developer ID
# Application certificate is importable from the keychain search list (CI sets
# up an ephemeral keychain; locally your login keychain is used). Notarization
# of the CLI additionally needs an App Store Connect API key:
#   APPLE_API_KEY_PATH — path to the .p8 private key file
#   APPLE_API_KEY      — the key ID
#   APPLE_API_ISSUER   — the issuer ID
# These map to the APPLE_DEV_ID_APP_NAME / ASC_API_* GitHub secrets — see
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

    # libssruntime.framework and ssruntime.bundle are already Developer-ID signed
    # (hardened runtime + timestamp) by their packaging helpers above
    # (make-macos-framework.sh / make-macos-bundle.sh), which honour
    # APPLE_SIGNING_IDENTITY — so they are NOT re-signed here. Like the standalone
    # dylibs, neither is separately notarized; the consuming app re-signs and
    # notarizes the whole .app.

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
