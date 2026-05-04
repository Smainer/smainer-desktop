#!/bin/bash
# Smainer Desktop Linux Installer Script
# Generated from: Tauri Desktop Build Process
# Date: $(date)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="/opt/smainer-desktop"
BIN_DIR="/usr/local/bin"
DESKTOP_FILE="/usr/share/applications/smainer.desktop"

echo "🚀 Installing Smainer Desktop Application"
echo "==========================================="

# Check if running as root
if [[ $EUID -ne 0 ]]; then
   echo "❌ This script must be run as root (use sudo)"
   exit 1
fi

# Create install directory
mkdir -p "$INSTALL_DIR"
mkdir -p "$(dirname "$DESKTOP_FILE")"

# Copy application files
echo "📋 Installing application files..."
cp "$SCRIPT_DIR/smainer-desktop" "$INSTALL_DIR/"
cp "$SCRIPT_DIR/smainer-provider" "$INSTALL_DIR/"
cp -r "$SCRIPT_DIR/web-dist" "$INSTALL_DIR/"

# Create symlink for easy access
ln -sf "$INSTALL_DIR/smainer-desktop" "$BIN_DIR/smainer-desktop"

# Set permissions
chmod +x "$INSTALL_DIR/smainer-desktop"
chmod +x "$INSTALL_DIR/smainer-provider"

# Create desktop file
cat > "$DESKTOP_FILE" << 'EOF'
[Desktop Entry]
Version=1.0
Type=Application
Name=Smainer Desktop
Comment=Smainer Desktop Provider Node Management
Exec=/usr/local/bin/smainer-desktop
Icon=smainer
Terminal=false
Categories=Network;System;Utility;
Keywords=blockchain;provider;node;smainer;
EOF

echo "✅ Smainer Desktop installed successfully!"
echo ""
echo "Usage:"
echo "  - Run from terminal: smainer-desktop"
echo "  - Launch from Applications menu: Smainer Desktop"
echo ""
echo "Files installed:"
echo "  - Application: $INSTALL_DIR/"
echo "  - Binary symlink: $BIN_DIR/smainer-desktop"
echo "  - Desktop entry: $DESKTOP_FILE"