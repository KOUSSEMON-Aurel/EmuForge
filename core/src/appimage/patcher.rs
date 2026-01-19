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

# ----------------------------------------------------------------------
# DETECTION INTELLIGENTE DES CONTROLEURS
# ----------------------------------------------------------------------
mkdir -p "$RPCS3_DIR/input_configs/global"
CONFIG_FILE="$RPCS3_DIR/input_configs/global/Default.yml"

# Fonction pour générer config CLAVIER (Fallback)
gen_keyboard_config() {
    cat > "$CONFIG_FILE" << 'EOKEY'
Player 1 Input:
  Handler: Keyboard
  Device: Keyboard
  Profile: Default
  Config:
    Left Stick Left: A
    Left Stick Down: S
    Left Stick Right: D
    Left Stick Up: W
    Right Stick Left: J
    Right Stick Down: K
    Right Stick Right: L
    Right Stick Up: I
    Start: Return
    Select: Backspace
    PS Button: Escape
    Square: Q
    Cross: E
    Circle: R
    Triangle: T
    Left: Left
    Down: Down
    Right: Right
    Up: Up
    R1: O
    R2: P
    L1: U
    L2: Y
Player 2 Input:
  Handler: "Null"
EOKEY
}

# Fonction pour convertir nom Linux en nom SDL
linux_to_sdl_name() {
    local linux_name="$1"
    
    # Table de correspondance Linux -> SDL
    case "$linux_name" in
        # Xbox 360
        "Microsoft X-Box 360 pad"|"Xbox 360 Wired Controller")
            echo "Xbox 360 Controller" ;;
        "Xbox 360 Wireless Receiver"*)
            echo "Xbox 360 Wireless Controller" ;;
        
        # Xbox One / Series
        "Microsoft X-Box One pad"|"Microsoft X-Box One S pad"|"Xbox Wireless Controller")
            echo "Xbox One Controller" ;;
        "Microsoft Xbox Series S|X Controller")
            echo "Xbox Series X Controller" ;;
        
        # PlayStation
        "Sony PLAYSTATION(R)3 Controller"|"PLAYSTATION(R)3 Controller")
            echo "PS3 Controller" ;;
        "Sony Computer Entertainment Wireless Controller"|"Wireless Controller")
            echo "Sony Interactive Entertainment Wireless Controller" ;;
        "Sony Interactive Entertainment DualSense Wireless Controller"|"DualSense Wireless Controller")
            echo "DualSense Wireless Controller" ;;
        
        # Nintendo
        "Nintendo Switch Pro Controller"|"Pro Controller")
            echo "Pro Controller" ;;
        "Nintendo Co., Ltd. Pro Controller")
            echo "Pro Controller" ;;
        
        # Générique - garder tel quel
        *)
            echo "$linux_name" ;;
    esac
}

