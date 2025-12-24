use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for launching an emulator, embedded into the stub.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LaunchConfig {
    /// Path to the emulator executable (relative or absolute).
    pub emulator_path: PathBuf,
    /// Path to the ROM file (relative or absolute).
    pub rom_path: PathBuf,
    /// Arguments to pass to the emulator.
    pub args: Vec<String>,
    /// Working directory for the emulator process.
    pub working_dir: Option<PathBuf>,
    /// Environment variables to set.
    pub env_vars: Vec<(String, String)>,
}
