// Test GUID avec la VRAIE formule Ryujinx corrigée

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let joystick_subsystem = sdl_context.joystick().unwrap();
    let game_controller_subsystem = sdl_context.game_controller().unwrap();
    
    let available = game_controller_subsystem.num_joysticks().unwrap_or(0);
    
    println!("Found {} controllers", available);
    
    for i in 0..available {
        if let Ok(_controller) = game_controller_subsystem.open(i) {
            if let Ok(joystick) = joystick_subsystem.open(i) {
                let guid = joystick.guid();
                let guid_bytes = guid.raw();
                
                // ÉTAPE 1: Convertir les 16 bytes en hex string (32 chars)
                let hex_str: String = guid_bytes.data.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect();
                
                println!("\n=== Controller {} ===", i);
                println!("Raw bytes: {:?}", guid_bytes.data);
                println!("Hex string (32 chars): {}", hex_str);
                
                // ÉTAPE 2: Appliquer SDLGuidToString de Ryujinx (ligne 68)
                // C#: $"{strGuid[4..6]}{strGuid[6..8]}{strGuid[2..4]}{strGuid[0..2]}-..."
                // strGuid indices sont des positions de CARACTÈRES dans le hex string !
                let sdl_guid_string = format!(
                    "{}{}{}{}-{}{}-{}-{}-{}",
                    &hex_str[4..6], &hex_str[6..8], &hex_str[2..4], &hex_str[0..2],  // Premier DWORD inversé
                    &hex_str[10..12], &hex_str[8..10],  // Deuxième WORD inversé
                    &hex_str[12..16],  // Troisième WORD tel quel
                    &hex_str[16..20],  // Quatrième WORD tel quel
                    &hex_str[20..32]   // Reste tel quel
                );
                println!("After SDLGuidToString: {}", sdl_guid_string);
                
                // ÉTAPE 3: Remplacer les 4 premiers caractères par "0000" (ligne 86)
                // guid.ToString()[4..] enlève les 4 premiers chars
                let final_guid = format!("0000{}", &sdl_guid_string[4..]);
                println!("After CRC removal:     {}", final_guid);
                
                // ÉTAPE 4: Ajouter l'index
                let ryujinx_id = format!("{}-{}", i, final_guid);
                println!("Final Ryujinx ID:      {}", ryujinx_id);
                
                println!("\nExpected (GUI):        0-00000003-045e-0000-8e02-000010010000");
                println!("Match: {}", ryujinx_id == "0-00000003-045e-0000-8e02-000010010000");
            }
        }
    }
}
