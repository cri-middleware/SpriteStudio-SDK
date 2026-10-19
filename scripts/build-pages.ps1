#!/usr/bin/env pwsh
#
# Build the GitHub Pages site locally -- everything .github/workflows/pages.yml
# publishes, in the same order: the documentation toolchain, the documentation
# in both locales into site/, then the workspace rustdoc grafted on at site/api/.
# PowerShell twin of build-pages.sh.
#
#   scripts/build-pages.ps1              # the tree pages.yml uploads
#   scripts/build-pages.ps1 serve=yes    # ... and serve it on :8000
#   scripts/build-pages.ps1 api=no       # the documentation on its own, no cargo
#   scripts/build-pages.ps1 docs=no      # the rustdoc on its own, onto the site as it is
#
# What this does NOT reproduce is the deploy job (actions/deploy-pages) -- there
# is no local equivalent of publishing.
#
# Rerunning is safe; nothing here is incremental (`cargo doc` is, and is left to
# manage its own target directory).
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
    docs    = "yes"
    api     = "yes"
    prepare = "auto"
    venv    = ".venv"
    serve   = "no"
    port    = "8000"
}

function Show-Usage {
    Write-Host "Usage: $APP [options]"
    Write-Host "$APP options:"
    Write-Host "  docs=<yes|no>           Build the documentation, both locales (default: yes)."
    Write-Host "                          no keeps whatever site/ already holds -- handy when"
    Write-Host "                          only the rustdoc is being re-checked."
    Write-Host "  api=<yes|no>            Build the workspace rustdoc and graft it onto the site"
    Write-Host "                          at site/api/ (default: yes). api=no leaves the nav's"
    Write-Host "                          Rust API entry pointing at nothing, which is what a"
    Write-Host "                          plain build-docs.ps1 run gives you."
    Write-Host "  prepare=<auto|yes|no>   Run prepare-docs.ps1 first (default: auto)."
    Write-Host "                          auto = only when no zensical is found."
    Write-Host "  venv=<dir>              Virtualenv for the docs toolchain, relative to the"
    Write-Host "                          repo root (default: .venv). Passed to prepare-docs.ps1"
    Write-Host "                          and build-docs.ps1."
    Write-Host "  serve=<yes|no>          Serve site/ over HTTP when the build is done"
    Write-Host "                          (default: no). Foreground; Ctrl-C to stop."
    Write-Host "  port=<n>                Port for serve=yes (default: 8000)"
    Write-Host "  -h | --help             Show this help"
    Write-Host ""
    Write-Host "Output: site/ (en) + site/ja/ (ja) + site/api/ (rustdoc) -- the exact tree"
    Write-Host "pages.yml uploads. Both documentation builds are --strict, so a broken internal"
    Write-Host "link fails the build here rather than on a release publish."
    Write-Host ""
    Write-Host "The rustdoc is deliberately built WITHOUT RUSTDOCFLAGS=-D warnings, matching"
    Write-Host "pages.yml: that gate belongs on the pull request (ci.yml), not on a publish."
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
        # Lower-cased where the option is a keyword; venv=/port= are not.
        $opts[$key] = if ($key -in "venv", "port") { $value } else { $value.ToLower() }
    } else {
        Fail "unknown argument '$item' (options are key=value; see $APP --help)" 2
    }
}

if ($opts.docs    -notin @("yes", "no"))         { Fail "docs must be yes or no (got '$($opts.docs)')" 2 }
if ($opts.api     -notin @("yes", "no"))         { Fail "api must be yes or no (got '$($opts.api)')" 2 }
if ($opts.prepare -notin @("auto", "yes", "no")) { Fail "prepare must be auto, yes or no (got '$($opts.prepare)')" 2 }
if ($opts.serve   -notin @("yes", "no"))         { Fail "serve must be yes or no (got '$($opts.serve)')" 2 }
if ($opts.port -notmatch '^\d+$')                { Fail "port must be a number (got '$($opts.port)')" 2 }

# Nothing to build at all is a typo, not a request: it would leave site/ exactly
# as it was and report success.
if ($opts.docs -eq "no" -and $opts.api -eq "no") {
    Fail "docs=no api=no builds nothing (see $APP --help)" 2
}

$prepareNote = if ($opts.docs -eq "yes") { "" } else { " (unused: docs=no)" }
$serveNote = if ($opts.serve -eq "no") { "" } else { " (port $($opts.port))" }

