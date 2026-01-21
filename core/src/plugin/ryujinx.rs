use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct RyujinxPlugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl RyujinxPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for RyujinxPlugin {
    fn id(&self) -> &str { "ryujinx" }
    fn name(&self) -> &str { "Ryujinx (Switch)" }
    fn supported_extensions(&self) -> &[&str] { &["nsp", "xci", "nca", "nro"] }

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        if let Ok(path) = which::which("Ryujinx") { return Ok(path); }
        if let Ok(path) = which::which("ryujinx") { return Ok(path); }
        
        Err(anyhow::anyhow!("Ryujinx executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate Ryujinx binary")?;
        
        // Ryujinx Args: <path> --fullscreen
        // Note: Ryujinx might expect path as argument, not necessarily last? 
        // Usage: Ryujinx [options] <path_to_application>
        // It's flexible.
        let args = vec![
            // "--fullscreen".to_string(), // Handled by generic toggle
        ];

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
        name.contains("ryujinx")
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(RyujinxPlugin::new(Some(binary_path)))
    }
}
