#!/usr/bin/env python3
"""
Regenerate desktop application icons from website source assets.
Fixes packaging defect where all icons were 512x512 regardless of filename.
"""

from PIL import Image
import os

def resize_and_save(source_path, target_path, size):
    """Resize image to specific dimensions and save"""
    print(f"Creating {os.path.basename(target_path)} ({size}x{size})")
    with Image.open(source_path) as img:
        # Convert to RGBA if not already
        if img.mode != 'RGBA':
            img = img.convert('RGBA')
        
        # Resize with high quality resampling
        resized = img.resize((size, size), Image.Resampling.LANCZOS)
        resized.save(target_path, 'PNG', optimize=True)

def create_ico_file(source_path, ico_path, sizes):
    """Create ICO file with multiple resolutions"""
    print(f"Creating {os.path.basename(ico_path)} with sizes: {sizes}")
    with Image.open(source_path) as img:
        if img.mode != 'RGBA':
            img = img.convert('RGBA')
        
        # Generate all required sizes
        icon_images = []
        for size in sizes:
            resized = img.resize((size, size), Image.Resampling.LANCZOS)
            icon_images.append(resized)
        
        # Save as ICO with multiple sizes
        icon_images[0].save(ico_path, format='ICO', sizes=[(s, s) for s in sizes])

def main():
    # Paths
    desktop_dir = "/home/smainer/Smainer/desktop"
    source_icon = "/home/smainer/Smainer/frontend/public/icon-512x512.png"
    icons_dir = os.path.join(desktop_dir, "src-tauri", "icons")
    
    print("Regenerating Tauri desktop icons from website assets...")
    print(f"Source: {source_icon}")
    print(f"Target directory: {icons_dir}")
    
    # Verify source exists
    if not os.path.exists(source_icon):
        print(f"ERROR: Source icon not found at {source_icon}")
        return 1
    
    # Create backup of original icons directory
    backup_dir = os.path.join(desktop_dir, "icons_backup_" + str(int(__import__('time').time())))
    print(f"Creating backup at: {backup_dir}")
    os.system(f"cp -r '{icons_dir}' '{backup_dir}'")
    
    # Generate required PNG sizes for Tauri
    icon_specs = [
        ("32x32.png", 32),
        ("128x128.png", 128), 
        ("128x128@2x.png", 256),  # @2x means double resolution
        ("icon.png", 512),  # Keep full size for tray icon
    ]
    
    for filename, size in icon_specs:
        target_path = os.path.join(icons_dir, filename)
        resize_and_save(source_icon, target_path, size)
    
    # Generate ICO file with multiple resolutions for Windows
    ico_sizes = [16, 20, 24, 32, 40, 48, 64, 96, 128, 256]
    ico_path = os.path.join(icons_dir, "icon.ico")
    create_ico_file(source_icon, ico_path, ico_sizes)
    
    # Also generate Windows Store logo sizes (all currently wrong)
    windows_logos = [
        ("Square30x30Logo.png", 30),
        ("Square44x44Logo.png", 44), 
        ("Square71x71Logo.png", 71),
        ("Square89x89Logo.png", 89),
        ("Square107x107Logo.png", 107),
        ("Square142x142Logo.png", 142),
        ("Square150x150Logo.png", 150),
        ("Square284x284Logo.png", 284),
        ("Square310x310Logo.png", 310),
        ("StoreLogo.png", 50),  # Store logo is typically 50x50
    ]
    
    print("\nGenerating Windows Store logos...")
    for filename, size in windows_logos:
        target_path = os.path.join(icons_dir, filename)
        resize_and_save(source_icon, target_path, size)
    
    print("\nIcon regeneration complete!")
    print("Verification:")
    
    # Verify the generated files
    for filename, expected_size in icon_specs + windows_logos:
        target_path = os.path.join(icons_dir, filename)
        if os.path.exists(target_path):
            with Image.open(target_path) as img:
                actual_size = img.size
                status = "✓" if actual_size == (expected_size, expected_size) else "✗"
                print(f"  {status} {filename}: {actual_size[0]}x{actual_size[1]} (expected {expected_size}x{expected_size})")
    
    # Check ICO file
    ico_path = os.path.join(icons_dir, "icon.ico")
    if os.path.exists(ico_path):
        with Image.open(ico_path) as ico:
            print(f"  ✓ icon.ico: {ico.size[0]}x{ico.size[1]} (ICO format)")
    
    print(f"\nBackup of original icons saved to: {backup_dir}")
    return 0

if __name__ == "__main__":
    exit(main())