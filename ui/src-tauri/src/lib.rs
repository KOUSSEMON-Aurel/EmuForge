use emuforge_core::forge::{ExecutableForge, LaunchConfig};
use emuforge_core::detection::FileAnalyzer;
use std::path::{Path, PathBuf};

use std::sync::Mutex;
// use tauri::State;


#[allow(dead_code)]
struct AppState {

    output_dir: Mutex<PathBuf>,
}

#[tauri::command]
fn forge_executable(
    game_name: String,
    emulator_path: String,
    rom_path: String,
    bios_path: Option<String>,
    output_dir: String,
    fullscreen: bool,
    args: Vec<String>,
) -> Result<String, String> {
    use emuforge_core::plugin::EmulatorPlugin;
    use emuforge_core::plugin::ppsspp::PpssppPlugin;
    
    let rom_p = PathBuf::from(&rom_path);
    let emu_p = PathBuf::from(&emulator_path);
    
    // Simple detection logic: 
    // If emulator path contains "ppsspp" (case insensitive) OR extension implies PSP
    let is_ppsspp = emulator_path.to_lowercase().contains("ppsspp");
    
    let mut config = if is_ppsspp {
        // Use the plugin
        let plugin = PpssppPlugin::new(Some(emu_p.clone()));
        plugin.prepare_launch_config(&rom_p, Path::new(&output_dir))
            .map_err(|e| format!("Plugin error: {}", e))?
    } else {
        // Fallback to manual config
        LaunchConfig {
            emulator_path: emu_p,
            rom_path: rom_p,
            bios_path: bios_path.map(PathBuf::from),
            args,
            working_dir: None,
            env_vars: vec![],
        }
    };

    // Apply generic full screen override if requested
    if fullscreen {
        // For PPSSPP, the flag is --fullscreen
        // For others, we might need a mapping, but for now we apply common flags or just this one
        if is_ppsspp {
            config.args.push("--fullscreen".to_string());
        }
        // TODO: support other emulators fullscreen flags
    }




    // Calculate stub path relative to the current working directory or binary location
    // In dev mode, we assume we run from the project root or we find it in ../stub
    // Note: In a real app we might want to bundle the stub binary as a resource.
    // For this MVP, we look for the stub crate in known locations
    
    let possible_paths = vec![
        "../../stub",
        "../stub",
        "stub" 
    ];
    
    let mut stub_crate_path = PathBuf::new();
    let mut found = false;
    
    for p in possible_paths {
        if let Ok(path) = PathBuf::from(p).canonicalize() {
            if path.join("Cargo.toml").exists() {
                stub_crate_path = path;
                found = true;
                break;
            }
        }
    }
    
    if !found {
        return Err("Could not find stub crate directory".to_string());
    }

    let out_path = PathBuf::from(output_dir);

    let forge = ExecutableForge::new(stub_crate_path, out_path);

    match forge.forge(&game_name, &config) {
        Ok(path) => Ok(path.to_string_lossy().to_string()),
        Err(e) => Err(format!("Forge failed: {:?}", e)),
    }
}

#[tauri::command]
fn validate_file(path: String) -> Result<String, String> {
    let path_buf = PathBuf::from(path);
    if let Some(ft) = FileAnalyzer::detect_type(&path_buf) {
        Ok(format!("{:?}", ft))
    } else {
        Err("Unknown file type".to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())

        .manage(AppState {
            output_dir: Mutex::new(PathBuf::from("output")),
        })
        .invoke_handler(tauri::generate_handler![forge_executable, validate_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
