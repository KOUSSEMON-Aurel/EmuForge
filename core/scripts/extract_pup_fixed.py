#!/usr/bin/env python3
"""
Fixed PS3UPDAT.PUP extractor based on real file format analysis
"""

import struct
import sys
import os
import tarfile
from pathlib import Path

def extract_pup(pup_path, output_dir):
    """Extract PS3UPDAT.PUP file with correct format"""
    
    print(f"📦 Extracting {pup_path} to {output_dir}...")
    
    with open(pup_path, 'rb') as f:
        # Read header (0x20 bytes)
        header = f.read(0x20)
        
        # Parse header (big-endian)
        magic = header[0:5]
        file_count = struct.unpack('>Q', header[0x18:0x20])[0]
        
        print(f"  Magic: {magic}")
        print(f"  File count: {file_count}")
        
        os.makedirs(output_dir, exist_ok=True)
        
        # Read file table (starts at 0x20)
        for i in range(file_count):
            # Each entry is 0x20 bytes
            entry_offset = 0x20 + (i * 0x20)
            f.seek(entry_offset)
            entry_header = f.read(0x20)
            
            # Parse entry (all big-endian)
            entry_id = struct.unpack('>Q', entry_header[0x00:0x08])[0]
            data_offset = struct.unpack('>Q', entry_header[0x08:0x10])[0]
            data_size = struct.unpack('>Q', entry_header[0x10:0x18])[0]
            
            # Map entry ID to filename
            filename = get_filename(entry_id)
            
            print(f"  Entry {i}: {filename} (ID: 0x{entry_id:x}, offset: 0x{data_offset:x}, size: {data_size})")
            
            # Read data
            f.seek(data_offset)
            data = f.read(data_size)
            
            # Write file
            output_path = os.path.join(output_dir, filename)
            with open(output_path, 'wb') as out:
                out.write(data)
            
            # Extract TAR if it's update_files.tar
            if filename == 'update_files.tar':
                print(f"  📦 Extracting {filename}...")
                try:
                    with tarfile.open(output_path, 'r') as tar:
                        tar.extractall(output_dir)
                    print(f"  ✅ Extracted {filename}")
                except Exception as e:
                    print(f"  ⚠️  Failed to extract TAR: {e}")
    
    print("✅ PUP extraction complete")
    
    # Find dev_flash
    dev_flash = find_dev_flash(output_dir)
    if dev_flash:
        print(f"✅ Found dev_flash at: {dev_flash}")
        return dev_flash
    else:
        print("❌ dev_flash not found")
        return None

def get_filename(entry_id):
    """Map entry ID to filename"""
    mapping = {
        0x100: "version.txt",
        0x101: "license.xml",
        0x102: "promo_flags.txt",
        0x103: "update_flags.txt",
        0x200: "ps3swu.self",
        0x201: "vsh.tar",
        0x202: "dots.txt",
        0x300: "update_files.tar",
        0x301: "spkg_hdr.tar",
        0x400: "ps3swu2.self",
        0x500: "sdk_version.txt",
        0x501: "spkg.tar",
        0x600: "ps3swu3.self",
        0x601: "ps3swu4.self",
    }
    return mapping.get(entry_id, f"segment_{entry_id:x}.bin")

def find_dev_flash(root_dir):
    """Find dev_flash directory recursively"""
    for dirpath, dirnames, filenames in os.walk(root_dir):
        if 'dev_flash' in dirnames:
            return os.path.join(dirpath, 'dev_flash')
    return None

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: extract_pup_fixed.py <pup_file> <output_dir>")
        sys.exit(1)
    
    pup_file = sys.argv[1]
    output_dir = sys.argv[2]
    
    if not os.path.exists(pup_file):
        print(f"Error: {pup_file} not found")
        sys.exit(1)
    
    dev_flash_path = extract_pup(pup_file, output_dir)
    
    if dev_flash_path:
        # Print path on last line for Rust to parse
        print(f"DEV_FLASH_PATH:{dev_flash_path}")
        sys.exit(0)
    else:
        sys.exit(1)
