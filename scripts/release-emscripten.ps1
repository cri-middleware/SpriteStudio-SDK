#!/usr/bin/env pwsh

# # prepare
# $ rustup toolchain install nightly
# $ rustup target add wasm32-unknown-unknown --toolchain nightly

$ErrorActionPreference = "Stop"

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

# $ErrorActionPreference does not reach a native command's exit code, and an
# exit code would not stop a caller either: throw.
function Invoke-Checked {
    param([string] $What, [scriptblock] $Body)
    & $Body
    if ($LASTEXITCODE -ne 0) { throw "$What failed ($LASTEXITCODE)" }
}

$buildMode = "release"
$cargoFlags = @("--release")
if ($args.Count -gt 0 -and $args[0] -eq "debug") {
    $buildMode = "debug"
    $cargoFlags = @()
}

pushd $rootDirectory

try {
    # Build a static library (.a / .rlib) for wasm32 consumers that statically link
    # Rust into their own wasm build (e.g. Godot Web / Unity WebGL via Emscripten).
    # JS-facing builds go through release-web.ps1 (wasm-pack). The cdylib crate-type
    # is not built: these consumers link the static library.
    # Uses emscripten_alloc for memory management to delegate allocations to the host's malloc,
    # while keeping math deterministic via libm.
    #
    # Godot's GDExtension for Web loads modules dynamically, requiring Position Independent Code (PIC).
    # The pre-compiled standard library for wasm32-unknown-unknown lacks PIC support.
    # Therefore, we must use the nightly toolchain and -Z build-std to recompile core/alloc with PIC.
    # Note: We specifically DO NOT include +atomics here to keep the build single-threaded (nothreads).
    $oldRustFlags = $env:RUSTFLAGS
    try {
        $env:RUSTFLAGS = "-C relocation-model=pic -C target-cpu=mvp -C target-feature=+simd128"

        Invoke-Checked "Emscripten static library build" {
            & cargo +nightly rustc -Z build-std=core,alloc `
                --target wasm32-unknown-unknown `
                -p ssruntime-ffi `
                @cargoFlags `
                --no-default-features `
                --features "emscripten_alloc,panic-handler,ffi" `
                --crate-type staticlib,rlib
        }
    } finally {
        $env:RUSTFLAGS = $oldRustFlags
    }
}
finally {
    popd
}
