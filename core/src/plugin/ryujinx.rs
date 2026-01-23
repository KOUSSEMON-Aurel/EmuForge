use crate::forge::LaunchConfig;
use crate::plugin::{EmulatorPlugin, RequirementInfo, ValidationResult};
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::fs;
use zip::ZipArchive;
use serde::{Serialize, Deserialize};


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
    // === Input Configuration Logic ===

    fn create_default_keyboard_config(player_index: &str) -> StandardKeyboardInputConfig {
        StandardKeyboardInputConfig {
            version: 1,
            backend: "WindowKeyboard".to_string(),
            id: "0".to_string(),
            name: "All Keyboards".to_string(), // Standard Ryujinx keyboard name
            player_index: player_index.to_string(),
            controller_type: "ProController".to_string(), // ProController is better default than JoyconPair for keyboard
            left_joycon: KeyboardJoyconConfig {
                dpad_up: "Up".to_string(),
                dpad_down: "Down".to_string(),
                dpad_left: "Left".to_string(),
                dpad_right: "Right".to_string(),
                button_minus: "Minus".to_string(),
                button_l: "E".to_string(),
                button_zl: "Q".to_string(),
                button_sl: "Unbound".to_string(),
                button_sr: "Unbound".to_string(),
                button_a: None, button_b: None, button_x: None, button_y: None, button_plus: None, button_r: None, button_zr: None,
            },
            left_joycon_stick: KeyboardStickConfig {
                stick_up: "W".to_string(),
                stick_down: "S".to_string(),
                stick_left: "A".to_string(),
                stick_right: "D".to_string(),
                stick_button: "F".to_string(),
            },
            right_joycon: KeyboardJoyconConfig {
                dpad_up: "Unbound".to_string(),
                dpad_down: "Unbound".to_string(),
                dpad_left: "Unbound".to_string(),
                dpad_right: "Unbound".to_string(),
                button_minus: "Unbound".to_string(),
                button_l: "Unbound".to_string(),
                button_zl: "Unbound".to_string(),
                button_sl: "Unbound".to_string(),
                button_sr: "Unbound".to_string(),
                button_a: Some("Z".to_string()),
                button_b: Some("X".to_string()),
                button_x: Some("C".to_string()),
                button_y: Some("V".to_string()),
                button_plus: Some("Plus".to_string()),
                button_r: Some("U".to_string()),
                button_zr: Some("O".to_string()),
            },
            right_joycon_stick: KeyboardStickConfig {
                stick_up: "I".to_string(),
                stick_down: "K".to_string(),
                stick_left: "J".to_string(),
                stick_right: "L".to_string(),
                stick_button: "H".to_string(),
            },
        }
    }

    fn generate_input_config() -> Vec<InputConfig> {
        let mut configs = Vec::new();
        let mut player_idx_counter = 1;

        // 1. Scan SDL2 Controllers
        if let Ok(sdl_context) = sdl2::init() {
            if let Ok(game_controller_subsystem) = sdl_context.game_controller() {
                let available_joysticks = game_controller_subsystem.num_joysticks().unwrap_or(0);
                
                for i in 0..available_joysticks {
                    if let Ok(controller) = game_controller_subsystem.open(i) {
                         // Max 8 players
                         if player_idx_counter > 8 { break; }

                        let name = controller.name();
                        
                        // Fix: Ryujinx expects a standard SDL2 GUID string (hex).
                        // ...
                        let mapping = controller.mapping();
                        let raw_guid = mapping.split(',').next().unwrap_or("0").to_string();

                        // Format GUID with dashes for Ryujinx: 8-4-4-4-12
                        // Input:  030000005e0400008e02000009010000
                        // Output: 03000000-5e04-0000-8e02-000009010000
                        let formatted_guid = if raw_guid.len() == 32 {
                            format!("{}-{}-{}-{}-{}", 
                                &raw_guid[0..8], &raw_guid[8..12], &raw_guid[12..16], &raw_guid[16..20], &raw_guid[20..32])
                        } else {
                            raw_guid.clone()
                        };

                        // Ryujinx ID format: "{index}-{formatted_guid}"
                        let ryujinx_id = format!("{}-{}", i, formatted_guid);

                        println!("🎮 Found Controller: '{}' (Raw GUID: {}, Final ID: {})", name, raw_guid, ryujinx_id); 
                        
                        let is_nintendo = name.to_lowercase().contains("nintendo");
                        
                        let player_enum = format!("Player{}", player_idx_counter);
                        
                        // Determine controller backend - SDL2 is standard for linux
                        let backend = "GamepadSDL2".to_string();

                        configs.push(InputConfig::StandardControllerInputConfig(StandardControllerInputConfig {
                            version: 1,
                            backend: backend,
                            id: ryujinx_id,
                            name: name,
                            player_index: player_enum,
                            controller_type: "ProController".to_string(),
                            deadzone_left: 0.1,
                            deadzone_right: 0.1,
                            range_left: 1.0,
                            range_right: 1.0,
                            trigger_threshold: 0.5,
                            left_joycon: ControllerJoyconConfig {
                                dpad_up: "DpadUp".to_string(),
                                dpad_down: "DpadDown".to_string(),
                                dpad_left: "DpadLeft".to_string(),
                                dpad_right: "DpadRight".to_string(),
                                button_minus: "Minus".to_string(),
                                button_l: "LeftShoulder".to_string(),
                                button_zl: "LeftTrigger".to_string(),
                                button_sl: "SingleLeftTrigger0".to_string(),
                                button_sr: "SingleRightTrigger0".to_string(),
                                button_a: None, button_b: None, button_x: None, button_y: None, button_plus: None, button_r: None, button_zr: None,
                            },
                            left_joycon_stick: ControllerStickConfig {
                                joystick: "Left".to_string(),
                                stick_button: "LeftStick".to_string(),
                                invert_stick_x: false,
                                invert_stick_y: false,
                                rotate90_cw: false,
                            },
                            right_joycon: ControllerJoyconConfig {
                                dpad_up: "Unbound".to_string(),
                                dpad_down: "Unbound".to_string(),
                                dpad_left: "Unbound".to_string(),
                                dpad_right: "Unbound".to_string(),
                                button_minus: "Unbound".to_string(),
                                button_l: "Unbound".to_string(),
                                button_zl: "Unbound".to_string(),
                                button_sl: "Unbound".to_string(),
                                button_sr: "Unbound".to_string(),
                                button_a: Some(if is_nintendo { "A" } else { "B" }.to_string()),
                                button_b: Some(if is_nintendo { "B" } else { "A" }.to_string()),
                                button_x: Some(if is_nintendo { "X" } else { "Y" }.to_string()),
                                button_y: Some(if is_nintendo { "Y" } else { "X" }.to_string()),
                                button_plus: Some("Plus".to_string()),
                                button_r: Some("RightShoulder".to_string()),
                                button_zr: Some("RightTrigger".to_string()),
                            },
                            right_joycon_stick: ControllerStickConfig {
                                joystick: "Right".to_string(),
                                stick_button: "RightStick".to_string(),
                                invert_stick_x: false,
                                invert_stick_y: false,
                                rotate90_cw: false,
                            },
                            motion: MotionConfig {
                                motion_backend: "GamepadDriver".to_string(),
                                enable_motion: true,
                                sensitivity: 100,
                                gyro_deadzone: 1.0,
                            },
                            rumble: RumbleConfig {
                                strong_rumble: 1.0,
                                weak_rumble: 1.0,
                                enable_rumble: true,
                            },
                        }));
                        player_idx_counter += 1;
                    }
                }
            }
        }

        // 2. Assign Keyboard to next available slot (or Player1 if no controllers)
        // If we have controller(s), keyboard goes to next slot (e.g. Player2)
        // If no controllers, keyboard is Player1.
        if player_idx_counter <= 8 {
             let keyboard_player = format!("Player{}", player_idx_counter);
             configs.push(InputConfig::StandardKeyboardInputConfig(
                Self::create_default_keyboard_config(&keyboard_player)
            ));
        }
        
        configs
    }

    fn update_ryujinx_input_config() -> Result<()> {
        let config_dir = dirs::config_dir().ok_or(anyhow::anyhow!("No config dir"))?.join("Ryujinx");
        let config_path = config_dir.join("Config.json");
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let mut json: serde_json::Value = serde_json::from_str(&content)?;
            
            // Generate Input Configs
            let new_input_configs = Self::generate_input_config();
            
            // Convert to Value using serde
            let input_config_value = serde_json::to_value(new_input_configs)?;
            
            // Update the InputConfig field at root level
            if let Some(obj) = json.as_object_mut() {
                obj.insert("InputConfig".to_string(), input_config_value);
                
                // Also ensure EnableDockedMode is true for best visuals
                // obj.insert("EnableDockedMode".to_string(), serde_json::Value::Bool(true));
            }
            
            fs::write(config_path, serde_json::to_string_pretty(&json)?)?;
            println!("🎮 Updated Ryujinx InputConfig with detected controllers.");
        } else {
             println!("⚠️ Ryujinx Config.json not found, skipping input auto-config.");
        }
        Ok(())
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
                    
                    // Fix: Remove existing file if it conflicts with the new directory structure
                    // This handles migration from old flat-file installs
                    if nca_dir.exists() && nca_dir.is_file() {
                        let _ = fs::remove_file(&nca_dir);
                    }
                    
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