Write-Host "options"
Write-Host "  docs    => $($opts.docs)"
Write-Host "  api     => $($opts.api)"
Write-Host "  prepare => $($opts.prepare)$prepareNote"
Write-Host "  venv    => $($opts.venv)"
Write-Host "  serve   => $($opts.serve)$serveNote"
Write-Host ""

# --- preflight ------------------------------------------------------------
if (-not (Test-Path (Join-Path $ROOT_DIR "mkdocs.base.yml")) -or
    -not (Test-Path (Join-Path $ROOT_DIR "Cargo.toml"))) {
    Fail "mkdocs.base.yml or Cargo.toml not found -- is this the SpriteStudio-SDK repo?"
}

if ($opts.api -eq "yes" -and -not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    [Console]::Error.WriteLine("${APP}: cargo not found -- install the Rust toolchain (https://rustup.rs), or")
    Fail "build the documentation on its own with '$APP api=no'"
}

$SITE_DIR = Join-Path $ROOT_DIR "site"
$API_DIR = Join-Path $SITE_DIR "api"

# Every script here resolves its own paths from the repo root, and site/ is
# written relative to it, so run from there however this was invoked.
Push-Location $ROOT_DIR
try {
    function Step {
        param([string] $What, [scriptblock] $Body)
        Write-Host ""
        Write-Host ">> $What"
        & $Body
        if ($LASTEXITCODE -ne 0) { Fail "$What failed ($LASTEXITCODE)" $LASTEXITCODE }
    }

    # --- documentation ----------------------------------------------------
    # Same probe as build-docs.ps1: a venv laid out either way, else PATH (which
    # is how a run with no venv at all still works).
    function Test-Zensical {
        $venvDir = Join-Path $ROOT_DIR $opts.venv
        $venvBin = if (Test-Path (Join-Path $venvDir "Scripts")) {
            Join-Path $venvDir "Scripts"
        } else {
            Join-Path $venvDir "bin"
        }
        foreach ($candidate in @((Join-Path $venvBin "zensical.exe"), (Join-Path $venvBin "zensical"))) {
            if (Test-Path $candidate) { return $true }
        }
        return [bool](Get-Command zensical -ErrorAction SilentlyContinue)
    }

    $prepareScript = Join-Path $ROOT_DIR "scripts/prepare-docs.ps1"
    $buildScript = Join-Path $ROOT_DIR "scripts/build-docs.ps1"

    if ($opts.docs -eq "yes") {
        switch ($opts.prepare) {
            "yes" {
                Step "prepare-docs.ps1 venv=$($opts.venv)" { & $prepareScript "venv=$($opts.venv)" }
            }
            "auto" {
                if (Test-Zensical) {
                    Write-Host "${APP}: zensical already available; skipping prepare-docs.ps1 (prepare=yes forces it)"
                } else {
                    Step "prepare-docs.ps1 venv=$($opts.venv)" { & $prepareScript "venv=$($opts.venv)" }
                }
            }
            "no" { }
        }

        # The guide trees and both locales, English first -- build-docs.ps1 owns
        # that order.
        Step "build-docs.ps1 venv=$($opts.venv)" { & $buildScript "venv=$($opts.venv)" }
    } else {
        Write-Host "${APP}: docs=no -- site/ keeps whatever the last build left there"
        if (-not (Test-Path (Join-Path $SITE_DIR "index.html"))) {
            [Console]::Error.WriteLine("${APP}: ...and there is no site/index.html, so the tree below is a bare /api/")
        }
    }

    # --- rustdoc ----------------------------------------------------------
    # `cargo doc` renders into <target>/doc; the site wants it at site/api/. A
    # local CARGO_TARGET_DIR is honoured, because plenty of working trees set one
    # and the hardcoded `target/` pages.yml can rely on would then be empty.
    if ($opts.api -eq "yes") {
        Step "cargo doc --workspace --no-deps --all-features" {
            & cargo doc --workspace --no-deps --all-features
        }

        $targetDir = if ($env:CARGO_TARGET_DIR) { $env:CARGO_TARGET_DIR } else { Join-Path $ROOT_DIR "target" }
        $docDir = Join-Path $targetDir "doc"
        if (-not (Test-Path $docDir)) {
            Fail "cargo doc produced no $docDir"
        }

        Write-Host ""
        Write-Host ">> copying $docDir/ into site/api/"
        if (Test-Path $API_DIR) { Remove-Item -Recurse -Force $API_DIR }
        New-Item -ItemType Directory -Force -Path $API_DIR | Out-Null
        Copy-Item -Path (Join-Path $docDir "*") -Destination $API_DIR -Recurse -Force

        # `cargo doc` writes no index.html at the root of a workspace -- with
        # several crates there is no single page to land on. The nav entry in both
        # locale configs links straight to ssruntime, and this redirect covers
        # anyone who trims the URL back to /api/. Keep it identical to the one in
        # pages.yml and in the .sh twin.
        #
        # Five of the seven workspace members land here: libs/ssruntime-ffi and
        # libs/ssconverter-ffi set `doc = false`, deliberately. They are cdylib +
        # staticlib with no rlib, so nothing can `use` them from Rust; their
        # surface is the C API, documented by the cbindgen headers and the Player
        # Porting Guide. More to the point, each renames its lib to the crate it
        # wraps (`ssruntime`, `ssconverter`), so documenting them would write two
        # crates into one target/doc/ssruntime and clobber the real one.
        $redirect = @'
<!doctype html>
<meta charset="utf-8">
<title>SpriteStudio SDK — Rust API</title>
<meta http-equiv="refresh" content="0; url=ssruntime/index.html">
<link rel="canonical" href="ssruntime/index.html">
<a href="ssruntime/index.html">SpriteStudio SDK Rust API reference</a>
'@
        # LF and a trailing newline, so the file matches the .sh twin's heredoc
        # byte for byte.
        [System.IO.File]::WriteAllText((Join-Path $API_DIR "index.html"), ($redirect -replace "`r`n", "`n") + "`n")
    } else {
        Write-Host ""
        Write-Host "${APP}: api=no -- site/api is not part of this build"
    }
} finally {
    Pop-Location
}

