use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for launching an emulator, embedded into the stub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    /// Path to the emulator executable (relative or absolute).
    pub emulator_path: PathBuf,
    /// Path to the ROM file (relative or absolute).
    pub rom_path: PathBuf,
    /// Optional path to the BIOS file.
    pub bios_path: Option<PathBuf>,
    /// Arguments to pass to the emulator BEFORE the ROM path.
    pub args: Vec<String>,
    /// Arguments to pass to the emulator AFTER the ROM path.
    #[serde(default)]
    pub args_after_rom: Vec<String>,
    /// Working directory for the emulator process.
    pub working_dir: Option<PathBuf>,
    /// Environment variables to set.
    pub env_vars: Vec<(String, String)>,
}

impl Default for LaunchConfig {
    fn default() -> Self {
        Self {
            emulator_path: PathBuf::new(),
            rom_path: PathBuf::new(),
            bios_path: None,
            args: vec![],
            args_after_rom: vec![],
            working_dir: None,
            env_vars: vec![],
        }
    }
}

