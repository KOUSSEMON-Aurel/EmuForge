use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct DuckStationPlugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl DuckStationPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for DuckStationPlugin {
    fn id(&self) -> &str { "duckstation" }
    fn name(&self) -> &str { "DuckStation (PS1)" }
    fn supported_extensions(&self) -> &[&str] { &["bin", "cue", "iso", "chd", "m3u", "pbp"] }

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        if let Ok(path) = which::which("duckstation-qt-x64-ReleaseLTCG") { return Ok(path); }
        if let Ok(path) = which::which("duckstation-no-gui") { return Ok(path); }
        if let Ok(path) = which::which("duckstation") { return Ok(path); }
        
        Err(anyhow::anyhow!("DuckStation executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate DuckStation binary")?;
        
        // DuckStation CLI Args: [flags] -- <filename>
        // See: https://github.com/stenzek/duckstation
        let args = vec![
            "-fullscreen".to_string(),
            "--".to_string(),
        ];

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
        name.contains("duckstation")
    }
}
