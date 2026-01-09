#!/bin/bash
set -e

# Configuration
MINIFORGE_DIR="$(pwd)/.miniforge"
ENV_NAME="fincept-dev"
ENV_FILE="../reproduction_environment.yml"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
RED='\033[0;31m'
NC='\033[0m' # No Color

echo -e "${BLUE}🚀 Starting Fincept Terminal Bootstrapper...${NC}"

# 1. Check/Install Miniforge
if ! command -v mamba &> /dev/null && ! command -v conda &> /dev/null; then
    if [ -d "$MINIFORGE_DIR" ]; then
        echo -e "${GREEN}✅ Miniforge found at $MINIFORGE_DIR${NC}"
        source "$MINIFORGE_DIR/bin/activate"
    else
        echo -e "${BLUE}📦 Miniforge not found. Installing locally to $MINIFORGE_DIR...${NC}"
        
        OS="$(uname -s)"
        ARCH="$(uname -m)"
        
        if [ "$OS" = "Darwin" ]; then
            target_os="MacOSX"
        elif [ "$OS" = "Linux" ]; then
            target_os="Linux"
        else
            echo -e "${RED}❌ Unsupported OS: $OS${NC}"
            exit 1
        fi
        
        INSTALLER_URL="https://github.com/conda-forge/miniforge/releases/latest/download/Miniforge3-${target_os}-${ARCH}.sh"
        echo -e "${BLUE}⬇️  Downloading Miniforge from $INSTALLER_URL...${NC}"
        
        curl -L -o miniforge.sh "$INSTALLER_URL"
        bash miniforge.sh -b -p "$MINIFORGE_DIR"
        rm miniforge.sh
        
        echo -e "${GREEN}✅ Miniforge installed.${NC}"
        source "$MINIFORGE_DIR/bin/activate"
    fi
else
    echo -e "${GREEN}✅ Detected existing Conda/Mamba installation.${NC}"
fi

# 2. Check/Create Environment
echo -e "${BLUE}🔍 Checking environment '$ENV_NAME'...${NC}"

if ! mamba info --envs | grep -q "$ENV_NAME"; then
    echo -e "${BLUE}📦 Creating environment '$ENV_NAME'...${NC}"
    
    OS="$(uname -s)"
    if [ "$OS" = "Darwin" ]; then
        echo -e "${BLUE}🍎 macOS detected. Installing Python 3.12 + Playwright...${NC}"
        # macOS Conda doesn't have webkit2gtk, but Playwright manages its own browsers
        mamba create -n "$ENV_NAME" python=3.12 playwright -c conda-forge -y
    else
        # Linux / Other
        if [ -f "$ENV_FILE" ]; then
            echo -e "${BLUE}🐧 Linux/other detected. Installing from $ENV_FILE...${NC}"
            mamba create -n "$ENV_NAME" -f "$ENV_FILE" -y
            # We might need to inject playwright if it's not in the file, but for now assuming default path:
            mamba install -n "$ENV_NAME" playwright -c conda-forge -y
        else
            echo -e "${BLUE}⚠️  Environment file not found. Installing default Linux dependencies...${NC}"
            mamba create -n "$ENV_NAME" -c conda-forge glib gtk3 webkit2gtk4.1 librsvg patchelf python=3.12 playwright -y
        fi
    fi
else
    echo -e "${GREEN}✅ Environment '$ENV_NAME' exists.${NC}"
fi

# 3. Activate and Run
echo -e "${BLUE}🚀 Activating environment and starting application...${NC}"

# We need to activate in the current shell script context to pass it to run_web.sh
# 'conda activate' usually requires 'eval "$(conda shell.bash hook)"'
eval "$(mamba shell.bash hook 2>/dev/null || conda shell.bash hook)"
conda activate "$ENV_NAME"

echo -e "${GREEN}✅ Active Environment: $CONDA_PREFIX${NC}"

# Install essential Python dependencies for the backend
REQUIREMENTS_FILE="src-tauri/resources/requirements-numpy2.txt"
if [ -f "$REQUIREMENTS_FILE" ]; then
    echo -e "${BLUE}📦 Installing Python dependencies from $REQUIREMENTS_FILE...${NC}"
    # Install key packages needed for market data - faster than full requirements
    echo -e "${BLUE}📦 Installing essential packages (yfinance, pandas, numpy)...${NC}"
    pip install --quiet yfinance pandas numpy requests aiohttp feedparser || {
        echo -e "${RED}⚠️  Some packages failed to install. Trying individually...${NC}"
        pip install yfinance || true
        pip install pandas || true
        pip install numpy || true
    }
    echo -e "${GREEN}✅ Essential Python packages installed.${NC}"
    
    # Optionally install full requirements (uncomment for complete setup)
    # echo -e "${BLUE}📦 Installing full requirements (this may take a while)...${NC}"
    # pip install -r "$REQUIREMENTS_FILE"
else
    echo -e "${RED}⚠️  Requirements file not found: $REQUIREMENTS_FILE${NC}"
fi

# Delegate to run_web.sh
./run_web.sh
