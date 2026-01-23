#!/usr/bin/env pwsh

# # prepare
# ## x86 host
# $ rustup target add x86_64-pc-windows-msvc
# $ rustup target add i686-pc-windows-msvc
#
# ## arm host
# $ rustup target add aarch64-pc-windows-msvc

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

pushd $rootDirectory
cargo build --release

popd