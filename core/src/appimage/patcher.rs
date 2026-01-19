use anyhow::{Result, Context};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::fs;

/// Patcher pour AppImages (modification et repackaging)
pub struct AppImagePatcher {
    appimagetool_path: PathBuf,
}

impl AppImagePatcher {
    pub fn new(appimagetool_path: PathBuf) -> Self {
        Self { appimagetool_path }
    }
    
    /// Patch une AppImage RPCS3 avec firmware et configurations
    pub fn patch_rpcs3(
        &self,
        original_appimage: &Path,
        dev_flash_path: &Path,
        output_dir: &Path,
    ) -> Result<PathBuf> {
        println!("🔧 Patching RPCS3 AppImage...");
        
        // 1. Extraire AppImage
        let squashfs = self.extract_appimage(original_appimage, output_dir)?;
        
        // 2. Copier dev_flash
        self.inject_firmware(&squashfs, dev_flash_path)?;
        
        // 3. Injecter le wrapper script
        self.inject_wrapper(&squashfs)?;
        
        // 4. Re-packager
        let patched = self.repackage_appimage(&squashfs, output_dir)?;
        
        // 5. Cleanup squashfs temp
        let _ = fs::remove_dir_all(&squashfs);
        
        println!("✅ AppImage patched successfully");
        Ok(patched)
    }
    
    fn extract_appimage(&self, appimage: &Path, work_dir: &Path) -> Result<PathBuf> {
        println!("  📂 Extracting AppImage...");
        
        // S'assurer que l'AppImage est exécutable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(appimage)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(appimage, perms)?;
        }
        
        // Extraire (crée squashfs-root/)
        let output = Command::new(appimage)
            .arg("--appimage-extract")
            .current_dir(work_dir)
            .output()
            .context("Failed to extract AppImage")?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!(
                "AppImage extraction failed: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
        
        let squashfs = work_dir.join("squashfs-root");
        if !squashfs.exists() {
            return Err(anyhow::anyhow!("squashfs-root not created after extraction"));
        }
        
        Ok(squashfs)
    }
    
    fn inject_firmware(&self, squashfs: &Path, dev_flash: &Path) -> Result<()> {
        println!("  💾 Injecting firmware...");
        
        let target = squashfs.join("usr/bin/dev_flash");
        
        // Vérifier si dev_flash contient un sous-dossier dev_flash (structure imbriquée)
        let inner_dev_flash = dev_flash.join("dev_flash");
        let source = if inner_dev_flash.exists() && inner_dev_flash.is_dir() {
            println!("     (using inner dev_flash structure)");
            inner_dev_flash
        } else {
            dev_flash.to_path_buf()
        };
        
        // Copier récursivement dev_flash
        copy_dir_all(&source, &target)
            .context("Failed to copy dev_flash into AppImage")?;
        
        Ok(())
    }
    
    fn inject_wrapper(&self, squashfs: &Path) -> Result<()> {
        println!("  ⚙️  Injecting wrapper script...");
        
        // Créer le wrapper script qui gère les deux modes
        let wrapper_script = r#"#!/bin/bash
# EmuForge RPCS3 Wrapper
# Configure XDG_CONFIG_HOME pour les deux modes (portable et shortcut)

APPDIR="$(dirname "$(readlink -f "$0")")"
REAL_EMULATOR="$APPDIR/usr/bin/rpcs3"
EMBEDDED_DEV_FLASH="$APPDIR/usr/bin/dev_flash"

# Configuration commune : XDG_CONFIG_HOME writable avec lien vers dev_flash
EMUFORGE_CONFIG="$HOME/.emuforge/rpcs3_config"
RPCS3_DIR="$EMUFORGE_CONFIG/rpcs3"
mkdir -p "$RPCS3_DIR/GuiConfigs"

# Toujours recréer le lien dev_flash (le point de montage change à chaque lancement)
rm -f "$RPCS3_DIR/dev_flash" 2>/dev/null
if [[ -d "$EMBEDDED_DEV_FLASH" ]]; then
    ln -s "$EMBEDDED_DEV_FLASH" "$RPCS3_DIR/dev_flash"
fi

# Créer les configs initiales si pas présentes
if [[ ! -f "$RPCS3_DIR/GuiConfigs/CurrentSettings.ini" ]]; then
    cat > "$RPCS3_DIR/GuiConfigs/CurrentSettings.ini" << 'EOCONFIG'
[main_window]
infoBoxEnabledWelcome=false
confirmationBoxExitGame=false
EOCONFIG
fi

export XDG_CONFIG_HOME="$EMUFORGE_CONFIG"
exec "$REAL_EMULATOR" "$@"
"#;
        
        // Sauvegarder l'AppRun original
        let apprun_path = squashfs.join("AppRun");
        let apprun_orig = squashfs.join("AppRun.orig");
        if apprun_path.exists() {
            fs::rename(&apprun_path, &apprun_orig)?;
        }
        
        // Écrire le nouveau wrapper
        fs::write(&apprun_path, wrapper_script)?;
        
        // Rendre exécutable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&apprun_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&apprun_path, perms)?;
        }
        
        Ok(())
    }
    
    fn repackage_appimage(&self, squashfs: &Path, output_dir: &Path) -> Result<PathBuf> {
        println!("  📦 Repackaging AppImage...");
        
        let output = output_dir.join("RPCS3-Patched.AppImage");
        
        let cmd_output = Command::new(&self.appimagetool_path)
            .arg(squashfs)
            .arg(&output)
            .output()
            .context("Failed to run appimagetool")?;
        
        if !cmd_output.status.success() {
            return Err(anyhow::anyhow!(
                "appimagetool failed: {}",
                String::from_utf8_lossy(&cmd_output.stderr)
            ));
        }
        
        // Rendre exécutable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&output)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&output, perms)?;
        }
        
        Ok(output)
    }
}

/// Copie récursive de répertoire
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let target = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), &target)?;
        }
    }
    
    Ok(())
}
