#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

pushd $rootDirectory

$FLATC_CMD = "flatc"

$FEATURE = "runtime"
mkdir -Force "./src/$FEATURE" > $null

foreach ($f in Get-ChildItem ./fbs/*.fbs) {
    $name = $f.BaseName
    & $FLATC_CMD --rust $f
    mv "${name}_generated.rs" "./src/$FEATURE/$name.rs" -Force
}

popd
