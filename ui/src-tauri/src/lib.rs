use emuforge_core::forge::ExecutableForge;
use emuforge_core::detection::FileAnalyzer;
use emuforge_core::plugin::HostSpecs;
use std::path::{Path, PathBuf};
use std::io::{Write, Read};
use std::process::Command;
use tauri::Emitter;

use std::sync::Mutex;
// use tauri::State;

/// Marker that indicates the start of embedded data in portable mode
const PORTABLE_MARKER: &[u8] = &[0xDE, 0xAD, 0xBE, 0xEF, 0x45, 0x4D, 0x55, 0x46, 0x4F, 0x52, 0x47, 0x45, 0x56, 0x32];

#[allow(dead_code)]
struct AppState {
    output_dir: Mutex<PathBuf>,
}

// Note: DUCKSTATION_SETTINGS a été déplacé dans core/src/plugin/duckstation.rs

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
    screen_width: Option<u32>,
    screen_height: Option<u32>,
) -> Result<String, String> {
    use emuforge_core::plugin::manager::PluginManager;
    use emuforge_core::forge::LaunchConfig;
    
    let rom_p = PathBuf::from(&rom_path);
    let emu_p = PathBuf::from(&emulator_path);
    
    let manager = PluginManager::new();
    
    // Use configured_driver_for to start with a fresh plugin instance 
    // that knows about the user-provided binary path.
    let maybe_plugin = manager.configured_driver_for(&emu_p);
    
    // Create progress callback
    let app_handle = app.clone();
    let progress_cb = move |msg: String| {
        let _ = app_handle.emit("forge-progress", serde_json::json!({
            "percentage": 0,
            "message": msg
        }));
    };

    // Détecter le support Vulkan
    let vulkan_support = Command::new("vulkaninfo")
        .arg("--summary")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    
    // Créer HostSpecs
    let host_specs = HostSpecs {
        screen_width: screen_width.unwrap_or(1920),
        screen_height: screen_height.unwrap_or(1080),
        vulkan_support,
    };

    // We determine config AND driver_id in one go to avoid ownership issues
    let (mut config, driver_id) = if let Some(plugin) = &maybe_plugin {
        let cfg = plugin.prepare_launch_config_with_specs(
            &rom_p, 
            Path::new(&output_dir),
            Some(host_specs),
            Some(&progress_cb)
        ).map_err(|e| format!("Plugin error: {}", e))?;
        (cfg, plugin.id().to_string())
    } else {
        (LaunchConfig {
            emulator_path: emu_p.clone(),
            rom_path: rom_p.clone(),
            bios_path: bios_path.as_ref().map(PathBuf::from),
            args,
            args_after_rom: vec![],
            working_dir: None,
            env_vars: vec![],
        }, "generic".to_string())
    };

    // === UTILISATION DES NOUVELLES MÉTHODES DU TRAIT ===
    let out_path = PathBuf::from(&output_dir);

    // 1. Setup environment moved to specific branches to avoid polluting output dir in portable mode
    // if let Some(plugin) = &maybe_plugin { ... } moved below

    // 2. Fullscreen args via le plugin
    if fullscreen {
        if let Some(plugin) = &maybe_plugin {
            // Args avant la ROM
            for arg in plugin.fullscreen_args() {
                config.args.push(arg);
            }
            // Args après la ROM (ex: Cemu -f)
            for arg in plugin.fullscreen_args_after_rom() {
                config.args_after_rom.push(arg);
            }
        } else {
            config.args.push("--fullscreen".to_string());
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
            out_path,
            driver_id,
            fullscreen,
        );
    }

    // Calculate stub path
    let possible_paths = vec!["../../stub", "../stub", "stub"];
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

    // NOUVEAU: Setup environment pour le mode NON-PORTABLE (Raccourci)
    // Ici, on VEUT que la structure persiste dans le dossier de sortie pour que le jeu fonctionne.
    if let Some(plugin) = &maybe_plugin {
        // Convert bios_path to PathBuf if present
        let bios_pathbuf = bios_path.as_ref().map(|s| PathBuf::from(s));
        let bios_p = bios_pathbuf.as_deref();
        
        // Setup environment (configs, BIOS, etc.)
        plugin.setup_environment(&out_path, bios_p)
            .map_err(|e| format!("Environment setup failed: {}", e))?;
        
        // Vérifier si le plugin nécessite un patch de l'émulateur (ex: RPCS3 avec firmware)
        let final_emulator_path = if let Some(patched) = plugin.prepare_portable_binary(
            &emu_p,
            bios_p,
            &out_path
        ).map_err(|e| format!("Emulator patching failed: {}", e))? {
            println!("🔧 Using patched emulator binary for shortcut");
            patched
        } else {
            emu_p.clone()
        };
        
        // Update config with potentially patched emulator path
        config.emulator_path = final_emulator_path;

        // 3. Generate wrapper script si nécessaire (ex: DuckStation)
        if plugin.requires_wrapper() {
            if let Some(wrapper_path) = plugin.generate_wrapper_script(&config, &out_path, &game_name)
                .map_err(|e| format!("Wrapper error: {}", e))? {
                // Modifier la config pour utiliser le wrapper
                config.emulator_path = wrapper_path;
                config.args.clear();
                config.rom_path = PathBuf::from("");
            }
        }
    }

    let forge = ExecutableForge::new(stub_crate_path, out_path.clone());

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
    fullscreen: bool,
) -> Result<String, String> {
    use std::fs::File;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;
    use emuforge_core::plugin::manager::PluginManager;
    
    // Create output directory for the FINAL file
    std::fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create output directory: {}", e))?;
    let output_dir_canonical = output_dir.canonicalize()
        .map_err(|e| format!("Failed to canonicalize output directory: {}", e))?;

    // Use SYSTEM TEMP directory for assembly to avoid triggering Tauri watcher
    let temp_work_dir = std::env::temp_dir().join(format!("emuforge_build_{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&temp_work_dir)
        .map_err(|e| format!("Failed to create temp work dir: {}", e))?;

    println!("🛠️  Build temporaire dans: {:?}", temp_work_dir);

    // Appeler setup_environment via le plugin DANS LE TEMP DIR
    let manager = PluginManager::new();
    let plugin_opt = manager.configured_driver_for(&emulator_path);
    
    // Vérifier si le plugin nécessite un patch de l'émulateur (ex: RPCS3 avec firmware)
    let final_emulator_path = if let Some(plugin) = &plugin_opt {
        println!("🔌 Plugin détecté: {}", plugin.id());
        let bios_p = bios_path.as_ref().map(|p| p.as_path());
        println!("📀 BIOS path fourni: {:?}", bios_p);
        
        plugin.setup_environment(&temp_work_dir, bios_p)
            .map_err(|e| format!("Plugin setup error: {}", e))?;
        println!("✅ setup_environment terminé");
        
        // Check if emulator needs patching (e.g. RPCS3 with firmware)
        if let Some(patched) = plugin.prepare_portable_binary(&emulator_path, bios_p, &temp_work_dir)
            .map_err(|e| format!("Emulator patching failed: {}", e))? {
            println!("🔧 Using patched emulator binary");
            patched
        } else {
            emulator_path.clone()
        }
    } else {
        println!("⚠️ Aucun plugin détecté pour: {:?}", emulator_path);
        emulator_path.clone()
    };
    
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
    // let temp_config_dir = output_dir.join(".temp_portable"); // REMOVED
    let temp_config_dir = temp_work_dir.clone(); // Use the main temp dir
    // std::fs::create_dir_all(&temp_config_dir) // Already created
    //     .map_err(|e| format!("Failed to create temp dir: {}", e))?;
    
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
    let zip_path = temp_work_dir.join("data.zip"); // ZIP in temp dir
    let zip_file = File::create(&zip_path)
        .map_err(|e| format!("Failed to create ZIP file: {}", e))?;
    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    
    // Add emulator
    let emu_filename = final_emulator_path.file_name()
        .ok_or("Invalid emulator path")?
        .to_string_lossy()
        .to_string();
    
    // Ensure emulator is executable
    let mut emu_options = options.clone();
    #[cfg(unix)]
    {
        emu_options = emu_options.unix_permissions(0o755);
    }
    
    // Check if emulator is a directory (e.g., patched Ryujinx squashfs-root)
    if final_emulator_path.is_dir() {
        println!("📁 Bundling emulator directory: {}", emu_filename);
        add_directory_to_zip(&app, &mut zip, &final_emulator_path, &emu_filename, emu_options)?;
    } else {
        add_file_to_zip(&app, &mut zip, &final_emulator_path, &emu_filename, emu_options)?;
    }
    
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
    println!("💿 Ajout ROM au ZIP: {} (source: {:?})", rom_filename, rom_path);
    if let Ok(metadata) = std::fs::metadata(&rom_path) {
        println!("   📦 Taille: {} bytes", metadata.len());
    } else {
        println!("   ⚠️ Impossible de lire les métadonnées de la ROM");
    }
    
    add_file_to_zip(&app, &mut zip, &rom_path, &rom_filename, rom_options)?;
    println!("✅ ROM ajoutée");

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
    
    // DuckStation: Create .duckstation_home structure BEFORE handling BIOS
    // This will be added to the ZIP and extracted next to the executable
    if driver_id == "duckstation" {
        let duckstation_home = temp_work_dir.join(".duckstation_home");
        let ds_data_dir = duckstation_home.join(".local/share/duckstation");
        std::fs::create_dir_all(&ds_data_dir)
            .map_err(|e| format!("Failed to create DuckStation data dir: {}", e))?;
        
        // Create empty bios directory (BIOS will be copied below)
        std::fs::create_dir_all(ds_data_dir.join("bios"))
            .map_err(|e| format!("Failed to create BIOS dir: {}", e))?;
    }
    
    // Add BIOS if present
    // Add BIOS if present
    // Strategy: Copy BIOS to the config folder on disk FIRST, so add_directory_to_zip includes it naturally.
    // This prevents "Duplicate filename" errors.
    // NOTE: Ryujinx handles firmware differently via prepare_portable_binary - skip generic copy
    if let Some(bios) = &bios_path {
        if bios.exists() && driver_id != "ryujinx" {
            // Only copy if it's a file (not a directory like Ryujinx firmware folder)
            if bios.is_file() {
                let bios_filename = bios.file_name()
                    .ok_or("Invalid BIOS path")?
                    .to_string_lossy()
                    .to_string();
                
                // Construct destination path based on emulator type
                let bios_dest_dir = if driver_id == "duckstation" {
                    temp_work_dir.join(".duckstation_home/.local/share/duckstation/bios")
                } else {
                    temp_work_dir.join("pcsx2_data/PCSX2/bios")
                };
                std::fs::create_dir_all(&bios_dest_dir)
                    .map_err(|e| format!("Failed to create BIOS dir: {}", e))?;
                    
                let bios_dest_path = bios_dest_dir.join(&bios_filename);
                std::fs::copy(bios, &bios_dest_path)
                    .map_err(|e| format!("Failed to copy BIOS to config dir: {}", e))?;
            }
        }
    }
    
    // Add PCSX2 config if exists
    let pcsx2_config_dir = temp_work_dir.join("pcsx2_data");
    println!("📂 Vérification pcsx2_data: {:?} exists={}", pcsx2_config_dir, pcsx2_config_dir.exists());
    if driver_id == "pcsx2" && pcsx2_config_dir.exists() {
        // List contents for debug
        if let Ok(entries) = std::fs::read_dir(&pcsx2_config_dir) {
            for entry in entries {
                if let Ok(e) = entry {
                    println!("   📄 {:?}", e.path());
                }
            }
        }
        add_directory_to_zip(&app, &mut zip, &pcsx2_config_dir, "pcsx2_data", options)?;
        println!("✅ pcsx2_data ajouté au ZIP");
    } else {
        println!("⚠️ pcsx2_data NON ajouté - driver_id={} exists={}", driver_id, pcsx2_config_dir.exists());
    }

    // DuckStation: Add .duckstation_home to ZIP (setup_environment a déjà créé le contenu)
    if driver_id == "duckstation" {
        let duckstation_home = temp_work_dir.join(".duckstation_home");
        if duckstation_home.exists() {
            add_directory_to_zip(&app, &mut zip, &duckstation_home, ".duckstation_home", options)?;
        }
    }
    
    zip.finish().map_err(|e| format!("Failed to finalize ZIP: {}", e))?;
    
    // Step 3: Create the portable config JSON
    // Récupérer les env_vars et args depuis le plugin
    let config_dir_name = if driver_id == "duckstation" { "./.duckstation_home" } else { "./pcsx2_data" };
    
    // Obtenir les configurations de lancement depuis le plugin
    let (env_vars_list, args_before, args_after) = if let Some(plugin) = manager.configured_driver_for(&emulator_path) {
        let config_path = PathBuf::from(config_dir_name);
        let env_vars = plugin.portable_env_vars(&config_path);
        let (before, after) = plugin.portable_launch_args(fullscreen);
        (env_vars, before, after)
    } else {
        // Fallback générique
        let before = if fullscreen { vec!["--fullscreen".to_string()] } else { vec![] };
        (vec![], before, vec![])
    };
    
    let portable_config = serde_json::json!({
        "game_name": sanitize_filename(&game_name),
        "emulator_filename": emu_filename,
        "rom_filename": rom_filename,
        "config_dir": config_dir_name,
        "fullscreen": fullscreen,
        "env_vars": env_vars_list,
        "args_before_rom": args_before,
        "args_after_rom": args_after,
        "driver_id": driver_id  // Identifiant du plugin pour la détection dynamique
    });
    let config_json = serde_json::to_vec(&portable_config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    // Step 4: Concatenate: stub + marker + config_len + config + zip
    let output_path = output_dir_canonical.join(sanitize_filename(&game_name));
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
    println!("🧹 Suppression du dossier temporaire: {:?}", temp_work_dir);
    let _ = std::fs::remove_dir_all(&temp_work_dir);
    
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
            println!("   📁 Dossier ZIP: {}", name);
            add_directory_to_zip(app, zip, &path, &name, options)?;
        } else {
            println!("   📄 Fichier ZIP: {} ({} bytes)", name, path.metadata().map(|m| m.len()).unwrap_or(0));
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
    let candidates = vec!["ppsspp", "pcsx2", "duckstation", "dolphin", "cemu", "rpcs3", "ryujinx", "xemu"];
    
    let mut installed = vec![];
    for id in candidates {
        if downloader.is_installed(id) {
            installed.push(id.to_string());
        }
    }
    Ok(installed)
}

#[tauri::command]
fn quit_app(app: tauri::AppHandle) {
    app.exit(0);
}

#[tauri::command]
fn detect_platform(path: String) -> String {
    use emuforge_core::detection::{FileAnalyzer};
    println!("🔍 Analyzing ROM: {}", path);
    let path = PathBuf::from(path);
    let platform = FileAnalyzer::identify_platform(&path);
    println!("✅ Detected Platform: {:?}", platform);
    platform.as_str().to_string()
}

#[tauri::command]
fn get_emu_requirements(plugin_id: String) -> Result<emuforge_core::plugin::RequirementInfo, String> {
    use emuforge_core::plugin::manager::PluginManager;
    let manager = PluginManager::new();
    
    // Get plugin directly by ID - no path needed for requirements check
    if let Some(plugin) = manager.get_plugin_by_id(&plugin_id) {
        Ok(plugin.get_requirements())
    } else {
        // Fallback or error
        Ok(emuforge_core::plugin::RequirementInfo::default())
    }
}

#[tauri::command]
fn validate_emu_requirements(plugin_id: String, source_path: Option<String>) -> Result<emuforge_core::plugin::ValidationResult, String> {
    use emuforge_core::plugin::manager::PluginManager;
    let manager = PluginManager::new();
    
    if let Some(plugin) = manager.get_plugin_by_id(&plugin_id) {
        let path = source_path.map(PathBuf::from);
        plugin.validate_requirements(path.as_deref())
            .map_err(|e| format!("Validation error: {}", e))
    } else {
        Err("Plugin not found".to_string())
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
        .invoke_handler(tauri::generate_handler![
            forge_executable, 
            validate_file, 
            download_emulator, 
            get_installed_emulators, 
            quit_app, 
            detect_platform,
            get_emu_requirements,
            validate_emu_requirements
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
