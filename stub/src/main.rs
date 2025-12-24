use serde::Deserialize;
use std::process::Command;
use std::path::PathBuf;

#[derive(Deserialize)]
struct LaunchConfig {
    emulator_path: PathBuf,
    rom_path: PathBuf,
    args: Vec<String>,
    working_dir: Option<PathBuf>,
    env_vars: Vec<(String, String)>,
}

fn main() {
    // Config injected at compile time via env var EMUFORGE_CONFIG_PATH
    let config_json = include_str!(env!("EMUFORGE_CONFIG_PATH"));
    let config: LaunchConfig = serde_json::from_str(config_json)
        .expect("Failed to parse launch config");

    let mut cmd = Command::new(&config.emulator_path);
    cmd.args(&config.args);
    cmd.arg(&config.rom_path);

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