if [ -d "$FIRMWARE_SRC" ] && [ ! -f "$TARGET_DIR/.emuforge_fw_v2" ]; then
    echo "[EmuForge] Installing keys and firmware..."
    mkdir -p "$TARGET_DIR/system"
    mkdir -p "$TARGET_DIR/bis/system/Contents/registered"
    
    # Copy keys (Force overwrite to ensure compatibility with new firmware)
    echo "[EmuForge] Updating keys..."
    [ -f "$FIRMWARE_SRC/prod.keys" ] && cp "$FIRMWARE_SRC/prod.keys" "$TARGET_DIR/system/" 2>/dev/null
    [ -f "$FIRMWARE_SRC/title.keys" ] && cp "$FIRMWARE_SRC/title.keys" "$TARGET_DIR/system/" 2>/dev/null
    
    # Copy firmware NCAs
    if [ -d "$FIRMWARE_SRC/firmware" ]; then
        # Safety: Clear target registered folder to avoid file/dir conflicts (migration fix)
        # Only if we are about to install new ones
        rm -rf "$TARGET_DIR/bis/system/Contents/registered/"* 2>/dev/null
        cp -r "$FIRMWARE_SRC/firmware/"* "$TARGET_DIR/bis/system/Contents/registered/" 2>/dev/null
    fi
    
    touch "$TARGET_DIR/.emuforge_fw_v2"
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

        // Update input configuration (Auto-detect controllers)
        Self::update_ryujinx_input_config().ok();

        println!("🎯 Ryujinx AppImage patched successfully!");
        
        // Return the directory, not the AppRun file
        Ok(Some(squashfs_dir))
    }
}

