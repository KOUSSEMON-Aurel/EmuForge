use std::path::Path;


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
    Unknown(String),
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
            other => Some(FileType::Unknown(other.to_string())),
        }
    }

    pub fn is_valid_rom(path: &Path) -> bool {
        if !path.exists() || !path.is_file() {
            return false;
        }
        // Basic check: file size > 0?
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() == 0 {
                return false;
            }
        }
        true
    }
}
