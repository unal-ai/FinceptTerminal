#!/bin/bash
set -e

# Configuration
ENV_NAME="fincept-dev"

echo "🧪 Starting Headless Test Orchestrator..."

# 1. Activate Environment (similar to run_all_venv.sh logic)
eval "$(mamba shell.bash hook 2>/dev/null || conda shell.bash hook)"
conda activate "$ENV_NAME"

# 2. Add dependencies if missing (quick check)
PYTHON_EXEC="$CONDA_PREFIX/bin/python"
echo "🐍 Using Python: $PYTHON_EXEC"

if ! "$PYTHON_EXEC" -c "import playwright" &> /dev/null; then
    echo "📦 Playwright not found in python. Installing via pip..."
    # Conda's playwright package can sometimes be just the binary/node deps. 
    # Reliability is higher with pip for the python bindings.
    "$PYTHON_EXEC" -m pip install playwright
fi

# 3. Install Playwright Browsers
echo "📦 Ensuring Playwright browsers are installed..."
# Run module directly to ensure we use the right one
"$PYTHON_EXEC" -m playwright install chromium

# 4. Start Server in Background
echo "🚀 Starting Web Server (run_web.sh)..."
# We run run_web.sh in background. It has internal waiting logic.
# Use setsid to easily kill the whole process tree later, or just track PID.
./run_web.sh &
SERVER_PID=$!

# Cleanup Function
cleanup() {
    echo "🛑 Stopping Server (PID $SERVER_PID)..."
    kill $SERVER_PID 2>/dev/null || true
    # run_web.sh spawns children (backend, frontend), attempt to kill them too by PGID or PIDs if known
    # relying on run_web.sh's internal trap for now, but a hard kill is safer for CI
    pkill -P $SERVER_PID || true
    lsof -ti:1420 | xargs kill -9 2>/dev/null || true
    lsof -ti:3000 | xargs kill -9 2>/dev/null || true
}
trap cleanup EXIT

# 5. Wait for Server to be ready (Frontend port 1420)
echo "⏳ Waiting for Frontend (port 1420)..."
attempt=0
while ! nc -z 127.0.0.1 1420; do
    sleep 2
    attempt=$((attempt+1))
    if [ $attempt -gt 30 ]; then
        echo "❌ Frontend failed to start in 60s."
        exit 1
    fi
    # Also check if server process died
    if ! kill -0 $SERVER_PID 2>/dev/null; then
        echo "❌ Server process died!"
        exit 1
    fi
done

# 6. Run Test
echo "🏃 Running Python Smoke Test..."
"$PYTHON_EXEC" tests/headless_smoke_test.py
