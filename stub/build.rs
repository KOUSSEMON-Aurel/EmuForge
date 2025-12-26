use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let config_path = env::var("EMUFORGE_CONFIG_PATH")
        .unwrap_or_else(|_| "default_config.json".to_string());
    
    println!("cargo:rerun-if-env-changed=EMUFORGE_CONFIG_PATH");
    
    let config_content = fs::read_to_string(&config_path)
        .unwrap_or_else(|_| {
            // Fallback safe if even default_config.json is missing
            r#"{"emulator_path":"","rom_path":"","args":[],"env_vars":[]}"#.to_string()
        });

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("config_gen.rs");
    
    let code = format!(
        "pub const CONFIG_JSON: &str = {:?};",
        config_content
    );
    
    fs::write(&dest_path, code).unwrap();
}
