use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::forge::LaunchConfig;

pub trait EmulatorPlugin: Send + Sync {
    /// Unique identifier for the emulator (e.g., "ppsspp").
    fn id(&self) -> &str;
    
    /// User-friendly name of the emulator.
    fn name(&self) -> &str;
    
    /// List of file extensions supported by this emulator (without dot).
    fn supported_extensions(&self) -> &[&str];
    
    /// Locate the emulator binary on the host system.
    fn find_binary(&self) -> Result<PathBuf>;
    
    /// prepare the launch configuration for a specific ROM.
    fn prepare_launch_config(&self, rom_path: &Path, output_dir: &Path) -> Result<LaunchConfig>;
}
