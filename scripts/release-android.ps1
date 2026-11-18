#!/usr/bin/env pwsh

# # prepare
# $ rustup target add aarch64-linux-android
# $ rustup target add armv7-linux-androideabi
# $ rustup target add x86_64-linux-android

$ErrorActionPreference = "Stop"

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

# $ErrorActionPreference does not reach a native command's exit code, and an
# exit code would not stop a caller either: throw.
function Invoke-Checked {
    param([string] $What, [scriptblock] $Body)
    & $Body
    if ($LASTEXITCODE -ne 0) { throw "$What failed ($LASTEXITCODE)" }
}

$cargoFlags = @("--release")
if ($args.Count -gt 0 -and $args[0] -eq "debug") {
    $cargoFlags = @()
}

pushd $rootDirectory

try {
    Invoke-Checked "aarch64-linux-android build" { & cargo ndk --target aarch64-linux-android build -p ssruntime-ffi --features libc_alloc,panic-handler @cargoFlags }
    Invoke-Checked "armv7-linux-androideabi build" { & cargo ndk --target armv7-linux-androideabi build -p ssruntime-ffi --features libc_alloc,panic-handler @cargoFlags }
    Invoke-Checked "x86_64-linux-android build" { & cargo ndk --target x86_64-linux-android build -p ssruntime-ffi --features libc_alloc,panic-handler @cargoFlags }
}
finally {
    popd
}
