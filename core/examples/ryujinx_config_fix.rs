// Petit utilitaire autonome pour mettre à jour la config Ryujinx
// À lancer avant le jeu

fn main() {
    println!("🔧 EmuForge Config Updater running...");
    
    // Code copié de ryujinx.rs update_ryujinx_input_config
    if let Err(e) = update_config() {
        eprintln!("❌ Error updating config: {:?}", e);
        std::process::exit(1);
    }
    
    println!("✅ Config updated successfully!");
}

fn update_config() -> anyhow::Result<()> {
    use std::fs;
    let config_dir = dirs::config_dir().map(|d| d.join("Ryujinx")).unwrap();
    let config_path = config_dir.join("Config.json");
    
    if !config_path.exists() {
        return Err(anyhow::anyhow!("Config not found at {:?}", config_path));
    }

    println!("Reading config from {:?}", config_path);
    let content = fs::read_to_string(&config_path)?;
    let mut json_config: serde_json::Value = serde_json::from_str(&content)?;

    // Generate Inputs
    let sdl = sdl2::init().map_err(|e| anyhow::anyhow!(e))?;
    let joystick_subsystem = sdl.joystick().map_err(|e| anyhow::anyhow!(e))?;
    let controller_subsystem = sdl.game_controller().map_err(|e| anyhow::anyhow!(e))?;
    
    let available = controller_subsystem.num_joysticks().unwrap_or(0);
    println!("Found {} controllers", available);

    let mut configs = Vec::new();
    let mut player_idx = 1;

    for i in 0..available {
        if let Ok(controller) = controller_subsystem.open(i) {
            if let Ok(joystick) = joystick_subsystem.open(i) {
                let name = controller.name();
                let guid = joystick.guid();
                
                // FORMULE CORRECTE
                 let hex_str: String = guid.raw().data.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect();
                
                let rearranged = format!(
                    "{}{}{}{}-{}{}-{}-{}-{}",
                    &hex_str[4..6], &hex_str[6..8], &hex_str[2..4], &hex_str[0..2],
                    &hex_str[10..12], &hex_str[8..10],
                    &hex_str[12..16], &hex_str[16..20], &hex_str[20..32]
                );
                let final_guid = format!("0000{}", &rearranged[4..]);
                let ryujinx_id = format!("{}-{}", i, final_guid);
                
                println!("Added {} as Player{}", name, player_idx);
                
                // Config JSON standard (simplifiée pour l'exemple, mais fonctionnelle)
                let config = serde_json::json!({
                    "left_joycon_stick": { "joystick": "Left", "stick_button": "LeftStick", "invert_stick_x": false, "invert_stick_y": false, "rotate90_cw": false },
                    "right_joycon_stick": { "joystick": "Right", "stick_button": "RightStick", "invert_stick_x": false, "invert_stick_y": false, "rotate90_cw": false },
                    "deadzone_left": 0.1, "deadzone_right": 0.1, "range_left": 1.0, "range_right": 1.0, "trigger_threshold": 0.5,
                    "motion": { "motion_backend": "GamepadDriver", "sensitivity": 100, "gyro_deadzone": 1.0, "enable_motion": true },
                    "rumble": { "strong_rumble": 1.0, "weak_rumble": 1.0, "enable_rumble": false }, // Rumble OFF pour éviter conflits
                     "led": { "enable_led": false, "turn_off_led": false, "use_rainbow": false, "led_color": 0 },
                    "left_joycon": { "button_minus": "Back", "button_l": "LeftShoulder", "button_zl": "LeftTrigger", "button_sl": "SingleLeftTrigger0", "button_sr": "SingleRightTrigger0", "dpad_up": "DpadUp", "dpad_down": "DpadDown", "dpad_left": "DpadLeft", "dpad_right": "DpadRight" },
                    "right_joycon": { "button_plus": "Start", "button_r": "RightShoulder", "button_zr": "RightTrigger", "button_sl": "SingleLeftTrigger1", "button_sr": "SingleRightTrigger1", "button_x": "Y", "button_b": "A", "button_y": "X", "button_a": "B" }, // Layout Xbox
                    "version": 1, "backend": "GamepadSDL2", "id": ryujinx_id, "name": name, "controller_type": "ProController", "player_index": format!("Player{}", player_idx)
                });
                configs.push(config);
                player_idx += 1;
            }
        }
    }
    
    // Add Keyboard if needed
    if player_idx <= 8 {
         let keyboard_config = serde_json::json!({
            "left_joycon_stick": { "stick_up": "W", "stick_down": "S", "stick_left": "A", "stick_right": "D", "stick_button": "F" },
            "right_joycon_stick": { "stick_up": "I", "stick_down": "K", "stick_left": "J", "stick_right": "L", "stick_button": "H" },
            "left_joycon": { "button_minus": "Minus", "button_l": "E", "button_zl": "Q", "button_sl": "Unbound", "button_sr": "Unbound", "dpad_up": "Up", "dpad_down": "Down", "dpad_left": "Left", "dpad_right": "Right" },
            "right_joycon": { "button_plus": "Plus", "button_r": "U", "button_zr": "O", "button_sl": "Unbound", "button_sr": "Unbound", "button_x": "C", "button_b": "X", "button_y": "V", "button_a": "Z" },
            "version": 1, "backend": "WindowKeyboard", "id": "0", "name": "All Keyboards", "controller_type": "ProController", "player_index": format!("Player{}", player_idx)
        });
        configs.push(keyboard_config);
    }

    json_config["input_config"] = serde_json::Value::Array(configs);
    
    fs::write(&config_path, serde_json::to_string_pretty(&json_config)?)?;
    Ok(())
}
