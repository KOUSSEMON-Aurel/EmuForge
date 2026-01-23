// Test script pour la génération GUID Ryujinx
use sdl2;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let joystick_subsystem = sdl_context.joystick().unwrap();
    let game_controller_subsystem = sdl_context.game_controller().unwrap();
    
    let available = game_controller_subsystem.num_joysticks().unwrap_or(0);
    
    println!("Found {} controllers", available);
    
    for i in 0..available {
        if let Ok(controller) = game_controller_subsystem.open(i) {
            let name = controller.name();
            
            // Get joystick to access GUID
            if let Ok(joystick) = joystick_subsystem.open(i) {
                let guid = joystick.guid();
                let guid_bytes = guid.raw();
                
                // Convert to hex string
                let hex_str: String = guid_bytes.iter()
                    .map(|b| format!("{:02x}", b))
                    .collect();
                
                println!("\nController {}: {}", i, name);
                println!("  Raw GUID bytes: {:?}", guid_bytes);
                println!("  Hex string (32 chars): {}", hex_str);
                
                // Apply Ryujinx's rearrangement
                let rearranged = format!(
                    "{}{}{}{}-{}{}-{}-{}-{}",
                    &hex_str[8..10],   // byte 4
                    &hex_str[10..12],  // byte 5
                    &hex_str[4..6],    // byte 2
                    &hex_str[0..2],    // byte 0
                    &hex_str[14..16],  // byte 7
                    &hex_str[12..14],  // byte 6
                    &hex_str[16..20],  // bytes 8-9
                    &hex_str[20..24],  // bytes 10-11
                    &hex_str[24..32]   // bytes 12-15
                );
                
                println!("  Rearranged: {}", rearranged);
                
                // Remove CRC and add "0000"
                let final_guid = format!("0000{}", &rearranged[4..]);
                let ryujinx_id = format!("{}-{}", i, final_guid);
                
                println!("  Final Ryujinx ID: {}", ryujinx_id);
            }
        }
    }
}
