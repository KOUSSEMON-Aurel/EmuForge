use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct CemuPlugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl CemuPlugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

const CEMU_DEFAULT_SETTINGS: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<content>
    <logflag>0</logflag>
    <check_update>true</check_update>
    <fullscreen>false</fullscreen>
    <Graphic>
        <api>1</api>
        <GX2DrawdoneSync>true</GX2DrawdoneSync>
        <AsyncCompile>true</AsyncCompile>
        <Overlay>
            <FPS>true</FPS>
        </Overlay>
    </Graphic>
    <Audio>
        <api>0</api>
        <delay>2</delay>
        <TVChannels>2</TVChannels>
        <PadChannels>2</PadChannels>
        <TVVolume>50</TVVolume>
        <TVDevice></TVDevice>
    </Audio>
</content>
"#;

impl EmulatorPlugin for CemuPlugin {
    fn id(&self) -> &str { "cemu" }
    fn name(&self) -> &str { "Cemu (Wii U)" }
    fn supported_extensions(&self) -> &[&str] { &["wua", "wud", "wux", "rpx", "elf"] } 

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        if let Ok(path) = which::which("cemu") { return Ok(path); }
        if let Ok(path) = which::which("Cemu") { return Ok(path); }
        
        Err(anyhow::anyhow!("Cemu executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, _output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate Cemu binary")?;
        
        // Cemu Args: -g <game_path> -f (fullscreen)
        let args = vec![
            "-g".to_string(),
        ];
        // Note: rom_path will be appended by Stub/LaunchConfig logic. 
        // Wait, Cemu requires -g BEFORE the path? 
        // LaunchConfig logic in stub: cmd.args(args).arg(rom_path).
        // This results in `cemu -g <rom_path>`. This is Correct.
        
        Ok(LaunchConfig {
            emulator_path: binary,
            rom_path: rom_path.to_path_buf(),
            args,
            ..Default::default()
        })
    }

    fn can_handle(&self, binary_path: &Path) -> bool {
        let name = binary_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        name.contains("cemu")
    }

    /// Cemu n'utilise pas d'args fullscreen avant la ROM
    fn fullscreen_args(&self) -> Vec<String> {
        vec![]  // -f doit être APRÈS la ROM, pas avant
    }

    /// -f pour fullscreen doit être après le chemin du jeu
    fn fullscreen_args_after_rom(&self) -> Vec<String> {
        vec!["-f".to_string()]
    }

