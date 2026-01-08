#!/bin/bash
echo "🚀 Starting Fincept Terminal Web Dev Environment..."

# Check if .venv exists
if [ ! -d ".venv" ]; then
    echo "❌ Virtual environment .venv not found!"
    echo "👉 Please run: uv venv .venv --python 3.11 && uv pip install -r src-tauri/resources/requirements-numpy2.txt"
    exit 1
fi

# Set environment variables for Python linkage and runtime
export PYO3_PYTHON="$(pwd)/.venv/bin/python"
export FINCEPT_PYTHON_PATH="$(pwd)/.venv/bin/python"
export FINCEPT_SCRIPTS_PATH="$(pwd)/src-tauri/resources/scripts"
export RUST_LOG=info

# Function to kill background processes on exit
cleanup() {
    echo ""
    echo "🛑 Shutting down..."
    kill $FRONTEND_PID 2>/dev/null
    exit
}
trap cleanup SIGINT SIGTERM

# Start Frontend
echo "📦 Starting Frontend (Vite)..."
bun run dev &
FRONTEND_PID=$!

# Wait a moment
sleep 2

# Start Backend
echo "🦀 Starting Backend (Rust/Axum)..."
echo "ℹ️  Note: Backend runs in foreground. Press Ctrl+C to stop both."
cd src-tauri
cargo run --bin fincept-server --features web
