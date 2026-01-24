use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct FlycastPlugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl FlycastPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for FlycastPlugin {
    fn id(&self) -> &str { "flycast" }
    fn name(&self) -> &str { "Flycast (Dreamcast)" }
    fn supported_extensions(&self) -> &[&str] { &["gdi", "cdi", "chd", "cue"] }

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        if let Ok(path) = which::which("flycast") { return Ok(path); }
        
        Err(anyhow::anyhow!("Flycast executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate Flycast binary")?;
        
        // Flycast Args: <rom> (works for AppImage/Binary)
        let args = vec![];

        Ok(LaunchConfig {
            emulator_path: binary,
            rom_path: rom_path.to_path_buf(),
            bios_path: None, 
            args,
            working_dir: None, 
            args_after_rom: vec![], // Flycast takes ROM as last arg usually, but args_after_rom works too
            env_vars: vec![],
        })
    }

    fn can_handle(&self, binary_path: &Path) -> bool {
        let name = binary_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        name.contains("flycast")
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(FlycastPlugin::new(Some(binary_path)))
    }
    
    // Flycast fullscreen argument
    fn fullscreen_args(&self) -> Vec<String> {
        // Flycast CLI options likely allow starting fullscreen, or it remembers config.
        // Recent Flycast accepts `flycast -config window:fullscreen=yes ROM` or similar?
        // Checking documentation: Flycast doesn't have a simple `--fullscreen` flag in all versions, 
        // but often respects config. Some sources say just launching is enough if config set.
        // However, standard intuitive flag might be missing. 
        // Let's assume user sets it in UI once, or we might need `emu.cfg` manipulation later.
        // For now, return empty or investigate CLI.
        // Found: `flycast "rom.gdi"` is standard.
        vec![]
    }
}
