use emuforge_core::forge::ExecutableForge;
use emuforge_core::detection::FileAnalyzer;
use std::path::{Path, PathBuf};
use std::io::{Write, Read};
use tauri::Emitter;

use std::sync::Mutex;
// use tauri::State;

/// Marker that indicates the start of embedded data in portable mode
const PORTABLE_MARKER: &[u8] = b"EMUFORGE_PORTABLE_DATA_START";

#[allow(dead_code)]
struct AppState {

    output_dir: Mutex<PathBuf>,
}

#[tauri::command]
async fn forge_executable(
    app: tauri::AppHandle,
    game_name: String,
    emulator_path: String,
    rom_path: String,
    bios_path: Option<String>,
    output_dir: String,
    _plugin: String,
    _target_os: String,
    fullscreen: bool,
    args: Vec<String>,
    portable_mode: Option<bool>,
) -> Result<String, String> {
    use emuforge_core::plugin::manager::PluginManager;
    use emuforge_core::forge::LaunchConfig;
    
    let rom_p = PathBuf::from(&rom_path);
    let emu_p = PathBuf::from(&emulator_path);
    
    let manager = PluginManager::new();
    
    // Use configured_driver_for to start with a fresh plugin instance 
    // that knows about the user-provided binary path.
    let maybe_plugin = manager.configured_driver_for(&emu_p);
    
    // We determine config AND driver_id in one go to avoid ownership issues
    let (mut config, driver_id) = if let Some(plugin) = &maybe_plugin {
        let cfg = plugin.prepare_launch_config(&rom_p, Path::new(&output_dir))
            .map_err(|e| format!("Plugin error: {}", e))?;
        (cfg, plugin.id().to_string())
    } else {
        (LaunchConfig {
            emulator_path: emu_p.clone(),
            rom_path: rom_p.clone(),
            bios_path: bios_path.as_ref().map(PathBuf::from),
            args,
            working_dir: None,
            env_vars: vec![],
        }, "generic".to_string())
    };

    // === BIOS HANDLING FOR PCSX2 ===
    // PCSX2 requires BIOS files to be in a specific folder structure.
    // The plugin creates output_dir/pcsx2_data/PCSX2/bios/, so we copy the user's BIOS there.
    if driver_id == "pcsx2" {
        if let Some(bios_src) = &bios_path {
            let bios_src_path = PathBuf::from(bios_src);
            if bios_src_path.exists() {
                // BIOS must go in pcsx2_data/PCSX2/bios/ to match XDG_CONFIG_HOME structure
                let bios_dest_dir = PathBuf::from(&output_dir).join("pcsx2_data").join("PCSX2").join("bios");
                std::fs::create_dir_all(&bios_dest_dir)
                    .map_err(|e| format!("Failed to create bios directory: {}", e))?;
                
                // Copy the BIOS file with its original name
                let bios_filename = bios_src_path.file_name()
                    .ok_or_else(|| "Invalid BIOS path".to_string())?;
                let bios_dest = bios_dest_dir.join(bios_filename);
                
                std::fs::copy(&bios_src_path, &bios_dest)
                    .map_err(|e| format!("Failed to copy BIOS file: {}", e))?;
            }
        }
    }

    // Apply generic full screen override if requested
    // NOTE: PCSX2 already has -fullscreen in its plugin args, so we skip it here
    if fullscreen {
        match driver_id.as_str() {
            "ppsspp" | "ryujinx" => {
                 config.args.push("--fullscreen".to_string());
            },
            "cemu" => {
                 config.args.push("-f".to_string());
            },
            "generic" => {
                 config.args.push("--fullscreen".to_string());
            },
            _ => { /* PCSX2 and others already configured by plugin */ }
        }
    }

    // Check if portable mode is requested
    if portable_mode.unwrap_or(false) {
        return forge_portable_executable(
            app,
            game_name,
            emu_p,
            rom_p,
            bios_path.map(PathBuf::from),
            PathBuf::from(&output_dir),
            driver_id,
        );
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

    let forge = ExecutableForge::new(stub_crate_path, out_path.clone());

    // Generic Fullscreen Logic handled above...

    // ISOLATED CONFIG STRATEGY provided by user
    // Instead of messing with global config, we create a local .duckstation folder
    // and force XDG_CONFIG_HOME/XDG_DATA_HOME to point to it.
    if driver_id == "duckstation" {
        // Define local config dir inside output
        let local_conf_name = ".duckstation";
        let local_conf_path = out_path.join(local_conf_name);
        
        std::fs::create_dir_all(&local_conf_path).map_err(|e| format!("Failed to create config dir: {}", e))?;
        
        // 1. Write the USER-PROVIDED robust settings.ini
        let settings_content = r#"[Main]
Language=en
ConfirmPowerOff=false
StartFullscreen=true

[BIOS]
SearchDirectory={{EXE_DIR}}/.duckstation/bios

[Console]
Region=Auto

[GPU]
Renderer=OpenGL

[Audio]
Backend=SDL

[UI]
ShowGameList=false
ShowStartWizard=false
"#;
        std::fs::write(local_conf_path.join("settings.ini"), settings_content)
            .map_err(|e| format!("Failed to write settings.ini: {}", e))?;
            
        // 2. Setup BIOS directory (if user provided one)
        if let Some(bios_src) = &bios_path {
            let bios_dest_dir = local_conf_path.join("bios");
            std::fs::create_dir_all(&bios_dest_dir).map_err(|e| format!("Failed to create bios dir: {}", e))?;
            
            let bios_src_path = PathBuf::from(bios_src);
            if let Some(name) = bios_src_path.file_name() {
                std::fs::copy(&bios_src_path, bios_dest_dir.join(name))
                     .map_err(|e| format!("Failed to copy BIOS: {}", e))?;
            }
        }

        // 3. Inject Environment Variables using {{EXE_DIR}} placeholder
        // The stub will replace {{EXE_DIR}} with the actual dir at runtime
        config.env_vars.push(("XDG_CONFIG_HOME".to_string(), format!("{{{{EXE_DIR}}}}/{}", local_conf_name)));
        config.env_vars.push(("XDG_DATA_HOME".to_string(), format!("{{{{EXE_DIR}}}}/{}", local_conf_name)));
    }

    match forge.forge(&game_name, &config) {
        Ok(path) => Ok(path.to_string_lossy().to_string()),
        Err(e) => Err(format!("Forge failed: {:?}", e)),
    }
}

/// Create a portable all-in-one executable with embedded emulator, ROM, BIOS, and config
fn forge_portable_executable(
    app: tauri::AppHandle,
    game_name: String,
    emulator_path: PathBuf,
    rom_path: PathBuf,
    bios_path: Option<PathBuf>,
    output_dir: PathBuf,
    driver_id: String,
) -> Result<String, String> {
    use std::fs::File;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;
    
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;
    let output_dir = output_dir.canonicalize()
        .map_err(|e| format!("Failed to canonicalize output directory: {}", e))?;
    
    // Step 1: Find and compile the stub
    let possible_paths = vec!["../../stub", "../stub", "stub"];
    let mut stub_crate_path = None;
    
    for p in possible_paths {
        if let Ok(path) = PathBuf::from(p).canonicalize() {
            if path.join("Cargo.toml").exists() {
                stub_crate_path = Some(path);
                break;
            }
        }
    }
    
    let stub_crate = stub_crate_path.ok_or("Could not find stub crate directory")?;
    
    // Compile the stub (we need a simple dummy config for now)
    let temp_config_dir = output_dir.join(".temp_portable");
    std::fs::create_dir_all(&temp_config_dir)
        .map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
    let temp_config_path = temp_config_dir.join("config.json");
    // Write a dummy config - the stub will detect portable mode and ignore this
    let dummy_config = r#"{"emulator_path":"","rom_path":"","args":[],"env_vars":[]}"#;
    std::fs::write(&temp_config_path, dummy_config)
        .map_err(|e| format!("Failed to write temp config: {}", e))?;
    
    // Compile stub
    let status = std::process::Command::new("cargo")
        .current_dir(&stub_crate)
        .env("EMUFORGE_CONFIG_PATH", temp_config_path.to_str().unwrap())
        .args(["build", "--release"])
        .status()
        .map_err(|e| format!("Failed to run cargo: {}", e))?;
    
    if !status.success() {
        return Err("Stub compilation failed".to_string());
    }
    
    // Find the compiled stub
    // Since we are in a workspace, artifacts are usually in the root target dir
    // We check both stub/target (standalone) and ../target (workspace)
    let standalone_path = stub_crate.join("target/release/emuforge-stub");
    let workspace_path = stub_crate.parent().unwrap().join("target/release/emuforge-stub");
    
    let stub_binary = if workspace_path.exists() {
        workspace_path
    } else if standalone_path.exists() {
        standalone_path
    } else {
        return Err(format!("Stub binary not found at {:?} or {:?}", workspace_path, standalone_path));
    };
    
    // Step 2: Create ZIP archive with all files
    let zip_path = temp_config_dir.join("data.zip");
    let zip_file = File::create(&zip_path)
        .map_err(|e| format!("Failed to create ZIP file: {}", e))?;
    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    
    // Add emulator
    let emu_filename = emulator_path.file_name()
        .ok_or("Invalid emulator path")?
        .to_string_lossy()
        .to_string();
    add_file_to_zip(&app, &mut zip, &emulator_path, &emu_filename, options)?;
    
    // Add ROM
    // Optimization: Use Stored (no compression) for ROMs to speed up forging significantly.
    // Game files are often already compressed (CSO, CHD, GZ) or don't compress well (ISO).
    let rom_options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .large_file(true);

    let rom_filename = rom_path.file_name()
        .ok_or("Invalid ROM path")?
        .to_string_lossy()
        .to_string();
    add_file_to_zip(&app, &mut zip, &rom_path, &rom_filename, rom_options)?;

    // Handle CUE files dependencies (.bin files)
    if let Some(ext) = rom_path.extension() {
        if ext.to_string_lossy().eq_ignore_ascii_case("cue") {
            let content = std::fs::read_to_string(&rom_path)
                .map_err(|e| format!("Failed to read CUE file: {}", e))?;
            
            let parent_dir = rom_path.parent().unwrap_or(Path::new("."));
            
            for line in content.lines() {
                if line.trim().starts_with("FILE") {
                    // Extract filename: FILE "filename.bin" BINARY
                    let parts: Vec<&str> = line.split('"').collect();
                    if parts.len() >= 2 {
                        let bin_filename = parts[1];
                        let bin_path = parent_dir.join(bin_filename);
                        
                        if bin_path.exists() {
                            let _ = app.emit("forge-progress", serde_json::json!({ 
                                "percentage": 0, 
                                "message": format!("Détection dépendance: {}...", bin_filename) 
                            }));
                            add_file_to_zip(&app, &mut zip, &bin_path, bin_filename, rom_options)?;
                        }
                    }
                }
            }
        }
    }
    
    // Add BIOS if present
    // Add BIOS if present
    // Strategy: Copy BIOS to the config folder on disk FIRST, so add_directory_to_zip includes it naturally.
    // This prevents "Duplicate filename" errors.
    if let Some(bios) = &bios_path {
        if bios.exists() {
            let bios_filename = bios.file_name()
                .ok_or("Invalid BIOS path")?
                .to_string_lossy()
                .to_string();
            
            // Construct destination path: output_dir/pcsx2_data/PCSX2/bios/filename
            let bios_dest_dir = output_dir.join("pcsx2_data/PCSX2/bios");
            std::fs::create_dir_all(&bios_dest_dir)
                .map_err(|e| format!("Failed to create BIOS dir: {}", e))?;
                
            let bios_dest_path = bios_dest_dir.join(&bios_filename);
            std::fs::copy(bios, &bios_dest_path)
                .map_err(|e| format!("Failed to copy BIOS to config dir: {}", e))?;
        }
    }
    
    // Add PCSX2 config if exists
    let pcsx2_config_dir = output_dir.join("pcsx2_data");
    if driver_id == "pcsx2" && pcsx2_config_dir.exists() {
        add_directory_to_zip(&app, &mut zip, &pcsx2_config_dir, "pcsx2_data", options)?;
    }

    // DuckStation: Inject minimal settings.ini to bypass First Run Wizard
    if driver_id == "duckstation" {
        // Create a dummy settings.ini content
        let settings_content = "[Main]\nSettingsVersion=3\nStartFullscreen=true\n";
        
        // We need to place it at: pcsx2_data/duckstation/settings.ini
        // (Since config_dir is hardcoded to "pcsx2_data" for now)
        let settings_path = Path::new("pcsx2_data").join("duckstation").join("settings.ini");
        
        // We write it to the ZIP using the same options as other files
        zip.start_file(settings_path.to_string_lossy(), options)
            .map_err(|e| format!("Failed to add settings.ini to zip: {}", e))?;
        zip.write_all(settings_content.as_bytes())
            .map_err(|e| format!("Failed to write settings.ini content: {}", e))?;
    }
    
    zip.finish().map_err(|e| format!("Failed to finalize ZIP: {}", e))?;
    
    // Step 3: Create the portable config JSON
    let portable_config = serde_json::json!({
        "game_name": sanitize_filename(&game_name),
        "emulator_filename": emu_filename,
        "rom_filename": rom_filename,
        "config_dir": "pcsx2_data"
    });
    let config_json = serde_json::to_vec(&portable_config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    // Step 4: Concatenate: stub + marker + config_len + config + zip
    let output_path = output_dir.join(sanitize_filename(&game_name));
    let mut output_file = File::create(&output_path)
        .map_err(|e| format!("Failed to create output file: {}", e))?;
    
    // Copy stub binary
    let mut stub_data = Vec::new();
    File::open(&stub_binary)
        .map_err(|e| format!("Failed to open stub: {}", e))?
        .read_to_end(&mut stub_data)
        .map_err(|e| format!("Failed to read stub: {}", e))?;
    output_file.write_all(&stub_data)
        .map_err(|e| format!("Failed to write stub: {}", e))?;
    
    // Write marker
    output_file.write_all(PORTABLE_MARKER)
        .map_err(|e| format!("Failed to write marker: {}", e))?;
    
    // Write config length (4 bytes, little-endian)
    let config_len = config_json.len() as u32;
    output_file.write_all(&config_len.to_le_bytes())
        .map_err(|e| format!("Failed to write config length: {}", e))?;
    
    // Write config
    output_file.write_all(&config_json)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    
    // Write ZIP data
    // Write ZIP data (Streaming)
    // Write ZIP data (Streaming with Progress)
    let mut zip_file = File::open(&zip_path)
        .map_err(|e| format!("Failed to open ZIP: {}", e))?;
    
    let total_size = zip_file.metadata().map(|m| m.len()).unwrap_or(0);
    // Optimization: Increase buffer to 1MB to speed up huge file copy
    let mut buffer = vec![0u8; 1024 * 1024]; 
    let mut written = 0u64;
    let mut last_percent = 0;

    let _ = app.emit("forge-progress", serde_json::json!({ "percentage": 0, "message": "Assemblage final..." }));

    loop {
        let n = zip_file.read(&mut buffer)
            .map_err(|e| format!("Failed to read ZIP: {}", e))?;
        if n == 0 { break; }
        
        output_file.write_all(&buffer[..n])
            .map_err(|e| format!("Failed to append ZIP data: {}", e))?;
            
        written += n as u64;
        
        if total_size > 0 {
            let percent = (written * 100) / total_size;
            if percent > last_percent {
                last_percent = percent;
                let _ = app.emit("forge-progress", serde_json::json!({ 
                    "percentage": percent, 
                    "message": format!("Assemblage final: {}%", percent) 
                }));
            }
        }
    }
    
    // Make executable on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&output_path)
            .map_err(|e| format!("Failed to get permissions: {}", e))?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&output_path, perms)
            .map_err(|e| format!("Failed to set permissions: {}", e))?;
    }
    
    // Cleanup temp files
    let _ = std::fs::remove_dir_all(&temp_config_dir);
    
    Ok(output_path.to_string_lossy().to_string())
}

