// Hide console window on Windows in release mode
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::Deserialize;
use std::env;
use std::fs::{self, File};
use std::io::{self, BufReader, Read};
use std::path::PathBuf;
use std::process::Command;

/// Marker that indicates the start of embedded data in portable mode
const PORTABLE_MARKER: &[u8] = b"EMUFORGE_PORTABLE_DATA_START";

#[derive(Deserialize)]
struct LaunchConfig {
    emulator_path: PathBuf,
    rom_path: PathBuf,
    #[allow(dead_code)]
    bios_path: Option<PathBuf>,
    args: Vec<String>,
    working_dir: Option<PathBuf>,
    env_vars: Vec<(String, String)>,
}

#[derive(Deserialize)]
struct PortableConfig {
    game_name: String,
    emulator_filename: String,
    rom_filename: String,
    config_dir: String,
}

fn main() {
    // Check if we're in portable mode by looking for the marker in ourselves
    let exe_path = env::current_exe().expect("Failed to get current exe path");
    
    if let Some(portable_config) = check_portable_mode(&exe_path) {
        run_portable_mode(exe_path, portable_config);
    } else {
        run_launcher_mode();
    }
}

/// Check if the executable contains embedded portable data
fn check_portable_mode(exe_path: &PathBuf) -> Option<PortableConfig> {
    let file = File::open(exe_path).ok()?;
    let mut reader = BufReader::new(file);
    
    // Read the entire file to find the marker
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer).ok()?;
    
    // Search for marker
    if let Some(pos) = find_subsequence(&buffer, PORTABLE_MARKER) {
        // Marker found! Read the config JSON that follows
        let config_start = pos + PORTABLE_MARKER.len();
        
        // Read 4 bytes for config length
        if buffer.len() < config_start + 4 {
            return None;
        }
        let config_len = u32::from_le_bytes([
            buffer[config_start],
            buffer[config_start + 1],
            buffer[config_start + 2],
            buffer[config_start + 3],
        ]) as usize;
        
        let config_data_start = config_start + 4;
        if buffer.len() < config_data_start + config_len {
            return None;
        }
        
        let config_json = &buffer[config_data_start..config_data_start + config_len];
        serde_json::from_slice(config_json).ok()
    } else {
        None
    }
}

/// Find subsequence in buffer
fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|window| window == needle)
}

/// Run in portable mode - extract and launch
fn run_portable_mode(exe_path: PathBuf, config: PortableConfig) {
    // Determine cache directory
    let cache_base = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("emuforge");
    let target_dir = cache_base.join(&config.game_name);
    
    // Check if already extracted
    let marker_file = target_dir.join(".emuforge_extracted");
    let needs_extraction = !marker_file.exists();
    
    if needs_extraction {
        eprintln!("🎮 Préparation du jeu: {}...", config.game_name);
        eprintln!("📁 Dossier de données: {:?}", target_dir);
        
        // Create cache directory
        fs::create_dir_all(&target_dir).expect("Failed to create cache directory");
        
        // Extract the embedded zip archive
        if let Err(e) = extract_embedded_archive(&exe_path, &target_dir) {
            eprintln!("❌ Erreur d'extraction: {}", e);
            std::process::exit(1);
        }
        
        // Create marker file
        let _ = File::create(&marker_file);
        eprintln!("✅ Extraction terminée !");
    }
    
    // Build paths to extracted files
    let emulator_path = target_dir.join(&config.emulator_filename);
    let rom_path = target_dir.join(&config.rom_filename);
    let config_path = target_dir.join(&config.config_dir);
    
    // Debug output
    eprintln!("🔍 DEBUG: Cache dir: {:?}", target_dir);
    eprintln!("🔍 DEBUG: Emulator path: {:?}", emulator_path);
    eprintln!("🔍 DEBUG: ROM path: {:?}", rom_path);
    eprintln!("🔍 DEBUG: ROM exists: {}", rom_path.exists());
    eprintln!("🔍 DEBUG: Config path: {:?}", config_path);
    
    // Make emulator executable (Linux)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = fs::metadata(&emulator_path) {
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&emulator_path, perms);
        }
    }
    
    // Launch the emulator
    let mut cmd = Command::new(&emulator_path);
    
    // CRITICAL: Set HOME to the config directory to isolate the emulator
    // This replicates the logic in our wrapper script and fixes the DuckStation error.
    cmd.env("HOME", &config_path);
    cmd.env("QT_QPA_PLATFORM", "xcb");
    
    // DuckStation syntax: [flags] -- <file>
    cmd.arg("-fullscreen");
    cmd.arg("--");
    cmd.arg(&rom_path);
    
    let mut child = cmd.spawn().expect("Failed to launch emulator");
    let _ = child.wait();
}

/// Extract the embedded ZIP archive from the executable
fn extract_embedded_archive(exe_path: &PathBuf, target_dir: &PathBuf) -> io::Result<()> {
    let mut file = File::open(exe_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    // Find the marker
    if let Some(pos) = find_subsequence(&buffer, PORTABLE_MARKER) {
        let config_start = pos + PORTABLE_MARKER.len();
        
        // Read config length (4 bytes)
        let config_len = u32::from_le_bytes([
            buffer[config_start],
            buffer[config_start + 1],
            buffer[config_start + 2],
            buffer[config_start + 3],
        ]) as usize;
        
        let zip_start = config_start + 4 + config_len;
        if buffer.len() > zip_start {
            let zip_data = &buffer[zip_start..];
            
            // Write ZIP to temporary file to use ZipArchive
            let zip_temp_path = target_dir.join("temp_data.zip");
            fs::write(&zip_temp_path, zip_data)?;
            
            // Extract ZIP
            let zip_file = File::open(&zip_temp_path)?;
            let mut archive = zip::ZipArchive::new(zip_file)?;
            
            for i in 0..archive.len() {
                let mut out_file = archive.by_index(i)?;
                let outpath = match out_file.enclosed_name() {
                    Some(path) => target_dir.join(path),
                    None => continue,
                };
                
                if out_file.name().ends_with('/') {
                    fs::create_dir_all(&outpath)?;
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            fs::create_dir_all(&p)?;
                        }
                    }
                    let mut outfile = File::create(&outpath)?;
                    io::copy(&mut out_file, &mut outfile)?;
                }
            }
            
            // Remove temporary ZIP
            let _ = fs::remove_file(&zip_temp_path);
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "ZIP data not found"))
        }
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "Marker not found"))
    }
}

/// Run in launcher mode - use local config
fn run_launcher_mode() {
    // Current behavior for non-portable mode
    // (Actual implementation depends on how you want the stub to behave as a standalone)
    // For now, it might just be the launcher itself or a placeholder.
    eprintln!("EmuForge Stub: Launcher mode not implemented yet.");
}
