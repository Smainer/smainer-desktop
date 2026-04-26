#!/usr/bin/env node
/**
 * Smainer Desktop Icon Generator (Node.js version)
 * Generates all required Tauri app icons from canonical SVG source
 */

import fs from 'fs';
import path from 'path';
import { exec } from 'child_process';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

console.log('🎨 Generating Smainer Desktop Icons from canonical source (Node.js)...');

// Source files
const sourceSvg = path.join(__dirname, 'assets', 'branding', 'smainer-logo-canonical.svg');
const iconsDir = path.join(__dirname, 'src-tauri', 'icons');

// Check if source SVG exists
if (!fs.existsSync(sourceSvg)) {
    console.error(`❌ Source SVG not found at: ${sourceSvg}`);
    process.exit(1);
}

console.log(`📍 Source: ${sourceSvg}`);
console.log(`📂 Icons directory: ${iconsDir}`);

// Read the SVG content
const svgContent = fs.readFileSync(sourceSvg, 'utf8');

console.log('✅ SVG loaded successfully!');
console.log('📋 SVG content preview:');
console.log(svgContent.substring(0, 200) + '...');

// Note: For actual PNG conversion, we would need a library like puppeteer, sharp, or canvas
// For now, let's create a script that uses system tools if available

console.log('\n🔧 To complete icon generation, run one of these commands:');
console.log('1. Install ImageMagick: sudo apt install imagemagick');
console.log('2. Install Sharp: npm install --save-dev sharp');  
console.log('3. Use online SVG to PNG converter');

// Try to find available conversion tools

exec('which convert', (error, stdout, stderr) => {
    if (error) {
        console.log('❌ ImageMagick convert not found');
        
        // Try rsvg-convert
        exec('which rsvg-convert', (error2, stdout2, stderr2) => {
            if (error2) {
                console.log('❌ rsvg-convert not found');
                console.log('\n⚠️  No SVG conversion tools available.');
                console.log('Installing ImageMagick or librsvg2-bin...');
                
                exec('sudo apt install -y librsvg2-bin', (error3, stdout3, stderr3) => {
                    if (error3) {
                        console.error('❌ Failed to install librsvg2-bin');
                        console.log('\nManual icon generation required:');
                        console.log('1. Open assets/branding/smainer-logo-canonical.svg in a graphics editor');
                        console.log('2. Export as PNG in these sizes: 32x32, 128x128, 256x256, 512x512');
                        console.log('3. Save to src-tauri/icons/ with appropriate filenames');
                        process.exit(1);
                    } else {
                        console.log('✅ librsvg2-bin installed! Generating icons...');
                        generateIcons();
                    }
                });
            } else {
                console.log('✅ rsvg-convert found! Generating icons...');
                generateIcons();
            }
        });
    } else {
        console.log('✅ ImageMagick convert found! Generating icons...');
        generateIconsWithImageMagick();
    }
});

function generateIcons() {
    const iconSizes = [
        { name: '32x32.png', size: '32x32' },
        { name: '128x128.png', size: '128x128' },
        { name: '128x128@2x.png', size: '256x256' },
        { name: 'icon.png', size: '512x512' },
        
        // Windows Store assets
        { name: 'Square30x30Logo.png', size: '30x30' },
        { name: 'Square44x44Logo.png', size: '44x44' },
        { name: 'Square71x71Logo.png', size: '71x71' },
        { name: 'Square89x89Logo.png', size: '89x89' },
        { name: 'Square107x107Logo.png', size: '107x107' },
        { name: 'Square142x142Logo.png', size: '142x142' },
        { name: 'Square150x150Logo.png', size: '150x150' },
        { name: 'Square284x284Logo.png', size: '284x284' },
        { name: 'Square310x310Logo.png', size: '310x310' },
        { name: 'StoreLogo.png', size: '50x50' }
    ];
    
    iconSizes.forEach(icon => {
        const outputPath = path.join(iconsDir, icon.name);
        const command = `rsvg-convert -w ${icon.size.split('x')[0]} -h ${icon.size.split('x')[1]} "${sourceSvg}" -o "${outputPath}"`;
        
        exec(command, (error, stdout, stderr) => {
            if (error) {
                console.error(`❌ Failed to generate ${icon.name}: ${error.message}`);
            } else {
                console.log(`✅ Generated ${icon.name}`);
            }
        });
    });
}

function generateIconsWithImageMagick() {
    // Use the original shell script logic with ImageMagick
    exec('./generate-icons.sh', (error, stdout, stderr) => {
        if (error) {
            console.error(`❌ Icon generation failed: ${error.message}`);
            console.error(stderr);
        } else {
            console.log('✅ Icons generated successfully with ImageMagick!');
            console.log(stdout);
        }
    });
}