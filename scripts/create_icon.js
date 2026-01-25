const fs = require('fs');
const path = require('path');
const iconPath = path.join('ui', 'src-tauri', 'icons', 'icon.ico');

// Ensure dir exists
const dir = path.dirname(iconPath);
if (!fs.existsSync(dir)) {
    fs.mkdirSync(dir, { recursive: true });
}

// 6 Header + 16 Entry + 40 BMPHeader + 4 Pixel + 4 Mask = 70 bytes
const buf = Buffer.alloc(70);
let offset = 0;

// ICO Header
buf.writeUInt16LE(0, offset); offset += 2;
buf.writeUInt16LE(1, offset); offset += 2; // Type 1
buf.writeUInt16LE(1, offset); offset += 2; // Count 1

// Entry
buf.writeUInt8(1, offset); offset += 1; // W
buf.writeUInt8(1, offset); offset += 1; // H
buf.writeUInt8(0, offset); offset += 1; // Colors
buf.writeUInt8(0, offset); offset += 1; // Res
buf.writeUInt16LE(1, offset); offset += 2; // Planes
buf.writeUInt16LE(32, offset); offset += 2; // BPP
buf.writeUInt32LE(48, offset); offset += 4; // Size of data (40 + 4 + 4)
buf.writeUInt32LE(22, offset); offset += 4; // Offset (6+16)

// Bitmap Info
buf.writeUInt32LE(40, offset); offset += 4; // Header Size
buf.writeInt32LE(1, offset); offset += 4; // W
buf.writeInt32LE(2, offset); offset += 4; // H (2 * height for XOR+AND)
buf.writeUInt16LE(1, offset); offset += 2; // Planes
buf.writeUInt16LE(32, offset); offset += 2; // BPP
buf.writeUInt32LE(0, offset); offset += 4; // Compression
buf.writeUInt32LE(0, offset); offset += 4; // SizeImage
buf.writeInt32LE(0, offset); offset += 4; // XPels
buf.writeInt32LE(0, offset); offset += 4; // YPels
buf.writeUInt32LE(0, offset); offset += 4; // ClrUsed
buf.writeUInt32LE(0, offset); offset += 4; // ClrImportant

// Pixel (0,0,0,0) - Transparent
buf.writeUInt32LE(0, offset); offset += 4;

// AND Mask (32 bits = 4 bytes) - All 0 for transparent
buf.writeUInt32LE(0, offset); offset += 4;

fs.writeFileSync(iconPath, buf);
console.log("Valid 1x1 Icon created at " + iconPath);
