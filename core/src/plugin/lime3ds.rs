use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct Lime3DSPlugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl Lime3DSPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for Lime3DSPlugin {
    fn id(&self) -> &str { "lime3ds" }
    fn name(&self) -> &str { "Lime3DS (3DS)" }
    fn supported_extensions(&self) -> &[&str] { &["3ds", "cia", "cxi", "cci", "3dsx"] }

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        if let Ok(path) = which::which("lime3ds-cli") { return Ok(path); }
        if let Ok(path) = which::which("lime3ds-gui") { return Ok(path); }
        if let Ok(path) = which::which("lime3ds") { return Ok(path); }
        // Fallback for Citra users? Maybe later.
        
        Err(anyhow::anyhow!("Lime3DS executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate Lime3DS binary")?;
        
        // Lime3DS Args: <rom>
        // No complicate args needed usually.
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
        name.contains("lime3ds") || name.contains("citra")
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(Lime3DSPlugin::new(Some(binary_path)))
    }
}
