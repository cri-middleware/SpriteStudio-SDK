#!/usr/bin/env pwsh

# # prepare
# ## x86 host
# $ rustup target add x86_64-pc-windows-msvc
# $ rustup target add i686-pc-windows-msvc
#

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

pushd $rootDirectory
cargo build -p ssconverter-cli -p ssconverter --target x86_64-pc-windows-msvc --release
cargo build -p ssruntime --features panic-handler --target x86_64-pc-windows-msvc --release
# cargo build -p ssconverter-cli -p ssconverter --target i686-pc-windows-msvc --release
# cargo build -p ssruntime --features panic-handler --target i686-pc-windows-msvc --release

popd
