//! Module d'extraction PS3 PUP (PlayStation Update Package)
//! Basé sur l'analyse du code source RPCS3

use anyhow::{Result, Context, anyhow};
use std::path::{Path, PathBuf};
use std::fs::{self, File};
use std::io::{Read, Write, Seek, SeekFrom};

/// Clé HMAC-SHA1 pour validation des entrées PUP (publique dans RPCS3)
#[allow(dead_code)]
const PUP_KEY: [u8; 0x40] = [
    0xF4, 0x91, 0xAD, 0x94, 0xC6, 0x81, 0x10, 0x96, 0x91, 0x5F, 0xD5, 0xD2, 0x44, 0x81, 0xAE, 0xDC,
    0xED, 0xED, 0xBE, 0x6B, 0xE5, 0x13, 0x72, 0x4D, 0xD8, 0xF7, 0xB6, 0x91, 0xE8, 0x8A, 0x38, 0xF4,
    0xB5, 0x16, 0x2B, 0xFB, 0xEC, 0xBE, 0x3A, 0x62, 0x18, 0x5D, 0xD7, 0xC9, 0x4D, 0xA2, 0x22, 0x5A,
    0xDA, 0x3F, 0xBF, 0xCE, 0x55, 0x5B, 0x9E, 0xA9, 0x64, 0x98, 0x29, 0xEB, 0x30, 0xCE, 0x83, 0x66,
];

/// Magic PUP : "SCEUF\0\0\0" lu en little-endian
const PUP_MAGIC: u64 = 0x0000004655454353; // "SCEUF\0\0\0" en LE

/// Entry ID pour update_files.tar
const UPDATE_FILES_ENTRY_ID: u64 = 0x300;

/// Entry ID pour version.txt
const VERSION_ENTRY_ID: u64 = 0x100;

/// Header PUP (0x30 bytes)
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PupHeader {
    magic: u64,              // "SCEUF\0\0\0" (LE)
    package_version: u64,    // Version du package (BE)
    image_version: u64,      // Version de l'image (BE)
    file_count: u64,         // Nombre de fichiers (BE)
    header_length: u64,      // Longueur du header (BE)
    data_length: u64,        // Longueur des données (BE)
}

/// Entry dans la table des fichiers PUP (0x20 bytes)
#[derive(Debug, Clone)]
struct PupFileEntry {
    entry_id: u64,           // ID du fichier (BE)
    data_offset: u64,        // Offset des données (BE)
    data_length: u64,        // Taille des données (BE)
    // 8 bytes padding
}

/// Lit un u64 big-endian depuis un buffer
fn read_be_u64(buf: &[u8]) -> u64 {
    u64::from_be_bytes([buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]])
}

/// Lit un u64 little-endian depuis un buffer
fn read_le_u64(buf: &[u8]) -> u64 {
    u64::from_le_bytes([buf[0], buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7]])
}

/// Parse le header PUP
fn parse_pup_header(data: &[u8]) -> Result<PupHeader> {
    if data.len() < 0x30 {
        return Err(anyhow!("PUP file too small for header"));
    }
    
    Ok(PupHeader {
        magic: read_le_u64(&data[0x00..0x08]),
        package_version: read_be_u64(&data[0x08..0x10]),
        image_version: read_be_u64(&data[0x10..0x18]),
        file_count: read_be_u64(&data[0x18..0x20]),
        header_length: read_be_u64(&data[0x20..0x28]),
        data_length: read_be_u64(&data[0x28..0x30]),
    })
}

/// Parse une entrée de fichier PUP
fn parse_pup_entry(data: &[u8]) -> Result<PupFileEntry> {
    if data.len() < 0x20 {
        return Err(anyhow!("Entry data too small"));
    }
    
    Ok(PupFileEntry {
        entry_id: read_be_u64(&data[0x00..0x08]),
        data_offset: read_be_u64(&data[0x08..0x10]),
        data_length: read_be_u64(&data[0x10..0x18]),
    })
}

