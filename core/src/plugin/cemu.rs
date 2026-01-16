use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct CemuPlugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl CemuPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for CemuPlugin {
    fn id(&self) -> &str { "cemu" }
    fn name(&self) -> &str { "Cemu (Wii U)" }
    fn supported_extensions(&self) -> &[&str] { &["wua", "wud", "wux", "rpx", "elf"] } 

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        if let Ok(path) = which::which("cemu") { return Ok(path); }
        if let Ok(path) = which::which("Cemu") { return Ok(path); }
        
        Err(anyhow::anyhow!("Cemu executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate Cemu binary")?;
        
        // Cemu Args: -g <game_path> -f (fullscreen)
        let args = vec![
            "-g".to_string(),
        ];
        // Note: rom_path will be appended by Stub/LaunchConfig logic. 
        // Wait, Cemu requires -g BEFORE the path? 
        // LaunchConfig logic in stub: cmd.args(args).arg(rom_path).
        // This results in `cemu -g <rom_path>`. This is Correct.
        
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
        name.contains("cemu")
    }

    fn fullscreen_args(&self) -> Vec<String> {
        vec!["-f".to_string()]
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(CemuPlugin::new(Some(binary_path)))
    }
}
