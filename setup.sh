#!/bin/bash

# 1a. setup.sh - one-click setup script
# 1b. installs all dependencies and builds the app
# 1c. run with: chmod +x setup.sh && ./setup.sh

echo "======================================"
echo "  B-Roll Scrambler Setup Script"
echo "======================================"
echo ""

# 2a. check if running as root (we dont want that)
if [ "$EUID" -eq 0 ]; then
  echo "dont run this as root bruh"
  echo "run as normal user, itll ask for sudo when needed"
  exit 1
fi

# 2b. detect package manager
if command -v apt &> /dev/null; then
    PKG_MANAGER="apt"
    INSTALL_CMD="sudo apt install -y"
elif command -v dnf &> /dev/null; then
    PKG_MANAGER="dnf"
    INSTALL_CMD="sudo dnf install -y"
elif command -v pacman &> /dev/null; then
    PKG_MANAGER="pacman"
    INSTALL_CMD="sudo pacman -S --noconfirm"
else
    echo "couldnt detect package manager"
    echo "install dependencies manually, check README.md"
    exit 1
fi

echo "detected package manager: $PKG_MANAGER"
echo ""

# 3a. install system dependencies
echo "[1/5] installing system dependencies..."

if [ "$PKG_MANAGER" = "apt" ]; then
    $INSTALL_CMD \
        ffmpeg \
        libwebkit2gtk-4.1-dev \
        build-essential \
        curl \
        wget \
        file \
        libssl-dev \
        libayatana-appindicator3-dev \
        librsvg2-dev \
        python3-pip
elif [ "$PKG_MANAGER" = "dnf" ]; then
    $INSTALL_CMD \
        ffmpeg \
        webkit2gtk4.1-devel \
        gcc \
        curl \
        wget \
        file \
        openssl-devel \
        libappindicator-gtk3-devel \
        librsvg2-devel \
        python3-pip
elif [ "$PKG_MANAGER" = "pacman" ]; then
    $INSTALL_CMD \
        ffmpeg \
        webkit2gtk-4.1 \
        base-devel \
        curl \
        wget \
        file \
        openssl \
        libappindicator-gtk3 \
        librsvg \
        python-pip
fi

# 3b. install yt-dlp
echo ""
echo "[2/5] installing yt-dlp..."
pip install --user yt-dlp

# make sure its in path
export PATH="$HOME/.local/bin:$PATH"

# 4a. check for rust, install if missing
echo ""
echo "[3/5] checking rust installation..."

if ! command -v rustc &> /dev/null; then
    echo "rust not found, installing..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    echo "rust already installed: $(rustc --version)"
fi

# 4b. check for node, warn if missing
echo ""
echo "[4/5] checking node installation..."

if ! command -v node &> /dev/null; then
    echo "WARNING: node.js not found"
    echo "install it with nvm or your package manager"
    echo "  curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash"
    echo "  nvm install 18"
    exit 1
else
    echo "node already installed: $(node --version)"
fi

# 5a. install npm dependencies
echo ""
echo "[5/5] installing npm dependencies..."
npm install

# 6a. verify everything is installed
echo ""
echo "======================================"
echo "  Checking installations..."
echo "======================================"

check_cmd() {
    if command -v $1 &> /dev/null; then
        echo "  ✓ $1 installed"
        return 0
    else
        echo "  ✗ $1 NOT FOUND"
        return 1
    fi
}

all_good=true
check_cmd ffmpeg || all_good=false
check_cmd yt-dlp || all_good=false
check_cmd rustc || all_good=false
check_cmd cargo || all_good=false
check_cmd node || all_good=false
check_cmd npm || all_good=false

echo ""

if [ "$all_good" = true ]; then
    echo "======================================"
    echo "  Setup complete!"
    echo "======================================"
    echo ""
    echo "to run in dev mode:"
    echo "  npm run tauri dev"
    echo ""
    echo "to build for production:"
    echo "  npm run tauri build"
    echo ""
else
    echo "======================================"
    echo "  Some dependencies missing!"
    echo "======================================"
    echo "check the errors above and install manually"
fi
