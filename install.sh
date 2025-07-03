#!/bin/bash
set -euo pipefail

# SweetMCP One-Line Installer - DOES IT ALL!
# Usage: curl -fsSL https://get.cyrup.ai/sweetmcp.sh | bash
# OR run directly from the project directory: ./install.sh

# Color output
red() { echo -e "\033[0;31m$1\033[0m"; }
green() { echo -e "\033[0;32m$1\033[0m"; }
yellow() { echo -e "\033[0;33m$1\033[0m"; }
blue() { echo -e "\033[0;34m$1\033[0m"; }

# Get sudo access upfront
blue "🍯 SweetMCP Installer - We Do It All!"
blue "This installer needs administrator privileges to:"
echo "  • Install system dependencies"
echo "  • Install the SweetMCP daemon"
echo "  • Configure certificates and services"
echo ""

# Get sudo privileges and keep them alive
sudo -v
# Keep sudo alive in background
while true; do sudo -n true; sleep 60; kill -0 "$$" || exit; done 2>/dev/null &
SUDO_PID=$!

# Cleanup function
cleanup() {
    # Kill the sudo keepalive process
    if [[ -n "${SUDO_PID:-}" ]]; then
        kill $SUDO_PID 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Get config directory
CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
SWEETMCP_HOME="$CONFIG_HOME/sweetmcp"

# Detect OS and package manager
detect_os() {
    if [[ -f /etc/os-release ]]; then
        . /etc/os-release
        OS=$ID
        OS_LIKE="${ID_LIKE:-}"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        OS="macos"
        OS_LIKE=""
    else
        OS="unknown"
        OS_LIKE=""
    fi
}

# Install dependencies based on OS
install_deps() {
    blue "Installing dependencies..."
    
    if [[ "$OS" == "ubuntu" ]] || [[ "$OS" == "debian" ]] || [[ "$OS_LIKE" == *"debian"* ]]; then
        # Update package list
        sudo apt-get update -qq
        # Install all build dependencies
        sudo apt-get install -y git curl build-essential pkg-config libssl-dev
    elif [[ "$OS" == "fedora" ]] || [[ "$OS" == "rhel" ]] || [[ "$OS" == "centos" ]]; then
        sudo dnf install -y git curl gcc gcc-c++ make pkgconfig openssl-devel
    elif [[ "$OS" == "arch" ]] || [[ "$OS" == "manjaro" ]]; then
        sudo pacman -S --needed --noconfirm git curl base-devel openssl
    elif [[ "$OS" == "opensuse"* ]]; then
        sudo zypper install -y git curl gcc gcc-c++ make pkg-config libopenssl-devel
    elif [[ "$OS" == "alpine" ]]; then
        sudo apk add --no-cache git curl build-base pkgconfig openssl-dev
    elif [[ "$OS" == "macos" ]]; then
        # Install Xcode Command Line Tools if needed
        if ! xcode-select -p >/dev/null 2>&1; then
            yellow "Installing Xcode Command Line Tools..."
            xcode-select --install 2>/dev/null || true
            # Wait for installation
            until xcode-select -p >/dev/null 2>&1; do
                sleep 5
            done
        fi
        # Install Homebrew if needed
        if ! command -v brew >/dev/null 2>&1; then
            yellow "Installing Homebrew..."
            /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
            if [[ -f /opt/homebrew/bin/brew ]]; then
                eval "$(/opt/homebrew/bin/brew shellenv)"
            fi
        fi
        # Install dependencies via Homebrew
        brew install git curl openssl pkg-config
    else
        yellow "Unknown OS, attempting generic install..."
        # Try to find and use available package manager
        if command -v apt-get >/dev/null 2>&1; then
            sudo apt-get update -qq && sudo apt-get install -y git curl build-essential pkg-config libssl-dev
        elif command -v yum >/dev/null 2>&1; then
            sudo yum install -y git curl gcc gcc-c++ make pkgconfig openssl-devel
        elif command -v apk >/dev/null 2>&1; then
            sudo apk add --no-cache git curl build-base pkgconfig openssl-dev
        else
            red "Could not detect package manager. Please install: git, curl, gcc, make, pkg-config, openssl-dev"
            exit 1
        fi
    fi
    
    green "Dependencies installed!"
}

# Main installation
detect_os
install_deps

# Install Rust if needed
if ! command -v rustc >/dev/null 2>&1; then
    yellow "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    source "$HOME/.cargo/env"
fi

# Check if we're running from inside the project directory
if [[ -f "Cargo.toml" ]] && grep -q "sweetmcp" "Cargo.toml" 2>/dev/null; then
    blue "Running from project directory..."
    PROJECT_DIR="$(pwd)"
else
    # We're running from curl, need to clone
    blue "Creating SweetMCP home directory..."
    mkdir -p "$SWEETMCP_HOME"
    cd "$SWEETMCP_HOME"
    
    # Clone repository (remove old clone if exists)
    blue "Cloning SweetMCP..."
    if [[ -d "sweetmcp" ]]; then
        rm -rf sweetmcp
    fi
    
    if ! git clone https://github.com/cyrup-ai/sweetmcp.git 2>/dev/null; then
        red "Failed to clone repository"
        exit 1
    fi
    cd sweetmcp
    PROJECT_DIR="$(pwd)"
fi

# Build
blue "Building SweetMCP (this may take a few minutes)..."
if ! cargo build --release --package sweetmcp-daemon; then
    red "Build failed"
    exit 1
fi

# Install with the sudo privileges we already have
blue "Installing SweetMCP daemon..."
if ! sudo ./target/release/sweetmcp-daemon install; then
    red "Installation failed"
    exit 1
fi

# Success!
green "
╔══════════════════════════════════════════════════════════════╗
║           SweetMCP Installation Completed! 🍯                ║
╚══════════════════════════════════════════════════════════════╝

Project location: $PROJECT_DIR

Available at:
  • https://sweetmcp.cyrup.dev:8443
  • https://sweetmcp.cyrup.ai:8443
  • https://sweetmcp.cyrup.cloud:8443
  • https://sweetmcp.cyrup.pro:8443

Go have fun!
"