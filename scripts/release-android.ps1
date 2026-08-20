#!/usr/bin/env pwsh

# # prepare
# $ rustup target add aarch64-linux-android
# $ rustup target add armv7-linux-androideabi
# $ rustup target add x86_64-linux-android

$ErrorActionPreference = "Stop"

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

$buildMode = "release"
$cargoFlags = @("--release")
if ($args.Count -gt 0 -and $args[0] -eq "debug") {
    $buildMode = "debug"
    $cargoFlags = @()
}

pushd $rootDirectory

try {
    & cargo ndk --target aarch64-linux-android build -p ssruntime-ffi --features libc_alloc,panic-handler @cargoFlags
    & cargo ndk --target armv7-linux-androideabi build -p ssruntime-ffi --features libc_alloc,panic-handler @cargoFlags
    & cargo ndk --target x86_64-linux-android build -p ssruntime-ffi --features libc_alloc,panic-handler @cargoFlags
}
finally {
    popd
}
