use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct PpssppPlugin {
    /// Path to the executable provided by the user (optional override)
    pub custom_binary_path: Option<PathBuf>,
}

impl PpssppPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for PpssppPlugin {
    fn id(&self) -> &str {
        "ppsspp"
    }

    fn name(&self) -> &str {
        "PPSSPP (PSP Emulator)"
    }

    fn supported_extensions(&self) -> &[&str] {
        &["iso", "cso", "pbp", "elf"]
    }

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // Try standard Linux paths
        if let Ok(path) = which::which("ppsspp") {
            return Ok(path);
        }
        if let Ok(path) = which::which("PPSSPPQt") {
            return Ok(path);
        }
        if let Ok(path) = which::which("PPSSPPSDL") {
            return Ok(path);
        }

        // Try standard Windows paths
        // TODO: Add Windows registry checks or common path checks

        Err(anyhow::anyhow!("PPSSPP executable not found. Please specify manually."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate PPSSPP binary")?;
        
        // PPSSPP Arguments:
        // https://github.com/hrydgard/ppsspp/wiki/Command-Line-Arguments
        // <filename> : The ISO/CSO to load
        // --fullscreen : Start in fullscreen
        
        let args = vec![];

        // args.push("--fullscreen".to_string());

        
        // TODO: Handle config file injection if needed
        // args.push(format!("--config={}", output_dir.join("ppsspp.ini").to_string_lossy()));

        Ok(LaunchConfig {
            emulator_path: binary,
            rom_path: rom_path.to_path_buf(),
            bios_path: None, // PPSSPP doesn't typically require an external BIOS file passed as arg
            args,
            working_dir: None, 
            args_after_rom: vec![],
            env_vars: vec![],
        })
    }

    fn can_handle(&self, binary_path: &Path) -> bool {
        let name = binary_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        name.contains("ppsspp")
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(PpssppPlugin::new(Some(binary_path)))
    }
}