# Fonction pour obtenir le nom SDL via la lib embarquée
get_sdl_name() {
    # Essayer d'utiliser la lib SDL de RPCS3 via Python (rapide, ~50ms)
    local sdl_lib="$APPDIR/usr/lib/libSDL2.so.0"
    if [[ -f "$sdl_lib" ]] && command -v python3 &>/dev/null; then
        local sdl_name
        sdl_name=$(python3 -c "
import ctypes
try:
    sdl = ctypes.CDLL('$sdl_lib')
    sdl.SDL_Init(0x200)  # SDL_INIT_JOYSTICK
    if sdl.SDL_NumJoysticks() > 0:
        sdl.SDL_JoystickNameForIndex.restype = ctypes.c_char_p
        name = sdl.SDL_JoystickNameForIndex(0)
        if name:
            print(name.decode())
    sdl.SDL_Quit()
except:
    pass
" 2>/dev/null)
        if [[ -n "$sdl_name" ]]; then
            echo "$sdl_name"
            return 0
        fi
    fi
    return 1
}

# Fonction pour générer config MANETTE (SDL - Universel multi-plateforme)
gen_gamepad_config() {
    local linux_name="$1"
    local sdl_device_name
    
    # 1. Essayer d'obtenir le nom SDL exact via la lib
    sdl_device_name=$(get_sdl_name)
    
    # 2. Sinon, convertir le nom Linux en nom SDL via la table
    if [[ -z "$sdl_device_name" ]]; then
        sdl_device_name=$(linux_to_sdl_name "$linux_name")
    fi
    
    # RPCS3 ajoute un numéro d'instance au nom du device
    # Ex: "Xbox 360 Controller" devient "X360 Controller 1"
    # On utilise le nom SDL détecté + " 1" pour le premier contrôleur
    local rpcs3_device_name
    if [[ "$sdl_device_name" == *"Xbox 360"* ]]; then
        rpcs3_device_name="X360 Controller 1"
    else
        # Pour les autres manettes, on garde le nom SDL + " 1"
        rpcs3_device_name="$sdl_device_name 1"
    fi
    
    # Générer le fichier avec le Device: correct
    cat > "$CONFIG_FILE" << EOPAD
Player 1 Input:
  Handler: SDL
  Device: $rpcs3_device_name
  Profile: Default
  Config:
    Left Stick Left: LS X-
    Left Stick Down: LS Y-
    Left Stick Right: LS X+
    Left Stick Up: LS Y+
    Right Stick Left: RS X-
    Right Stick Down: RS Y-
    Right Stick Right: RS X+
    Right Stick Up: RS Y+
    Start: Start
    Select: Back
    PS Button: Guide
    Square: West
    Cross: South
    Circle: East
    Triangle: North
    Left: Left
    Down: Down
    Right: Right
    Up: Up
    R1: RB
    R2: RT
    R3: RS
    L1: LB
    L2: LT
    L3: LS
Player 2 Input:
  Handler: Keyboard
  Device: Keyboard
  Profile: Default
  Config:
    Left Stick Left: Left
    Left Stick Down: Down
    Left Stick Right: Right
    Left Stick Up: Up
    Start: Return
    Select: Backspace
    Cross: Space
    Circle: Escape
Player 3 Input:
  Handler: "Null"
  Device: ""
Player 4 Input:
  Handler: "Null"
  Device: ""
Player 5 Input:
  Handler: "Null"
  Device: ""
Player 6 Input:
  Handler: "Null"
  Device: ""
Player 7 Input:
  Handler: "Null"
  Device: ""
EOPAD
}

# 1. Vérifier si une manette est connectée (/dev/input/js*)
JS_DEVICE=$(ls /dev/input/js* 2>/dev/null | head -n 1)

if [[ -z "$JS_DEVICE" ]]; then
    # PAS DE MANETTE -> Config Clavier
    gen_keyboard_config
else
    # MANETTE DETECTEE -> Essayer de trouver son nom
    # Le nom est dans /sys/class/input/jsX/device/name
    JS_NAME=$(basename "$JS_DEVICE") # js0
    NAME_FILE="/sys/class/input/$JS_NAME/device/name"
    
    if [[ -f "$NAME_FILE" ]]; then
        GAMEPAD_NAME=$(cat "$NAME_FILE")
        # Nettoyer le nom (enlever retour à la ligne)
        GAMEPAD_NAME=$(echo "$GAMEPAD_NAME" | tr -d '\n')
        
        # Générer la config avec ce nom spécifique
        gen_gamepad_config "$GAMEPAD_NAME"
    else
        # Fallback si on peut pas lire le nom : Clavier (pour éviter plantage)
        gen_keyboard_config
    fi
fi

# 2. CRÉER CONFIG.YML TEMPLATE SI ABSENT
# Si on crée le config AVANT le 1er lancement RPCS3, il ne le regénère pas
GLOBAL_CONFIG="$RPCS3_DIR/config.yml"
if [[ ! -f "$GLOBAL_CONFIG" ]]; then
    cat > "$GLOBAL_CONFIG" << 'EOGLOBAL'
Meta:
  CheckUpdateStart: false
Input/Output:
  Keyboard: Keyboard
  Mouse: Basic
  Camera: "Null"
  Camera type: Unknown
  Camera flip: None
  Camera ID: Default
  SDL Camera ID: Default
  Move: "Null"
  Buzz emulated controller: "Null"
  Turntable emulated controller: "Null"
  GHLtar emulated controller: "Null"
  Pad handler mode: Multi-threaded
  Keep pads connected: false
  Pad handler sleep (microseconds): 1000
  Background input enabled: true
  Show move cursor: false
  Paint move spheres: false
  Allow move hue set by game: false
  Lock overlay input to player one: false
  Emulated Midi devices: ßßß@@@ßßß@@@ßßß@@@
  Load SDL GameController Mappings: true
  IO Debug overlay: false
  Mouse Debug overlay: false
  Fake Move Rotation Cone: 10
  Fake Move Rotation Cone (Vertical): 10
System:
  License Area: SCEA
  Language: English (US)
  Keyboard Type: English keyboard (US standard)
  Enter button assignment: Enter with cross
EOGLOBAL
fi

export XDG_CONFIG_HOME="$EMUFORGE_CONFIG"
# Fix SDL priority (juste au cas où)
export SDL_JOYSTICK_DEVICE="/dev/input/js0"
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
