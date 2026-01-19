//! Module de déchiffrement SCE pour PS3 firmware
//! Basé sur le code RPCS3 (rpcs3/Crypto/unself.cpp)

use aes::Aes128;
use aes::Aes256;
use aes::cipher::{BlockDecryptMut, KeyIvInit, StreamCipher};
use aes::cipher::generic_array::GenericArray;
use anyhow::{Result, anyhow, Context};
use std::io::Read;

/// Clé ERK pour déchiffrer les packages SCE
pub const SCEPKG_ERK: [u8; 0x20] = [
    0xA9, 0x78, 0x18, 0xBD, 0x19, 0x3A, 0x67, 0xA1, 0x6F, 0xE8, 0x3A, 0x85, 0x5E, 0x1B, 0xE9, 0xFB,
    0x56, 0x40, 0x93, 0x8D, 0x4D, 0xBC, 0xB2, 0xCB, 0x52, 0xC5, 0xA2, 0xF8, 0xB0, 0x2B, 0x10, 0x31,
];

/// RIV pour déchiffrer les packages SCE
pub const SCEPKG_RIV: [u8; 0x10] = [
    0x4A, 0xCE, 0xF0, 0x12, 0x24, 0xFB, 0xEE, 0xDF, 0x82, 0x45, 0xF8, 0xFF, 0x10, 0x21, 0x1E, 0x6E,
];

/// SCE Header magic: "SCE\0" = 0x53434500
const SCE_MAGIC: u32 = 0x53434500;

/// SCE Header structure (0x20 bytes)
#[derive(Debug, Clone, Default)]
pub struct SceHeader {
    pub se_magic: u32,    // "SCE\0"
    pub se_hver: u32,     // Header version
    pub se_flags: u16,    // Flags
    pub se_type: u16,     // Type
    pub se_meta: u32,     // Metadata offset
    pub se_hsize: u64,    // Header size
    pub se_esize: u64,    // ???
}

/// Metadata Info structure (0x40 bytes)
#[derive(Debug, Clone, Default)]
pub struct MetadataInfo {
    pub key: [u8; 0x10],      // AES key
    pub key_pad: [u8; 0x10],  // Padding (should be zeros after decryption)
    pub iv: [u8; 0x10],       // AES IV
    pub iv_pad: [u8; 0x10],   // Padding (should be zeros after decryption)
}

/// Metadata Header structure
#[derive(Debug, Clone, Default)]
pub struct MetadataHeader {
    pub signature_input_length: u64,
    pub unknown1: u32,
    pub section_count: u32,
    pub key_count: u32,
    pub opt_header_size: u32,
    pub unknown2: u32,
    pub unknown3: u32,
}

/// Metadata Section Header structure
#[derive(Debug, Clone, Default)]
pub struct MetadataSectionHeader {
    pub data_offset: u64,
    pub data_size: u64,
    pub typ: u32,
    pub program_idx: u32,
    pub hashed: u32,
    pub sha1_idx: u32,
    pub encrypted: u32,
    pub key_idx: u32,
    pub iv_idx: u32,
    pub compressed: u32,
}

fn read_be_u32(data: &[u8]) -> u32 {
    u32::from_be_bytes([data[0], data[1], data[2], data[3]])
}

fn read_be_u64(data: &[u8]) -> u64 {
    u64::from_be_bytes([data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7]])
}

fn read_be_u16(data: &[u8]) -> u16 {
    u16::from_be_bytes([data[0], data[1]])
}

impl SceHeader {
    pub fn load(data: &[u8]) -> Result<Self> {
        if data.len() < 0x20 {
            return Err(anyhow!("Not enough data for SCE header"));
        }
        Ok(Self {
            se_magic: read_be_u32(&data[0..4]),
            se_hver: read_be_u32(&data[4..8]),
            se_flags: read_be_u16(&data[8..10]),
            se_type: read_be_u16(&data[10..12]),
            se_meta: read_be_u32(&data[12..16]),
            se_hsize: read_be_u64(&data[16..24]),
            se_esize: read_be_u64(&data[24..32]),
        })
    }
    
    pub fn check_magic(&self) -> bool {
        self.se_magic == SCE_MAGIC
    }
}

