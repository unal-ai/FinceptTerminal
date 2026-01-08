#!/bin/bash
# Fincept Terminal - Cross-platform deployment helper (Linux/macOS)
# Uses uv (preferred) or mamba/conda to provision Python deps from conda-forge.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FRONTEND_DIR="$ROOT_DIR/fincept-terminal-desktop"
TAURI_DIR="$FRONTEND_DIR/src-tauri"
REQ_FILE="$TAURI_DIR/resources/requirements-numpy2.txt"
if [ ! -f "$REQ_FILE" ]; then
  REQ_FILE="$TAURI_DIR/resources/requirements.txt"
fi

echo ""
echo "========================================"
echo "  Fincept Terminal Deploy (Unix)"
echo "========================================"
echo ""

ensure_uv() {
  if command -v uv >/dev/null 2>&1; then
    return 0
  fi
  echo "[..] uv not found. Attempting to install uv..."
  if command -v curl >/dev/null 2>&1; then
    # WARNING: This downloads and executes a remote installer script without verification.
    # For production use, consider installing uv via a package manager or verifying
    # the installer against a pinned checksum/signature to mitigate supply-chain risks.
    echo "WARNING: This will download and execute the uv remote installer script without verification."
    echo "         For production use, install uv via a trusted package manager or verify the"
    echo "         installer against a pinned checksum or signature to mitigate supply-chain risks."
    curl -LsSf https://astral.sh/uv/install.sh | sh
    export PATH="$HOME/.cargo/bin:$PATH"
  fi
  command -v uv >/dev/null 2>&1
}

ensure_mamba() {
  if command -v mamba >/dev/null 2>&1; then
    return 0
  fi
  if command -v conda >/dev/null 2>&1; then
    return 0
  fi
  return 1
}

setup_python_env() {
  local env_python=""
  local old_dir="$(pwd)"
  if ensure_uv; then
    echo "[OK] Using uv for Python environment"
    cd "$FRONTEND_DIR"
    uv venv --python 3.11
    uv pip install -r "$REQ_FILE"
    env_python="$FRONTEND_DIR/.venv/bin/python"
    cd "$old_dir"
  elif ensure_mamba; then
    echo "[OK] Using mamba/conda for Python environment"
    local env_name="fincept-terminal"
    if command -v mamba >/dev/null 2>&1; then
      mamba create -y -n "$env_name" -c conda-forge python=3.11
      mamba run -n "$env_name" python -m pip install -r "$REQ_FILE"
    else
      conda create -y -n "$env_name" -c conda-forge python=3.11
      conda run -n "$env_name" python -m pip install -r "$REQ_FILE"
    fi
    env_python="conda run -n $env_name python"
  else
    echo "[!!] Neither uv nor conda/mamba found."
    echo "Install one of:"
    echo "  - uv: https://astral.sh/uv/"
    echo "  - mamba/conda (conda-forge) and re-run."
    exit 1
  fi

  export FINCEPT_PYTHON_PATH="$env_python"
  export FINCEPT_SCRIPTS_PATH="$TAURI_DIR/resources/scripts"
  echo "[OK] FINCEPT_PYTHON_PATH set"
}

build_frontend() {
  echo "[..] Building frontend..."
  cd "$FRONTEND_DIR"
  if command -v bun >/dev/null 2>&1; then
    bun install
    bun run build
  elif command -v npm >/dev/null 2>&1; then
    npm install
    npm run build
  else
    echo "[!!] Bun or npm is required to build the frontend."
    exit 1
  fi
}

build_server() {
  echo "[..] Building web server..."
  cd "$TAURI_DIR"
  cargo build --release --features web --bin fincept-server
}

setup_python_env
build_frontend
build_server

echo ""
echo "[OK] Deployment build complete."
echo "Run the server with:"
echo "  FINCEPT_HOST=0.0.0.0 FINCEPT_PORT=3000 \\"
echo "  FINCEPT_SCRIPTS_PATH=\"$FINCEPT_SCRIPTS_PATH\" \\"
echo "  FINCEPT_PYTHON_PATH=\"$FINCEPT_PYTHON_PATH\" \\"
echo "  \"$TAURI_DIR/target/release/fincept-server\""
