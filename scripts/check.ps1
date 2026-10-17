#!/usr/bin/env pwsh
#
# Run the checks a change has to pass before it is pushed: ci.yml's lint job
# and its workspace tests, and on request the slower ones. PowerShell twin of check.sh.
#
#   scripts/check.ps1                # ci.yml's lint job and its workspace tests
#   scripts/check.ps1 features=yes   # ... and ssruntime's feature sets, one at a time
#   scripts/check.ps1 msrv=yes       # ... and the toolchain Cargo.toml declares
#   scripts/check.ps1 docs=yes       # ... and the Player Porting Guide, both locales
#
# Every check runs, whatever failed before it, and the summary at the end names
# each one as passed, FAILED or skipped. A check whose prerequisite is missing
# (the wasm32 target, the MSRV toolchain) is skipped rather than failed, so read
# the summary, not the exit status: 0 says that nothing failed, not that
# everything ran.
#
# The default set is ci.yml's lint job plus its Test step; keep the two in step.
# features=yes goes further than CI does. --all-features compiles every cfg at
# once, so it hides code that only one feature set compiles, or leaves dead; CI
# builds a few of the sets and lints none of them.
$ErrorActionPreference = "Stop"

$APP = Split-Path -Leaf $PSCommandPath
$ROOT_DIR = Split-Path -Parent (Split-Path -Parent $PSCommandPath)

function Fail {
    param([string] $Message, [int] $Code = 1)
    [Console]::Error.WriteLine("${APP}: $Message")
    exit $Code
}

# --- options --------------------------------------------------------------
$opts = [ordered]@{
    features = "no"
    msrv     = "no"
    docs     = "no"
}

function Show-Usage {
    Write-Host "Usage: $APP [options]"
    Write-Host "$APP options:"
    Write-Host "  features=<yes|no>   Also lint and document ssruntime with each feature set on"
    Write-Host "                      its own, native and wasm32, and lint the browser converter"
    Write-Host "                      (default: no)."
    Write-Host "  msrv=<yes|no>       Also cargo check the workspace with the rust-version in"
    Write-Host "                      Cargo.toml, as ci.yml's msrv job does (default: no)."
    Write-Host "  docs=<yes|no>       Also build the Player Porting Guide in both locales, with"
    Write-Host "                      scripts/build-docs.ps1 (default: no). Run this when"
    Write-Host "                      libs/ssruntime/docs changed."
    Write-Host "  -h | --help         Show this help"
    Write-Host ""
    Write-Host "Always run: cargo fmt --check, the license copies, clippy and rustdoc with"
    Write-Host "warnings denied, and the workspace tests."
    Write-Host ""
    Write-Host "Exit status: 0 when no check failed, 1 when one did, 2 on a usage error. A"
    Write-Host "skipped check does not fail the run; the summary lists it with the reason."
}

foreach ($item in $args) {
    if ($item -in "-h", "--help", "help") {
        Show-Usage
        exit 0
    } elseif ($item -match "=") {
        $key, $value = $item -split "=", 2
        if (-not $opts.Contains($key)) {
            Fail "unknown option '$key' (see $APP --help)" 2
        }
        $opts[$key] = $value.ToLower()
    } else {
        Fail "unknown argument '$item' (options are key=value; see $APP --help)" 2
    }
}

if ($opts.features -notin @("yes", "no")) { Fail "features must be yes or no (got '$($opts.features)')" 2 }
if ($opts.msrv     -notin @("yes", "no")) { Fail "msrv must be yes or no (got '$($opts.msrv)')" 2 }
if ($opts.docs     -notin @("yes", "no")) { Fail "docs must be yes or no (got '$($opts.docs)')" 2 }

Write-Host "options"
Write-Host "  features => $($opts.features)"
Write-Host "  msrv     => $($opts.msrv)"
Write-Host "  docs     => $($opts.docs)"

# --- preflight ------------------------------------------------------------
if (-not (Test-Path (Join-Path $ROOT_DIR "Cargo.toml")) -or
    -not (Test-Path (Join-Path $ROOT_DIR "libs/ssruntime/Cargo.toml"))) {
    Fail "Cargo.toml not found -- is this the SpriteStudio-SDK repo?"
}
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Fail "cargo not found -- install the Rust toolchain (https://rustup.rs)"
}

