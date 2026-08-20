#!/usr/bin/env pwsh

param(
    # Output directory for the generated third-party notices. These are
    # throwaway build artifacts bundled into release packages, NOT committed to
    # the repo (default lives under the gitignored target/).
    [string]$OutDir
)

$ErrorActionPreference = "Stop"

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

pushd $rootDirectory

try {
    if (-not $OutDir) {
        $OutDir = Join-Path $rootDirectory "target/licenses"
    }

    # Check if cargo-about is installed
    if (-not (Get-Command "cargo-about" -ErrorAction SilentlyContinue)) {
        Write-Host "cargo-about is not installed. Installing it now..."
        cargo install cargo-about --locked --features cli
    }

    $template = "about-markdown.hbs"

    # List of crates with the cargo-about flags pinning each to the feature set
    # it actually ships with, so the notices match the distributed binaries
    # rather than the default-feature build.
    #
    # ssruntime: union of the shipped features (mobile/desktop FFI, wasm, web).
    # Deliberately omit `std` — it pulls in ssconverter (tests only) and would
    # pollute the runtime's minimal notices. So do NOT use --all-features here.
    #
    # ssconverter: native + web. No self-coupling, so --all-features is fine.
    $crates = @(
        @{ Path = "cli";              Flags = @() },
        @{ Path = "libs/ssconverter"; Flags = @("--all-features") },
        @{ Path = "libs/sspj";        Flags = @() },
        @{ Path = "libs/ssab";        Flags = @() },
        @{ Path = "libs/ssruntime";   Flags = @("--no-default-features", "--features", "ffi,libc_alloc,wasm_alloc,emscripten_alloc,web,panic-handler") },
    )

    New-Item -ItemType Directory -Path $OutDir -Force | Out-Null

    # 1. Sync the root license into each crate. cargo-about (and cargo publish)
    # read the license text from a file inside the crate; without it,
    # cargo-about falls back to the generic SPDX template and drops the real
    # copyright holder. These per-crate LICENSE.md copies ARE committed.
    foreach ($crate in $crates) {
        Copy-Item "LICENSE.md" "$($crate.Path)/LICENSE.md" -Force
    }

    # 2. Generate the third-party notices into $OutDir as
    # THIRD-PARTY-LICENSES.<crate>.md. Output goes outside the crate dirs so
    # cargo-about never mistakes a previously generated file for a license file.
    foreach ($crate in $crates) {
        $path = $crate.Path
        $flags = $crate.Flags
        $name = Split-Path $path -Leaf
        $output = Join-Path $OutDir "THIRD-PARTY-LICENSES.$name.md"

        Write-Host "Generating third-party licenses for $path..."
        # $flags is an array; PowerShell splats it into separate arguments
        # (nothing when the crate has no flags).
        cargo about generate $flags `
            --manifest-path "$path/Cargo.toml" `
            --config about.toml `
            $template `
            -o $output
    }

    Write-Host "License generation complete. Notices written to $OutDir"
}
finally {
    popd
}
