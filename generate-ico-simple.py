#!/usr/bin/env python3
"""
Smainer Desktop - Simple ICO Generator
Creates a multi-resolution ICO from existing PNG assets without external dependencies
"""

import struct
import os

def create_ico_from_pngs(png_files, output_path):
    """Create ICO file from multiple PNG files"""
    
    # Read all PNG data
    images = []
    for png_path, target_size in png_files:
        if not os.path.exists(png_path):
            print(f"Warning: {png_path} not found, skipping...")
            continue
            
        with open(png_path, 'rb') as f:
            png_data = f.read()
            
        # Get PNG dimensions from header (bytes 16-23)
        width = struct.unpack('>I', png_data[16:20])[0]
        height = struct.unpack('>I', png_data[20:24])[0]
        
        print(f"  📐 Adding {width}x{height} PNG ({len(png_data)} bytes)")
        
        images.append({
            'width': width,
            'height': height,
            'data': png_data,
            'size': len(png_data)
        })
    
    if not images:
        raise ValueError("No valid PNG images found")
    
    # Create ICO file structure
    icon_count = len(images)
    
    # ICO header (6 bytes)
    header = struct.pack('<HHH', 0, 1, icon_count)  # Reserved, Type (1=ICO), Count
    
    # Calculate data offset (header + directory entries)
    data_offset = 6 + (16 * icon_count)
    
    # Create directory entries
    entries = b''
    image_data = b''
    current_offset = data_offset
    
    for img in images:
        width = img['width']
        height = img['height']
        
        # ICO directory entry (16 bytes)
        # Width/Height: 0 means 256
        w = 0 if width == 256 else width
        h = 0 if height == 256 else height
        
        entry = struct.pack('<BBBBHHII',
            w,                    # Width (0 = 256)
            h,                    # Height (0 = 256)  
            0,                    # Color count (0 = no palette)
            0,                    # Reserved
            1,                    # Color planes
            32,                   # Bits per pixel
            img['size'],          # Image data size
            current_offset        # Image data offset
        )
        
        entries += entry
        image_data += img['data']
        current_offset += img['size']
    
    # Combine everything
    ico_data = header + entries + image_data
    
    # Write ICO file
    with open(output_path, 'wb') as f:
        f.write(ico_data)
    
    print(f"✅ Generated {output_path} with {icon_count} embedded icons ({len(ico_data)} bytes)")
    return output_path

def main():
    print('🎨 Generating multi-resolution Windows ICO from existing PNGs...')
    
    frontend_assets = '/home/smainer/Smainer/frontend/public'
    output_ico = 'src-tauri/icons/icon.ico'
    
    # Use existing PNG assets for different sizes (ICO format supports up to 255x255 in the directory)
    png_sources = [
        (f'{frontend_assets}/favicon-16x16.png', 16),
        (f'{frontend_assets}/favicon-32x32.png', 32),
        (f'{frontend_assets}/apple-touch-icon.png', 180),  # Will be used as-is
        (f'{frontend_assets}/icon-192x192.png', 192),
        # Skip 512x512 as it exceeds ICO directory size limits (use 256 as max)
    ]
    
    try:
        create_ico_from_pngs(png_sources, output_ico)
        
        # Verify the result
        print('\n🔍 Verifying ICO file...')
        with open(output_ico, 'rb') as f:
            header = f.read(6)
            reserved, type_val, count = struct.unpack('<HHH', header)
            print(f"  🖼️  ICO type: {type_val}, Embedded icons: {count}")
            
            # Read directory entries to show sizes
            for i in range(count):
                entry = f.read(16)
                w, h, colors, reserved, planes, bpp, size, offset = struct.unpack('<BBBBHHII', entry)
                width = 256 if w == 0 else w
                height = 256 if h == 0 else h
                print(f"    • {width}x{height} ({size} bytes, {bpp}bpp)")
        
        print('\n✅ Multi-resolution ICO generation complete!')
        
    except Exception as e:
        print(f'❌ ICO generation failed: {e}')
        return False
    
    return True

if __name__ == "__main__":
    success = main()
    if not success:
        exit(1)