/// Add a file to ZIP archive with Progress
fn add_file_to_zip<W: Write + std::io::Seek>(
    app: &tauri::AppHandle,
    zip: &mut zip::ZipWriter<W>,
    file_path: &Path,
    archive_name: &str,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    zip.start_file(archive_name, options)
        .map_err(|e| format!("Failed to start file in ZIP: {}", e))?;
    
    let mut file = std::fs::File::open(file_path)
        .map_err(|e| format!("Failed to open {}: {}", file_path.display(), e))?;
    
    // Progress Loop
    let total_size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut buffer = vec![0u8; 1024 * 1024]; // 1MB Buffer
    let mut written = 0u64;
    let mut last_percent = 0;
    
    let file_name = file_path.file_name().unwrap_or_default().to_string_lossy();
    let _ = app.emit("forge-progress", serde_json::json!({ 
        "percentage": 0, 
        "message": format!("Mise en boîte: {}...", file_name) 
    }));

    loop {
        let n = file.read(&mut buffer)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        if n == 0 { break; }
        
        zip.write_all(&buffer[..n])
            .map_err(|e| format!("Failed to write to ZIP: {}", e))?;
            
        written += n as u64;
        
        if total_size > 0 {
            let percent = (written * 100) / total_size;
            if percent > last_percent {
                last_percent = percent;
                let _ = app.emit("forge-progress", serde_json::json!({ 
                    "percentage": percent, 
                    "message": format!("Ajout de {}: {}%", file_name, percent) 
                }));
            }
        }
    }
    
    Ok(())
}