    fn setup_environment(&self, _output_dir: &Path, _bios_path: Option<&Path>) -> Result<()> {
        // Déployer keys.txt automatiquement depuis les assets intégrés
        // Le chemin est relatif à ce fichier source : ../assets/keys.txt
        let keys_content = include_str!("../assets/keys.txt");
        
        // Déterminer les dossiers de config cibles
        let mut targets = Vec::new();

        if cfg!(target_os = "linux") {
            if let Some(home) = dirs::home_dir() {
                targets.push(home.join(".local/share/Cemu/keys.txt"));
                targets.push(home.join(".config/Cemu/keys.txt"));
            }
        } else if cfg!(target_os = "windows") {
             // Sur Windows, c'est souvent dans le dossier de l'exécutable ou %APPDATA%
             // Pour l'instant on se concentre sur Linux comme demandé par l'user
        }

        for target in targets {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).context("Failed to create Cemu config dir")?;
            }
            std::fs::write(&target, keys_content).context("Failed to write keys.txt")?;
            eprintln!("✅ Cemu keys installed to: {:?}", target);
        }
        
        // Copie aussi dans le dossier de sortie (pour le mode portable)
        let local_target = _output_dir.join("keys.txt");
        if let Ok(_) = std::fs::write(&local_target, keys_content) {
             eprintln!("✅ Copie locale de keys.txt réussie (pour mode portable)");
        }

        // --- SETTINGS.XML (AUDIO FIX) ---
        // On force des paramètres sains (Audio Stéréo, Vulkan) pour éviter les crashs.
        // Cemu utilise ~/.config/Cemu/settings.xml sur Linux.
        if cfg!(target_os = "linux") {
            if let Some(home) = dirs::home_dir() {
                let config_file = home.join(".config/Cemu/settings.xml");
                // On écrase pour être sûr de réparer (Plug & Play) ou on vérifie si ça existe ?
                // User a validé "vas y" pour la réparation auto.
                if let Some(parent) = config_file.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = std::fs::write(&config_file, CEMU_DEFAULT_SETTINGS) {
                    eprintln!("⚠️ Failed to write Cemu settings.xml: {}", e);
                } else {
                    eprintln!("✅ Cemu settings.xml repaired (Audio Stereo Fix)");
                }
            }
        }

        // --- SMART CONTROLLER CONFIGURATION ---
        // Logic:
        // - Count connected gamepads (checking /dev/input/js*)
        // - P1 assigned to Gamepad if present, otherwise Keyboard
        // - P2 assigned to Gamepad (if >1) or Keyboard (if P1 is Gamepad)
        // - We overwrite controller0.xml and controller1.xml to enforce this "Plug & Play" behavior
        
        // --- SMART CONTROLLER CONFIGURATION WITH SDL2 ---
        // Logic:
        // - Init SDL2
        // - Enumerate GameControllers (better than raw Joysticks for mapping, but Cemu uses raw GUIDs often same as disk)
        // - If found, get Name and GUID
        // - Inject into XML template
        // - Write to controller0.xml
        
        let mut gamepad_count = 0;
        
        // Use a block to ensure SDL context is dropped or we handle it gracefully
        // Note: Initializing SDL in a plugin might be tricky if the main app also uses it, 
        // but here we just need it for a split second to check devices.
        if let Ok(sdl_context) = sdl2::init() {
            // We need both subsystems: 
            // - GameController to check if it's a supported gamepad (mapping available)
            // - Joystick to get the GUID (GameControllerSubsystem doesn't expose it by index directly in some versions)
            let gc_subsystem = sdl_context.game_controller();
            let joy_subsystem = sdl_context.joystick();

            if let (Ok(gc), Ok(joy)) = (gc_subsystem, joy_subsystem) {
                 let available = joy.num_joysticks().unwrap_or(0);
                 eprintln!("🎮 SDL2 Detected {} potential devices", available);
                 
                 if available > 0 {
                     for i in 0..available {
                         if gc.is_game_controller(i) {
                             // It's a recognized controller
                             // Get GUID via Joystick Subsystem
                             let guid = joy.device_guid(i).map(|g| g.to_string()).unwrap_or("".to_string());
                             let name = gc.name_for_index(i).unwrap_or("Unknown Controller".to_string());
                             
                             eprintln!("🎮 Gamepad #{} Found: '{}' GUID: {}", i, name, guid);
                             
                             gamepad_count += 1;
                             
                             // We only support auto-configuring P1 for now with the first found controller
                             // P2 will be keyboard if P1 is gamepad
                             if gamepad_count == 1 {
                                 if cfg!(target_os = "linux") {
                                    if let Some(home) = dirs::home_dir() {
                                        let profile_dir = home.join(".config/Cemu/controllerProfiles");
                                        let _ = std::fs::create_dir_all(&profile_dir);
                                        
                                        let mut sdl_content = include_str!("../assets/cemu_controller_sdl.xml").to_string();
                                        sdl_content = sdl_content.replace("{GUID}", &guid);
                                        sdl_content = sdl_content.replace("{NAME}", &name);
                                        
                                        let p1_file = profile_dir.join("controller0.xml");
                                        if let Ok(_) = std::fs::write(&p1_file, sdl_content) {
                                            eprintln!("✅ P1 Configured with SDL Controller: {}", name);
                                        }
                                        
                                        // Auto-configure P2 as Keyboard (Pro Controller) so it's usable
                                        let kbd_template = include_str!("../assets/cemu_controller_keyboard.xml");
                                        let p2_content = kbd_template.replace("Wii U GamePad", "Wii U Pro Controller");
                                        let p2_file = profile_dir.join("controller1.xml");
                                        let _ = std::fs::write(p2_file, p2_content);
                                    }
                                 }
                             }
                         }
                     }
                 }
            }
        } else {
            eprintln!("⚠️ Failed to init SDL2 for controller detection");
        }

        // Fallback: If no gamepads found (gamepad_count == 0), set P1 to Keyboard
        if gamepad_count == 0 {
            if cfg!(target_os = "linux") {
               if let Some(home) = dirs::home_dir() {
                   let profile_dir = home.join(".config/Cemu/controllerProfiles");
                   let _ = std::fs::create_dir_all(&profile_dir);
                   let kbd_template = include_str!("../assets/cemu_controller_keyboard.xml");
                   let p1_file = profile_dir.join("controller0.xml");
                   let _ = std::fs::write(&p1_file, kbd_template);
                   eprintln!("✅ No gamepad found. P1 assigned to Keyboard (Wii U GamePad Mode)");
               }
            }
        }

        Ok(())
    }

    /// Arguments de lancement pour le mode portable.
    /// Cemu requiert: cemu -g <rom_path> [-f]
    fn portable_launch_args(&self, fullscreen: bool) -> (Vec<String>, Vec<String>) {
        // -g doit être AVANT la ROM
        let before = vec!["-g".to_string()];
        // -f (fullscreen) doit être APRÈS la ROM
        let after = if fullscreen { vec!["-f".to_string()] } else { vec![] };
        (before, after)
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(CemuPlugin::new(Some(binary_path)))
    }
}
