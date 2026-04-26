#!/bin/bash
# Smainer Desktop Icon Generator
# Generates all required Tauri app icons from canonical SVG source

set -e

echo "🎨 Generating Smainer Desktop Icons from canonical source..."

# Check if ImageMagick is installed
if ! command -v convert &> /dev/null; then
    echo "❌ ImageMagick not found. Installing..."
    sudo apt update && sudo apt install -y imagemagick
fi

# Source files
SOURCE_SVG="../assets/branding/smainer-logo-canonical.svg"
ICONS_DIR="./src-tauri/icons"

# Ensure icons directory exists
mkdir -p "$ICONS_DIR"

# Change to desktop directory
cd /home/smainer/Smainer/desktop

# Generate PNG icons in required sizes
echo "📸 Generating PNG icons..."

# Windows & Linux icons
convert -background transparent "$SOURCE_SVG" -resize 32x32 "$ICONS_DIR/32x32.png"
convert -background transparent "$SOURCE_SVG" -resize 128x128 "$ICONS_DIR/128x128.png"
convert -background transparent "$SOURCE_SVG" -resize 256x256 "$ICONS_DIR/128x128@2x.png"
convert -background transparent "$SOURCE_SVG" -resize 512x512 "$ICONS_DIR/icon.png"

# Windows ICO format (multiple sizes embedded)
echo "🪟 Generating Windows ICO..."
convert -background transparent "$SOURCE_SVG" \
    \( -clone 0 -resize 16x16 \) \
    \( -clone 0 -resize 24x24 \) \
    \( -clone 0 -resize 32x32 \) \
    \( -clone 0 -resize 48x48 \) \
    \( -clone 0 -resize 64x64 \) \
    \( -clone 0 -resize 128x128 \) \
    \( -clone 0 -resize 256x256 \) \
    -delete 0 "$ICONS_DIR/icon.ico"

# macOS ICNS format
echo "🍎 Generating macOS ICNS..."
if command -v png2icns &> /dev/null; then
    png2icns "$ICONS_DIR/icon.icns" "$ICONS_DIR/icon.png"
else
    echo "⚠️  png2icns not available - ICNS generation skipped"
    echo "   For macOS builds, install: brew install libicns"
fi

# Windows Store / Microsoft Store logos
echo "🏪 Generating Windows Store assets..."
convert -background transparent "$SOURCE_SVG" -resize 30x30 "$ICONS_DIR/Square30x30Logo.png"
convert -background transparent "$SOURCE_SVG" -resize 44x44 "$ICONS_DIR/Square44x44Logo.png" 
convert -background transparent "$SOURCE_SVG" -resize 71x71 "$ICONS_DIR/Square71x71Logo.png"
convert -background transparent "$SOURCE_SVG" -resize 89x89 "$ICONS_DIR/Square89x89Logo.png"
convert -background transparent "$SOURCE_SVG" -resize 107x107 "$ICONS_DIR/Square107x107Logo.png"
convert -background transparent "$SOURCE_SVG" -resize 142x142 "$ICONS_DIR/Square142x142Logo.png"
convert -background transparent "$SOURCE_SVG" -resize 150x150 "$ICONS_DIR/Square150x150Logo.png"
convert -background transparent "$SOURCE_SVG" -resize 284x284 "$ICONS_DIR/Square284x284Logo.png"
convert -background transparent "$SOURCE_SVG" -resize 310x310 "$ICONS_DIR/Square310x310Logo.png"
convert -background transparent "$SOURCE_SVG" -resize 50x50 "$ICONS_DIR/StoreLogo.png"

echo "✅ Icon generation complete!"
echo ""
echo "📋 Generated files:"
ls -la "$ICONS_DIR"/*.png "$ICONS_DIR"/*.ico "$ICONS_DIR"/*.icns 2>/dev/null || true
echo ""
echo "🔧 Next steps:"
echo "1. Run: npm run build"
echo "2. Test: npm run tauri build"
echo "3. Verify icons appear correctly in installer"