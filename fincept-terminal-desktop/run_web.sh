#!/bin/bash
echo "🚀 Starting Fincept Terminal Web Dev Environment..."

# ==================== LOGGING CONFIGURATION ====================
LOG_DIR="$(pwd)/logs"
LOG_MAX_SIZE_MB=128
LOG_MAX_SIZE_BYTES=$((LOG_MAX_SIZE_MB * 1024 * 1024))
BACKEND_LOG="$LOG_DIR/backend.log"
FRONTEND_LOG="$LOG_DIR/frontend.log"

# Create logs directory if it doesn't exist
mkdir -p "$LOG_DIR"

# Function to rotate and trim logs to stay under size limit
rotate_log() {
    local log_file="$1"
    local max_bytes="$2"
    
    if [ -f "$log_file" ]; then
        local current_size=$(stat -c%s "$log_file" 2>/dev/null || stat -f%z "$log_file" 2>/dev/null || echo 0)
        
        if [ "$current_size" -gt "$max_bytes" ]; then
            # Keep the last half of the log (newest entries)
            local lines_to_keep=$(($(wc -l < "$log_file") / 2))
            tail -n "$lines_to_keep" "$log_file" > "${log_file}.tmp"
            mv "${log_file}.tmp" "$log_file"
            echo "[$(date '+%Y-%m-%d %H:%M:%S')] Log rotated (was ${current_size} bytes)" >> "$log_file"
        fi
    fi
}

# Function to manage total logs directory size
manage_logs_quota() {
    local max_total_bytes="$1"
    local total_size=$(du -sb "$LOG_DIR" 2>/dev/null | cut -f1 || du -sk "$LOG_DIR" 2>/dev/null | awk '{print $1 * 1024}' || echo 0)
    
    if [ "$total_size" -gt "$max_total_bytes" ]; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] Total log size (${total_size} bytes) exceeds quota. Cleaning up..." >> "$BACKEND_LOG"
        # Delete oldest log files (those with .1, .2, etc. suffixes first)
        find "$LOG_DIR" -name "*.log.*" -type f -delete 2>/dev/null
        # If still over quota, truncate main logs
        for log in "$BACKEND_LOG" "$FRONTEND_LOG"; do
            if [ -f "$log" ]; then
                # Keep only last 1000 lines
                tail -n 1000 "$log" > "${log}.tmp" 2>/dev/null
                mv "${log}.tmp" "$log" 2>/dev/null
            fi
        done
    fi
}

# Background log rotation daemon
start_log_rotation_daemon() {
    (
        while true; do
            sleep 60  # Check every minute
            rotate_log "$BACKEND_LOG" "$LOG_MAX_SIZE_BYTES"
            rotate_log "$FRONTEND_LOG" "$LOG_MAX_SIZE_BYTES"
            manage_logs_quota "$LOG_MAX_SIZE_BYTES"
        done
    ) &
    LOG_ROTATION_PID=$!
    echo "📝 Log rotation daemon started (PID: $LOG_ROTATION_PID, max ${LOG_MAX_SIZE_MB}MB)"
}

# ==================== ENVIRONMENT DETECTION ====================

# Check environment (Conda or .venv)
if [ -n "$CONDA_PREFIX" ]; then
    echo "✅ Detected active Conda environment: $CONDA_PREFIX"
    # Use the active Conda python
    export PYO3_PYTHON="$CONDA_PREFIX/bin/python"
    export FINCEPT_PYTHON_PATH="$CONDA_PREFIX/bin/python"
    # Fix for macOS dyld / Linux ld not finding libpython
    export DYLD_LIBRARY_PATH="$CONDA_PREFIX/lib:$DYLD_LIBRARY_PATH"
    export LD_LIBRARY_PATH="$CONDA_PREFIX/lib:$LD_LIBRARY_PATH"
    # Fix for pkg-config not finding conda system libs (glib, gtk3, etc.)
    export PKG_CONFIG_PATH="$CONDA_PREFIX/lib/pkgconfig:$PKG_CONFIG_PATH"
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
    kill $LOG_ROTATION_PID 2>/dev/null
    exit
}
trap cleanup SIGINT SIGTERM

# Start log rotation daemon
start_log_rotation_daemon

# Initialize log files with timestamp
echo "[$(date '+%Y-%m-%d %H:%M:%S')] ========== Backend Starting ==========" >> "$BACKEND_LOG"
echo "[$(date '+%Y-%m-%d %H:%M:%S')] ========== Frontend Starting ==========" >> "$FRONTEND_LOG"

# Start Backend (listen on 0.0.0.0) with logging
echo "🦀 Starting Backend (Rust/Axum) on 0.0.0.0:3000..."
echo "📝 Backend logs: $BACKEND_LOG"
cd src-tauri
FINCEPT_HOST=0.0.0.0 cargo run --bin fincept-server --features web 2>&1 | tee -a "$BACKEND_LOG" &
BACKEND_PID=$!
cd ..

# Wait for backend to be ready
echo "⏳ Waiting for backend to start on port 3000..."
while ! nc -z 127.0.0.1 3000; do
  if ! kill -0 $BACKEND_PID 2>/dev/null; then
    echo "❌ Backend process died unexpectedly!"
    echo "📝 Check logs at: $BACKEND_LOG"
    tail -50 "$BACKEND_LOG"
    exit 1
  fi
  sleep 1
done
echo "✅ Backend is ready!"

# Start Frontend (listen on 0.0.0.0) with logging
echo "📦 Starting Frontend (Vite) on 0.0.0.0:1420..."
echo "📝 Frontend logs: $FRONTEND_LOG"
echo "ℹ️  Note: Frontend runs in foreground. Press Ctrl+C to stop both."
bun run dev --host 0.0.0.0 2>&1 | tee -a "$FRONTEND_LOG"

