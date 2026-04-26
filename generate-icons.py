#!/usr/bin/env python3
"""
Smainer Desktop Icon Generator (Python version)
Generates all required Tauri app icons from canonical SVG source
"""

import os
import sys
import subprocess
import tempfile
import base64

print("🎨 Generating Smainer Desktop Icons from canonical source (Python)...")

# SVG content from canonical logo
svg_content = '''<svg width="512" height="512" viewBox="0 0 512 512" xmlns="http://www.w3.org/2000/svg">
  <!-- Smainer Canonical Logo - Distributed Compute Blocks -->
  <!-- Background container - rounded for modern feel -->
  <rect width="512" height="512" rx="96" fill="#09090B"/>
  
  <!-- Distributed compute blocks arranged in S formation -->
  <!-- Top row -->
  <rect x="200" y="104" width="112" height="48" rx="8" fill="#FFFFFF"/>
  <rect x="328" y="104" width="48" height="48" rx="8" fill="#3B82F6"/>
  
  <!-- Second row -->
  <rect x="136" y="168" width="48" height="48" rx="8" fill="#FFFFFF"/>
  
  <!-- Third row -->
  <rect x="200" y="232" width="112" height="48" rx="8" fill="#FFFFFF"/>
  
  <!-- Fourth row -->
  <rect x="328" y="296" width="48" height="48" rx="8" fill="#FFFFFF"/>
  
  <!-- Bottom row -->
  <rect x="136" y="360" width="48" height="48" rx="8" fill="#3B82F6"/>
  <rect x="200" y="360" width="112" height="48" rx="8" fill="#FFFFFF"/>
</svg>'''

# Icon sizes needed for Tauri
icon_sizes = [
    ('32x32.png', 32),
    ('128x128.png', 128),
    ('128x128@2x.png', 256),
    ('icon.png', 512),
    ('Square30x30Logo.png', 30),
    ('Square44x44Logo.png', 44),
    ('Square71x71Logo.png', 71),
    ('Square89x89Logo.png', 89),
    ('Square107x107Logo.png', 107),
    ('Square142x142Logo.png', 142),
    ('Square150x150Logo.png', 150),
    ('Square284x284Logo.png', 284),
    ('Square310x310Logo.png', 310),
    ('StoreLogo.png', 50),
]

def check_tool(tool_name):
    """Check if a tool is available"""
    try:
        subprocess.run([tool_name, '--version'], capture_output=True, check=False)
        return True
    except FileNotFoundError:
        return False

def generate_icons():
    """Generate icons using available tools"""
    icons_dir = 'src-tauri/icons'
    
    # Create temporary SVG file
    with tempfile.NamedTemporaryFile(mode='w', suffix='.svg', delete=False) as f:
        f.write(svg_content)
        temp_svg = f.name
    
    try:
        # Try different tools in order of preference
        if check_tool('rsvg-convert'):
            print("✅ Using rsvg-convert...")
            for filename, size in icon_sizes:
                cmd = ['rsvg-convert', '-w', str(size), '-h', str(size), temp_svg, '-o', os.path.join(icons_dir, filename)]
                subprocess.run(cmd, check=True)
                print(f"✅ Generated {filename}")
                
        elif check_tool('inkscape'):
            print("✅ Using Inkscape...")
            for filename, size in icon_sizes:
                cmd = ['inkscape', temp_svg, '--export-type=png', f'--export-filename={os.path.join(icons_dir, filename)}', f'--export-width={size}', f'--export-height={size}']
                subprocess.run(cmd, check=True)
                print(f"✅ Generated {filename}")
                
        elif check_tool('convert'):
            print("✅ Using ImageMagick...")
            for filename, size in icon_sizes:
                cmd = ['convert', '-background', 'transparent', temp_svg, '-resize', f'{size}x{size}', os.path.join(icons_dir, filename)]
                subprocess.run(cmd, check=True)
                print(f"✅ Generated {filename}")
        else:
            print("❌ No suitable SVG conversion tools found.")
            print("Please install one of: rsvg-convert, inkscape, or imagemagick")
            print("Ubuntu/Debian: apt install librsvg2-bin")
            print("Ubuntu/Debian: apt install inkscape")  
            print("Ubuntu/Debian: apt install imagemagick")
            return False
            
    except subprocess.CalledProcessError as e:
        print(f"❌ Error generating icons: {e}")
        return False
    finally:
        os.unlink(temp_svg)
    
    print("\n✅ Icon generation complete!")
    return True

if __name__ == "__main__":
    if not generate_icons():
        sys.exit(1)