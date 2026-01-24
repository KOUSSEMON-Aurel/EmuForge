use anyhow::{Context, Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;
use reqwest;


pub struct EmulatorDownloader {
    base_dir: PathBuf,
}

impl EmulatorDownloader {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    fn get_url(&self, emu_id: &str) -> Option<(&'static str, &'static str)> {
        // Returns (url, filename_in_archive_or_archive_name)
        // TODO: This should ideally be a dynamic manifest fetched from a server.
        // For this implementation, we hardcode stable links (or placeholders) for demonstration.
        // REAL URLS ARE NEEDED FOR PRODUCTION.
        
        let is_windows = cfg!(target_os = "windows");
        
        match (emu_id, is_windows) {
            // PPSSPP - Pinned Versions
            // Windows: v1.17.1 (Last stable ZIP on GitHub)
            // Linux: v1.19.3 (AppImage)
            ("ppsspp", true) => Some(("https://github.com/hrydgard/ppsspp/releases/download/v1.17.1/PPSSPPWindows.zip", "PPSSPPWindows.zip")),
            ("ppsspp", false) => Some(("https://github.com/hrydgard/ppsspp/releases/download/v1.19.3/PPSSPP-v1.19.3-anylinux-x86_64.AppImage", "PPSSPP.AppImage")),
            
            // PCSX2 - Pinned v2.2.0 (Stable)
            ("pcsx2", true) => Some(("https://github.com/PCSX2/pcsx2/releases/download/v2.2.0/pcsx2-v2.2.0-windows-x64-Qt.7z", "pcsx2-windows.7z")),
            ("pcsx2", false) => Some(("https://github.com/PCSX2/pcsx2/releases/download/v2.2.0/pcsx2-v2.2.0-linux-appimage-x64-Qt.AppImage", "PCSX2.AppImage")),
            
            // DuckStation - Pinned v0.1-10530 (Specific Rolling Tag)
            ("duckstation", true) => Some(("https://github.com/stenzek/duckstation/releases/download/v0.1-10530/duckstation-windows-x64-release.zip", "duckstation-windows.zip")),
            ("duckstation", false) => Some(("https://github.com/stenzek/duckstation/releases/download/v0.1-10530/DuckStation-x64.AppImage", "DuckStation.AppImage")),

            // Dolphin - Pinned 5.0-19870
            ("dolphin", true) => Some(("https://dl.dolphin-emu.org/releases/202309/dolphin-master-5.0-19870-x64.7z", "dolphin-x64.7z")),
            ("dolphin", false) => Some(("https://github.com/pkgforge-dev/Dolphin-emu-AppImage/releases/download/2512%402026-01-15_1768463428/Dolphin_Emulator-2512-anylinux.squashfs-x86_64.AppImage", "Dolphin.AppImage")),
            
            // RPCS3 - Pinned v0.0.39-18703
            ("rpcs3", true) => Some(("https://github.com/RPCS3/rpcs3-binaries-win/releases/download/build-eaebd3426e7050c35beb8f24952d6da4d6a75360/rpcs3-v0.0.39-18703-eaebd342_win64_msvc.7z", "rpcs3-windows.7z")),
            ("rpcs3", false) => Some(("https://github.com/RPCS3/rpcs3-binaries-linux/releases/download/build-eaebd3426e7050c35beb8f24952d6da4d6a75360/rpcs3-v0.0.39-18703-eaebd342_linux64.AppImage", "RPCS3.AppImage")),

            // Cemu - Pinned v2.6 (Latest stable Wii U emulator)
            ("cemu", true) => Some(("https://github.com/cemu-project/Cemu/releases/download/v2.6/cemu-2.6-windows-x64.zip", "cemu-windows.zip")),

            ("cemu", false) => Some(("https://github.com/cemu-project/Cemu/releases/download/v2.6/Cemu-2.6-x86_64.AppImage", "Cemu.AppImage")),

            // Ryujinx - Ryubing Fork v1.3.3 (Stable)
            ("ryujinx", true) => Some(("https://git.ryujinx.app/api/v4/projects/1/packages/generic/Ryubing/1.3.3/ryujinx-1.3.3-win_x64.zip", "ryujinx-windows.zip")),
            ("ryujinx", false) => Some(("https://git.ryujinx.app/api/v4/projects/1/packages/generic/Ryubing/1.3.3/ryujinx-1.3.3-x64.AppImage", "Ryujinx.AppImage")),

            // xemu - v0.8.133 (Xbox Original Emulator)
            ("xemu", true) => Some(("https://github.com/xemu-project/xemu/releases/download/v0.8.133/xemu-0.8.133-windows-x86_64.zip", "xemu-windows.zip")),
            ("xemu", false) => Some(("https://github.com/xemu-project/xemu/releases/download/v0.8.133/xemu-0.8.133-x86_64.AppImage", "xemu.AppImage")),

            // extract-xiso - Tool for XISO conversion
            ("extract-xiso", true) => Some(("https://github.com/XboxDev/extract-xiso/releases/download/build-202505152050/extract-xiso-Win64_Release.zip", "extract-xiso.zip")),
            ("extract-xiso", false) => Some(("https://github.com/XboxDev/extract-xiso/releases/download/build-202505152050/extract-xiso_Linux.zip", "extract-xiso.zip")),

            // Flycast - Dreamcast Emulator (v2.6 Stable)
            ("flycast", true) => Some(("https://github.com/flyinghead/flycast/releases/download/v2.6/flycast-win64-2.6.zip", "flycast.zip")),
            ("flycast", false) => Some(("https://github.com/flyinghead/flycast/releases/download/v2.6/flycast-x86_64.AppImage", "flycast.AppImage")),

            // Lime3DS/Azahar - 3DS Emulator (Azahar v2124.1 Stable)
            ("lime3ds", true) => Some(("https://github.com/azahar-emu/azahar/releases/download/2124.1/azahar-2124.1-windows-msvc.zip", "azahar.zip")),
            ("lime3ds", false) => Some(("https://github.com/azahar-emu/azahar/releases/download/2124.1/azahar.AppImage", "azahar.AppImage")),

            // Other emulators would need manual implementation or usage of system packages
            _ => None
        }
    }

    fn get_binary_name(&self, emu_id: &str) -> String {
        if cfg!(target_os = "windows") {
             match emu_id {
                 "ppsspp" => "PPSSPPWindows64.exe",
                 "duckstation" => "duckstation-qt-x64-ReleaseLTCG.exe",
                 "pcsx2" => "pcsx2-qtx64.exe", 
                 "dolphin" => "Dolphin.exe",
                 "rpcs3" => "rpcs3.exe",
                 "cemu" => "Cemu.exe",
                 "ryujinx" => "Ryujinx.exe",
                 "xemu" => "xemu.exe",
                 "extract-xiso" => "extract-xiso.exe",
                 "flycast" => "flycast.exe",
                 "lime3ds" => "azahar.exe", // Exécutable Windows
                 _ => "emulator.exe"
             }.to_string()
        } else {
             match emu_id {
                 "ppsspp" => "PPSSPP.AppImage",
                 "duckstation" => "DuckStation.AppImage",
                 "pcsx2" => "PCSX2.AppImage",
                 "dolphin" => "Dolphin.AppImage", 
                 "rpcs3" => "RPCS3.AppImage",
                 "cemu" => "Cemu.AppImage", 
                 "ryujinx" => "Ryujinx.AppImage",
                 "xemu" => "xemu.AppImage",
                 "extract-xiso" => "extract-xiso",
                 "flycast" => "flycast.AppImage",
                 "lime3ds" => "azahar.AppImage", // Nom exact téléchargé
                 _ => "emulator"
             }.to_string()
        }
    }

    fn find_binary_recursive(&self, dir: &Path, binary_name: &str, depth: usize) -> Option<PathBuf> {
        if depth == 0 { return None; }
        if !dir.exists() { return None; }

        // Check current dir
        let candidate = dir.join(binary_name);
        if candidate.exists() {
            return Some(candidate);
        }

        // Search subdirs
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(found) = self.find_binary_recursive(&path, binary_name, depth - 1) {
                        return Some(found);
                    }
                }
            }
        }
        None
    }

    pub fn is_installed(&self, emu_id: &str) -> bool {
        let install_dir = self.base_dir.join(emu_id);
        let binary_name = self.get_binary_name(emu_id);
        
        let found = self.find_binary_recursive(&install_dir, &binary_name, 3).is_some();
        if found {
             println!("DEBUG: Found installed {} binary: {}", emu_id, binary_name);
        } else {
             println!("DEBUG: Could not find {} binary: {} in {:?}", emu_id, binary_name, install_dir);
        }
        found
    }

    pub async fn download(&self, emu_id: &str) -> Result<PathBuf> {
        let install_dir = self.base_dir.join(emu_id);
        let binary_name = self.get_binary_name(emu_id);
        
        // Check if already installed
        if let Some(existing) = self.find_binary_recursive(&install_dir, &binary_name, 3) {
             println!("Emulator {} already exists at {:?}", emu_id, existing);
             return Ok(existing);
        }

        fs::create_dir_all(&install_dir).context("Failed to create emu dir")?;


        let (url, archive_name) = self.get_url(emu_id)
            .ok_or_else(|| anyhow!("Download URL not defined for {} on this OS", emu_id))?;

        println!("Downloading {} from {}...", emu_id, url);
        
        
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .context("Failed to build http client")?;

        let response = client.get(url).send().await.context("Failed to fetch URL")?;
        let bytes = response.bytes().await.context("Failed to get bytes")?;

        // Save to temporary file to allow seeking (required for 7z and efficient for others)
        let temp_archive_path = install_dir.join(archive_name);
        fs::write(&temp_archive_path, &bytes).context("Failed to write temp archive")?;

        // Extract based on extension
        if archive_name.ends_with(".zip") {
            let file = fs::File::open(&temp_archive_path)?;
            let mut archive = zip::ZipArchive::new(file).context("Failed to open ZIP archive")?;
            archive.extract(&install_dir).context("Failed to extract ZIP")?;
        } else if archive_name.ends_with(".tar.gz") {
            let file = fs::File::open(&temp_archive_path)?;
            let tar = flate2::read::GzDecoder::new(file);
            let mut archive = tar::Archive::new(tar);
            archive.unpack(&install_dir).context("Failed to unpack TAR.GZ")?;
        } else if archive_name.ends_with(".7z") {
            sevenz_rust::decompress_file(&temp_archive_path, &install_dir)
                .context("Failed to extract 7z archive")?;
        } else if archive_name.ends_with(".AppImage") {
             // AppImage IS the binary.
        } else {
             // Generic file
        }
        
        // Clean up archive if extracted
        if archive_name.ends_with(".zip") || archive_name.ends_with(".tar.gz") || archive_name.ends_with(".7z") {
             let _ = fs::remove_file(&temp_archive_path);
        }

        // After extraction, find binary using recursive search
        if let Some(found) = self.find_binary_recursive(&install_dir, &binary_name, 3) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(metadata) = fs::metadata(&found) {
                    let mut perms = metadata.permissions();
                    perms.set_mode(0o755);
                    let _ = fs::set_permissions(&found, perms);
                }
            }
            return Ok(found);
        }
        
        // Fallback
        Ok(install_dir.join(binary_name))
    }
    
    /// Télécharge appimagetool pour le patching d'AppImages
    pub async fn download_appimagetool(&self) -> Result<PathBuf> {
        let tools_dir = self.base_dir.parent()
            .unwrap_or(&self.base_dir)
            .join("tools");
        fs::create_dir_all(&tools_dir)
            .context("Failed to create tools directory")?;
        
        let output_path = tools_dir.join("appimagetool");
        
        // Vérifier si déjà téléchargé
        if output_path.exists() {
            println!("appimagetool already exists");
            return Ok(output_path);
        }
        
        let url = "https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage";
        
        println!("Downloading appimagetool from {}...", url);
        let response = reqwest::get(url).await.context("Failed to fetch appimagetool")?;
        let bytes = response.bytes().await.context("Failed to download appimagetool bytes")?;
        
        fs::write(&output_path, bytes).context("Failed to write appimagetool")?;
        
        // Rendre exécutable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&output_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&output_path, perms)?;
        }
        
        println!("✅ appimagetool downloaded successfully");
        Ok(output_path)
    }
}