impl MetadataInfo {
    pub fn load(data: &[u8]) -> Result<Self> {
        if data.len() < 0x40 {
            return Err(anyhow!("Not enough data for MetadataInfo"));
        }
        let mut info = Self::default();
        info.key.copy_from_slice(&data[0..0x10]);
        info.key_pad.copy_from_slice(&data[0x10..0x20]);
        info.iv.copy_from_slice(&data[0x20..0x30]);
        info.iv_pad.copy_from_slice(&data[0x30..0x40]);
        Ok(info)
    }
}

impl MetadataHeader {
    pub fn load(data: &[u8]) -> Result<Self> {
        if data.len() < 0x20 {
            return Err(anyhow!("Not enough data for MetadataHeader"));
        }
        Ok(Self {
            signature_input_length: read_be_u64(&data[0..8]),
            unknown1: read_be_u32(&data[8..12]),
            section_count: read_be_u32(&data[12..16]),
            key_count: read_be_u32(&data[16..20]),
            opt_header_size: read_be_u32(&data[20..24]),
            unknown2: read_be_u32(&data[24..28]),
            unknown3: read_be_u32(&data[28..32]),
        })
    }
}

impl MetadataSectionHeader {
    pub fn load(data: &[u8]) -> Result<Self> {
        if data.len() < 0x30 {
            return Err(anyhow!("Not enough data for MetadataSectionHeader"));
        }
        Ok(Self {
            data_offset: read_be_u64(&data[0..8]),
            data_size: read_be_u64(&data[8..16]),
            typ: read_be_u32(&data[16..20]),
            program_idx: read_be_u32(&data[20..24]),
            hashed: read_be_u32(&data[24..28]),
            sha1_idx: read_be_u32(&data[28..32]),
            encrypted: read_be_u32(&data[32..36]),
            key_idx: read_be_u32(&data[36..40]),
            iv_idx: read_be_u32(&data[40..44]),
            compressed: read_be_u32(&data[44..48]),
        })
    }
}

/// AES-256 CBC decrypt (pour metadata_info)
fn aes256_cbc_decrypt(key: &[u8; 32], iv: &[u8; 16], data: &mut [u8]) {
    type Aes256CbcDec = cbc::Decryptor<Aes256>;
    
    let cipher = Aes256CbcDec::new(
        GenericArray::from_slice(key),
        GenericArray::from_slice(iv),
    );
    
    // Decrypt in-place
    let _ = cipher.decrypt_padded_mut::<aes::cipher::block_padding::NoPadding>(data);
}

/// AES-128 CTR encrypt/decrypt
fn aes128_ctr(key: &[u8; 16], iv: &[u8; 16], data: &mut [u8]) {
    type Aes128Ctr = ctr::Ctr128BE<Aes128>;
    
    let mut cipher = <Aes128Ctr as KeyIvInit>::new(
        GenericArray::from_slice(key),
        GenericArray::from_slice(iv),
    );
    cipher.apply_keystream(data);
}

/// Décompresse des données zlib
fn decompress_zlib(data: &[u8]) -> Result<Vec<u8>> {
    use flate2::read::ZlibDecoder;
    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed)
        .context("Failed to decompress zlib data")?;
    Ok(decompressed)
}

