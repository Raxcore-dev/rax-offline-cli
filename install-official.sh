#!/bin/bash
# Official Rax CLI Installer
# This script installs Rax CLI using apt or from source

set -e

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║              🚀 Rax CLI Installer                        ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Check if running as root
if [ "$EUID" -eq 0 ]; then
    echo "❌ Please don't run this script as root"
    echo "   The script will use sudo when needed"
    exit 1
fi

# Detect OS
if [ -f /etc/debian_version ]; then
    OS="debian"
    echo "✅ Detected Debian/Ubuntu system"
elif [ -f /etc/redhat-release ]; then
    OS="rhel"
    echo "⚠️  Detected RHEL/CentOS system (limited support)"
else
    OS="unknown"
    echo "⚠️  Unknown OS, will install from source"
fi

# Function to install from apt
install_from_apt() {
    echo ""
    echo "📦 Installing Rax CLI via apt..."
    echo ""
    
    # Add repository key
    echo "🔑 Adding repository key..."
    curl -fsSL https://raxcore-dev.github.io/rax-offline-cli/repo.key | \
        sudo gpg --dearmor -o /usr/share/keyrings/rax-archive-keyring.gpg
    
    # Add repository
    echo "📂 Adding apt repository..."
    echo "deb [signed-by=/usr/share/keyrings/rax-archive-keyring.gpg] https://raxcore-dev.github.io/rax-offline-cli/ /" | \
        sudo tee /etc/apt/sources.list.d/rax.list
    
    # Update and install
    echo "🔄 Updating package list..."
    sudo apt update
    
    echo "📦 Installing Rax CLI..."
    sudo apt install -y rax
    
    echo ""
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║              ✅ Installation Complete!                   ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo ""
    echo "🎉 Rax CLI has been installed!"
    echo ""
    echo "Run 'rax' to start the setup and begin chatting!"
    echo ""
}

# Function to install from source
install_from_source() {
    echo ""
    echo "🔨 Installing Rax CLI from source..."
    echo ""
    
    # Check for Rust
    if ! command -v cargo &> /dev/null; then
        echo "⚠️  Rust not found. Installing rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
        source $HOME/.cargo/env
    fi
    
    # Build
    echo "🔨 Building Rax CLI..."
    cargo build --release
    
    # Install
    echo "📦 Installing binary..."
    mkdir -p $HOME/.local/bin
    cp target/release/rax $HOME/.local/bin/
    chmod +x $HOME/.local/bin/rax
    
    # Add to PATH if needed
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        echo ""
        echo "⚠️  $HOME/.local/bin is not in your PATH"
        echo ""
        echo "Add this to your ~/.bashrc or ~/.zshrc:"
        echo ""
        echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
        echo ""
        
        # Offer to add automatically
        read -p "Add it automatically? [Y/n] " response
        if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]] || [[ -z "$response" ]]; then
            echo "" >> ~/.bashrc
            echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> ~/.bashrc
            echo "✅ Added to ~/.bashrc"
            
            if [ -f ~/.zshrc ]; then
                echo "" >> ~/.zshrc
                echo "export PATH=\"\$HOME/.local/bin:\$PATH\"" >> ~/.zshrc
                echo "✅ Added to ~/.zshrc"
            fi
            
            echo ""
            echo "Run 'source ~/.bashrc' or restart your terminal"
        fi
    fi
    
    echo ""
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║              ✅ Installation Complete!                   ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo ""
    echo "🎉 Rax CLI has been installed!"
    echo ""
    echo "Run 'rax' to start the setup and begin chatting!"
    echo ""
}

# Function to install from .deb file
install_from_deb() {
    DEB_FILE="$1"
    
    if [ ! -f "$DEB_FILE" ]; then
        echo "❌ File not found: $DEB_FILE"
        exit 1
    fi
    
    echo ""
    echo "📦 Installing from local package..."
    echo ""
    
    sudo apt install -y ./ "$DEB_FILE"
    
    echo ""
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║              ✅ Installation Complete!                   ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo ""
    echo "🎉 Rax CLI has been installed!"
    echo ""
    echo "Run 'rax' to start the setup and begin chatting!"
    echo ""
}

# Main installation logic
if [ "$1" = "--local" ] && [ -n "$2" ]; then
    install_from_deb "$2"
    exit 0
fi

if [ "$OS" = "debian" ]; then
    echo ""
    echo "Installation methods:"
    echo "  1) apt repository (recommended)"
    echo "  2) local .deb file"
    echo "  3) build from source"
    echo ""
    read -p "Choose installation method [1-3]: " method
    
    case $method in
        1)
            install_from_apt
            ;;
        2)
            read -p "Enter path to .deb file: " deb_path
            install_from_deb "$deb_path"
            ;;
        3|*)
            install_from_source
            ;;
    esac
else
    echo ""
    echo "ℹ️  apt installation not available for your OS"
    echo "   Will install from source"
    echo ""
    read -p "Continue with source installation? [Y/n] " response
    
    if [[ "$response" =~ ^([nN][oO]|[nN])$ ]]; then
        echo "Installation cancelled."
        exit 0
    fi
    
    install_from_source
fi
