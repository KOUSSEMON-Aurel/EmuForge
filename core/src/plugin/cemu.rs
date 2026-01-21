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