/// Déchiffre un fichier SELF/PKG et retourne les sections déchiffrées
pub fn decrypt_sce_file(data: &[u8]) -> Result<Vec<Vec<u8>>> {
    println!("    🔐 Decrypting SCE file ({} bytes)...", data.len());
    
    // Load SCE header
    let sce_hdr = SceHeader::load(data)?;
    
    if !sce_hdr.check_magic() {
        return Err(anyhow!("Invalid SCE magic: 0x{:08x}", sce_hdr.se_magic));
    }
    
    println!("      SCE header valid: type=0x{:x}, meta_offset=0x{:x}", sce_hdr.se_type, sce_hdr.se_meta);
    
    // Read encrypted metadata info
    let meta_info_offset = (sce_hdr.se_meta as usize) + 0x20; // sizeof(sce_hdr) = 0x20
    if meta_info_offset + 0x40 > data.len() {
        return Err(anyhow!("File too small for metadata info"));
    }
    
    let mut metadata_info_bytes = [0u8; 0x40];
    metadata_info_bytes.copy_from_slice(&data[meta_info_offset..meta_info_offset + 0x40]);
    
    // Decrypt metadata info with ERK/RIV if not debug
    if (sce_hdr.se_flags & 0x8000) != 0x8000 {
        aes256_cbc_decrypt(&SCEPKG_ERK, &SCEPKG_RIV, &mut metadata_info_bytes);
    }
    
    let meta_info = MetadataInfo::load(&metadata_info_bytes)?;
    
    // Check if decryption was successful (padding should be zeros)
    if meta_info.key_pad[0] != 0 || meta_info.iv_pad[0] != 0 {
        return Err(anyhow!("Failed to decrypt SCE metadata info - wrong keys?"));
    }
    
    println!("      ✅ Metadata info decrypted");
    
    // Read and decrypt metadata headers
    let metadata_headers_offset = meta_info_offset + 0x40;
    let metadata_headers_size = (sce_hdr.se_hsize as usize) - (0x20 + sce_hdr.se_meta as usize + 0x40);
    
    if metadata_headers_offset + metadata_headers_size > data.len() {
        return Err(anyhow!("File too small for metadata headers"));
    }
    
    let mut metadata_headers = data[metadata_headers_offset..metadata_headers_offset + metadata_headers_size].to_vec();
    
    // Decrypt with AES-CTR using keys from metadata_info
    aes128_ctr(&meta_info.key, &meta_info.iv, &mut metadata_headers);
    
    // Parse metadata header
    let meta_hdr = MetadataHeader::load(&metadata_headers)?;
    println!("      Section count: {}, Key count: {}", meta_hdr.section_count, meta_hdr.key_count);
    
    // Parse section headers
    let mut section_headers = Vec::new();
    for i in 0..meta_hdr.section_count as usize {
        let offset = 0x20 + i * 0x30; // sizeof(MetadataHeader) = 0x20, sizeof(MetadataSectionHeader) = 0x30
        if offset + 0x30 > metadata_headers.len() {
            break;
        }
        let shdr = MetadataSectionHeader::load(&metadata_headers[offset..offset + 0x30])?;
        section_headers.push(shdr);
    }
    
    // Get data keys
    let data_keys_offset = 0x20 + (meta_hdr.section_count as usize) * 0x30;
    let data_keys_len = (meta_hdr.key_count as usize) * 0x10;
    if data_keys_offset + data_keys_len > metadata_headers.len() {
        return Err(anyhow!("Not enough data for keys"));
    }
    let data_keys = &metadata_headers[data_keys_offset..data_keys_offset + data_keys_len];
    
    // Decrypt each section
    let mut result_sections = Vec::new();
    
    for (i, shdr) in section_headers.iter().enumerate() {
        let section_offset = shdr.data_offset as usize;
        let section_size = shdr.data_size as usize;
        
        if section_offset + section_size > data.len() {
            println!("      ⚠️  Section {} extends beyond file, skipping", i);
            continue;
        }
        
        let mut section_data = data[section_offset..section_offset + section_size].to_vec();
        
        // Decrypt if encrypted (encrypted == 3)
        if shdr.encrypted == 3 {
            let key_idx = shdr.key_idx as usize;
            let iv_idx = shdr.iv_idx as usize;
            
            if (key_idx + 1) * 0x10 <= data_keys.len() && (iv_idx + 1) * 0x10 <= data_keys.len() {
                let mut key = [0u8; 16];
                let mut iv = [0u8; 16];
                key.copy_from_slice(&data_keys[key_idx * 0x10..(key_idx + 1) * 0x10]);
                iv.copy_from_slice(&data_keys[iv_idx * 0x10..(iv_idx + 1) * 0x10]);
                
                aes128_ctr(&key, &iv, &mut section_data);
            }
        }
        
        // Decompress if compressed (compressed == 2)
        let final_data = if shdr.compressed == 2 {
            match decompress_zlib(&section_data) {
                Ok(decompressed) => decompressed,
                Err(e) => {
                    println!("      ⚠️  Section {} decompression failed: {}", i, e);
                    section_data
                }
            }
        } else {
            section_data
        };
        
        result_sections.push(final_data);
    }
    
    println!("      ✅ Decrypted {} sections", result_sections.len());
    Ok(result_sections)
}
