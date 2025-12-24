use emuforge_core::forge::{ExecutableForge, LaunchConfig};
use emuforge_core::detection::FileAnalyzer;
use std::path::PathBuf;
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
    args: Vec<String>,
) -> Result<String, String> {
    let config = LaunchConfig {
        emulator_path: PathBuf::from(emulator_path),
        rom_path: PathBuf::from(rom_path),
        bios_path: bios_path.map(PathBuf::from),
        args,
        working_dir: None,
        env_vars: vec![],
    };


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
