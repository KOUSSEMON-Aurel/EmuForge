use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct RedreamPlugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl RedreamPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for RedreamPlugin {
    fn id(&self) -> &str { "redream" }
    fn name(&self) -> &str { "Redream (Dreamcast)" }
    fn supported_extensions(&self) -> &[&str] { &["gdi", "cdi", "chd", "cue"] }

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        if let Ok(path) = which::which("redream") { return Ok(path); }
        
        Err(anyhow::anyhow!("Redream executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate Redream binary")?;
        
        // Redream Args: <rom>
        let args = vec![];

        Ok(LaunchConfig {
            emulator_path: binary,
            rom_path: rom_path.to_path_buf(),
            bios_path: None, 
            args,
            working_dir: None, 
            args_after_rom: vec![],
            env_vars: vec![],
        })
    }

    fn can_handle(&self, binary_path: &Path) -> bool {
        let name = binary_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        name.contains("redream") || name.contains("flycast")
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(RedreamPlugin::new(Some(binary_path)))
    }
}