/// Extrait le firmware PS3 depuis PS3UPDAT.PUP
pub fn extract_firmware(pup_path: &Path, output_dir: &Path) -> Result<PathBuf> {
    // Créer répertoire temporaire pour extraction
    let temp_dir = output_dir.join(format!("ps3_fw_extract_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir)
        .context("Failed to create temp directory for firmware extraction")?;
    
    println!("📦 Extracting PS3UPDAT.PUP to {:?}...", temp_dir);
    
    // Lire le fichier PUP complet
    let mut file = File::open(pup_path)
        .context("Failed to open PS3UPDAT.PUP file")?;
    
    let file_size = file.metadata()?.len() as usize;
    let mut pup_data = vec![0u8; file_size];
    file.read_exact(&mut pup_data)
        .context("Failed to read PUP file")?;
    
    // Parser le header
    let header = parse_pup_header(&pup_data)?;
    
    // Vérifier le magic
    if header.magic != PUP_MAGIC {
        return Err(anyhow!(
            "Invalid PUP magic: expected SCEUF, got 0x{:016x}",
            header.magic
        ));
    }
    
    println!("  ✅ Valid PUP header:");
    println!("     File count: {}", header.file_count);
    println!("     Header length: 0x{:x}", header.header_length);
    println!("     Data length: 0x{:x}", header.data_length);
    
    // Parser la table des fichiers (après le header de 0x30 bytes)
    let mut entries = Vec::new();
    let table_offset = 0x30usize;
    
    for i in 0..header.file_count as usize {
        let entry_offset = table_offset + (i * 0x20);
        if entry_offset + 0x20 > pup_data.len() {
            return Err(anyhow!("PUP file truncated at entry {}", i));
        }
        
        let entry = parse_pup_entry(&pup_data[entry_offset..entry_offset + 0x20])?;
        entries.push(entry);
    }
    
    // Trouver et extraire update_files.tar (entry_id = 0x300)
    let update_files_entry = entries.iter()
        .find(|e| e.entry_id == UPDATE_FILES_ENTRY_ID)
        .ok_or_else(|| anyhow!("update_files.tar (entry 0x300) not found in PUP"))?;
    
    println!("  📄 Found update_files.tar:");
    println!("     Offset: 0x{:x}", update_files_entry.data_offset);
    println!("     Size: {} bytes", update_files_entry.data_length);
    
    // Extraire update_files.tar
    let tar_start = update_files_entry.data_offset as usize;
    let tar_end = tar_start + update_files_entry.data_length as usize;
    
    if tar_end > pup_data.len() {
        return Err(anyhow!("update_files.tar extends beyond PUP file"));
    }
    
    let tar_data = &pup_data[tar_start..tar_end];
    let tar_path = temp_dir.join("update_files.tar");
    
    let mut tar_file = File::create(&tar_path)?;
    tar_file.write_all(tar_data)?;
    
    println!("  📦 Extracting update_files.tar...");
    
    // Extraire le TAR
    let tar_file = File::open(&tar_path)?;
    let mut archive = tar::Archive::new(tar_file);
    archive.unpack(&temp_dir)
        .context("Failed to extract update_files.tar")?;
    
    // Créer le répertoire dev_flash de destination
    let dev_flash_dir = temp_dir.join("dev_flash");
    fs::create_dir_all(&dev_flash_dir)?;
    
    // Trouver et traiter les fichiers dev_flash_*.tar.*
    println!("  🔐 Decrypting dev_flash packages...");
    
    let mut dev_flash_files: Vec<_> = fs::read_dir(&temp_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.starts_with("dev_flash_") && name.contains(".tar.")
        })
        .collect();
    
    dev_flash_files.sort_by_key(|e| e.file_name());
    
    if dev_flash_files.is_empty() {
        return Err(anyhow!("No dev_flash_* packages found in update_files.tar"));
    }
    
    println!("     Found {} dev_flash packages", dev_flash_files.len());
    
    for entry in &dev_flash_files {
        let path = entry.path();
        let filename = path.file_name().unwrap().to_string_lossy();
        println!("     Processing {}...", filename);
        
        // Lire le fichier SELF/SCE
        let sce_data = fs::read(&path)?;
        
        // Déchiffrer avec SCEDecrypter
        match crate::crypto::sce::decrypt_sce_file(&sce_data) {
            Ok(sections) => {
                for section in sections.iter() {
                    // Vérifier si c'est un TAR valide (magic "ustar" à offset 257)
                    let is_tar = section.len() > 262 && &section[257..262] == b"ustar";
                    if is_tar {
                        let cursor = std::io::Cursor::new(section);
                        let mut inner_tar = tar::Archive::new(cursor);
                        let _ = inner_tar.unpack(&dev_flash_dir);
                    }
                }
                println!("       ✅ Extracted");
            }
            Err(e) => {
                println!("       ⚠️  Decryption failed: {}", e);
            }
        }
    }
    
    // Vérifier que dev_flash contient des fichiers
    let content_count = fs::read_dir(&dev_flash_dir)?.count();
    if content_count == 0 {
        return Err(anyhow!("dev_flash is empty after extraction"));
    }
    
    println!("✅ Firmware extracted successfully");
    println!("   dev_flash: {:?} ({} entries)", dev_flash_dir, content_count);
    
    Ok(dev_flash_dir)
}



/// Extrait la version du firmware depuis le PUP
#[allow(dead_code)]
pub fn get_firmware_version(pup_path: &Path) -> Result<String> {
    let mut file = File::open(pup_path)?;
    
    let file_size = file.metadata()?.len() as usize;
    let mut pup_data = vec![0u8; file_size.min(1024 * 1024)]; // Lire max 1MB pour le header
    file.read(&mut pup_data)?;
    
    let header = parse_pup_header(&pup_data)?;
    
    // Parser les entrées
    let table_offset = 0x30usize;
    for i in 0..header.file_count.min(256) as usize {
        let entry_offset = table_offset + (i * 0x20);
        if entry_offset + 0x20 > pup_data.len() {
            break;
        }
        
        let entry = parse_pup_entry(&pup_data[entry_offset..entry_offset + 0x20])?;
        
        if entry.entry_id == VERSION_ENTRY_ID {
            file.seek(SeekFrom::Start(entry.data_offset))?;
            let mut version = vec![0u8; entry.data_length as usize];
            file.read_exact(&mut version)?;
            
            let version_str = String::from_utf8_lossy(&version);
            // Prendre la première ligne
            return Ok(version_str.lines().next().unwrap_or("").to_string());
        }
    }
    
    Err(anyhow!("Version not found in PUP"))
}
