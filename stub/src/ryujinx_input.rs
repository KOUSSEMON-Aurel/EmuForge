// Module de détection dynamique des manettes pour Ryujinx
// Adapté de core/src/plugin/ryujinx.rs pour être utilisé au runtime dans le stub

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

// === Structures de Configuration Input ===

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum InputConfig {
    StandardKeyboardInputConfig(StandardKeyboardInputConfig),
    StandardControllerInputConfig(StandardControllerInputConfig),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StandardKeyboardInputConfig {
    pub version: u32,
    pub backend: String,
    pub id: String,
    pub name: String,
    pub player_index: String,
    pub controller_type: String,
    pub left_joycon: KeyboardJoyconConfig,
    pub left_joycon_stick: KeyboardStickConfig,
    pub right_joycon: KeyboardJoyconConfig,
    pub right_joycon_stick: KeyboardStickConfig,
    pub deadzone_left: f32,
    pub deadzone_right: f32,
    pub range_left: f32,
    pub range_right: f32,
    pub trigger_threshold: f32,
    pub motion: MotionConfig,
    pub rumble: RumbleConfig,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StandardControllerInputConfig {
    // Ordre des champs important pour la sérialisation JSON Ryujinx
    pub left_joycon_stick: ControllerStickConfig,
    pub right_joycon_stick: ControllerStickConfig,
    pub deadzone_left: f32,
    pub deadzone_right: f32,
    pub range_left: f32,
    pub range_right: f32,
    pub trigger_threshold: f32,
    pub motion: MotionConfig,
    pub rumble: RumbleConfig,
    pub led: LedConfig,
    pub left_joycon: ControllerJoyconConfig,
    pub right_joycon: ControllerJoyconConfig,
    pub version: u32,
    pub backend: String,
    pub id: String,
    pub name: String,
    pub controller_type: String,
    pub player_index: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LedConfig {
    pub enable_led: bool,
    pub turn_off_led: bool,
    pub use_rainbow: bool,
    pub led_color: u32,
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
    pub joystick: String,
    pub stick_button: String,
    pub invert_stick_x: bool,
    pub invert_stick_y: bool,
    pub rotate90_cw: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MotionConfig {
    pub motion_backend: String,
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

// === Fonctions de génération de configuration ===

/// Crée une configuration clavier par défaut pour le joueur spécifié
fn create_default_keyboard_config(player_index: &str) -> StandardKeyboardInputConfig {
    StandardKeyboardInputConfig {
        version: 1,
        backend: "WindowKeyboard".to_string(),
        id: "0".to_string(),
        name: "All Keyboards".to_string(),
        player_index: player_index.to_string(),
        controller_type: "ProController".to_string(),
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
            button_a: None, button_b: None, button_x: None, button_y: None, 
            button_plus: None, button_r: None, button_zr: None,
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
        deadzone_left: 0.1,
        deadzone_right: 0.1,
        range_left: 1.0,
        range_right: 1.0,
        trigger_threshold: 0.5,
        motion: MotionConfig {
            motion_backend: "GamepadDriver".to_string(),
            enable_motion: true,
            sensitivity: 100,
            gyro_deadzone: 1.0,
        },
        rumble: RumbleConfig {
            strong_rumble: 1.0,
            weak_rumble: 1.0,
            enable_rumble: false,
        },
    }
}

/// Détecte les manettes SDL2 et génère les configurations d'entrée
pub fn generate_input_config() -> Vec<InputConfig> {
    // Exécuter SDL2 dans un thread séparé pour éviter les problèmes de mémoire
    let handle = std::thread::spawn(|| {
        let mut configs = Vec::new();
        let mut player_idx_counter = 1;

        // 1. Scanner les contrôleurs SDL2
        if let Ok(sdl_context) = sdl2::init() {
            if let Ok(game_controller_subsystem) = sdl_context.game_controller() {
                let available_joysticks = game_controller_subsystem.num_joysticks().unwrap_or(0);
                
                for i in 0..available_joysticks {
                    if let Ok(controller) = game_controller_subsystem.open(i) {
                        // Max 8 joueurs
                        if player_idx_counter > 8 { break; }

                        let name = controller.name();
                        
                        // Génération GUID authentique Ryujinx (depuis SDL3GamepadDriver.cs)
                        let joystick = match sdl_context.joystick().unwrap().open(i) {
                            Ok(j) => j,
                            Err(_) => {
                                eprintln!("⚠️  Impossible d'ouvrir le joystick {} pour le GUID", i);
                                continue;
                            }
                        };
                        
                        let guid = joystick.guid();
                        let guid_bytes = guid.raw();
                        
                        // Convertir en chaîne hex 32 caractères
                        let hex_str: String = guid_bytes.data.iter()
                            .map(|b| format!("{:02x}", b))
                            .collect();
                        
                        // Réarrangement SDLGuidToString de Ryujinx
                        let rearranged = format!(
                            "{}{}{}{}-{}{}-{}-{}-{}",
                            &hex_str[4..6], &hex_str[6..8], &hex_str[2..4], &hex_str[0..2],
                            &hex_str[10..12], &hex_str[8..10],
                            &hex_str[12..16],
                            &hex_str[16..20],
                            &hex_str[20..32]
                        );
                        
                        // Supprimer CRC, ajouter "0000"
                        let final_guid = format!("0000{}", &rearranged[4..]);
                        let ryujinx_id = format!("{}-{}", i, final_guid);
                        eprintln!("🎮 Manette {}: {} → {}", i, name, ryujinx_id);
                        
                        let is_nintendo = name.to_lowercase().contains("nintendo");
                        let player_enum = format!("Player{}", player_idx_counter);

                        configs.push(InputConfig::StandardControllerInputConfig(StandardControllerInputConfig {
                            version: 1,
                            backend: "GamepadSDL2".to_string(),
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
                                button_minus: "Back".to_string(),
                                button_l: "LeftShoulder".to_string(),
                                button_zl: "LeftTrigger".to_string(),
                                button_sl: "SingleLeftTrigger0".to_string(),
                                button_sr: "SingleRightTrigger0".to_string(),
                                button_a: None, button_b: None, button_x: None, button_y: None,
                                button_plus: None, button_r: None, button_zr: None,
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
                                button_a: Some(if is_nintendo { "A" } else { "B" }.to_string()),
                                button_b: Some(if is_nintendo { "B" } else { "A" }.to_string()),
                                button_x: Some(if is_nintendo { "X" } else { "Y" }.to_string()),
                                button_y: Some(if is_nintendo { "Y" } else { "X" }.to_string()),
                                button_plus: Some("Start".to_string()),
                                button_r: Some("RightShoulder".to_string()),
                                button_zr: Some("RightTrigger".to_string()),
                                button_sl: "SingleLeftTrigger1".to_string(),
                                button_sr: "SingleRightTrigger1".to_string(),
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
                                enable_rumble: false,
                            },
                            led: LedConfig {
                                enable_led: false,
                                turn_off_led: false,
                                use_rainbow: false,
                                led_color: 0,
                            },
                        }));
                        player_idx_counter += 1;
                    }
                }
            }
        }

        // 2. Ajouter le clavier au slot disponible suivant (ou Player1 si pas de manettes)
        if player_idx_counter <= 8 {
            let keyboard_player = format!("Player{}", player_idx_counter);
            configs.push(InputConfig::StandardKeyboardInputConfig(
                create_default_keyboard_config(&keyboard_player)
            ));
        }
        
        configs
    });

    handle.join().unwrap_or_else(|_| Vec::new())
}

/// Met à jour la configuration d'entrée Ryujinx au chemin système par défaut
pub fn update_ryujinx_input_config() -> Result<(), String> {
    let config_dir = if cfg!(target_os = "windows") {
        dirs::data_dir().map(|d| d.join("Ryujinx"))
    } else {
        dirs::config_dir().map(|d| d.join("Ryujinx"))
    }.ok_or_else(|| "Impossible de déterminer le répertoire de config Ryujinx".to_string())?;

    update_ryujinx_input_config_at_path(&config_dir)
}

/// Met à jour la configuration d'entrée Ryujinx à un chemin spécifique
pub fn update_ryujinx_input_config_at_path(config_dir: &Path) -> Result<(), String> {
    // Créer le dossier config si nécessaire
    if !config_dir.exists() {
        fs::create_dir_all(config_dir)
            .map_err(|e| format!("Erreur création dossier config: {}", e))?;
        eprintln!("📁 Dossier Ryujinx créé: {:?}", config_dir);
    }
    
    let config_path = config_dir.join("Config.json");
    eprintln!("🔧 Mise à jour config Ryujinx: {:?}", config_path);
    
    // Générer la nouvelle config d'entrée
    let new_input_config = generate_input_config();
    
    if config_path.exists() {
        // Config existe: mettre à jour uniquement input_config
        let content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Erreur lecture Config.json: {}", e))?;
        
        // Parser en JSON générique pour préserver les autres paramètres
        let mut json_config: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("Erreur parsing Config.json: {}", e))?;
        
        // Mettre à jour le champ "input_config"
        json_config["input_config"] = serde_json::to_value(&new_input_config)
            .map_err(|e| format!("Erreur sérialisation: {}", e))?;
        
        // Écrire le résultat
        let new_content = serde_json::to_string_pretty(&json_config)
            .map_err(|e| format!("Erreur stringify: {}", e))?;
        fs::write(&config_path, new_content)
            .map_err(|e| format!("Erreur écriture: {}", e))?;
        
        eprintln!("✅ InputConfig Ryujinx mis à jour avec les manettes détectées.");
    } else {
        // Config n'existe pas: créer un fichier minimal avec input_config
        eprintln!("📝 Config.json absent, création d'un fichier minimal...");
        
        let minimal_config = serde_json::json!({
            "input_config": new_input_config
        });
        
        let new_content = serde_json::to_string_pretty(&minimal_config)
            .map_err(|e| format!("Erreur stringify: {}", e))?;
        fs::write(&config_path, new_content)
            .map_err(|e| format!("Erreur écriture: {}", e))?;
        
        eprintln!("✅ Config.json minimal créé avec les manettes détectées.");
    }
    
    // Log du résultat
    if !new_input_config.is_empty() {
        if let InputConfig::StandardControllerInputConfig(ref ctrl) = new_input_config[0] {
            eprintln!("✅ Config Manette Sauvegardée → ID: {} | Nom: {}", ctrl.id, ctrl.name);
        } else if let InputConfig::StandardKeyboardInputConfig(ref kb) = new_input_config[0] {
            eprintln!("✅ Config Clavier Sauvegardée → ID: {} | Nom: {}", kb.id, kb.name);
        }
    }
    
    Ok(())
}

/// Récupère le GUID SDL brut de la première manette connectée (pour Azahar/Citra)
pub fn get_first_controller_guid() -> Option<String> {
    // Exécuter dans un thread comme pour generate_input_config
    std::thread::spawn(|| {
        if let Ok(sdl_context) = sdl2::init() {
            if let Ok(game_controller_subsystem) = sdl_context.game_controller() {
                if let Ok(joystick_subsystem) = sdl_context.joystick() {
                    let available_joysticks = game_controller_subsystem.num_joysticks().unwrap_or(0);
                    if available_joysticks > 0 {
                        // Ouvrir le premier joystick
                        if let Ok(joystick) = joystick_subsystem.open(0) {
                            let guid = joystick.guid();
                            let guid_bytes = guid.raw();
                            
                            // Retourner la chaîne hex brute (standard SDL)
                            let hex_str: String = guid_bytes.data.iter()
                                .map(|b| format!("{:02x}", b))
                                .collect();
                            return Some(hex_str);
                        }
                    }
                }
            }
        }
        None
    }).join().unwrap_or(None)
}