/// Recursively add a directory to ZIP archive
fn add_directory_to_zip<W: Write + std::io::Seek>(
    app: &tauri::AppHandle,
    zip: &mut zip::ZipWriter<W>,
    dir_path: &Path,
    prefix: &str,
    options: zip::write::SimpleFileOptions,
) -> Result<(), String> {
    for entry in std::fs::read_dir(dir_path).map_err(|e| format!("Failed to read dir: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        let name = format!("{}/{}", prefix, entry.file_name().to_string_lossy());
        
        if path.is_dir() {
            add_directory_to_zip(app, zip, &path, &name, options)?;
        } else {
            add_file_to_zip(app, zip, &path, &name, options)?;
        }
    }
    Ok(())
}

/// Sanitize filename to be safe for filesystem
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' { c } else { '_' })
        .collect()
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

#[tauri::command]
async fn download_emulator(emu_id: String) -> Result<String, String> {
    use emuforge_core::downloader::EmulatorDownloader;
    
    // Download to a local "emulators" folder in the app directory or home
    let base_dir = dirs::home_dir().unwrap_or(PathBuf::from(".")).join(".emuforge/emulators");
    let downloader = EmulatorDownloader::new(base_dir);
    
    match downloader.download(&emu_id).await {
        Ok(path) => Ok(path.to_string_lossy().to_string()),
        Err(e) => Err(format!("Download failed: {}", e)),
    }
}

#[tauri::command]
async fn get_installed_emulators() -> Result<Vec<String>, String> {

    use emuforge_core::downloader::EmulatorDownloader;
    let base_dir = dirs::home_dir().unwrap_or(PathBuf::from(".")).join(".emuforge/emulators");
    let downloader = EmulatorDownloader::new(base_dir);
    
    // Check list of known emulators
    // Ideally we iterate known plugins or a static list.
    // For now, let's just check the ones we hardcoded in downloader logic or UI.
    let candidates = vec!["ppsspp", "pcsx2", "duckstation", "dolphin", "ryujinx", "cemu", "xemu", "rpcs3", "melonds", "lime3ds", "redream"];
    
    let mut installed = vec![];
    for id in candidates {
        if downloader.is_installed(id) {
            installed.push(id.to_string());
        }
    }
    Ok(installed)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())

        .manage(AppState {
            output_dir: Mutex::new(PathBuf::from("output")),
        })
        .invoke_handler(tauri::generate_handler![forge_executable, validate_file, download_emulator, get_installed_emulators])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