# --- runners --------------------------------------------------------------
$script:summary = New-Object System.Collections.Generic.List[string]
$script:passed = 0
$script:failed = 0
$script:skipped = 0

# Invoke-Check <label> <command and its arguments> [-Environment @{ NAME = value }]
function Invoke-Check {
    param([string] $Label, [string[]] $Command, [hashtable] $Environment = @{})
    $shown = @()
    foreach ($name in $Environment.Keys) { $shown += "$name=$($Environment[$name])" }
    $shown += $Command
    Write-Host ""
    Write-Host "==> $Label"
    Write-Host "    $($shown -join ' ')"

    $saved = @{}
    foreach ($name in $Environment.Keys) {
        $saved[$name] = [Environment]::GetEnvironmentVariable($name)
        [Environment]::SetEnvironmentVariable($name, $Environment[$name])
    }
    $ok = $false
    try {
        # A command that never starts sets no exit code, so -1 marks that case.
        $global:LASTEXITCODE = -1
        $rest = @()
        if ($Command.Count -gt 1) { $rest = $Command[1..($Command.Count - 1)] }
        & $Command[0] @rest
        $ok = ($LASTEXITCODE -eq 0)
    } catch {
        [Console]::Error.WriteLine("${APP}: $_")
    } finally {
        foreach ($name in $saved.Keys) { [Environment]::SetEnvironmentVariable($name, $saved[$name]) }
    }

    if ($ok) {
        $script:passed++
        $script:summary.Add("  passed   $Label")
    } else {
        $script:failed++
        $script:summary.Add("  FAILED   $Label")
    }
}

function Skip-Check {
    param([string] $Label, [string] $Reason)
    Write-Host ""
    Write-Host "==> ${Label}: skipped -- $Reason"
    $script:skipped++
    $script:summary.Add("  skipped  $Label -- $Reason")
}

# `cargo package` and wasm-pack ship each crate's own copy of the root LICENSE
# files, which scripts/generate-licenses.ps1 keeps in sync.
function Test-LicenseCopies {
    $status = 0
    foreach ($copy in (& git ls-files "*/LICENSE.md" "*/LICENSE.ja.md")) {
        $root = Split-Path -Leaf $copy
        if ((Get-FileHash $root).Hash -ne (Get-FileHash $copy).Hash) {
            [Console]::Error.WriteLine("$copy differs from the root ${root}: copy it over, or run scripts/generate-licenses.ps1")
            $status = 1
        }
    }
    $global:LASTEXITCODE = $status
}

# No --features at all for the empty set: Windows PowerShell drops an empty
# argument on its way to a native command.
function Get-FeatureArgs {
    param([string] $Set)
    if ($Set) { return @("--no-default-features", "--features", $Set) }
    return @("--no-default-features")
}

# The lines rustup prints, or $null when it fails to run.
function Invoke-Rustup {
    try {
        $global:LASTEXITCODE = -1
        $lines = & rustup @args
        if ($LASTEXITCODE -ne 0) { return $null }
        return ,@($lines)
    } catch {
        return $null
    }
}

function Get-SetName {
    param([string] $Set)
    if ($Set) { return $Set }
    return "none"
}

$denyRustdoc = @{ RUSTDOCFLAGS = "-D warnings" }

