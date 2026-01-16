use std::path::Path;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileType {
    ISO,
    CSO,
    BIN,
    CUE,
    NSP,
    XCI,
    RVZ,
    WIA,
    WBFS,
    GCM,
    CHD,
    ELF,
    PBP,
    NDS,
    CIA,
    WUA,
    WUD,
    GDI,
    CDI,
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Platform {
    Wii,
    GameCube,
    PS1,
    PS2,
    PSP,
    Xbox,
    Switch,
    Nintendo3DS,
    NintendoDS,
    WiiU,
    Dreamcast,
    PS3,
    PS4,
    Unknown,
}

impl Platform {
    pub fn as_str(&self) -> &'static str {
        match self {
            Platform::Wii => "wii",
            Platform::GameCube => "gamecube",
            Platform::PS1 => "ps1",
            Platform::PS2 => "ps2",
            Platform::PSP => "psp",
            Platform::Xbox => "xbox",
            Platform::Switch => "switch",
            Platform::Nintendo3DS => "3ds",
            Platform::NintendoDS => "nds",
            Platform::WiiU => "wiiu",
            Platform::Dreamcast => "dreamcast",
            Platform::PS3 => "ps3",
            Platform::PS4 => "ps4",
            Platform::Unknown => "unknown",
        }
    }
}

pub struct FileAnalyzer;

