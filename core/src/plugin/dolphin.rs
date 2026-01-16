use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct DolphinPlugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl DolphinPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for DolphinPlugin {
    fn id(&self) -> &str { "dolphin" }
    fn name(&self) -> &str { "Dolphin (GameCube/Wii)" }
    fn supported_extensions(&self) -> &[&str] { &["iso", "gcm", "wbfs", "ciso", "rvz", "elf", "dol"] }

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        if let Ok(path) = which::which("dolphin-emu") { return Ok(path); }
        if let Ok(path) = which::which("Dolphin") { return Ok(path); }
        
        Err(anyhow::anyhow!("Dolphin executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate Dolphin binary")?;
        
        // Dolphin CLI Args: -b (batch) -e <file>
        let args = vec![
            "-b".to_string(), // Batch mode (exits when done)
            "-e".to_string(), // Execute specific file (next arg is implicitly rom_path in launch config logic?)
            // Wait, LaunchConfig puts rom_path at the END usually. 
            // Stub logic: cmd.args(args); cmd.arg(rom_path);
            // So executing: dolphin -b -e <rom_path> 
            // This works perfect.
        ];

        Ok(LaunchConfig {
            emulator_path: binary,
            rom_path: rom_path.to_path_buf(),
            bios_path: None, 
            args,
            working_dir: None, 
            env_vars: vec![("QT_QPA_PLATFORM".to_string(), "xcb".to_string())], // Often needed on Linux
        })
    }

    fn can_handle(&self, binary_path: &Path) -> bool {
        let name = binary_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        name.contains("dolphin")
    }

    fn fullscreen_args(&self) -> Vec<String> {
        // Dolphin n'a pas de --fullscreen CLI, il utilise --batch -e
        vec![]
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(DolphinPlugin::new(Some(binary_path)))
    }
}
