#!/usr/bin/env pwsh

# # prepare
# ## x86 host
# $ rustup target add x86_64-pc-windows-msvc
# $ rustup target add i686-pc-windows-msvc
#

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
    Invoke-Checked "x86_64 workspace build" { cargo build --target x86_64-pc-windows-msvc --release }
    Invoke-Checked "x86_64 ssruntime-ffi build" { cargo build --target x86_64-pc-windows-msvc --release -p ssruntime-ffi --features libc_alloc,panic-handler }
    # cargo build --target i686-pc-windows-msvc --release
    # cargo build --target i686-pc-windows-msvc --release -p ssruntime-ffi --features libc_alloc,panic-handler
}
finally {
    popd
}
