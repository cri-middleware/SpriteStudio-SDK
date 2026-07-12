#!/bin/bash

# Wrap a libssruntime dylib into a macOS .bundle — Unity's documented native-plugin
# format for macOS (Unity does not officially support a .framework on macOS). Unity
# dlopens the bundle and resolves [DllImport("ssruntime")] to it. The bundle wraps
# the SAME Mach-O as the dylib; unlike the versioned .framework it has no version
# symlinks, so it survives git / npm pack / the release zip with a plain copy.
#
# Single source of truth for bundle packaging: called by this repo's
# release-macos.sh (release/universal build, later Developer-ID re-signed in that
# script's signing block) and by SSPlayerForUnity's build-runtime.sh on its fast
# host/debug path (plain `cargo build`, which does not run release-macos.sh).
#
# Usage: make-macos-bundle.sh <src-dylib> <dest-bundle> [version] [bundle-id] [min-os]

set -e

SRC=$1
BUNDLE=$2
VER=${3:-0.0.0}
VER=${VER%%-*}                                   # strip any -prerelease suffix
BUNDLE_ID=${4:-jp.co.cri-mw.optpix.ssruntime}
MIN_OS=${5:-11.0}

if [ -z "$SRC" ] || [ -z "$BUNDLE" ]; then
    echo "Usage: $0 <src-dylib> <dest-bundle> [version] [bundle-id] [min-os]" >&2
    exit 1
fi
if [ ! -f "$SRC" ]; then
    echo "ERROR: source dylib not found: $SRC" >&2
    exit 1
fi

/bin/rm -rf "${BUNDLE}"
/bin/mkdir -p "${BUNDLE}/Contents/MacOS"
# The binary's LC_ID_DYLIB is irrelevant: Unity loads the bundle by path.
cp "${SRC}" "${BUNDLE}/Contents/MacOS/libssruntime"
cat > "${BUNDLE}/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key><string>en</string>
	<key>CFBundleExecutable</key><string>libssruntime</string>
	<key>CFBundleIdentifier</key><string>${BUNDLE_ID}</string>
	<key>CFBundleInfoDictionaryVersion</key><string>6.0</string>
	<key>CFBundleName</key><string>libssruntime</string>
	<key>CFBundlePackageType</key><string>BNDL</string>
	<key>CFBundleShortVersionString</key><string>${VER}</string>
	<key>CFBundleVersion</key><string>${VER}</string>
	<key>CFBundleSupportedPlatforms</key><array><string>MacOSX</string></array>
	<key>LSMinimumSystemVersion</key><string>${MIN_OS}</string>
</dict>
</plist>
EOF
plutil -lint "${BUNDLE}/Contents/Info.plist" > /dev/null

# Ad-hoc sign so the bundle loads on Apple Silicon (arm64 refuses to dlopen
# unsigned code). release-macos.sh re-signs with Developer ID when a signing
# identity is available; the ad-hoc signature keeps unsigned CI/local builds
# loadable. Guarded so a non-macOS host does not hard-fail.
codesign --force --sign - "${BUNDLE}" > /dev/null 2>&1 || true