# --- what landed ----------------------------------------------------------
Write-Host ""
Write-Host "${APP}: done -> site/"
function Report {
    param([string] $Path, [string] $Label)
    if (Test-Path (Join-Path $ROOT_DIR $Path)) {
        Write-Host "  ${Label}: $Path"
    } else {
        Write-Host "  ${Label}: MISSING ($Path)"
    }
}
Report "site/index.html" "en  "
Report "site/ja/index.html" "ja  "
if ($opts.api -eq "yes") {
    Report "site/api/ssruntime/index.html" "api "
}

# --- serve ----------------------------------------------------------------
# Bound to loopback, in the command it suggests as well. On every interface,
# http.server announces http://[::]:PORT/, which is not a secure context, so
# Copy as Markdown cannot copy there.
#
# py first: it is the Windows launcher, and what build-docs.ps1 already tells
# people to serve with. python/python3 are the fallbacks elsewhere -- on Windows
# a bare `python` can be the Store stub that does nothing.
$PYTHON_CMD = $null
foreach ($candidate in @("py", "python", "python3")) {
    if (Get-Command $candidate -ErrorAction SilentlyContinue) { $PYTHON_CMD = $candidate; break }
}

Write-Host ""
if ($opts.serve -eq "yes") {
    if (-not $PYTHON_CMD) {
        Fail "no python on PATH to serve with -- use any static server on site/"
    }
    Write-Host "${APP}: http://localhost:$($opts.port)/  (docs)"
    Write-Host "${APP}: http://localhost:$($opts.port)/ja/  (Japanese)"
    Write-Host "${APP}: http://localhost:$($opts.port)/api/  (Rust API)"
    Write-Host "${APP}: Ctrl-C to stop"
    Write-Host ""
    Push-Location $ROOT_DIR
    try {
        & $PYTHON_CMD -m http.server $opts.port --bind 127.0.0.1 -d site
    } finally {
        Pop-Location
    }
    exit $LASTEXITCODE
} else {
    $shown = if ($PYTHON_CMD) { $PYTHON_CMD } else { "python" }
    Write-Host "${APP}: serve it (one server over the whole tree, so the language selector and"
    Write-Host "${APP}: the nav's Rust API entry both resolve):"
    Write-Host "${APP}:   $APP serve=yes"
    Write-Host "${APP}:   $shown -m http.server $($opts.port) --bind 127.0.0.1 -d site"
}
