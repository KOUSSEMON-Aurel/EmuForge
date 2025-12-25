use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct Rpcs3Plugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl Rpcs3Plugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for Rpcs3Plugin {
    fn id(&self) -> &str { "rpcs3" }
    fn name(&self) -> &str { "RPCS3 (PS3)" }
    fn supported_extensions(&self) -> &[&str] { &["iso", "pkg", "bin", "edat", "self", "sprx", "elf"] } // EBOOT.BIN handling might be tricky via launcher without valid folder structure, but .iso is standard for dumps.

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        if let Ok(path) = which::which("rpcs3") { return Ok(path); }
        if let Ok(path) = which::which("rpcs3.AppImage") { return Ok(path); }
        
        Err(anyhow::anyhow!("RPCS3 executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate RPCS3 binary")?;
        
        // RPCS3 Args: --no-gui <boot_path>
        let args = vec![
            "--no-gui".to_string(),
        ];

        Ok(LaunchConfig {
            emulator_path: binary,
            rom_path: rom_path.to_path_buf(),
            bios_path: None, // Firmware is usually installed globally in RPCS3, not passed per game
            args,
            working_dir: None, 
            env_vars: vec![],
        })
    }

    fn can_handle(&self, binary_path: &Path) -> bool {
        let name = binary_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        name.contains("rpcs3")
    }
}
