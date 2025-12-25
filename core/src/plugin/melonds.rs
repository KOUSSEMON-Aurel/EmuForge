use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct MelonDSPlugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl MelonDSPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for MelonDSPlugin {
    fn id(&self) -> &str { "melonds" }
    fn name(&self) -> &str { "melonDS (NDS)" }
    fn supported_extensions(&self) -> &[&str] { &["nds", "srl", "dsi"] }

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        if let Ok(path) = which::which("melonDS") { return Ok(path); }
        if let Ok(path) = which::which("melonds") { return Ok(path); }
        
        Err(anyhow::anyhow!("melonDS executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate melonDS binary")?;
        
        // melonDS Args: <rom>
        let args = vec![];

        Ok(LaunchConfig {
            emulator_path: binary,
            rom_path: rom_path.to_path_buf(),
            bios_path: None, 
            args,
            working_dir: None, 
            env_vars: vec![],
        })
    }

    fn can_handle(&self, binary_path: &Path) -> bool {
        let name = binary_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        name.contains("melonds")
    }
}
