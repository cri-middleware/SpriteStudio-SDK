#!/usr/bin/env pwsh

# # prepare
# ## arm host
# $ rustup target add aarch64-pc-windows-msvc

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

pushd $rootDirectory
cargo build -p ssconverter-cli -p ssconverter --target aarch64-pc-windows-msvc --release
cargo build -p ssruntime --features panic-handler --target aarch64-pc-windows-msvc --release

popd
