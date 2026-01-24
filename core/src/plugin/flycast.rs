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

    fn prepare_launch_config(&self, rom_path: &Path, output_dir: &Path) -> Result<LaunchConfig> {
        self.prepare_launch_config_with_specs(rom_path, output_dir, None, None)
    }
    
    fn prepare_launch_config_with_specs(
        &self,
        rom_path: &Path,
        _output_dir: &Path,
        host_specs: Option<crate::plugin::HostSpecs>,
        _progress: Option<&dyn Fn(String)>
    ) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate Flycast binary")?;
        
        let mut args = vec![];
        
        // --- Configuration Dynamique ---
        if let Some(specs) = host_specs {
            // 1. Renderer: Vulkan (4) ou OpenGL (0) - Section [config] Key pvr.rend
            let renderer = if specs.vulkan_support { "4" } else { "0" };
            args.push("-config".to_string());
            args.push(format!("config:pvr.rend={}", renderer));

            // 2. Fullscreen - Section [window] Key fullscreen
            args.push("-config".to_string());
            args.push("window:fullscreen=yes".to_string());
            
            // Résolution Fenêtre (si pas fullscreen ou pour la taille interne de la fenêtre)
            args.push("-config".to_string());
            args.push(format!("window:width={}", specs.screen_width));
            args.push("-config".to_string());
            args.push(format!("window:height={}", specs.screen_height));

            // 3. Scaling (Internal Resolution) - Section [config] Key rend.Resolution
            let scale_factor = if specs.screen_height >= 2160 {
                6 
            } else if specs.screen_height >= 1440 {
                4 
            } else if specs.screen_height >= 1080 {
                3 
            } else if specs.screen_height >= 720 {
                2 
            } else {
                1 
            };
            let internal_res = 480 * scale_factor;
            
            args.push("-config".to_string());
            args.push(format!("config:rend.Resolution={}", internal_res));

            // 4. Aspect Ratio (Widescreen) - Section [config] Key rend.WideScreen
            let ratio = specs.screen_width as f32 / specs.screen_height as f32;
            let widescreen = if ratio > 1.7 { "yes" } else { "no" };
            args.push("-config".to_string());
            args.push(format!("config:rend.WideScreen={}", widescreen));
            
            // 5. VSync - Section [config] Key rend.vsync
            args.push("-config".to_string());
            args.push("config:rend.vsync=yes".to_string());
        } else {
            // Fallback
            args.push("-config".to_string());
            args.push("window:fullscreen=yes".to_string());
        }

        Ok(LaunchConfig {
            emulator_path: binary,
            rom_path: rom_path.to_path_buf(),
            bios_path: None, 
            args, // Ces arguments seront passés AVANT le chemin de la ROM par défaut ? 
                  // LaunchConfig met `args` + `rom` + `args_after_rom`.
                  // Flycast accepte `flycast [options] rom`. C'est parfait.
            working_dir: None, 
            args_after_rom: vec![],
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
    
    // Flycast fullscreen argument via trait (backup, non utilisé si prepare_launch_config_with_specs fonctionne bien)
    fn fullscreen_args(&self) -> Vec<String> {
        vec!["-config".to_string(), "window:fullscreen=yes".to_string()]
    }
}
