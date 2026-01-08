#!/bin/bash
echo "🚀 Starting Fincept Terminal Web Dev Environment..."

# Check if .venv exists
if [ ! -d ".venv" ]; then
    echo "❌ Virtual environment .venv not found!"
    echo "👉 Please run: uv venv .venv --python 3.11 && uv pip install -r src-tauri/resources/requirements-numpy2.txt"
    exit 1
fi

# Kill previous instances
echo "🧹 Cleaning up previous instances..."
lsof -ti:1420 | xargs kill -9 2>/dev/null || true
lsof -ti:3000 | xargs kill -9 2>/dev/null || true
sleep 1

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

# Start Frontend (listen on 0.0.0.0)
echo "📦 Starting Frontend (Vite) on 0.0.0.0:1420..."
bun run dev --host 0.0.0.0 &
FRONTEND_PID=$!

# Wait a moment
sleep 2

# Start Backend (listen on 0.0.0.0)
echo "🦀 Starting Backend (Rust/Axum) on 0.0.0.0:3000..."
echo "ℹ️  Note: Backend runs in foreground. Press Ctrl+C to stop both."
cd src-tauri
FINCEPT_HOST=0.0.0.0 cargo run --bin fincept-server --features web
