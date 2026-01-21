use crate::forge::LaunchConfig;
use crate::plugin::EmulatorPlugin;
use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

/// Configuration PCSX2 par défaut pour éviter le Setup Wizard
const PCSX2_INI_CONTENT: &str = r#"[UI]
SettingsVersion = 1
SetupWizardIncomplete = false
StartFullscreen = true
HideMainWindowWhenRunning = true

[GSWindow]
AspectRatio = Stretch
Zoom = 100.0
IntegerScaling = false
Stretch = true
KeepAspect = false

[EmuCore/GS]
AspectRatio = Stretch
FMVAspectRatioSwitch = Off

[InputSources]
SDL = true
Keyboard = true

[Folders]
Bios = bios

[EmuCore]
EnableFastBoot = true
EnableWideScreenPatches = true

[Pad1]
Type = DualShock2
Deadzone = 0.15
AxisScale = 1.33
ButtonDeadzone = 0.25

Up = Keyboard/Up
Up = SDL-0/DPadUp
Down = Keyboard/Down
Down = SDL-0/DPadDown
Left = Keyboard/Left
Left = SDL-0/DPadLeft
Right = Keyboard/Right
Right = SDL-0/DPadRight

Cross = Keyboard/Z
Cross = SDL-0/A
Circle = Keyboard/X
Circle = SDL-0/B
Square = Keyboard/C
Square = SDL-0/X
Triangle = Keyboard/V
Triangle = SDL-0/Y

L1 = Keyboard/Q
L1 = SDL-0/LeftShoulder
R1 = Keyboard/E
R1 = SDL-0/RightShoulder
L2 = Keyboard/1
L2 = SDL-0/+LeftTrigger
R2 = Keyboard/2
R2 = SDL-0/+RightTrigger
L3 = Keyboard/3
L3 = SDL-0/LeftStick
R3 = Keyboard/4
R3 = SDL-0/RightStick

Start = Keyboard/Return
Start = SDL-0/Start
Select = Keyboard/Backspace
Select = SDL-0/Back

LUp = Keyboard/W
LUp = SDL-0/-LeftY
LDown = Keyboard/S
LDown = SDL-0/+LeftY
LLeft = Keyboard/A
LLeft = SDL-0/-LeftX
LRight = Keyboard/D
LRight = SDL-0/+LeftX

RUp = Keyboard/I
RUp = SDL-0/-RightY
RDown = Keyboard/K
RDown = SDL-0/+RightY
RLeft = Keyboard/J
RLeft = SDL-0/-RightX
RRight = Keyboard/L
RRight = SDL-0/+RightX
"#;

pub struct Pcsx2Plugin {
    pub custom_binary_path: Option<PathBuf>,
}

impl Pcsx2Plugin {
    pub fn new(custom_binary_path: Option<PathBuf>) -> Self {
        Self { custom_binary_path }
    }
}

impl EmulatorPlugin for Pcsx2Plugin {
    fn id(&self) -> &str { "pcsx2" }
    fn name(&self) -> &str { "PCSX2 (PS2 Emulator)" }
    fn supported_extensions(&self) -> &[&str] { &["iso", "cso", "bin", "gz", "chd"] }

    fn find_binary(&self) -> Result<PathBuf> {
        if let Some(path) = &self.custom_binary_path {
            if path.exists() { return Ok(path.clone()); }
        }
        // Try standard paths
        if let Ok(path) = which::which("pcsx2-qt") { return Ok(path); }
        if let Ok(path) = which::which("pcsx2x64") { return Ok(path); }
        if let Ok(path) = which::which("pcsx2") { return Ok(path); }
        
        Err(anyhow::anyhow!("PCSX2 executable not found."))
    }

