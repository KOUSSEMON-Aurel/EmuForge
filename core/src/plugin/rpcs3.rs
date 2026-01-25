use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Télécharge appimagetool de manière synchrone via curl (évite les problèmes de runtime tokio)
fn download_appimagetool_sync() -> Result<PathBuf> {
    let tools_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("emuforge/tools");
    
    fs::create_dir_all(&tools_dir)?;
    
    let appimagetool_path = tools_dir.join("appimagetool-x86_64.AppImage");
    
    if appimagetool_path.exists() {
        println!("     ✅ appimagetool already available");
        return Ok(appimagetool_path);
    }
    
    println!("     ⬇️ Downloading appimagetool...");
    let url = "https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage";
    
    // Utiliser curl pour éviter les conflits de runtime tokio
    let output = std::process::Command::new("curl")
        .args(["-L", "-o", appimagetool_path.to_str().unwrap(), url])
        .output()
        .context("Failed to run curl for appimagetool download")?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "curl failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    
    // Rendre exécutable
    #[allow(unused_mut)]
    let mut perms = fs::metadata(&appimagetool_path)?.permissions();
    #[cfg(unix)]
    perms.set_mode(0o755);
    fs::set_permissions(&appimagetool_path, perms)?;
    
    println!("     ✅ appimagetool downloaded");
    Ok(appimagetool_path)
}

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
        if let Ok(path) = which::which("RPCS3.AppImage") { return Ok(path); }
        
        Err(anyhow::anyhow!("RPCS3 executable not found."))
    }

    fn setup_environment(&self, output_dir: &Path, _bios_path: Option<&Path>) -> Result<()> {
        // Create config directory structure: XDG_CONFIG_HOME/rpcs3/
        let config_dir = output_dir.join("rpcs3");
        std::fs::create_dir_all(&config_dir).context("Failed to create RPCS3 config dir")?;

        // Write minimal config.yml to skip Welcome Screen
        // Strategy: Create minimal config.yml AND CurrentSettings.ini
        let config_content = r#"
GuiSettings:
    TIMEF: 1
    current_language: en
    welcome_screen_shown: true
"#;
        let config_path = config_dir.join("config.yml");
        if !config_path.exists() {
            std::fs::write(&config_path, config_content).context("Failed to write RPCS3 config.yml")?;
        }

        // Also write GuiConfigs/CurrentSettings.ini which is often the robust check
        let gui_configs_dir = config_dir.join("GuiConfigs");
        std::fs::create_dir_all(&gui_configs_dir).context("Failed to create GuiConfigs dir")?;
        
        let current_settings_content = r#"[main_window]
show_setup_wizard=false
"#;
        let settings_path = gui_configs_dir.join("CurrentSettings.ini");
        if !settings_path.exists() {
            std::fs::write(&settings_path, current_settings_content).context("Failed to write CurrentSettings.ini")?;
        }

        Ok(())
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate RPCS3 binary")?;
        
        // RPCS3 Args: Just the boot path
        let args = vec![];
        
        // We need to ensure XDG_CONFIG_HOME points to our portable structure.
        // The stub handles XDG_CONFIG_HOME = extraction_dir/config
        // So we create config/rpcs3/config.yml.
        // The stub sets XDG_CONFIG_HOME to {extraction}/config.
        // So RPCS3 sees {extraction}/config/rpcs3 (standard xdg path).
        
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
        name.contains("rpcs3")
    }

    fn fullscreen_args(&self) -> Vec<String> {
        // --no-gui pour que RPCS3 se ferme quand le jeu se termine
        vec!["--no-gui".to_string()]
    }
    
    fn portable_launch_args(&self, _fullscreen: bool) -> (Vec<String>, Vec<String>) {
        // RPCS3 utilise --no-gui pour le mode headless (pas de GUI, pas de wizard)
        // Le ROM path va après --
        let before = vec![
            "--no-gui".to_string(),
        ];
        let after = vec![];
        (before, after)
    }
    
    fn prepare_portable_binary(
        &self,
        original_binary: &Path,
        bios_firmware_path: Option<&Path>,
        work_dir: &Path,
    ) -> Result<Option<PathBuf>> {
        // Vérifier si un firmware PS3 est fourni
        if let Some(fw_path) = bios_firmware_path {
            if fw_path.extension().and_then(|s| s.to_str()) == Some("PUP") {
                println!("🔧 PS3 Firmware detected, patching RPCS3 AppImage...");
                
                // 1. Extraire le firmware depuis le PUP
                println!("📦 Extracting PS3 firmware from PUP...");
                let dev_flash = crate::firmware::ps3::extract_firmware(fw_path, work_dir)?;
                
                // 2. Télécharger appimagetool si nécessaire (sync version)
                println!("🔨 Preparing appimagetool...");
                let appimagetool = download_appimagetool_sync()?;
                
                // 3. Patcher AppImage (injection dev_flash + configs)
                println!("⚙️ Patching RPCS3 AppImage...");
                let patcher = crate::appimage::patcher::AppImagePatcher::new(appimagetool);
                let patched = patcher.patch_rpcs3(original_binary, &dev_flash, work_dir)?;
                
                // 4. Cleanup temp extract dir
                let _ = std::fs::remove_dir_all(dev_flash.parent().unwrap_or(&dev_flash));
                
                println!("✅ RPCS3 AppImage patched successfully!");
                return Ok(Some(patched));
            }
        }
        
        // Pas de firmware fourni ou pas un .PUP
        Ok(None)
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(Rpcs3Plugin::new(Some(binary_path)))
    }
}
