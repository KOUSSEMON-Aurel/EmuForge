use serde::Deserialize;
use std::process::Command;
use std::path::PathBuf;

#[derive(Deserialize)]
struct LaunchConfig {
    emulator_path: PathBuf,
    rom_path: PathBuf,
    bios_path: Option<PathBuf>,
    args: Vec<String>,
    working_dir: Option<PathBuf>,
    env_vars: Vec<(String, String)>,
}

// Hide console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {

    // Config injected at compile time via env var EMUFORGE_CONFIG_PATH
    let config_json = include_str!(env!("EMUFORGE_CONFIG_PATH"));
    let config: LaunchConfig = serde_json::from_str(config_json)
        .expect("Failed to parse launch config");

    let mut cmd = Command::new(&config.emulator_path);
    cmd.args(&config.args);
    cmd.arg(&config.rom_path);

    // Some emulators might need the bios path passed as an argument
    // But for now, since we don't have plugin logic in the stub yet, 
    // we just ignore it here or maybe pass it if we knew the flag.
    // The LaunchConfig logic in Core/Plugin should have already baked the BIOS path into 'args' correctly if needed.
    // However, keeping it in the struct allows the stub to potentially do something with it later (like copying it).
    
    if let Some(dir) = config.working_dir {
        cmd.current_dir(dir);
    }
    
    for (key, val) in config.env_vars {
        cmd.env(key, val);
    }

    match cmd.status() {
        Ok(status) => {
             if !status.success() {
                 eprintln!("Emulator exited with non-zero status: {:?}", status.code());
                 std::process::exit(status.code().unwrap_or(1));
             }
        }
        Err(e) => {
            eprintln!("Failed to launch emulator: {}", e);
            eprintln!("Path: {:?}", config.emulator_path);
            std::process::exit(1);
        }
    }
}
