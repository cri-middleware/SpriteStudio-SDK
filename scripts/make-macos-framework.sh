#!/bin/bash

# Wrap a libssruntime dylib into a versioned macOS .framework — the format
# CocoaPods `vendored_frameworks` consumes (Flutter / Ren'Py). Unlike the iOS flat
# framework this is a macOS *versioned* bundle (Versions/A + Current symlinks). The
# binary is the SAME Mach-O as the dylib; we only wrap it and re-point its
# install_name to @rpath so an embedded framework resolves at runtime.
#
# Single source of truth for framework packaging: called by this repo's
# release-macos.sh (release/universal build, later Developer-ID re-signed in that
# script's signing block) and by SSPlayerForFlutter's build-runtime.sh on its fast
# host/debug path (plain `cargo build`, which does not run release-macos.sh) — the
# mirror of make-macos-bundle.sh for Unity.
#
# Usage: make-macos-framework.sh <src-dylib> <dest-framework> [version] [bundle-id] [min-os] [header]
#   header : optional C-API header; when given, a Headers/ + Modules/ module map is
#            added so Swift / CocoaPods can `import <name>`.

set -e

SRC=$1
FRAMEWORK=$2
VER=${3:-0.0.0}
VER=${VER%%-*}                                   # strip any -prerelease suffix
BUNDLE_ID=${4:-jp.co.cri-mw.optpix.ssruntime}
MIN_OS=${5:-11.0}
HEADER=${6:-}

if [ -z "$SRC" ] || [ -z "$FRAMEWORK" ]; then
    echo "Usage: $0 <src-dylib> <dest-framework> [version] [bundle-id] [min-os] [header]" >&2
    exit 1
fi
if [ ! -f "$SRC" ]; then
    echo "ERROR: source dylib not found: $SRC" >&2
    exit 1
fi

NAME=$(basename "$FRAMEWORK" .framework)         # e.g. libssruntime

/bin/rm -rf "${FRAMEWORK}"
/bin/mkdir -p "${FRAMEWORK}/Versions/A/Resources"

# Binary: copy the dylib in and re-point its install_name to the rpath-relative
# path dyld expects for an embedded framework. Operating on the copy leaves the
# source dylib's LC_ID_DYLIB untouched.
cp "${SRC}" "${FRAMEWORK}/Versions/A/${NAME}"
install_name_tool -id "@rpath/${NAME}.framework/Versions/A/${NAME}" \
    "${FRAMEWORK}/Versions/A/${NAME}"

# Optional C-API header + a clang module map so Swift / CocoaPods can `import`.
if [ -n "${HEADER}" ] && [ -f "${HEADER}" ]; then
    /bin/mkdir -p "${FRAMEWORK}/Versions/A/Headers" "${FRAMEWORK}/Versions/A/Modules"
    cp "${HEADER}" "${FRAMEWORK}/Versions/A/Headers/$(basename "${HEADER}")"
    cat > "${FRAMEWORK}/Versions/A/Modules/module.modulemap" <<EOF
framework module ${NAME} {
    umbrella header "$(basename "${HEADER}")"
    export *
    module * { export * }
}
EOF
fi

cat > "${FRAMEWORK}/Versions/A/Resources/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key><string>en</string>
	<key>CFBundleExecutable</key><string>${NAME}</string>
	<key>CFBundleIdentifier</key><string>${BUNDLE_ID}</string>
	<key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
	<key>CFBundleName</key><string>${NAME}</string>
	<key>CFBundlePackageType</key><string>FMWK</string>
	<key>CFBundleShortVersionString</key><string>${VER}</string>
	<key>CFBundleVersion</key><string>${VER}</string>
	<key>CFBundleSupportedPlatforms</key><array><string>MacOSX</string></array>
	<key>LSMinimumSystemVersion</key><string>${MIN_OS}</string>
</dict>
</plist>
EOF
# Catch a malformed plist here rather than at App Store / notarization time.
plutil -lint "${FRAMEWORK}/Versions/A/Resources/Info.plist" > /dev/null

# Versioned-bundle symlinks (macOS convention). Targets are relative to the link's
# own directory: Current -> A at Versions/, and the bundle-root aliases -> Current/*.
ln -sfn A "${FRAMEWORK}/Versions/Current"
ln -sfn "Versions/Current/${NAME}" "${FRAMEWORK}/${NAME}"
[ -d "${FRAMEWORK}/Versions/A/Headers" ] && ln -sfn Versions/Current/Headers "${FRAMEWORK}/Headers"
[ -d "${FRAMEWORK}/Versions/A/Modules" ] && ln -sfn Versions/Current/Modules "${FRAMEWORK}/Modules"
ln -sfn Versions/Current/Resources "${FRAMEWORK}/Resources"

# Sign the framework so it loads on Apple Silicon (arm64 refuses to dlopen unsigned
# code). A versioned macOS framework is signed at its version directory.
# When APPLE_SIGNING_IDENTITY is set (same env var release-macos.sh uses), sign with
# that Developer ID + hardened runtime + secure timestamp so the framework is
# notarization-ready; a failure here is surfaced (an explicit identity should not be
# silently downgraded). Otherwise ad-hoc sign so unsigned CI/local builds still load
# — guarded so a non-macOS host does not hard-fail.
if [ -n "${APPLE_SIGNING_IDENTITY:-}" ]; then
    codesign --force --sign "${APPLE_SIGNING_IDENTITY}" \
        --options runtime --timestamp "${FRAMEWORK}/Versions/A"
    codesign --verify --strict --verbose=2 "${FRAMEWORK}"
else
    codesign --force --sign - "${FRAMEWORK}/Versions/A" > /dev/null 2>&1 || true
fi
