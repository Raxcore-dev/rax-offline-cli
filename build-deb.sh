#!/bin/bash
# Build script for Rax CLI Debian package

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║           📦 Building Rax CLI Package                    ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Clean previous builds
echo "🧹 Cleaning previous builds..."
rm -rf debian/rax
rm -rf debian/.debhelper
rm -rf debian/rax.debhelper.log
rm -rf debian/files
rm -rf debian/substvars
rm -rf debian/DEBIAN

# Build release binary
echo "🔨 Building release binary..."
cargo build --release

# Create DEBIAN directory
mkdir -p debian/DEBIAN
mkdir -p debian/usr/bin

# Copy binary
echo "📦 Copying binary..."
cp target/release/rax debian/usr/bin/rax
chmod 755 debian/usr/bin/rax

# Create control file
cat > debian/DEBIAN/control << 'EOF'
Package: rax
Version: 0.1.0
Section: utils
Priority: optional
Architecture: amd64
Maintainer: RaxCore <rax@localhost>
Depends: libc6 (>= 2.31)
Installed-Size: 9000
Description: Next-Gen Offline AI Assistant
 Rax is a fast, secure, and context-aware offline AI assistant powered by
 Llama 3.2. It runs entirely on your CPU with no data leaving your machine.
 .
 Features:
  - Claude Code-style interactive interface
  - Persistent chat history with auto-save
  - Project context awareness
  - Beautiful TUI mode
  - CPU-optimized (Llama-3.2-1B, ~650MB)
Homepage: https://github.com/raxcore/rax
EOF

# Create postinst script
cat > debian/DEBIAN/postinst << 'EOF'
#!/bin/bash
set -e

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║           🚀 Rax CLI - Installation Complete             ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "✅ Rax CLI has been installed!"
echo ""
echo "📦 The AI model (~650 MB) will be downloaded on first run."
echo "   This is a one-time download."
echo ""
echo "Run 'rax' to start the setup and begin chatting!"
echo ""
exit 0
EOF
chmod +x debian/DEBIAN/postinst

# Create postrm script
cat > debian/DEBIAN/postrm << 'EOF'
#!/bin/bash
set -e

if [ "$1" = "remove" ] || [ "$1" = "purge" ]; then
    echo ""
    echo "🗑️  Rax CLI removed"
    
    if [ "$1" = "purge" ]; then
        if [ -d "$HOME/.local/share/RaxCli" ]; then
            echo "   Removing user data..."
            rm -rf "$HOME/.local/share/RaxCli"
        fi
    else
        echo "   (User data kept in ~/.local/share/RaxCli)"
        echo "   Run 'sudo apt purge rax' to remove data too"
    fi
fi

exit 0
EOF
chmod +x debian/DEBIAN/postrm

# Get version
VERSION=$(grep "^version" Cargo.toml | head -1 | sed 's/.*= *"\([^"]*\)".*/\1/')
echo "📦 Building rax_${VERSION}_amd64.deb..."

# Build the package
dpkg-deb --build debian rax_${VERSION}_amd64.deb

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║              ✅ Package built successfully!              ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""
echo "Package: ./rax_${VERSION}_amd64.deb"
echo ""
echo "To install:"
echo "  sudo apt install ./rax_${VERSION}_amd64.deb"
echo ""
echo "Or:"
echo "  sudo dpkg -i rax_${VERSION}_amd64.deb"
echo "  sudo apt-get install -f  # Fix dependencies if needed"
echo ""
