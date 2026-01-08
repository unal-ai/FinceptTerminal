#!/usr/bin/env pwsh
# Fincept Terminal - Cross-platform deployment helper (Windows)
# Uses uv (preferred) or mamba/conda to provision Python deps from conda-forge.

$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$frontendDir = Join-Path $root "fincept-terminal-desktop"
$tauriDir = Join-Path $frontendDir "src-tauri"
$reqFile = Join-Path $tauriDir "resources\\requirements-numpy2.txt"
if (-not (Test-Path $reqFile)) {
  $reqFile = Join-Path $tauriDir "resources\\requirements.txt"
}

Write-Host ""
Write-Host "========================================"
Write-Host "  Fincept Terminal Deploy (Windows)"
Write-Host "========================================"
Write-Host ""

function Ensure-Uv {
  if (Get-Command uv -ErrorAction SilentlyContinue) { return $true }
  Write-Host "[..] uv not found. Attempting to install uv..."
  try {
    # WARNING: This downloads and executes a remote PowerShell script without verification.
    # For production use, consider installing uv via a package manager or verifying
    # the installer against a pinned checksum/signature to mitigate supply-chain risks.
    iwr https://astral.sh/uv/install.ps1 -UseBasicParsing | iex
  } catch {
    return $false
  }
  return [bool](Get-Command uv -ErrorAction SilentlyContinue)
}

function Ensure-Conda {
  if (Get-Command mamba -ErrorAction SilentlyContinue) { return $true }
  if (Get-Command conda -ErrorAction SilentlyContinue) { return $true }
  return $false
}

function Setup-PythonEnv {
  $envPython = $null
  if (Ensure-Uv) {
    Write-Host "[OK] Using uv for Python environment"
    Push-Location $frontendDir
    uv venv --python 3.11
    uv pip install -r $reqFile
    Pop-Location
    $envPython = (Join-Path $frontendDir ".venv\\Scripts\\python.exe")
  } elseif (Ensure-Conda) {
    Write-Host "[OK] Using mamba/conda for Python environment"
    $envName = "fincept-terminal"
    if (Get-Command mamba -ErrorAction SilentlyContinue) {
      mamba create -y -n $envName -c conda-forge python=3.11
      mamba run -n $envName python -m pip install -r $reqFile
      $envPython = "mamba run -n $envName python"
    } else {
      conda create -y -n $envName -c conda-forge python=3.11
      conda run -n $envName python -m pip install -r $reqFile
      $envPython = "conda run -n $envName python"
    }
  } else {
    Write-Host "[!!] Neither uv nor conda/mamba found."
    Write-Host "Install one of:"
    Write-Host "  - uv: https://astral.sh/uv/"
    Write-Host "  - Mambaforge/Miniconda (conda-forge) and re-run."
    exit 1
  }

  $env:FINCEPT_PYTHON_PATH = $envPython
  $env:FINCEPT_SCRIPTS_PATH = (Join-Path $tauriDir "resources\\scripts")
  Write-Host "[OK] FINCEPT_PYTHON_PATH set"
}

function Build-Frontend {
  Write-Host "[..] Building frontend..."
  Push-Location $frontendDir
  if (Get-Command bun -ErrorAction SilentlyContinue) {
    bun install
    bun run build
  } elseif (Get-Command npm -ErrorAction SilentlyContinue) {
    npm install
    npm run build
  } else {
    Write-Host "[!!] Bun or npm is required to build the frontend."
    exit 1
  }
  Pop-Location
}

function Build-Server {
  Write-Host "[..] Building web server..."
  Push-Location $tauriDir
  cargo build --release --features web --bin fincept-server
  Pop-Location
}

Setup-PythonEnv
Build-Frontend
Build-Server

Write-Host ""
Write-Host "[OK] Deployment build complete."
Write-Host "Run the server with:"
Write-Host "  `$env:FINCEPT_HOST='0.0.0.0'"
Write-Host "  `$env:FINCEPT_PORT='3000'"
Write-Host "  `$env:FINCEPT_SCRIPTS_PATH='$($env:FINCEPT_SCRIPTS_PATH)'"
Write-Host "  `$env:FINCEPT_PYTHON_PATH='$($env:FINCEPT_PYTHON_PATH)'"
Write-Host "  `"$tauriDir\\target\\release\\fincept-server.exe`""
