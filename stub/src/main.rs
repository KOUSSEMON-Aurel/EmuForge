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
    let cache_dir = cache_base.join(&config.game_name);
    
    // Check if already extracted
    let marker_file = cache_dir.join(".emuforge_extracted");
    let needs_extraction = !marker_file.exists();
    
    if needs_extraction {
        eprintln!("🎮 Préparation du jeu (première exécution)...");
        
        // Extract the embedded ZIP archive
        if let Err(e) = extract_embedded_archive(&exe_path, &cache_dir) {
            eprintln!("❌ Erreur d'extraction: {}", e);
            std::process::exit(1);
        }
        
        // Create marker file
        let _ = File::create(&marker_file);
        eprintln!("✅ Extraction terminée !");
    }
    
    // Build paths to extracted files
    let emulator_path = cache_dir.join(&config.emulator_filename);
    let rom_path = cache_dir.join(&config.rom_filename);
    let config_path = cache_dir.join(&config.config_dir);
    
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
    // DuckStation syntax: [flags] -- <file>
    cmd.arg("-fullscreen");
    cmd.arg("--");
    cmd.arg(&rom_path);
    cmd.env("XDG_CONFIG_HOME", &config_path);
    
    match cmd.status() {
        Ok(status) => {
            if !status.success() {
                eprintln!("Emulator exited with status: {:?}", status.code());
                std::process::exit(status.code().unwrap_or(1));
            }
        }
        Err(e) => {
            eprintln!("Failed to launch emulator: {}", e);
            eprintln!("Path: {:?}", emulator_path);
            std::process::exit(1);
        }
    }
}

/// Extract the embedded ZIP archive to cache directory
fn extract_embedded_archive(exe_path: &PathBuf, cache_dir: &PathBuf) -> io::Result<()> {
    let file = File::open(exe_path)?;
    let mut reader = BufReader::new(file);
    
    // Read entire file
    let mut buffer = Vec::new();
    reader.read_to_end(&mut buffer)?;
    
    // Find the marker
    let marker_pos = find_subsequence(&buffer, PORTABLE_MARKER)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Marker not found"))?;
    
    // Skip marker + config length + config data to find ZIP start
    let config_start = marker_pos + PORTABLE_MARKER.len();
    let config_len = u32::from_le_bytes([
        buffer[config_start],
        buffer[config_start + 1],
        buffer[config_start + 2],
        buffer[config_start + 3],
    ]) as usize;
    
    let zip_start = config_start + 4 + config_len;
    let zip_data = &buffer[zip_start..];
    
    // Create cache directory
    fs::create_dir_all(cache_dir)?;
    
    // Extract ZIP
    let cursor = io::Cursor::new(zip_data);
    let mut archive = zip::ZipArchive::new(cursor)?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = cache_dir.join(file.mangled_name());
        
        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    
    Ok(())
}

/// Run in launcher mode (original behavior)
fn run_launcher_mode() {
    // Config injected at compile time via env var EMUFORGE_CONFIG_PATH
    let config_json = include_str!(env!("EMUFORGE_CONFIG_PATH"));
    let config: LaunchConfig = serde_json::from_str(config_json)
        .expect("Failed to parse launch config");

    let mut cmd = Command::new(&config.emulator_path);
    
    // Standard convention: [options] [file]
    // Passing args before ROM is safer for most emulators (DuckStation, Dolphin, etc.)
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
