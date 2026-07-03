#!/usr/bin/env pwsh

# # prepare
# $ cargo install wasm-pack

$ErrorActionPreference = "Stop"

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

$buildMode = "release"
$wasmPackFlags = @()
if ($args.Count -gt 0 -and $args[0] -eq "debug") {
    $buildMode = "debug"
    $wasmPackFlags = @("--dev")
}

$wasmPack = "wasm-pack"
if (!(Get-Command $wasmPack -ErrorAction SilentlyContinue)) {
    if (Test-Path "$env:USERPROFILE\.cargo\bin\wasm-pack.exe") {
        $wasmPack = "$env:USERPROFILE\.cargo\bin\wasm-pack.exe"
    } elseif (Test-Path "$env:HOME\.cargo\bin\wasm-pack") {
        $wasmPack = "$env:HOME\.cargo\bin\wasm-pack"
    } else {
        Write-Error "wasm-pack could not be found. Please install it with 'cargo install wasm-pack'"
        exit 1
    }
}

pushd $rootDirectory

try {
    # This script stays build-only: it must not depend on cargo-about. wasm-pack
    # bundles each crate's committed LICENSE.md automatically; the third-party
    # THIRD-PARTY-LICENSES.md is added downstream by the release pipeline (the
    # package job runs scripts/generate-licenses.ps1 and copies it into each
    # package). Run that script manually first if you want the notice locally.

    # ssruntime (playback Brain) -> pkg/runtime-web
    $outDir = "../../pkg/runtime-web/$buildMode"
    if (Test-Path "pkg/runtime-web/$buildMode") {
        Remove-Item -Recurse -Force "pkg/runtime-web/$buildMode"
    }
    
    & $wasmPack build libs/ssruntime @wasmPackFlags --target web --out-dir $outDir -- --features web --no-default-features

    # ssconverter (sspj -> .ssab) -> pkg/converter-web. Sequential conversion;
    # JS supplies project files in-memory (no rayon, no filesystem on wasm).
    # The converter does not need SIMD, so override the repo-wide `+simd128`
    # (.cargo/config.toml, intended for the runtime) for this build.
    $converterOutDir = "../../pkg/converter-web/$buildMode"
    if (Test-Path "pkg/converter-web/$buildMode") {
        Remove-Item -Recurse -Force "pkg/converter-web/$buildMode"
    }
    
    $oldRustFlags = $env:RUSTFLAGS
    try {
        $env:RUSTFLAGS = "-C target-feature=-simd128"
        & $wasmPack build libs/ssconverter @wasmPackFlags --target web --out-dir $converterOutDir -- --features web --no-default-features
    } finally {
        $env:RUSTFLAGS = $oldRustFlags
    }
}
finally {
    popd
}
