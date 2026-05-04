# Smainer Desktop Linux Installation Package

## Package Information
- **Filename**: smainer-desktop-linux-20260504.tar.gz
- **Size**: 33 MB
- **SHA256**: 17bb6d55404b118e01c74c7434a13dd70b62d4179469f2d256b0f6505e4a21f5
- **Platform**: Linux x86_64
- **Build Date**: May 4, 2026

## Contents
- Smainer Desktop Application Wrapper (`smainer-desktop`)
- Smainer Provider Binary (`smainer-provider`)
- Web Interface Assets (`web-dist/`)
- Installation Script (`install.sh`)

## Quick Installation

### Option 1: System Installation (Recommended)
```bash
# Extract the package
tar -xzf smainer-desktop-linux-20260504.tar.gz
cd package-linux/

# Install system-wide (requires sudo)
sudo ./install.sh
```

### Option 2: Local Installation (No sudo required)
```bash
# Extract the package
tar -xzf smainer-desktop-linux-20260504.tar.gz
cd package-linux/

# Run directly from package directory
./smainer-desktop
```

## Usage

### After System Installation
```bash
# Launch from terminal
smainer-desktop

# Or find "Smainer Desktop" in your Applications menu
```

### Direct Run (Local Installation)
```bash
cd package-linux/
./smainer-desktop
```

## What It Does
1. Starts the Smainer Provider daemon in the background
2. Launches a local web server for the desktop interface
3. Opens your default browser to http://localhost:8080
4. Provides a complete node management interface

## System Requirements
- Linux x86_64 system
- Python 3 (for local web server)
- Web browser (Firefox, Chrome, etc.)
- Network connectivity for provider operations

## Stopping the Application
Press `Ctrl+C` in the terminal where smainer-desktop is running, or close the terminal window.

## Files Installed (System Installation)
- `/opt/smainer-desktop/` - Application files
- `/usr/local/bin/smainer-desktop` - Binary symlink
- `/usr/share/applications/smainer.desktop` - Desktop entry

## Troubleshooting

### Provider Binary Not Found
Ensure the smainer-provider binary has execute permissions:
```bash
chmod +x smainer-provider
```

### Web Interface Won't Load
Check that port 8080 is available:
```bash
sudo lsof -i :8080
```

### Permission Issues
For system installation, ensure you run install.sh with sudo privileges.

## Support
For issues and support, visit the Smainer documentation or contact support.