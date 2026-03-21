#!/bin/bash
# Script to update the apt repository with new releases
# This should be run after creating a new GitHub release

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_DIR="$SCRIPT_DIR/repo"

echo ""
echo "╔══════════════════════════════════════════════════════════╗"
echo "║           📦 Rax CLI - Update apt Repository            ║"
echo "╚══════════════════════════════════════════════════════════╝"
echo ""

# Check if deb file exists
DEB_FILE=$(ls -t "$SCRIPT_DIR"/rax_*.deb 2>/dev/null | head -1)

if [ -z "$DEB_FILE" ]; then
    echo "❌ No .deb package found. Run ./build-deb.sh first."
    exit 1
fi

echo "📦 Found package: $DEB_FILE"

# Copy to repo
cp "$DEB_FILE" "$REPO_DIR/pool/main/r/rax/"

# Update the package pool
echo "📂 Updating package pool..."

# Generate Packages.gz
cd "$REPO_DIR"
apt-ftparchive packages pool/main/r/rax/ > Packages
gzip -c Packages > Packages.gz

# Generate Release file
apt-ftparchive release . > Release
echo "SignWith: yes" >> Release

echo ""
echo "✅ Repository updated!"
echo ""
echo "To deploy to GitHub Pages:"
echo "  1. Commit the repo/ directory"
echo "  2. Push to the gh-pages branch"
echo "  3. Enable GitHub Pages in repository settings"
echo ""
echo "Users can then install with:"
echo "  curl -fsSL https://rax-cli.github.io/repo.key | sudo gpg --dearmor -o /usr/share/keyrings/rax-archive-keyring.gpg"
echo "  echo 'deb [signed-by=/usr/share/keyrings/rax-archive-keyring.gpg] https://rax-cli.github.io/ /' | sudo tee /etc/apt/sources.list.d/rax.list"
echo "  sudo apt update && sudo apt install rax"
echo ""
