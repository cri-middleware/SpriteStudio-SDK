#!/usr/bin/env pwsh

$ErrorActionPreference = "Stop"

$baseDirectory = Split-Path -Parent $PSCommandPath
$rootDirectory = Split-Path -Parent $baseDirectory

pushd $rootDirectory

try {
    # Check if cargo-about is installed
    if (-not (Get-Command "cargo-about" -ErrorAction SilentlyContinue)) {
        Write-Host "cargo-about is not installed. Installing it now..."
        cargo install cargo-about --locked --features cli
    }

    $template = "about-markdown.hbs"

    # List of crates
    $crates = @(
        "cli",
        "libs/ssconverter",
        "libs/sspj",
        "libs/ssab",
        "libs/ssruntime",
        "libs/ssinterpolate"
    )

    # Create a temporary directory for outputs
    $tempDir = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), [System.IO.Path]::GetRandomFileName())
    New-Item -ItemType Directory -Path $tempDir | Out-Null

    try {
        # 1. Clean existing generated files in workspace to prevent cargo-about recursive matching
        foreach ($path in $crates) {
            $output = "$path/DEPENDENCIES.md"
            if (Test-Path $output) {
                Remove-Item $output -Force
            }
        }

        # 2. Run cargo about to generate outputs in the temp directory
        foreach ($path in $crates) {
            # Replace slashes in path for temp filename
            $tempName = $path.Replace("/", "_")
            $outputTemp = Join-Path $tempDir "$tempName.md"
            
            Write-Host "Generating third-party licenses for $path..."
            cargo about generate `
                --manifest-path "$path/Cargo.toml" `
                --config about.toml `
                $template `
                -o $outputTemp
        }

        # 3. Move generated files to their final locations in each crate directory
        foreach ($path in $crates) {
            $tempName = $path.Replace("/", "_")
            $outputTemp = Join-Path $tempDir "$tempName.md"
            $output = "$path/DEPENDENCIES.md"
            Move-Item $outputTemp $output -Force
        }

        Write-Host "License generation complete."
    }
    finally {
        # Clean up temp directory
        if (Test-Path $tempDir) {
            Remove-Item $tempDir -Recurse -Force | Out-Null
        }
    }
}
finally {
    popd
}
