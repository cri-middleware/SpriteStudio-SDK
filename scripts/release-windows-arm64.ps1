#!/usr/bin/env pwsh

# # prepare
# ## arm host
# $ rustup target add aarch64-pc-windows-msvc

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

pushd $rootDirectory
cargo build --target aarch64-pc-windows-msvc --release
cargo build --target aarch64-pc-windows-msvc --release -p ssruntime --features libc_alloc,panic-handler

popd
