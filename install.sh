#!/bin/bash
# Rax CLI Installation Script
# This script installs rax to your system and optionally adds shell aliases

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="$HOME/.local/bin"
RAX_BINARY="$SCRIPT_DIR/target/release/rax"

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║              🚀 Rax CLI Installer                        ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Check if binary exists
if [ ! -f "$RAX_BINARY" ]; then
    echo "❌ Building release binary first..."
    cd "$SCRIPT_DIR" && cargo build --release
fi

# Create install directory
mkdir -p "$INSTALL_DIR"

# Install binary
echo "📦 Installing rax to $INSTALL_DIR..."
cp "$RAX_BINARY" "$INSTALL_DIR/rax"
chmod +x "$INSTALL_DIR/rax"

echo ""
echo "✅ Rax installed successfully!"
echo ""

# Check if PATH includes ~/.local/bin
if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
    echo "⚠️  $HOME/.local/bin is not in your PATH"
    echo ""
    echo "Add this to your ~/.bashrc or ~/.zshrc:"
    echo ""
    echo "    export PATH=\"\$HOME/.local/bin:\$PATH\""
    echo ""
fi

# Offer to add shell alias
echo "Would you like to add a shell alias for quicker access? [Y/n]"
read -r response

if [[ "$response" =~ ^([yY][eE][sS]|[yY])$ ]] || [[ -z "$response" ]]; then
    # Detect shell
    SHELL_RC=""
    if [ -n "$ZSH_VERSION" ] || [ -f "$HOME/.zshrc" ]; then
        SHELL_RC="$HOME/.zshrc"
    elif [ -n "$BASH_VERSION" ] || [ -f "$HOME/.bashrc" ]; then
        SHELL_RC="$HOME/.bashrc"
    fi

    if [ -n "$SHELL_RC" ]; then
        # Check if alias already exists
        if ! grep -q "alias rax=" "$SHELL_RC" 2>/dev/null; then
            echo "" >> "$SHELL_RC"
            echo "# Rax CLI alias" >> "$SHELL_RC"
            echo "alias rax='$INSTALL_DIR/rax'" >> "$SHELL_RC"
            echo "✅ Added alias to $SHELL_RC"
            echo ""
            echo "Run 'source $SHELL_RC' or restart your terminal to use 'rax' anywhere!"
        else
            echo "ℹ️  Alias already exists in $SHELL_RC"
        fi
    else
        echo "Could not detect shell config file."
        echo "Manually add this to your shell config:"
        echo "    alias rax='$INSTALL_DIR/rax'"
    fi
fi

echo ""
echo "═══════════════════════════════════════════════════════════"
echo ""
echo "🎉 You're all set! Run 'rax' to start chatting!"
echo ""
