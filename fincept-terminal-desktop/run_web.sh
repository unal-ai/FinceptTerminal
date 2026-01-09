#!/bin/bash
echo "🚀 Starting Fincept Terminal Web Dev Environment..."

# Check environment (Conda or .venv)
if [ -n "$CONDA_PREFIX" ]; then
    echo "✅ Detected active Conda environment: $CONDA_PREFIX"
    # Use the active Conda python
    export PYO3_PYTHON="$CONDA_PREFIX/bin/python"
    export FINCEPT_PYTHON_PATH="$CONDA_PREFIX/bin/python"
    # Fix for macOS dyld / Linux ld not finding libpython
    export DYLD_LIBRARY_PATH="$CONDA_PREFIX/lib:$DYLD_LIBRARY_PATH"
    export LD_LIBRARY_PATH="$CONDA_PREFIX/lib:$LD_LIBRARY_PATH"
elif [ -d ".venv" ]; then
     echo "✅ Using local .venv"
     # Use the local .venv python
    export PYO3_PYTHON="$(pwd)/.venv/bin/python"
    export FINCEPT_PYTHON_PATH="$(pwd)/.venv/bin/python"
else
    echo "❌ No valid Python environment found!"
    echo "👉 Option 1 (uv): uv venv .venv --python 3.11 && uv pip install -r src-tauri/resources/requirements-numpy2.txt"
    echo "👉 Option 2 (Conda): mamba create -n fincept ... && mamba activate fincept"
    exit 1
fi

# Kill previous instances
echo "🧹 Cleaning up previous instances..."
lsof -ti:1420 | xargs kill -9 2>/dev/null || true
lsof -ti:3000 | xargs kill -9 2>/dev/null || true
sleep 1

# Set environment variables for Python linkage and runtime
export FINCEPT_SCRIPTS_PATH="$(pwd)/src-tauri/resources/scripts"
export RUST_LOG=info

# Function to kill background processes on exit
cleanup() {
    echo ""
    echo "🛑 Shutting down..."
    kill $BACKEND_PID 2>/dev/null
    exit
}
trap cleanup SIGINT SIGTERM

# Start Backend (listen on 0.0.0.0)
echo "🦀 Starting Backend (Rust/Axum) on 0.0.0.0:3000..."
cd src-tauri
FINCEPT_HOST=0.0.0.0 cargo run --bin fincept-server --features web &
BACKEND_PID=$!
cd ..

# Wait for backend to be ready
echo "⏳ Waiting for backend to start on port 3000..."
while ! nc -z 127.0.0.1 3000; do
  if ! kill -0 $BACKEND_PID 2>/dev/null; then
    echo "❌ Backend process died unexpectedly!"
    exit 1
  fi
  sleep 1
done
echo "✅ Backend is ready!"

# Start Frontend (listen on 0.0.0.0)
echo "📦 Starting Frontend (Vite) on 0.0.0.0:1420..."
echo "ℹ️  Note: Frontend runs in foreground. Press Ctrl+C to stop both."
bun run dev --host 0.0.0.0
