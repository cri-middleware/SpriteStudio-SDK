#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

pushd $rootDirectory

$FLATC_CMD = "flatc"

# for runtime
$FEATURE = "runtime"
mkdir -Force "./src/$FEATURE" > $null

foreach ($f in Get-ChildItem ./fbs/*.fbs) {
    $name = $f.BaseName
    & $FLATC_CMD --rust $f
    mv "${name}_generated.rs" "./src/$FEATURE/$name.rs" -Force
}

# for converter
$FEATURE = "converter"
mkdir -Force "./src/$FEATURE" > $null

foreach ($f in Get-ChildItem ./fbs/*.fbs) {
    $name = $f.BaseName
    & $FLATC_CMD --rust --gen-compare --gen-object-api --gen-mutable $f
    mv "${name}_generated.rs" "./src/$FEATURE/$name.rs" -Force
}

popd