    fn prepare_launch_config(&self, rom_path: &Path, output_dir: &Path) -> Result<LaunchConfig> {
        let binary = self.find_binary().context("Failed to locate PCSX2 binary")?;
        
        // --- PORTABLE MODE STRATEGY ---
        // PCSX2 AppImage supports -portable flag, which creates all data alongside the binary.
        // We create a dedicated data folder and use -portable to isolate this game's config.
        
        let config_dir = output_dir.join("pcsx2_data");
        std::fs::create_dir_all(&config_dir).context("Failed to create config dir")?;
        
        // Convert to absolute path - critical for XDG_CONFIG_HOME to work
        let config_dir = config_dir.canonicalize().context("Failed to get absolute path for config dir")?;

        // Create bios subfolder - must be inside PCSX2/ to match XDG structure
        // When XDG_CONFIG_HOME=pcsx2_data, PCSX2 looks for bios in pcsx2_data/PCSX2/bios
        let bios_dir = config_dir.join("PCSX2").join("bios");
        std::fs::create_dir_all(&bios_dir).context("Failed to create bios dir")?;

        // Generate minimal ini to skip the First Run Wizard
        // PCSX2 Qt with XDG_CONFIG_HOME looks for config in $XDG_CONFIG_HOME/PCSX2/inis/
        let ini_path = config_dir.join("PCSX2").join("inis").join("PCSX2.ini");
        std::fs::create_dir_all(ini_path.parent().unwrap())?;
        
        // The critical setting is SetupWizardIncomplete = false
        // We also need to point to the bios folder within our config structure
        // Include default keyboard bindings so the game responds to input
        std::fs::write(&ini_path, PCSX2_INI_CONTENT).context("Failed to write PCSX2.ini")?;

        // Note: SDL controller auto-mapping requires the PCSX2 Qt GUI to be used at least once.
        // Users will need to open PCSX2 normally and use "Automatic Mapping" for gamepads.
        // The launcher provides keyboard bindings that work out of the box.

        // Argument order matters: ROM is added first by the stub logic now.
        // -batch: Exits after game closes
        // -nogui: Hides main window
        // -fullscreen: Starts in fullscreen
        let args = vec![
            "-batch".to_string(),
            "-nogui".to_string(),
            "-fullscreen".to_string(),
        ];
        
        // Use environment variables to control config location
        // PCSX2 AppImage respects XDG standards
        let env_vars = vec![
            ("XDG_CONFIG_HOME".to_string(), config_dir.to_string_lossy().to_string()),
        ];
        
        // We store the bios_dir path in a special field so lib.rs can copy the BIOS file there
        // Actually, LaunchConfig doesn't have a bios_dir field. We can use working_dir or just
        // handle it differently. Let me reconsider.
        //
        // Alternative: Return the bios destination path via an environment variable or just
        // have lib.rs reconstruct it (output/pcsx2_data/bios/).
        //
        // Simpler: lib.rs will check if driver_id == "pcsx2", and if so, copy bios to
        // output_dir/pcsx2_data/bios/<filename>.
        //
        // For now, let's just return the config. The copying logic will be in lib.rs.

        Ok(LaunchConfig {
            emulator_path: binary,
            rom_path: rom_path.to_path_buf(),
            args,
            env_vars,
            ..Default::default()
        })
    }

    fn can_handle(&self, binary_path: &Path) -> bool {
        let name = binary_path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
        name.contains("pcsx2")
    }

    fn fullscreen_args(&self) -> Vec<String> {
        // Déjà inclus dans prepare_launch_config (-fullscreen)
        vec![]
    }

    fn setup_environment(&self, output_dir: &Path, bios_path: Option<&Path>) -> Result<()> {
        // PCSX2 utilise PCSX2_USER_PATH, créer la structure complète
        let pcsx2_base = output_dir.join("pcsx2_data").join("PCSX2");
        
        // Créer dossier bios
        let bios_dest_dir = pcsx2_base.join("bios");
        std::fs::create_dir_all(&bios_dest_dir)
            .context("Failed to create BIOS directory")?;
        
        // Copier le BIOS si fourni
        if let Some(bios) = bios_path {
            eprintln!("🔍 BIOS fourni: {:?}", bios);
            eprintln!("🔍 BIOS exists: {}", bios.exists());
            if bios.exists() {
                let bios_filename = bios.file_name()
                    .ok_or_else(|| anyhow::anyhow!("Invalid BIOS path"))?;
                let bios_dest = bios_dest_dir.join(bios_filename);
                eprintln!("📂 Copie vers: {:?}", bios_dest);
                
                std::fs::copy(bios, &bios_dest)
                    .context("Failed to copy BIOS file")?;
                eprintln!("✅ BIOS copié avec succès!");
            } else {
                eprintln!("⚠️ BIOS n'existe pas, copie ignorée");
            }
        } else {
            eprintln!("⚠️ Aucun BIOS fourni");
        }
        
        // Créer dossier inis et le fichier PCSX2.ini
        let inis_dir = pcsx2_base.join("inis");
        std::fs::create_dir_all(&inis_dir)
            .context("Failed to create inis directory")?;
        
        let ini_path = inis_dir.join("PCSX2.ini");
        let ini_content = PCSX2_INI_CONTENT;
        std::fs::write(&ini_path, ini_content)
            .context("Failed to write PCSX2.ini")?;
        
        Ok(())
    }

    fn clone_with_path(&self, binary_path: PathBuf) -> Box<dyn EmulatorPlugin> {
        Box::new(Pcsx2Plugin::new(Some(binary_path)))
    }

    fn portable_env_vars(&self, config_dir: &Path) -> Vec<(String, String)> {
        // PCSX2 AppImage utilise souvent XDG_CONFIG_HOME
        // Structure attendue: $XDG_CONFIG_HOME/PCSX2/inis/PCSX2.ini
        // Notre config_dir est "pcsx2_data", qui contient le dossier "PCSX2".
        vec![
            ("XDG_CONFIG_HOME".to_string(), config_dir.to_string_lossy().to_string()),
            // On garde aussi l'autre variable au cas où, pointant aussi vers la racine de config
            ("PCSX2_USER_PATH".to_string(), config_dir.to_string_lossy().to_string()),
        ]
    }

    fn portable_launch_args(&self, fullscreen: bool) -> (Vec<String>, Vec<String>) {
        // PCSX2 syntax: [flags] [rom] (flags AVANT la ROM)
        let before = if fullscreen { 
            vec!["-fullscreen".to_string()] 
        } else { 
            vec![] 
        };
        (before, vec![])
    }
}