impl FileAnalyzer {
    pub fn detect_type(path: &Path) -> Option<FileType> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "iso" => Some(FileType::ISO),
            "cso" => Some(FileType::CSO),
            "bin" => Some(FileType::BIN),
            "cue" => Some(FileType::CUE),
            "nsp" => Some(FileType::NSP),
            "xci" => Some(FileType::XCI),
            "rvz" => Some(FileType::RVZ),
            "wia" => Some(FileType::WIA),
            "wbfs" => Some(FileType::WBFS),
            "gcm" => Some(FileType::GCM),
            "chd" => Some(FileType::CHD),
            "elf" => Some(FileType::ELF),
            "pbp" => Some(FileType::PBP),
            "nds" => Some(FileType::NDS),
            "cia" => Some(FileType::CIA),
            "wua" => Some(FileType::WUA),
            "wud" => Some(FileType::WUD),
            "gdi" => Some(FileType::GDI),
            "cdi" => Some(FileType::CDI),
            other => Some(FileType::Unknown(other.to_string())),
        }
    }

    pub fn is_valid_rom(path: &Path) -> bool {
        if !path.exists() || !path.is_file() {
            return false;
        }
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() == 0 {
                return false;
            }
        }
        true
    }

    pub fn identify_platform(path: &Path) -> Platform {
        // 1. Fast path: Extension check for unambiguous formats
        if let Some(ext) = path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) {
            match ext.as_str() {
                "nds" => return Platform::NintendoDS,
                "3ds" | "cia" => return Platform::Nintendo3DS,
                "nsp" | "xci" => return Platform::Switch,
                "wua" | "wud" => return Platform::WiiU,
                "gdi" | "cdi" => return Platform::Dreamcast,
                "gcm" => return Platform::GameCube, // GCM is always GC
                "wbfs" => return Platform::Wii, // WBFS is always Wii
                "pbp" => {
                   // PBP is mostly PSP, but technically PS1 classics too. Defaults to PSP.
                   return Platform::PSP;
                },
                "cso" => {
                    // CSO is mostly PSP, sometimes PS2. Cheching magic `Cqb`... 
                    // Let's rely on magic check below if possible, or default to PSP.
                }, 
                _ => {} // ISO, BIN, CUE, RVZ, CHD needs analysis
            }
        }

        // 2. Magic Bytes Analysis
        let mut file = match File::open(path) {
            Ok(f) => f,
            Err(_) => return Platform::Unknown,
        };

        // --- Nintendo Wii & GameCube ---
        // Wii: 0x18 = 0x5D1C9EA3
        // GC: 0x1C = 0xC2339F3D
        let mut buffer = [0u8; 32];
        if file.read_exact(&mut buffer).is_ok() {
            // Wii Magic Word at 0x18
            if buffer[0x18] == 0x5D && buffer[0x19] == 0x1C && buffer[0x1A] == 0x9E && buffer[0x1B] == 0xA3 {
                return Platform::Wii;
            }
            // GameCube Magic Word at 0x1C
            if buffer[0x1C] == 0xC2 && buffer[0x1D] == 0x33 && buffer[0x1E] == 0x9F && buffer[0x1F] == 0x3D {
                return Platform::GameCube;
            }
        }

        // --- ISO 9660 Check (PS1, PS2, PSP) ---
        // Sector 16 (0x8000 for 2048 sector size, or raw binary could vary)
        // System Identifier at 0x8008 (32 bytes)
        // "PLAYSTATION" => PS1 or PS2
        // "PSP GAME" => PSP

        // Essayer l'offset standard 0x8000 (Mode 1 / 2048 bytes sectors)
        if file.seek(SeekFrom::Start(0x8000)).is_ok() {
            let mut pvd = [0u8; 100]; // Read enough for System ID
            if file.read_exact(&mut pvd).is_ok() {
                // Check PVD Identifier "CD001" at start + 1
                if pvd[1] == b'C' && pvd[2] == b'D' && pvd[3] == b'0' && pvd[4] == b'0' && pvd[5] == b'1' {
                    let system_id = &pvd[8..40]; // 32 bytes
                    let sys_id_str = String::from_utf8_lossy(system_id);
                    
                    if sys_id_str.contains("PSP GAME") {
                        return Platform::PSP;
                    }

                    if sys_id_str.contains("PLAYSTATION") {
                        // 1. Check for PS3 Disc IDs (BLES, BLUS, BCjs, etc.)
                        // PS3 IDs start with 'B', PS1/PS2 IDs start with 'S'.
                        if Self::scan_for_string(&mut file, "BLES", 2 * 1024 * 1024) 
                           || Self::scan_for_string(&mut file, "BLUS", 2 * 1024 * 1024)
                           || Self::scan_for_string(&mut file, "BCJS", 2 * 1024 * 1024)
                           || Self::scan_for_string(&mut file, "BLJS", 2 * 1024 * 1024) {
                            return Platform::PS3;
                        }

                        // 2. Check for "BOOT2" (PS2 specific config param in SYSTEM.CNF)
                        if Self::scan_for_string(&mut file, "BOOT2", 2 * 1024 * 1024) {
                            return Platform::PS2;
                        }
                        
                        // 3. Check Size > 750MB -> Likely PS2 (DVD) or PS3 (BD)
                        // If we missed the PS3 ID check but size is huge, it could be PS3 or PS2.
                        // However, PS2 DVDs are common. PS3 ISOs are rarer but exist.
                        // We rely on ID check for PS3 mostly.
                        // If it's big, it's definitely not PS1.
                        if let Ok(m) = file.metadata() {
                             if m.len() > 750 * 1024 * 1024 {
                                 // Could be PS2 or PS3. 
                                 // If "PLAYSTATION" magic is present, but no BLES/BLUS found:
                                 // Default to PS2 as it's the legacy format using this magic heavily.
                                 return Platform::PS2;
                             }
                        }

                        // 4. Fallback to PS1
                        return Platform::PS1;
                    }
                }
            }
        }

        // --- Xbox (Original) ---
        // "MICROSOFT*XBOX*MEDIA" at 0x10000 (standard XISO)
        if file.seek(SeekFrom::Start(0x10000)).is_ok() {
             let mut xiso_magic = [0u8; 20];
             if file.read_exact(&mut xiso_magic).is_ok() {
                 let s = String::from_utf8_lossy(&xiso_magic);
                 if s.contains("MICROSOFT*XBOX*MEDIA") {
                     return Platform::Xbox;
                 }
             }
        }
        
        // --- RVZ / GCZ (Dolphin) ---
        // RVZ Header: 52 56 5A 01 (RVZ\x01)
        if file.seek(SeekFrom::Start(0)).is_ok() {
            let mut header = [0u8; 4];
            if file.read_exact(&mut header).is_ok() {
                if &header == b"RVZ\x01" {
                    // RVZ est utilisé par Dolphin pour GC et Wii. 
                    // On retourne Wii par défaut car c'est plus commun pour ce format compressé moderne.
                    return Platform::Wii;
                }
            }
        }

        // --- Switch (XCI / NSP) ---
        // XCI: "HEAD" at 0x100
        if file.seek(SeekFrom::Start(0x100)).is_ok() {
            let mut buf = [0u8; 4];
            if file.read_exact(&mut buf).is_ok() && &buf == b"HEAD" {
                return Platform::Switch;
            }
        }
        // NSP: "PFS0" at 0x0
        if file.seek(SeekFrom::Start(0)).is_ok() {
            let mut buf = [0u8; 4];
            if file.read_exact(&mut buf).is_ok() && &buf == b"PFS0" {
                return Platform::Switch;
            }
        }

        // --- 3DS (NCSD / NCCH) ---
        // often at 0x100
        if file.seek(SeekFrom::Start(0x100)).is_ok() {
            let mut buf = [0u8; 4];
            if file.read_exact(&mut buf).is_ok() {
                if &buf == b"NCSD" || &buf == b"NCCH" {
                    return Platform::Nintendo3DS;
                }
            }
        }

        // --- Dreamcast (GDI) ---
        // GDI is a text file describing tracks.
        // We can check if it looks like a text file and contains "Track" or is small enough to be a descriptor.
        if let Some(ext) = path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()) {
            if ext == "gdi" {
                 // Verify it's text-like?
                 // Simple check: read first few bytes, should be ASCII digit (number of tracks)
                 if file.seek(SeekFrom::Start(0)).is_ok() {
                     let mut buf = [0u8; 1];
                     if file.read_exact(&mut buf).is_ok() && buf[0].is_ascii_digit() {
                         return Platform::Dreamcast;
                     }
                 }
            }
        }


        // --- PS3 (PKG) ---
        // Magic: 7F 50 4B 47 (.PKG)
        if file.seek(SeekFrom::Start(0)).is_ok() {
            let mut buf = [0u8; 4];
            if file.read_exact(&mut buf).is_ok() {
                if buf == [0x7F, 0x50, 0x4B, 0x47] { // \x7FPKG
                     // Could be PS3 or PS4.
                     // For now, we default to PS3 as it is supported by RPCS3.
                     return Platform::PS3;
                }
            }
        }

        Platform::Unknown
    }
    
    fn scan_for_string(file: &mut File, ex: &str, limit: u64) -> bool {
        // Simple buffer scan
        let _ = file.seek(SeekFrom::Start(0));
        let mut buffer = [0u8; 4096];
        let mut scanned = 0;
        let bytes = ex.as_bytes();
        
        while scanned < limit {
            let n = match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => n,
                Err(_) => break,
            };
            
            // Naive search
            if buffer[..n].windows(bytes.len()).any(|w| w == bytes) {
                return true;
            }
            
            scanned += n as u64;
        }
        false
    }
}
