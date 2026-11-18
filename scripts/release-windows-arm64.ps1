#!/usr/bin/env pwsh

# # prepare
# ## arm host
# $ rustup target add aarch64-pc-windows-msvc

$ErrorActionPreference = "Stop"

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

# $ErrorActionPreference does not reach a native command's exit code, and an
# exit code would not stop a caller either (release.yml's pwsh step): throw.
function Invoke-Checked {
    param([string] $What, [scriptblock] $Body)
    & $Body
    if ($LASTEXITCODE -ne 0) { throw "$What failed ($LASTEXITCODE)" }
}

pushd $rootDirectory

try {
    Invoke-Checked "aarch64 workspace build" { cargo build --target aarch64-pc-windows-msvc --release }
    Invoke-Checked "aarch64 ssruntime-ffi build" { cargo build --target aarch64-pc-windows-msvc --release -p ssruntime-ffi --features libc_alloc,panic-handler }
}
finally {
    popd
}
