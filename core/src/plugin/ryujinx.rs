use crate::forge::LaunchConfig;
use crate::plugin::{EmulatorPlugin, RequirementInfo, ValidationResult};
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use zip::ZipArchive;

pub struct RyujinxPlugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl RyujinxPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }

    /// Scan recursively pour trouver prod.keys, title.keys, et fichiers .nca
    /// Extrait automatiquement les archives ZIP si nécessaire
    fn deep_scan_for_files(dir: &Path, temp_extract_dir: &Path) -> (Option<PathBuf>, Option<PathBuf>, Vec<PathBuf>) {
        let mut prod_keys: Option<PathBuf> = None;
        let mut title_keys: Option<PathBuf> = None;
        let mut nca_files: Vec<PathBuf> = Vec::new();

        Self::scan_directory_recursive(dir, temp_extract_dir, &mut prod_keys, &mut title_keys, &mut nca_files);
        
        (prod_keys, title_keys, nca_files)
    }

    fn scan_directory_recursive(
        dir: &Path, 
        temp_extract_dir: &Path,
        prod_keys: &mut Option<PathBuf>, 
        title_keys: &mut Option<PathBuf>, 
        nca_files: &mut Vec<PathBuf>
    ) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let name_lower = name.to_lowercase();
                
                // Fichiers clés
                if name == "prod.keys" && prod_keys.is_none() {
                    *prod_keys = Some(path.clone());
                    continue;
                }
                if name == "title.keys" && title_keys.is_none() {
                    *title_keys = Some(path.clone());
                    continue;
                }
                
                // Fichiers NCA (firmware)
                if name_lower.ends_with(".nca") {
                    nca_files.push(path.clone());
                    continue;
                }
                
                // Archives ZIP - Extraire et scanner
                if name_lower.ends_with(".zip") {
                    if let Ok(extracted) = Self::extract_zip_to_temp(&path, temp_extract_dir) {
                        Self::scan_directory_recursive(&extracted, temp_extract_dir, prod_keys, title_keys, nca_files);
                    }
                    continue;
                }
                
                // Sous-dossiers - Scanner récursivement
                if path.is_dir() {
                    Self::scan_directory_recursive(&path, temp_extract_dir, prod_keys, title_keys, nca_files);
                }
            }
        }
    }

    fn extract_zip_to_temp(zip_path: &Path, temp_dir: &Path) -> Result<PathBuf> {
        let file = fs::File::open(zip_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        // Créer un sous-dossier unique pour cette extraction
        let zip_name = zip_path.file_stem().and_then(|s| s.to_str()).unwrap_or("archive");
        let extract_dir = temp_dir.join(zip_name);
        fs::create_dir_all(&extract_dir)?;
        
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = match file.enclosed_name() {
                Some(path) => extract_dir.join(path),
                None => continue,
            };
            
            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    fs::create_dir_all(parent)?;
                }
                let mut outfile = fs::File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
        }
        
        Ok(extract_dir)
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
        
        let args = vec![];

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

    fn get_requirements(&self) -> RequirementInfo {
        RequirementInfo {
            needs_bios: true,
            needs_firmware: true,
            keys_file: Some("prod.keys".to_string()),
            description: "Ryujinx requires 'prod.keys' and Firmware (.nca files) to run Switch games.".to_string(),
        }
    }

    fn validate_requirements(&self, source_path: Option<&Path>) -> Result<ValidationResult> {
        // Target directories
        let ryujinx_config = if cfg!(target_os = "windows") {
            dirs::data_dir().map(|d| d.join("Ryujinx"))
        } else {
            dirs::config_dir().map(|d| d.join("Ryujinx"))
        }.ok_or_else(|| anyhow::anyhow!("Could not determine Ryujinx config directory"))?;

        let keys_dir = ryujinx_config.join("system");
        let firmware_dir = ryujinx_config.join("bis/system/Contents/registered");
        
        let keys_path = keys_dir.join("prod.keys");

        // Check if already satisfied (keys + firmware installed)
        let keys_exist = keys_path.exists();
        let firmware_exists = firmware_dir.exists() && 
            fs::read_dir(&firmware_dir).map(|d| d.count() > 0).unwrap_or(false);

        if keys_exist && firmware_exists && source_path.is_none() {
            return Ok(ValidationResult { 
                valid: true, 
                message: "Keys and Firmware already installed.".to_string(), 
                fixed: false 
            });
        }

        // Need source path to install
        let src = match source_path {
            Some(s) if s.exists() && s.is_dir() => s,
            Some(_) => return Ok(ValidationResult { 
                valid: false, 
                message: "Invalid source folder.".to_string(), 
                fixed: false 
            }),
            None if keys_exist => return Ok(ValidationResult { 
                valid: true, 
                message: "Keys found. Select folder with firmware for full setup.".to_string(), 
                fixed: false 
            }),
            None => return Ok(ValidationResult { 
                valid: false, 
                message: "Select folder containing keys and firmware.".to_string(), 
                fixed: false 
            }),
        };

        // Create temp directory for ZIP extraction
        let temp_dir = std::env::temp_dir().join(format!("emuforge_ryujinx_{}", std::process::id()));
        fs::create_dir_all(&temp_dir)?;

        // Deep scan with ZIP extraction
        let (prod_keys, title_keys, nca_files) = Self::deep_scan_for_files(src, &temp_dir);

        // Install keys
        let keys_installed = if let Some(pk) = prod_keys {
            fs::create_dir_all(&keys_dir)?;
            fs::copy(&pk, &keys_path)?;
            
            if let Some(tk) = title_keys {
                let _ = fs::copy(&tk, keys_dir.join("title.keys"));
            }
            true
        } else if keys_exist {
            true // Already had keys
        } else {
            false
        };

        // Install firmware NCAs
        let firmware_count = nca_files.len();
        let firmware_installed = if !nca_files.is_empty() {
            fs::create_dir_all(&firmware_dir)?;
            for nca in &nca_files {
                if let Some(name) = nca.file_name().and_then(|n| n.to_str()) {
                    // Ryujinx structure: registered/{filename}/00
                    let nca_dir = firmware_dir.join(name);
                    fs::create_dir_all(&nca_dir)?;
                    let dest = nca_dir.join("00");
                    let _ = fs::copy(nca, &dest);
                }
            }
            true
        } else {
            firmware_exists // Already had firmware
        };

        // Cleanup temp directory
        let _ = fs::remove_dir_all(&temp_dir);

        // Build result message
        let message = match (keys_installed, firmware_installed) {
            (true, true) => format!("✅ Success! Keys + {} firmware files installed. Ready to play!", firmware_count),
            (true, false) => "⚠️ Keys installed, but no firmware found. Add firmware folder.".to_string(),
            (false, true) => "⚠️ Firmware found, but missing prod.keys file.".to_string(),
            (false, false) => "❌ No keys or firmware found in folder.".to_string(),
        };

        Ok(ValidationResult { 
            valid: keys_installed && firmware_installed, 
            message, 
            fixed: keys_installed || firmware_installed 
        })
    }

    /// Prepare portable binary avec auto-installation firmware
    /// Extrait l'AppImage, ajoute le firmware, modifie le script de lancement
    fn prepare_portable_binary(
        &self,
        original_binary: &Path,
        bios_firmware_path: Option<&Path>,
        work_dir: &Path,
    ) -> Result<Option<PathBuf>> {
        // Skip if not an AppImage
        let name = original_binary.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        if !name.to_lowercase().contains("appimage") {
            return Ok(None);
        }

        // Skip if no firmware path provided
        let firmware_src = match bios_firmware_path {
            Some(p) if p.exists() => p,
            _ => return Ok(None),
        };

        println!("🔧 Patching Ryujinx AppImage for auto firmware install...");

        // 1. Extract AppImage
        let extract_dir = work_dir.join("ryujinx_appimage");
        fs::create_dir_all(&extract_dir)?;
        
        let status = std::process::Command::new(original_binary)
            .arg("--appimage-extract")
            .current_dir(&extract_dir)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .context("Failed to extract AppImage")?;

        if !status.success() {
            return Err(anyhow::anyhow!("AppImage extraction failed"));
        }

        let squashfs_dir = extract_dir.join("squashfs-root");
        if !squashfs_dir.exists() {
            return Err(anyhow::anyhow!("squashfs-root not found after extraction"));
        }

        // 2. Create firmware bundle directory in AppImage
        let firmware_bundle = squashfs_dir.join("usr/bin/emuforge_firmware");
        let firmware_bundle_nca = firmware_bundle.join("firmware");
        fs::create_dir_all(&firmware_bundle_nca)?;

        // 3. Deep scan source for keys and NCAs
        let temp_scan = work_dir.join("firmware_scan");
        fs::create_dir_all(&temp_scan)?;
        let (prod_keys, title_keys, nca_files) = Self::deep_scan_for_files(firmware_src, &temp_scan);

        // Copy keys to bundle
        if let Some(pk) = prod_keys {
            fs::copy(&pk, firmware_bundle.join("prod.keys"))?;
            println!("  📁 Bundled prod.keys");
        }
        if let Some(tk) = title_keys {
            fs::copy(&tk, firmware_bundle.join("title.keys"))?;
            println!("  📁 Bundled title.keys");
        }

        // Copy NCAs to bundle
        let nca_count = nca_files.len();
        for nca in &nca_files {
            if let Some(name) = nca.file_name().and_then(|n| n.to_str()) {
                // Ryujinx structure: registered/{filename}/00
                // We recreate this structure in the bundle so the script just copies folders
                let nca_dir = firmware_bundle_nca.join(name);
                fs::create_dir_all(&nca_dir)?;
                let _ = fs::copy(nca, nca_dir.join("00"));
            }
        }
        println!("  📁 Bundled {} firmware NCAs", nca_count);

        // Cleanup scan temp
        let _ = fs::remove_dir_all(&temp_scan);

        // 4. Modify Ryujinx.sh to auto-install firmware on first run
        let launch_script_path = squashfs_dir.join("usr/bin/Ryujinx.sh");
        let patched_script = r#"#!/bin/sh

SCRIPT_DIR=$(dirname "$(realpath "$0")")

# === EmuForge Auto-Firmware Install ===
FIRMWARE_SRC="$SCRIPT_DIR/emuforge_firmware"
TARGET_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/Ryujinx"

if [ -d "$FIRMWARE_SRC" ] && [ ! -f "$TARGET_DIR/.emuforge_installed" ]; then
    echo "[EmuForge] Installing keys and firmware..."
    mkdir -p "$TARGET_DIR/system"
    mkdir -p "$TARGET_DIR/bis/system/Contents/registered"
    
    # Copy keys
    [ -f "$FIRMWARE_SRC/prod.keys" ] && cp -n "$FIRMWARE_SRC/prod.keys" "$TARGET_DIR/system/" 2>/dev/null
    [ -f "$FIRMWARE_SRC/title.keys" ] && cp -n "$FIRMWARE_SRC/title.keys" "$TARGET_DIR/system/" 2>/dev/null
    
    # Copy firmware NCAs
    if [ -d "$FIRMWARE_SRC/firmware" ]; then
        cp -rn "$FIRMWARE_SRC/firmware/"* "$TARGET_DIR/bis/system/Contents/registered/" 2>/dev/null
    fi
    
    touch "$TARGET_DIR/.emuforge_installed"
    echo "[EmuForge] Installation complete!"
fi
# === End EmuForge ===

if [ -f "$SCRIPT_DIR/Ryujinx.Headless.SDL2" ]; then
    RYUJINX_BIN="Ryujinx.Headless.SDL2"
fi

if [ -f "$SCRIPT_DIR/Ryujinx" ]; then
    RYUJINX_BIN="Ryujinx"
fi

if [ -z "$RYUJINX_BIN" ]; then
    exit 1
fi

COMMAND="env LANG=C.UTF-8 DOTNET_EnableAlternateStackCheck=1"

if command -v gamemoderun > /dev/null 2>&1; then
    COMMAND="$COMMAND gamemoderun"
fi

exec $COMMAND "$SCRIPT_DIR/$RYUJINX_BIN" "$@"
"#;

        fs::write(&launch_script_path, patched_script)?;
        println!("  ✅ Patched Ryujinx.sh with auto-firmware script");

        // 5. Return the squashfs-root directory (will be bundled as a directory)
        // Make AppRun and scripts executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let app_run = squashfs_dir.join("AppRun");
            if app_run.exists() {
                let mut perms = fs::metadata(&app_run)?.permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&app_run, perms)?;
            }
            
            // Also ensure Ryujinx.sh is executable
            let mut perms2 = fs::metadata(&launch_script_path)?.permissions();
            perms2.set_mode(0o755);
            fs::set_permissions(&launch_script_path, perms2)?;
            
            // Make main binary executable
            let main_bin = squashfs_dir.join("usr/bin/Ryujinx");
            if main_bin.exists() {
                let mut perms3 = fs::metadata(&main_bin)?.permissions();
                perms3.set_mode(0o755);
                fs::set_permissions(&main_bin, perms3)?;
            }
        }

        println!("🎯 Ryujinx AppImage patched successfully!");
        
        // Return the directory, not the AppRun file
        Ok(Some(squashfs_dir))
    }
}