// === Input Configuration Structures ===

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum InputConfig {
    StandardKeyboardInputConfig(StandardKeyboardInputConfig),
    StandardControllerInputConfig(StandardControllerInputConfig),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StandardKeyboardInputConfig {
    pub version: u32,
    pub backend: String, // "WindowKeyboard"
    pub id: String,      // "0"
    pub name: String,    // Added
    pub player_index: String,
    pub controller_type: String,
    pub left_joycon: KeyboardJoyconConfig,
    pub left_joycon_stick: KeyboardStickConfig,
    pub right_joycon: KeyboardJoyconConfig,
    pub right_joycon_stick: KeyboardStickConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StandardControllerInputConfig {
    pub version: u32,
    pub backend: String, // "GamepadSDL2"
    pub id: String,      // GUID
    pub name: String,    // Added
    pub player_index: String,
    pub controller_type: String,
    pub deadzone_left: f32,
    pub deadzone_right: f32,
    pub range_left: f32,
    pub range_right: f32,
    pub trigger_threshold: f32,
    pub left_joycon: ControllerJoyconConfig,
    pub left_joycon_stick: ControllerStickConfig,
    pub right_joycon: ControllerJoyconConfig,
    pub right_joycon_stick: ControllerStickConfig,
    pub motion: MotionConfig,
    pub rumble: RumbleConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyboardJoyconConfig {
    pub dpad_up: String,
    pub dpad_down: String,
    pub dpad_left: String,
    pub dpad_right: String,
    pub button_minus: String,
    pub button_l: String,
    pub button_zl: String,
    pub button_sl: String,
    pub button_sr: String,
    // Right Joycon specific
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_a: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_b: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_plus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_r: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_zr: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct KeyboardStickConfig {
    pub stick_up: String,
    pub stick_down: String,
    pub stick_left: String,
    pub stick_right: String,
    pub stick_button: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ControllerJoyconConfig {
    pub dpad_up: String,
    pub dpad_down: String,
    pub dpad_left: String,
    pub dpad_right: String,
    pub button_minus: String,
    pub button_l: String,
    pub button_zl: String,
    pub button_sl: String,
    pub button_sr: String,
    // Right Joycon specific
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_a: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_b: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_x: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_y: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_plus: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_r: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub button_zr: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ControllerStickConfig {
    pub joystick: String, // "Left" or "Right"
    pub stick_button: String,
    pub invert_stick_x: bool,
    pub invert_stick_y: bool,
    pub rotate90_cw: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MotionConfig {
    pub motion_backend: String, // "GamepadDriver"
    pub enable_motion: bool,
    pub sensitivity: i32,
    pub gyro_deadzone: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RumbleConfig {
    pub strong_rumble: f32,
    pub weak_rumble: f32,
    pub enable_rumble: bool,
}

