use sdl2;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let joystick_subsystem = sdl_context.joystick().unwrap();
    
    let available = joystick_subsystem.num_joysticks().unwrap_or(0);
    
    println!("Found {} joysticks", available);
    
    for i in 0..available {
        if let Ok(joystick) = joystick_subsystem.open(i) {
            let guid = joystick.guid();
            let guid_bytes = guid.raw();
            
            let hex_str: String = guid_bytes.data.iter()
                .map(|b| format!("{:02x}", b))
                .collect();
            
            println!("\nJoystick {}: {}", i, joystick.name());
            println!("  Raw bytes: {:?}", guid_bytes.data);
            println!("  Hex string: {}", hex_str);
            
            // Ryujinx rearrangement
            let rearranged = format!(
                "{}{}{}{}-{}{}-{}-{}-{}",
                &hex_str[8..10],   &hex_str[10..12],
                &hex_str[4..6],    &hex_str[0..2],
                &hex_str[14..16],  &hex_str[12..14],
                &hex_str[16..20],  &hex_str[20..24],  &hex_str[24..32]
            );
            
            println!("  Rearranged: {}", rearranged);
            
            let final_guid = format!("0000{}", &rearranged[4..]);
            let ryujinx_id = format!("{}-{}", i, final_guid);
            
            println!("  Final Ryujinx ID: {}", ryujinx_id);
            println!("  Expected:         0-00000003-045e-0000-8e02-000010010000");
        }
    }
}
