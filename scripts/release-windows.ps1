#!/usr/bin/env pwsh

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

pushd $rootDirectory
cargo build --target aarch64-pc-windows-msvc --release
cargo build --target x86_64-pc-windows-msvc --release

popd