Push-Location $ROOT_DIR
try {
    # --- what ci.yml runs on a pull request -------------------------------
    Invoke-Check "fmt" @("cargo", "fmt", "--all", "--", "--check")
    Invoke-Check "license copies" @("Test-LicenseCopies")
    Invoke-Check "clippy (workspace)" @("cargo", "clippy", "--workspace", "--all-targets", "--all-features", "--", "-D", "warnings")
    Invoke-Check "rustdoc (workspace)" @("cargo", "doc", "--workspace", "--no-deps", "--all-features") -Environment $denyRustdoc
    Invoke-Check "tests (workspace)" @("cargo", "test", "--workspace", "--all-features", "--", "--test-threads=1")

    # --- features=yes -----------------------------------------------------
    if ($opts.features -eq "yes") {
        # With the tests, so a test-only item that one set leaves unused is caught.
        foreach ($combo in @("", "ffi", "std", "ffi,std", "web", "web,std", "ffi,web", "ffi,web,std")) {
            Invoke-Check "clippy ssruntime [$(Get-SetName $combo)]" (@("cargo", "clippy", "-p", "ssruntime", "--all-targets") + (Get-FeatureArgs $combo) + @("--", "-D", "warnings"))
        }
        # The allocator sets are for the no_std binaries; a test build already has
        # std's allocator, so they are linted as a library only.
        foreach ($combo in @("ffi,libc_alloc,panic-handler", "ffi,custom_alloc,panic-handler")) {
            Invoke-Check "clippy ssruntime [$combo]" (@("cargo", "clippy", "-p", "ssruntime") + (Get-FeatureArgs $combo) + @("--", "-D", "warnings"))
        }
        foreach ($combo in @("", "web", "std", "ffi", "ffi,web")) {
            Invoke-Check "rustdoc ssruntime [$(Get-SetName $combo)]" (@("cargo", "doc", "-p", "ssruntime", "--no-deps") + (Get-FeatureArgs $combo)) -Environment $denyRustdoc
        }

        $wasm = "wasm32-unknown-unknown"
        if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
            Skip-Check "clippy for $wasm" "rustup not found, so the target cannot be checked for"
        } else {
            $targets = Invoke-Rustup target list --installed
            if ($null -eq $targets) {
                Skip-Check "clippy for $wasm" "rustup target list failed"
            } elseif ($targets -notcontains $wasm) {
                Skip-Check "clippy for $wasm" "the target is not installed (rustup target add $wasm)"
            } else {
                foreach ($combo in @("web", "ffi,wasm_alloc,panic-handler", "ffi,std")) {
                    Invoke-Check "clippy ssruntime $wasm [$combo]" (@("cargo", "clippy", "-p", "ssruntime", "--target", $wasm) + (Get-FeatureArgs $combo) + @("--", "-D", "warnings"))
                }
                Invoke-Check "clippy ssconverter $wasm [web]" @("cargo", "clippy", "-p", "ssconverter", "--target", $wasm, "--no-default-features", "--features", "web", "--", "-D", "warnings")
            }
        }
    }

    # --- msrv=yes ---------------------------------------------------------
    if ($opts.msrv -eq "yes") {
        $match = Select-String -Path "Cargo.toml" -Pattern '^rust-version = "(.+)"' | Select-Object -First 1
        if (-not $match) {
            [Console]::Error.WriteLine("")
            [Console]::Error.WriteLine("==> msrv: rust-version not found in the workspace Cargo.toml")
            $script:failed++
            $script:summary.Add("  FAILED   msrv -- rust-version not found in the workspace Cargo.toml")
        } else {
            $version = $match.Matches[0].Groups[1].Value
            if (-not (Get-Command rustup -ErrorAction SilentlyContinue)) {
                Skip-Check "msrv $version" "rustup not found"
            } else {
                $toolchains = Invoke-Rustup toolchain list
                if ($null -eq $toolchains) {
                    Skip-Check "msrv $version" "rustup toolchain list failed"
                } elseif (-not ($toolchains -match "^$([regex]::Escape($version))-")) {
                    Skip-Check "msrv $version" "the toolchain is not installed (rustup toolchain install $version)"
                } else {
                    Invoke-Check "msrv $version" @("cargo", "+$version", "check", "--workspace", "--all-targets", "--all-features", "--locked")
                }
            }
        }
    }

    # --- docs=yes ---------------------------------------------------------
    if ($opts.docs -eq "yes") {
        Invoke-Check "docs (both locales)" @((Join-Path $ROOT_DIR "scripts/build-docs.ps1"))
    }
} finally {
    Pop-Location
}

# --- summary --------------------------------------------------------------
Write-Host ""
Write-Host "summary"
foreach ($line in $script:summary) { Write-Host $line }
Write-Host "$($script:passed) passed, $($script:failed) failed, $($script:skipped) skipped"
if ($script:failed -gt 0) { exit 1 }
exit 0
